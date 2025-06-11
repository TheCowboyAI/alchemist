//! NATS Object Store wrapper for CIM-IPLD integration

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use async_nats::jetstream::{self, object_store::ObjectStore};
use cid::Cid;
use cim_ipld::TypedContent;
use futures::StreamExt;
use tokio::io::AsyncReadExt;
use tokio::sync::RwLock;
use zstd::stream::{decode_all, encode_all};

/// Error types for object store operations
#[derive(Debug, thiserror::Error)]
pub enum ObjectStoreError {
    #[error("NATS error: {0}")]
    Nats(#[from] async_nats::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Object not found: {0}")]
    NotFound(String),

    #[error("Bucket not found: {0}")]
    BucketNotFound(String),

    #[error("Bucket creation failed: {0}")]
    BucketCreation(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("CID mismatch: expected {expected}, got {actual}")]
    CidMismatch { expected: String, actual: String },
}

pub type Result<T> = std::result::Result<T, ObjectStoreError>;

/// Content bucket types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentBucket {
    Events,
    Graphs,
    Nodes,
    Edges,
    ConceptualSpaces,
    Workflows,
    Media,
    Documents,
}

impl ContentBucket {
    /// Get all bucket types
    pub fn all() -> Vec<Self> {
        vec![
            Self::Events,
            Self::Graphs,
            Self::Nodes,
            Self::Edges,
            Self::ConceptualSpaces,
            Self::Workflows,
            Self::Media,
            Self::Documents,
        ]
    }

    /// Get bucket name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Events => "cim-events",
            Self::Graphs => "cim-graphs",
            Self::Nodes => "cim-nodes",
            Self::Edges => "cim-edges",
            Self::ConceptualSpaces => "cim-conceptual",
            Self::Workflows => "cim-workflows",
            Self::Media => "cim-media",
            Self::Documents => "cim-documents",
        }
    }

    /// Get bucket for content type
    pub fn for_content_type(content_type: u64) -> Self {
        match content_type {
            0x300100 => Self::Graphs,
            0x300101 => Self::Nodes,
            0x300102 => Self::Edges,
            0x300103 => Self::ConceptualSpaces,
            0x300104 => Self::Workflows,
            0x300105 => Self::Events,
            0x300106 => Self::Events, // EventChainMetadata
            _ => Self::Documents, // Default
        }
    }
}

/// Object information
#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub cid: Cid,
    pub size: usize,
    pub created_at: SystemTime,
    pub compressed: bool,
}

/// Bucket statistics
#[derive(Debug, Clone)]
pub struct BucketStats {
    pub bucket_name: String,
    pub object_count: usize,
    pub total_size: u64,
    pub compressed_objects: usize,
}

/// Wrapper around NATS Object Store for content-addressed storage
pub struct NatsObjectStore {
    jetstream: jetstream::Context,
    buckets: Arc<RwLock<HashMap<ContentBucket, ObjectStore>>>,
    compression_threshold: usize,
}

impl NatsObjectStore {
    /// Create a new NATS Object Store wrapper
    pub async fn new(
        jetstream: jetstream::Context,
        compression_threshold: usize,
    ) -> Result<Self> {
        let store = Self {
            jetstream,
            buckets: Arc::new(RwLock::new(HashMap::new())),
            compression_threshold,
        };

        // Initialize all buckets
        for bucket in ContentBucket::all() {
            store.ensure_bucket(bucket).await?;
        }

        Ok(store)
    }

    /// Ensure a bucket exists, creating it if necessary
    async fn ensure_bucket(&self, bucket: ContentBucket) -> Result<()> {
        let bucket_name = bucket.as_str();

        // Try to get existing bucket
        match self.jetstream.get_object_store(bucket_name).await {
            Ok(object_store) => {
                let mut buckets = self.buckets.write().await;
                buckets.insert(bucket, object_store);
                Ok(())
            }
            Err(_) => {
                // Create new bucket
                let config = jetstream::object_store::Config {
                    bucket: bucket_name.to_string(),
                    description: Some(format!("CIM content bucket for {bucket_name}")),
                    max_age: Duration::from_secs(365 * 24 * 60 * 60), // 365 days
                    ..Default::default()
                };

                let object_store = self.jetstream.create_object_store(config).await
                    .map_err(|e| ObjectStoreError::BucketCreation(e.to_string()))?;

                let mut buckets = self.buckets.write().await;
                buckets.insert(bucket, object_store);
                Ok(())
            }
        }
    }

    /// Get the object store for a specific bucket
    async fn get_bucket(&self, bucket: ContentBucket) -> Result<ObjectStore> {
        let buckets = self.buckets.read().await;
        buckets.get(&bucket)
            .cloned()
            .ok_or_else(|| ObjectStoreError::BucketNotFound(bucket.as_str().to_string()))
    }

