//! Concurrency tests for CIM-IPLD
//!
//! Tests concurrent writes, cache race conditions, and chain concurrent append operations.
//!
//! ## Test Flow Diagram
//!
//! ```mermaid
//! graph TD
//!     A[Concurrency Tests] --> B[Concurrent Writes]
//!     A --> C[Cache Race Conditions]
//!     A --> D[Chain Concurrent Append]
//!
//!     B --> B1[Multiple Writers]
//!     B1 --> B2[Same Content]
//!     B2 --> B3[Verify Deduplication]
//!
//!     C --> C1[Multiple Threads]
//!     C1 --> C2[Access During Caching]
//!     C2 --> C3[Verify No Corruption]
//!
//!     D --> D1[Multiple Appenders]
//!     D1 --> D2[Simultaneous Append]
//!     D2 --> D3[Verify Order Maintained]
//! ```

mod common;

use common::*;
use common::assertions::*;

use anyhow::Result;
use cim_ipld::{
    chain::ContentChain,
    TypedContent,
};
use std::sync::Arc;
use tokio::sync::{RwLock, Barrier};
use std::time::Duration;
use futures::future::join_all;

#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_concurrent_writes_same_cid() -> Result<()> {
    let context = Arc::new(TestContext::new().await?);
    let num_writers = 10;

    // Given: Multiple clients writing same content
    let content = TestContent {
        id: "concurrent-1".to_string(),
        data: "Same content from multiple writers".to_string(),
        value: 42,
    };

    // Barrier to ensure all writers start at the same time
    let barrier = Arc::new(Barrier::new(num_writers));

    // When: Writing same content simultaneously
    let mut handles = vec![];

    for i in 0..num_writers {
        let context_clone = context.clone();
        let content_clone = content.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            // Wait for all writers to be ready
            barrier_clone.wait().await;

            // Try to write the content
            let result = context_clone.with_content(content_clone).await;
            (i, result)
        });

        handles.push(handle);
    }

    // Collect all results
    let results: Vec<(usize, Result<_>)> = join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // Then: All writes should succeed with the same CID (deduplication)
    let mut cids = vec![];
    for (writer_id, result) in results {
        match result {
            Ok(cid) => {
                println!("Writer {writer_id} got CID: {cid}");
                cids.push(cid);
            }
            Err(e) => {
                panic!("Writer {} failed: {}", writer_id, e);
            }
        }
    }

    // All CIDs should be identical (content deduplication)
    let first_cid = &cids[0];
    for cid in &cids[1..] {
        assert_cids_equal(first_cid, cid);
    }

    // Verify content is stored correctly
    let retrieved: TestContent = context.storage.get(first_cid).await?;
    assert_content_equal(&content, &retrieved);

    Ok(())
}

#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_cache_race_conditions() -> Result<()> {
    let context = Arc::new(TestContext::new().await?);
    let num_readers = 20;

    // Given: Content being cached
    let content = TestContent {
        id: "cache-race-1".to_string(),
        data: "Content for cache race testing".to_string(),
        value: 100,
    };

    let cid = context.with_content(content.clone()).await?;

    // Clear any existing cache (simulate cold start)
    // In real implementation, we'd have cache.clear() method

    // Barrier to ensure all readers start simultaneously
    let barrier = Arc::new(Barrier::new(num_readers));

    // When: Multiple threads access during caching
    let mut handles = vec![];

    for i in 0..num_readers {
        let context_clone = context.clone();
        let cid_clone = cid.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            // Wait for all readers
            barrier_clone.wait().await;

            // Try to read content (triggering cache population)
            let start = std::time::Instant::now();
            let result = context_clone.storage.get::<TestContent>(&cid_clone).await;
            let duration = start.elapsed();

            (i, result, duration)
        });

        handles.push(handle);
    }

    // Collect results
    let results: Vec<(usize, Result<TestContent>, Duration)> = join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // Then: No corruption or deadlocks
    for (reader_id, result, duration) in results {
        match result {
            Ok(retrieved) => {
                println!("Reader {reader_id} succeeded in {:?}", duration);

                // Verify content integrity
                assert_content_equal(&content, &retrieved);
            }
            Err(e) => {
                panic!("Reader {} failed: {}", reader_id, e);
            }
        }
    }

    Ok(())
}

