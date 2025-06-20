# Event Store NATS JetStream Integration Plan

## Overview

This plan outlines the implementation of event store integration with NATS JetStream for the cim-domain library. The event store will provide persistent, distributed event storage with CID chains for integrity verification.

## Current State

### Completed:
1. **Domain Model**: All 7 aggregates implemented (Person, Organization, Agent, Location, Policy, Document, Workflow)
2. **Command Handlers**: All command handlers implemented with EventPublisher trait
3. **Query Handlers**: All query handlers implemented with ReadModelStorage trait
4. **State Machines**: Moore and Mealy machines for aggregate state transitions
5. **Mock Infrastructure**: InMemoryRepository and MockEventPublisher for testing

### Missing:
1. **NATS JetStream Integration**: No actual NATS connection or JetStream configuration
2. **Event Store Implementation**: No persistent event storage
3. **CID Chain Implementation**: No content-addressed event chains
4. **Event Replay**: No mechanism to replay events from JetStream
5. **Snapshot Support**: No aggregate snapshot functionality

## Implementation Tasks

### Phase 1: NATS JetStream Setup (2 hours)

#### 1.1 Create NATS Client Module
```rust
// cim-domain/src/infrastructure/nats_client.rs
pub struct NatsClient {
    client: async_nats::Client,
    jetstream: async_nats::jetstream::Context,
}

impl NatsClient {
    pub async fn connect(config: NatsConfig) -> Result<Self, NatsError> {
        // Connect to NATS with retry logic
        // Create JetStream context
        // Verify connectivity
    }
}
```

#### 1.2 Configure JetStream Streams
```rust
// cim-domain/src/infrastructure/jetstream_config.rs
pub struct EventStreamConfig {
    pub name: String,
    pub subjects: Vec<String>,
    pub retention: RetentionPolicy,
    pub storage: StorageType,
    pub max_age: Duration,
}

pub async fn create_event_streams(js: &JetStreamContext) -> Result<(), Error> {
    // Create stream for each aggregate type
    // Configure retention and storage
    // Set up consumer groups
}
```

### Phase 2: Event Store Implementation (3 hours)

#### 2.1 Implement EventStore Trait
```rust
// cim-domain/src/infrastructure/event_store.rs
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append_events(
        &self,
        aggregate_id: &str,
        events: Vec<DomainEventEnum>,
        expected_version: Option<u64>,
    ) -> Result<(), EventStoreError>;

    async fn get_events(
        &self,
        aggregate_id: &str,
        from_version: Option<u64>,
    ) -> Result<Vec<StoredEvent>, EventStoreError>;

    async fn get_events_by_type(
        &self,
        event_type: &str,
        limit: usize,
    ) -> Result<Vec<StoredEvent>, EventStoreError>;
}
```

#### 2.2 JetStream Event Store Implementation
```rust
// cim-domain/src/infrastructure/jetstream_event_store.rs
pub struct JetStreamEventStore {
    client: Arc<NatsClient>,
    stream_name: String,
    cache: Arc<RwLock<LruCache<String, Vec<StoredEvent>>>>,
}

impl EventStore for JetStreamEventStore {
    // Implement append_events with:
    // - CID calculation for each event
    // - Previous CID chaining
    // - Optimistic concurrency control
    // - JetStream message publishing

    // Implement get_events with:
    // - JetStream consumer creation
    // - Event deserialization
    // - CID verification
    // - Cache management
}
```

### Phase 3: CID Chain Implementation (2 hours)

#### 3.1 Event CID Calculation
```rust
// cim-domain/src/infrastructure/cid_chain.rs
use cid::Cid;
use multihash::{Code, MultihashDigest};

pub struct EventWithCid {
    pub event: DomainEventEnum,
    pub cid: Cid,
    pub previous_cid: Option<Cid>,
    pub aggregate_version: u64,
}

pub fn calculate_event_cid(
    event: &DomainEventEnum,
    previous_cid: Option<&Cid>,
) -> Result<Cid, CidError> {
    // Serialize event to CBOR
    // Include previous CID in calculation
    // Use BLAKE3 hashing
    // Return CID v1
}
```

#### 3.2 CID Verification
```rust
pub fn verify_event_chain(
    events: &[EventWithCid],
) -> Result<(), ChainVerificationError> {
    // Verify each event's CID
    // Check previous_cid links
    // Ensure no gaps in chain
    // Return detailed error on failure
}
```

### Phase 4: Event Replay and Projections (3 hours)

