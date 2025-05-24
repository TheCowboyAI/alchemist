# Graph Implementation Status

## Overview

Following the frustration with our ad-hoc graph implementation (December 2024), we've begun transitioning to a proper dual-layer architecture using established graph libraries.

## Current Status

### ✅ Completed
1. **Dependencies Added**
   - `petgraph = "0.6"` - General graph algorithms
   - `daggy = "0.8"` - DAG-specific operations for Merkle DAGs

2. **Core Infrastructure**
   - `GraphData` resource using petgraph's DiGraph
   - `MerkleDag` resource using Daggy for CIM requirements
   - Bidirectional mapping between graph indices and ECS entities

3. **Documentation**
   - Comprehensive architecture plan in `doc/plan/graph-architecture.md`
   - Implementation patterns in cursor rules
   - Business requirements documented in `cim-graphs.md`

### 🚧 In Progress
1. **Migration from Current System**
   - Current: GraphEdge stores Entity references (tight coupling)
   - Target: GraphEdge stores node indices, entities are separate
   - Status: Example systems created, not yet integrated

2. **Edge Rendering Fix**
   - Current: Complex rotation calculations failing
   - Target: Simple cylinder alignment using Quat::from_rotation_arc
   - Status: Code written but needs testing with new architecture

### ❌ Not Started
1. **Performance Optimizations**
   - Change detection for graph updates
   - Batched mesh generation
   - LOD system for large graphs
   - Spatial indexing

2. **Advanced Features**
   - Graph layout algorithms
   - Subgraph extraction
   - NATS event integration
   - Full Merkle proof validation

## Key Issues Resolved

1. **No Graph Algorithms** → Now have full petgraph algorithm suite
2. **Entity Coupling** → Separated graph data from ECS entities
3. **Constant Re-rendering** → Architecture supports change detection
4. **No DAG Semantics** → Daggy provides cycle detection and DAG operations

## Next Steps

### Immediate (This Week)
1. Replace `handle_create_node_events` with `handle_create_node_with_graph`
2. Update edge creation to use graph indices
3. Fix edge rendering using simplified rotation
4. Remove `PendingEdges` hack

### Short Term (Next Sprint)
1. Implement change detection
2. Add graph validation using Daggy
3. Create UI for graph inspection
4. Performance benchmarking

### Long Term (Q1 2025)
1. Full Merkle DAG implementation
2. 250k+ element support
3. Real-time collaboration
4. Advanced visualization modes

## Code Examples

### Creating a Node (New Way)
```rust
// 1. Add to graph
let node_idx = graph_data.add_node(NodeData { ... });

// 2. Create visual entity
let entity = commands.spawn(VisualNodeBundle { ... }).id();

// 3. Link them
graph_data.set_node_entity(node_idx, entity);
```

### Creating an Edge (New Way)
```rust
// 1. Add to graph (by UUID, not Entity!)
let edge_idx = graph_data.add_edge(source_uuid, target_uuid, EdgeData { ... })?;

// 2. Get connected entities for rendering
let (source_entity, target_entity) = graph_data.get_edge_entities(edge_idx)?;

// 3. Create visual edge
commands.spawn(VisualEdgeBundle { source_entity, target_entity, ... });
```

## Performance Targets

| Metric | Current | Target | Status |
|--------|---------|---------|---------|
| Max Nodes | ~100 | 250,000 | 🚧 Architecture ready |
| Max Edges | ~200 | 500,000 | 🚧 Architecture ready |
| FPS @ 10k nodes | 15 | 60 | ❌ Needs optimization |
| Graph Algorithms | None | O(V+E) | ✅ Petgraph provides |

## Resources

- [Petgraph Docs](https://docs.rs/petgraph)
- [Daggy Docs](https://docs.rs/daggy)
- [Bevy ECS Best Practices](../bevy-graphs.md)
- [CIM Requirements](../cim-graphs.md)
