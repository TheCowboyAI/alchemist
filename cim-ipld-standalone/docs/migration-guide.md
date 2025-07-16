# CIM-IPLD Migration Guide

## Table of Contents

1. [Overview](#overview)
2. [Migrating from Traditional Databases](#migrating-from-traditional-databases)
3. [Migrating from IPFS](#migrating-from-ipfs)
4. [Migrating from Event Stores](#migrating-from-event-stores)
5. [Migrating from Object Storage](#migrating-from-object-storage)
6. [Data Migration Strategies](#data-migration-strategies)
7. [Common Patterns](#common-patterns)
8. [Tools and Utilities](#tools-and-utilities)

## Overview

This guide helps developers migrate existing applications to CIM-IPLD from various storage systems. CIM-IPLD provides content-addressed storage with cryptographic integrity, making it ideal for event-sourced systems, audit logs, and immutable data storage.

### Key Differences

| Feature | Traditional DB | CIM-IPLD |
|---------|---------------|----------|
| Addressing | By ID/Key | By Content (CID) |
| Mutability | Mutable | Immutable |
| Integrity | Application-level | Cryptographic |
| History | Optional | Built-in (chains) |
| Schema | Fixed | Flexible (codecs) |

## Migrating from Traditional Databases

### From SQL Databases

#### Before (PostgreSQL)
```sql
-- Traditional mutable table
CREATE TABLE events (
    id SERIAL PRIMARY KEY,
    aggregate_id UUID NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Update existing record
UPDATE events SET payload = '{"status": "completed"}' WHERE id = 123;
```

#### After (CIM-IPLD)
```rust
use cim_ipld::{TypedContent, ContentType, ContentChain};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Event {
    aggregate_id: String,
    event_type: String,
    payload: serde_json::Value,
    created_at: u64,
}

impl TypedContent for Event {
    const CODEC: u64 = 0x300000;
    const CONTENT_TYPE: ContentType = ContentType::Event;
}

// Create immutable event chain
let mut chain = ContentChain::<Event>::new();

// Add events (no updates, only appends)
let event = Event {
    aggregate_id: "user-123".to_string(),
    event_type: "StatusChanged".to_string(),
    payload: json!({"status": "completed"}),
    created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
};

let chained = chain.append(event)?;
println!("Event CID: {}", chained.cid);
```

### From NoSQL Databases

#### Before (MongoDB)
```javascript
// Mutable document store
db.users.insertOne({
    _id: ObjectId(),
    username: "alice",
    email: "alice@example.com",
    profile: { bio: "Developer" }
});

// Update document
db.users.updateOne(
    { username: "alice" },
    { $set: { "profile.bio": "Senior Developer" } }
);
```

#### After (CIM-IPLD)
```rust
#[derive(Serialize, Deserialize)]
struct UserProfile {
    username: String,
    email: String,
    bio: String,
}

#[derive(Serialize, Deserialize)]
struct UserProfileUpdate {
    user_id: String,
    previous_cid: Option<Cid>,
    profile: UserProfile,
    updated_at: u64,
}

impl TypedContent for UserProfileUpdate {
    const CODEC: u64 = 0x330001;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x330001);
}

// Store immutable versions
let update = UserProfileUpdate {
    user_id: "user-123".to_string(),
    previous_cid: Some(previous_version_cid),
    profile: UserProfile {
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        bio: "Senior Developer".to_string(),
    },
    updated_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
};

let cid = store.put_typed(&update).await?;
```

### Migration Strategy

1. **Identify Immutable Data**: Start with append-only data (logs, events, history)
2. **Version Mutable Data**: Convert updates to versioned records
3. **Maintain Index**: Keep a mapping of logical IDs to latest CIDs

```rust
// ID to CID mapping for latest versions
pub struct EntityIndex {
    index: HashMap<String, Cid>,
}

impl EntityIndex {
    pub async fn update(&mut self, entity_id: &str, new_cid: Cid) {
        self.index.insert(entity_id.to_string(), new_cid);
    }

    pub fn get_latest(&self, entity_id: &str) -> Option<&Cid> {
        self.index.get(entity_id)
    }
}
```

## Migrating from IPFS

CIM-IPLD is IPLD-compatible but adds domain-specific features:

### Before (go-ipfs)
```go
// Basic IPFS storage
data := map[string]interface{}{
    "name": "example",
    "value": 42,
}

cid, err := ipfs.Dag().Put(ctx, data)
```

### After (CIM-IPLD)
```rust
// Typed content with custom codec
#[derive(Serialize, Deserialize)]
struct ExampleData {
    name: String,
    value: i32,
}

impl TypedContent for ExampleData {
    const CODEC: u64 = 0x330002;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x330002);
}

// Store with type safety
let data = ExampleData {
    name: "example".to_string(),
    value: 42,
};

let cid = store.put_typed(&data).await?;
```

### Key Improvements

1. **Type Safety**: Compile-time guarantees
2. **Custom Codecs**: Domain-specific serialization
3. **Chain Support**: Built-in linked data structures
4. **Event Sourcing**: Native event chain support

## Migrating from Event Stores

### From EventStore DB

#### Before
```csharp
// EventStore stream
var events = new[] {
    new EventData(
        Guid.NewGuid(),
        "OrderCreated",
        true,
        Encoding.UTF8.GetBytes(JsonConvert.SerializeObject(orderData)),
        null
    )
};

await connection.AppendToStreamAsync("order-123", ExpectedVersion.Any, events);
```

#### After
```rust
use cim_ipld::{ContentChain, TypedContent};

#[derive(Serialize, Deserialize)]
enum OrderEvent {
    Created { order_id: String, customer: String, items: Vec<Item> },
    Updated { changes: HashMap<String, Value> },
    Shipped { tracking_number: String },
    Completed,
}

impl TypedContent for OrderEvent {
    const CODEC: u64 = 0x300010;
    const CONTENT_TYPE: ContentType = ContentType::Event;
}

// Event chain per aggregate
let mut order_chain = ContentChain::<OrderEvent>::new();

// Append events
order_chain.append(OrderEvent::Created {
    order_id: "order-123".to_string(),
    customer: "alice".to_string(),
    items: vec![item1, item2],
})?;

order_chain.append(OrderEvent::Shipped {
    tracking_number: "TRACK-123".to_string(),
})?;

// Save chain
let chain_cid = order_chain.save(&store).await?;
```

### Migration Benefits

1. **Cryptographic Integrity**: Each event is hash-linked
2. **No Central Server**: Distributed by design
3. **Content Addressing**: Reference events by content, not position
4. **Built-in Validation**: Chain integrity checking

## Migrating from Object Storage

### From S3/MinIO

#### Before
```python
# S3 object storage
s3.put_object(
    Bucket='my-bucket',
    Key='data/user-123/profile.json',
    Body=json.dumps(profile_data)
)

# Versioning through S3
s3.put_object(
    Bucket='my-bucket',
    Key='data/user-123/profile.json',
    Body=json.dumps(updated_profile),
    Metadata={'version': '2'}
)
```

#### After
```rust
// Content-addressed storage
let profile_v1 = UserProfile { /* ... */ };
let cid_v1 = store.put_typed(&profile_v1).await?;

// New version links to previous
let profile_v2 = VersionedContent {
    previous: Some(cid_v1),
    content: UserProfile { /* updated */ },
    version: 2,
};
let cid_v2 = store.put_typed(&profile_v2).await?;

// S3 backend support
use cim_ipld::object_store::S3Store;

let store = S3Store::builder()
    .bucket("cim-ipld-data")
    .prefix("production/")
    .build()
    .await?;
```

## Data Migration Strategies

### 1. Parallel Run Strategy

Run both systems in parallel during migration:

```rust
// Dual write during migration
async fn save_data(data: &MyData) -> Result<()> {
    // Write to legacy system
    legacy_db.save(data).await?;

    // Write to CIM-IPLD
    let cid = ipld_store.put_typed(data).await?;

    // Update mapping
    migration_index.map_legacy_id(data.id, cid).await?;

    Ok(())
}
```

### 2. Batch Migration

Migrate historical data in batches:

```rust
use futures::stream::{self, StreamExt};

async fn migrate_batch(legacy_db: &LegacyDB, store: &impl ObjectStore) -> Result<()> {
    let batch_size = 1000;
    let mut offset = 0;

    loop {
        // Fetch batch from legacy system
        let records = legacy_db.fetch_batch(offset, batch_size).await?;
        if records.is_empty() {
            break;
        }

        // Convert and store in parallel
        let results: Vec<_> = stream::iter(records)
            .map(|record| async move {
                let converted = convert_to_ipld(record)?;
                let cid = store.put_typed(&converted).await?;
                Ok((record.id, cid))
            })
            .buffer_unordered(10)
            .collect()
            .await;

        // Update index
        for result in results {
            let (id, cid) = result?;
            migration_index.add(id, cid).await?;
        }

        offset += batch_size;
    }

    Ok(())
}
```

### 3. Event Replay Strategy

For event-sourced systems:

```rust
async fn replay_events(legacy_events: Vec<LegacyEvent>) -> Result<ContentChain<Event>> {
    let mut chain = ContentChain::<Event>::new();

    for legacy_event in legacy_events {
        let event = convert_event(legacy_event)?;
        chain.append(event)?;
    }

    // Validate migrated chain
    chain.validate()?;

    Ok(chain)
}
```

## Common Patterns

### 1. Versioned Records

```rust
#[derive(Serialize, Deserialize)]
struct VersionedRecord<T> {
    version: u32,
    previous_cid: Option<Cid>,
    data: T,
    updated_at: SystemTime,
    updated_by: String,
}

impl<T: Serialize + DeserializeOwned> TypedContent for VersionedRecord<T> {
    const CODEC: u64 = 0x330100;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x330100);
}
```

### 2. Snapshot + Events Pattern

```rust
#[derive(Serialize, Deserialize)]
struct AggregateSnapshot {
    aggregate_id: String,
    version: u64,
    state: serde_json::Value,
    events_since_snapshot: Vec<Cid>,
}

async fn load_aggregate(id: &str) -> Result<Aggregate> {
    // Load latest snapshot
    let snapshot_cid = index.get_latest_snapshot(id)?;
    let snapshot: AggregateSnapshot = store.get_typed(&snapshot_cid).await?;

    // Replay events since snapshot
    let mut aggregate = Aggregate::from_snapshot(snapshot.state)?;

    for event_cid in snapshot.events_since_snapshot {
        let event = store.get_typed(&event_cid).await?;
        aggregate.apply_event(event)?;
    }

    Ok(aggregate)
}
```

### 3. Index Maintenance

```rust
#[derive(Serialize, Deserialize)]
struct IndexEntry {
    entity_type: String,
    entity_id: String,
    latest_cid: Cid,
    version: u64,
    updated_at: SystemTime,
}

pub struct SecondaryIndex {
    by_type: HashMap<String, Vec<Cid>>,
    by_date: BTreeMap<SystemTime, Vec<Cid>>,
}
```

## Tools and Utilities

### Migration CLI Tool

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct MigrationCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze legacy data
    Analyze {
        #[arg(short, long)]
        source: String,
    },

    /// Migrate data
    Migrate {
        #[arg(short, long)]
        source: String,
        #[arg(short, long)]
        target: String,
        #[arg(long)]
        batch_size: Option<usize>,
    },

    /// Verify migration
    Verify {
        #[arg(short, long)]
        legacy_id: String,
    },
}
```

### Data Validation

```rust
pub struct MigrationValidator {
    legacy_db: Box<dyn LegacyDatabase>,
    ipld_store: Box<dyn ObjectStore>,
}

impl MigrationValidator {
    pub async fn validate_record(&self, legacy_id: &str) -> Result<ValidationResult> {
        // Fetch from both systems
        let legacy_record = self.legacy_db.get(legacy_id).await?;
        let cid = self.index.get_cid(legacy_id)?;
        let ipld_record = self.ipld_store.get_typed(&cid).await?;

        // Compare
        let matches = compare_records(&legacy_record, &ipld_record)?;

        Ok(ValidationResult {
            legacy_id: legacy_id.to_string(),
            cid,
            matches,
            differences: if !matches {
                Some(find_differences(&legacy_record, &ipld_record))
            } else {
                None
            },
        })
    }
}
```

### Progress Tracking

```rust
pub struct MigrationProgress {
    total_records: u64,
    migrated_records: u64,
    failed_records: u64,
    start_time: Instant,
}

impl MigrationProgress {
    pub fn update(&mut self, success: bool) {
        if success {
            self.migrated_records += 1;
        } else {
            self.failed_records += 1;
        }
    }

    pub fn report(&self) {
        let elapsed = self.start_time.elapsed();
        let rate = self.migrated_records as f64 / elapsed.as_secs_f64();

        println!("Migration Progress:");
        println!("  Total: {}", self.total_records);
        println!("  Migrated: {} ({:.1}%)",
            self.migrated_records,
            (self.migrated_records as f64 / self.total_records as f64) * 100.0
        );
        println!("  Failed: {}", self.failed_records);
        println!("  Rate: {:.1} records/sec", rate);

        if rate > 0.0 {
            let remaining = self.total_records - self.migrated_records - self.failed_records;
            let eta = Duration::from_secs_f64(remaining as f64 / rate);
            println!("  ETA: {:?}", eta);
        }
    }
}
```

## Best Practices

1. **Start Small**: Migrate non-critical data first
2. **Maintain Indexes**: Keep mappings between old IDs and CIDs
3. **Validate Thoroughly**: Compare data after migration
4. **Plan Rollback**: Keep legacy system running during transition
5. **Monitor Performance**: Track migration progress and system load
6. **Document Mappings**: Record how legacy concepts map to CIM-IPLD

## Troubleshooting

### Common Issues

1. **Performance**: Use batch operations and parallel processing
2. **Memory Usage**: Stream large datasets instead of loading all at once
3. **ID Mapping**: Maintain bidirectional mapping for rollback capability
4. **Data Validation**: Implement checksums or hash comparison
5. **Schema Evolution**: Plan for future schema changes

### Recovery Strategies

```rust
// Checkpoint-based recovery
pub struct MigrationCheckpoint {
    last_processed_id: String,
    processed_count: u64,
    checkpoint_time: SystemTime,
}

// Resume from checkpoint
pub async fn resume_migration(checkpoint: MigrationCheckpoint) -> Result<()> {
    println!("Resuming from checkpoint: {} records processed",
             checkpoint.processed_count);

    // Continue from last processed ID
    migrate_from_id(&checkpoint.last_processed_id).await
}
```

This migration guide provides comprehensive strategies for moving from various storage systems to CIM-IPLD while maintaining data integrity and system availability.
