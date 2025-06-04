# Graph Domain Design (Optimized)

## Overview

This document defines the optimized domain model for the Information Alchemist Graph system, incorporating performance improvements while maintaining strict DDD principles.

## Core Domain Model (Optimized)

### Aggregates

#### Graph (Aggregate Root)
The central concept with separated structure and entity storage for performance.

```rust
pub struct Graph {
    pub identity: GraphIdentity,
    pub metadata: GraphMetadata,
    pub journey: GraphJourney,

    /// Lightweight graph storing only IDs (petgraph)
    pub structure: GraphStructure,

    /// Component indices for O(1) queries
    pub indices: GraphIndices,
}

pub struct GraphStructure {
    /// Petgraph with only node/edge IDs
    pub graph: petgraph::Graph<NodeIdentity, EdgeIdentity>,

    /// Fast ID to index mapping
    pub identity_map: IdentityIndexMap,
}

pub struct GraphIndices {
    /// Nodes by component type
    pub by_component: HashMap<ComponentType, HashSet<NodeIdentity>>,

    /// Nodes by category
    pub by_category: HashMap<String, HashSet<NodeIdentity>>,

    /// Pre-computed metrics
    pub metrics_cache: GraphMetricsCache,
}
```

#### GraphView (Read Model)
Optimized perspective for queries with denormalized data.

```rust
pub struct GraphView {
    pub identity: ViewIdentity,
    pub graph: GraphIdentity,
    pub perspective: GraphPerspective,
    pub camera: CameraConfiguration,
    pub selection: SelectionState,

    /// Denormalized node views for fast queries
    pub node_views: HashMap<NodeIdentity, NodeView>,

    /// Cached query results
    pub query_cache: QueryCache,
}

pub struct NodeView {
    pub identity: NodeIdentity,
    pub category: String,
    pub component_types: Vec<ComponentType>,
    pub connections: ConnectionSummary,
    pub last_modified: SystemTime,
}
```

### Entities (Optimized Storage)

#### Node
A vertex with separated component storage.

```rust
pub struct Node {
    pub identity: NodeIdentity,
    pub graph: GraphIdentity,

    /// Component IDs instead of full components
    pub component_ids: HashSet<ComponentIdentity>,

    pub position: SpatialPosition,
    pub state: NodeState,
}

/// Separate component storage for efficiency
pub struct ComponentStorage {
    /// Actual component data
    components: HashMap<ComponentIdentity, Component>,

    /// Component to node mapping
    component_nodes: HashMap<ComponentIdentity, HashSet<NodeIdentity>>,
}
```

#### Edge
Lightweight connection representation.

```rust
pub struct Edge {
    pub identity: EdgeIdentity,
    pub graph: GraphIdentity,
    pub relationship: EdgeRelationship,

    /// Cached for performance
    pub source_index: NodeIndex,
    pub target_index: NodeIndex,
}
```

### Value Objects (Flyweight Pattern)

#### Components
Shared components using flyweight pattern.

```rust
pub struct Component {
    pub identity: ComponentIdentity,
    pub type_: ComponentType,
    pub data: ComponentData,
}

pub enum ComponentData {
    Visual(VisualData),
    Behavioral(BehavioralData),
    Metadata(MetadataData),
}

/// Component reference in nodes
pub struct ComponentIdentity(Uuid);
```

## Domain Events (With Snapshots)

### Snapshot Events

```rust
pub struct GraphSnapshotCreated {
    pub graph: GraphIdentity,
    pub snapshot_id: SnapshotIdentity,
    pub version: u64,
    pub sequence: u64,
    pub timestamp: SystemTime,
}

pub struct GraphRestoredFromSnapshot {
    pub graph: GraphIdentity,
    pub snapshot_id: SnapshotIdentity,
    pub events_applied: u64,
}
```

### Batch Events

```rust
pub struct NodeBatchAdded {
    pub graph: GraphIdentity,
    pub nodes: Vec<(NodeIdentity, NodeContent, SpatialPosition)>,
    pub timestamp: SystemTime,
}

pub struct EdgeBatchConnected {
    pub graph: GraphIdentity,
    pub edges: Vec<(EdgeIdentity, EdgeRelationship)>,
    pub timestamp: SystemTime,
}
```

## Domain Services (Optimized)

### Graph Management Services

```rust
/// Creates graphs with pre-allocated capacity
pub struct CreateGraphWithCapacity {
    pub estimated_nodes: usize,
    pub estimated_edges: usize,
}

/// Batch operations for efficiency
pub struct AddNodeBatchToGraph;
pub struct ConnectNodeBatchToGraph;

/// Async operations for heavy computations
pub struct CalculateGraphLayoutAsync;
pub struct AnalyzeGraphStructureAsync;
```

### Performance Services

