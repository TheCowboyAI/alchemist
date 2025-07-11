//! Graph Theory Algorithms for Component Detection and Analysis
//!
//! In graph theory, a "component" is a maximal connected subgraph.
//! This module provides algorithms to identify, extract, and work with
//! graph components for proper visualization and analysis.

use bevy::prelude::*;
use crate::graph_components::{GraphNode, GraphEdge, NodeConnections, Graph, GraphAlgorithmResult};
use std::collections::{HashMap, HashSet, VecDeque};

/// Identifies connected components in a graph
pub fn find_connected_components(
    nodes: &Query<(Entity, &GraphNode, &NodeConnections)>,
    edges: &Query<&GraphEdge>,
) -> Vec<GraphComponent> {
    let mut visited = HashSet::new();
    let mut components = Vec::new();
    let mut component_id = 0;
    
    // Build adjacency list
    let mut adjacency: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for (entity, _, connections) in nodes.iter() {
        let mut neighbors = Vec::new();
        neighbors.extend(&connections.outgoing);
        neighbors.extend(&connections.incoming);
        adjacency.insert(entity, neighbors);
    }
    
    // Find components using BFS
    for (entity, node, _) in nodes.iter() {
        if !visited.contains(&entity) {
            let component_nodes = bfs_component(entity, &adjacency, &mut visited);
            
            if !component_nodes.is_empty() {
                let component = GraphComponent {
                    id: component_id,
                    nodes: component_nodes.clone(),
                    edges: find_component_edges(&component_nodes, edges),
                    properties: analyze_component(&component_nodes, nodes, edges),
                };
                components.push(component);
                component_id += 1;
            }
        }
    }
    
    components
}

/// A connected component in the graph
#[derive(Debug, Clone)]
pub struct GraphComponent {
    pub id: usize,
    pub nodes: HashSet<Entity>,
    pub edges: HashSet<Entity>,
    pub properties: ComponentProperties,
}

/// Properties of a graph component
#[derive(Debug, Clone, Default)]
pub struct ComponentProperties {
    pub size: usize,
    pub density: f32,
    pub diameter: usize,
    pub is_tree: bool,
    pub is_cyclic: bool,
    pub is_bipartite: bool,
    pub center_nodes: Vec<Entity>,
    pub articulation_points: Vec<Entity>,
    pub bridges: Vec<Entity>,
}

/// Performs BFS to find all nodes in a component
fn bfs_component(
    start: Entity,
    adjacency: &HashMap<Entity, Vec<Entity>>,
    visited: &mut HashSet<Entity>,
) -> HashSet<Entity> {
    let mut component = HashSet::new();
    let mut queue = VecDeque::new();
    
    queue.push_back(start);
    visited.insert(start);
    component.insert(start);
    
    while let Some(current) = queue.pop_front() {
        if let Some(neighbors) = adjacency.get(&current) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    component.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }
    }
    
    component
}

/// Finds all edges within a component
fn find_component_edges(
    component_nodes: &HashSet<Entity>,
    edges: &Query<&GraphEdge>,
) -> HashSet<Entity> {
    let mut component_edges = HashSet::new();
    
    for (entity, edge) in edges.iter().enumerate() {
        if component_nodes.contains(&edge.source) && component_nodes.contains(&edge.target) {
            component_edges.insert(Entity::from_raw(entity as u32));
        }
    }
    
    component_edges
}

/// Analyzes properties of a component
fn analyze_component(
    component_nodes: &HashSet<Entity>,
    nodes: &Query<(Entity, &GraphNode, &NodeConnections)>,
    edges: &Query<&GraphEdge>,
) -> ComponentProperties {
    let mut props = ComponentProperties::default();
    props.size = component_nodes.len();
    
    // Calculate density
    let max_edges = props.size * (props.size - 1) / 2;
    let actual_edges = count_edges_in_component(component_nodes, edges);
    props.density = if max_edges > 0 {
        actual_edges as f32 / max_edges as f32
    } else {
        0.0
    };
    
    // Check if it's a tree
    props.is_tree = actual_edges == props.size - 1;
    props.is_cyclic = actual_edges >= props.size;
    
    // Find diameter
    props.diameter = calculate_diameter(component_nodes, nodes);
    
    // Find center nodes
    props.center_nodes = find_center_nodes(component_nodes, nodes);
    
    // Find articulation points
    props.articulation_points = find_articulation_points(component_nodes, nodes);
    
    // Check bipartiteness
    props.is_bipartite = is_bipartite(component_nodes, nodes);
    
    props
}

/// Counts edges within a component
fn count_edges_in_component(
    component_nodes: &HashSet<Entity>,
    edges: &Query<&GraphEdge>,
) -> usize {
    edges.iter()
        .filter(|edge| {
            component_nodes.contains(&edge.source) && component_nodes.contains(&edge.target)
        })
        .count()
}