    /// Store content by its CID
    pub async fn put<T: TypedContent>(&self, content: &T) -> Result<Cid> {
        let bucket = ContentBucket::for_content_type(T::CONTENT_TYPE.codec());
        let object_store = self.get_bucket(bucket).await?;

        // Calculate CID
        let cid = content.calculate_cid()
            .map_err(|e| ObjectStoreError::Serialization(e.to_string()))?;

        // Serialize content
        let data = content.to_bytes()
            .map_err(|e| ObjectStoreError::Serialization(e.to_string()))?;

        // Compress if over threshold
        let (data, _compressed) = if data.len() > self.compression_threshold {
            let compressed = encode_all(&data[..], 3)
                .map_err(|e| ObjectStoreError::Compression(e.to_string()))?;
            (compressed, true)
        } else {
            (data, false)
        };

        // Store in NATS
        let key = cid.to_string();

        // Put the object - use key.as_str() to get &str
        object_store.put(key.as_str(), &mut data.as_slice()).await
            .map_err(|e| ObjectStoreError::Storage(e.to_string()))?;

        Ok(cid)
    }

    /// Retrieve content by CID
    pub async fn get<T: TypedContent>(&self, cid: &Cid) -> Result<T> {
        let bucket = ContentBucket::for_content_type(T::CONTENT_TYPE.codec());
        let object_store = self.get_bucket(bucket).await?;

        let key = cid.to_string();

        // Get the object
        let mut object = object_store.get(&key).await
            .map_err(|_| ObjectStoreError::NotFound(key.clone()))?;

        // Read all data from the stream
        let mut data = Vec::new();
        object.read_to_end(&mut data).await
            .map_err(|e| ObjectStoreError::Storage(e.to_string()))?;

        // For now, assume compressed if data looks compressed (starts with zstd magic)
        let compressed = data.len() >= 4 && data[0..4] == [0x28, 0xb5, 0x2f, 0xfd];

        // Decompress if needed
        let data = if compressed {
            decode_all(&data[..])
                .map_err(|e| ObjectStoreError::Compression(e.to_string()))?
        } else {
            data
        };

        // Deserialize and verify CID
        let content = T::from_bytes(&data)
            .map_err(|e| ObjectStoreError::Deserialization(e.to_string()))?;

        let computed_cid = content.calculate_cid()
            .map_err(|e| ObjectStoreError::Serialization(e.to_string()))?;

        if computed_cid != *cid {
            return Err(ObjectStoreError::CidMismatch {
                expected: cid.to_string(),
                actual: computed_cid.to_string(),
            });
        }

        Ok(content)
    }

    /// Check if content exists
    pub async fn exists(&self, cid: &Cid, content_type: u64) -> Result<bool> {
        let bucket = ContentBucket::for_content_type(content_type);
        let object_store = self.get_bucket(bucket).await?;

        let key = cid.to_string();
        match object_store.info(&key).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Delete content by CID
    pub async fn delete(&self, cid: &Cid, content_type: u64) -> Result<()> {
        let bucket = ContentBucket::for_content_type(content_type);
        let object_store = self.get_bucket(bucket).await?;

        let key = cid.to_string();
        object_store.delete(&key).await
            .map_err(|e| ObjectStoreError::Storage(e.to_string()))?;

        Ok(())
    }

    /// List all objects in a bucket
    pub async fn list(&self, bucket: ContentBucket) -> Result<Vec<ObjectInfo>> {
        let object_store = self.get_bucket(bucket).await?;

        let mut list = object_store.list().await
            .map_err(|e| ObjectStoreError::Storage(e.to_string()))?;

        let mut objects = Vec::new();
        while let Some(info) = list.next().await {
            let info = info.map_err(|e| ObjectStoreError::Storage(e.to_string()))?;

            if let Ok(cid) = Cid::try_from(info.name.as_str()) {
                objects.push(ObjectInfo {
                    cid,
                    size: info.size,
                    created_at: SystemTime::now(), // NATS doesn't provide mtime in the API
                    compressed: info.headers
                        .as_ref()
                        .and_then(|h| h.get("Compressed"))
                        .and_then(|v| v.as_str().parse::<bool>().ok())
                        .unwrap_or(false),
                });
            }
        }

        Ok(objects)
    }

    /// Get bucket statistics
    pub async fn stats(&self, bucket: ContentBucket) -> Result<BucketStats> {
        let bucket_name = bucket.as_str();

        // For now, return basic stats
        // The async-nats 0.41 API doesn't expose detailed bucket stats easily
        Ok(BucketStats {
            bucket_name: bucket_name.to_string(),
            object_count: 0,
            total_size: 0,
            compressed_objects: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bucket_names() {
        assert_eq!(ContentBucket::Events.as_str(), "cim-events");
        assert_eq!(ContentBucket::Graphs.as_str(), "cim-graphs");
        assert_eq!(ContentBucket::for_content_type(0x300100), ContentBucket::Graphs);
    }
}