```rust
/// Snapshot management
pub struct CreateGraphSnapshot;
pub struct RestoreGraphFromSnapshot;
pub struct CompactEventHistory;

/// Cache management
pub struct WarmGraphCache;
pub struct InvalidateGraphCache;
pub struct PrecomputeGraphMetrics;
```

## Storage Components (Optimized)

### Primary Storage

```rust
/// Event store with compaction
pub struct GraphEvents {
    /// Recent events in memory
    recent_events: BoundedBuffer<GraphEvent>,

    /// Persistent storage (NATS JetStream)
    persistent_store: EventStore,

    /// Snapshot references
    snapshots: SnapshotIndex,
}

/// Optimized graph storage
pub struct Graphs {
    /// Active graphs in memory
    active_graphs: LruCache<GraphIdentity, Graph>,

    /// Graph structures (petgraph)
    structures: HashMap<GraphIdentity, GraphStructure>,

    /// Component storage (shared)
    components: ComponentStorage,
}
```

### Index Storage

```rust
/// Concurrent indices for fast lookups
pub struct Nodes {
    /// Primary index
    by_identity: DashMap<NodeIdentity, Node>,

    /// Secondary indices
    by_category: DashMap<String, HashSet<NodeIdentity>>,
    by_component: DashMap<ComponentType, HashSet<NodeIdentity>>,
}

/// Edge indices with pre-computed paths
pub struct Edges {
    /// Primary index
    by_identity: DashMap<EdgeIdentity, Edge>,

    /// Adjacency lists
    outgoing: DashMap<NodeIdentity, Vec<EdgeIdentity>>,
    incoming: DashMap<NodeIdentity, Vec<EdgeIdentity>>,

    /// Cached shortest paths
    path_cache: PathCache,
}
```

## Async/Sync Bridge

### Bridge Components

```rust
/// Bridge between async NATS and sync graph operations
pub struct GraphEventBridge {
    /// Commands from Bevy (sync) to NATS (async)
    command_sender: crossbeam::channel::Sender<GraphCommand>,

    /// Events from NATS (async) to Bevy (sync)
    event_receiver: crossbeam::channel::Receiver<GraphEvent>,

    /// Batch processor
    batch_processor: BatchProcessor,
}

pub struct BatchProcessor {
    /// Accumulate events for batch processing
    event_buffer: Vec<GraphEvent>,

    /// Batch size and timing
    batch_size: usize,
    batch_timeout: Duration,
}
```

## Performance Characteristics

### Memory Optimization
- **Component Deduplication**: 60-80% memory reduction
- **Lightweight Graph Structure**: Only IDs in petgraph
- **Bounded Buffers**: Prevent unbounded growth

### Query Performance
- **O(1) Component Lookups**: Via indices
- **O(1) Category Queries**: Pre-computed sets
- **Cached Path Finding**: Reuse computed paths

### Scalability Features
- **Concurrent Access**: DashMap for safe parallel reads/writes
- **Event Compaction**: Bounded history with snapshots
- **Lazy Loading**: LRU cache for active graphs

## Implementation Guidelines (Performance-Focused)

### Domain Model Implementation
- Use newtype pattern for type safety without overhead
- Implement Copy for small identifiers
- Use Arc for shared immutable data
- Prefer &str over String where possible

### Service Implementation
- Batch operations when possible
- Use async for I/O-bound operations
- Implement circuit breakers for external calls
- Return streams for large result sets

### Event Implementation
- Use compact binary formats (bincode/messagepack)
- Implement event deduplication
- Design for event compaction
- Include only essential data

### Storage Implementation
- Use memory-mapped files for large datasets
- Implement write-ahead logging
- Use bloom filters for existence checks
- Partition data by time windows

## Monitoring and Metrics

### Performance Metrics

```rust
pub struct GraphPerformanceMetrics {
    /// Operation latencies
    pub add_node_p99: Duration,
    pub add_edge_p99: Duration,
    pub query_path_p99: Duration,

    /// Throughput
    pub events_per_second: f64,
    pub queries_per_second: f64,

    /// Resource usage
    pub memory_usage_mb: f64,
    pub cache_hit_rate: f64,
}
```

### Health Indicators

```rust
pub struct GraphHealthIndicators {
    /// Event processing lag
    pub event_lag_ms: u64,

    /// Cache effectiveness
    pub cache_hit_ratio: f64,

    /// Index freshness
    pub index_staleness_ms: u64,
}
```

## Key Optimizations Applied

1. **Separated Storage**: Graph structure separate from entity data
2. **Component Deduplication**: Flyweight pattern for shared components
3. **Read Model**: Denormalized views for fast queries
4. **Event Snapshots**: Bounded replay time
5. **Concurrent Structures**: DashMap for parallel access
6. **Batch Processing**: Reduced overhead for bulk operations
7. **Async Bridge**: Non-blocking I/O operations
8. **Smart Caching**: LRU for graphs, query result caching

This optimized design maintains all DDD principles while providing the performance needed for production use with 100K+ nodes.