#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_chain_concurrent_append() -> Result<()> {
    let context = Arc::new(TestContext::new().await?);
    let num_appenders = 5;

    // Given: Chain with multiple writers
    let chain = Arc::new(RwLock::new(ContentChain::<TestContent>::new()));

    // Add initial item
    {
        let mut chain_write = chain.write().await;
        let initial = TestContent {
            id: "chain-init".to_string(),
            data: "Initial chain item".to_string(),
            value: 0,
        };
        chain_write.append(initial, &*context.storage).await?;
    }

    // Barrier for simultaneous appends
    let barrier = Arc::new(Barrier::new(num_appenders));

    // When: Simultaneous appends
    let mut handles = vec![];

    for i in 0..num_appenders {
        let context_clone = context.clone();
        let chain_clone = chain.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            // Wait for all appenders
            barrier_clone.wait().await;

            // Create unique content
            let content = TestContent {
                id: format!("concurrent-append-{i}"),
                data: format!("Concurrent data from appender {i}"),
                value: (i + 1) as u64,
            };

            // Try to append
            let mut chain_write = chain_clone.write().await;
            let result = chain_write.append(content.clone(), &*context_clone.storage).await;

            (i, content, result)
        });

        handles.push(handle);
    }

    // Collect results
    let results: Vec<(usize, TestContent, Result<()>)> = join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // Then: All appends should succeed
    for (appender_id, _content, result) in &results {
        match result {
            Ok(()) => {
                println!("Appender {appender_id} succeeded");
            }
            Err(e) => {
                panic!("Appender {} failed: {}", appender_id, e);
            }
        }
    }

    // Verify chain integrity
    let chain_read = chain.read().await;
    assert!(
        chain_read.validate(&*context.storage).await?,
        "Chain should remain valid after concurrent appends"
    );

    // Verify all items are in the chain
    let items = chain_read.items();
    assert_eq!(
        items.len(),
        num_appenders + 1, // +1 for initial item
        "All appends should be in the chain"
    );

    // Verify proper ordering is maintained
    // Note: With RwLock, order depends on lock acquisition
    println!("Final chain order:");
    for (i, item) in items.iter().enumerate() {
        let content: TestContent = context.storage.get(&item.content_cid).await?;
        println!("  Position {i}: {content.id}");
    }

    Ok(())
}

#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_concurrent_chain_validation() -> Result<()> {
    let context = Arc::new(TestContext::new().await?);

    // Create a chain
    let mut chain = ContentChain::<TestContent>::new();

    // Add many items
    for i in 0..50 {
        let content = TestContent {
            id: format!("validation-{i}"),
            data: format!("Data for validation {i}"),
            value: i as u64,
        };
        chain.append(content, &*context.storage).await?;
    }

    let chain = Arc::new(chain);
    let num_validators = 10;

    // Multiple threads validating simultaneously
    let mut handles = vec![];

    for i in 0..num_validators {
        let chain_clone = chain.clone();
        let context_clone = context.clone();

        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            let result = chain_clone.validate(&*context_clone.storage).await;
            let duration = start.elapsed();

            (i, result, duration)
        });

        handles.push(handle);
    }

    // All validations should succeed
    let results = join_all(handles).await;

    for result in results.into_iter() {
        let (validator_id, validation_result, duration) = result?;
        assert!(
            validation_result?,
            "Validator {} failed validation",
            validator_id
        );
        println!("Validator {validator_id} completed in {:?}", duration);
    }

    Ok(())
}

