# Single Graph Model Architecture

## Overview

This document describes the new architecture where Bevy ECS serves as the single source of truth for all graph data, eliminating the need for synchronization between multiple models.

## Core Architecture

### Base Graph Model (ECS Components)

```rust
// ============= Core Components =============

#[derive(Component)]
pub struct Graph {
    pub metadata: GraphMetadata,
    pub journey: GraphJourney,
}

#[derive(Component)]
pub struct Node {
    pub content: NodeContent,
    // Position is now in Transform component
}

#[derive(Component)]
pub struct Edge {
    pub source: Entity,      // Direct entity reference
    pub target: Entity,      // Direct entity reference
    pub category: String,
    pub strength: f32,
    pub properties: HashMap<String, serde_json::Value>,
}

// ============= Index Components =============

#[derive(Component)]
pub struct GraphMember {
    pub graph: Entity,
}

#[derive(Component)]
pub struct NodeIndex {
    pub outgoing: Vec<Entity>,  // Edge entities
    pub incoming: Vec<Entity>,  // Edge entities
}

#[derive(Component)]
pub struct GraphIndex {
    pub nodes: HashSet<Entity>,
    pub edges: HashSet<Entity>,
}

// ============= Bundles =============

#[derive(Bundle)]
pub struct GraphBundle {
    pub graph: Graph,
    pub index: GraphIndex,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
}

#[derive(Bundle)]
pub struct NodeBundle {
    pub node: Node,
    pub member: GraphMember,
    pub index: NodeIndex,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
}

#[derive(Bundle)]
pub struct EdgeBundle {
    pub edge: Edge,
    pub member: GraphMember,
}
```

### Query-Based Views

Instead of maintaining separate data structures, we create views through queries:

```rust
// ============= View Queries =============

/// Get all nodes in a graph
pub fn query_graph_nodes(
    graph_entity: Entity,
    graph_index: Query<&GraphIndex>,
    nodes: Query<(&Node, &Transform)>,
) -> Vec<(Entity, &Node, Vec3)> {
    if let Ok(index) = graph_index.get(graph_entity) {
        index.nodes
            .iter()
            .filter_map(|&node_entity| {
                nodes.get(node_entity)
                    .ok()
                    .map(|(node, transform)| (node_entity, node, transform.translation))
            })
            .collect()
    } else {
        vec![]
    }
}

/// Find all edges connected to a node
pub fn query_node_edges(
    node_entity: Entity,
    node_index: Query<&NodeIndex>,
    edges: Query<&Edge>,
) -> NodeConnections {
    if let Ok(index) = node_index.get(node_entity) {
        NodeConnections {
            outgoing: index.outgoing
                .iter()
                .filter_map(|&e| edges.get(e).ok())
                .collect(),
            incoming: index.incoming
                .iter()
                .filter_map(|&e| edges.get(e).ok())
                .collect(),
        }
    } else {
        NodeConnections::default()
    }
}

/// Traverse graph from a starting node
pub fn traverse_graph(
    start: Entity,
    max_depth: usize,
    node_index: Query<&NodeIndex>,
    edges: Query<&Edge>,
) -> Vec<(Entity, usize)> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut result = Vec::new();

    queue.push_back((start, 0));
    visited.insert(start);

    while let Some((current, depth)) = queue.pop_front() {
        if depth >= max_depth {
            continue;
        }

        result.push((current, depth));

        if let Ok(index) = node_index.get(current) {
            for &edge_entity in &index.outgoing {
                if let Ok(edge) = edges.get(edge_entity) {
                    if visited.insert(edge.target) {
                        queue.push_back((edge.target, depth + 1));
                    }
                }
            }
        }
    }

    result
}
```

### System Examples

