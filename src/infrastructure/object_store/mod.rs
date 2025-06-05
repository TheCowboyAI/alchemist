//! Object store infrastructure for CIM-IPLD

mod nats_object_store;
mod content_storage;

pub use nats_object_store::{
    NatsObjectStore,
    ObjectStoreError,
    ContentBucket,
    ObjectInfo,
    BucketStats,
};
pub use content_storage::{
    ContentStorageService,
    CacheStats,
};

// Re-export Result type
pub type Result<T> = std::result::Result<T, ObjectStoreError>;
