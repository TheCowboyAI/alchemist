//! Integration tests for NATS Object Store
//!
//! These tests require a running NATS server with JetStream enabled.
//! Run with: nats-server -js

use ia::infrastructure::object_store::{NatsObjectStore, ContentStorageService, ContentBucket};
use ia::domain::content_types::{GraphContent, NodeIPLDContent, EdgeIPLDContent};
use ia::edge_content::EdgeIPLDType;
use ia::domain::value_objects::{GraphId, NodeId, EdgeId, Position3D, GraphMetadata};
use async_nats::jetstream;
use cid::Cid;
use cim_ipld::TypedContent;
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use tokio;

/// Helper to connect to NATS for testing
async fn setup_test_nats() -> Result<(async_nats::Client, jetstream::Context), Box<dyn std::error::Error>> {
    // Try to connect to local NATS server
    let client = async_nats::connect("nats://localhost:4222").await?;
    let jetstream = jetstream::new(client.clone());

    Ok((client, jetstream))
}

/// Helper to create a unique test bucket name
fn test_bucket_name(base: &str) -> String {
    format!("test-{}-{}", base, uuid::Uuid::new_v4().to_string().split('-').next().unwrap())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_object_store_basic_operations() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = setup_test_nats().await?;

    // Create object store
    let object_store = NatsObjectStore::new(jetstream, 1024).await?;

    // Create test content
    let mut metadata = GraphMetadata::default();
    metadata.name = "test-graph".to_string();
    let graph = GraphContent::new(
        GraphId::new(),
        metadata.clone(),
    );

    // Store content
    let cid = object_store.put(&graph).await?;
    println!("Stored graph with CID: {}", cid);

    // Retrieve content
    let retrieved: GraphContent = object_store.get(&cid).await?;
    assert_eq!(retrieved.metadata.name, "test-graph");
    assert_eq!(retrieved.metadata, metadata);

    // Check existence
    let exists = object_store.exists(&cid, GraphContent::CONTENT_TYPE.codec()).await?;
    assert!(exists);

    // Delete content
    object_store.delete(&cid, GraphContent::CONTENT_TYPE.codec()).await?;

    // Verify deletion
    let exists_after = object_store.exists(&cid, GraphContent::CONTENT_TYPE.codec()).await?;
    assert!(!exists_after);

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_compression_threshold() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = setup_test_nats().await?;

    // Create object store with 100 byte compression threshold
    let object_store = NatsObjectStore::new(jetstream, 100).await?;

    // Create small content (won't be compressed)
    let small_node = NodeIPLDContent::new(
        NodeId::new(),
        "small".to_string(),
        Position3D::default(),
    );

    // Create large content (will be compressed)
    let mut large_node = NodeIPLDContent::new(
        NodeId::new(),
        "large".to_string(),
        Position3D::default(),
    );

    // Add properties to make it large
    for i in 0..100 {
        large_node.properties.insert(
            format!("prop_{}", i),
            serde_json::Value::String("x".repeat(50)),
        );
    }

    // Store both
    let small_cid = object_store.put(&small_node).await?;
    let large_cid = object_store.put(&large_node).await?;

    println!("Small node CID: {}", small_cid);
    println!("Large node CID: {}", large_cid);

    // Retrieve and verify
    let retrieved_small: NodeIPLDContent = object_store.get(&small_cid).await?;
    let retrieved_large: NodeIPLDContent = object_store.get(&large_cid).await?;

    assert_eq!(retrieved_small.label, "small");
    assert_eq!(retrieved_large.label, "large");
    assert_eq!(retrieved_large.properties.len(), 100);

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_content_storage_service() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = setup_test_nats().await?;

    // Create object store and storage service
    let object_store = Arc::new(NatsObjectStore::new(jetstream, 1024).await?);
    let storage_service = ContentStorageService::new(
        object_store,
        10,  // Small cache for testing
        Duration::from_secs(60),
        1024 * 1024,  // 1MB cache size
    );

    // Create test content
    let edge = EdgeIPLDContent::new(
        EdgeId::new(),
        NodeId::new(),
        NodeId::new(),
        "test-edge".to_string(),
    );

    // Store content
    let cid = storage_service.store(&edge).await?;
    println!("Stored edge with CID: {}", cid);

    // First retrieval (from object store)
    let retrieved1: EdgeIPLDContent = storage_service.get(&cid).await?;
    assert_eq!(retrieved1.label, "test-edge");
    assert_eq!(retrieved1.edge_type, EdgeIPLDType::Directed);

    // Second retrieval (should be from cache)
    let retrieved2: EdgeIPLDContent = storage_service.get(&cid).await?;
    assert_eq!(retrieved2.label, "test-edge");
    assert_eq!(retrieved2.edge_type, EdgeIPLDType::Directed);

    // Check cache stats
    let stats = storage_service.cache_stats().await;
    assert_eq!(stats.entries, 1);
    assert!(stats.size > 0);

    // Test deduplication - storing same content should return same CID
    let cid2 = storage_service.store(&edge).await?;
    assert_eq!(cid, cid2);

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_bucket_management() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = setup_test_nats().await?;

    // Create object store
    let object_store = NatsObjectStore::new(jetstream, 1024).await?;

    // List objects in graphs bucket (should be empty initially)
    let objects = object_store.list(ContentBucket::Graphs).await?;
    println!("Initial objects in Graphs bucket: {}", objects.len());

    // Store multiple graphs
    for i in 0..5 {
        let mut metadata = GraphMetadata::default();
        metadata.name = format!("graph-{}", i);
        let graph = GraphContent::new(
            GraphId::new(),
            metadata,
        );
        object_store.put(&graph).await?;
    }

    // List again
    let objects = object_store.list(ContentBucket::Graphs).await?;
    assert!(objects.len() >= 5);

    // Get bucket stats
    let stats = object_store.stats(ContentBucket::Graphs).await?;
    println!("Bucket stats: {:?}", stats);
    assert_eq!(stats.bucket_name, "cim-graphs");

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_cid_integrity_verification() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = setup_test_nats().await?;

    // Create object store
    let object_store = NatsObjectStore::new(jetstream, 1024).await?;

    // Create test content
    let node = NodeIPLDContent::new(
        NodeId::new(),
        "integrity-test".to_string(),
        Position3D::default(),
    );

    // Store content
    let _cid = object_store.put(&node).await?;

    // Try to retrieve with wrong CID (should fail)
    let wrong_cid = Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap();

    match object_store.get::<NodeIPLDContent>(&wrong_cid).await {
        Err(e) => {
            println!("Expected error: {}", e);
            assert!(e.to_string().contains("NotFound"));
        }
        Ok(_) => panic!("Should have failed with wrong CID"),
    }

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_batch_operations() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = setup_test_nats().await?;

    // Create storage service
    let object_store = Arc::new(NatsObjectStore::new(jetstream, 1024).await?);
    let storage_service = ContentStorageService::new(
        object_store,
        100,
        Duration::from_secs(300),
        10 * 1024 * 1024,
    );

    // Create multiple nodes
    let nodes: Vec<NodeIPLDContent> = (0..10)
        .map(|i| NodeIPLDContent::new(
            NodeId::new(),
            format!("batch-node-{}", i),
            Position3D::default(),
        ))
        .collect();

    // Store batch
    let cids = storage_service.store_batch(&nodes).await?;
    assert_eq!(cids.len(), 10);

    // Retrieve batch
    let retrieved = storage_service.get_batch::<NodeIPLDContent>(&cids).await?;
    assert_eq!(retrieved.len(), 10);

    // Verify content
    for (i, node) in retrieved.iter().enumerate() {
        assert_eq!(node.label, format!("batch-node-{}", i));
    }

    Ok(())
}

/// Helper function to clean up test buckets
async fn cleanup_test_buckets(jetstream: &jetstream::Context) -> Result<(), Box<dyn std::error::Error>> {
    // For now, just clean up known buckets
    // async-nats 0.41 doesn't have a simple way to list all object stores
    let buckets = vec![
        "cim-events",
        "cim-graphs",
        "cim-nodes",
        "cim-edges",
        "cim-conceptual",
        "cim-workflows",
        "cim-media",
        "cim-documents",
    ];

    for bucket in buckets {
        match jetstream.delete_object_store(bucket).await {
            Ok(_) => println!("Cleaned up bucket: {}", bucket),
            Err(_) => {} // Ignore if doesn't exist
        }
    }

    Ok(())
}
