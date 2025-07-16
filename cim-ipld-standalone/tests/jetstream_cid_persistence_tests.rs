//! Comprehensive tests for CID persistence through JetStream
//!
//! These tests verify that CIDs remain consistent when content is stored
//! and retrieved through NATS JetStream, ensuring content-addressed storage
//! integrity across the distributed system.
//!
//! # Test Coverage
//! - Basic CID persistence and retrieval
//! - CID consistency across multiple storage/retrieval cycles
//! - CID verification with different content types
//! - CID integrity under concurrent operations
//! - CID persistence with compression
//! - CID chain consistency
//!
//! # Mermaid Diagrams
//! Each test includes a Mermaid diagram showing the test flow and what is being verified.

mod common;

use cim_ipld::{
    object_store::ContentStorageService,
    chain::ContentChain,
    TypedContent, ContentType, Cid,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use common::{TestContext, TestContent};

/// Test content with metadata to verify canonical payload extraction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct MetadataContent {
    // Core content (included in CID)
    pub data: String,
    pub value: u64,
    
    // Metadata (excluded from CID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl TypedContent for MetadataContent {
    const CODEC: u64 = 0x400001;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x400001);
    
    fn canonical_payload(&self) -> cim_ipld::Result<Vec<u8>> {
        // Only include data and value in CID calculation
        #[derive(Serialize)]
        struct CanonicalPayload<'a> {
            data: &'a str,
            value: u64,
        }
        
        let payload = CanonicalPayload {
            data: &self.data,
            value: self.value,
        };
        
        Ok(serde_json::to_vec(&payload)?)
    }
}

/// Test event type for event chain testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestEvent {
    pub event_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
}

impl TypedContent for TestEvent {
    const CODEC: u64 = 0x400002;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x400002);
}

// ChainedContent is not a trait, it's a struct wrapper
// We'll use TestEvent directly with ContentChain

/// Test that CIDs remain consistent across store and retrieve operations
///
/// ```mermaid
/// graph TD
///     A[Create Content] --> B[Calculate CID]
///     B --> C[Store in JetStream]
///     C --> D[Retrieve by CID]
///     D --> E[Verify Content Match]
///     E --> F[Recalculate CID]
///     F --> G[Verify CID Match]
/// ```
#[tokio::test]
#[ignore = "Requires running NATS server"]
/// basic_cid_persistence
async fn test_basic_cid_persistence() -> Result<(), Box<dyn std::error::Error>> {
    let context = TestContext::new().await?;
    let content = TestContent {
        id: "test-cid-1".to_string(),
        data: "Testing CID persistence".to_string(),
        value: 42,
    };
    
    // Calculate CID before storage
    let expected_cid = content.calculate_cid()?;
    println!("Expected CID: {expected_cid}");
    
    // Store content
    let stored_cid = context.storage.put(&content).await?;
    assert_eq!(stored_cid, expected_cid, "Stored CID should match calculated CID");
    
    // Retrieve content
    let retrieved: TestContent = context.storage.get(&stored_cid).await?;
    assert_eq!(retrieved, content, "Retrieved content should match original");
    
    // Verify CID remains consistent
    let retrieved_cid = retrieved.calculate_cid()?;
    assert_eq!(retrieved_cid, expected_cid, "Retrieved content CID should match original");
    
    Ok(())
}

#[tokio::test]
#[ignore = "Requires running NATS server"]
/// cid_consistency_across_cycles
async fn test_cid_consistency_across_cycles() -> Result<(), Box<dyn std::error::Error>> {
    // Test CID consistency across multiple storage/retrieval cycles
    //
    // ```mermaid
    // graph TD
    //     A[Original Content] --> B[Store Cycle 1]
    //     B --> C[Retrieve Cycle 1]
    //     C --> D[Verify CID 1]
    //     D --> E[Store Cycle 2]
    //     E --> F[Retrieve Cycle 2]
    //     F --> G[Verify CID 2]
    //     G --> H[Compare All CIDs]
    // ```
    
    let context = TestContext::new().await?;
    let original_content = TestContent {
        id: "cycle-test".to_string(),
        data: "Testing multiple cycles".to_string(),
        value: 100,
    };
    
    let original_cid = original_content.calculate_cid()?;
    let mut cids = vec![original_cid];
    
    // Perform multiple store/retrieve cycles
    let mut current_content = original_content.clone();
    for cycle in 0..5 {
        println!("Cycle {}", cycle + 1);
        
        // Store
        let stored_cid = context.storage.put(&current_content).await?;
        cids.push(stored_cid);
        
        // Retrieve
        current_content = context.storage.get(&stored_cid).await?;
        
        // Verify content hasn't changed
        assert_eq!(current_content, original_content, "Content changed in cycle {}", cycle + 1);
    }
    
    // All CIDs should be identical
    for (i, cid) in cids.iter().enumerate() {
        assert_eq!(cid, &original_cid, "CID mismatch at position {}", i);
    }
    
    Ok(())
}

