//! Object store infrastructure for CIM-IPLD

mod nats_object_store;
mod content_storage;
mod pull_utils;
mod domain_partitioner;

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
pub use pull_utils::{
    PullOptions,
    PullResult,
    BatchPullResult,
    helpers,
};
pub use domain_partitioner::{
    ContentDomain,
    PartitionStrategy,
    PatternMatcher,
    DomainContentInfo,
    DetectionMethod,
};

// Re-export Result type
pub type Result<T> = std::result::Result<T, ObjectStoreError>;
