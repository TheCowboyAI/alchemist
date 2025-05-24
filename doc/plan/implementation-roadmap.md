# 3D Graph Editor Implementation Roadmap

## Quick Start Guide

This roadmap provides actionable steps to implement the 3D graph editor following the **dual-layer architecture** specified in `graph-architecture.md`.

## Prerequisites Checklist

- [x] Bevy 0.16.0 configured in Cargo.toml
- [x] Petgraph and Daggy dependencies added
- [ ] NixOS development environment with direnv
- [ ] NATS JetStream for event persistence
- [x] Egui integration for UI panels

## Week 1: Foundation - Dual-Layer Architecture

### Day 1-2: Graph Data Layer
```rust
// Create the GraphData resource
#[derive(Resource)]
pub struct GraphData {
    graph: DiGraph<NodeData, EdgeData>,
    uuid_to_node: HashMap<Uuid, NodeIndex>,
    node_to_entity: HashMap<NodeIndex, Entity>,
    edge_to_entity: HashMap<EdgeIndex, Entity>,
}

// Initialize in plugin
app.init_resource::<GraphData>()
```

### Day 3-4: Migration Systems
1. Update `handle_create_node_with_graph` to use GraphData
2. Update `handle_create_edge_with_graph` to use graph indices
3. Implement `process_deferred_edges` for edge creation
4. Keep old systems temporarily for compatibility

### Day 5: Event Flow Implementation
```rust
// Implement the event flow:
// User Input → Graph Event → Update GraphData → Sync to ECS → Render

fn create_node_flow(
    events: EventReader<CreateNodeEvent>,
    mut graph_data: ResMut<GraphData>,
    mut commands: Commands,
) {
    // 1. Add to petgraph
    let node_idx = graph_data.add_node(node_data);

    // 2. Create visual entity
    let entity = commands.spawn(VisualNodeBundle { ... }).id();

    // 3. Link them
    graph_data.set_node_entity(node_idx, entity);
}
```

## Week 2: Core Functionality with Graph Library

### Day 1-2: Graph Algorithms
- [ ] Implement cycle detection using Daggy
- [ ] Add topological sort for workflow validation
- [ ] Create shortest path queries using petgraph
- [ ] Add subgraph extraction

### Day 3-4: Performance Optimization
- [ ] Implement change detection for graph updates
- [ ] Add spatial indexing for node queries
- [ ] Create batched update systems
- [ ] Profile with 1000+ nodes

### Day 5: Edge Rendering Fix
```rust
// Fix edge rendering using graph traversal
fn render_edges_with_graph(
    graph_data: Res<GraphData>,
    mut edge_query: Query<(&GraphEdgeRef, &mut Transform)>,
) {
    for (edge_ref, mut transform) in edge_query.iter_mut() {
        if let Some((source_entity, target_entity)) =
            graph_data.get_edge_entities(edge_ref.dag_index) {
            // Update edge position/rotation
        }
    }
}
```

## Week 3: Advanced Graph Features

### Day 1-2: Merkle DAG Integration
- [ ] Add MerkleDag resource using Daggy
- [ ] Implement CID generation for nodes
- [ ] Create Merkle proof validation
- [ ] Add content-addressed storage

### Day 3-4: Layout Algorithms
- [ ] Force-directed layout using graph structure
- [ ] Hierarchical layout for DAGs
- [ ] Circular layout for cycles
- [ ] Custom CIM workflow layouts

### Day 5: Serialization
```rust
// Implement graph serialization
fn save_graph(graph_data: &GraphData) -> Result<String, Error> {
    let nodes: Vec<_> = graph_data.graph.node_weights().collect();
    let edges: Vec<_> = graph_data.graph.edge_weights().collect();
    serde_json::to_string(&(nodes, edges))
}
```

## Week 4: Integration and Polish

### Day 1-2: NATS Event Stream
- [ ] Connect graph events to NATS JetStream
- [ ] Implement event replay from stream
- [ ] Add collaborative editing support
- [ ] Create event sourcing for undo/redo

### Day 3-4: Performance at Scale
- [ ] Test with 250k+ elements (CIM requirement)
- [ ] Implement LOD system for large graphs
- [ ] Add frustum culling
- [ ] GPU instancing for nodes

### Day 5: Migration Completion
- [ ] Remove old HashMap-based graph implementation
- [ ] Update all systems to use GraphData
- [ ] Clean up PendingEdges workarounds
- [ ] Performance benchmarking

## Critical Implementation Order

1. **GraphData Resource First**
   ```rust
   // This is the foundation - get it right
   app.init_resource::<GraphData>()
      .add_systems(Update, (
          handle_create_node_with_graph,
          handle_create_edge_with_graph,
          process_deferred_edges,
      ).chain())
   ```

2. **Fix Edge Creation**
   - Use DeferredEdgeEvent for patterns/loading
   - Resolve entities through GraphData
   - Remove PendingEdges hack

3. **Leverage Graph Algorithms**
   - Use petgraph for traversals
   - Implement layout algorithms
   - Add validation using DAG properties

4. **Optimize Rendering**
   - Only update changed elements
   - Batch similar operations
   - Use spatial indexing

## Testing Strategy

### Graph Data Layer Tests
```rust
#[test]
fn test_graph_data_operations() {
    let mut graph_data = GraphData::default();

    // Test node addition
    let node_idx = graph_data.add_node(node_data);
    assert!(graph_data.get_node(uuid).is_some());

    // Test edge creation
    let edge_idx = graph_data.add_edge(source_uuid, target_uuid, edge_data);
    assert!(edge_idx.is_ok());
}
```

### Integration Tests
```rust
#[test]
fn test_node_entity_sync() {
    // Verify GraphData and ECS stay synchronized
    // Test entity creation matches graph nodes
    // Verify edge entities reference correct nodes
}
```

## Common Pitfalls to Avoid

1. **Mixing Layers**
   - Don't store Entity references in GraphData
   - Keep visual properties out of graph data
   - Use events to communicate between layers

2. **Ignoring Graph Library Features**
   - Use petgraph algorithms, don't reimplement
   - Leverage Daggy for DAG validation
   - Let the library handle graph complexity

3. **Tight Coupling**
   - GraphData should work without ECS
   - Rendering should work with any graph
   - Keep systems focused and composable

## Success Metrics by Week

### Week 1
- [x] GraphData resource implemented
- [x] Node/edge creation using dual-layer
- [x] DeferredEdgeEvent system working
- [ ] Basic graph algorithms accessible

### Week 2
- [ ] Edge rendering fixed
- [ ] 60 FPS with 1000 nodes
- [ ] Graph algorithms integrated
- [ ] Change detection working

### Week 3
- [ ] Merkle DAG support
- [ ] Layout algorithms implemented
- [ ] Serialization working
- [ ] 10k+ nodes performing well

### Week 4
- [ ] NATS integration complete
- [ ] 250k+ elements supported
- [ ] Old implementation removed
- [ ] Full test coverage

## Migration Checklist

- [x] Add GraphData resource
- [x] Update plugin to use new handlers
- [x] Implement DeferredEdgeEvent
- [ ] Update all node creation sites
- [ ] Update all edge creation sites
- [ ] Remove PendingEdges
- [ ] Update save/load functions
- [ ] Remove old graph.rs HashMap implementation

This roadmap follows the architecture specified in `graph-architecture.md` and provides a clear path to a scalable, maintainable graph editor.
