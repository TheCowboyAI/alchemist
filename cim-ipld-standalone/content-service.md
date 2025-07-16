# CIM-IPLD Content Service

The Content Service provides a high-level API for managing typed content with automatic indexing, transformation capabilities, and lifecycle management.

## Features

- **Type-Safe Storage**: Store and retrieve strongly-typed content
- **Automatic Indexing**: Full-text and tag-based search capabilities
- **Content Transformation**: Convert between formats (with extensible transformers)
- **Deduplication**: Automatic content deduplication based on CIDs
- **Lifecycle Hooks**: Pre/post storage and retrieval hooks
- **Batch Operations**: Efficient batch storage with parallel processing
- **Content Validation**: Verify content integrity and format compliance

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Content Service                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────────┐   │
│  │   Storage   │  │   Indexing  │  │  Transformation  │   │
│  │   (NATS)    │  │  (In-Memory)│  │   (Extensible)   │   │
│  └─────────────┘  └─────────────┘  └──────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Content Types                             │
├─────────────────────────────────────────────────────────────┤
│  Documents │  Images  │   Audio   │   Video   │  Custom    │
│  PDF       │  JPEG    │   MP3     │   MP4     │  ...       │
│  DOCX      │  PNG     │   WAV     │   MKV     │            │
│  Markdown  │          │   FLAC    │   MOV     │            │
│  Text      │          │   AAC     │           │            │
└─────────────────────────────────────────────────────────────┘
```

## Usage

### Basic Setup

```rust
use cim_ipld::{
    content_types::service::{ContentService, ContentServiceConfig},
    object_store::NatsObjectStore,
};
use std::sync::Arc;

// Connect to NATS
let client = async_nats::connect("nats://localhost:4222").await?;
let jetstream = async_nats::jetstream::new(client);

// Create object store
let object_store = Arc::new(NatsObjectStore::new(jetstream).await?);

// Configure service
let config = ContentServiceConfig {
    auto_index: true,
    validate_on_store: true,
    max_content_size: 10 * 1024 * 1024, // 10MB
    allowed_types: vec![], // All types allowed
    enable_deduplication: true,
};

// Create service
let service = ContentService::new(object_store, config);
```

### Storing Documents

```rust
use cim_ipld::content_types::DocumentMetadata;

// Store a markdown document
let content = "# My Document\n\nThis is the content.";
let metadata = DocumentMetadata {
    title: Some("My Document".to_string()),
    author: Some("John Doe".to_string()),
    tags: vec!["example".to_string(), "demo".to_string()],
    ..Default::default()
};

let result = service.store_document(
    content.as_bytes().to_vec(),
    metadata,
    "markdown"
).await?;

println!("Stored document with CID: {}", result.cid);
println!("Deduplicated: {}", result.deduplicated);
```

### Storing Images

```rust
use cim_ipld::content_types::ImageMetadata;

let image_data = std::fs::read("photo.jpg")?;
let metadata = ImageMetadata {
    width: Some(1920),
    height: Some(1080),
    format: Some("JPEG".to_string()),
    tags: vec!["vacation".to_string(), "2024".to_string()],
    ..Default::default()
};

let result = service.store_image(image_data, metadata, "jpeg").await?;
```

### Searching Content

```rust
use cim_ipld::content_types::indexing::SearchQuery;

// Text search
let results = service.search(SearchQuery {
    text: Some("vacation".to_string()),
    ..Default::default()
}).await?;

// Tag search
let results = service.search(SearchQuery {
    tags: vec!["demo".to_string()],
    ..Default::default()
}).await?;

// Combined search with filters
let results = service.search(SearchQuery {
    text: Some("document".to_string()),
    tags: vec!["example".to_string()],
    content_types: vec![ContentType::Custom(codec::MARKDOWN)],
    limit: Some(10),
    ..Default::default()
}).await?;
```

### Retrieving Content

```rust
use cim_ipld::content_types::MarkdownDocument;

let retrieved = service.retrieve::<MarkdownDocument>(&cid).await?;
println!("Title: {:?}", retrieved.content.metadata.title);
println!("Content: {}", retrieved.content.content);
```

### Batch Operations

```rust
use cim_ipld::content_types::TextDocument;