/// Calculates the diameter (longest shortest path) of a component
fn calculate_diameter(
    component_nodes: &HashSet<Entity>,
    nodes: &Query<(Entity, &GraphNode, &NodeConnections)>,
) -> usize {
    let mut max_distance = 0;
    
    for &start in component_nodes {
        let distances = bfs_distances(start, component_nodes, nodes);
        for &distance in distances.values() {
            max_distance = max_distance.max(distance);
        }
    }
    
    max_distance
}

/// Performs BFS to find distances from a start node
fn bfs_distances(
    start: Entity,
    component_nodes: &HashSet<Entity>,
    nodes: &Query<(Entity, &GraphNode, &NodeConnections)>,
) -> HashMap<Entity, usize> {
    let mut distances = HashMap::new();
    let mut queue = VecDeque::new();
    
    distances.insert(start, 0);
    queue.push_back(start);
    
    while let Some(current) = queue.pop_front() {
        let current_distance = distances[&current];
        
        if let Ok((_, _, connections)) = nodes.get(current) {
            for &neighbor in connections.outgoing.iter().chain(&connections.incoming) {
                if component_nodes.contains(&neighbor) && !distances.contains_key(&neighbor) {
                    distances.insert(neighbor, current_distance + 1);
                    queue.push_back(neighbor);
                }
            }
        }
    }
    
    distances
}

/// Finds the center nodes (nodes with minimum eccentricity)
fn find_center_nodes(
    component_nodes: &HashSet<Entity>,
    nodes: &Query<(Entity, &GraphNode, &NodeConnections)>,
) -> Vec<Entity> {
    let mut eccentricities = HashMap::new();
    
    for &node in component_nodes {
        let distances = bfs_distances(node, component_nodes, nodes);
        let eccentricity = distances.values().max().copied().unwrap_or(0);
        eccentricities.insert(node, eccentricity);
    }
    
    let min_eccentricity = eccentricities.values().min().copied().unwrap_or(0);
    
    eccentricities
        .into_iter()
        .filter(|(_, ecc)| *ecc == min_eccentricity)
        .map(|(node, _)| node)
        .collect()
}

/// Finds articulation points (nodes whose removal disconnects the graph)
fn find_articulation_points(
    component_nodes: &HashSet<Entity>,
    nodes: &Query<(Entity, &GraphNode, &NodeConnections)>,
) -> Vec<Entity> {
    let mut articulation_points = Vec::new();
    
    for &node in component_nodes {
        // Remove node temporarily and check connectivity
        let mut temp_nodes = component_nodes.clone();
        temp_nodes.remove(&node);
        
        if !temp_nodes.is_empty() {
            let components = find_components_excluding(temp_nodes, nodes, node);
            if components.len() > 1 {
                articulation_points.push(node);
            }
        }
    }
    
    articulation_points
}

