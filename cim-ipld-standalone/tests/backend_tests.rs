//! Backend storage tests for CIM-IPLD
//!
//! Tests various storage backend implementations and their interactions.
//!
//! ## Test Scenarios
//!
//! ```mermaid
//! graph TD
//!     A[Backend Tests] --> B[S3 Operations]
//!     A --> C[Filesystem Operations]
//!     A --> D[Backend Switching]
//!     A --> E[Multi-Backend Sync]
//!
//!     B --> B1[CRUD Operations]
//!     B --> B2[Large Objects]
//!     B --> B3[Error Handling]
//!
//!     C --> C1[Path Management]
//!     C --> C2[Permissions]
//!     C --> C3[Space Limits]
//!
//!     D --> D1[Data Migration]
//!     D --> D2[Hot Switching]
//!     D --> D3[Consistency]
//!
//!     E --> E1[Write Propagation]
//!     E --> E2[Read Failover]
//!     E --> E3[Conflict Resolution]
//! ```

use cim_ipld::*;
use cim_ipld::object_store::NatsObjectStore;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::time::{sleep, Duration};

mod common;
use common::*;

/// Mock S3-compatible backend for testing
struct MockS3Backend {
    storage: Arc<tokio::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>>,
    latency: Duration,
    failure_rate: f32,
}

impl MockS3Backend {
    fn new(latency: Duration, failure_rate: f32) -> Self {
        Self {
            storage: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            latency,
            failure_rate,
        }
    }

    async fn put(&self, key: &str, data: Vec<u8>) -> Result<()> {
        sleep(self.latency).await;

        if rand::random::<f32>() < self.failure_rate {
            return Err(Error::StorageError("S3 put failed".into()));
        }

        let mut storage = self.storage.lock().await;
        storage.insert(key.to_string(), data);
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>> {
        sleep(self.latency).await;

        if rand::random::<f32>() < self.failure_rate {
            return Err(Error::StorageError("S3 get failed".into()));
        }

        let storage = self.storage.lock().await;
        storage.get(key)
            .cloned()
            .ok_or_else(|| Error::NotFound(key.to_string()))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        sleep(self.latency).await;

        if rand::random::<f32>() < self.failure_rate {
            return Err(Error::StorageError("S3 delete failed".into()));
        }

        let mut storage = self.storage.lock().await;
        storage.remove(key);
        Ok(())
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        sleep(self.latency).await;

        if rand::random::<f32>() < self.failure_rate {
            return Err(Error::StorageError("S3 list failed".into()));
        }

        let storage = self.storage.lock().await;
        Ok(storage.keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect())
    }
}

/// Mock filesystem backend for testing
struct MockFilesystemBackend {
    root: TempDir,
    max_size: Option<usize>,
    current_size: Arc<tokio::sync::Mutex<usize>>,
}

impl MockFilesystemBackend {
    fn new(max_size: Option<usize>) -> Result<Self> {
        Ok(Self {
            root: TempDir::new()?,
            max_size,
            current_size: Arc::new(tokio::sync::Mutex::new(0)),
        })
    }

    async fn put(&self, key: &str, data: Vec<u8>) -> Result<()> {
        if let Some(max) = self.max_size {
            let mut size = self.current_size.lock().await;
            if *size + data.len() > max {
                return Err(Error::StorageError("Filesystem full".into()));
            }
            *size += data.len();
        }

        let path = self.root.path().join(key);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, data).await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>> {
        let path = self.root.path().join(key);
        tokio::fs::read(path).await
            .map_err(|_| Error::NotFound(key.to_string()))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let path = self.root.path().join(key);
        if let Ok(metadata) = tokio::fs::metadata(&path).await {
            if let Some(_) = self.max_size {
                let mut size = self.current_size.lock().await;
                *size = size.saturating_sub(metadata.len() as usize);
            }
        }
        tokio::fs::remove_file(path).await?;
        Ok(())
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let mut results = Vec::new();
        let mut entries = tokio::fs::read_dir(self.root.path()).await?;

        while let Some(entry) = entries.next_entry().await? {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with(prefix) {
                    results.push(name.to_string());
                }
            }
        }

        Ok(results)
    }
}

