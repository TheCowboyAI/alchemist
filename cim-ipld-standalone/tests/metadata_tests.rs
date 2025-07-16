//! Metadata tests for CIM-IPLD
//!
//! Tests metadata storage, retrieval, and search functionality.
//!
//! ## Test Flow Diagram
//!
//! ```mermaid
//! graph TD
//!     A[Metadata Tests] --> B[Storage & Retrieval]
//!     A --> C[Search Functionality]
//!
//!     B --> B1[Content with Metadata]
//!     B1 --> B2[Store]
//!     B2 --> B3[Retrieve]
//!     B3 --> B4[Metadata Preserved]
//!
//!     C --> C1[Searchable Metadata]
//!     C1 --> C2[Search Performed]
//!     C2 --> C3[Correct Results]
//! ```

mod common;
use common::*;
use common::assertions::*;

use anyhow::Result;
use cim_ipld::{
    object_store::ObjectStore,
    codec::ContentCodec,
    types::TypedContent,
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Extended content type with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContentWithMetadata {
    pub content: TestContent,
    pub metadata: ContentMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContentMetadata {
    pub created_at: DateTime<Utc>,
    pub author: String,
    pub tags: Vec<String>,
    pub properties: HashMap<String, String>,
    pub version: String,
    pub content_type: String,
}

impl TypedContent for ContentWithMetadata {
    const CONTENT_TYPE: &'static str = "test/content-with-metadata";
}

#[tokio::test]
async fn test_metadata_storage_retrieval() -> Result<()> {
    let context = TestContext::new().await?;

    // Given: Content with metadata
    let content = TestContent {
        id: "metadata-test-1".to_string(),
        data: "Content with rich metadata".to_string(),
        value: 42,
    };

    let metadata = ContentMetadata {
        created_at: Utc::now(),
        author: "test-author".to_string(),
        tags: vec!["test".to_string(), "metadata".to_string(), "ipld".to_string()],
        properties: {
            let mut props = HashMap::new();
            props.insert("category".to_string(), "test-category".to_string());
            props.insert("priority".to_string(), "high".to_string());
            props.insert("environment".to_string(), "development".to_string());
            props
        },
        version: "1.0.0".to_string(),
        content_type: "application/test".to_string(),
    };

    let content_with_metadata = ContentWithMetadata {
        content: content.clone(),
        metadata: metadata.clone(),
    };

    // When: Stored and retrieved
    let encoded = ContentCodec::encode(&content_with_metadata)?;
    let cid = context.storage.put(&encoded).await?;
    println!("Stored content with metadata: CID {cid}");

    let retrieved_bytes = context.storage.get(&cid).await?;
    let retrieved: ContentWithMetadata = ContentCodec::decode(&retrieved_bytes)?;

    // Then: Metadata preserved
    assert_eq!(retrieved.content, content);
    assert_eq!(retrieved.metadata.author, metadata.author);
    assert_eq!(retrieved.metadata.tags, metadata.tags);
    assert_eq!(retrieved.metadata.properties, metadata.properties);
    assert_eq!(retrieved.metadata.version, metadata.version);
    assert_eq!(retrieved.metadata.content_type, metadata.content_type);

    // Timestamps should be very close (within 1 second)
    let time_diff = retrieved.metadata.created_at
        .signed_duration_since(metadata.created_at)
        .num_seconds()
        .abs();
    assert!(time_diff < 1, "Timestamps should be preserved accurately");

    Ok(())
}

#[tokio::test]
async fn test_metadata_search() -> Result<()> {
    let context = TestContext::new().await?;

    // Given: Multiple content items with searchable metadata
    let test_items = vec![
        ("item-1", "First item", vec!["rust", "ipld"], "development"),
        ("item-2", "Second item", vec!["rust", "test"], "production"),
        ("item-3", "Third item", vec!["ipld", "test"], "development"),
        ("item-4", "Fourth item", vec!["metadata", "search"], "staging"),
    ];

    let mut stored_items = HashMap::new();

    for (id, data, tags, env) in test_items {
        let content = TestContent {
            id: id.to_string(),
            data: data.to_string(),
            value: 1,
        };

        let metadata = ContentMetadata {
            created_at: Utc::now(),
            author: "search-test".to_string(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            properties: {
                let mut props = HashMap::new();
                props.insert("environment".to_string(), env.to_string());
                props
            },
            version: "1.0.0".to_string(),
            content_type: "test/searchable".to_string(),
        };

        let item = ContentWithMetadata { content, metadata };
        let encoded = ContentCodec::encode(&item)?;
        let cid = context.storage.put(&encoded).await?;

        stored_items.insert(cid, item);
        println!("Stored {id} with tags {:?} in {tags}", env);
    }

    // When: Search performed (simulated - real implementation would have search API)
    // For now, we'll implement a simple in-memory search

    // Search by tag
    let search_tag = "rust";
    let mut found_by_tag = vec![];

    for (cid, item) in &stored_items {
        if item.metadata.tags.contains(&search_tag.to_string()) {
            found_by_tag.push((cid.clone(), item.clone()));
        }
    }

    // Then: Correct results returned
    assert_eq!(found_by_tag.len(), 2, "Should find 2 items with 'rust' tag");

    // Search by property
    let search_env = "development";
    let mut found_by_env = vec![];

    for (cid, item) in &stored_items {
        if let Some(env) = item.metadata.properties.get("environment") {
            if env == search_env {
                found_by_env.push((cid.clone(), item.clone()));
            }
        }
    }

    assert_eq!(found_by_env.len(), 2, "Should find 2 items in development environment");

    // Combined search (tag AND property)
    let mut found_combined = vec![];

    for (cid, item) in &stored_items {
        let has_tag = item.metadata.tags.contains(&"test".to_string());
        let has_env = item.metadata.properties.get("environment")
            .map(|e| e == "development")
            .unwrap_or(false);

        if has_tag && has_env {
            found_combined.push((cid.clone(), item.clone()));
        }
    }

    assert_eq!(found_combined.len(), 1, "Should find 1 item with 'test' tag in development");

    Ok(())
}

#[tokio::test]
async fn test_metadata_versioning() -> Result<()> {
    let context = TestContext::new().await?;

    // Test storing multiple versions with metadata
    let base_content = TestContent {
        id: "versioned-content".to_string(),
        data: "Original content".to_string(),
        value: 1,
    };

    let mut version_cids = vec![];

    // Create multiple versions
    for version in 1..=3 {
        let content = TestContent {
            id: base_content.id.clone(),
            data: format!("Content version {version}"),
            value: version as u64,
        };

        let metadata = ContentMetadata {
            created_at: Utc::now(),
            author: "version-test".to_string(),
            tags: vec!["versioned".to_string()],
            properties: {
                let mut props = HashMap::new();
                props.insert("previous_version".to_string(),
                    if version > 1 {
                        format!("{version - 1}")
                    } else {
                        "none".to_string()
                    }
                );
                props
            },
            version: format!("{version}.0.0"),
            content_type: "test/versioned".to_string(),
        };

        let versioned = ContentWithMetadata { content, metadata };
        let encoded = ContentCodec::encode(&versioned)?;
        let cid = context.storage.put(&encoded).await?;

        version_cids.push((version, cid));
        println!("Stored version {version} with CID {cid}");
    }

    // Verify all versions are retrievable
    for (version, cid) in &version_cids {
        let retrieved_bytes = context.storage.get(cid).await?;
        let retrieved: ContentWithMetadata = ContentCodec::decode(&retrieved_bytes)?;

        assert_eq!(retrieved.metadata.version, format!("{version}.0.0"));
        assert_eq!(retrieved.content.value, *version as u64);
    }

    Ok(())
}

#[tokio::test]
async fn test_large_metadata() -> Result<()> {
    let context = TestContext::new().await?;

    // Test with large metadata
    let content = TestContent {
        id: "large-metadata".to_string(),
        data: "Content with large metadata".to_string(),
        value: 100,
    };

    // Create large metadata
    let mut large_properties = HashMap::new();
    for i in 0..1000 {
        large_properties.insert(
            format!("property_{i}"),
            format!("Value for property {i} with some additional text to make it larger"),
        );
    }

    let mut large_tags = vec![];
    for i in 0..100 {
        large_tags.push(format!("tag_{i}"));
    }

    let metadata = ContentMetadata {
        created_at: Utc::now(),
        author: "large-metadata-test".to_string(),
        tags: large_tags,
        properties: large_properties,
        version: "1.0.0".to_string(),
        content_type: "test/large-metadata".to_string(),
    };

    let large_item = ContentWithMetadata { content, metadata };

    // Store and retrieve
    let encoded = ContentCodec::encode(&large_item)?;
    println!("Encoded size with large metadata: {} bytes", encoded.len());

    let cid = context.storage.put(&encoded).await?;
    let retrieved_bytes = context.storage.get(&cid).await?;
    let retrieved: ContentWithMetadata = ContentCodec::decode(&retrieved_bytes)?;

    // Verify all metadata preserved
    assert_eq!(retrieved.metadata.properties.len(), 1000);
    assert_eq!(retrieved.metadata.tags.len(), 100);

    // Spot check some values
    assert_eq!(
        retrieved.metadata.properties.get("property_500"),
        Some(&"Value for property 500 with some additional text to make it larger".to_string())
    );
    assert!(retrieved.metadata.tags.contains(&"tag_50".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_metadata_filtering() -> Result<()> {
    let context = TestContext::new().await?;

    // Store items with various metadata for filtering
    let items = vec![
        ("high-priority-dev", "high", "development", vec!["urgent", "bug"]),
        ("low-priority-dev", "low", "development", vec!["feature"]),
        ("high-priority-prod", "high", "production", vec!["urgent"]),
        ("medium-priority-staging", "medium", "staging", vec!["test"]),
    ];

    let mut stored = HashMap::new();

    for (id, priority, env, tags) in items {
        let content = TestContent {
            id: id.to_string(),
            data: format!("Content for {id}"),
            value: 1,
        };

        let metadata = ContentMetadata {
            created_at: Utc::now(),
            author: "filter-test".to_string(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            properties: {
                let mut props = HashMap::new();
                props.insert("priority".to_string(), priority.to_string());
                props.insert("environment".to_string(), env.to_string());
                props
            },
            version: "1.0.0".to_string(),
            content_type: "test/filterable".to_string(),
        };

        let item = ContentWithMetadata { content, metadata };
        let encoded = ContentCodec::encode(&item)?;
        let cid = context.storage.put(&encoded).await?;

        stored.insert(id, (cid, item));
    }

    // Filter by priority
    let high_priority: Vec<_> = stored.iter()
        .filter(|(_, (_, item))| {
            item.metadata.properties.get("priority") == Some(&"high".to_string())
        })
        .collect();

    assert_eq!(high_priority.len(), 2, "Should find 2 high priority items");

    // Filter by environment and tag
    let dev_urgent: Vec<_> = stored.iter()
        .filter(|(_, (_, item))| {
            let is_dev = item.metadata.properties.get("environment") == Some(&"development".to_string());
            let is_urgent = item.metadata.tags.contains(&"urgent".to_string());
            is_dev && is_urgent
        })
        .collect();

    assert_eq!(dev_urgent.len(), 1, "Should find 1 urgent item in development");

    Ok(())
}

#[tokio::test]
async fn test_metadata_update_pattern() -> Result<()> {
    let context = TestContext::new().await?;

    // Test pattern for updating metadata while preserving content
    let original_content = TestContent {
        id: "updatable".to_string(),
        data: "Content that won't change".to_string(),
        value: 42,
    };

    let original_metadata = ContentMetadata {
        created_at: Utc::now(),
        author: "original-author".to_string(),
        tags: vec!["original".to_string()],
        properties: HashMap::new(),
        version: "1.0.0".to_string(),
        content_type: "test/updatable".to_string(),
    };

    let original = ContentWithMetadata {
        content: original_content.clone(),
        metadata: original_metadata,
    };

    // Store original
    let encoded = ContentCodec::encode(&original)?;
    let original_cid = context.storage.put(&encoded).await?;
    println!("Original CID: {original_cid}");

    // Create updated version with new metadata
    let updated_metadata = ContentMetadata {
        created_at: Utc::now(),
        author: "updated-author".to_string(),
        tags: vec!["original".to_string(), "updated".to_string()],
        properties: {
            let mut props = HashMap::new();
            props.insert("updated_from".to_string(), original_cid.to_string());
            props.insert("update_reason".to_string(), "metadata correction".to_string());
            props
        },
        version: "1.0.1".to_string(),
        content_type: "test/updatable".to_string(),
    };

    let updated = ContentWithMetadata {
        content: original_content, // Same content
        metadata: updated_metadata,
    };

    // Store updated version
    let encoded = ContentCodec::encode(&updated)?;
    let updated_cid = context.storage.put(&encoded).await?;
    println!("Updated CID: {updated_cid}");

    // CIDs should be different (different metadata)
    assert_ne!(original_cid, updated_cid, "Different metadata should produce different CIDs");

    // Both versions should be retrievable
    let original_retrieved_bytes = context.storage.get(&original_cid).await?;
    let original_retrieved: ContentWithMetadata = ContentCodec::decode(&original_retrieved_bytes)?;
    assert_eq!(original_retrieved.metadata.version, "1.0.0");

    let updated_retrieved_bytes = context.storage.get(&updated_cid).await?;
    let updated_retrieved: ContentWithMetadata = ContentCodec::decode(&updated_retrieved_bytes)?;
    assert_eq!(updated_retrieved.metadata.version, "1.0.1");
    assert_eq!(
        updated_retrieved.metadata.properties.get("updated_from"),
        Some(&original_cid.to_string())
    );

    Ok(())
}

#[cfg(test)]
mod metadata_test_helpers {
    use super::*;

    /// Create test metadata with common defaults
    pub fn create_test_metadata(tags: Vec<&str>) -> ContentMetadata {
        ContentMetadata {
            created_at: Utc::now(),
            author: "test".to_string(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            properties: HashMap::new(),
            version: "1.0.0".to_string(),
            content_type: "test/default".to_string(),
        }
    }

    /// Verify metadata search results
    pub fn assert_search_results<T>(
        results: &[T],
        expected_count: usize,
        description: &str,
    ) {
        assert_eq!(
            results.len(),
            expected_count,
            "Search '{}' should return {} results, got {}",
            description,
            expected_count,
            results.len()
        );
    }
}
