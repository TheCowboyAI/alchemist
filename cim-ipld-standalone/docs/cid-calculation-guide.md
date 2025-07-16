# CID Calculation Guide for CIM-IPLD

## Overview

Content Identifiers (CIDs) in CIM-IPLD are cryptographic hashes that uniquely identify content. A critical principle is that **identical content must always produce the same CID**, regardless of when or where it's created.

## The Problem with Metadata

Many data structures include metadata that changes with each instance:
- Message IDs (UUIDs)
- Timestamps
- Correlation IDs
- Sender information
- Routing headers

If we include this metadata in CID calculation, identical payloads would produce different CIDs, breaking content deduplication and making it impossible to verify that two messages contain the same data.

## The Solution: Canonical Payloads

CIM-IPLD introduces the concept of **canonical payloads** - the stable, meaningful content extracted from a data structure, excluding transient metadata.

### The `canonical_payload` Method

The `TypedContent` trait provides a `canonical_payload()` method that implementations can override:

```rust
pub trait TypedContent {
    /// Extract the canonical payload for CID calculation
    fn canonical_payload(&self) -> Result<Vec<u8>> {
        // Default: serialize entire struct
        self.to_bytes()
    }
}
```

## Implementation Patterns

### Pattern 1: Domain Events

Domain events typically have metadata (event ID, timestamp) and payload (the actual event data):

```rust
#[derive(Serialize, Deserialize)]
pub struct DomainEvent {
    // Metadata - excluded from CID
    pub event_id: String,
    pub timestamp: String,
    pub correlation_id: Option<String>,

    // Payload - included in CID
    pub event_type: String,
    pub aggregate_id: String,
    pub data: EventData,
}

impl TypedContent for DomainEvent {
    fn canonical_payload(&self) -> Result<Vec<u8>> {
        // Only include stable fields
        let canonical = json!({
            "event_type": self.event_type,
            "aggregate_id": self.aggregate_id,
            "data": self.data,
        });
        Ok(serde_json::to_vec(&canonical)?)
    }
}
```

### Pattern 2: Message Envelopes

Message wrappers should extract their content:

```rust
#[derive(Serialize, Deserialize)]
pub struct Message<T> {
    pub id: String,
    pub headers: HashMap<String, String>,
    pub content: T,
}

impl<T: Serialize> TypedContent for Message<T> {
    fn canonical_payload(&self) -> Result<Vec<u8>> {
        // Only the content matters
        Ok(serde_json::to_vec(&self.content)?)
    }
}
```

### Pattern 3: Versioned Data

For data that evolves over time, include version in the canonical form:

```rust
#[derive(Serialize, Deserialize)]
pub struct VersionedData {
    pub version: u32,
    pub data: serde_json::Value,
    pub updated_at: String, // Excluded
    pub updated_by: String, // Excluded
}

impl TypedContent for VersionedData {
    fn canonical_payload(&self) -> Result<Vec<u8>> {
        let canonical = json!({
            "version": self.version,
            "data": self.data,
        });
        Ok(serde_json::to_vec(&canonical)?)
    }
}
```

## Best Practices

### 1. Identify Stable vs Transient Fields

**Stable fields** (include in CID):
- Business data
- Aggregate IDs
- Event types
- Domain-specific content

**Transient fields** (exclude from CID):
- Timestamps
- Message IDs
- Correlation IDs
- System metadata
- Routing information

### 2. Document Canonical Form

Always document what fields are included in the canonical payload:

```rust
/// Canonical payload includes: event_type, aggregate_id, data
/// Excludes: event_id, timestamp, correlation_id
impl TypedContent for MyEvent {
    // ...
}
```

### 3. Test CID Consistency

Write tests to ensure identical payloads produce identical CIDs:

```rust
#[test]
fn test_cid_consistency() {
    let payload = MyData { value: 42 };

    let msg1 = Message {
        id: "msg-1",
        timestamp: "2024-01-01",
        content: payload.clone(),
    };

    let msg2 = Message {
        id: "msg-2",
        timestamp: "2024-01-02",
        content: payload,
    };

    assert_eq!(
        msg1.calculate_cid().unwrap(),
        msg2.calculate_cid().unwrap()
    );
}
```

## Common Pitfalls

### ❌ Including Timestamps

```rust
// WRONG - timestamp makes every CID unique
fn canonical_payload(&self) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec(self)?) // Includes timestamp!
}
```

### ❌ Including Generated IDs

```rust
// WRONG - UUID changes every time
struct Event {
    id: Uuid, // Different each time!
    data: String,
}
```

### ✅ Correct Implementation

```rust
// RIGHT - only stable business data
fn canonical_payload(&self) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec(&self.data)?)
}
```

## Benefits

1. **Content Deduplication**: Identical content stored only once
2. **Verification**: Can verify content matches expected CID
3. **Caching**: Same content always has same CID
4. **Integrity**: CID proves content hasn't changed

## Example Use Cases

### Event Deduplication

Multiple systems might emit the same event:
- System A: `{id: "123", time: "10:00", event: "UserCreated", user: "alice"}`
- System B: `{id: "456", time: "10:01", event: "UserCreated", user: "alice"}`

With canonical payloads, both produce the same CID for the `UserCreated` event.

### Content Verification

When receiving content, you can verify it matches an expected CID:

```rust
let expected_cid = "bafyreif...";
let received_content = get_content();
let actual_cid = received_content.calculate_cid()?;

assert_eq!(expected_cid, actual_cid.to_string());
```

### Migration and Replay

During event replay or migration, events might get new timestamps or IDs, but their canonical CIDs remain constant, preserving content integrity.

## Conclusion

Proper CID calculation through canonical payloads is essential for:
- Content-addressed storage
- Deduplication
- Integrity verification
- Cross-system content sharing

Always design your `TypedContent` implementations with canonical payloads in mind, separating stable business data from transient metadata.