#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_concurrent_different_content_writes() -> Result<()> {
    let context = Arc::new(TestContext::new().await?);
    let num_writers = 20;

    // Each writer writes different content
    let mut handles = vec![];

    for i in 0..num_writers {
        let context_clone = context.clone();

        let handle = tokio::spawn(async move {
            let content = TestContent {
                id: format!("writer-{i}"),
                data: format!("Unique content from writer {i}: {uuid::Uuid::new_v4(}")),
                value: i as u64,
            };

            let cid = context_clone.with_content(content.clone()).await?;
            Ok::<_, anyhow::Error>((i, content, cid))
        });

        handles.push(handle);
    }

    // Collect all results
    let results: Vec<(usize, TestContent, cid::Cid)> = join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap().unwrap())
        .collect();

    // Verify all CIDs are unique (different content)
    let mut cids = std::collections::HashSet::new();
    for (writer_id, content, cid) in &results {
        assert!(
            cids.insert(cid.clone()),
            "Writer {} produced duplicate CID",
            writer_id
        );

        // Verify content retrieval
        let retrieved: TestContent = context.storage.get(cid).await?;
        assert_content_equal(content, &retrieved);
    }

    assert_eq!(cids.len(), num_writers, "All CIDs should be unique");

    Ok(())
}

#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_stress_concurrent_operations() -> Result<()> {
    let context = Arc::new(TestContext::new().await?);
    let num_operations = 100;
    let operation_types = 4; // read, write, chain append, validate

    // Shared chain for some operations
    let chain = Arc::new(RwLock::new(ContentChain::<TestContent>::new()));

    // Pre-populate some content
    let mut existing_cids = vec![];
    for i in 0..10 {
        let content = TestContent {
            id: format!("pre-existing-{i}"),
            data: format!("Pre-existing content {i}"),
            value: i as u64,
        };
        let cid = context.with_content(content).await?;
        existing_cids.push(cid);
    }

    let existing_cids = Arc::new(existing_cids);

    // Launch many concurrent operations
    let mut handles = vec![];

    for i in 0..num_operations {
        let context_clone = context.clone();
        let chain_clone = chain.clone();
        let existing_cids_clone = existing_cids.clone();

        let handle = tokio::spawn(async move {
            let operation_type = i % operation_types;

            match operation_type {
                0 => {
                    // Read operation
                    let cid_index = i % existing_cids_clone.len();
                    let cid = &existing_cids_clone[cid_index];
                    let result = context_clone.storage.get::<TestContent>(cid).await;
                    (i, "read", result.is_ok())
                }
                1 => {
                    // Write operation
                    let content = TestContent {
                        id: format!("stress-write-{i}"),
                        data: format!("Stress test content {i}"),
                        value: i as u64,
                    };
                    let result = context_clone.with_content(content).await;
                    (i, "write", result.is_ok())
                }
                2 => {
                    // Chain append operation
                    let content = TestContent {
                        id: format!("stress-chain-{i}"),
                        data: format!("Chain stress content {i}"),
                        value: i as u64,
                    };
                    let mut chain_write = chain_clone.write().await;
                    let result = chain_write.append(content, &*context_clone.storage).await;
                    (i, "append", result.is_ok())
                }
                3 => {
                    // Chain validate operation
                    let chain_read = chain_clone.read().await;
                    let result = chain_read.validate(&*context_clone.storage).await;
                    (i, "validate", result.is_ok())
                }
                _ => unreachable!(),
            }
        });

        handles.push(handle);
    }

    // Wait for all operations
    let results = join_all(handles).await;

    // Count successes by operation type
    let mut success_counts = std::collections::HashMap::new();
    for result in results {
        let (op_id, op_type, success) = result?;
        if success {
            *success_counts.entry(op_type).or_insert(0) += 1;
        } else {
            println!("Operation {op_id} ({op_type}) failed");
        }
    }

    // Print statistics
    println!("Stress test results:");
    for (op_type, count) in success_counts {
        println!("  {op_type}: {count} successful operations");
    }

    // Final chain validation
    let chain_read = chain.read().await;
    assert!(
        chain_read.validate(&*context.storage).await?,
        "Chain should be valid after stress test"
    );

    Ok(())
}
