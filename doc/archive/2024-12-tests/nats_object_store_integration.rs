//! Integration tests for NATS Object Store
//!
//! These tests require a running NATS server with JetStream enabled.
//! Run with: nats-server -js

use async_nats::jetstream;
use cid::Cid;
use cim_ipld::TypedContent;
use ia::domain::content_types::GraphContent;
use ia::domain::value_objects::{EdgeId, GraphId, GraphMetadata, NodeId, Position3D};
use ia::infrastructure::object_store::{ContentBucket, ContentStorageService, NatsObjectStore};
use std::sync::Arc;
use std::time::Duration;

/// Helper to connect to NATS for testing
async fn connect_nats()
-> Result<(async_nats::Client, jetstream::Context), Box<dyn std::error::Error>> {
    let client = async_nats::connect("nats://localhost:4222").await?;
    let jetstream = jetstream::new(client.clone());
    Ok((client, jetstream))
}

/// Helper to create a unique test bucket name
#[allow(dead_code)]
fn test_bucket_name(base: &str) -> String {
    format!(
        "test-{}-{}",
        base,
        uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
    )
}

#[tokio::test]
async fn test_store_and_retrieve_graph() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = connect_nats().await?;
    let object_store = Arc::new(NatsObjectStore::new(jetstream, 1024).await?);

    // Create a test graph
    let graph = GraphContent {
        id: GraphId::new(),
        metadata: GraphMetadata {
            name: "test-graph".to_string(),
            ..Default::default()
        },
        nodes: vec![],
        edges: Default::default(),
        conceptual_position: None,
    };

    // Store the graph
    let cid = object_store.put(&graph).await?;
    println!("Stored graph with CID: {cid}");

    // Retrieve the graph
    let retrieved: GraphContent = object_store.get(&cid).await?;
    assert_eq!(retrieved.id, graph.id);
    assert_eq!(retrieved.metadata.name, graph.metadata.name);

    // Check existence
    assert!(
        object_store
            .exists(&cid, GraphContent::CONTENT_TYPE.codec())
            .await?
    );

    Ok(())
}

// NOTE: This test has been commented out because NodeIPLDContent has been removed
// #[tokio::test]
// #[ignore = "requires running NATS server"]
// async fn test_compression_threshold() -> Result<(), Box<dyn std::error::Error>> {
//     // Test removed - NodeIPLDContent no longer exists
//     Ok(())
// }

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_content_storage_service() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = connect_nats().await?;

    // Create object store and storage service
    let object_store = Arc::new(NatsObjectStore::new(jetstream, 1024).await?);
    let storage_service = ContentStorageService::new(
        object_store,
        10, // Small cache for testing
        Duration::from_secs(60),
        1024 * 1024, // 1MB cache size
    );

    // NOTE: EdgeIPLDContent has been removed - this test needs updating
    // Create test content using GraphContent instead
    let graph = GraphContent {
        id: GraphId::new(),
        metadata: GraphMetadata {
            name: "test-graph-for-cache".to_string(),
            ..Default::default()
        },
        nodes: vec![],
        edges: Default::default(),
        conceptual_position: None,
    };

    // Store content
    let cid = storage_service.store(&graph).await?;
    println!("Stored graph with CID: {cid}");

    // First retrieval (from object store)
    let retrieved1: GraphContent = storage_service.get(&cid).await?;
    assert_eq!(retrieved1.metadata.name, "test-graph-for-cache");

    // Second retrieval (should be from cache)
    let retrieved2: GraphContent = storage_service.get(&cid).await?;
    assert_eq!(retrieved2.metadata.name, "test-graph-for-cache");

    // Check cache stats
    let stats = storage_service.cache_stats().await;
    assert_eq!(stats.entries, 1);
    assert!(stats.size > 0);

    // Test deduplication - storing same content should return same CID
    let cid2 = storage_service.store(&graph).await?;
    assert_eq!(cid, cid2);

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_bucket_management() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = connect_nats().await?;

    // Create object store
    let object_store = NatsObjectStore::new(jetstream, 1024).await?;

    // List objects in graphs bucket (should be empty initially)
    let objects = object_store.list(ContentBucket::Graphs).await?;
    println!("Initial objects in Graphs bucket: {}", objects.len());

    // Store multiple graphs
    for i in 0..5 {
        let metadata = GraphMetadata {
            name: format!("graph-{i}"),
            ..Default::default()
        };
        let graph = GraphContent::new(GraphId::new(), metadata);
        object_store.put(&graph).await?;
    }

    // List again
    let objects = object_store.list(ContentBucket::Graphs).await?;
    assert!(objects.len() >= 5);

    // Get bucket stats
    let stats = object_store.stats(ContentBucket::Graphs).await?;
    println!("Bucket stats: {stats:?}");
    assert_eq!(stats.bucket_name, "cim-graphs");

    Ok(())
}

// NOTE: This test has been commented out because NodeIPLDContent has been removed
// #[tokio::test]
// #[ignore = "requires running NATS server"]
// async fn test_cid_integrity_verification() -> Result<(), Box<dyn std::error::Error>> {
//     // Test removed - NodeIPLDContent no longer exists
//     Ok(())
// }

// NOTE: This test has been commented out because NodeIPLDContent has been removed
// #[tokio::test]
// #[ignore = "requires running NATS server"]
// async fn test_batch_operations() -> Result<(), Box<dyn std::error::Error>> {
//     // Test removed - NodeIPLDContent no longer exists
//     Ok(())
// }

/// Helper function to clean up test buckets
#[allow(dead_code)]
async fn cleanup_test_buckets(
    jetstream: &jetstream::Context,
) -> Result<(), Box<dyn std::error::Error>> {
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
        if let Ok(_) = jetstream.delete_object_store(bucket).await {
            println!("Cleaned up bucket: {bucket}");
        }
    }

    Ok(())
}
