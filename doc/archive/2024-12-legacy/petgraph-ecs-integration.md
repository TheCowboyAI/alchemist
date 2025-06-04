# Petgraph + ECS Integration Design

## Overview

Use petgraph as the core graph data structure while maintaining ECS entities that reference petgraph indices. This gives us:
- Optimized graph algorithms from petgraph
- ECS for rendering and interaction
- Single source of truth (petgraph)
- No synchronization overhead

## Architecture

### Core Components

```rust
use petgraph::graph::{Graph, NodeIndex, EdgeIndex};
use petgraph::Directed;

// The single graph resource
#[derive(Resource)]
pub struct GraphModel {
    graph: Graph<NodeData, EdgeData, Directed>,
}

// Domain data stored in petgraph
#[derive(Debug, Clone)]
pub struct NodeData {
    pub label: String,
    pub category: String,
    pub position: Vec3,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct EdgeData {
    pub category: String,
    pub weight: f32,
    pub properties: HashMap<String, String>,
}

// ECS components that reference petgraph
#[derive(Component)]
pub struct GraphNode {
    pub index: NodeIndex,
}

#[derive(Component)]
pub struct GraphEdge {
    pub index: EdgeIndex,
    pub source_entity: Entity,
    pub target_entity: Entity,
}

// For fast lookups
#[derive(Resource, Default)]
pub struct EntityMapping {
    node_to_entity: HashMap<NodeIndex, Entity>,
    edge_to_entity: HashMap<EdgeIndex, Entity>,
}
```

### Bundles

```rust
#[derive(Bundle)]
pub struct NodeBundle {
    pub graph_node: GraphNode,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
}

#[derive(Bundle)]
pub struct EdgeBundle {
    pub graph_edge: GraphEdge,
}
```

## Graph Operations

### Adding Nodes

```rust
pub fn add_node(
    commands: &mut Commands,
    graph: &mut GraphModel,
    mapping: &mut EntityMapping,
    label: String,
    category: String,
    position: Vec3,
) -> (NodeIndex, Entity) {
    // Add to petgraph
    let node_data = NodeData {
        label,
        category,
        position,
        properties: HashMap::new(),
    };
    let node_idx = graph.graph.add_node(node_data);

    // Create ECS entity
    let entity = commands.spawn(NodeBundle {
        graph_node: GraphNode { index: node_idx },
        transform: Transform::from_translation(position),
        global_transform: GlobalTransform::default(),
        visibility: Visibility::default(),
    }).id();

    // Update mapping
    mapping.node_to_entity.insert(node_idx, entity);

    (node_idx, entity)
}
```

### Adding Edges

```rust
pub fn add_edge(
    commands: &mut Commands,
    graph: &mut GraphModel,
    mapping: &mut EntityMapping,
    source_idx: NodeIndex,
    target_idx: NodeIndex,
    category: String,
    weight: f32,
) -> Result<(EdgeIndex, Entity), GraphError> {
    // Get entities for source and target
    let source_entity = mapping.node_to_entity.get(&source_idx)
        .ok_or(GraphError::NodeNotFound)?;
    let target_entity = mapping.node_to_entity.get(&target_idx)
        .ok_or(GraphError::NodeNotFound)?;

    // Add to petgraph
    let edge_data = EdgeData {
        category,
        weight,
        properties: HashMap::new(),
    };
    let edge_idx = graph.graph.add_edge(source_idx, target_idx, edge_data);

    // Create ECS entity
    let entity = commands.spawn(EdgeBundle {
        graph_edge: GraphEdge {
            index: edge_idx,
            source_entity: *source_entity,
            target_entity: *target_entity,
        },
    }).id();

    // Update mapping
    mapping.edge_to_entity.insert(edge_idx, entity);

    Ok((edge_idx, entity))
}
```

## Systems

### Graph Mutation System

```rust
#[derive(Event)]
pub enum GraphMutation {
    AddNode { label: String, category: String, position: Vec3 },
    AddEdge { source: NodeIndex, target: NodeIndex, category: String, weight: f32 },
    RemoveNode { index: NodeIndex },
    RemoveEdge { index: EdgeIndex },
    UpdateNodePosition { index: NodeIndex, position: Vec3 },
}

pub fn process_graph_mutations(
    mut commands: Commands,
    mut graph: ResMut<GraphModel>,
    mut mapping: ResMut<EntityMapping>,
    mut events: EventReader<GraphMutation>,
) {
    for event in events.read() {
        match event {
            GraphMutation::AddNode { label, category, position } => {
                add_node(&mut commands, &mut graph, &mut mapping,
                    label.clone(), category.clone(), *position);
            }
            GraphMutation::AddEdge { source, target, category, weight } => {
                if let Err(e) = add_edge(&mut commands, &mut graph, &mut mapping,
                    *source, *target, category.clone(), *weight) {
                    warn!("Failed to add edge: {:?}", e);
                }
            }
            GraphMutation::RemoveNode { index } => {
                if let Some(entity) = mapping.node_to_entity.remove(index) {
                    commands.entity(entity).despawn_recursive();
                    graph.graph.remove_node(*index);
                }
            }
            // ... other mutations
        }
    }
}
```

### Sync Transform System

```rust
/// Keep ECS transforms in sync with petgraph positions
pub fn sync_node_transforms(
    graph: Res<GraphModel>,
    mut query: Query<(&GraphNode, &mut Transform)>,
) {
    for (graph_node, mut transform) in query.iter_mut() {
        if let Some(node_data) = graph.graph.node_weight(graph_node.index) {
            transform.translation = node_data.position;
        }
    }
}
```

### Render System

