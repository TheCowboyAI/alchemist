# 3D Graph Editor Implementation Plan

## Executive Summary

This document outlines the implementation plan for a 3D-enabled graph editor built using Bevy v0.16.0, following CIM (Composable Information Machine) principles and a **dual-layer architecture** that separates graph data (Daggy/Petgraph) from visualization (Bevy ECS).

## Architecture Overview

### Core Principles
- **Dual-Layer Design**: Daggy manages graph topology, Bevy handles visualization
- **Event-Sourced State**: Graph modifications flow through events
- **Decoupled Systems**: Graph algorithms run independently of rendering
- **Composable Modules**: Each system is a reusable "Lego block"

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

## Phase 1: Foundation Components (Week 1-2)

### 1.1 Graph Data Layer
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

### 1.2 Visualization Components
```rust
// Reference to graph data
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

// Visual components only
#[derive(Component)]
struct NodeVisual {
    base_color: Color,
    current_color: Color,
}

#[derive(Component)]
struct EdgeVisual {
    width: f32,
    color: Color,
}
```

### 1.3 Event Flow Architecture
```rust
#[derive(Event)]
enum GraphEvent {
    // User input events
    NodeCreated { id: Uuid, position: Vec3, domain_type: DomainNodeType },
    NodeMoved { id: Uuid, from: Vec3, to: Vec3 },
    EdgeCreated { id: Uuid, source: Uuid, target: Uuid },

    // Graph data events (after Daggy update)
    GraphNodeAdded { index: NodeIndex, data: NodeData },
    GraphEdgeAdded { index: EdgeIndex, data: EdgeData },
    GraphTopologyChanged,
}
```

## Phase 2: Core Systems Implementation (Week 2-3)

### 2.1 Graph Data Management
```rust
/// Handles graph events and updates Daggy
fn process_graph_events(
    mut events: EventReader<GraphEvent>,
    mut graph_data: ResMut<GraphData>,
    mut sync_events: EventWriter<GraphSyncEvent>,
) {
    for event in events.read() {
        match event {
            GraphEvent::NodeCreated { id, position, domain_type } => {
                // Add to Daggy first
                let node_data = NodeData { id, position, domain_type, ... };
                let index = graph_data.graph.add_node(node_data);
                graph_data.uuid_to_node.insert(id, index);

                // Trigger sync to ECS
                sync_events.send(GraphSyncEvent::CreateNodeEntity { index });
            }
            // ... other events
        }
    }
}

/// Syncs graph changes to ECS entities
fn sync_graph_to_ecs(
    mut commands: Commands,
    mut sync_events: EventReader<GraphSyncEvent>,
    mut graph_data: ResMut<GraphData>,
) {
    for event in sync_events.read() {
        match event {
            GraphSyncEvent::CreateNodeEntity { index } => {
                let node = graph_data.graph.node_weight(index).unwrap();
                let entity = commands.spawn(NodeVisualBundle {
                    // Only visual components
                    transform: Transform::from_translation(node.position),
                    visual: NodeVisual { ... },
                }).id();

                graph_data.node_to_entity.insert(index, entity);
            }
            // ... other sync events
        }
    }
}
```

### 2.2 Graph Algorithm Systems
```rust
/// Runs graph algorithms on Daggy structure
fn graph_algorithm_system(
    graph_data: Res<GraphData>,
    algorithm_requests: EventReader<AlgorithmRequest>,
    mut results: EventWriter<AlgorithmResult>,
) {
    for request in algorithm_requests.read() {
        match request {
            AlgorithmRequest::ShortestPath { source, target } => {
                // Use petgraph algorithms directly
                let path = petgraph::algo::astar(&graph_data.graph, source, target, ...);
                results.send(AlgorithmResult::Path(path));
            }
            AlgorithmRequest::TopologicalSort => {
                let sorted = petgraph::algo::toposort(&graph_data.graph, None);
                results.send(AlgorithmResult::Ordering(sorted));
            }
        }
    }
}
```