/// Helper to find components excluding a specific node
fn find_components_excluding(
    nodes_set: HashSet<Entity>,
    nodes: &Query<(Entity, &GraphNode, &NodeConnections)>,
    exclude: Entity,
) -> Vec<HashSet<Entity>> {
    let mut visited = HashSet::new();
    let mut components = Vec::new();
    
    for &node in &nodes_set {
        if !visited.contains(&node) && node != exclude {
            let mut component = HashSet::new();
            let mut queue = VecDeque::new();
            
            queue.push_back(node);
            visited.insert(node);
            
            while let Some(current) = queue.pop_front() {
                component.insert(current);
                
                if let Ok((_, _, connections)) = nodes.get(current) {
                    for &neighbor in connections.outgoing.iter().chain(&connections.incoming) {
                        if nodes_set.contains(&neighbor) && 
                           !visited.contains(&neighbor) && 
                           neighbor != exclude {
                            visited.insert(neighbor);
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
            
            if !component.is_empty() {
                components.push(component);
            }
        }
    }
    
    components
}

/// Checks if a component is bipartite
fn is_bipartite(
    component_nodes: &HashSet<Entity>,
    nodes: &Query<(Entity, &GraphNode, &NodeConnections)>,
) -> bool {
    let mut colors: HashMap<Entity, bool> = HashMap::new();
    let mut queue = VecDeque::new();
    
    // Start with arbitrary node
    if let Some(&start) = component_nodes.iter().next() {
        colors.insert(start, false);
        queue.push_back(start);
        
        while let Some(current) = queue.pop_front() {
            let current_color = colors[&current];
            
            if let Ok((_, _, connections)) = nodes.get(current) {
                for &neighbor in connections.outgoing.iter().chain(&connections.incoming) {
                    if component_nodes.contains(&neighbor) {
                        if let Some(&neighbor_color) = colors.get(&neighbor) {
                            if neighbor_color == current_color {
                                return false; // Not bipartite
                            }
                        } else {
                            colors.insert(neighbor, !current_color);
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }
    }
    
    true
}

/// System to identify and mark graph components
pub fn identify_graph_components_system(
    nodes: Query<(Entity, &GraphNode, &NodeConnections)>,
    edges: Query<&GraphEdge>,
    mut commands: Commands,
    mut graph_manager: ResMut<crate::graph_components::GraphManager>,
) {
    let components = find_connected_components(&nodes, &edges);
    
    // Assign component IDs to nodes
    for component in components.iter() {
        for &node_entity in &component.nodes {
            commands.entity(node_entity).insert(ComponentMembership {
                component_id: component.id,
                component_size: component.properties.size,
                is_center: component.properties.center_nodes.contains(&node_entity),
                is_articulation_point: component.properties.articulation_points.contains(&node_entity),
            });
        }
    }
    
    info!("Identified {} connected components", components.len());
}

/// Component membership for nodes
#[derive(Component)]
pub struct ComponentMembership {
    pub component_id: usize,
    pub component_size: usize,
    pub is_center: bool,
    pub is_articulation_point: bool,
}

/// Layout strategy for multiple components
pub fn layout_multiple_components(
    components: &[GraphComponent],
    nodes: &mut Query<&mut Transform, With<GraphNode>>,
) {
    let grid_size = (components.len() as f32).sqrt().ceil() as usize;
    let spacing = 20.0;
    
    for (idx, component) in components.iter().enumerate() {
        let row = idx / grid_size;
        let col = idx % grid_size;
        let offset = Vec3::new(
            col as f32 * spacing - (grid_size as f32 * spacing / 2.0),
            0.0,
            row as f32 * spacing - (grid_size as f32 * spacing / 2.0),
        );
        
        // Apply offset to all nodes in this component
        for &node in &component.nodes {
            if let Ok(mut transform) = nodes.get_mut(node) {
                transform.translation += offset;
            }
        }
    }
}

/// Extracts a subgraph containing only specified components
pub fn extract_component_subgraph(
    component_ids: &[usize],
    components: &[GraphComponent],
) -> (HashSet<Entity>, HashSet<Entity>) {
    let mut nodes = HashSet::new();
    let mut edges = HashSet::new();
    
    for component in components.iter() {
        if component_ids.contains(&component.id) {
            nodes.extend(&component.nodes);
            edges.extend(&component.edges);
        }
    }
    
    (nodes, edges)
}

/// Merges multiple components into a single component
pub fn merge_components(
    component_ids: &[usize],
    components: &mut Vec<GraphComponent>,
) -> Option<GraphComponent> {
    if component_ids.is_empty() {
        return None;
    }
    
    let mut merged = GraphComponent {
        id: component_ids[0],
        nodes: HashSet::new(),
        edges: HashSet::new(),
        properties: ComponentProperties::default(),
    };
    
    for &id in component_ids {
        if let Some(component) = components.iter().find(|c| c.id == id) {
            merged.nodes.extend(&component.nodes);
            merged.edges.extend(&component.edges);
        }
    }
    
    // Remove merged components and add the new one
    components.retain(|c| !component_ids.contains(&c.id));
    components.push(merged.clone());
    
    Some(merged)
}

/// Visualizes component boundaries
pub fn visualize_component_boundaries(
    components: &[GraphComponent],
    nodes: &Query<&Transform, With<GraphNode>>,
    mut gizmos: Gizmos,
) {
    for component in components {
        // Calculate bounding box for component
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        
        for &node in &component.nodes {
            if let Ok(transform) = nodes.get(node) {
                min = min.min(transform.translation - Vec3::splat(2.0));
                max = max.max(transform.translation + Vec3::splat(2.0));
            }
        }
        
        // Draw bounding box
        let color = Color::hsla(
            (component.id as f32 * 67.0) % 360.0,
            0.7,
            0.5,
            0.3,
        );
        
        // Draw box edges
        let corners = [
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(max.x, max.y, max.z),
            Vec3::new(min.x, max.y, max.z),
        ];
        
        // Bottom face
        gizmos.line(corners[0], corners[1], color);
        gizmos.line(corners[1], corners[2], color);
        gizmos.line(corners[2], corners[3], color);
        gizmos.line(corners[3], corners[0], color);
        
        // Top face
        gizmos.line(corners[4], corners[5], color);
        gizmos.line(corners[5], corners[6], color);
        gizmos.line(corners[6], corners[7], color);
        gizmos.line(corners[7], corners[4], color);
        
        // Vertical edges
        gizmos.line(corners[0], corners[4], color);
        gizmos.line(corners[1], corners[5], color);
        gizmos.line(corners[2], corners[6], color);
        gizmos.line(corners[3], corners[7], color);
    }
}