```rust
pub fn render_graph(
    graph: Res<GraphModel>,
    nodes: Query<(&GraphNode, &Transform)>,
    edges: Query<&GraphEdge>,
    mut gizmos: Gizmos,
) {
    // Draw nodes
    for (graph_node, transform) in nodes.iter() {
        if let Some(node_data) = graph.graph.node_weight(graph_node.index) {
            let color = match node_data.category.as_str() {
                "start" => Color::GREEN,
                "end" => Color::RED,
                _ => Color::BLUE,
            };
            gizmos.sphere(transform.translation, Quat::IDENTITY, 0.5, color);
        }
    }

    // Draw edges
    for graph_edge in edges.iter() {
        if let (Ok(source_tf), Ok(target_tf)) = (
            nodes.get(graph_edge.source_entity),
            nodes.get(graph_edge.target_entity),
        ) {
            gizmos.line(
                source_tf.1.translation,
                target_tf.1.translation,
                Color::GRAY,
            );
        }
    }
}
```

## Algorithm Usage

### Direct Petgraph Algorithms

```rust
use petgraph::algo::{dijkstra, all_simple_paths, tarjan_scc};

pub fn find_shortest_path(
    graph: &GraphModel,
    start: NodeIndex,
    end: NodeIndex,
) -> Option<Vec<NodeIndex>> {
    let predecessors = dijkstra(
        &graph.graph,
        start,
        Some(end),
        |e| e.weight().weight as i32,
    );

    // Reconstruct path
    let mut path = vec![end];
    let mut current = end;

    while current != start {
        if let Some(&pred) = predecessors.get(&current) {
            path.push(pred);
            current = pred;
        } else {
            return None;
        }
    }

    path.reverse();
    Some(path)
}

pub fn find_all_paths(
    graph: &GraphModel,
    start: NodeIndex,
    end: NodeIndex,
    max_length: Option<usize>,
) -> Vec<Vec<NodeIndex>> {
    all_simple_paths(&graph.graph, start, end, 0, max_length)
        .collect()
}

pub fn find_strongly_connected_components(
    graph: &GraphModel,
) -> Vec<Vec<NodeIndex>> {
    tarjan_scc(&graph.graph)
}
```

### Query Integration

```rust
/// Find entities for a path
pub fn path_to_entities(
    path: &[NodeIndex],
    mapping: &EntityMapping,
) -> Vec<Entity> {
    path.iter()
        .filter_map(|idx| mapping.node_to_entity.get(idx).copied())
        .collect()
}

/// Highlight a path in the visualization
pub fn highlight_path_system(
    selected_path: Res<SelectedPath>,
    mapping: Res<EntityMapping>,
    mut query: Query<&mut Visibility, With<GraphNode>>,
) {
    // Dim all nodes
    for mut visibility in query.iter_mut() {
        *visibility = Visibility::Hidden;
    }

    // Highlight path nodes
    for node_idx in &selected_path.0 {
        if let Some(&entity) = mapping.node_to_entity.get(node_idx) {
            if let Ok(mut visibility) = query.get_mut(entity) {
                *visibility = Visibility::Visible;
            }
        }
    }
}
```

## Benefits

1. **Best of Both Worlds**:
   - Petgraph's optimized algorithms
   - ECS for rendering and interaction
   - No duplicate data structures

2. **Performance**:
   - Graph algorithms run on compact petgraph structure
   - ECS queries only for visualization/interaction
   - No synchronization overhead

3. **Simplicity**:
   - Clear separation of concerns
   - Single source of truth (petgraph)
   - Easy to reason about

4. **Flexibility**:
   - Can use any petgraph algorithm
   - Can extend with custom ECS components
   - Can optimize rendering independently

## Example Usage

```rust
fn setup_graph_example(
    mut commands: Commands,
    mut graph: ResMut<GraphModel>,
    mut mapping: ResMut<EntityMapping>,
) {
    // Create nodes
    let (a_idx, _) = add_node(&mut commands, &mut graph, &mut mapping,
        "A".to_string(), "start".to_string(), Vec3::new(-5.0, 0.0, 0.0));

    let (b_idx, _) = add_node(&mut commands, &mut graph, &mut mapping,
        "B".to_string(), "normal".to_string(), Vec3::new(0.0, 3.0, 0.0));

    let (c_idx, _) = add_node(&mut commands, &mut graph, &mut mapping,
        "C".to_string(), "normal".to_string(), Vec3::new(0.0, -3.0, 0.0));

    let (d_idx, _) = add_node(&mut commands, &mut graph, &mut mapping,
        "D".to_string(), "end".to_string(), Vec3::new(5.0, 0.0, 0.0));

    // Create edges
    add_edge(&mut commands, &mut graph, &mut mapping,
        a_idx, b_idx, "path".to_string(), 1.0).unwrap();

    add_edge(&mut commands, &mut graph, &mut mapping,
        a_idx, c_idx, "path".to_string(), 2.0).unwrap();

    add_edge(&mut commands, &mut graph, &mut mapping,
        b_idx, d_idx, "path".to_string(), 1.0).unwrap();

    add_edge(&mut commands, &mut graph, &mut mapping,
        c_idx, d_idx, "path".to_string(), 1.0).unwrap();

    // Find shortest path
    if let Some(path) = find_shortest_path(&graph, a_idx, d_idx) {
        info!("Shortest path: {:?}", path);
    }
}
```

## Conclusion

This design uses petgraph as the authoritative graph data structure while maintaining lightweight ECS entities that reference petgraph indices. This eliminates synchronization issues while providing access to petgraph's extensive algorithm library and maintaining Bevy's rendering capabilities.