#### 4.1 Event Replay Service
```rust
// cim-domain/src/infrastructure/event_replay.rs
pub struct EventReplayService {
    event_store: Arc<dyn EventStore>,
    handlers: HashMap<String, Box<dyn EventHandler>>,
}

impl EventReplayService {
    pub async fn replay_all_events(
        &self,
        from_timestamp: Option<DateTime<Utc>>,
    ) -> Result<ReplayStats, ReplayError> {
        // Create JetStream consumer from beginning
        // Process events in order
        // Update projections
        // Track progress
    }

    pub async fn replay_aggregate(
        &self,
        aggregate_id: &str,
    ) -> Result<Box<dyn AggregateRoot>, ReplayError> {
        // Get all events for aggregate
        // Create new aggregate instance
        // Apply events in order
        // Return rebuilt aggregate
    }
}
```

#### 4.2 Projection Updates
```rust
// cim-domain/src/infrastructure/projection_updater.rs
pub struct ProjectionUpdater {
    event_store: Arc<dyn EventStore>,
    read_model: Arc<dyn ReadModelStorage>,
    last_processed: Arc<RwLock<HashMap<String, u64>>>,
}

impl ProjectionUpdater {
    pub async fn start(&self) -> Result<(), ProjectionError> {
        // Subscribe to event stream
        // Process new events
        // Update read models
        // Track last processed position
    }
}
```

### Phase 5: Snapshot Support (2 hours)

#### 5.1 Snapshot Storage
```rust
// cim-domain/src/infrastructure/snapshot_store.rs
#[async_trait]
pub trait SnapshotStore: Send + Sync {
    async fn save_snapshot(
        &self,
        aggregate_id: &str,
        snapshot: AggregateSnapshot,
    ) -> Result<(), SnapshotError>;

    async fn get_latest_snapshot(
        &self,
        aggregate_id: &str,
    ) -> Result<Option<AggregateSnapshot>, SnapshotError>;
}

pub struct JetStreamSnapshotStore {
    object_store: Arc<NatsObjectStore>,
    metadata_stream: String,
}
```

#### 5.2 Snapshot Integration
```rust
// Update AggregateRepository to use snapshots
impl<A: AggregateRoot> AggregateRepository<A> {
    pub async fn load_with_snapshot(
        &self,
        aggregate_id: &str,
    ) -> Result<A, RepositoryError> {
        // Try to load latest snapshot
        // Get events after snapshot
        // Apply events to snapshot
        // Return aggregate
    }

    pub async fn save_with_snapshot(
        &self,
        aggregate: &A,
    ) -> Result<(), RepositoryError> {
        // Save events
        // Check if snapshot needed
        // Save snapshot if threshold met
    }
}
```

### Phase 6: Integration and Testing (2 hours)

#### 6.1 Update Command Handlers
```rust
// Update all command handlers to use JetStreamEventStore
impl PersonCommandHandler {
    pub fn new(
        event_store: Arc<dyn EventStore>,
        repository: Arc<dyn AggregateRepository<Person>>,
    ) -> Self {
        // Use real event store instead of mock
    }
}
```

#### 6.2 Integration Tests
```rust
// cim-domain/tests/event_store_integration.rs
#[tokio::test]
async fn test_event_store_append_and_retrieve() {
    // Start NATS server
    // Create JetStream event store
    // Append events
    // Retrieve and verify
}

#[tokio::test]
async fn test_cid_chain_integrity() {
    // Create events with CID chain
    // Verify chain integrity
    // Test tampering detection
}

#[tokio::test]
async fn test_event_replay() {
    // Store events
    // Clear projections
    // Replay events
    // Verify projections rebuilt
}
```

## Configuration

### NATS Connection Configuration
```toml
[nats]
url = "nats://localhost:4222"
user = "cim-domain"
password = "secure-password"
tls_required = true

[jetstream]
domain = "cim"
prefix = "CIM"

[event_store]
stream_name = "CIM-EVENTS"
retention_days = 365
max_events_per_aggregate = 10000
snapshot_threshold = 100
```

## Success Criteria

1. ✅ NATS JetStream connection established and configured
2. ✅ Events persisted to JetStream with CID chains
3. ✅ Event retrieval and replay working correctly
4. ✅ CID chain verification detects tampering
5. ✅ Projections update from event stream
6. ✅ Snapshots reduce replay time for large aggregates
7. ✅ All existing tests pass with real event store
8. ✅ Performance: <10ms event append, <50ms aggregate load

## Dependencies

Add to `cim-domain/Cargo.toml`:
```toml
[dependencies]
async-nats = "0.35"
cid = "0.11"
multihash = "0.19"
cbor4ii = "0.3"
lru = "0.12"
```

## Timeline

- Phase 1: NATS Setup (2 hours)
- Phase 2: Event Store Implementation (3 hours)
- Phase 3: CID Chain (2 hours)
- Phase 4: Replay & Projections (3 hours)
- Phase 5: Snapshots (2 hours)
- Phase 6: Integration & Testing (2 hours)

**Total: 14 hours**

## Next Steps

After completing this integration:
1. Update IA main application to use cim-domain with NATS
2. Implement distributed event routing between IA instances
3. Add monitoring and observability for event flows
4. Create performance benchmarks for large event streams