#[tokio::test]
#[ignore = "Requires running NATS server"]
/// canonical_payload_cid_consistency
async fn test_canonical_payload_cid_consistency() -> Result<(), Box<dyn std::error::Error>> {
    // Test that canonical payload extraction ensures consistent CIDs
    //
    // ```mermaid
    // graph TD
    //     A[Content with Metadata] --> B[Extract Canonical Payload]
    //     B --> C[Calculate CID]
    //     C --> D[Store in JetStream]
    //     D --> E[Add Different Metadata]
    //     E --> F[Recalculate CID]
    //     F --> G[Verify CID Unchanged]
    // ```
    
    let context = TestContext::new().await?;
    
    // Create content with metadata
    let content1 = MetadataContent {
        data: "Important data".to_string(),
        value: 999,
        timestamp: Some(1234567890),
        version: Some("v1.0".to_string()),
    };
    
    // Create same content with different metadata
    let content2 = MetadataContent {
        data: "Important data".to_string(),
        value: 999,
        timestamp: Some(9876543210),
        version: Some("v2.0".to_string()),
    };
    
    // CIDs should be identical (metadata excluded)
    let cid1 = content1.calculate_cid()?;
    let cid2 = content2.calculate_cid()?;
    assert_eq!(cid1, cid2, "CIDs should match despite different metadata");
    
    // Store both
    let stored_cid1 = context.storage.put(&content1).await?;
    let stored_cid2 = context.storage.put(&content2).await?;
    
    assert_eq!(stored_cid1, cid1);
    assert_eq!(stored_cid2, cid2);
    assert_eq!(stored_cid1, stored_cid2);
    
    // Retrieve and verify
    let retrieved: MetadataContent = context.storage.get(&stored_cid1).await?;
    let retrieved_cid = retrieved.calculate_cid()?;
    assert_eq!(retrieved_cid, cid1);
    
    Ok(())
}

