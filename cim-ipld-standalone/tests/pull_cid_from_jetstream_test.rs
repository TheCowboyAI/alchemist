//! Test for pulling CIDs from NATS JetStream
//!
//! This test verifies the complete workflow of:
//! 1. Storing content with CIDs
//! 2. Listing available CIDs
//! 3. Pulling content by CID
//! 4. Verifying CID integrity
//!
//! ```mermaid
//! graph TD
//!     A[Store Content] --> B[Calculate CID]
//!     B --> C[Store in JetStream]
//!     C --> D[List CIDs in Bucket]
//!     D --> E[Pull by CID]
//!     E --> F[Verify Content]
//!     F --> G[Verify CID Match]
//! ```

mod common;

use cim_ipld::{
    object_store::{ContentBucket, PullOptions, helpers},
    TypedContent, ContentType, Cid,
};
use serde::{Deserialize, Serialize};
use common::TestContext;

/// Test content for pulling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct PullTestContent {
    id: String,
    title: String,
    data: Vec<u8>,
    metadata: std::collections::HashMap<String, String>,
}

impl TypedContent for PullTestContent {
    const CODEC: u64 = 0x500000;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x500000);
}

/// Test that we can pull a CID from JetStream
///
/// ```mermaid
/// graph LR
///     A[Store Content] --> B[Get CID]
///     B --> C[Pull by CID]
///     C --> D[Verify Match]
/// ```
#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_pull_single_cid() -> Result<(), Box<dyn std::error::Error>> {
    let context = TestContext::new().await?;
    
    // Create test content
    let content = PullTestContent {
        id: "pull-test-001".to_string(),
        title: "Test Pull Content".to_string(),
        data: vec![1, 2, 3, 4, 5],
        metadata: [
            ("author".to_string(), "test".to_string()),
            ("version".to_string(), "1.0".to_string()),
        ].into_iter().collect(),
    };
    
    // Store the content
    let cid = context.storage.put(&content).await?;
    println!("Stored content with CID: {cid}");
    
    // Pull it back
    let pulled: PullTestContent = context.storage.get(&cid).await?;
    
    // Verify content matches
    assert_eq!(pulled, content);
    
    // Verify CID matches
    let calculated_cid = pulled.calculate_cid()?;
    assert_eq!(calculated_cid, cid);
    
    println!("✓ Successfully pulled CID {cid} from JetStream");
    
    Ok(())
}

/// Test listing and pulling multiple CIDs
///
/// ```mermaid
/// graph TD
///     A[Store Multiple] --> B[List CIDs]
///     B --> C[Pull Each CID]
///     C --> D[Verify All]
/// ```
#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_list_and_pull_cids() -> Result<(), Box<dyn std::error::Error>> {
    let context = TestContext::new().await?;
    
    // Store multiple items
    let mut stored_cids = Vec::new();
    for i in 0..5 {
        let content = PullTestContent {
            id: format!("list-test-{i}"),
            title: format!("List Test Item {i}"),
            data: vec![i as u8; 10],
            metadata: [
                ("index".to_string(), i.to_string()),
            ].into_iter().collect(),
        };
        
        let cid = context.storage.put(&content).await?;
        stored_cids.push(cid);
    }
    
    println!("Stored {stored_cids.len(} items"));
    
    // List CIDs in bucket
    let bucket = ContentBucket::for_content_type(PullTestContent::CONTENT_TYPE.codec());
    let objects = context.storage.list(bucket).await?;
    
    // Should find at least our stored items
    assert!(objects.len() >= stored_cids.len());
    
    // Pull each stored CID
    for (i, cid) in stored_cids.iter().enumerate() {
        let pulled: PullTestContent = context.storage.get(cid).await?;
        assert_eq!(pulled.id, format!("list-test-{i}"));
        println!("✓ Pulled item {i}: {cid}");
    }
    
    Ok(())
}

