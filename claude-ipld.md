# CLAUDE.md for cim-ipld

This file provides guidance to Claude Code (claude.ai/code) when working with the cim-ipld repository - the content-addressed storage layer for CIM.

## Critical Context

**cim-ipld is the CONTENT LAYER of CIM** - it provides content-addressed storage using IPLD (InterPlanetary Linked Data) standards. This module handles all content persistence, CID generation, and cryptographic integrity for the entire CIM system.

## Project Overview

**cim-ipld** provides production-ready content-addressed storage with:
- IPLD-compliant CID generation and content addressing
- Support for multiple content types (documents, images, audio, video)
- Cryptographic content chains for audit trails
- NATS JetStream integration for distributed storage
- Codec registry supporting DAG-JSON, DAG-CBOR, and custom formats
- Domain-based content partitioning

## Architecture Principles

### Core Design Patterns
1. **Content-First Design** - Everything is content with a CID
2. **Type Safety** - Strongly typed content wrappers
3. **Immutability** - Content never changes once stored
4. **Cryptographic Integrity** - All content is hash-verified
5. **Chain Linking** - Content forms cryptographic chains

### Key Components

#### 1. Content Types (`src/content_types/`)
- Document types: PDF, DOCX, Markdown, Text
- Image types: JPEG, PNG, GIF, WebP  
- Audio types: MP3, WAV, FLAC, AAC, OGG
- Video types: MP4, MOV, MKV, AVI
- Magic byte detection for format verification
- Metadata extraction and preservation

#### 2. Codec System (`src/codec/`)
- IPLD standard codecs (DAG-JSON, DAG-CBOR)
- Custom CIM codecs for specialized content
- Codec registry with dynamic registration
- Content transformation between formats

#### 3. Content Chains (`src/chain/`)
- Cryptographically linked content sequences
- Parent-child relationships with CID references
- Chain validation and integrity checks
- Sequence numbering and timestamps

#### 4. Object Store (`src/object_store/`)
- NATS JetStream backend integration
- LRU caching for performance
- Domain-based partitioning
- Compression and deduplication

#### 5. Traits (`src/traits.rs`)
- `TypedContent` - Core trait for all content
- Canonical payload extraction
- CID calculation with BLAKE3
- Serialization/deserialization

## Development Rules

### MANDATORY Test Coverage
1. **Every content type must have tests** - Format verification, CID generation
2. **Chain operations need integration tests** - Multi-step scenarios
3. **Codec implementations need round-trip tests** - Encode/decode verification
4. **Performance benchmarks required** - For CID generation and storage

### Code Organization
```
cim-ipld/
├── src/
│   ├── lib.rs              # Public API and re-exports
│   ├── chain/              # Content chain implementation
│   │   └── mod.rs          # ChainedContent, ContentChain
│   ├── codec/              # IPLD codec support
│   │   ├── mod.rs          # Codec registry
│   │   └── ipld_codecs.rs  # Codec implementations
│   ├── content_types/      # Content type support
│   │   ├── mod.rs          # Type definitions
│   │   ├── service.rs      # Content services
│   │   ├── transformers.rs # Format conversions
│   │   └── indexing.rs     # Content indexing
│   ├── object_store/       # Storage backend
│   │   ├── mod.rs          # Store trait
│   │   └── nats_store.rs   # NATS implementation
│   ├── traits.rs           # Core traits
│   ├── types.rs            # Type definitions
│   └── error.rs            # Error handling
├── tests/                  # Integration tests
├── benches/               # Performance benchmarks
└── examples/              # Usage examples
```

### API Design Guidelines
1. **Content Immutability** - Once created, content never changes
2. **CID Determinism** - Same content always produces same CID
3. **Type Safety** - Use typed wrappers, not raw bytes
4. **Async Everything** - All I/O operations are async
5. **Error Propagation** - Use Result<T, Error> everywhere

## Usage Patterns

### Basic Content Storage
```rust
use cim_ipld::{TextDocument, DocumentMetadata};

// Create typed content
let doc = TextDocument {
    content: "Hello, IPLD!".to_string(),
    metadata: DocumentMetadata {
        title: Some("My Document".to_string()),
        ..Default::default()
    },
};

// Calculate CID
use cim_ipld::TypedContent;
let cid = doc.calculate_cid()?;
```

### Content Chains
```rust
use cim_ipld::{ContentChain, ChainedContent};

// Create a chain
let mut chain = ContentChain::<TextDocument>::new();

// Add content (automatically linked)
let item = chain.append(doc)?;
assert_eq!(item.sequence, 0);

// Add more content
let item2 = chain.append(doc2)?;
assert_eq!(item2.sequence, 1);
assert_eq!(item2.previous_cid, Some(item.cid));
```

### Object Store Integration
```rust
use cim_ipld::object_store::{ObjectStore, NatsObjectStore};

// Create store
let store = NatsObjectStore::new(nats_client, "content-bucket").await?;

// Store content
let cid = store.put_content(&content).await?;

// Retrieve content
let retrieved = store.get_content(&cid).await?;
```

## Testing Requirements

### Content Type Tests
- Format detection from magic bytes
- Valid/invalid content verification
- Metadata extraction
- CID generation consistency

### Chain Tests
- Sequential linking verification
- Chain validation
- Parent-child relationships
- Integrity checking

### Codec Tests
- Round-trip encode/decode
- Cross-codec compatibility
- Error handling
- Performance benchmarks

### Integration Tests
- End-to-end scenarios
- Multi-domain content
- NATS storage operations
- Concurrent access

## Current Status

- **100% Feature Complete**
- **76+ tests passing**
- **5 doc tests passing**
- **All content types implemented**
- **Production-ready storage backend**

## Performance Targets

- CID generation: <1ms for 1MB content
- Content storage: >10,000 ops/sec
- Content retrieval: <10ms p99
- Chain validation: <100ms for 1000 items
- Codec operations: <5ms for typical content

## Dependencies

### Core
- `cid` - Content identifiers
- `multihash` - Cryptographic hashing  
- `blake3` - BLAKE3 hashing
- `serde` - Serialization

### Storage
- `async-nats` - NATS client
- `tokio` - Async runtime
- `lru` - Caching
- `zstd` - Compression

### Content Processing
- `image` - Image format support
- `pulldown-cmark` - Markdown parsing
- `symphonia` - Audio format support

## Important Conventions

1. **CIDs are immutable** - Never modify content after CID generation
2. **Use canonical payloads** - Extract only stable content for CIDs
3. **Verify content types** - Always check magic bytes
4. **Chain integrity** - Validate parent references
5. **Async operations** - All I/O must be async

## Future Enhancements

1. **Encryption Support** - Client-side encryption before storage
2. **Replication** - Multi-region content replication
3. **Garbage Collection** - Automated cleanup of orphaned content
4. **Advanced Indexing** - Full-text search capabilities
5. **Streaming Support** - Large file streaming

## Working with this Module

When making changes:
1. **Run all tests** - `cargo test --all-features`
2. **Check benchmarks** - `cargo bench`
3. **Verify examples** - `cargo run --example basic_usage`
4. **Update docs** - `cargo doc --open`
5. **Test with NATS** - Requires running NATS server

This module is critical infrastructure - changes must maintain backward compatibility and performance.