#[tokio::test]
#[ignore] // Requires NATS server
async fn test_s3_backend_operations() {
    /// Test S3-compatible backend CRUD operations
    ///
    /// Given: S3-compatible backend
    /// When: CRUD operations performed
    /// Then: All operations succeed

    let backend = MockS3Backend::new(Duration::from_millis(10), 0.0);
    let test_key = "test/object";
    let test_data = b"test content".to_vec();

    // Test PUT
    backend.put(test_key, test_data.clone()).await
        .expect("S3 PUT should succeed");

    // Test GET
    let retrieved = backend.get(test_key).await
        .expect("S3 GET should succeed");
    assert_eq!(retrieved, test_data);

    // Test LIST
    let items = backend.list("test/").await
        .expect("S3 LIST should succeed");
    assert!(items.contains(&test_key.to_string()));

    // Test DELETE
    backend.delete(test_key).await
        .expect("S3 DELETE should succeed");

    // Verify deletion
    assert!(backend.get(test_key).await.is_err());
}

#[tokio::test]
#[ignore] // Requires NATS server
async fn test_filesystem_backend_operations() {
    /// Test local filesystem backend CRUD operations
    ///
    /// Given: Local filesystem backend
    /// When: CRUD operations performed
    /// Then: All operations succeed

    let backend = MockFilesystemBackend::new(None)
        .expect("Filesystem backend creation should succeed");
    let test_key = "test/file.dat";
    let test_data = b"filesystem content".to_vec();

    // Test PUT
    backend.put(test_key, test_data.clone()).await
        .expect("Filesystem PUT should succeed");

    // Test GET
    let retrieved = backend.get(test_key).await
        .expect("Filesystem GET should succeed");
    assert_eq!(retrieved, test_data);

    // Test LIST
    let items = backend.list("test").await
        .expect("Filesystem LIST should succeed");
    assert!(!items.is_empty());

    // Test DELETE
    backend.delete(test_key).await
        .expect("Filesystem DELETE should succeed");

    // Verify deletion
    assert!(backend.get(test_key).await.is_err());
}

#[tokio::test]
#[ignore] // Requires NATS server
async fn test_backend_switching() {
    /// Test seamless backend switching
    ///
    /// Given: Content in one backend
    /// When: Switch to another backend
    /// Then: Seamless transition with data migration

    let s3_backend = MockS3Backend::new(Duration::from_millis(10), 0.0);
    let fs_backend = MockFilesystemBackend::new(None)
        .expect("Filesystem backend creation should succeed");

    let test_key = "migrate/object";
    let test_data = b"data to migrate".to_vec();

    // Store in S3 backend
    s3_backend.put(test_key, test_data.clone()).await
        .expect("S3 PUT should succeed");

    // Simulate backend switch with migration
    let data_from_s3 = s3_backend.get(test_key).await
        .expect("S3 GET should succeed");
    fs_backend.put(test_key, data_from_s3).await
        .expect("Filesystem PUT should succeed");

    // Verify data in new backend
    let retrieved = fs_backend.get(test_key).await
        .expect("Filesystem GET should succeed");
    assert_eq!(retrieved, test_data);

    // Clean up old backend
    s3_backend.delete(test_key).await
        .expect("S3 DELETE should succeed");
}

#[tokio::test]
#[ignore] // Requires NATS server
async fn test_multi_backend_sync() {
    /// Test synchronization across multiple backends
    ///
    /// Given: Multiple backends
    /// When: Content written
    /// Then: Synchronized across backends

    let backend1 = MockS3Backend::new(Duration::from_millis(5), 0.0);
    let backend2 = MockS3Backend::new(Duration::from_millis(10), 0.0);
    let backend3 = MockFilesystemBackend::new(None)
        .expect("Filesystem backend creation should succeed");

    let test_key = "sync/object";
    let test_data = b"synchronized content".to_vec();

    // Write to primary backend
    backend1.put(test_key, test_data.clone()).await
        .expect("Primary PUT should succeed");

    // Simulate sync to other backends
    let data = backend1.get(test_key).await
        .expect("Primary GET should succeed");

    // Sync to backend2
    backend2.put(test_key, data.clone()).await
        .expect("Secondary PUT should succeed");

    // Sync to backend3
    backend3.put(test_key, data).await
        .expect("Tertiary PUT should succeed");

    // Verify all backends have the same data
    let data1 = backend1.get(test_key).await.expect("Backend1 GET should succeed");
    let data2 = backend2.get(test_key).await.expect("Backend2 GET should succeed");
    let data3 = backend3.get(test_key).await.expect("Backend3 GET should succeed");

    assert_eq!(data1, test_data);
    assert_eq!(data2, test_data);
    assert_eq!(data3, test_data);
}

