# Refactor to Single Graph Model Plan

## Problem Statement
Currently synchronizing 3 separate graph representations:
1. **ECS Entities** - Bevy components (Graph, Node, Edge)
2. **Daggy Storage** - GraphStorage for persistence
3. **Event Store** - Domain events in Merkle DAG

This causes:
- Performance overhead from constant synchronization
- Memory waste from triple storage
- Complexity from keeping models in sync
- Potential consistency issues

## Solution: Single Base Model with Query Views

### Core Principle
Use Bevy ECS as the single source of truth for graph data. Everything else becomes a view or query on this base model.

## Implementation Plan

### Phase 1: Refactor Core Domain Model
**Goal**: Optimize graph representation for ECS

1. **Update Edge Component** to use Entity references:
```rust
#[derive(Component)]
pub struct Edge {
    pub identity: EdgeIdentity,
    pub graph: Entity,  // Instead of GraphIdentity
    pub source: Entity, // Instead of NodeIdentity
    pub target: Entity, // Instead of NodeIdentity
    pub category: String,
    pub strength: f32,
    pub properties: HashMap<String, serde_json::Value>,
}
```

2. **Add Graph Indexing Components**:
```rust
#[derive(Component)]
pub struct GraphMember {
    pub graph: Entity,
}

#[derive(Component)]
pub struct NodeIndex {
    pub edges_out: Vec<Entity>,
    pub edges_in: Vec<Entity>,
}
```

3. **Create Relationship Components**:
```rust
#[derive(Component)]
pub struct GraphContains {
    pub nodes: HashSet<Entity>,
    pub edges: HashSet<Entity>,
}
```

### Phase 2: Remove Redundant Storage
**Goal**: Eliminate GraphStorage and synchronization

1. **Delete** `src/contexts/graph_management/storage.rs`
2. **Remove** all `SyncGraphWithStorage` systems
3. **Update** GraphManagementPlugin to remove storage initialization
4. **Refactor** import/export to work directly with ECS

### Phase 3: Create Query-Based Views
**Goal**: Efficient views without data duplication

1. **GraphView Query**:
```rust
pub fn query_graph_view(
    graph_query: Query<(&Graph, &GraphContains)>,
    node_query: Query<(&Node, &Transform), With<GraphMember>>,
    edge_query: Query<&Edge, With<GraphMember>>,
) -> GraphView {
    // Build view by querying ECS
}
```

2. **Traversal Queries**:
```rust
pub fn find_connected_nodes(
    start: Entity,
    node_index: Query<&NodeIndex>,
    edges: Query<&Edge>,
) -> Vec<Entity> {
    // Efficient graph traversal using ECS queries
}
```

3. **Analysis Queries**:
```rust
pub fn calculate_graph_metrics(
    graph: Entity,
    contains: Query<&GraphContains>,
    nodes: Query<&Node>,
) -> GraphMetrics {
    // Compute metrics on demand
}
```

### Phase 4: Optimize Event System
**Goal**: Events for audit only, not data storage

1. **Simplify Event Adapter** - Remove payload duplication
2. **Event Queries** - Read historical data from event store when needed
3. **Lazy Event Materialization** - Only reconstruct past states on demand

### Phase 5: Persistence Strategy
**Goal**: Efficient save/load without intermediate formats

1. **Direct ECS Serialization**:
```rust
pub fn save_graph(world: &World, graph: Entity) -> Result<Vec<u8>, Error> {
    // Serialize ECS components directly
}
```

2. **Scene-based Loading**:
```rust
pub fn load_graph(world: &mut World, data: &[u8]) -> Result<Entity, Error> {
    // Deserialize directly into ECS
}
```

### Phase 6: Performance Optimizations
**Goal**: Leverage Bevy's ECS optimizations

1. **Use Bevy Relations** (when available) for graph edges
2. **Spatial Indexing** with Bevy's transform hierarchy
3. **Change Detection** for incremental updates
4. **Parallel Queries** for analysis operations

## Benefits

1. **Performance**:
   - Single memory allocation per graph element
   - Cache-friendly ECS iteration
   - No synchronization overhead
   - Parallel query execution

2. **Simplicity**:
   - One source of truth
   - No synchronization logic
   - Cleaner codebase
   - Easier debugging

3. **Flexibility**:
   - Easy to add new views/queries
   - Can optimize queries independently
   - Natural integration with Bevy systems

4. **Correctness**:
   - No synchronization bugs
   - Automatic consistency
   - Change detection built-in

## Migration Strategy

1. **Week 1**: Refactor domain model (Phase 1)
2. **Week 2**: Remove redundant storage (Phase 2)
3. **Week 3**: Implement query views (Phase 3)
4. **Week 4**: Optimize events and persistence (Phases 4-5)
5. **Week 5**: Performance optimization and testing (Phase 6)

## Success Criteria

- [ ] Single graph representation in ECS
- [ ] No synchronization systems
- [ ] All views use queries
- [ ] Performance improvement of 50%+
- [ ] Reduced memory usage by 66%
- [ ] Simplified codebase
- [ ] All tests passing

## Next Steps

1. Create feature branch: `refactor/single-graph-model`
2. Start with Phase 1 domain model changes
3. Update tests to use new model
4. Incrementally migrate each phase
