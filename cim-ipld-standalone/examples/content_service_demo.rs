//! Comprehensive content service demonstration
//!
//! This example shows how to use the content service for:
//! - Storing various content types with metadata
//! - Searching content by text and tags
//! - Transforming content between formats
//! - Batch operations
//! - Lifecycle hooks

use cim_ipld::{
    content_types::{
        service::{ContentService, ContentServiceConfig},
        indexing::SearchQuery,
        DocumentMetadata, ImageMetadata,
    },
    object_store::NatsObjectStore,
    ContentType,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CIM-IPLD Content Service Demo ===\n");

    // Connect to NATS
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    println!("Connecting to NATS at {nats_url}");
    
    let client = async_nats::connect(&nats_url).await?;
    let jetstream = async_nats::jetstream::new(client);
    
    // Create object store
    let object_store = Arc::new(NatsObjectStore::new(jetstream.clone()).await?);
    
    // Configure content service
    let mut config = ContentServiceConfig::default();
    config.auto_index = true;
    config.validate_on_store = true;
    config.max_content_size = 10 * 1024 * 1024; // 10MB
    
    // Create content service
    let service = ContentService::new(object_store, config);
    
    // Add lifecycle hooks
    service.add_post_store_hook(|cid, content_type| {
        println!("✓ Stored content: {cid} (type: {:?})", content_type);
    }).await;
    
    // Demo 1: Store documents with metadata
    println!("\n1. Storing Documents");
    println!("-------------------");
    
    // Store a markdown document
    let md_content = r#"# Project Documentation

This is a sample markdown document demonstrating the content service.

## Features
- Automatic indexing
- Content deduplication
- Metadata preservation

## Code Example
```rust
let service = ContentService::new(storage, config);
let result = service.store_document(data, metadata, "markdown").await?;
```
"#;
    
    let md_metadata = DocumentMetadata {
        title: Some("Project Documentation".to_string()),
        author: Some("CIM Team".to_string()),
        tags: vec!["documentation".to_string(), "example".to_string(), "rust".to_string()],
        ..Default::default()
    };
    
    let md_result = service.store_document(
        md_content.as_bytes().to_vec(),
        md_metadata,
        "markdown"
    ).await?;
    
    println!("Markdown document stored:");
    println!("  CID: {md_result.cid}");
    println!("  Size: {md_result.size} bytes");
    println!("  Deduplicated: {md_result.deduplicated}");
    
    // Store a text document
    let txt_content = "This is a plain text document for testing the content service.";
    let txt_metadata = DocumentMetadata {
        title: Some("Test Document".to_string()),
        tags: vec!["test".to_string(), "plain-text".to_string()],
        ..Default::default()
    };
    
    let txt_result = service.store_document(
        txt_content.as_bytes().to_vec(),
        txt_metadata,
        "text"
    ).await?;
    
    println!("\nText document stored:");
    println!("  CID: {txt_result.cid}");
    println!("  Size: {txt_result.size} bytes");
    
    // Demo 2: Store images with metadata
    println!("\n\n2. Storing Images");
    println!("-----------------");
    
    // Create sample PNG data
    let png_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG header
        0x00, 0x00, 0x00, 0x0D, // Chunk length
        0x49, 0x48, 0x44, 0x52, // IHDR
        0x00, 0x00, 0x00, 0x10, // Width: 16
        0x00, 0x00, 0x00, 0x10, // Height: 16
        0x08, 0x02, 0x00, 0x00, 0x00, // Bit depth, color type, etc.
        0x90, 0x91, 0x68, 0x36, // CRC
        0x00, 0x00, 0x00, 0x00, // IEND chunk length
        0x49, 0x45, 0x4E, 0x44, // IEND
        0xAE, 0x42, 0x60, 0x82, // CRC
    ];
    
    let img_metadata = ImageMetadata {
        width: Some(16),
        height: Some(16),
        format: Some("PNG".to_string()),
        tags: vec!["sample".to_string(), "test-image".to_string()],
        ..Default::default()
    };
    
    let img_result = service.store_image(png_data, img_metadata, "png").await?;
    
    println!("Image stored:");
    println!("  CID: {img_result.cid}");
    println!("  Type: {:?}", img_result.content_type);
    
    // Demo 3: Search functionality
    println!("\n\n3. Content Search");
    println!("-----------------");
    
    // Search by text
    let text_query = SearchQuery {
        text: Some("documentation".to_string()),
        ..Default::default()
    };
    
    let text_results = service.search(text_query).await?;
    println!("\nSearch for 'documentation': {text_results.len(} results"));
    for result in text_results.iter().take(3) {
        println!("  - CID: {result.cid}, Score: {:.2}", result.score);
        if let Some(title) = &result.metadata.title {
            println!("    Title: {title}");
        }
    }
    
    // Search by tags
    let tag_query = SearchQuery {
        tags: vec!["test".to_string()],
        ..Default::default()
    };
    
    let tag_results = service.search(tag_query).await?;
    println!("\nSearch for tag 'test': {tag_results.len(} results"));
    for result in tag_results.iter().take(3) {
        println!("  - CID: {result.cid}, Tags: {:?}", result.metadata.tags);
    }
    
    // Demo 4: Content statistics
    println!("\n\n4. Content Statistics");
    println!("---------------------");
    
    let stats = service.stats().await;
    println!("Total documents: {stats.total_documents}");
    println!("Total images: {stats.total_images}");
    println!("Unique words indexed: {stats.unique_words}");
    println!("Unique tags: {stats.unique_tags}");
    
    // Demo 5: Batch operations
    println!("\n\n5. Batch Operations");
    println!("-------------------");
    
    use cim_ipld::content_types::TextDocument;
    
    let batch_items: Vec<TextDocument> = (0..5)
        .map(|i| {
            let content = format!("Batch document #{i}");
            let metadata = DocumentMetadata {
                title: Some(format!("Batch Doc {i}")),
                tags: vec!["batch".to_string(), format!("doc-{i}")],
                ..Default::default()
            };
            TextDocument::new(content, metadata).unwrap()
        })
        .collect();
    
    let batch_result = service.batch_store(batch_items).await;
    
    println!("Batch store results:");
    println!("  Successful: {batch_result.successful.len(}"));
    println!("  Failed: {batch_result.failed.len(}"));
    
    for (i, result) in batch_result.successful.iter().enumerate() {
        println!("  [{i}] CID: {result.cid}, Deduplicated: {result.deduplicated}");
    }
    
    // Demo 6: Content retrieval
    println!("\n\n6. Content Retrieval");
    println!("--------------------");
    
    use cim_ipld::content_types::MarkdownDocument;
    
    let retrieved: cim_ipld::content_types::service::RetrieveResult<MarkdownDocument> = 
        service.retrieve(&md_result.cid).await?;
    
    println!("Retrieved document:");
    println!("  CID: {retrieved.cid}");
    if let Some(title) = &retrieved.content.metadata.title {
        println!("  Title: {title}");
    }
    println!("  Content preview: {&retrieved.content.content[..50.min(retrieved.content.content.len(}..."))]);
    
    // Demo 7: List by type
    println!("\n\n7. List Content by Type");
    println!("-----------------------");
    
    use cim_ipld::object_store::PullOptions;
    
    let list_options = PullOptions {
        limit: Some(10),
        ..Default::default()
    };
    
    let text_cids = service.list_by_type(
        ContentType::Custom(0x600001), // TEXT codec
        list_options.clone()
    ).await?;
    
    println!("Text documents: {text_cids.len(} found"));
    for (i, cid) in text_cids.iter().take(3).enumerate() {
        println!("  [{i}] {cid}");
    }
    
    // Demo complete
    println!("\n\n✅ Content Service Demo Complete!");
    println!("All operations completed successfully.");
    
    Ok(())
}

// Helper function to demonstrate error handling
async fn demonstrate_error_handling(service: &ContentService) {
    println!("\n\n8. Error Handling");
    println!("-----------------");
    
    // Try to store oversized content
    let oversized = vec![0u8; 20 * 1024 * 1024]; // 20MB
    let metadata = DocumentMetadata::default();
    
    match service.store_document(oversized, metadata, "text").await {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("✓ Correctly rejected oversized content: {e}"),
    }
    
    // Try to store with invalid format
    match service.store_document(vec![1, 2, 3], DocumentMetadata::default(), "invalid").await {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("✓ Correctly rejected invalid format: {e}"),
    }
} 