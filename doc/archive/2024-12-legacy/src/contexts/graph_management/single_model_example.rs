//! Example implementation of single graph model using ECS as the graph database

use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

// ============= Core Components (Single Model) =============

#[derive(Component, Debug, Clone)]
pub struct Graph {
    pub name: String,
    pub description: String,
}

#[derive(Component, Debug, Clone)]
pub struct Node {
    pub label: String,
    pub category: String,
    pub properties: HashMap<String, String>,
}

#[derive(Component, Debug, Clone)]
pub struct Edge {
    pub source: Entity,
    pub target: Entity,
    pub category: String,
    pub weight: f32,
}

// ============= Index Components (For Fast Queries) =============

#[derive(Component, Debug, Default)]
pub struct GraphIndex {
    pub nodes: HashSet<Entity>,
    pub edges: HashSet<Entity>,
}

#[derive(Component, Debug, Default)]
pub struct NodeIndex {
    pub outgoing: Vec<Entity>, // Edge entities
    pub incoming: Vec<Entity>, // Edge entities
}

#[derive(Component, Debug)]
pub struct GraphMember {
    pub graph: Entity,
}

// ============= Bundles =============

#[derive(Bundle)]
pub struct GraphBundle {
    pub graph: Graph,
    pub index: GraphIndex,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

#[derive(Bundle)]
pub struct NodeBundle {
    pub node: Node,
    pub member: GraphMember,
    pub index: NodeIndex,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

#[derive(Bundle)]
pub struct EdgeBundle {
    pub edge: Edge,
    pub member: GraphMember,
}

// ============= Graph Operations (No External Dependencies) =============

/// Create a new graph
pub fn create_graph(commands: &mut Commands, name: String, description: String) -> Entity {
    commands
        .spawn(GraphBundle {
            graph: Graph { name, description },
            index: GraphIndex::default(),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
        })
        .id()
}

/// Add a node to a graph
pub fn add_node(
    commands: &mut Commands,
    graph_entity: Entity,
    label: String,
    category: String,
    position: Vec3,
    graph_index: &mut GraphIndex,
) -> Entity {
    let node_entity = commands
        .spawn(NodeBundle {
            node: Node {
                label,
                category,
                properties: HashMap::new(),
            },
            member: GraphMember {
                graph: graph_entity,
            },
            index: NodeIndex::default(),
            transform: Transform::from_translation(position),
            global_transform: GlobalTransform::default(),
        })
        .id();

    // Update graph index
    graph_index.nodes.insert(node_entity);

    node_entity
}

/// Connect two nodes with an edge
pub fn connect_nodes(
    commands: &mut Commands,
    graph_entity: Entity,
    source: Entity,
    target: Entity,
    category: String,
    weight: f32,
    graph_index: &mut GraphIndex,
    source_index: &mut NodeIndex,
    target_index: &mut NodeIndex,
) -> Entity {
    let edge_entity = commands
        .spawn(EdgeBundle {
            edge: Edge {
                source,
                target,
                category,
                weight,
            },
            member: GraphMember {
                graph: graph_entity,
            },
        })
        .id();

    // Update indices
    graph_index.edges.insert(edge_entity);
    source_index.outgoing.push(edge_entity);
    target_index.incoming.push(edge_entity);

    edge_entity
}

// ============= Query-Based Graph Algorithms =============

/// Find all neighbors of a node (pure ECS, no external graph library)
pub fn find_neighbors(
    node: Entity,
    node_indices: &Query<&NodeIndex>,
    edges: &Query<&Edge>,
) -> Vec<Entity> {
    if let Ok(index) = node_indices.get(node) {
        index
            .outgoing
            .iter()
            .filter_map(|&edge_entity| edges.get(edge_entity).ok().map(|edge| edge.target))
            .collect()
    } else {
        vec![]
    }
}

/// Breadth-first search (pure ECS implementation)
pub fn bfs_traversal(
    start: Entity,
    node_indices: &Query<&NodeIndex>,
    edges: &Query<&Edge>,
) -> Vec<(Entity, usize)> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut result = Vec::new();

    queue.push_back((start, 0));
    visited.insert(start);

