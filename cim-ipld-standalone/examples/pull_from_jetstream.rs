//! Example: Pulling CIDs from NATS JetStream
//!
//! This example demonstrates how to:
//! 1. List available CIDs in a bucket
//! 2. Retrieve content by CID
//! 3. Verify CID integrity
//! 4. Handle different content types

use cim_ipld::{
    object_store::{NatsObjectStore, ContentBucket},
    TypedContent, ContentType, Cid,
};
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Example content type for demonstration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ExampleDocument {
    title: String,
    content: String,
    tags: Vec<String>,
    version: u32,
}

impl TypedContent for ExampleDocument {
    const CODEC: u64 = 0x400000; // Example codec
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x400000);
}

/// Example event type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ExampleEvent {
    event_id: String,
    event_type: String,
    timestamp: u64,
    data: serde_json::Value,
}

impl TypedContent for ExampleEvent {
    const CODEC: u64 = 0x300105; // Events codec
    const CONTENT_TYPE: ContentType = ContentType::Event;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to NATS
    println!("Connecting to NATS...");
    let client = async_nats::connect("nats://localhost:4222").await?;
    let jetstream = async_nats::jetstream::new(client);

    // Create object store wrapper
    let object_store = NatsObjectStore::new(jetstream, 1024).await?;

    // First, let's store some example content to retrieve later
    println!("\n=== Storing Example Content ===");
    
    // Store a document
    let doc = ExampleDocument {
        title: "CIM Architecture Guide".to_string(),
        content: "This document describes the CIM architecture...".to_string(),
        tags: vec!["architecture".to_string(), "cim".to_string(), "guide".to_string()],
        version: 1,
    };
    
    let doc_cid = object_store.put(&doc).await?;
    println!("Stored document with CID: {doc_cid}");

    // Store an event
    let event = ExampleEvent {
        event_id: "evt-001".to_string(),
        event_type: "system.started".to_string(),
        timestamp: 1234567890,
        data: serde_json::json!({
            "service": "graph-processor",
            "version": "0.1.0"
        }),
    };
    
    let event_cid = object_store.put(&event).await?;
    println!("Stored event with CID: {event_cid}");

    // Now demonstrate pulling from JetStream
    println!("\n=== Pulling from JetStream ===");

    // Method 1: List all CIDs in a bucket
    println!("\n1. Listing all CIDs in Documents bucket:");
    let document_objects = object_store.list(ContentBucket::Documents).await?;
    
    for obj in &document_objects {
        println!("  - CID: {obj.cid}");
        println!("    Size: {obj.size} bytes");
        println!("    Compressed: {obj.compressed}");
    }

    println!("\n2. Listing all CIDs in Events bucket:");
    let event_objects = object_store.list(ContentBucket::Events).await?;
    
    for obj in &event_objects {
        println!("  - CID: {obj.cid}");
        println!("    Size: {obj.size} bytes");
    }

    // Method 2: Pull specific content by CID
    println!("\n3. Retrieving specific content by CID:");
    
    // Retrieve the document
    println!("\nRetrieving document with CID: {doc_cid}");
    let retrieved_doc: ExampleDocument = object_store.get(&doc_cid).await?;
    
    println!("Retrieved document:");
    println!("  Title: {retrieved_doc.title}");
    println!("  Content: {retrieved_doc.content}");
    println!("  Tags: {:?}", retrieved_doc.tags);
    println!("  Version: {retrieved_doc.version}");
    
    // Verify it matches
    assert_eq!(retrieved_doc, doc);
    println!("✓ Document integrity verified!");

    // Retrieve the event
    println!("\nRetrieving event with CID: {event_cid}");
    let retrieved_event: ExampleEvent = object_store.get(&event_cid).await?;
    
