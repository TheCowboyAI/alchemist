# CIM-IPLD Encryption Design

## Overview

The encryption capability in cim-ipld provides a unique approach to content-addressed storage with privacy preservation. The key innovation is maintaining the original content's CID in metadata while the encrypted content gets its own CID, enabling searchability without exposing content.

## Core Concept

```
Original Content → Original CID
       ↓
   Encryption
       ↓
Encrypted Content → Encrypted CID
                    (contains Original CID in metadata)
```

This design enables:
- **Privacy**: Content is encrypted with strong encryption algorithms
- **Searchability**: Can find encrypted content by its original CID
- **Relationships**: Can maintain graph relationships between encrypted items
- **Access Control**: Fine-grained key management and access policies

## Encryption Flow

### 1. Basic Encryption
```rust
// Original content gets CID
let original_cid = calculate_cid(content, codec);

// Encrypt content
let encrypted = EncryptedContent::encrypt(
    content,
    &key,
    EncryptionType::AES256_GCM,
)?;

// Encrypted content has different CID but preserves original
let encrypted_cid = encrypted.calculate_cid()?;
assert_ne!(original_cid, encrypted_cid);
assert_eq!(encrypted.metadata.original_cid, original_cid);
```

### 2. Searchable Encryption
```rust
// Store encrypted content
let encrypted_cid = store.store_encrypted(content, &key, enc_type).await?;

// Later: Find by original CID without decrypting
let result = store.find_by_original_cid(&original_cid).await?;
let (encrypted_cid, metadata) = result.unwrap();

// Retrieve and decrypt only when needed
let encrypted = store.get_encrypted(&encrypted_cid).await?;
let decrypted = encrypted.decrypt(&key)?;
```

## Key Features

### 1. CID Preservation
- Original content CID is calculated before encryption
- Stored in encrypted content metadata
- Enables searching without decryption
- Maintains content relationships

### 2. Multiple Encryption Types
- AES-128-GCM (legacy support)
- AES-256-GCM (recommended)
- ChaCha20-Poly1305 (performance)
- XChaCha20-Poly1305 (large nonces)

### 3. Key Management Hierarchy
```
Master Key
    ├── Key Encryption Keys (KEK)
    │   └── Data Encryption Keys (DEK)
    └── User/Group Keys
        └── Access to DEKs
```

### 4. Access Control
- User-based access
- Group-based access
- Time-based expiry
- Audit trails

### 5. Advanced Features

#### Encrypted Relationships
- Maintain graph structure without exposing content
- Traverse relationships using original CIDs
- Build knowledge graphs on encrypted data

#### Selective Processing
- Decrypt with one key
- Transform content
- Re-encrypt with different key
- Useful for data migration, redaction

#### Zero-Knowledge Search
- Search without revealing search terms
- Server cannot see queries or content
- Client-side search token generation

#### Migration Support
- Upgrade encryption schemes
- Batch processing with progress tracking
- Rollback capability
- Zero downtime migration

## Use Cases

### 1. Private Knowledge Management
- Store sensitive documents
- Maintain relationships between documents
- Search without exposing content
- Share with specific users/groups

### 2. Compliance and Redaction
- Store original content encrypted
- Process to redact sensitive information
- Re-encrypt redacted version
- Maintain audit trail

### 3. Multi-tenant Systems
- Each tenant has own encryption keys
- Content isolation at encryption level
- Shared infrastructure, isolated data
- Key rotation per tenant

### 4. Secure Content Distribution
- Encrypt content once
- Distribute encrypted CID
- Recipients with keys can decrypt
- Revoke access by key management

## Implementation Requirements

### 1. Encryption Module
```rust
pub mod encryption {
    pub struct EncryptedContent {
        ciphertext: Vec<u8>,
        metadata: EncryptionMetadata,
        nonce: Vec<u8>,
    }
    
    pub struct EncryptionMetadata {
        original_cid: Cid,
        encryption_type: EncryptionType,
        encrypted_at: u64,
        key_id: Option<Uuid>,
        policy_id: Option<Uuid>,
    }
}
```

### 2. Storage Integration
- Extend NatsObjectStore for encrypted storage
- Metadata indexing for original CID lookup
- Batch operations for migration

### 3. Key Management Service
- Secure key generation
- Key derivation functions
- Access policy enforcement
- Key rotation support

### 4. Search Infrastructure
- Encrypted index structures
- Search token generation
- Blinded query processing
- Result ranking without decryption

## Security Considerations

1. **Key Storage**: Keys should never be stored with encrypted content
2. **Nonce Reuse**: Each encryption must use unique nonce
3. **Metadata Protection**: Metadata itself may need encryption
4. **Side Channels**: Consider timing attacks in search operations
5. **Key Derivation**: Use proper KDF for key generation
6. **Access Logs**: Audit all decryption operations

## Future Enhancements

1. **Homomorphic Operations**: Compute on encrypted data
2. **Threshold Encryption**: Require N-of-M keys to decrypt
3. **Format-Preserving Encryption**: Maintain data structure
4. **Searchable Encryption v2**: More complex queries on encrypted data
5. **Post-Quantum Algorithms**: Future-proof encryption schemes 