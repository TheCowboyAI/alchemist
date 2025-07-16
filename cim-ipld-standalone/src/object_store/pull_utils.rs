//! Utility functions for pulling content from NATS JetStream by CID

use super::{NatsObjectStore, ContentBucket, ObjectInfo, ObjectStoreError, Result};
use crate::{TypedContent, Cid};
use futures::stream::{self, StreamExt};
use std::collections::HashMap;

/// Options for pulling content from JetStream
#[derive(Debug, Clone, Default)]
pub struct PullOptions {
    /// Maximum number of items to pull
    pub limit: Option<usize>,
    /// Filter by minimum size
    pub min_size: Option<usize>,
    /// Filter by maximum size
    pub max_size: Option<usize>,
    /// Only pull compressed objects
    pub compressed_only: bool,
}

/// Result of a pull operation
#[derive(Debug, Clone)]
pub struct PullResult<T> {
    pub cid: Cid,
    pub content: T,
    pub metadata: ObjectInfo,
}

/// Batch pull result
#[derive(Debug)]
pub struct BatchPullResult<T> {
    pub successful: Vec<PullResult<T>>,
    pub failed: Vec<(Cid, ObjectStoreError)>,
}

impl NatsObjectStore {
    /// Pull all objects of a specific type from a bucket
    pub async fn pull_all<T: TypedContent>(
        &self,
        bucket: ContentBucket,
        options: PullOptions,
    ) -> Result<Vec<PullResult<T>>> {
        // List objects in bucket
        let mut objects = self.list(bucket).await?;

        // Apply filters
        if let Some(min_size) = options.min_size {
            objects.retain(|obj| obj.size >= min_size);
        }
        if let Some(max_size) = options.max_size {
            objects.retain(|obj| obj.size <= max_size);
        }
        if options.compressed_only {
            objects.retain(|obj| obj.compressed);
        }

        // Apply limit
        if let Some(limit) = options.limit {
            objects.truncate(limit);
        }

        // Pull each object
        let mut results = Vec::new();
        for obj in objects {
            match self.get::<T>(&obj.cid).await {
                Ok(content) => {
                    results.push(PullResult {
                        cid: obj.cid,
                        content,
                        metadata: obj,
                    });
                }
                Err(e) => {
                    // Log error but continue with other objects
                    eprintln!("Failed to pull CID {}: {}", obj.cid, e);
                }
            }
        }

        Ok(results)
    }

    /// Pull multiple objects by CID in parallel
    pub async fn pull_batch<T: TypedContent>(
        &self,
        cids: &[Cid],
        max_concurrent: usize,
    ) -> BatchPullResult<T> {
        let futures = cids.iter().map(|cid| {
            let cid = *cid;
            async move {
                match self.get::<T>(&cid).await {
                    Ok(content) => {
                        // Get metadata if available
                        let bucket = ContentBucket::for_content_type(T::CONTENT_TYPE.codec());
                        let metadata = self.list(bucket).await
                            .ok()
                            .and_then(|objects| {
                                objects.into_iter()
                                    .find(|obj| obj.cid == cid)
                            });

                        Ok(PullResult {
                            cid,
                            content,
                            metadata: metadata.unwrap_or_else(|| ObjectInfo {
                                cid,
                                size: 0,
                                created_at: std::time::SystemTime::now(),
                                compressed: false,
                            }),
                        })
                    }
                    Err(e) => Err((cid, e)),
                }
            }
        });

        // Process in parallel with concurrency limit
        let results: Vec<_> = stream::iter(futures)
            .buffer_unordered(max_concurrent)
            .collect()
            .await;

        // Separate successful and failed results
        let mut successful = Vec::new();
        let mut failed = Vec::new();

        for result in results {
            match result {
                Ok(pull_result) => successful.push(pull_result),
                Err((cid, error)) => failed.push((cid, error)),
            }
        }

        BatchPullResult { successful, failed }
    }

    /// Pull the latest N objects from a bucket
    pub async fn pull_latest<T: TypedContent>(
        &self,
        bucket: ContentBucket,
        count: usize,
    ) -> Result<Vec<PullResult<T>>> {
        let options = PullOptions {
            limit: Some(count),
            ..Default::default()
        };

        self.pull_all(bucket, options).await
    }

