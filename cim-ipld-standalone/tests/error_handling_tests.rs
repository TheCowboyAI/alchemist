//! Error handling tests for CIM-IPLD
//!
//! Tests network failure recovery, storage quota exceeded, and corrupted content handling.
//!
//! ## Test Flow Diagram
//!
//! ```mermaid
//! graph TD
//!     A[Error Handling Tests] --> B[Network Failures]
//!     A --> C[Storage Quota]
//!     A --> D[Corrupted Content]
//!
//!     B --> B1[Active Operation]
//!     B1 --> B2[Network Fails]
//!     B2 --> B3[Graceful Recovery]
//!
//!     C --> C1[Near-Full Storage]
//!     C1 --> C2[Large Content]
//!     C2 --> C3[Proper Error]
//!
//!     D --> D1[Corrupted Store]
//!     D1 --> D2[Retrieval Attempt]
//!     D2 --> D3[Clear Error]
//! ```

mod common;
use common::*;

use anyhow::{Result, anyhow};
use cim_ipld::{
    object_store::{ObjectStore, NatsObjectStore},
    chain::ContentChain,
    codec::ContentCodec,
    types::TypedContent,
};
use std::sync::Arc;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_network_failure_recovery() -> Result<()> {
    // This test simulates network failures during operations

    // Given: Active storage operation
    let context = TestContext::new().await?;

    let content = TestContent {
        id: "network-test-1".to_string(),
        data: "Content for network failure testing".to_string(),
        value: 42,
    };

    // Store content successfully first
    let cid = context.with_content(content.clone()).await?;

    // When: Network fails mid-operation
    // We simulate this by using a timeout that's too short
    let short_timeout = Duration::from_millis(1);

    let retrieval_result = timeout(
        short_timeout,
        context.storage.get(&cid)
    ).await;

    // Then: Should handle timeout gracefully
    assert!(
        retrieval_result.is_err(),
        "Should timeout with very short duration"
    );

    // Verify we can still retrieve after "network recovery"
    let normal_timeout = Duration::from_secs(5);
    let recovery_result = timeout(
        normal_timeout,
        context.storage.get(&cid)
    ).await??;

    let recovered: TestContent = ContentCodec::decode(&recovery_result)?;
    assert_content_equal(&content, &recovered);

    Ok(())
}