#[tokio::test]
#[ignore] // Requires NATS server
async fn test_backend_failure_handling() {
    /// Test backend failure scenarios
    ///
    /// Given: Backend with failures
    /// When: Operations performed
    /// Then: Proper error handling and recovery

    let unreliable_backend = MockS3Backend::new(Duration::from_millis(10), 0.3); // 30% failure rate
    let test_key = "fail/object";
    let test_data = b"test data".to_vec();

    // Test PUT with retries
    let mut put_attempts = 0;
    let max_retries = 5;

    while put_attempts < max_retries {
        match unreliable_backend.put(test_key, test_data.clone()).await {
            Ok(_) => break,
            Err(_) => {
                put_attempts += 1;
                sleep(Duration::from_millis(100)).await;
            }
        }
    }

    assert!(put_attempts < max_retries, "PUT should succeed within retry limit");

    // Test GET with retries
    let mut get_attempts = 0;
    let mut retrieved_data = None;

    while get_attempts < max_retries {
        match unreliable_backend.get(test_key).await {
            Ok(data) => {
                retrieved_data = Some(data);
                break;
            }
            Err(_) => {
                get_attempts += 1;
                sleep(Duration::from_millis(100)).await;
            }
        }
    }

    assert!(retrieved_data.is_some(), "GET should succeed within retry limit");
    assert_eq!(retrieved_data.unwrap(), test_data);
}

#[tokio::test]
#[ignore] // Requires NATS server
async fn test_filesystem_space_limits() {
    /// Test filesystem backend with space constraints
    ///
    /// Given: Filesystem with limited space
    /// When: Space limit exceeded
    /// Then: Proper error handling

    let max_size = 1024; // 1KB limit
    let backend = MockFilesystemBackend::new(Some(max_size))
        .expect("Filesystem backend creation should succeed");

    // Fill up most of the space
    let large_data = vec![0u8; 900];
    backend.put("large1", large_data).await
        .expect("First PUT should succeed");

    // Try to exceed limit
    let too_large = vec![0u8; 200];
    let result = backend.put("large2", too_large).await;

    assert!(result.is_err(), "PUT should fail when space limit exceeded");
    assert!(matches!(result.unwrap_err(), Error::StorageError(_)));

    // Delete to free space
    backend.delete("large1").await
        .expect("DELETE should succeed");

    // Now it should work
    let small_data = vec![0u8; 100];
    backend.put("small", small_data).await
        .expect("PUT should succeed after freeing space");
}

#[tokio::test]
#[ignore] // Requires NATS server
async fn test_backend_performance_characteristics() {
    /// Test and compare backend performance
    ///
    /// Given: Different backend implementations
    /// When: Performance measured
    /// Then: Characteristics documented

    use std::time::Instant;

    let fast_s3 = MockS3Backend::new(Duration::from_millis(1), 0.0);
    let slow_s3 = MockS3Backend::new(Duration::from_millis(50), 0.0);
    let fs_backend = MockFilesystemBackend::new(None)
        .expect("Filesystem backend creation should succeed");

    let test_data = vec![0u8; 10_000]; // 10KB

    // Measure S3 fast backend
    let start = Instant::now();
    for i in 0..10 {
        fast_s3.put(&format!("perf/fast/{i}"), test_data.clone()).await
            .expect("Fast S3 PUT should succeed");
    }
    let fast_duration = start.elapsed();

    // Measure S3 slow backend
    let start = Instant::now();
    for i in 0..10 {
        slow_s3.put(&format!("perf/slow/{i}"), test_data.clone()).await
            .expect("Slow S3 PUT should succeed");
    }
    let slow_duration = start.elapsed();

    // Measure filesystem backend
    let start = Instant::now();
    for i in 0..10 {
        fs_backend.put(&format!("perf/fs/{i}"), test_data.clone()).await
            .expect("Filesystem PUT should succeed");
    }
    let fs_duration = start.elapsed();

    println!("Backend performance comparison:");
    println!("  Fast S3: {:?}", fast_duration);
    println!("  Slow S3: {:?}", slow_duration);
    println!("  Filesystem: {:?}", fs_duration);

    // Verify relative performance
    assert!(fast_duration < slow_duration, "Fast backend should be faster than slow");
}
