//! NATS Object Store wrapper for CIM-IPLD integration

use super::{ObjectStoreConfig, ObjectStoreError, Result, BucketNames, ContentMetadata};
use async_nats::jetstream::{self, object_store::{ObjectStore, Config as ObjectStoreConfigNats}};
use cim_ipld::{TypedContent, Cid};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use futures::StreamExt;

/// Object metadata for NATS
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObjectMeta {
    pub name: String,
    pub description: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

/// NATS Object Store wrapper
pub struct NatsObjectStore {
    /// JetStream context
    jetstream: jetstream::Context,
    /// Configuration
    config: ObjectStoreConfig,
    /// Bucket names
    buckets: BucketNames,
    /// Cached object stores
    stores: Arc<RwLock<HashMap<String, ObjectStore>>>,
}

impl NatsObjectStore {
    /// Create new NATS object store
    pub async fn new(
        jetstream: jetstream::Context,
        config: ObjectStoreConfig,
    ) -> Result<Self> {
        let buckets = BucketNames::new(&config.bucket_prefix);

        Ok(Self {
            jetstream,
            config,
            buckets,
            stores: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Initialize all buckets
    pub async fn initialize_buckets(&self) -> Result<()> {
        info!("Initializing NATS object store buckets");

        let bucket_names = vec![
            &self.buckets.events,
            &self.buckets.graphs,
            &self.buckets.nodes,
            &self.buckets.edges,
            &self.buckets.conceptual,
            &self.buckets.workflows,
            &self.buckets.media,
            &self.buckets.documents,
        ];

        for bucket_name in bucket_names {
            self.ensure_bucket(bucket_name).await?;
            info!("Initialized bucket: {}", bucket_name);
        }

        Ok(())
    }

    /// Ensure a bucket exists
    async fn ensure_bucket(&self, bucket_name: &str) -> Result<ObjectStore> {
        // Check cache first
        {
            let stores = self.stores.read().await;
            if let Some(store) = stores.get(bucket_name) {
                return Ok(store.clone());
            }
        }

        // Create or get bucket
        let config = ObjectStoreConfigNats {
            bucket: bucket_name.to_string(),
            description: Some(format!("CIM-IPLD content bucket: {}", bucket_name)),
            max_bytes: self.config.max_object_size as i64,
            ..Default::default()
        };

        let store = match self.jetstream.get_object_store(&bucket_name).await {
            Ok(store) => {
                debug!("Using existing bucket: {}", bucket_name);
                store
            }
            Err(_) => {
                debug!("Creating new bucket: {}", bucket_name);
                self.jetstream.create_object_store(config).await
                    .map_err(|e| ObjectStoreError::Nats(async_nats::Error::from(e)))?
            }
        };

        // Cache the store
        {
            let mut stores = self.stores.write().await;
            stores.insert(bucket_name.to_string(), store.clone());
        }

        Ok(store)
    }

    /// Store content and return its CID
    pub async fn store<T: TypedContent>(&self, content: &T) -> Result<Cid> {
        let cid = content.to_cid()
            .map_err(|e| ObjectStoreError::Cid(e.to_string()))?;
        let bucket_name = self.buckets.for_content_type::<T>()?;

        // Check if already exists
        if self.exists(&cid).await? {
            debug!("Content already exists: {}", cid);
            return Ok(cid);
        }

        // Serialize content
        let data = serde_json::to_vec(content)?;
        let original_size = data.len();

        // Optionally compress
        let (data, compressed) = if self.config.compression && original_size > 1024 {
            let compressed = compress_data(&data)?;
            if compressed.len() < original_size {
                (compressed, true)
            } else {
                (data, false)
            }
        } else {
            (data, false)
        };

        // Create metadata
        let metadata = ContentMetadata {
            codec: T::CODEC,
            original_size,
            compressed,
            stored_at: std::time::SystemTime::now(),
            tags: vec![],
        };

        // Store in NATS
        let store = self.ensure_bucket(bucket_name).await?;

        // Store with metadata in description
        let mut object_meta = async_nats::jetstream::object_store::ObjectMeta {
            name: cid.to_string(),
            description: Some(serde_json::to_string(&metadata)?),
            ..Default::default()
        };

        // Put the object
        store.put(object_meta, &mut data.as_slice()).await
            .map_err(|e| ObjectStoreError::Nats(async_nats::Error::from(e)))?;

        info!("Stored content: {} in bucket: {}", cid, bucket_name);
        Ok(cid)
    }

    /// Retrieve content by CID
    pub async fn get<T: TypedContent>(&self, cid: &Cid) -> Result<T> {
        let bucket_name = self.buckets.for_content_type::<T>()?;
        let store = self.ensure_bucket(bucket_name).await?;

        // Get object
        let object = store.get(&cid.to_string()).await
            .map_err(|_| ObjectStoreError::NotFound(cid.to_string()))?;

        // Get info first
        let info = object.info;
        let metadata: ContentMetadata = if let Some(desc) = &info.description {
            serde_json::from_str(desc)?
        } else {
            return Err(ObjectStoreError::Cid("Missing metadata".to_string()));
        };

        // Read data from stream
        let mut data = Vec::new();
        let mut stream = object.stream;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| ObjectStoreError::Nats(e))?;
            data.extend_from_slice(&chunk);
        }

        // Decompress if needed
        let data = if metadata.compressed {
            decompress_data(&data)?
        } else {
            data
        };

        // Deserialize
        let content: T = serde_json::from_slice(&data)?;

        // Verify CID
        let computed_cid = content.to_cid()
            .map_err(|e| ObjectStoreError::Cid(e.to_string()))?;
        if &computed_cid != cid {
            return Err(ObjectStoreError::Cid(
                format!("CID mismatch: expected {}, got {}", cid, computed_cid)
            ));
        }

        Ok(content)
    }

    /// Check if content exists
    pub async fn exists(&self, cid: &Cid) -> Result<bool> {
        // Check all buckets since we don't know the content type
        for bucket_name in [
            &self.buckets.events,
            &self.buckets.graphs,
            &self.buckets.nodes,
            &self.buckets.edges,
            &self.buckets.conceptual,
            &self.buckets.workflows,
            &self.buckets.media,
            &self.buckets.documents,
        ] {
            if let Ok(store) = self.ensure_bucket(bucket_name).await {
                if store.info(&cid.to_string()).await.is_ok() {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Delete content by CID
    pub async fn delete(&self, cid: &Cid) -> Result<()> {
        // Try to delete from all buckets
        let mut deleted = false;

        for bucket_name in [
            &self.buckets.events,
            &self.buckets.graphs,
            &self.buckets.nodes,
            &self.buckets.edges,
            &self.buckets.conceptual,
            &self.buckets.workflows,
            &self.buckets.media,
            &self.buckets.documents,
        ] {
            if let Ok(store) = self.ensure_bucket(bucket_name).await {
                if store.delete(&cid.to_string()).await.is_ok() {
                    info!("Deleted content: {} from bucket: {}", cid, bucket_name);
                    deleted = true;
                }
            }
        }

        if !deleted {
            return Err(ObjectStoreError::NotFound(cid.to_string()));
        }

        Ok(())
    }

    /// List all CIDs in a bucket
    pub async fn list_bucket(&self, bucket_name: &str) -> Result<Vec<Cid>> {
        let store = self.ensure_bucket(bucket_name).await?;
        let mut list = store.list().await;

        let mut cids = Vec::new();
        while let Some(info) = list.next().await {
            let info = info.map_err(|e| ObjectStoreError::Nats(e))?;
            if let Ok(cid) = info.name.parse::<Cid>() {
                cids.push(cid);
            }
        }

        Ok(cids)
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> Result<HashMap<String, (usize, usize)>> {
        let mut stats = HashMap::new();

        for (name, bucket_name) in [
            ("events", &self.buckets.events),
            ("graphs", &self.buckets.graphs),
            ("nodes", &self.buckets.nodes),
            ("edges", &self.buckets.edges),
            ("conceptual", &self.buckets.conceptual),
            ("workflows", &self.buckets.workflows),
            ("media", &self.buckets.media),
            ("documents", &self.buckets.documents),
        ] {
            if let Ok(store) = self.ensure_bucket(bucket_name).await {
                let mut list = store.list().await;

                let mut count = 0;
                let mut size = 0;

                while let Some(info) = list.next().await {
                    if let Ok(info) = info {
                        count += 1;
                        size += info.size as usize;
                    }
                }

                stats.insert(name.to_string(), (count, size));
            }
        }

        Ok(stats)
    }
}

/// Compress data using zstd
fn compress_data(data: &[u8]) -> Result<Vec<u8>> {
    use std::io::Write;
    let mut encoder = zstd::Encoder::new(Vec::new(), 3)?;
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

/// Decompress data using zstd
fn decompress_data(data: &[u8]) -> Result<Vec<u8>> {
    Ok(zstd::decode_all(data)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compress_decompress() {
        let data = b"Hello, World! This is a test of compression.";
        let compressed = compress_data(data).unwrap();
        let decompressed = decompress_data(&compressed).unwrap();
        assert_eq!(data.to_vec(), decompressed);
    }
}
