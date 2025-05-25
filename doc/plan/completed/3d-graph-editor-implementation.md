# 3D Graph Editor Implementation Plan

## ⚠️ Updated Status (December 2024)

This implementation plan has been partially superseded by the **ECS Refactoring** initiative. The core dual-layer architecture remains valid, but the implementation approach has evolved significantly.

### Key Updates:
- ✅ **Event-Sourced State**: Fully implemented in ECS Phase 4
- ✅ **Decoupled Systems**: Complete system decomposition in ECS Phase 5
- ✅ **Edge Architecture**: Redesigned as components (OutgoingEdge) instead of entities
- ✅ **Graph Algorithms**: Fully implemented including layouts
- 🚧 **Dual-Layer Design**: GraphData layer implemented, integration ongoing

For the most current architecture, see:
- [ECS Refactoring Plan](./ecs-refactoring-plan.md)
- [Graph Implementation Status](./graph-implementation-status.md)

---

## Executive Summary

This document outlines the implementation plan for a 3D-enabled graph editor built using Bevy v0.16.0, following CIM (Composable Information Machine) principles and a **dual-layer architecture** that separates graph data (Daggy/Petgraph) from visualization (Bevy ECS).

## Architecture Overview

### Core Principles
- **Dual-Layer Design**: Daggy manages graph topology, Bevy handles visualization
- **Event-Sourced State**: Graph modifications flow through events ✅ IMPLEMENTED
- **Decoupled Systems**: Graph algorithms run independently of rendering ✅ IMPLEMENTED
- **Composable Modules**: Each system is a reusable "Lego block" ✅ IMPLEMENTED

### Layer Separation
```
Layer 1: Computational Graph (Daggy)
  - Graph topology and relationships
  - Node/edge data storage
  - Graph algorithms (traversal, shortest path, etc.)
  - Serialization/deserialization

Layer 2: Visualization (Bevy ECS)
  - Spatial positioning
  - Visual properties
  - User interaction
  - Animation and rendering
```

## Phase 1: Foundation Components (Week 1-2) ✅ COMPLETED

### 1.1 Graph Data Layer ✅
```rust
use petgraph::graph::{DiGraph, NodeIndex, EdgeIndex};
use daggy::Dag;

#[derive(Resource)]
pub struct GraphData {
    /// The petgraph directed graph
    graph: DiGraph<NodeData, EdgeData>,
    /// Map from UUID to petgraph NodeIndex
    uuid_to_node: HashMap<Uuid, NodeIndex>,
    /// Map from NodeIndex to ECS Entity
    node_to_entity: HashMap<NodeIndex, Entity>,
    /// Map from EdgeIndex to ECS Entity
    edge_to_entity: HashMap<EdgeIndex, Entity>,
}

#[derive(Debug, Clone)]
pub struct NodeData {
    pub id: Uuid,
    pub name: String,
    pub domain_type: DomainNodeType,
    pub position: Vec3,
    pub labels: Vec<String>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct EdgeData {
    pub id: Uuid,
    pub edge_type: DomainEdgeType,
    pub labels: Vec<String>,
    pub properties: HashMap<String, String>,
}
```

### 1.2 Visualization Components ✅ UPDATED
```rust
// Reference to graph data
#[derive(Component)]
struct GraphNodeRef {
    dag_index: NodeIndex,
    version: u64
}

// NEW: Edges as components (not entities)
#[derive(Component)]
struct OutgoingEdge {
    id: Uuid,
    target: Entity,
    edge_type: DomainEdgeType,
    labels: Vec<String>,
    properties: HashMap<String, String>,
}

// Visual components only
#[derive(Component)]
struct NodeVisual {
    base_color: Color,
    current_color: Color,
}
```

### 1.3 Event Flow Architecture ✅ FULLY IMPLEMENTED
See `src/events/` for comprehensive event definitions covering:
- Graph events (creation, deletion, modification)
- UI events (notifications, status updates)
- I/O events (file operations)
- Camera events (movement, focus)

## Phase 2: Core Systems Implementation (Week 2-3) ✅ COMPLETED

### 2.1 Graph Data Management ✅
Implemented in `src/systems/graph/`:
- `creation.rs` - Node and edge creation with patterns
- `deletion.rs` - Safe deletion with cleanup
- `selection.rs` - Mouse and keyboard selection
- `movement.rs` - Dragging and alignment
- `validation.rs` - Property and connection validation
- `algorithms.rs` - Pathfinding and layouts

### 2.2 Graph Algorithm Systems ✅
All algorithms implemented in `src/systems/graph/algorithms.rs`:
- Shortest path (Dijkstra)
- Topological sort
- Cycle detection
- Force-directed layout
- Hierarchical layout
- Circular and grid layouts

### 2.3 Rendering Systems ✅
Basic structure implemented in `src/systems/rendering/`:
- Node rendering with mesh generation
- Edge rendering via OutgoingEdge components
- Material updates for selection
- LOD system structure

## Phase 3: Performance Optimization (Week 3-4) 🚧 IN PROGRESS

### 3.1 Change Detection ✅
Implemented in various systems:
- Component change detection using Bevy's Changed<T>
- Event-driven updates only when needed
- Selective visual updates

### 3.2 Batched Rendering 🚧
```rust
// Planned implementation for performance
fn batch_node_meshes(
    graph_data: Res<GraphData>,
    mut meshes: ResMut<Assets<Mesh>>,
    changed: Res<GraphChangeTracker>,
) -> Vec<InstanceData> {
    // Implementation pending
}
```

## Phase 4: Advanced Features (Week 4-5) ✅ PARTIALLY COMPLETE

### 4.1 Merkle DAG Support ✅
```rust
use daggy::Dag;

#[derive(Resource)]
pub struct MerkleDag {
    dag: Dag<MerkleNode, MerkleEdge>,
    cid_to_node: HashMap<Cid, NodeIndex>,
}
```

### 4.2 Graph Layout Algorithms ✅
All implemented and working:
- Force-directed layout with physics simulation
- Hierarchical layout for DAGs
- Circular layout for visualization
- Grid layout for organization

## Implementation Guidelines

### Event Flow Pattern ✅ IMPLEMENTED
```
1. User Input → CreateNodeEvent
2. Event Handler → Update Daggy/Petgraph
3. Sync System → Create/Update ECS Entity
4. Render System → Draw Visual Representation
```

### System Ordering ✅ IMPLEMENTED
See `src/systems/` for the complete modular system architecture with proper ordering.

### Testing Strategy 🚧
1. **Graph Data Tests**: Test Daggy operations independently
2. **Sync Tests**: Verify entity creation matches graph data
3. **Visual Tests**: Ensure rendering reflects graph state
4. **Performance Tests**: Benchmark with large graphs (10k+ nodes)

## Success Metrics
- Graph operations complete in < 16ms (60 FPS) ✅
- Support 250k+ elements as per CIM requirements 🚧
- Zero tight coupling between graph data and rendering ✅
- All graph algorithms available from petgraph ✅

## Migration Path ✅ IN PROGRESS
1. ✅ Keep existing HashMap implementation temporarily
2. ✅ Add GraphData resource alongside
3. 🚧 Gradually migrate systems to use GraphData
4. 🚧 Remove old implementation once complete

This plan ensures strict separation between graph computation and visualization, enabling maximum performance and flexibility while leveraging the full power of established graph libraries.
