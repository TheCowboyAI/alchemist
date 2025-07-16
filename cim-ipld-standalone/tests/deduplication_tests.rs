//! Deduplication tests for CIM-IPLD
//!
//! Tests content deduplication and cross-bucket deduplication strategies.
//!
//! ## Test Flow Diagram
//!
//! ```mermaid
//! graph TD
//!     A[Deduplication Tests] --> B[Content Deduplication]
//!     A --> C[Cross-Bucket Dedup]
//!
//!     B --> B1[Identical Content]
//!     B1 --> B2[Store Multiple Times]
//!     B2 --> B3[Single Storage]
//!     B3 --> B4[Multiple References]
//!
//!     C --> C1[Same Content]
//!     C1 --> C2[Different Buckets]
//!     C2 --> C3[Dedup Strategy Applied]
//! ```

mod common;

use common::*;
use common::assertions::*;

use anyhow::Result;
use cim_ipld::{
    object_store::{ObjectStore, NatsObjectStore},
    codec::ContentCodec,
    types::TypedContent,
};
use std::sync::Arc;
use std::collections::HashMap;

#[tokio::test]
async fn test_content_deduplication() -> Result<()> {
    let context = TestContext::new().await?;

    // Given: Identical content
    let content = TestContent {
        id: "dedup-test-1".to_string(),
        data: "This content will be stored multiple times".to_string(),
        value: 42,
    };

    // When: Stored multiple times
    let mut cids = vec![];
    for i in 0..5 {
        println!("Storing content attempt {i + 1}");
        let cid = context.with_content(content.clone()).await?;
        cids.push(cid);
    }

    // Then: All CIDs should be identical (deduplication working)
    let first_cid = &cids[0];
    for (i, cid) in cids.iter().enumerate() {
        assert_cids_equal(first_cid, cid);
        println!("Store attempt {i + 1} resulted in CID: {cid}");
    }

    // Verify content is stored only once
    // In a real implementation, we would check storage metrics
    // For now, verify retrieval works
    let retrieved_bytes = context.storage.get(first_cid).await?;
    let retrieved: TestContent = ContentCodec::decode(&retrieved_bytes)?;
    assert_content_equal(&content, &retrieved);

    Ok(())
}

#[tokio::test]
async fn test_dedup_across_buckets() -> Result<()> {
    let context = TestContext::new().await?;

    // Given: Same content, different buckets
    let content = TestContent {
        id: "cross-bucket-dedup".to_string(),
        data: "Content to be stored in multiple buckets".to_string(),
        value: 100,
    };

    // Create multiple buckets
    let bucket_names = vec!["bucket-a", "bucket-b", "bucket-c"];
    let mut bucket_stores = HashMap::new();

    for bucket_name in &bucket_names {
        let store = Arc::new(
            NatsObjectStore::new(
                context.nats.client.clone(),
                bucket_name.to_string(),
            ).await?
        );
        bucket_stores.insert(bucket_name.to_string(), store);
    }

    // When: Stored in each bucket
    let mut bucket_cids = HashMap::new();

    for (bucket_name, store) in &bucket_stores {
        let encoded = ContentCodec::encode(&content)?;
        let cid = store.put(&encoded).await?;
        bucket_cids.insert(bucket_name.clone(), cid);
        println!("Stored in {bucket_name} with CID: {cid}");
    }

    // Then: CIDs should be identical across buckets
    // (content-addressed storage produces same CID for same content)
    let cids: Vec<_> = bucket_cids.values().collect();
    let first_cid = cids[0];

    for cid in &cids[1..] {
        assert_cids_equal(first_cid, cid);
    }

    // Verify content can be retrieved from any bucket
    for (bucket_name, store) in &bucket_stores {
        let retrieved_bytes = store.get(&bucket_cids[bucket_name]).await?;
        let retrieved: TestContent = ContentCodec::decode(&retrieved_bytes)?;
        assert_content_equal(&content, &retrieved);
        println!("Successfully retrieved from {bucket_name}");
    }

    Ok(())
}

#[tokio::test]
async fn test_dedup_with_different_metadata() -> Result<()> {
    let context = TestContext::new().await?;

    // Test that content with same data but different metadata
    // still deduplicates correctly

    let base_data = "Deduplication test data";

    // Create content with different IDs but same data
    let content1 = TestContent {
        id: "metadata-1".to_string(),
        data: base_data.to_string(),
        value: 42,
    };

    let content2 = TestContent {
        id: "metadata-2".to_string(),
        data: base_data.to_string(),
        value: 42,
    };

    let cid1 = context.with_content(content1.clone()).await?;
    let cid2 = context.with_content(content2.clone()).await?;

    // Different metadata means different CIDs
    assert_ne!(cid1, cid2, "Different metadata should produce different CIDs");

    // But same exact content produces same CID
    let content3 = TestContent {
        id: "metadata-1".to_string(), // Same as content1
        data: base_data.to_string(),
        value: 42,
    };

    let cid3 = context.with_content(content3).await?;
    assert_cids_equal(&cid1, &cid3);

    Ok(())
}

