# EventStore-CQRS-Graph Quick Reference

## Architecture Overview

```
NATS JetStream (Event Store) → CQRS → Bevy ECS (Visualization)
     ↓                          ↓            ↓
  CID Chain              Write/Read     Real-time
  Persistence            Separation     Updates
```

## Key Concepts

### 1. Event Store with CID Chains

**Purpose**: Immutable event history with content-addressed integrity

```rust
// Every event has a CID calculated from:
// - Event payload
// - Previous CID (creates chain)
// - Aggregate ID
// - Event type
// - Timestamp

let event_cid = calculate_event_cid(
    &payload_bytes,
    previous_cid,    // Links to previous event
    &aggregate_id,
    &event_type,
    timestamp,
)?;
```

### 2. EventStream Transactions

**Purpose**: Atomic batches of related events

```rust
pub struct EventStreamTransaction {
    pub transaction_id: TransactionId,
    pub sequence_range: SequenceRange,  // Start/end sequences
    pub aggregate_id: AggregateId,
    pub events: Vec<DomainEvent>,       // Batch of events
    pub metadata: TransactionMetadata,
}

// Fetch transaction
let transaction = event_service.fetch_transaction(
    aggregate_id,
    TransactionOptions {
        replay_policy: ReplayPolicy::FromBeginning,
        max_events: Some(1000),
    },
).await?;
```

### 3. CQRS Pattern

**Write Model**: Optimized for commands and consistency
```rust
pub struct GraphAggregate {
    id: GraphId,
    graph: petgraph::Graph<NodeId, EdgeId>,  // Lightweight
    nodes: DashMap<NodeId, NodeEntity>,       // Concurrent
    component_indices: ComponentIndices,       // Fast lookups
}
```

**Read Model**: Optimized for queries
```rust
pub struct GraphReadModel {
    node_views: DashMap<NodeId, NodeView>,    // Denormalized
    metrics: Arc<RwLock<GraphMetrics>>,       // Pre-computed
    query_cache: QueryCache,                  // Cached results
}
```

### 4. Async/Sync Bridge

**Purpose**: Connect async NATS with sync Bevy

```rust
// Sync → Async (Commands)
bridge.send_command(BridgeCommand::Graph(
    GraphCommand::AddNode { graph_id, node }
))?;

// Async → Sync (Events)
let events = bridge.receive_events();  // Batched
for event in events {
    // Process in Bevy
}
```

### 5. Component Deduplication

**Purpose**: 60-80% memory reduction

```rust
// Before: Each node stores full components
Node { components: Vec<Component> }  // 500+ bytes

// After: Nodes reference shared components
Node { component_ids: HashSet<ComponentId> }  // 64 bytes
ComponentStorage { components: HashMap<ComponentId, Component> }
```

## Common Patterns

### Creating a Graph

```rust
// 1. Send command through bridge
let command = GraphCommand::CreateGraph {
    id: GraphId::new(),
    metadata: GraphMetadata {
        name: "Workflow Graph".to_string(),
        domain: "business".to_string(),
        ..default()
    },
};
bridge.send_command(BridgeCommand::Graph(command))?;

// 2. Event flows through NATS
// 3. Read model updates
// 4. Bevy receives event and updates visualization
```

### Querying Nodes

```rust
// Fast query using read model
let nodes = read_model.find_nodes_with_component(
    ComponentType::Visual,
    ComponentCriteria::HasProperty("color", "blue"),
)?;

// Result from cache or pre-computed index
```

### Real-time Updates

```rust
// Subscribe to graph events
subscription_manager.subscribe(
    format!("event.store.{}.>", graph_id),
    None,
    SubscriptionHandler::GraphUpdate {
        target_graph: graph_id
    },
).await?;

// Events automatically flow to Bevy
```

### Time Travel

```rust
// Replay graph at specific time
let historical_graph = replay_service.replay_at_time(
    graph_id,
    SystemTime::now() - Duration::from_hours(24),
).await?;

// Animate evolution
let animation = replay_service.create_replay_animation(
    graph_id,
    start_time,
    end_time,
    Duration::from_millis(100),  // Step duration
).await?;
```

## Performance Characteristics

| Operation | Performance | Scale |
|-----------|------------|-------|
| Node lookup | O(1) | 1M+ nodes |
| Component query | O(1) | Via indices |
| Event append | < 1ms | 10K/sec |
| Snapshot load | < 100ms | Any size |
| Query cache | < 1μs | Hit rate 90%+ |

## Error Handling

```rust
// Event store errors
match event_store.append_event(aggregate_id, event_type, payload).await {
    Ok(event) => {
        // Event has CID and sequence
    }
    Err(EventStoreError::ChainBroken { expected, actual }) => {
        // CID chain integrity violation
    }
    Err(e) => {
        // Other errors
    }
}

// Bridge errors
if let Err(BridgeError::CommandChannelClosed) = bridge.send_command(cmd) {
    // Reconnect or fail gracefully
}
```

## Testing Patterns

```rust
// Test CID chain integrity
#[tokio::test]
async fn test_event_chain() {
    let event1 = store.append_event(...).await?;
    let event2 = store.append_event(...).await?;

    assert_eq!(event2.previous_cid, event1.event_cid);
}

// Test CQRS consistency
#[test]
fn test_read_write_consistency() {
    // Write through command
    write_model.process_command(AddNode { ... })?;

    // Read should reflect change
    let node = read_model.get_node(node_id)?;
    assert_eq!(node.version, 1);
}
```

## Configuration

### NATS JetStream
```yaml
jetstream:
  store_dir: "./data/jetstream"
  max_memory_store: 1GB
  max_file_store: 10GB
```

### Bridge Settings
```rust
BatchConfig {
    max_batch_size: 100,        // Events per batch
    batch_timeout: 10ms,        // Max wait time
    max_pending_events: 10_000, // Buffer size
}
```

### Performance Tuning
```rust
// Component deduplication threshold
DEDUP_THRESHOLD: 0.8  // 80% similarity

// Cache settings
CACHE_SIZE: 10_000    // Entries
CACHE_TTL: 300s       // 5 minutes

// Snapshot frequency
SNAPSHOT_INTERVAL: 1000  // Events
```

## Debugging

### Check Event Chain
```bash
# View event stream
nats stream view event-store

# Check specific aggregate
nats stream get event-store --filter "event.store.graph-123.>"

# Verify CID chain
nats stream get event-store --json | jq '.headers.Cid'
```

### Monitor Performance
```rust
// Built-in metrics
let metrics = graph_service.get_metrics();
println!("Event lag: {}ms", metrics.event_lag_ms);
println!("Cache hit rate: {}%", metrics.cache_hit_ratio * 100.0);
println!("Query p99: {}ms", metrics.query_p99_ms);
```

## Common Issues

1. **CID Mismatch**: Previous CID doesn't match expected
   - Solution: Check for concurrent modifications

2. **Memory Growth**: Unbounded event history
   - Solution: Enable snapshots and compaction

3. **Slow Queries**: Cache misses
   - Solution: Warm cache, add indices

4. **Event Lag**: Processing falling behind
   - Solution: Increase batch size, add workers

This quick reference covers the essential concepts and patterns for working with the EventStore-CQRS-Graph engine.