```rust
// ============= Graph Systems =============

/// System to maintain node indices when edges are added
pub fn update_node_indices(
    new_edges: Query<&Edge, Added<Edge>>,
    mut node_indices: Query<&mut NodeIndex>,
) {
    for edge in new_edges.iter() {
        // Update source node's outgoing edges
        if let Ok(mut source_index) = node_indices.get_mut(edge.source) {
            source_index.outgoing.push(edge_entity);
        }

        // Update target node's incoming edges
        if let Ok(mut target_index) = node_indices.get_mut(edge.target) {
            target_index.incoming.push(edge_entity);
        }
    }
}

/// System to render graph based on ECS data
pub fn render_graph(
    graphs: Query<(Entity, &GraphIndex)>,
    nodes: Query<(&Node, &Transform), With<GraphMember>>,
    edges: Query<&Edge, With<GraphMember>>,
    mut gizmos: Gizmos,
) {
    for (graph_entity, index) in graphs.iter() {
        // Draw nodes
        for &node_entity in &index.nodes {
            if let Ok((node, transform)) = nodes.get(node_entity) {
                gizmos.sphere(
                    transform.translation,
                    Quat::IDENTITY,
                    0.5,
                    Color::BLUE,
                );
            }
        }

        // Draw edges
        for &edge_entity in &index.edges {
            if let Ok(edge) = edges.get(edge_entity) {
                if let (Ok((_, source_transform)), Ok((_, target_transform))) =
                    (nodes.get(edge.source), nodes.get(edge.target)) {
                    gizmos.line(
                        source_transform.translation,
                        target_transform.translation,
                        Color::GREEN,
                    );
                }
            }
        }
    }
}

/// System for graph analysis
pub fn analyze_graph_structure(
    graphs: Query<(Entity, &Graph, &GraphIndex)>,
    nodes: Query<&Node>,
    edges: Query<&Edge>,
    mut analysis_events: EventWriter<GraphAnalyzed>,
) {
    for (graph_entity, graph, index) in graphs.iter() {
        let metrics = GraphMetrics {
            node_count: index.nodes.len(),
            edge_count: index.edges.len(),
            density: calculate_density(&index, &edges),
            clustering_coefficient: calculate_clustering(&index, &nodes, &edges),
        };

        analysis_events.send(GraphAnalyzed {
            graph: graph_entity,
            metrics,
        });
    }
}
```

### Benefits of Single Model

1. **No Synchronization**
   - Data exists in one place only
   - No sync bugs or performance overhead
   - Automatic consistency

2. **Query Performance**
   - Bevy's archetype system optimizes queries
   - Cache-friendly iteration
   - Parallel query execution

3. **Memory Efficiency**
   - Single allocation per component
   - No duplicate data
   - Efficient entity references

4. **Flexibility**
   - Easy to add new components
   - Views are just queries
   - Can optimize queries independently

### Migration Example

Before (Triple Model):
```rust
// Old: Three separate representations
let daggy_graph = storage.get_graph(graph_id)?;
let ecs_entities = world.query::<&Graph>().filter(|g| g.identity == graph_id);
let events = event_store.get_events(graph_id)?;

// Synchronization nightmare
sync_ecs_to_daggy(&world, &mut storage);
sync_daggy_to_events(&storage, &mut event_store);
```

After (Single Model):
```rust
// New: One source of truth
let graph_entity = /* Entity reference */;
let (graph, index) = world.query::<(&Graph, &GraphIndex)>().get(graph_entity)?;

// Direct queries, no sync needed
let nodes = query_graph_nodes(graph_entity, &world);
let metrics = analyze_graph_structure(graph_entity, &world);
```

### Event System Integration

Events become purely observational:
```rust
/// Events observe changes, don't store data
pub fn observe_graph_changes(
    changed_nodes: Query<(Entity, &Node), Changed<Node>>,
    mut events: EventWriter<NodeModified>,
) {
    for (entity, node) in changed_nodes.iter() {
        events.send(NodeModified {
            node: entity,
            timestamp: SystemTime::now(),
        });
    }
}
```

### Persistence

Direct serialization of ECS data:
```rust
pub fn save_graph(world: &World, graph: Entity) -> Result<GraphSnapshot> {
    let scene = DynamicSceneBuilder::from_world(world)
        .extract_entity(graph)
        .extract_entities(/* nodes and edges */)
        .build();

    Ok(GraphSnapshot {
        scene: scene.serialize()?,
        timestamp: SystemTime::now(),
    })
}
```

## Conclusion

This architecture leverages Bevy ECS as a graph database, eliminating the need for multiple synchronized models. All graph operations become ECS queries, providing better performance, simpler code, and automatic consistency.