#[tokio::test]
async fn test_dedup_statistics() -> Result<()> {
    let context = TestContext::new().await?;

    // Track deduplication statistics
    let mut unique_content = 0;
    let mut duplicate_stores = 0;
    let mut cid_map = HashMap::new();

    // Store various content, some duplicates
    let test_data = vec![
        ("unique-1", "Unique content 1", 1),
        ("unique-2", "Unique content 2", 2),
        ("duplicate-1", "Duplicate content", 3),
        ("duplicate-2", "Duplicate content", 3), // Same as duplicate-1
        ("unique-3", "Unique content 3", 4),
        ("duplicate-3", "Duplicate content", 3), // Another duplicate
    ];

    for (id, data, value) in test_data {
        let content = TestContent {
            id: id.to_string(),
            data: data.to_string(),
            value,
        };

        let cid = context.with_content(content).await?;

        if cid_map.contains_key(&cid) {
            duplicate_stores += 1;
            println!("Duplicate detected for {id}: CID {cid}");
        } else {
            unique_content += 1;
            cid_map.insert(cid.clone(), id);
            println!("New content stored for {id}: CID {cid}");
        }
    }

    // Verify statistics
    assert_eq!(unique_content, 5, "Should have 5 unique content items");
    assert_eq!(duplicate_stores, 1, "Should have 1 duplicate store");

    // Verify all content is retrievable
    for (cid, original_id) in &cid_map {
        let retrieved_bytes = context.storage.get(cid).await?;
        let _: TestContent = ContentCodec::decode(&retrieved_bytes)?;
        println!("Retrieved content for original ID: {original_id}");
    }

    Ok(())
}

#[tokio::test]
async fn test_large_content_deduplication() -> Result<()> {
    let context = TestContext::new().await?;

    // Test deduplication with large content
    let sizes = vec![
        1024 * 100,      // 100 KB
        1024 * 1024,     // 1 MB
        1024 * 1024 * 5, // 5 MB
    ];

    for size in sizes {
        println!("Testing deduplication for {size} bytes");

        // Generate deterministic large content
        let large_data = vec![0x42u8; size]; // Fill with 'B'
        let content = TestContent {
            id: format!("large-{size}"),
            data: base64::encode(&large_data),
            value: size as u64,
        };

        // Store multiple times
        let mut cids = vec![];
        for i in 0..3 {
            let start = std::time::Instant::now();
            let cid = context.with_content(content.clone()).await?;
            let duration = start.elapsed();

            cids.push(cid);
            println!("  Attempt {i + 1}: CID {cid} (took {:?})", duration);
        }

        // All should be deduplicated
        let first_cid = &cids[0];
        for cid in &cids[1..] {
            assert_cids_equal(first_cid, cid);
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_deduplication() -> Result<()> {
    let context = Arc::new(TestContext::new().await?);

    // Test that concurrent stores of same content deduplicate correctly
    let content = TestContent {
        id: "concurrent-dedup".to_string(),
        data: "Content for concurrent deduplication testing".to_string(),
        value: 777,
    };

    let num_concurrent = 10;
    let barrier = Arc::new(tokio::sync::Barrier::new(num_concurrent));

    let mut handles = vec![];

    for i in 0..num_concurrent {
        let context_clone = context.clone();
        let content_clone = content.clone();
        let barrier_clone = barrier.clone();

        let handle = tokio::spawn(async move {
            // Synchronize start
            barrier_clone.wait().await;

            let cid = context_clone.with_content(content_clone).await?;
            Ok::<_, anyhow::Error>((i, cid))
        });

        handles.push(handle);
    }

    // Collect results
    let results: Vec<(usize, cid::Cid)> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap().unwrap())
        .collect();

    // All should have same CID
    let first_cid = &results[0].1;
    for (thread_id, cid) in &results {
        assert_cids_equal(first_cid, cid);
        println!("Thread {thread_id} got CID: {cid}");
    }

    Ok(())
}

#[tokio::test]
async fn test_dedup_with_compression() -> Result<()> {
    let context = TestContext::new().await?;

    // Test that compressed content still deduplicates correctly
    let original_data = "This is test data that will be compressed. ".repeat(100);

    let content = TestContent {
        id: "compression-dedup".to_string(),
        data: original_data,
        value: 999,
    };

    // Store multiple times (compression should happen internally)
    let mut cids = vec![];
    for i in 0..3 {
        let cid = context.with_content(content.clone()).await?;
        cids.push(cid);
        println!("Store {i + 1} resulted in CID: {cid}");
    }

    // Verify deduplication
    let first_cid = &cids[0];
    for cid in &cids[1..] {
        assert_cids_equal(first_cid, cid);
    }

    // Verify content integrity after compression/decompression
    let retrieved_bytes = context.storage.get(first_cid).await?;
    let retrieved: TestContent = ContentCodec::decode(&retrieved_bytes)?;
    assert_content_equal(&content, &retrieved);

    Ok(())
}

#[cfg(test)]
mod dedup_test_helpers {
    use super::*;

    /// Generate content with specific patterns for dedup testing
    pub fn generate_patterned_content(pattern: &str, size: usize) -> String {
        pattern.repeat(size / pattern.len())
    }

    /// Calculate deduplication ratio
    pub fn calculate_dedup_ratio(
        total_stores: usize,
        unique_content: usize,
    ) -> f64 {
        if total_stores == 0 {
            return 0.0;
        }

        let duplicates = total_stores - unique_content;
        (duplicates as f64) / (total_stores as f64) * 100.0
    }

    /// Verify deduplication effectiveness
    pub fn assert_dedup_effective(
        total_stores: usize,
        unique_content: usize,
        min_ratio: f64,
    ) {
        let ratio = calculate_dedup_ratio(total_stores, unique_content);
        assert!(
            ratio >= min_ratio,
            "Deduplication ratio {:.2}% is below minimum {:.2}%",
            ratio,
            min_ratio
        );
    }
}