    while let Some((current, depth)) = queue.pop_front() {
        result.push((current, depth));

        for neighbor in find_neighbors(current, node_indices, edges) {
            if visited.insert(neighbor) {
                queue.push_back((neighbor, depth + 1));
            }
        }
    }

    result
}

/// Find shortest path using Dijkstra's algorithm (pure ECS)
pub fn shortest_path(
    start: Entity,
    end: Entity,
    node_indices: &Query<&NodeIndex>,
    edges: &Query<&Edge>,
) -> Option<Vec<Entity>> {
    use std::cmp::Ordering;
    use std::collections::BinaryHeap;

    #[derive(Eq, PartialEq)]
    struct State {
        cost: u32,
        node: Entity,
    }

    impl Ord for State {
        fn cmp(&self, other: &Self) -> Ordering {
            other.cost.cmp(&self.cost)
        }
    }

    impl PartialOrd for State {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    let mut distances = HashMap::new();
    let mut previous = HashMap::new();
    let mut heap = BinaryHeap::new();

    distances.insert(start, 0);
    heap.push(State {
        cost: 0,
        node: start,
    });

    while let Some(State { cost, node }) = heap.pop() {
        if node == end {
            // Reconstruct path
            let mut path = vec![end];
            let mut current = end;

            while let Some(&prev) = previous.get(&current) {
                path.push(prev);
                current = prev;
            }

            path.reverse();
            return Some(path);
        }

        if cost > *distances.get(&node).unwrap_or(&u32::MAX) {
            continue;
        }

        if let Ok(index) = node_indices.get(node) {
            for &edge_entity in &index.outgoing {
                if let Ok(edge) = edges.get(edge_entity) {
                    let next_cost = cost + (edge.weight * 1000.0) as u32;

                    if next_cost < *distances.get(&edge.target).unwrap_or(&u32::MAX) {
                        distances.insert(edge.target, next_cost);
                        previous.insert(edge.target, node);
                        heap.push(State {
                            cost: next_cost,
                            node: edge.target,
                        });
                    }
                }
            }
        }
    }

    None
}

// ============= Systems =============

/// System to maintain indices when edges are added
pub fn update_edge_indices(
    new_edges: Query<(Entity, &Edge), Added<Edge>>,
    mut node_indices: Query<&mut NodeIndex>,
) {
    for (edge_entity, edge) in new_edges.iter() {
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

/// System to render the graph using gizmos
pub fn render_graph_system(
    graphs: Query<(Entity, &GraphIndex)>,
    nodes: Query<(&Node, &Transform)>,
    edges: Query<&Edge>,
    mut gizmos: Gizmos,
) {
    for (_graph_entity, index) in graphs.iter() {
        // Draw nodes
        for &node_entity in &index.nodes {
            if let Ok((node, transform)) = nodes.get(node_entity) {
                // Color based on category
                let color = match node.category.as_str() {
                    "start" => Color::srgb(0.0, 1.0, 0.0),
                    "end" => Color::srgb(1.0, 0.0, 0.0),
                    _ => Color::srgb(0.0, 0.0, 1.0),
                };

                gizmos.sphere(transform.translation, Quat::IDENTITY, 0.5, color);
            }
        }

        // Draw edges
        for &edge_entity in &index.edges {
            if let Ok(edge) = edges.get(edge_entity) {
                if let (Ok((_, source_tf)), Ok((_, target_tf))) =
                    (nodes.get(edge.source), nodes.get(edge.target))
                {
                    gizmos.line(
                        source_tf.translation,
                        target_tf.translation,
                        Color::srgb(0.5, 0.5, 0.5),
                    );
                }
            }
        }
    }
}

// ============= Example Usage =============

pub fn setup_example_graph(mut commands: Commands) {
    // Create a graph
    let graph = create_graph(
        &mut commands,
        "Example Graph".to_string(),
        "Demo".to_string(),
    );

    // We'll need to get the graph index to update it
    // In a real system, this would be done through queries
    let mut graph_index = GraphIndex::default();

    // Create nodes
    let node_a = add_node(
        &mut commands,
        graph,
        "A".to_string(),
        "start".to_string(),
        Vec3::new(-5.0, 0.0, 0.0),
        &mut graph_index,
    );

    let node_b = add_node(
        &mut commands,
        graph,
        "B".to_string(),
        "normal".to_string(),
        Vec3::new(0.0, 3.0, 0.0),
        &mut graph_index,
    );

    let node_c = add_node(
        &mut commands,
        graph,
        "C".to_string(),
        "normal".to_string(),
        Vec3::new(0.0, -3.0, 0.0),
        &mut graph_index,
    );

    let node_d = add_node(
        &mut commands,
        graph,
        "D".to_string(),
        "end".to_string(),
        Vec3::new(5.0, 0.0, 0.0),
        &mut graph_index,
    );

    // In practice, we'd get these from queries
    let mut index_a = NodeIndex::default();
    let mut index_b = NodeIndex::default();
    let mut index_c = NodeIndex::default();
    let mut index_d = NodeIndex::default();

    // Connect nodes
    connect_nodes(
        &mut commands,
        graph,
        node_a,
        node_b,
        "path".to_string(),
        1.0,
        &mut graph_index,
        &mut index_a,
        &mut index_b,
    );

    connect_nodes(
        &mut commands,
        graph,
        node_a,
        node_c,
        "path".to_string(),
        2.0,
        &mut graph_index,
        &mut index_a,
        &mut index_c,
    );

    connect_nodes(
        &mut commands,
        graph,
        node_b,
        node_d,
        "path".to_string(),
        1.0,
        &mut graph_index,
        &mut index_b,
        &mut index_d,
    );

    connect_nodes(
        &mut commands,
        graph,
        node_c,
        node_d,
        "path".to_string(),
        1.0,
        &mut graph_index,
        &mut index_c,
        &mut index_d,
    );

    // Update the graph entity with the final index
    commands.entity(graph).insert(graph_index);

    // Update node entities with their indices
    commands.entity(node_a).insert(index_a);
    commands.entity(node_b).insert(index_b);
    commands.entity(node_c).insert(index_c);
    commands.entity(node_d).insert(index_d);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_as_ecs() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Create graph
        let graph = app
            .world_mut()
            .spawn(GraphBundle {
                graph: Graph {
                    name: "Test".to_string(),
                    description: "Test graph".to_string(),
                },
                index: GraphIndex::default(),
                transform: Transform::default(),
                global_transform: GlobalTransform::default(),
            })
            .id();

        // Create nodes
        let node_a = app
            .world_mut()
            .spawn(NodeBundle {
                node: Node {
                    label: "A".to_string(),
                    category: "test".to_string(),
                    properties: HashMap::new(),
                },
                member: GraphMember { graph },
                index: NodeIndex::default(),
                transform: Transform::default(),
                global_transform: GlobalTransform::default(),
            })
            .id();

        let node_b = app
            .world_mut()
            .spawn(NodeBundle {
                node: Node {
                    label: "B".to_string(),
                    category: "test".to_string(),
                    properties: HashMap::new(),
                },
                member: GraphMember { graph },
                index: NodeIndex::default(),
                transform: Transform::default(),
                global_transform: GlobalTransform::default(),
            })
            .id();

        // Create edge
        let edge = app
            .world_mut()
            .spawn(EdgeBundle {
                edge: Edge {
                    source: node_a,
                    target: node_b,
                    category: "test".to_string(),
                    weight: 1.0,
                },
                member: GraphMember { graph },
            })
            .id();

        // Update indices
        app.world_mut()
            .entity_mut(graph)
            .get_mut::<GraphIndex>()
            .unwrap()
            .nodes
            .insert(node_a);
        app.world_mut()
            .entity_mut(graph)
            .get_mut::<GraphIndex>()
            .unwrap()
            .nodes
            .insert(node_b);
        app.world_mut()
            .entity_mut(graph)
            .get_mut::<GraphIndex>()
            .unwrap()
            .edges
            .insert(edge);
        app.world_mut()
            .entity_mut(node_a)
            .get_mut::<NodeIndex>()
            .unwrap()
            .outgoing
            .push(edge);
        app.world_mut()
            .entity_mut(node_b)
            .get_mut::<NodeIndex>()
            .unwrap()
            .incoming
            .push(edge);

        // Test traversal
        let node_indices = app.world().query::<&NodeIndex>();
        let edges = app.world().query::<&Edge>();

        let neighbors = find_neighbors(node_a, &node_indices, &edges);
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0], node_b);
    }
}
