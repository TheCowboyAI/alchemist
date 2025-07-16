# CIM-IPLD

IPLD (InterPlanetary Linked Data) implementation for Composable Information Machines (CIM).

## Overview

CIM-IPLD provides a content-addressed storage foundation for all CIM nodes, enabling:

- **Content-addressed storage** with CIDs (Content Identifiers)
- **Cryptographic integrity** through hash chains
- **Type-safe content handling** with custom codecs
- **Extensible architecture** for domain-specific types

## Features

- 🔗 **Chain Linking**: Create tamper-evident chains of content
- 🎯 **Type Safety**: Strongly typed content with compile-time guarantees
- 🔌 **Extensible**: Register custom codecs for your content types
- 🚀 **Performance**: BLAKE3 hashing for fast content addressing
- 📦 **Codec Registry**: Manage content types with codec identifiers
- 🌐 **Standard IPLD Codecs**: Full support for dag-cbor, dag-json, raw, and more
- 🏗️ **CIM-Specific JSON Types**: Custom codecs for alchemist, workflow-graph, context-graph
- 📄 **Rich Content Types**: Built-in support for PDF, DOCX, Markdown, JPEG, PNG, MP3, MP4, and more
- 🔍 **Content Service**: High-level API with full-text search and indexing
- 🔄 **Content Transformation**: Convert between formats (extensible transformer system)
- 💾 **NATS Object Store**: Reliable distributed storage backend
- ♻️ **Automatic Deduplication**: Content deduplication based on CIDs
- 🗂️ **Domain Partitioning**: Intelligent content routing to domain-specific buckets

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
# From GitHub
cim-ipld = { git = "https://github.com/thecowboyai/cim-ipld" }

# From crates.io (once published)
cim-ipld = "0.1"
```

## Quick Start

### Basic Usage

```rust
use cim_ipld::{ChainedContent, ContentChain, TypedContent, ContentType};
use serde::{Serialize, Deserialize};

// Define your content type
#[derive(Serialize, Deserialize)]
struct MyEvent {
    id: String,
    action: String,
    timestamp: u64,
}

// Implement TypedContent
impl TypedContent for MyEvent {
    const CODEC: u64 = 0x300100; // Your codec in range 0x300000-0x3FFFFF
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x300100);
}

// Create a chain
let mut chain = ContentChain::<MyEvent>::new();

// Add events
let event = MyEvent {
    id: "evt-001".to_string(),
    action: "user.login".to_string(),
    timestamp: 1234567890,
};

let chained = chain.append(event)?;
println!("Event CID: {}", chained.cid);
```

### Custom Codecs

```rust
use cim_ipld::{CimCodec, CodecRegistry};
use std::sync::Arc;

struct MyCustomCodec;

impl CimCodec for MyCustomCodec {
    fn code(&self) -> u64 {
        0x330000 // Your codec identifier
    }

    fn name(&self) -> &str {
        "my-custom-codec"
    }

    // Optionally override encode/decode for custom serialization
}

// Register your codec
let mut registry = CodecRegistry::new();
registry.register(Arc::new(MyCustomCodec))?;
```

## IPLD Codec Support

### Standard IPLD Codecs

CIM-IPLD includes full support for standard IPLD codecs:

| Codec      | Code   | Description        |
| ---------- | ------ | ------------------ |
| raw        | 0x55   | Raw binary data    |
| json       | 0x0200 | Standard JSON      |
| cbor       | 0x51   | Standard CBOR      |
| dag-pb     | 0x70   | MerkleDAG protobuf |
| dag-cbor   | 0x71   | MerkleDAG CBOR     |
| dag-json   | 0x0129 | MerkleDAG JSON     |
| libp2p-key | 0x72   | Libp2p public key  |
| git-raw    | 0x78   | Git objects        |

### CIM-Specific JSON Types

| Type           | Code     | Description                  |
| -------------- | -------- | ---------------------------- |
| alchemist      | 0x340000 | Alchemist configuration      |
| workflow-graph | 0x340001 | Workflow graph definitions   |
| context-graph  | 0x340002 | Context graph structures     |
| concept-space  | 0x340003 | Conceptual space definitions |
| domain-model   | 0x340004 | Domain model specifications  |
| event-stream   | 0x340005 | Event stream metadata        |
| command-batch  | 0x340006 | Command batch definitions    |
| query-result   | 0x340007 | Query result structures      |

### Using IPLD Codecs

```rust
use cim_ipld::{DagCborCodec, DagJsonCodec, CodecOperations};