    println!("Retrieved event:");
    println!("  Event ID: {retrieved_event.event_id}");
    println!("  Type: {retrieved_event.event_type}");
    println!("  Timestamp: {retrieved_event.timestamp}");
    println!("  Data: {serde_json::to_string_pretty(&retrieved_event.data}")?);
    
    assert_eq!(retrieved_event, event);
    println!("✓ Event integrity verified!");

    // Method 3: Check if a CID exists
    println!("\n4. Checking CID existence:");
    
    let exists = object_store.exists(&doc_cid, ExampleDocument::CONTENT_TYPE.codec()).await?;
    println!("Document CID {doc_cid} exists: {exists}");
    
    // Try a non-existent CID
    let fake_cid = Cid::try_from("bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku")?;
    let fake_exists = object_store.exists(&fake_cid, ExampleDocument::CONTENT_TYPE.codec()).await?;
    println!("Fake CID {fake_cid} exists: {fake_exists}");

    // Method 4: Pull from a specific bucket
    println!("\n5. Pulling from specific buckets:");
    
    // Helper function to pull and display content from a bucket
    async fn pull_from_bucket(
        store: &NatsObjectStore,
        bucket: ContentBucket,
    ) -> Result<(), Box<dyn Error>> {
        println!("\nBucket: {:?} ({bucket})", bucket.as_str());
        
        let objects = store.list(bucket).await?;
        
        if objects.is_empty() {
            println!("  No objects found");
        } else {
            println!("  Found {objects.len(} objects:"));
            for (i, obj) in objects.iter().enumerate().take(5) {
                println!("    {i + 1}. CID: {obj.cid} ({obj.size} bytes)");
            }
            
            if objects.len() > 5 {
                println!("    ... and {objects.len(} more") - 5);
            }
        }
        
        Ok(())
    }

    // Check multiple buckets
    for bucket in &[
        ContentBucket::Events,
        ContentBucket::Graphs,
        ContentBucket::Nodes,
        ContentBucket::Documents,
    ] {
        pull_from_bucket(&object_store, *bucket).await?;
    }

    // Method 5: Error handling when pulling
    println!("\n6. Error handling:");
    
    // Try to retrieve with wrong type
    println!("\nTrying to retrieve document CID as event type:");
    match object_store.get::<ExampleEvent>(&doc_cid).await {
        Ok(_) => println!("Unexpectedly succeeded!"),
        Err(e) => println!("Expected error: {e}"),
    }

    // Try to retrieve non-existent CID
    println!("\nTrying to retrieve non-existent CID:");
    match object_store.get::<ExampleDocument>(&fake_cid).await {
        Ok(_) => println!("Unexpectedly found fake CID!"),
        Err(e) => println!("Expected error: {e}"),
    }

    // Advanced: Pull and process in batches
    println!("\n7. Batch processing example:");
    
    // Store multiple documents
    let mut stored_cids = Vec::new();
    for i in 0..5 {
        let batch_doc = ExampleDocument {
            title: format!("Document {i}"),
            content: format!("Content for document {i}"),
            tags: vec![format!("tag{i}")],
            version: 1,
        };
        
        let cid = object_store.put(&batch_doc).await?;
        stored_cids.push(cid);
    }
    
    println!("Stored {stored_cids.len(} documents"));
    
    // Pull them back in parallel
    use futures::future::join_all;
    
    let pull_futures = stored_cids.iter().map(|cid| {
        let store = &object_store;
        async move {
            store.get::<ExampleDocument>(cid).await
        }
    });
    
    let results = join_all(pull_futures).await;
    
    let mut success_count = 0;
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(doc) => {
                println!("  ✓ Retrieved: {doc.title}");
                success_count += 1;
            }
            Err(e) => {
                println!("  ✗ Failed to retrieve document {i}: {e}");
            }
        }
    }
    
    println!("Successfully retrieved {success_count}/{stored_cids.len(} documents"));

    println!("\n=== Example Complete ===");
    
    Ok(())
}

/// Helper function to demonstrate pulling with metadata
async fn pull_with_metadata(
    store: &NatsObjectStore,
    cid: &Cid,
) -> Result<(ExampleDocument, Cid), Box<dyn Error>> {
    // Pull the content
    let content: ExampleDocument = store.get(cid).await?;
    
    // Recalculate CID to verify
    let calculated_cid = content.calculate_cid()?;
    
    // Ensure CID matches
    if calculated_cid != *cid {
        return Err("CID mismatch!".into());
    }
    
    Ok((content, calculated_cid))
}

/// Example of pulling and transforming content
async fn pull_and_transform(
    store: &NatsObjectStore,
    cid: &Cid,
) -> Result<serde_json::Value, Box<dyn Error>> {
    // Pull as document
    let doc: ExampleDocument = store.get(cid).await?;
    
    // Transform to JSON
    let json = serde_json::json!({
        "cid": cid.to_string(),
        "title": doc.title,
        "content": doc.content,
        "tags": doc.tags,
        "version": doc.version,
        "retrieved_at": chrono::Utc::now().to_rfc3339(),
    });
    
    Ok(json)
} 