let documents: Vec<TextDocument> = (0..100)
    .map(|i| TextDocument::new(
        format!("Document #{}", i),
        DocumentMetadata {
            title: Some(format!("Doc {}", i)),
            ..Default::default()
        },
    ).unwrap())
    .collect();

let batch_result = service.batch_store(documents).await;
println!("Successful: {}", batch_result.successful.len());
println!("Failed: {}", batch_result.failed.len());
```

### Lifecycle Hooks

```rust
// Add validation hook
service.add_pre_store_hook(|data, content_type| {
    if data.len() > 1_000_000 {
        return Err(Error::InvalidContent("Content too large".to_string()));
    }
    Ok(())
}).await;

// Add logging hook
service.add_post_store_hook(|cid, content_type| {
    println!("Stored {} with CID: {}", content_type_name(*content_type), cid);
}).await;
```

## Content Transformation

The transformation module provides extensible content conversion capabilities:

```rust
use cim_ipld::content_types::transformers::{
    TransformTarget, TransformOptions,
};

// Transform markdown to HTML (when implemented)
let html_result = service.transform(
    &markdown_cid,
    TransformTarget::Html,
    TransformOptions::default(),
).await?;
```

### Available Transformations

- **Documents**: Markdown ↔ HTML, Any → Plain Text
- **Images**: Format conversion, resizing, thumbnail generation (placeholders)
- **Audio**: Format conversion, metadata extraction (placeholders)
- **Video**: Format conversion, thumbnail extraction (placeholders)

## Content Indexing

The indexing system provides efficient search capabilities:

### Index Features

- **Text Search**: Full-text search with relevance scoring
- **Tag Search**: Exact tag matching with AND operations
- **Type Filtering**: Filter by content type
- **Pagination**: Offset and limit support
- **Metadata Search**: Search by title, author, etc.

### Index Statistics

```rust
let stats = service.stats().await;
println!("Total documents: {}", stats.total_documents);
println!("Total images: {}", stats.total_images);
println!("Unique words indexed: {}", stats.unique_words);
println!("Unique tags: {}", stats.unique_tags);
```

## Configuration Options

### ContentServiceConfig

| Field                  | Type               | Default | Description                         |
| ---------------------- | ------------------ | ------- | ----------------------------------- |
| `auto_index`           | `bool`             | `true`  | Enable automatic indexing on store  |
| `validate_on_store`    | `bool`             | `true`  | Validate content before storing     |
| `max_content_size`     | `usize`            | `100MB` | Maximum allowed content size        |
| `allowed_types`        | `Vec<ContentType>` | `[]`    | Allowed content types (empty = all) |
| `enable_deduplication` | `bool`             | `true`  | Enable content deduplication        |

## Error Handling

The service uses the unified `cim_ipld::Error` type:

```rust
match service.store_document(data, metadata, format).await {
    Ok(result) => println!("Success: {}", result.cid),
    Err(Error::InvalidContent(msg)) => eprintln!("Invalid content: {}", msg),
    Err(Error::StorageError(msg)) => eprintln!("Storage error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Performance Considerations

1. **Batch Operations**: Use `batch_store` for multiple items to leverage parallelism
2. **Indexing**: Consider disabling auto-indexing for bulk imports
3. **Deduplication**: Disable for content that's unlikely to duplicate
4. **Large Files**: Consider streaming APIs for files > 100MB

## Extending the Service

### Adding New Content Types

1. Define the content type in `content_types.rs`
2. Implement `TypedContent` trait
3. Add verification logic
4. Update the service to handle the new type

### Adding Transformers

1. Implement transformation functions in `transformers.rs`
2. Update `TransformTarget` enum
3. Wire up in the service's `transform` method

### Custom Indexes

1. Extend the `ContentIndex` struct
2. Add custom indexing logic
3. Update search functionality

## Examples

See the following examples for complete demonstrations:

- `examples/content_service_demo.rs` - Comprehensive service demonstration
- `examples/content_types_demo.rs` - Basic content type usage
- `examples/pull_from_jetstream.rs` - Content retrieval patterns

## Future Enhancements

- Streaming API for large files
- Real transformation implementations (not placeholders)
- Persistent indexing (currently in-memory)
- Content versioning support
- Access control and permissions
- Content expiration policies
- Compression support
- Encryption at rest 