// Any serializable type can use IPLD codecs
let data = MyStruct { ... };

// Encode as DAG-CBOR
let cbor = data.to_dag_cbor()?;
let cbor_alt = DagCborCodec::encode(&data)?;

// Encode as DAG-JSON
let json = data.to_dag_json()?;
let pretty = data.to_dag_json_pretty()?;

// Decode
let decoded: MyStruct = DagCborCodec::decode(&cbor)?;
```

## Content Types

CIM-IPLD defines standard content types:

| Type     | Codec     | Range          |
| -------- | --------- | -------------- |
| Event    | 0x300000  | Core CIM types |
| Graph    | 0x300001  |                |
| Node     | 0x300002  |                |
| Edge     | 0x300003  |                |
| Command  | 0x300004  |                |
| Query    | 0x300005  |                |
| Markdown | 0x310000  | Document types |
| JSON     | 0x310001  |                |
| YAML     | 0x310002  |                |
| TOML     | 0x310003  |                |
| Image    | 0x320000  | Media types    |
| Video    | 0x320001  |                |
| Audio    | 0x320002  |                |
| Custom   | 0x330000+ | Your types     |

## Content Service

The Content Service provides a high-level API for managing typed content:

```rust
use cim_ipld::content_types::service::{ContentService, ContentServiceConfig};

// Configure and create service
let config = ContentServiceConfig {
    auto_index: true,
    max_content_size: 10 * 1024 * 1024, // 10MB
    enable_deduplication: true,
    ..Default::default()
};
let service = ContentService::new(storage, config);

// Store documents with metadata
let result = service.store_document(
    content.as_bytes().to_vec(),
    DocumentMetadata {
        title: Some("My Document".to_string()),
        tags: vec!["example".to_string()],
        ..Default::default()
    },
    "markdown"
).await?;

// Search content
let results = service.search(SearchQuery {
    text: Some("example".to_string()),
    tags: vec!["demo".to_string()],
    ..Default::default()
}).await?;
```

See [CONTENT_SERVICE.md](CONTENT_SERVICE.md) for comprehensive documentation.

## Chain Validation

CIM-IPLD provides cryptographic chain validation:

```rust
// Validate entire chain
chain.validate()?;

// Get items since a specific CID
let recent = chain.items_since(&previous_cid)?;

// Detect tampering
let mut tampered = chain.items()[0].clone();
tampered.sequence = 999; // Modify
assert!(tampered.validate_chain(None).is_err()); // Validation fails
```

## Architecture

```
cim-ipld/
├── src/
│   ├── chain/      # Chain linking implementation
│   ├── codec/      # Codec registry and traits
│   ├── traits/     # Core traits (TypedContent)
│   ├── types/      # Type definitions
│   └── error.rs    # Error types
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Documentation

- [User Stories](docs/USER_STORIES.md) - Detailed user stories and use cases
- [Architecture](docs/ARCHITECTURE.md) - System architecture and design decisions
- [Developer Guide](docs/DEVELOPER_GUIDE.md) - Comprehensive guide for developers
- [API Reference](docs/API.md) - Complete API documentation
- [Migration Guide](docs/MIGRATION_GUIDE.md) - Guide for migrating from other storage systems
- [Test Coverage Assessment](docs/TEST_COVERAGE_ASSESSMENT.md) - Current test coverage analysis
- [Test Implementation Roadmap](docs/TEST_IMPLEMENTATION_ROADMAP.md) - Plan for comprehensive testing
- [Content Types](CONTENT_TYPES.md) - Guide to using built-in content types
- [Content Service](CONTENT_SERVICE.md) - High-level content management API
- [IPLD Codecs](IPLD_CODECS.md) - Standard IPLD codec support and CIM-specific types
- [Domain Partitioning](docs/DOMAIN_PARTITIONING.md) - Automatic content routing to domain buckets

## Links

- [CIM Architecture](https://github.com/thecowboyai/alchemist)
- [IPLD Specification](https://ipld.io/)
- [CID Specification](https://github.com/multiformats/cid)