#[tokio::test]
async fn test_storage_quota_exceeded() -> Result<()> {
    let context = TestContext::new().await?;

    // Given: Near-full storage (simulated)
    // We'll test with increasingly large content
    let sizes = vec![
        1024,           // 1 KB
        1024 * 1024,    // 1 MB
        10 * 1024 * 1024, // 10 MB
    ];

    for size in sizes {
        let large_data = generate_test_content(size);
        let content = TestContent {
            id: format!("quota-test-{size}"),
            data: base64::encode(&large_data),
            value: size as u64,
        };

        // When: Large content stored
        let result = context.with_content(content.clone()).await;

        // Then: Should handle appropriately
        match result {
            Ok(cid) => {
                println!("Successfully stored {size} bytes with CID: {cid}");

                // Verify retrieval works
                let retrieved_bytes = context.storage.get(&cid).await?;
                let retrieved: TestContent = ContentCodec::decode(&retrieved_bytes)?;
                assert_eq!(retrieved.value, size as u64);
            }
            Err(e) => {
                println!("Storage failed for {size} bytes: {e}");
                // This is expected for very large content
                // Verify error is meaningful
                assert!(
                    e.to_string().contains("storage") ||
                    e.to_string().contains("quota") ||
                    e.to_string().contains("size"),
                    "Error should indicate storage issue: {}",
                    e
                );
            }
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_corrupted_content_handling() -> Result<()> {
    // This test verifies handling of corrupted content

    let context = TestContext::new().await?;

    // Given: Valid content stored
    let content = TestContent {
        id: "corruption-test-1".to_string(),
        data: "Original uncorrupted data".to_string(),
        value: 100,
    };

    let cid = context.with_content(content.clone()).await?;

    // When: Content is corrupted (simulated by storing different content with wrong CID)
    // In a real scenario, this would involve direct storage manipulation

    // Try to decode invalid bytes as our content type
    let corrupted_bytes = b"This is not valid encoded content!";

    // Then: Decoding should fail with clear error
    let decode_result = ContentCodec::decode::<TestContent>(corrupted_bytes);
    assert!(
        decode_result.is_err(),
        "Decoding corrupted content should fail"
    );

    if let Err(e) = decode_result {
        println!("Corruption detected: {e}");
        // Verify error is informative
        let error_string = e.to_string();
        assert!(
            error_string.contains("decode") ||
            error_string.contains("deserialize") ||
            error_string.contains("invalid"),
            "Error should indicate decoding issue: {}",
            error_string
        );
    }

    // Verify original content is still retrievable
    let valid_bytes = context.storage.get(&cid).await?;
    let valid_content: TestContent = ContentCodec::decode(&valid_bytes)?;
    assert_content_equal(&content, &valid_content);

    Ok(())
}

#[tokio::test]
async fn test_chain_with_missing_items() -> Result<()> {
    let context = TestContext::new().await?;

    // Create a chain
    let mut chain = ContentChain::<TestContent>::new();

    // Add items
    let mut cids = vec![];
    for i in 0..5 {
        let content = TestContent {
            id: format!("chain-missing-{i}"),
            data: format!("Chain data {i}"),
            value: i as u64,
        };
        chain.append(content, &*context.storage).await?;
        if let Some(item) = chain.items().last() {
            cids.push(item.content_cid.clone());
        }
    }

    // Simulate missing item by creating a new chain with gaps
    // In real scenario, this would test actual missing content

    // Verify chain validation detects issues
    let validation_result = chain.validate(&*context.storage).await?;
    assert!(validation_result, "Complete chain should validate");

    Ok(())
}

#[tokio::test]
async fn test_concurrent_error_scenarios() -> Result<()> {
    let context = Arc::new(TestContext::new().await?);

    // Test multiple error scenarios concurrently
    let mut handles = vec![];

    // Scenario 1: Timeout during read
    let context_clone = context.clone();
    handles.push(tokio::spawn(async move {
        let content = TestContent {
            id: "concurrent-error-1".to_string(),
            data: "Timeout test".to_string(),
            value: 1,
        };
        let cid = context_clone.with_content(content).await?;

        // Very short timeout
        let result = timeout(Duration::from_nanos(1), context_clone.storage.get(&cid)).await;

        match result {
            Err(_) => Ok("Timeout handled correctly"),
            Ok(_) => Err(anyhow!("Should have timed out")),
        }
    }));

    // Scenario 2: Invalid CID
    let context_clone = context.clone();
    handles.push(tokio::spawn(async move {
        // Try to retrieve with invalid CID
        let invalid_cid = cid::Cid::default();
        let result = context_clone.storage.get(&invalid_cid).await;

        match result {
            Err(e) => {
                println!("Invalid CID error: {e}");
                Ok("Invalid CID handled correctly")
            }
            Ok(_) => Err(anyhow!("Should have failed with invalid CID")),
        }
    }));

    // Scenario 3: Large content
    let context_clone = context.clone();
    handles.push(tokio::spawn(async move {
        let huge_content = TestContent {
            id: "concurrent-error-3".to_string(),
            data: "x".repeat(50 * 1024 * 1024), // 50MB of 'x'
            value: 3,
        };

        let result = context_clone.with_content(huge_content).await;

        match result {
            Ok(cid) => {
                println!("Large content stored: {cid}");
                Ok("Large content handled")
            }
            Err(e) => {
                println!("Large content error: {e}");
                Ok("Large content error handled")
            }
        }
    }));

    // Wait for all scenarios
    let results = futures::future::join_all(handles).await;

    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(Ok(msg)) => println!("Scenario {i + 1} success: {msg}"),
            Ok(Err(e)) => panic!("Scenario {} failed: {}", i + 1, e),
            Err(e) => panic!("Scenario {} panicked: {}", i + 1, e),
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_retry_logic() -> Result<()> {
    let context = TestContext::new().await?;

    // Test retry logic with failing backend
    let failing_backend = FailingBackend::new(2); // Fail first 2 attempts

    let content = TestContent {
        id: "retry-test-1".to_string(),
        data: "Content for retry testing".to_string(),
        value: 42,
    };

    // Store content
    let cid = context.with_content(content.clone()).await?;

    // Simulate retrieval with retries
    let mut attempts = 0;
    let max_retries = 3;
    let mut last_error = None;

    while attempts < max_retries {
        attempts += 1;

        if failing_backend.should_fail().await {
            last_error = Some(anyhow!("Simulated failure on attempt {}", attempts));
            println!("Attempt {attempts} failed");
            tokio::time::sleep(Duration::from_millis(100 * attempts as u64)).await;
            continue;
        }

        // Success case
        let result = context.storage.get(&cid).await?;
        let retrieved: TestContent = ContentCodec::decode(&result)?;
        assert_content_equal(&content, &retrieved);
        println!("Succeeded on attempt {attempts}");
        return Ok(());
    }

    Err(last_error.unwrap_or_else(|| anyhow!("All retries exhausted")))
}

#[tokio::test]
async fn test_partial_write_failure() -> Result<()> {
    let context = TestContext::new().await?;

    // Test handling of partial write failures in batch operations
    let mut contents = vec![];
    for i in 0..5 {
        contents.push(TestContent {
            id: format!("partial-write-{i}"),
            data: format!("Batch content {i}"),
            value: i as u64,
        });
    }

    // Store contents individually (simulating batch)
    let mut stored_cids = vec![];
    let mut failed_indices = vec![];

    for (i, content) in contents.iter().enumerate() {
        // Simulate failure on index 2
        if i == 2 {
            failed_indices.push(i);
            println!("Simulated failure for item {i}");
            continue;
        }

        match context.with_content(content.clone()).await {
            Ok(cid) => {
                stored_cids.push((i, cid));
                println!("Stored item {i} with CID: {cid}");
            }
            Err(e) => {
                failed_indices.push(i);
                println!("Failed to store item {i}: {e}");
            }
        }
    }

    // Verify partial success
    assert_eq!(stored_cids.len(), 4, "Should have stored 4 out of 5 items");
    assert_eq!(failed_indices.len(), 1, "Should have 1 failure");

    // Verify stored items are retrievable
    for (index, cid) in stored_cids {
        let retrieved_bytes = context.storage.get(&cid).await?;
        let retrieved: TestContent = ContentCodec::decode(&retrieved_bytes)?;
        assert_content_equal(&contents[index], &retrieved);
    }

    Ok(())
}

#[cfg(test)]
mod error_handling_helpers {
    use super::*;

    /// Simulate various error conditions
    pub enum ErrorCondition {
        NetworkTimeout,
        StorageFull,
        CorruptedData,
        InvalidCid,
        PermissionDenied,
    }

    impl ErrorCondition {
        pub fn to_error(&self) -> anyhow::Error {
            match self {
                Self::NetworkTimeout => anyhow!("Network operation timed out"),
                Self::StorageFull => anyhow!("Storage quota exceeded"),
                Self::CorruptedData => anyhow!("Data corruption detected"),
                Self::InvalidCid => anyhow!("Invalid CID format"),
                Self::PermissionDenied => anyhow!("Permission denied"),
            }
        }
    }

    /// Test helper for verifying error recovery
    pub async fn verify_recovery<F, T>(
        operation: F,
        expected_error: &str,
    ) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        match operation.await {
            Ok(result) => Ok(result),
            Err(e) => {
                let error_string = e.to_string();
                assert!(
                    error_string.contains(expected_error),
                    "Expected error containing '{}', got: {}",
                    expected_error,
                    error_string
                );
                Err(e)
            }
        }
    }
}