/// Test pull with options
///
/// ```mermaid
/// graph TD
///     A[Store Various Sizes] --> B[Apply Filters]
///     B --> C[Pull Filtered]
///     C --> D[Verify Filters]
/// ```
#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_pull_with_options() -> Result<(), Box<dyn std::error::Error>> {
    let context = TestContext::new().await?;
    
    // Store items of various sizes
    for i in 0..10 {
        let content = PullTestContent {
            id: format!("size-test-{i}"),
            title: format!("Size Test {i}"),
            data: vec![0u8; i * 100], // Varying sizes
            metadata: Default::default(),
        };
        
        context.storage.put(&content).await?;
    }
    
    // Pull with size filter
    let options = PullOptions {
        min_size: Some(200), // At least 200 bytes
        max_size: Some(600), // At most 600 bytes
        limit: Some(3),      // Max 3 items
        compressed_only: false,
    };
    
    let bucket = ContentBucket::for_content_type(PullTestContent::CONTENT_TYPE.codec());
    let results = context.storage.pull_all::<PullTestContent>(bucket, options).await?;
    
    // Verify filters were applied
    assert!(results.len() <= 3, "Limit not applied");
    
    for result in &results {
        assert!(result.metadata.size >= 200, "Min size filter failed");
        assert!(result.metadata.size <= 600, "Max size filter failed");
        println!("✓ Pulled {result.cid} (size: {result.metadata.size} bytes)");
    }
    
    Ok(())
}

/// Test batch pull
///
/// ```mermaid
/// graph TD
///     A[Store Multiple] --> B[Collect CIDs]
///     B --> C[Batch Pull]
///     C --> D[Verify Results]
///     D --> E[Check Failures]
/// ```
#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_batch_pull() -> Result<(), Box<dyn std::error::Error>> {
    let context = TestContext::new().await?;
    
    // Store some content
    let mut cids = Vec::new();
    for i in 0..5 {
        let content = PullTestContent {
            id: format!("batch-{i}"),
            title: format!("Batch Item {i}"),
            data: vec![i as u8; 20],
            metadata: Default::default(),
        };
        
        let cid = context.storage.put(&content).await?;
        cids.push(cid);
    }
    
    // Add a fake CID that doesn't exist
    let fake_cid = Cid::try_from("bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku")?;
    cids.push(fake_cid.clone());
    
    // Batch pull
    let batch_result = context.storage.pull_batch::<PullTestContent>(&cids, 3).await;
    
    // Verify results
    assert_eq!(batch_result.successful.len(), 5);
    assert_eq!(batch_result.failed.len(), 1);
    
    // Check the failed CID
    assert_eq!(batch_result.failed[0].0, fake_cid);
    
    println!("✓ Batch pull: {batch_result.successful.len(} successful, {} failed"), 
        batch_result.failed.len()
    );
    
    Ok(())
}

/// Test pull by prefix
///
/// ```mermaid
/// graph TD
///     A[Store with Known CIDs] --> B[Search by Prefix]
///     B --> C[Pull Matching]
///     C --> D[Verify Matches]
/// ```
#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_pull_by_prefix() -> Result<(), Box<dyn std::error::Error>> {
    let context = TestContext::new().await?;
    
    // Store content and track one CID
    let content = PullTestContent {
        id: "prefix-test".to_string(),
        title: "Prefix Search Test".to_string(),
        data: vec![99; 50],
        metadata: [
            ("searchable".to_string(), "true".to_string()),
        ].into_iter().collect(),
    };
    
    let target_cid = context.storage.put(&content).await?;
    println!("Target CID: {target_cid}");
    
    // Get first few characters of the CID as prefix
    let prefix = &target_cid.to_string()[..10];
    println!("Searching with prefix: {prefix}");
    
    // Pull by prefix
    let bucket = ContentBucket::for_content_type(PullTestContent::CONTENT_TYPE.codec());
    let results = context.storage.pull_by_prefix::<PullTestContent>(bucket, prefix).await?;
    
    // Should find at least our target
    assert!(!results.is_empty(), "No results found for prefix");
    
    // Verify our content is in the results
    let found = results.iter().any(|r| r.cid == target_cid);
    assert!(found, "Target CID not found in prefix search");
    
    println!("✓ Found {results.len(} items with prefix {}"), prefix);
    
    Ok(())
}

