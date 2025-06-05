//! NATS Object Store integration for content-addressed storage

use cim_ipld::TypedContent;
use thiserror::Error;

pub mod nats_object_store;
pub mod content_storage;

pub use nats_object_store::NatsObjectStore;
pub use content_storage::ContentStorageService;

/// Object store errors
#[derive(Error, Debug)]
pub enum ObjectStoreError {
    #[error("NATS error: {0}")]
    Nats(#[from] async_nats::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("CID error: {0}")]
    Cid(String),

    #[error("Object not found: {0}")]
    NotFound(String),

    #[error("Bucket error: {0}")]
    Bucket(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ObjectStoreError>;

/// Configuration for object store
#[derive(Debug, Clone)]
pub struct ObjectStoreConfig {
    /// Prefix for all buckets
    pub bucket_prefix: String,
    /// Maximum object size in bytes
    pub max_object_size: usize,
    /// Enable compression
    pub compression: bool,
    /// Retention policy in days (0 = unlimited)
    pub retention_days: u32,
}

impl Default for ObjectStoreConfig {
    fn default() -> Self {
        Self {
            bucket_prefix: "cim.ia".to_string(),
            max_object_size: 10 * 1024 * 1024, // 10MB
            compression: true,
            retention_days: 0, // Unlimited
        }
    }
}

/// Bucket names for different content types
#[derive(Debug, Clone)]
pub struct BucketNames {
    pub events: String,
    pub graphs: String,
    pub nodes: String,
    pub edges: String,
    pub conceptual: String,
    pub workflows: String,
    pub media: String,
    pub documents: String,
}

impl BucketNames {
    pub fn new(prefix: &str) -> Self {
        Self {
            events: format!("{}.events", prefix),
            graphs: format!("{}.graphs", prefix),
            nodes: format!("{}.nodes", prefix),
            edges: format!("{}.edges", prefix),
            conceptual: format!("{}.conceptual", prefix),
            workflows: format!("{}.workflows", prefix),
            media: format!("{}.media", prefix),
            documents: format!("{}.documents", prefix),
        }
    }

    /// Get bucket name for a content type
    pub fn for_content_type<T: TypedContent>(&self) -> Result<&str> {
        // Map content types to buckets based on codec
        match T::CODEC {
            0x300100 => Ok(&self.graphs),      // GraphContent
            0x300101 => Ok(&self.nodes),       // NodeIPLDContent
            0x300102 => Ok(&self.edges),       // EdgeIPLDContent
            0x300103 => Ok(&self.conceptual),  // ConceptualSpaceContent
            0x300104 => Ok(&self.workflows),   // WorkflowContent
            0x300105 => Ok(&self.events),      // EventContent
            0x300106 => Ok(&self.events),      // EventChainMetadata
            _ => Err(ObjectStoreError::Bucket(
                format!("Unknown content type codec: {:#x}", T::CODEC)
            )),
        }
    }
}

/// Metadata stored with each object
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContentMetadata {
    /// Content type codec
    pub codec: u64,
    /// Original size before compression
    pub original_size: usize,
    /// Whether the content is compressed
    pub compressed: bool,
    /// Timestamp when stored
    pub stored_at: std::time::SystemTime,
    /// Optional tags
    pub tags: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket_names() {
        let buckets = BucketNames::new("test");
        assert_eq!(buckets.events, "test.events");
        assert_eq!(buckets.graphs, "test.graphs");
    }

    #[test]
    fn test_default_config() {
        let config = ObjectStoreConfig::default();
        assert_eq!(config.bucket_prefix, "cim.ia");
        assert_eq!(config.max_object_size, 10 * 1024 * 1024);
        assert!(config.compression);
    }
}
