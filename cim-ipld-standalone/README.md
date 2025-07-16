# CIM-IPLD

Content-addressed storage for the Composable Information Machine using IPLD (InterPlanetary Linked Data).

## Overview

CIM-IPLD provides a robust content-addressed storage system with:
- **Content Type Support**: Documents (PDF, DOCX, Markdown, Text), Images (JPEG, PNG, GIF, WebP), Audio (MP3, WAV, FLAC, AAC, OGG), Video (MP4, MOV, MKV, AVI)
- **IPLD Codecs**: DAG-JSON, DAG-CBOR, Raw, and custom CIM-specific codecs
- **Content Chains**: Cryptographically linked content with integrity verification
- **NATS Object Store**: Distributed storage backend with caching
- **Transformation Pipeline**: Convert between formats while preserving CIDs
- **Domain Partitioning**: Organize content by domain boundaries

## Features

### Content Management
- Automatic content type detection from magic bytes
- Typed wrappers for all supported formats
- Metadata extraction and preservation
- Content validation and verification

### IPLD Integration
- CID generation using BLAKE3 hashing
- Multiple codec support with registry pattern
- Canonical payload extraction for consistent CIDs
- Chain-based content linking

### Storage Backend
- NATS JetStream object store integration
- LRU caching for performance
- Domain-based partitioning
- Compression support

## Usage

```rust
use cim_ipld::{ContentChain, TextDocument, DocumentMetadata};

// Create a content chain
let mut chain = ContentChain::new();

// Add documents to the chain
let doc = TextDocument {
    content: "Hello, IPLD!".to_string(),
    metadata: DocumentMetadata {
        title: Some("My Document".to_string()),
        ..Default::default()
    },
};

let item = chain.append(doc)?;
println!("Document CID: {}", item.cid);
```

## Architecture

```
cim-ipld/
├── src/
│   ├── chain/          # Content chain implementation
│   ├── codec/          # IPLD codec support
│   ├── content_types/  # Type-specific content handling
│   ├── object_store/   # NATS storage backend
│   ├── traits.rs       # Core traits (TypedContent)
│   ├── types.rs        # Type definitions
│   └── error.rs        # Error handling
├── tests/              # Comprehensive test suite
└── examples/           # Usage examples
```

## Testing

The module includes comprehensive test coverage:
- **Unit Tests**: Core functionality testing
- **Integration Tests**: End-to-end scenarios
- **Doc Tests**: Example code in documentation
- **Performance Benchmarks**: Throughput and latency tests

Run tests with:
```bash
cargo test          # All tests
cargo test --lib    # Library tests only
cargo test --doc    # Documentation tests
cargo bench         # Performance benchmarks
```

## Status

✅ **100% Feature Complete**
- All content types implemented and tested
- Full IPLD codec support
- Content chain operations working
- NATS integration functional
- 76+ tests passing
- 5 doc tests passing
- Ready for production use

## Dependencies

- `cid` - Content identifiers
- `multihash` - Cryptographic hashing
- `blake3` - BLAKE3 hashing algorithm
- `serde` - Serialization framework
- `async-nats` - NATS client
- `tokio` - Async runtime

## License

Apache-2.0 OR MIT