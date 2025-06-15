//! Object store infrastructure for CIM-IPLD

mod content_storage;
mod nats_object_store;

pub use content_storage::{CacheStats, ContentStorageService};
pub use nats_object_store::{
    BucketStats, ContentBucket, NatsObjectStore, ObjectInfo, ObjectStoreError,
};

// Re-export Result type
pub type Result<T> = std::result::Result<T, ObjectStoreError>;
