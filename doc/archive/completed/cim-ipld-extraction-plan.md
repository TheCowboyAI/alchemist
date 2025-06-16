# CIM-IPLD Standalone Module Extraction Plan

## Overview

Extract the CIM-IPLD functionality into a standalone Rust library that can be:
- Published as a separate crate on crates.io
- Used as a Git dependency from GitHub
- Extended by individual CIM implementations
- Versioned independently

## Architecture

### Core Library Structure

```
cim-ipld/
├── Cargo.toml
├── README.md
├── LICENSE (Apache-2.0 + MIT)
├── src/
│   ├── lib.rs
│   ├── traits/
│   │   ├── mod.rs
│   │   ├── typed_content.rs
│   │   └── codec.rs
│   ├── codecs/
│   │   ├── mod.rs
│   │   ├── registry.rs
│   │   └── base/
│   │       ├── mod.rs
│   │       ├── event.rs
│   │       ├── graph.rs
│   │       └── document.rs
│   ├── types/
│   │   ├── mod.rs
│   │   ├── content_type.rs
│   │   └── cid_chain.rs
│   ├── relationships/
│   │   ├── mod.rs
│   │   ├── predicates.rs
│   │   └── index.rs
│   └── error.rs
├── examples/
│   ├── basic_usage.rs
│   └── custom_codec.rs
└── tests/
    └── integration_tests.rs
```

### Core Components

#### 1. Base Traits
```rust
// src/traits/typed_content.rs
pub trait TypedContent: Serialize + DeserializeOwned + Send + Sync {
    /// The IPLD codec for this content type
    const CODEC: u64;

    /// The content type identifier
    const CONTENT_TYPE: ContentType;

    /// Convert to IPLD representation
    fn to_ipld(&self) -> Result<Ipld>;

    /// Create from IPLD representation
    fn from_ipld(ipld: &Ipld) -> Result<Self> where Self: Sized;

    /// Calculate the CID for this content
    fn calculate_cid(&self) -> Result<Cid>;
}

// src/traits/codec.rs
pub trait CimCodec: Send + Sync {
    /// Unique codec identifier (0x300000-0x3FFFFF range)
    fn code(&self) -> u64;

    /// Encode content to bytes
    fn encode<T: TypedContent>(&self, content: &T) -> Result<Vec<u8>>;

    /// Decode content from bytes
    fn decode<T: TypedContent>(&self, bytes: &[u8]) -> Result<T>;
}
```

#### 2. Base Content Types
```rust
// src/types/content_type.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentType {
    // Core CIM Types (0x300000-0x30FFFF)
    Event,
    Graph,
    Node,
    Edge,
    Command,
    Query,

    // Document Types (0x310000-0x31FFFF)
    Markdown,
    Json,
    Yaml,
    Toml,

    // Media Types (0x320000-0x32FFFF)
    Image,
    Video,
    Audio,

    // Extension point for custom types
    Custom(u64),
}
```

#### 3. Codec Registry
```rust
// src/codecs/registry.rs
pub struct CodecRegistry {
    codecs: HashMap<u64, Arc<dyn CimCodec>>,
}

impl CodecRegistry {
    /// Create a new registry with base codecs
    pub fn new() -> Self {
        let mut registry = Self {
            codecs: HashMap::new(),
        };

        // Register base codecs
        registry.register_base_codecs();
        registry
    }

    /// Register a custom codec
    pub fn register(&mut self, codec: Arc<dyn CimCodec>) -> Result<()> {
        let code = codec.code();

        // Validate codec range
        if !(0x300000..=0x3FFFFF).contains(&code) {
            return Err(Error::InvalidCodecRange(code));
        }

        self.codecs.insert(code, codec);
        Ok(())
    }
}
```

#### 4. CID Chain Support
```rust
// src/types/cid_chain.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainedContent<T: TypedContent> {
    pub content: T,
    pub cid: Cid,
    pub previous_cid: Option<Cid>,
    pub sequence: u64,
    pub timestamp: SystemTime,
}

impl<T: TypedContent> ChainedContent<T> {
    pub fn new(content: T, previous: Option<&ChainedContent<T>>) -> Result<Self> {
        // Implementation
    }

    pub fn validate_chain(&self, previous: Option<&ChainedContent<T>>) -> Result<()> {
        // Implementation
    }
}
```

### Extension Pattern

Projects using cim-ipld can extend it:

```rust
// In Information Alchemist
use cim_ipld::{TypedContent, ContentType, CodecRegistry};

// Define custom content type
#[derive(Serialize, Deserialize)]
pub struct ConceptualSpace {
    pub dimensions: Vec<Dimension>,
    pub embeddings: HashMap<String, Vec<f32>>,
}

impl TypedContent for ConceptualSpace {
    const CODEC: u64 = 0x330000; // Custom range
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x330000);

    // Implement required methods
}

// Register custom codec
let mut registry = CodecRegistry::new();
registry.register(Arc::new(ConceptualSpaceCodec))?;
```

## Implementation Steps

### Phase 1: Create Repository (Week 1)
1. Create new GitHub repository: `thecowboyai/cim-ipld`
2. Set up Rust project structure
3. Configure CI/CD with GitHub Actions
4. Add comprehensive documentation

### Phase 2: Core Implementation (Week 1-2)
1. Implement base traits and types
2. Create codec registry system
3. Add CID chain functionality
4. Implement base codecs for common types

### Phase 3: Testing & Documentation (Week 2)
1. Write comprehensive unit tests
2. Create integration tests
3. Add usage examples
4. Write API documentation

### Phase 4: Integration (Week 3)
1. Update Information Alchemist to use external cim-ipld
2. Remove duplicate code from IA
3. Test integration thoroughly
4. Document migration process

## Cargo.toml Structure

```toml
[package]
name = "cim-ipld"
version = "0.1.0"
edition = "2021"
authors = ["The Cowboy AI Team"]
description = "IPLD implementation for Composable Information Machines"
repository = "https://github.com/thecowboyai/cim-ipld"
license = "Apache-2.0 OR MIT"
keywords = ["ipld", "cid", "content-addressing", "cim"]
categories = ["data-structures", "encoding"]

[dependencies]
cid = "0.11"
multihash = "0.19"
ipld-core = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
blake3 = "1.5"
thiserror = "2.0"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

## Usage in Information Alchemist

```toml
# Cargo.toml
[dependencies]
# Option 1: From crates.io (once published)
cim-ipld = "0.1"

# Option 2: From GitHub
cim-ipld = { git = "https://github.com/thecowboyai/cim-ipld" }

# Option 3: From GitHub with specific branch/tag
cim-ipld = { git = "https://github.com/thecowboyai/cim-ipld", tag = "v0.1.0" }
```

## Benefits

1. **Reusability**: All CIM nodes can use the same base implementation
2. **Consistency**: Ensures codec compatibility across the network
3. **Extensibility**: Each project can add domain-specific types
4. **Versioning**: Independent versioning and updates
5. **Community**: Can accept contributions from all CIM users

## Migration Plan

1. Extract existing CID chain code from IA
2. Generalize for broader use cases
3. Create extension points for custom types
4. Update IA to use external dependency
5. Document migration for other projects

## Success Criteria

- [ ] Standalone library compiles and tests pass
- [ ] Published to GitHub with CI/CD
- [ ] Information Alchemist successfully uses external lib
- [ ] Documentation complete with examples
- [ ] At least one other CIM project adopts it

---

*Plan Created: 2025-06-07*
*Target Completion: 3 Weeks*