### 2.3 Rendering Systems
```rust
/// Updates visual entities based on graph state
fn update_node_visuals(
    graph_data: Res<GraphData>,
    mut nodes: Query<(&GraphNodeRef, &mut Transform, &mut NodeVisual)>,
) {
    for (node_ref, mut transform, mut visual) in nodes.iter_mut() {
        if let Some(node_data) = graph_data.graph.node_weight(node_ref.dag_index) {
            transform.translation = node_data.position;
            // Update other visual properties
        }
    }
}
```

## Phase 3: Performance Optimization (Week 3-4)

### 3.1 Change Detection
```rust
#[derive(Resource)]
struct GraphChangeTracker {
    modified_nodes: HashSet<NodeIndex>,
    modified_edges: HashSet<EdgeIndex>,
    last_update: u64,
}

/// Only update visuals for changed elements
fn selective_visual_update(
    changes: Res<GraphChangeTracker>,
    graph_data: Res<GraphData>,
    mut visuals: Query<(&GraphNodeRef, &mut Transform)>,
) {
    for (node_ref, mut transform) in visuals.iter_mut() {
        if changes.modified_nodes.contains(&node_ref.dag_index) {
            // Update only changed nodes
        }
    }
}
```

### 3.2 Batched Rendering
```rust
/// Batch mesh generation for performance
fn batch_node_meshes(
    graph_data: Res<GraphData>,
    mut meshes: ResMut<Assets<Mesh>>,
    changed: Res<GraphChangeTracker>,
) -> Vec<InstanceData> {
    graph_data.graph.node_indices()
        .filter(|idx| changed.modified_nodes.contains(idx))
        .map(|idx| {
            let node = graph_data.graph.node_weight(idx).unwrap();
            InstanceData {
                position: node.position,
                color: get_color_for_type(&node.domain_type),
            }
        })
        .collect()
}
```

## Phase 4: Advanced Features (Week 4-5)

### 4.1 Merkle DAG Support
```rust
use daggy::Dag;

#[derive(Resource)]
pub struct MerkleDag {
    dag: Dag<MerkleNode, MerkleEdge>,
    cid_to_node: HashMap<Cid, NodeIndex>,
}

#[derive(Clone)]
struct MerkleNode {
    cid: Cid,
    links: Vec<Cid>,
    data: NodeData,
}
```

### 4.2 Graph Layout Algorithms
```rust
/// Apply force-directed layout using graph structure
fn force_directed_layout(
    mut graph_data: ResMut<GraphData>,
    time: Res<Time>,
) {
    // Run layout algorithm on Daggy structure
    let positions = calculate_force_layout(&graph_data.graph, time.delta_seconds());

    // Update node positions in graph data
    for (idx, pos) in positions {
        if let Some(node) = graph_data.graph.node_weight_mut(idx) {
            node.position = pos;
        }
    }
}
```

## Implementation Guidelines

### Event Flow Pattern
```
1. User Input → CreateNodeEvent
2. Event Handler → Update Daggy/Petgraph
3. Sync System → Create/Update ECS Entity
4. Render System → Draw Visual Representation
```

### System Ordering
```rust
app.add_systems(Update, (
    // Input handling
    handle_mouse_input,
    handle_keyboard_input,
    // Graph data updates
    process_graph_events,
    validate_graph_constraints,
    // Sync to visualization
    sync_graph_to_ecs,
    update_graph_bounds,
    // Visual updates
    update_node_visuals,
    update_edge_visuals,
    // Algorithms (can run in parallel)
    graph_algorithm_system,
).chain());
```

### Testing Strategy
1. **Graph Data Tests**: Test Daggy operations independently
2. **Sync Tests**: Verify entity creation matches graph data
3. **Visual Tests**: Ensure rendering reflects graph state
4. **Performance Tests**: Benchmark with large graphs (10k+ nodes)

## Success Metrics
- Graph operations complete in < 16ms (60 FPS)
- Support 250k+ elements as per CIM requirements
- Zero tight coupling between graph data and rendering
- All graph algorithms available from petgraph

## Migration Path
1. Keep existing HashMap implementation temporarily
2. Add GraphData resource alongside
3. Gradually migrate systems to use GraphData
4. Remove old implementation once complete

This plan ensures strict separation between graph computation and visualization, enabling maximum performance and flexibility while leveraging the full power of established graph libraries.