/// Test streaming pull
///
/// ```mermaid
/// graph TD
///     A[Store Many Items] --> B[Create Stream]
///     B --> C[Process Stream]
///     C --> D[Verify Order]
/// ```
#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_stream_pull() -> Result<(), Box<dyn std::error::Error>> {
    use futures::StreamExt;
    
    let context = TestContext::new().await?;
    
    // Store several items
    for i in 0..3 {
        let content = PullTestContent {
            id: format!("stream-{i}"),
            title: format!("Stream Item {i}"),
            data: vec![i as u8; 15],
            metadata: Default::default(),
        };
        
        context.storage.put(&content).await?;
    }
    
    // Stream objects
    let bucket = ContentBucket::for_content_type(PullTestContent::CONTENT_TYPE.codec());
    let stream = context.storage.stream_objects::<PullTestContent>(bucket);
    
    // Pin the stream to make it work with next()
    use futures::pin_mut;
    pin_mut!(stream);
    
    let mut count = 0;
    while let Some(result) = stream.next().await {
        match result {
            Ok(pull_result) => {
                println!("✓ Streamed: {pull_result.cid} - {pull_result.content.title}");
                count += 1;
            }
            Err(e) => {
                eprintln!("Stream error: {e}");
            }
        }
    }
    
    assert!(count > 0, "No items streamed");
    println!("✓ Streamed {count} items total");
    
    Ok(())
}

/// Test pull and group
///
/// ```mermaid
/// graph TD
///     A[Store Categorized Items] --> B[Pull All]
///     B --> C[Group by Category]
///     C --> D[Verify Groups]
/// ```
#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_pull_and_group() -> Result<(), Box<dyn std::error::Error>> {
    let context = TestContext::new().await?;
    
    // Store items in categories
    let categories = vec!["alpha", "beta", "gamma"];
    for (i, category) in categories.iter().enumerate() {
        for j in 0..3 {
            let content = PullTestContent {
                id: format!("{category}-{j}"),
                title: format!("{category} Item {j}"),
                data: vec![(i * 10 + j) as u8; 20],
                metadata: [
                    ("category".to_string(), category.to_string()),
                ].into_iter().collect(),
            };
            
            context.storage.put(&content).await?;
        }
    }
    
    // Pull and group by category
    let bucket = ContentBucket::for_content_type(PullTestContent::CONTENT_TYPE.codec());
    let grouped = context.storage.pull_and_group::<PullTestContent, String, _>(
        bucket,
        |content| {
            content.metadata.get("category")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string())
        }
    ).await?;
    
    // Verify groups
    for category in &categories {
        let group = grouped.get(*category);
        assert!(group.is_some(), "Category {} not found", category);
        
        let items = group.unwrap();
        println!("✓ Category '{category}' has {items.len(} items"));
    }
    
    Ok(())
}

/// Test helper functions
///
/// ```mermaid
/// graph TD
///     A[Pull Results] --> B[Filter]
///     B --> C[Sort]
///     C --> D[Extract]
///     D --> E[Map to CIDs]
/// ```
#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_pull_helpers() -> Result<(), Box<dyn std::error::Error>> {
    let context = TestContext::new().await?;
    
    // Store test data
    for i in 0..5 {
        let content = PullTestContent {
            id: format!("helper-{i}"),
            title: format!("Helper Test {i}"),
            data: vec![i as u8; 10],
            metadata: [
                ("priority".to_string(), (5 - i).to_string()),
            ].into_iter().collect(),
        };
        
        context.storage.put(&content).await?;
    }
    
    // Pull all
    let bucket = ContentBucket::for_content_type(PullTestContent::CONTENT_TYPE.codec());
    let results = context.storage.pull_all::<PullTestContent>(
        bucket, 
        PullOptions::default()
    ).await?;
    
    // Test filter
    let filtered = helpers::filter_by_content(results.clone(), |content| {
        content.id.contains("helper-")
    });
    assert!(!filtered.is_empty());
    
    // Test sort
    let sorted = helpers::sort_by_key(filtered, |content| {
        content.metadata.get("priority")
            .and_then(|p| p.parse::<u32>().ok())
            .unwrap_or(0)
    });
    
    // Verify sort order
    for i in 1..sorted.len() {
        let prev_priority = sorted[i-1].content.metadata.get("priority")
            .and_then(|p| p.parse::<u32>().ok())
            .unwrap_or(0);
        let curr_priority = sorted[i].content.metadata.get("priority")
            .and_then(|p| p.parse::<u32>().ok())
            .unwrap_or(0);
        
        assert!(prev_priority <= curr_priority, "Sort order incorrect");
    }
    
    // Test extract
    let contents = helpers::extract_content(sorted);
    assert!(!contents.is_empty());
    
    // Test CID map
    let cid_map = helpers::to_cid_map(results);
    assert!(!cid_map.is_empty());
    
    println!("✓ All helper functions working correctly");
    
    Ok(())
} 