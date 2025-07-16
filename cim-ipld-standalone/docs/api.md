# CIM-IPLD API Reference

## Table of Contents

1. [Core Types](#core-types)
2. [Traits](#traits)
3. [Chain Management](#chain-management)
4. [Codec Registry](#codec-registry)
5. [Storage Interface](#storage-interface)
6. [Error Types](#error-types)
7. [Utility Functions](#utility-functions)

## Core Types

### `Cid`

Content Identifier - uniquely identifies content by its cryptographic hash.

```rust
pub struct Cid { /* opaque */ }

impl Cid {
    /// Parse a CID from a string
    pub fn try_from(s: &str) -> Result<Self, Error>

    /// Convert to string representation
    pub fn to_string(&self) -> String

    /// Get the codec used for this CID
    pub fn codec(&self) -> u64

    /// Get the multihash
    pub fn hash(&self) -> &Multihash
}
```

### `ContentType`

Categorizes different types of content in the system.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    // Core types (0x300000-0x30FFFF)
    Event,      // 0x300000
    Graph,      // 0x300001
    Node,       // 0x300002
    Edge,       // 0x300003
    Command,    // 0x300004
    Query,      // 0x300005

    // Document types (0x310000-0x31FFFF)
    Markdown,   // 0x310000
    Json,       // 0x310001
    Yaml,       // 0x310002
    Toml,       // 0x310003

    // Media types (0x320000-0x32FFFF)
    Image,      // 0x320000
    Video,      // 0x320001
    Audio,      // 0x320002

    // Custom types (0x330000-0x3FFFFF)
    Custom(u64),
}

impl ContentType {
    /// Get the codec identifier for this content type
    pub fn codec(&self) -> u64

    /// Get a human-readable name
    pub fn name(&self) -> &'static str

    /// Check if this is a custom type
    pub fn is_custom(&self) -> bool
}
```

## Traits

### `TypedContent`

Core trait for content that can be stored in CIM-IPLD.

```rust
pub trait TypedContent: Serialize + DeserializeOwned + Send + Sync {
    /// Unique codec identifier (must be in valid range)
    const CODEC: u64;

    /// Content type classification
    const CONTENT_TYPE: ContentType;

    /// Convert to bytes for storage
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    /// Create from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }

    /// Generate CID for this content
    fn generate_cid(&self) -> Result<Cid> {
        let bytes = self.to_bytes()?;
        let hash = blake3::hash(&bytes);
        Ok(Cid::new_v1(Self::CODEC, hash))
    }
}
```

### `CimCodec`

Trait for custom content encoders/decoders.

```rust
pub trait CimCodec: Send + Sync {
    /// Unique codec identifier
    fn code(&self) -> u64;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Encode data
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }

    /// Decode data
    fn decode(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }
}
```

## Chain Management

### `ChainedContent<T>`

Represents a single item in a content chain.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainedContent<T: TypedContent> {
    /// Content identifier
    pub cid: Cid,

    /// Previous item in chain (None for first item)
    pub previous_cid: Option<Cid>,

    /// Sequence number (starts at 0)
    pub sequence: u64,

    /// Creation timestamp
    pub timestamp: SystemTime,

    /// The actual content
    pub content: T,
}

impl<T: TypedContent> ChainedContent<T> {
    /// Create a new chained content item
    pub fn new(content: T, previous_cid: Option<Cid>, sequence: u64) -> Result<Self>

    /// Validate this item's chain properties
    pub fn validate_chain(&self, expected_previous: Option<&Cid>) -> Result<()>

    /// Check if this is the first item in a chain
    pub fn is_genesis(&self) -> bool
}
```

### `ContentChain<T>`

Manages a chain of content items.

```rust
pub struct ContentChain<T: TypedContent> {
    // Private fields
}

impl<T: TypedContent> ContentChain<T> {
    /// Create a new empty chain
    pub fn new() -> Self

    /// Append content to the chain
    pub fn append(&mut self, content: T) -> Result<ChainedContent<T>>

    /// Get the number of items in the chain
    pub fn len(&self) -> usize

    /// Check if chain is empty
    pub fn is_empty(&self) -> bool

    /// Get all items in the chain
    pub fn items(&self) -> &[ChainedContent<T>]

    /// Get the head (latest) item
    pub fn head(&self) -> Option<&ChainedContent<T>>

    /// Get the tail (first) item
    pub fn tail(&self) -> Option<&ChainedContent<T>>

    /// Get items since a specific CID
    pub fn items_since(&self, cid: &Cid) -> Result<Vec<&ChainedContent<T>>>

    /// Validate the entire chain
    pub fn validate(&self) -> Result<()>

    /// Find an item by CID
    pub fn find_by_cid(&self, cid: &Cid) -> Option<&ChainedContent<T>>

    /// Save chain to storage
    pub async fn save<S: ObjectStore>(&self, store: &S) -> Result<Cid>

    /// Load chain from storage
    pub async fn load<S: ObjectStore>(store: &S, head_cid: &Cid) -> Result<Self>
}
```

## Codec Registry

### `CodecRegistry`

Manages available codecs for content encoding/decoding.

```rust
pub struct CodecRegistry {
    // Private fields
}

impl CodecRegistry {
    /// Create a new registry with default codecs
    pub fn new() -> Self

    /// Register a custom codec
    pub fn register(&mut self, codec: Arc<dyn CimCodec>) -> Result<()>

    /// Get a codec by its code
    pub fn get(&self, code: u64) -> Option<Arc<dyn CimCodec>>

    /// Get a codec by name
    pub fn get_by_name(&self, name: &str) -> Option<Arc<dyn CimCodec>>

    /// List all registered codecs
    pub fn list(&self) -> Vec<(u64, String)>

    /// Check if a codec is registered
    pub fn contains(&self, code: u64) -> bool
}
```

## Storage Interface

### `ObjectStore`

Trait for storage backends.

```rust
#[async_trait]
pub trait ObjectStore: Send + Sync {
    /// Store data with a key
    async fn put(&self, key: &str, data: Vec<u8>) -> Result<()>;

    /// Retrieve data by key
    async fn get(&self, key: &str) -> Result<Vec<u8>>;

    /// Delete data by key
    async fn delete(&self, key: &str) -> Result<()>;

    /// Check if key exists
    async fn exists(&self, key: &str) -> Result<bool>;

    /// List keys with prefix
    async fn list(&self, prefix: &str) -> Result<Vec<String>>;

    /// Store typed content
    async fn put_typed<T: TypedContent>(&self, content: &T) -> Result<Cid> {
        let cid = content.generate_cid()?;
        let bytes = content.to_bytes()?;
        self.put(&cid.to_string(), bytes).await?;
        Ok(cid)
    }

    /// Retrieve typed content
    async fn get_typed<T: TypedContent>(&self, cid: &Cid) -> Result<T> {
        let bytes = self.get(&cid.to_string()).await?;
        T::from_bytes(&bytes)
    }
}
```

### `MemoryStore`

In-memory storage implementation.

```rust
pub struct MemoryStore {
    // Private fields
}

impl MemoryStore {
    /// Create new memory store
    pub fn new() -> Self

    /// Create with capacity limit
    pub fn with_capacity(max_bytes: usize) -> Self

    /// Get current size in bytes
    pub fn size(&self) -> usize

    /// Clear all stored data
    pub fn clear(&self)
}
```

### `FileStore`

File system storage implementation.

```rust
pub struct FileStore {
    // Private fields
}

impl FileStore {
    /// Create new file store
    pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self>

    /// Builder pattern for configuration
    pub fn builder() -> FileStoreBuilder
}

pub struct FileStoreBuilder {
    // Private fields
}

impl FileStoreBuilder {
    /// Set base path
    pub fn path<P: AsRef<Path>>(mut self, path: P) -> Self

    /// Set maximum file size
    pub fn max_file_size(mut self, size: usize) -> Self

    /// Enable compression
    pub fn compression(mut self, enabled: bool) -> Self

    /// Build the FileStore
    pub fn build(self) -> Result<FileStore>
}
```

## Error Types

### `Error`

Main error type for CIM-IPLD operations.

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Content not found: {0}")]
    NotFound(Cid),

    #[error("Invalid content: {0}")]
    InvalidContent(String),

    #[error("Chain validation failed: {0}")]
    ChainValidation(String),

    #[error("Codec error: {0}")]
    Codec(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

## Utility Functions

### CID Generation

```rust
/// Generate a CID for arbitrary bytes
pub fn generate_cid(bytes: &[u8], codec: u64) -> Result<Cid>

/// Generate a CID for typed content
pub fn generate_typed_cid<T: TypedContent>(content: &T) -> Result<Cid>
```

### Hash Functions

```rust
/// Compute BLAKE3 hash
pub fn blake3_hash(data: &[u8]) -> [u8; 32]

/// Compute SHA-256 hash (for compatibility)
pub fn sha256_hash(data: &[u8]) -> [u8; 32]
```

### Validation Helpers

```rust
/// Validate a codec is in allowed range
pub fn validate_codec(codec: u64) -> Result<()>

/// Check if codec is reserved for CIM core types
pub fn is_core_codec(codec: u64) -> bool

/// Check if codec is for custom types
pub fn is_custom_codec(codec: u64) -> bool
```

## Constants

```rust
/// Codec ranges
pub const CORE_CODEC_START: u64 = 0x300000;
pub const CORE_CODEC_END: u64 = 0x30FFFF;
pub const DOCUMENT_CODEC_START: u64 = 0x310000;
pub const DOCUMENT_CODEC_END: u64 = 0x31FFFF;
pub const MEDIA_CODEC_START: u64 = 0x320000;
pub const MEDIA_CODEC_END: u64 = 0x32FFFF;
pub const CUSTOM_CODEC_START: u64 = 0x330000;
pub const CUSTOM_CODEC_END: u64 = 0x3FFFFF;

/// Default codec for JSON content
pub const DEFAULT_CODEC: u64 = 0x0129;

/// Maximum chain length before requiring snapshots
pub const MAX_CHAIN_LENGTH: usize = 10_000;
```

## Examples

### Basic Usage

```rust
use cim_ipld::prelude::*;

// Define content type
#[derive(Serialize, Deserialize)]
struct MyData {
    id: String,
    value: i32,
}

impl TypedContent for MyData {
    const CODEC: u64 = 0x330001;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x330001);
}

// Store and retrieve
async fn example() -> Result<()> {
    let store = MemoryStore::new();

    let data = MyData {
        id: "test".to_string(),
        value: 42,
    };

    let cid = store.put_typed(&data).await?;
    let retrieved: MyData = store.get_typed(&cid).await?;

    assert_eq!(retrieved.value, 42);
    Ok(())
}
```

### Chain Example

```rust
// Create and validate chain
async fn chain_example() -> Result<()> {
    let mut chain = ContentChain::<MyData>::new();

    for i in 0..10 {
        let data = MyData {
            id: format!("item-{}", i),
            value: i,
        };
        chain.append(data)?;
    }

    // Validate chain integrity
    chain.validate()?;

    // Get recent items
    let head = chain.head().unwrap();
    let recent = chain.items_since(&chain.items()[5].cid)?;

    assert_eq!(recent.len(), 4);
    Ok(())
}
