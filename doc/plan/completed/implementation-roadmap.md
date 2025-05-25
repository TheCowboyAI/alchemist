# 3D Graph Editor Implementation Roadmap

## ⚠️ Updated Status (December 2024)

This roadmap has been partially superseded by the **ECS Refactoring** initiative, which has restructured the entire codebase. Many items below have been completed or replaced with better approaches.

### Major Changes:
- ✅ **Event System** (Phase 4) - Fully implemented with comprehensive event types
- ✅ **System Decomposition** (Phase 5) - Complete modular system architecture
- ✅ **Edge Architecture** - Redesigned as components instead of entities
- 🚧 **Dual-Layer Architecture** - Still in progress with GraphData integration

## Quick Start Guide

This roadmap provides actionable steps to implement the 3D graph editor following the **dual-layer architecture** specified in `graph-architecture.md`.

## Prerequisites Checklist

- [x] Bevy 0.16.0 configured in Cargo.toml
- [x] Petgraph and Daggy dependencies added
- [x] NixOS development environment with direnv
- [ ] NATS JetStream for event persistence
- [x] Egui integration for UI panels

## Week 1: Foundation - Dual-Layer Architecture ✅ PARTIALLY COMPLETE

### Day 1-2: Graph Data Layer ✅
```rust
// GraphData resource implemented
#[derive(Resource)]
pub struct GraphData {
    graph: DiGraph<NodeData, EdgeData>,
    uuid_to_node: HashMap<Uuid, NodeIndex>,
    node_to_entity: HashMap<NodeIndex, Entity>,
    edge_to_entity: HashMap<EdgeIndex, Entity>,
}
```

### Day 3-4: Migration Systems ✅
1. ✅ Update `handle_create_node_with_graph` to use GraphData
2. ✅ Update `handle_create_edge_with_graph` to use graph indices
3. ✅ Implement `process_deferred_edges` for edge creation
4. ✅ Keep old systems temporarily for compatibility

### Day 5: Event Flow Implementation ✅
The event flow has been fully implemented as part of ECS Phase 4:
- Comprehensive event types for all domains
- Event-driven architecture throughout
- Clear separation of concerns

## Week 2: Core Functionality with Graph Library 🚧 IN PROGRESS

### Day 1-2: Graph Algorithms ✅
Implemented in `src/systems/graph/algorithms.rs`:
- ✅ Cycle detection using petgraph
- ✅ Topological sort for workflow validation
- ✅ Shortest path queries
- ✅ Subgraph extraction
- ✅ Force-directed layout
- ✅ Hierarchical, circular, and grid layouts

### Day 3-4: Performance Optimization 🚧
- ✅ Change detection for graph updates
- 🚧 Spatial indexing for node queries
- 🚧 Batched update systems
- ✅ Profile with 1000+ nodes

### Day 5: Edge Rendering Fix ✅
Edge rendering completely redesigned with `OutgoingEdge` components

## Week 3: Advanced Graph Features 🚧 PARTIALLY COMPLETE

### Day 1-2: Merkle DAG Integration ✅
- ✅ MerkleDag resource using Daggy
- ✅ CID generation for nodes
- 🚧 Merkle proof validation
- ✅ Content-addressed storage

### Day 3-4: Layout Algorithms ✅
All implemented in `src/systems/graph/algorithms.rs`:
- ✅ Force-directed layout
- ✅ Hierarchical layout for DAGs
- ✅ Circular layout
- ✅ Grid layout

### Day 5: Serialization ✅
- ✅ JSON serialization implemented
- ✅ Arrows.app format support
- ✅ Import/export functionality

## Week 4: Integration and Polish 🚧 IN PROGRESS

### Day 1-2: NATS Event Stream ❌ NOT STARTED
- [ ] Connect graph events to NATS JetStream
- [ ] Implement event replay from stream
- [ ] Add collaborative editing support
- [ ] Create event sourcing for undo/redo

### Day 3-4: Performance at Scale 🚧
- 🚧 Test with 250k+ elements (CIM requirement)
- 🚧 Implement LOD system for large graphs
- ✅ Frustum culling
- 🚧 GPU instancing for nodes

### Day 5: Migration Completion 🚧
- 🚧 Remove old HashMap-based graph implementation
- ✅ Update all systems to use events
- ✅ Clean up PendingEdges workarounds
- 🚧 Performance benchmarking

## Updated Implementation Order

Based on the ECS refactoring, the recommended order is now:

1. **Complete Component Extraction** (ECS Phase 2)
   - Separate all components into dedicated modules
   - Remove component definitions from mixed files

2. **Resource Consolidation** (ECS Phase 3)
   - Minimize global state
   - Convert unnecessary resources to components

3. **Bundle Implementation** (ECS Phase 6)
   - Create standard node/edge bundles
   - Simplify entity spawning

4. **Plugin Architecture** (ECS Phase 7)
   - Organize systems with SystemSets
   - Define clear execution order

5. **Testing & Optimization** (ECS Phase 8)
   - Integration tests
   - Performance profiling
   - Documentation

## Testing Strategy ✅ UPDATED

### Graph Systems Tests
```rust
// Tests now focus on the new system architecture
#[test]
fn test_node_creation_system() {
    // Test event-driven node creation
}

#[test]
fn test_edge_as_component() {
    // Test OutgoingEdge component behavior
}
```

## Success Metrics by Week ✅ UPDATED

### Week 1 ✅
- [x] GraphData resource implemented
- [x] Event system fully operational
- [x] System decomposition complete
- [x] Basic graph algorithms accessible

### Week 2 🚧
- [x] Edge rendering fixed with new architecture
- [x] 60 FPS with 1000 nodes
- [x] Graph algorithms integrated
- [x] Change detection working

### Week 3 🚧
- [x] Merkle DAG support
- [x] Layout algorithms implemented
- [x] Serialization working
- 🚧 10k+ nodes performing well

### Week 4 ❌
- [ ] NATS integration complete
- 🚧 250k+ elements supported
- 🚧 Old implementation removed
- 🚧 Full test coverage

## Migration Checklist ✅ UPDATED

- [x] Add GraphData resource
- [x] Implement comprehensive event system
- [x] Create modular system architecture
- [x] Update all node creation to use events
- [x] Redesign edges as components
- [x] Remove PendingEdges
- [x] Update save/load functions
- 🚧 Remove old graph.rs HashMap implementation

## References

- [ECS Refactoring Plan](./ecs-refactoring-plan.md) - Current architectural approach
- [Graph Implementation Status](./graph-implementation-status.md) - Detailed implementation status
- [Graph Architecture](./graph-architecture.md) - Original dual-layer design

This roadmap is being actively updated as the ECS refactoring progresses. See the ECS Refactoring Plan for the most current architectural decisions.
