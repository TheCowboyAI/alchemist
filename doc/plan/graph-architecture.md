# Graph Architecture Plan for Alchemist

## Executive Summary

The Alchemist graph editor requires a robust, scalable graph data structure that supports both computational graph operations (algorithms, traversals, validation) and real-time visualization. After extensive research and failed attempts with ad-hoc solutions, we're adopting a **dual-layer architecture** that separates graph logic from rendering concerns.

## Current Problems (December 2024)

1. **Ad-hoc graph implementation using HashMaps**
   - No proper graph algorithms (shortest path, topological sort, cycle detection)
   - Edges store Entity references, creating tight coupling with ECS lifecycle
   - Complex edge rendering due to lack of proper traversal APIs

2. **Performance issues**
   - Constant re-rendering every frame (logs show 5 nodes, 4 edges rendered repeatedly)
   - No change detection for graph updates
   - Inefficient node/edge lookups

3. **Architectural debt**
   - `PendingEdges` system is a band-aid for proper edge management
   - GraphEdge components store Entity references instead of node IDs
   - No separation between data model and visualization

## Proposed Architecture

### Layer 1: Computational Graph (Daggy/Petgraph)

**Primary Library**: Daggy (built on petgraph)
- Provides true DAG semantics with cycle detection
- Efficient graph algorithms out of the box
- Stable node/edge indices
- Serialization support via serde

**Data Structures**:
```rust
#[derive(Resource)]
pub struct GraphData {
    dag: Dag<NodeData, EdgeData>,
    uuid_to_node: HashMap<Uuid, NodeIndex>,
    node_to_entity: HashMap<NodeIndex, Entity>,
    edge_to_entity: HashMap<EdgeIndex, Entity>,
}
```

**Responsibilities**:
- Graph topology management
- Node/edge relationships
- Graph algorithms (traversal, shortest path, etc.)
- Serialization/deserialization
- Merkle DAG operations for CIM

### Layer 2: Visualization Layer (Bevy ECS)

**Components**:
```rust
#[derive(Component)]
struct GraphNodeRef {
    dag_index: NodeIndex,
    version: u64
}

#[derive(Component)]
struct GraphEdgeRef {
    dag_index: EdgeIndex,
    source_entity: Entity,
    target_entity: Entity
}
```

**Responsibilities**:
- Spatial positioning (Transform components)
- Visual properties (materials, meshes)
- User interaction (selection, hover)
- Animation and transitions
- Batched rendering

## Implementation Phases

### Phase 1: Foundation (Current Sprint)
- [x] Add petgraph dependency
- [x] Create GraphData resource with Daggy
- [ ] Migrate node creation to use dual-layer approach
- [ ] Migrate edge creation to reference graph indices
- [ ] Fix edge rendering using proper graph traversal

### Phase 2: Performance Optimization
- [ ] Implement change detection for graph updates
- [ ] Add spatial indexing for large graphs
- [ ] Batch mesh generation for nodes/edges
- [ ] Implement LOD system for 10k+ node graphs

### Phase 3: Advanced Features
- [ ] Merkle DAG support for CIM workflows
- [ ] Graph layout algorithms (force-directed, hierarchical)
- [ ] Subgraph extraction and visualization
- [ ] Real-time collaboration via NATS events

## Key Design Decisions

### Why Daggy over raw Petgraph?
1. **Cleaner API** for DAG-specific operations
2. **Built-in cycle detection** during edge insertion
3. **Stable indices** across node removals (via StableDag)
4. **Lighter weight** than full petgraph when we only need DAGs

### Why maintain separate graph and ECS entities?
1. **Decoupling**: Graph logic independent of rendering
2. **Performance**: Can run graph algorithms without touching ECS
3. **Flexibility**: Can have multiple visual representations of same graph
4. **Persistence**: Graph data survives rendering changes

### Event Flow Architecture
```
User Input → Graph Event → Daggy Update → Sync System → ECS Update → Render
```

## Integration with CIM Requirements

Per `cim-graphs.md`, we need:
1. **Merkle DAG support**: Daggy nodes will store CIDs and proofs
2. **Event sourcing**: Graph modifications create immutable event stream
3. **3D visualization**: Bevy handles complex 3D layouts
4. **250k+ elements**: Dual-layer architecture enables this scale

## Migration Strategy

### Immediate Actions
1. Keep existing components for compatibility
2. Add GraphData resource alongside current system
3. Gradually migrate systems to use GraphData
4. Remove old HashMap-based graph once migration complete

### Code Example - New Node Creation
```rust
fn create_node(
    mut commands: Commands,
    mut graph_data: ResMut<GraphData>,
    event: CreateNodeEvent,
) {
    // 1. Add to Daggy
    let node_idx = graph_data.add_node(NodeData {
        id: event.id,
        position: event.position,
        // ... other fields
    });

    // 2. Create ECS entity
    let entity = commands.spawn(GraphNodeBundle {
        // Visual components only
    }).id();

    // 3. Link them
    graph_data.set_node_entity(node_idx, entity);
}
```

## Success Metrics

1. **Performance**: 60 FPS with 10k nodes, 50k edges
2. **Correctness**: No edge rendering glitches
3. **Features**: Full graph algorithm support
4. **Maintainability**: Clear separation of concerns

## References

- [bevy-graphs.md](../bevy-graphs.md) - Detailed dual-layer implementation
- [cim-graphs.md](../cim-graphs.md) - Business requirements
- [Cursor Rules: graphs](../../.cursorrules) - Implementation patterns
- [Bevy #6719](https://github.com/bevyengine/bevy/discussions/6719) - Community discussion on graph needs

## Note on Bevy 0.16 Graph Support

While Bevy 0.16 does **not** include a built-in graph library (this is still in discussion phase), we're following the architectural patterns recommended by the Bevy community for integrating external graph libraries with the ECS.