    /// Pull objects by CID prefix (useful for searching)
    pub async fn pull_by_prefix<T: TypedContent>(
        &self,
        bucket: ContentBucket,
        prefix: &str,
    ) -> Result<Vec<PullResult<T>>> {
        let objects = self.list(bucket).await?;

        let matching_cids: Vec<_> = objects
            .into_iter()
            .filter(|obj| obj.cid.to_string().starts_with(prefix))
            .map(|obj| obj.cid)
            .collect();

        let batch_result = self.pull_batch::<T>(&matching_cids, 10).await;

        if !batch_result.failed.is_empty() {
            eprintln!("Warning: {} objects failed to pull", batch_result.failed.len());
        }

        Ok(batch_result.successful)
    }

    /// Pull and group objects by a key function
    pub async fn pull_and_group<T, K, F>(
        &self,
        bucket: ContentBucket,
        key_fn: F,
    ) -> Result<HashMap<K, Vec<PullResult<T>>>>
    where
        T: TypedContent,
        K: Eq + std::hash::Hash,
        F: Fn(&T) -> K,
    {
        let all_objects = self.pull_all::<T>(bucket, PullOptions::default()).await?;

        let mut grouped = HashMap::new();
        for result in all_objects {
            let key = key_fn(&result.content);
            grouped.entry(key).or_insert_with(Vec::new).push(result);
        }

        Ok(grouped)
    }

    /// Stream objects from a bucket
    pub fn stream_objects<T: TypedContent>(
        &self,
        bucket: ContentBucket,
    ) -> impl futures::Stream<Item = Result<PullResult<T>>> + '_ {
        stream::unfold(Some(0usize), move |state| async move {
            let offset = state?;

            // Get next batch of objects
            let objects = match self.list(bucket).await {
                Ok(objs) => objs,
                Err(e) => return Some((Err(e), None)),
            };

            // Check if we've processed all objects
            if offset >= objects.len() {
                return None;
            }

            // Get next object
            let obj = &objects[offset];
            
            match self.get::<T>(&obj.cid).await {
                Ok(content) => {
                    let result = Ok(PullResult {
                        cid: obj.cid,
                        content,
                        metadata: obj.clone(),
                    });
                    Some((result, Some(offset + 1)))
                }
                Err(e) => Some((Err(e), Some(offset + 1))),
            }
        })
    }
}

/// Helper functions for working with pulled content
pub mod helpers {
    use super::*;

    /// Filter pull results by content predicate
    pub fn filter_by_content<T, F>(
        results: Vec<PullResult<T>>,
        predicate: F,
    ) -> Vec<PullResult<T>>
    where
        F: Fn(&T) -> bool,
    {
        results
            .into_iter()
            .filter(|result| predicate(&result.content))
            .collect()
    }

    /// Sort pull results by a key function
    pub fn sort_by_key<T, K, F>(
        mut results: Vec<PullResult<T>>,
        key_fn: F,
    ) -> Vec<PullResult<T>>
    where
        K: Ord,
        F: Fn(&T) -> K,
    {
        results.sort_by_key(|result| key_fn(&result.content));
        results
    }

    /// Extract just the content from pull results
    pub fn extract_content<T>(results: Vec<PullResult<T>>) -> Vec<T> {
        results.into_iter().map(|result| result.content).collect()
    }

    /// Create a CID to content mapping
    pub fn to_cid_map<T>(results: Vec<PullResult<T>>) -> HashMap<Cid, T> {
        results
            .into_iter()
            .map(|result| (result.cid, result.content))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pull_options_default() {
        let options = PullOptions::default();
        assert!(options.limit.is_none());
        assert!(options.min_size.is_none());
        assert!(options.max_size.is_none());
        assert!(!options.compressed_only);
    }

    #[test]
    fn test_pull_options_builder() {
        let options = PullOptions {
            limit: Some(10),
            min_size: Some(1024),
            max_size: Some(1024 * 1024),
            compressed_only: true,
        };

        assert_eq!(options.limit, Some(10));
        assert_eq!(options.min_size, Some(1024));
        assert_eq!(options.max_size, Some(1024 * 1024));
        assert!(options.compressed_only);
    }
} 