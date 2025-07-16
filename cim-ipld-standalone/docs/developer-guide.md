# CIM-IPLD Developer Guide

## Table of Contents

1. [Getting Started](#getting-started)
2. [Basic Concepts](#basic-concepts)
3. [Working with Content](#working-with-content)
4. [Building Chains](#building-chains)
5. [Custom Codecs](#custom-codecs)
6. [Storage Backends](#storage-backends)
7. [Best Practices](#best-practices)
8. [Troubleshooting](#troubleshooting)
9. [Examples](#examples)

## Getting Started

### Installation

Add CIM-IPLD to your `Cargo.toml`:

```toml
[dependencies]
cim-ipld = { git = "https://github.com/thecowboyai/cim-ipld" }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
```

### Basic Setup

```rust
use cim_ipld::{TypedContent, ContentType, ChainedContent, ContentChain};
use serde::{Serialize, Deserialize};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Your code here
    Ok(())
}
```

## Basic Concepts

### Content Identifiers (CIDs)

A CID is a self-describing content address that uniquely identifies a piece of data:

```rust
use cim_ipld::Cid;

// CIDs are typically created automatically when storing content
let cid_string = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
let cid = Cid::try_from(cid_string)?;

// CIDs contain:
// - Version (0 or 1)
// - Codec (how data is encoded)
// - Multihash (cryptographic hash)
```

### Content Types

CIM-IPLD defines standard content types:

```rust
use cim_ipld::ContentType;

// Core types
ContentType::Event      // Domain events
ContentType::Graph      // Graph structures
ContentType::Node       // Graph nodes
ContentType::Edge       // Graph edges
ContentType::Command    // Commands
ContentType::Query      // Queries

// Document types
ContentType::Markdown   // Markdown documents
ContentType::Json       // JSON data
ContentType::Yaml       // YAML data
ContentType::Toml       // TOML configuration

// Media types
ContentType::Image      // Images
ContentType::Video      // Videos
ContentType::Audio      // Audio files

// Custom types
ContentType::Custom(0x330000)  // Your custom type
```

## Working with Content

### Defining Typed Content

Create strongly-typed content by implementing `TypedContent`:

```rust
use cim_ipld::{TypedContent, ContentType, Result};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserProfile {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: u64,
}

impl TypedContent for UserProfile {
    const CODEC: u64 = 0x330001;  // Custom codec in allowed range
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x330001);
}
```

### Storing Content

```rust
use cim_ipld::object_store::{ObjectStore, MemoryStore};

// Create a storage backend
let store = MemoryStore::new();

// Create content
let profile = UserProfile {
    id: "user-123".to_string(),
    username: "alice".to_string(),
    email: "alice@example.com".to_string(),
    created_at: 1234567890,
};

// Store content
let cid = store.put_typed(&profile).await?;
println!("Stored profile with CID: {}", cid);
```

### Retrieving Content

```rust
// Retrieve by CID
let retrieved: UserProfile = store.get_typed(&cid).await?;
assert_eq!(retrieved.username, "alice");

// Handle missing content
match store.get_typed::<UserProfile>(&unknown_cid).await {
    Ok(profile) => println!("Found: {}", profile.username),
    Err(e) => println!("Not found: {}", e),
}
```

## Building Chains

### Creating Event Chains

Chains provide tamper-evident sequences of events:

```rust
use cim_ipld::ContentChain;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum AuditEvent {
    UserLogin { user_id: String, ip: String },
    DataAccess { user_id: String, resource: String },
    UserLogout { user_id: String },
}

impl TypedContent for AuditEvent {
    const CODEC: u64 = 0x300010;
    const CONTENT_TYPE: ContentType = ContentType::Event;
}

// Create a new chain
let mut chain = ContentChain::<AuditEvent>::new();

// Add events
let event1 = chain.append(AuditEvent::UserLogin {
    user_id: "alice".to_string(),
    ip: "192.168.1.1".to_string(),
})?;

let event2 = chain.append(AuditEvent::DataAccess {
    user_id: "alice".to_string(),
    resource: "/api/users".to_string(),
})?;

// Each event has:
println!("Event CID: {}", event1.cid);
println!("Previous CID: {:?}", event1.previous_cid);
println!("Sequence: {}", event1.sequence);
println!("Timestamp: {:?}", event1.timestamp);
```

### Chain Validation

Validate chain integrity:

```rust
// Validate entire chain
chain.validate()?;

// Get chain head
let head = chain.head().unwrap();
println!("Chain head CID: {}", head.cid);

// Get items since a specific CID
let recent = chain.items_since(&event1.cid)?;
println!("Events since {}: {}", event1.cid, recent.len());

// Detect tampering
let mut tampered = event2.clone();
tampered.sequence = 999;  // Modify sequence
match tampered.validate_chain(Some(&event1.cid)) {
    Ok(_) => println!("Chain valid"),
    Err(e) => println!("Tampering detected: {}", e),
}
```

### Chain Persistence

Save and load chains:

```rust
// Save chain to storage
let chain_cid = chain.save(&store).await?;

// Load chain from storage
let loaded_chain = ContentChain::<AuditEvent>::load(&store, &chain_cid).await?;
assert_eq!(loaded_chain.len(), chain.len());
```

## Custom Codecs

### Implementing a Custom Codec

Create optimized codecs for your content types:

```rust
use cim_ipld::{CimCodec, CodecRegistry, Result};
use std::sync::Arc;

struct CompressedJsonCodec;

impl CimCodec for CompressedJsonCodec {
    fn code(&self) -> u64 {
        0x330100  // Custom codec identifier
    }

    fn name(&self) -> &str {
        "compressed-json"
    }

    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Implement compression
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    fn decode(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Implement decompression
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(data);
        let mut result = Vec::new();
        decoder.read_to_end(&mut result)?;
        Ok(result)
    }
}
```

### Registering Codecs

```rust
// Create codec registry
let mut registry = CodecRegistry::new();

// Register custom codec
registry.register(Arc::new(CompressedJsonCodec))?;

// Use with typed content
#[derive(Serialize, Deserialize)]
struct CompressedData {
    data: Vec<u8>,
}

impl TypedContent for CompressedData {
    const CODEC: u64 = 0x330100;  // Match codec code
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x330100);
}
```

## Storage Backends

### File System Storage

```rust
use cim_ipld::object_store::FileStore;
use std::path::Path;

// Create file-based storage
let store = FileStore::new(Path::new("/var/lib/cim-ipld"))?;

// Configure options
let store = FileStore::builder()
    .path("/var/lib/cim-ipld")
    .max_file_size(1024 * 1024)  // 1MB max file size
    .compression(true)
    .build()?;
```

### S3-Compatible Storage

```rust
use cim_ipld::object_store::S3Store;

// Create S3 storage
let store = S3Store::new(
    "my-bucket",
    "us-east-1",
    "access_key",
    "secret_key",
).await?;

// With custom endpoint (MinIO, etc.)
let store = S3Store::builder()
    .bucket("my-bucket")
    .endpoint("http://localhost:9000")
    .access_key("minioadmin")
    .secret_key("minioadmin")
    .build()
    .await?;
```

### Memory Storage (Testing)

```rust
use cim_ipld::object_store::MemoryStore;

// Create in-memory storage
let store = MemoryStore::new();

// With size limit
let store = MemoryStore::with_capacity(1024 * 1024 * 100);  // 100MB
```

### Custom Storage Backend

Implement your own storage backend:

```rust
use cim_ipld::object_store::{ObjectStore, Result};
use async_trait::async_trait;

struct MyCustomStore {
    // Your storage implementation
}

#[async_trait]
impl ObjectStore for MyCustomStore {
    async fn put(&self, key: &str, data: Vec<u8>) -> Result<()> {
        // Store data with key
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>> {
        // Retrieve data by key
        Ok(vec![])
    }

    async fn delete(&self, key: &str) -> Result<()> {
        // Delete data by key
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        // Check if key exists
        Ok(false)
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        // List keys with prefix
        Ok(vec![])
    }
}
```

## Best Practices

### 1. Content Design

```rust
// ✅ Good: Focused, single-purpose content
#[derive(Serialize, Deserialize)]
struct OrderCreated {
    order_id: String,
    customer_id: String,
    items: Vec<OrderItem>,
    total: f64,
    created_at: u64,
}

// ❌ Bad: Mixed concerns
#[derive(Serialize, Deserialize)]
struct OrderAndCustomer {
    order_data: serde_json::Value,
    customer_data: serde_json::Value,
    ui_state: serde_json::Value,  // Don't mix UI state
}
```

### 2. Chain Design

```rust
// ✅ Good: Logical event sequences
let mut order_chain = ContentChain::<OrderEvent>::new();
order_chain.append(OrderEvent::Created { ... })?;
order_chain.append(OrderEvent::Paid { ... })?;
order_chain.append(OrderEvent::Shipped { ... })?;

// ❌ Bad: Unrelated events in same chain
let mut mixed_chain = ContentChain::<MixedEvent>::new();
mixed_chain.append(MixedEvent::UserLogin { ... })?;
mixed_chain.append(MixedEvent::OrderCreated { ... })?;  // Different domain!
```

### 3. Error Handling

```rust
use cim_ipld::{Error, Result};

// Handle specific errors
match store.get_typed::<MyContent>(&cid).await {
    Ok(content) => process(content),
    Err(Error::NotFound(cid)) => {
        println!("Content {} not found", cid);
    }
    Err(Error::InvalidContent(msg)) => {
        println!("Content validation failed: {}", msg);
    }
    Err(e) => {
        println!("Unexpected error: {}", e);
    }
}

// Chain validation with context
if let Err(e) = chain.validate() {
    match e {
        Error::ChainValidation { sequence, expected, actual } => {
            println!("Chain broken at sequence {}: expected {}, got {}",
                     sequence, expected, actual);
        }
        _ => println!("Validation error: {}", e),
    }
}
```

### 4. Performance Optimization

```rust
// Batch operations for better performance
let contents = vec![content1, content2, content3];
let cids = store.put_batch(contents).await?;

// Use streaming for large files
use tokio::io::AsyncReadExt;
let mut file = tokio::fs::File::open("large_file.dat").await?;
let mut buffer = vec![0; 8192];  // 8KB chunks

while let Ok(n) = file.read(&mut buffer).await {
    if n == 0 { break; }
    // Process chunk
}

// Cache frequently accessed content
use lru::LruCache;
let mut cache = LruCache::<Cid, MyContent>::new(1000);
```

## Troubleshooting

### Common Issues

#### 1. CID Mismatch

**Problem**: "CID mismatch: expected X, got Y"

**Solution**: Content was modified after CID generation
```rust
// Ensure content is immutable
let content = MyContent { ... };
let cid = generate_cid(&content)?;
// Don't modify content after this point!
```

#### 2. Chain Validation Failure

**Problem**: "Chain validation failed at sequence N"

**Solution**: Check for gaps or tampering
```rust
// Debug chain issues
for (i, item) in chain.items().iter().enumerate() {
    println!("Item {}: CID={}, Prev={:?}, Seq={}",
             i, item.cid, item.previous_cid, item.sequence);
}
```

#### 3. Codec Not Found

**Problem**: "No codec registered for code X"

**Solution**: Register codec before use
```rust
let mut registry = CodecRegistry::new();
registry.register(Arc::new(MyCodec))?;
// Now you can use content with this codec
```

### Performance Issues

#### Slow Chain Validation

For long chains, validate incrementally:
```rust
// Instead of validating entire chain
chain.validate()?;  // O(n)

// Validate only new items
let last_validated = get_last_validated_cid();
chain.validate_since(&last_validated)?;  // O(k) where k < n
```

#### High Memory Usage

Use streaming for large content:
```rust
// Instead of loading entire file
let data = std::fs::read("large_file")?;

// Stream in chunks
use cim_ipld::streaming::StreamingStore;
let cid = store.put_stream(file_stream).await?;
```

## Examples

### Complete Event Sourcing Example

```rust
use cim_ipld::{TypedContent, ContentType, ContentChain, object_store::FileStore};
use serde::{Serialize, Deserialize};
use std::path::Path;

// Define domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
enum AccountEvent {
    Created { id: String, owner: String },
    Deposited { amount: f64 },
    Withdrawn { amount: f64 },
    Closed { reason: String },
}

impl TypedContent for AccountEvent {
    const CODEC: u64 = 0x300020;
    const CONTENT_TYPE: ContentType = ContentType::Event;
}

// Event sourced aggregate
struct Account {
    id: String,
    balance: f64,
    is_closed: bool,
    events: ContentChain<AccountEvent>,
}

impl Account {
    fn new(id: String, owner: String) -> Self {
        let mut account = Self {
            id: id.clone(),
            balance: 0.0,
            is_closed: false,
            events: ContentChain::new(),
        };

        account.apply_event(AccountEvent::Created { id, owner });
        account
    }

    fn deposit(&mut self, amount: f64) -> Result<()> {
        if self.is_closed {
            return Err("Account is closed".into());
        }

        self.apply_event(AccountEvent::Deposited { amount });
        Ok(())
    }

    fn withdraw(&mut self, amount: f64) -> Result<()> {
        if self.is_closed {
            return Err("Account is closed".into());
        }

        if self.balance < amount {
            return Err("Insufficient funds".into());
        }

        self.apply_event(AccountEvent::Withdrawn { amount });
        Ok(())
    }

    fn close(&mut self, reason: String) {
        self.apply_event(AccountEvent::Closed { reason });
    }

    fn apply_event(&mut self, event: AccountEvent) {
        // Update state
        match &event {
            AccountEvent::Created { .. } => {},
            AccountEvent::Deposited { amount } => self.balance += amount,
            AccountEvent::Withdrawn { amount } => self.balance -= amount,
            AccountEvent::Closed { .. } => self.is_closed = true,
        }

        // Store event
        self.events.append(event).unwrap();
    }

    async fn save(&self, store: &FileStore) -> Result<Cid> {
        self.events.save(store).await
    }

    async fn load(store: &FileStore, cid: &Cid) -> Result<Self> {
        let events = ContentChain::<AccountEvent>::load(store, cid).await?;

        // Replay events to rebuild state
        let mut account = Self {
            id: String::new(),
            balance: 0.0,
            is_closed: false,
            events: ContentChain::new(),
        };

        for event in events.items() {
            match &event.content {
                AccountEvent::Created { id, .. } => account.id = id.clone(),
                AccountEvent::Deposited { amount } => account.balance += amount,
                AccountEvent::Withdrawn { amount } => account.balance -= amount,
                AccountEvent::Closed { .. } => account.is_closed = true,
            }
        }

        account.events = events;
        Ok(account)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create storage
    let store = FileStore::new(Path::new("./data"))?;

    // Create account
    let mut account = Account::new("acc-123".to_string(), "alice".to_string());

    // Perform operations
    account.deposit(1000.0)?;
    account.withdraw(250.0)?;
    account.deposit(500.0)?;

    println!("Balance: ${}", account.balance);
    println!("Events: {}", account.events.len());

    // Save to storage
    let cid = account.save(&store).await?;
    println!("Saved with CID: {}", cid);

    // Load from storage
    let loaded = Account::load(&store, &cid).await?;
    println!("Loaded balance: ${}", loaded.balance);

    // Validate event chain
    loaded.events.validate()?;
    println!("Event chain validated successfully");

    Ok(())
}
```

This developer guide provides comprehensive documentation for working with CIM-IPLD, from basic concepts to advanced usage patterns.
