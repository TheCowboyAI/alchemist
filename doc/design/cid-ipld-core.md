# Core CID/IPLD Implementation

> Part of the [CID/IPLD Architecture](./cid-ipld-architecture.md)

## Overview

This document covers the foundational implementation of Content Identifiers (CIDs) and IPLD integration in CIM. CIDs provide content-addressed storage with cryptographic integrity, while IPLD enables typed, linked data structures.

## CID Creation

### Basic Implementation

```rust
use cid::Cid;
use multihash::{Code, MultihashDigest};

// Create a CID from content
pub fn create_cid(content: &[u8]) -> Result<Cid> {
    // Use BLAKE3 for performance and security
    let hash = Code::Blake3_256.digest(content);

    // Create CID v1 with raw codec
    Ok(Cid::new_v1(0x55, hash))
}
```

### Multihash Selection

CIM uses different hash functions based on content type and compatibility requirements:

```rust
use multihash::Multihash;
use blake3;

// Option 1: Direct BLAKE3 usage (problematic - manual construction)
let hash = blake3::hash(content);
let mh = Multihash::wrap(0x1e, hash.as_bytes())?;

// Option 2: Using multihash crate (recommended)
let mh = Code::Blake3_256.digest(content);

// Option 3: Custom hasher implementation
pub struct Blake3Hasher;
impl MultihashDigest for Blake3Hasher {
    // Implementation details
}
```

**Recommendation**: Use Option 2 for simplicity and correctness.

## TypedContent Trait

The `TypedContent` trait provides type-safe content handling:

```rust
pub trait TypedContent: Serialize + DeserializeOwned {
    /// Returns the content type for routing and processing
    fn content_type(&self) -> ContentType;

    /// Returns the IPLD codec for this content type
    fn codec(&self) -> u64;

    /// Serializes the content to bytes
    fn to_bytes(&self) -> Result<Vec<u8>>;

    /// Creates a CID for this content
    fn to_cid(&self) -> Result<Cid> {
        let bytes = self.to_bytes()?;
        let hash = blake3::hash(&bytes);
        let mh = Multihash::wrap(0x1e, hash.as_bytes())?;
        Ok(Cid::new_v1(self.codec(), mh))
    }
}
```

## CID Chain Implementation

For sequential data like events, CIM implements CID chains:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CidChain {
    pub current_cid: Cid,
    pub previous_cid: Option<Cid>,
    pub height: u64,
    pub timestamp: SystemTime,
}

impl CidChain {
    pub fn new_genesis(content: &[u8]) -> Result<Self> {
        let cid = create_cid(content)?;
        Ok(Self {
            current_cid: cid,
            previous_cid: None,
            height: 0,
            timestamp: SystemTime::now(),
        })
    }

    pub fn append(&self, content: &[u8]) -> Result<Self> {
        // Include previous CID in hash calculation
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.current_cid.to_bytes().as_slice());
        hasher.update(content);

        let hash = hasher.finalize();
        let mh = Multihash::wrap(0x1e, hash.as_bytes())?;
        let cid = Cid::new_v1(0x55, mh);

        Ok(Self {
            current_cid: cid,
            previous_cid: Some(self.current_cid),
            height: self.height + 1,
            timestamp: SystemTime::now(),
        })
    }
}
```

## IPLD Integration

### Basic IPLD Node

```rust
use ipld::Ipld;

pub fn content_to_ipld(content: &TypedContent) -> Result<Ipld> {
    let value = serde_json::to_value(content)?;
    Ok(ipld_from_json(value))
}

pub fn ipld_to_content<T: TypedContent>(ipld: &Ipld) -> Result<T> {
    let value = json_from_ipld(ipld)?;
    Ok(serde_json::from_value(value)?)
}
```

### IPLD Schema Validation

```rust
pub struct IpldSchema {
    pub name: String,
    pub version: String,
    pub fields: Vec<FieldDefinition>,
}

impl IpldSchema {
    pub fn validate(&self, ipld: &Ipld) -> Result<()> {
        // Validate IPLD structure against schema
        for field in &self.fields {
            field.validate(ipld)?;
        }
        Ok(())
    }
}
```

## Performance Considerations

1. **Hash Function Selection**
   - BLAKE3 for general content (fast, secure)
   - SHA-256 for Git compatibility
   - Custom hashers for specific requirements

2. **CID Caching**
   - Cache computed CIDs to avoid re-hashing
   - Use weak references for memory efficiency

3. **Batch Operations**
   - Process multiple CIDs in parallel
   - Use streaming for large content

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum CidError {
    #[error("Invalid multihash: {0}")]
    MultihashError(#[from] multihash::Error),

    #[error("Serialization failed: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid content type: {0}")]
    InvalidContentType(String),
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cid_creation() {
        let content = b"Hello, CIM!";
        let cid = create_cid(content).unwrap();

        // Verify CID properties
        assert_eq!(cid.version(), cid::Version::V1);
        assert_eq!(cid.codec(), 0x55); // Raw codec
    }

    #[test]
    fn test_cid_chain() {
        let chain = CidChain::new_genesis(b"Genesis").unwrap();
        let chain2 = chain.append(b"Block 1").unwrap();

        assert_eq!(chain2.height, 1);
        assert_eq!(chain2.previous_cid, Some(chain.current_cid));
    }
}
```

## Related Documents

- [Content Types and Codecs](./cid-ipld-content-types.md) - Domain-specific content types
- [Event and Object Stores](./cid-ipld-stores.md) - Storage implementation
- [MIME Type Intelligence](./cid-ipld-mime-filegroups.md) - Dynamic content detection

## Next Steps

1. Implement the `TypedContent` trait for your domain types
2. Set up CID chains for event sourcing
3. Configure appropriate hash functions for your content types