#[tokio::test]
#[ignore = "Requires running NATS server"]
/// concurrent_cid_operations
async fn test_concurrent_cid_operations() -> Result<(), Box<dyn std::error::Error>> {
    // Test CID consistency under concurrent operations
    //
    // ```mermaid
    // graph TD
    //     A[Create Content] --> B[Calculate CID]
    //     B --> C[Spawn Concurrent Tasks]
    //     C --> D1[Task 1: Store]
    //     C --> D2[Task 2: Store]
    //     C --> D3[Task 3: Store]
    //     D1 --> E[Retrieve All]
    //     D2 --> E
    //     D3 --> E
    //     E --> F[Verify All CIDs Match]
    // ```
    
    let context = Arc::new(TestContext::new().await?);
    let content = TestContent {
        id: "concurrent-test".to_string(),
        data: "Testing concurrent operations".to_string(),
        value: 777,
    };
    
    let expected_cid = content.calculate_cid()?;
    let results = Arc::new(Mutex::new(Vec::new()));
    
    // Spawn concurrent storage operations
    let mut handles = vec![];
    for i in 0..10 {
        let ctx = context.clone();
        let content = content.clone();
        let results = results.clone();
        
        let handle = tokio::spawn(async move {
            println!("Task {i} storing content");
            let cid = ctx.storage.put(&content).await.unwrap();
            results.lock().await.push(cid);
            cid
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        let cid = handle.await?;
        assert_eq!(cid, expected_cid);
    }
    
    // Verify all stored CIDs are identical
    let stored_cids = results.lock().await;
    for cid in stored_cids.iter() {
        assert_eq!(cid, &expected_cid);
    }
    
    // Retrieve and verify
    let retrieved: TestContent = context.storage.get(&expected_cid).await?;
    assert_eq!(retrieved, content);
    
    Ok(())
}

#[tokio::test]
#[ignore = "Requires running NATS server"]
/// cid_persistence_with_compression
async fn test_cid_persistence_with_compression() -> Result<(), Box<dyn std::error::Error>> {
    // Test CID consistency with compression enabled
    //
    // ```mermaid
    // graph TD
    //     A[Small Content] --> B[Store Uncompressed]
    //     B --> C[Verify CID]
    //     D[Large Content] --> E[Store Compressed]
    //     E --> F[Verify CID]
    //     C --> G[Compare CIDs]
    //     F --> G
    //     G --> H[Retrieve Both]
    //     H --> I[Verify Content Match]
    // ```
    
    let context = TestContext::new().await?;
    
    // Small content (below compression threshold)
    let small_content = TestContent {
        id: "small".to_string(),
        data: "Small data".to_string(),
        value: 1,
    };
    
    // Large content (above compression threshold)
    let large_content = TestContent {
        id: "large".to_string(),
        data: "x".repeat(2000), // Over 1KB
        value: 2,
    };
    
    // Calculate CIDs
    let small_cid = small_content.calculate_cid()?;
    let large_cid = large_content.calculate_cid()?;
    
    // Store both
    let stored_small_cid = context.storage.put(&small_content).await?;
    let stored_large_cid = context.storage.put(&large_content).await?;
    
    assert_eq!(stored_small_cid, small_cid);
    assert_eq!(stored_large_cid, large_cid);
    
    // Retrieve and verify
    let retrieved_small: TestContent = context.storage.get(&small_cid).await?;
    let retrieved_large: TestContent = context.storage.get(&large_cid).await?;
    
    assert_eq!(retrieved_small, small_content);
    assert_eq!(retrieved_large, large_content);
    
    // Verify CIDs remain consistent
    assert_eq!(retrieved_small.calculate_cid()?, small_cid);
    assert_eq!(retrieved_large.calculate_cid()?, large_cid);
    
    Ok(())
}

#[tokio::test]
#[ignore = "Requires running NATS server"]
/// event_chain_cid_persistence
async fn test_event_chain_cid_persistence() -> Result<(), Box<dyn std::error::Error>> {
    // Test CID chain consistency for event streams
    //
    // ```mermaid
    // graph TD
    //     A[Create Event Chain] --> B[Add Event 1]
    //     B --> C[Store Chain State 1]
    //     C --> D[Add Event 2]
    //     D --> E[Store Chain State 2]
    //     E --> F[Add Event 3]
    //     F --> G[Store Chain State 3]
    //     G --> H[Verify Chain Integrity]
    //     H --> I[Reload Chain]
    //     I --> J[Verify All CIDs Match]
    // ```
    
    let context = TestContext::new().await?;
    let mut chain = ContentChain::<TestEvent>::new();
    let mut expected_cids = Vec::new();
    
    // Add events to chain
    for i in 0..5 {
        let event = TestEvent {
            event_id: format!("event-{i}"),
            event_type: "test.event".to_string(),
            payload: serde_json::json!({
                "index": i,
                "data": format!("Event data {i}")
            }),
        };
        
        // Append to chain
        let _chained = chain.append(event.clone())?;
        
        // Store the event separately
        let cid = context.storage.put(&event).await?;
        expected_cids.push(cid);
        
        // Verify the event can be retrieved
        let retrieved: TestEvent = context.storage.get(&cid).await?;
        assert_eq!(retrieved, event);
    }
    
    // Verify chain integrity
    let items = chain.items();
    assert_eq!(items.len(), 5);
    
    for (i, item) in items.iter().enumerate() {
        assert_eq!(item.sequence, i as u64);
        
        if i > 0 {
            assert!(item.previous_cid.is_some());
        } else {
            assert!(item.previous_cid.is_none());
        }
    }
    
    // Create a new chain and load from storage
    let mut loaded_chain = ContentChain::<TestEvent>::new();
    
    // Load events in order
    for cid in &expected_cids {
        let event: TestEvent = context.storage.get(cid).await?;
        loaded_chain.append(event)?;
    }
    
    // Verify loaded chain matches original
    let loaded_items = loaded_chain.items();
    assert_eq!(loaded_items.len(), items.len());
    
    for (i, (orig_item, loaded_item)) in 
        items.iter().zip(loaded_items.iter()).enumerate() 
    {
        assert_eq!(orig_item.sequence, loaded_item.sequence, "Sequence mismatch at position {}", i);
        assert_eq!(orig_item.previous_cid, loaded_item.previous_cid, "Previous CID mismatch at position {}", i);
    }
    
    Ok(())
}

#[tokio::test]
#[ignore = "Requires running NATS server"]
/// cid_persistence_across_buckets
async fn test_cid_persistence_across_buckets() -> Result<(), Box<dyn std::error::Error>> {
    // Test CID consistency across different content buckets
    //
    // ```mermaid
    // graph TD
    //     A[Create Content] --> B[Store in Events Bucket]
    //     B --> C[Store in Graphs Bucket]
    //     C --> D[Store in Nodes Bucket]
    //     D --> E[Retrieve from Each]
    //     E --> F[Verify All CIDs Match]
    //     F --> G[Verify Content Integrity]
    // ```
    
    let context = TestContext::new().await?;
    let content = TestContent {
        id: "bucket-test".to_string(),
        data: "Testing across buckets".to_string(),
        value: 123,
    };
    
    let expected_cid = content.calculate_cid()?;
    
    // Store in different buckets (simulated by using the same content type)
    // In a real scenario, we'd have different content types for different buckets
    let cid1 = context.storage.put(&content).await?;
    let cid2 = context.storage.put(&content).await?;
    let cid3 = context.storage.put(&content).await?;
    
    // All CIDs should be identical
    assert_eq!(cid1, expected_cid);
    assert_eq!(cid2, expected_cid);
    assert_eq!(cid3, expected_cid);
    
    // Retrieve and verify
    let retrieved1: TestContent = context.storage.get(&cid1).await?;
    let retrieved2: TestContent = context.storage.get(&cid2).await?;
    let retrieved3: TestContent = context.storage.get(&cid3).await?;
    
    assert_eq!(retrieved1, content);
    assert_eq!(retrieved2, content);
    assert_eq!(retrieved3, content);
    
    Ok(())
}

#[tokio::test]
#[ignore = "Requires running NATS server"]
/// cid_error_handling
async fn test_cid_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    // Test error handling for CID mismatches and invalid CIDs
    //
    // ```mermaid
    // graph TD
    //     A[Store Valid Content] --> B[Try Invalid CID]
    //     B --> C[Expect NotFound Error]
    //     C --> D[Try Random CID]
    //     D --> E[Expect NotFound Error]
    //     E --> F[Verify Valid CID Works]
    // ```
    
    let context = TestContext::new().await?;
    let content = TestContent {
        id: "error-test".to_string(),
        data: "Testing error handling".to_string(),
        value: 404,
    };
    
    // Store content
    let valid_cid = context.storage.put(&content).await?;
    
    // Try to retrieve with invalid CID
    let invalid_cid = Cid::try_from("bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku")?;
    let result: Result<TestContent, _> = context.storage.get(&invalid_cid).await;
    assert!(result.is_err(), "Should fail with invalid CID");
    
    // Try with random CID
    let random_content = TestContent {
        id: "random".to_string(),
        data: "Random data".to_string(),
        value: 999,
    };
    let random_cid = random_content.calculate_cid()?;
    let result: Result<TestContent, _> = context.storage.get(&random_cid).await;
    assert!(result.is_err(), "Should fail with non-existent CID");
    
    // Verify valid CID still works
    let retrieved: TestContent = context.storage.get(&valid_cid).await?;
    assert_eq!(retrieved, content);
    
    Ok(())
}

#[tokio::test]
#[ignore = "Requires running NATS server"]
/// storage_service_cid_caching
async fn test_storage_service_cid_caching() -> Result<(), Box<dyn std::error::Error>> {
    // Test CID consistency with caching layer
    //
    // ```mermaid
    // graph TD
    //     A[Create Storage Service] --> B[Store Content]
    //     B --> C[First Retrieve - From Store]
    //     C --> D[Second Retrieve - From Cache]
    //     D --> E[Verify CIDs Match]
    //     E --> F[Clear Cache]
    //     F --> G[Third Retrieve - From Store]
    //     G --> H[Verify CID Consistency]
    // ```
    
    let context = TestContext::new().await?;
    let storage_service = ContentStorageService::new(
        context.storage.clone(),
        10, // cache size
        Duration::from_secs(60),
        1024 * 1024, // 1MB cache
    );
    
    let content = TestContent {
        id: "cache-test".to_string(),
        data: "Testing with cache".to_string(),
        value: 555,
    };
    
    let expected_cid = content.calculate_cid()?;
    
    // Store content
    let stored_cid = storage_service.store(&content).await?;
    assert_eq!(stored_cid, expected_cid);
    
    // First retrieval (from store)
    let retrieved1: TestContent = storage_service.get(&stored_cid).await?;
    assert_eq!(retrieved1, content);
    assert_eq!(retrieved1.calculate_cid()?, expected_cid);
    
    // Check cache stats
    let stats = storage_service.cache_stats().await;
    assert_eq!(stats.entries, 1);
    
    // Second retrieval (from cache)
    let retrieved2: TestContent = storage_service.get(&stored_cid).await?;
    assert_eq!(retrieved2, content);
    assert_eq!(retrieved2.calculate_cid()?, expected_cid);
    
    // Clear cache
    storage_service.clear_cache().await;
    
    // Third retrieval (from store again)
    let retrieved3: TestContent = storage_service.get(&stored_cid).await?;
    assert_eq!(retrieved3, content);
    assert_eq!(retrieved3.calculate_cid()?, expected_cid);
    
    Ok(())
}

#[tokio::test]
#[ignore = "Requires running NATS server"]
/// batch_cid_operations
async fn test_batch_cid_operations() -> Result<(), Box<dyn std::error::Error>> {
    // Test CID consistency in batch operations
    //
    // ```mermaid
    // graph TD
    //     A[Create Multiple Contents] --> B[Calculate Expected CIDs]
    //     B --> C[Batch Store]
    //     C --> D[Batch Retrieve]
    //     D --> E[Verify All CIDs]
    //     E --> F[Verify Content Order]
    //     F --> G[Random Access by CID]
    // ```
    
    let context = TestContext::new().await?;
    let storage_service = ContentStorageService::new(
        context.storage.clone(),
        100,
        Duration::from_secs(300),
        10 * 1024 * 1024,
    );
    
    // Create multiple contents
    let contents: Vec<TestContent> = (0..20)
        .map(|i| TestContent {
            id: format!("batch-{i}"),
            data: format!("Batch content number {i}"),
            value: i as u64 * 10,
        })
        .collect();
    
    // Calculate expected CIDs
    let mut expected_cids = Vec::new();
    for content in &contents {
        expected_cids.push(content.calculate_cid()?);
    }
    
    // Store all contents
    let mut stored_cids = Vec::new();
    for content in &contents {
        let cid = storage_service.store(content).await?;
        stored_cids.push(cid);
    }
    
    // Verify stored CIDs match expected
    assert_eq!(stored_cids, expected_cids);
    
    // Batch retrieve
    let retrieved = storage_service.get_batch::<TestContent>(&stored_cids).await?;
    
    // Verify all content and CIDs
    assert_eq!(retrieved.len(), contents.len());
    for (i, (retrieved_content, original_content)) in retrieved.iter().zip(contents.iter()).enumerate() {
        assert_eq!(retrieved_content, original_content, "Content mismatch at index {}", i);
        let recalculated_cid = retrieved_content.calculate_cid()?;
        assert_eq!(recalculated_cid, expected_cids[i], "CID mismatch at index {}", i);
    }
    
    // Random access verification
    let random_indices = vec![5, 12, 3, 18, 7];
    for idx in random_indices {
        let cid = &stored_cids[idx];
        let retrieved: TestContent = storage_service.get(cid).await?;
        assert_eq!(retrieved, contents[idx]);
        assert_eq!(retrieved.calculate_cid()?, expected_cids[idx]);
    }
    
    Ok(())
} 