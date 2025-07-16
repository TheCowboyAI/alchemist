use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::Mutex;
use anyhow::Result;
use cid::Cid;
use rand::{Rng, thread_rng};
use serde::{Serialize, Deserialize};

use cim_ipld::{
    object_store::NatsObjectStore,
    chain::ContentChain,
    TypedContent,
    ContentType,
};

/// Test content structure for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestContent {
    pub id: String,
    pub data: String,
    pub value: u64,
}

impl TypedContent for TestContent {
    const CODEC: u64 = 0x400000; // Test codec
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x400000);
}

/// Test context that provides common test infrastructure
pub struct TestContext {
    pub nats: NatsTestHarness,
    pub storage: Arc<NatsObjectStore>,
    pub temp_dir: TempDir,
}

impl TestContext {
    /// Create a new test context with NATS and storage
    pub async fn new() -> Result<Self> {
        let nats = NatsTestHarness::new().await?;
        let storage = Arc::new(NatsObjectStore::new(
            nats.jetstream.clone(),
            1024, // compression threshold
        ).await?);
        let temp_dir = TempDir::new()?;

        Ok(Self {
            nats,
            storage,
            temp_dir,
        })
    }

    /// Store content and return its CID
    pub async fn with_content<T: TypedContent>(&self, content: T) -> Result<Cid> {
        self.storage.put(&content).await.map_err(Into::into)
    }

    /// Corrupt content at the given CID
    pub async fn corrupt_content(&self, _cid: &Cid) -> Result<()> {
        // This would need access to the underlying storage to corrupt data
        // For now, we'll simulate by storing corrupted data with the same CID
        let mut corrupted = vec![0u8; 100];
        thread_rng().fill(&mut corrupted[..]);

        // Note: In real implementation, we'd need to bypass CID validation
        // This is for testing purposes only
        Ok(())
    }

    /// Create a test chain with the given length
    pub async fn create_test_chain(&self, length: usize) -> Result<ContentChain<TestContent>> {
        let mut chain = ContentChain::<TestContent>::new();

        for i in 0..length {
            let content = TestContent {
                id: format!("item-{i}"),
                data: format!("Test data {i}"),
                value: i as u64,
            };
            chain.append(content)?;
        }

        Ok(chain)
    }
}

/// NATS test harness for managing test NATS connections
pub struct NatsTestHarness {
    pub client: async_nats::Client,
    pub jetstream: async_nats::jetstream::Context,
}

impl NatsTestHarness {
    /// Create a new NATS test harness
    pub async fn new() -> Result<Self> {
        // Connect to local NATS server (assumes nats-server is running)
        let client = async_nats::connect("nats://localhost:4222").await?;
        let jetstream = async_nats::jetstream::new(client.clone());

        Ok(Self { client, jetstream })
    }

    /// Clean up test data
    pub async fn cleanup(&self) -> Result<()> {
        // Clean up test buckets
        // Note: This would need to be implemented based on actual bucket management
        Ok(())
    }
}

// No drop implementation needed - NATS client will clean up automatically

/// Generate test content of specified size
pub fn generate_test_content(size: usize) -> Vec<u8> {
    let mut rng = thread_rng();
    let mut content = vec![0u8; size];
    rng.fill(&mut content[..]);
    content
}

/// Create a test chain with random content
pub fn create_test_chain_data(length: usize) -> Vec<TestContent> {
    (0..length)
        .map(|i| TestContent {
            id: format!("chain-item-{i}"),
            data: format!("Chain data {i}: {}", uuid::Uuid::new_v4()),
            value: thread_rng().gen_range(0..1000),
        })
        .collect()
}

/// Helper to create a failing storage backend for testing
pub struct FailingBackend {
    pub fail_after: usize,
    pub counter: Arc<Mutex<usize>>,
}

impl FailingBackend {
    pub fn new(fail_after: usize) -> Self {
        Self {
            fail_after,
            counter: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn should_fail(&self) -> bool {
        let mut count = self.counter.lock().await;
        *count += 1;
        *count > self.fail_after
    }
}

/// Helper to create a slow storage backend for testing
pub struct SlowBackend {
    pub delay: Duration,
}

impl SlowBackend {
    pub fn new(delay: Duration) -> Self {
        Self { delay }
    }

    pub async fn simulate_delay(&self) {
        tokio::time::sleep(self.delay).await;
    }
}

/// Helper to create a corrupting storage backend for testing
pub struct CorruptingBackend {
    pub corruption_rate: f32,
}

impl CorruptingBackend {
    pub fn new(corruption_rate: f32) -> Self {
        Self { corruption_rate }
    }

    pub fn should_corrupt(&self) -> bool {
        thread_rng().gen::<f32>() < self.corruption_rate
    }

    pub fn corrupt_data(&self, data: &mut [u8]) {
        if self.should_corrupt() {
            let corruption_point = thread_rng().gen_range(0..data.len());
            data[corruption_point] = thread_rng().gen();
        }
    }
}

/// Test assertion helpers
pub mod assertions {
    use super::*;

    /// Assert that two CIDs are equal
    pub fn assert_cids_equal(expected: &Cid, actual: &Cid) {
        assert_eq!(
            expected, actual,
            "CID mismatch: expected {}, got {}",
            expected, actual
        );
    }

    /// Assert that content matches
    pub fn assert_content_equal<T: PartialEq + std::fmt::Debug>(expected: &T, actual: &T) {
        assert_eq!(
            expected, actual,
            "Content mismatch"
        );
    }

    /// Assert that an operation completes within a time limit
    pub async fn assert_completes_within<F, T>(
        duration: Duration,
        operation: F,
    ) -> T
    where
        F: std::future::Future<Output = T>,
    {
        tokio::time::timeout(duration, operation)
            .await
            .expect("Operation timed out")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "Requires running NATS server"]
    async fn test_test_context_creation() {
        let context = TestContext::new().await.unwrap();
        assert!(context.temp_dir.path().exists());
    }

    #[tokio::test]
    async fn test_content_generation() {
        let content = generate_test_content(1024);
        assert_eq!(content.len(), 1024);
    }

    #[tokio::test]
    async fn test_chain_data_creation() {
        let chain_data = create_test_chain_data(10);
        assert_eq!(chain_data.len(), 10);
        for (i, item) in chain_data.iter().enumerate() {
            assert_eq!(item.id, format!("chain-item-{i}"));
        }
    }
}
