//! Systems for graph algorithms and analysis
//!
//! These systems handle:
//! - Pathfinding algorithms
//! - Layout algorithms
//! - Graph metrics calculation
//! - Centrality analysis

use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    components::*,
    events::*,
    resources::*,
};

/// System that handles graph analysis requests
///
/// This system performs various graph analyses based on the requested type
pub fn handle_graph_analysis(
    mut events: EventReader<AnalyzeGraphEvent>,
    nodes: Query<(Entity, &NodeId)>,
    edges: Query<&Edge>,
    uuid_to_entity: Res<UuidToEntity>,
    mut notification_events: EventWriter<ShowNotificationEvent>,
) {
    for event in events.read() {
        match &event.analysis_type {
            GraphAnalysisType::ShortestPath { from, to } => {
                if let (Some(&from_entity), Some(&to_entity)) = (
                    uuid_to_entity.0.get(from),
                    uuid_to_entity.0.get(to)
                ) {
                    let path = find_shortest_path(from_entity, to_entity, &edges);

                    let message = if let Some(path) = path {
                        format!("Shortest path found with {} steps", path.len() - 1)
                    } else {
                        "No path found between nodes".to_string()
                    };

                    notification_events.send(ShowNotificationEvent {
                        message,
                        notification_type: NotificationType::Info,
                        duration_seconds: 3.0,
                    });
                }
            }
            GraphAnalysisType::Centrality => {
                let centrality = calculate_centrality(&nodes, &edges);

                // Find most central node
                if let Some((node_id, score)) = centrality.iter()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                {
                    notification_events.send(ShowNotificationEvent {
                        message: format!("Most central node has score: {:.2}", score),
                        notification_type: NotificationType::Info,
                        duration_seconds: 3.0,
                    });
                }
            }
            GraphAnalysisType::TopologicalSort => {
                let sorted = topological_sort(&nodes, &edges);

                let message = match sorted {
                    Ok(order) => format!("Topological sort completed: {} nodes ordered", order.len()),
                    Err(_) => "Cannot perform topological sort: graph contains cycles".to_string(),
                };

                notification_events.send(ShowNotificationEvent {
                    message,
                    notification_type: if sorted.is_ok() { NotificationType::Success } else { NotificationType::Warning },
                    duration_seconds: 3.0,
                });
            }
            _ => {} // Other analysis types handled elsewhere
        }
    }
}

/// System that calculates graph layout
///
/// This system applies various layout algorithms to position nodes
pub fn handle_layout_request(
    mut events: EventReader<RequestLayoutEvent>,
    nodes: Query<(Entity, &Transform), With<NodeId>>,
    edges: Query<&Edge>,
    mut batch_move_events: EventWriter<BatchMoveNodesEvent>,
) {
    for event in events.read() {
        let positions = match &event.layout_type {
            LayoutType::ForceDirected => calculate_force_directed_layout(&nodes, &edges),
            LayoutType::Hierarchical => calculate_hierarchical_layout(&nodes, &edges),
            LayoutType::Circular => calculate_circular_layout(&nodes),
            LayoutType::Grid => calculate_grid_layout(&nodes),
        };

        // Convert to move events
        let moves: Vec<(Entity, Vec3, Vec3)> = positions.into_iter()
            .filter_map(|(entity, new_pos)| {
                nodes.get(entity).ok().map(|(_, transform)| {
                    (entity, transform.translation, new_pos)
                })
            })
            .collect();

        if !moves.is_empty() {
            batch_move_events.send(BatchMoveNodesEvent { moves });
        }
    }
}

/// System that updates graph metrics periodically
///
/// This system calculates and updates various graph metrics
pub fn update_graph_metrics(
    time: Res<Time>,
    mut timer: ResMut<MetricsUpdateTimer>,
    nodes: Query<&NodeId>,
    edges: Query<&Edge>,
    mut metrics_events: EventWriter<GraphMetricsEvent>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let node_count = nodes.iter().count();
        let edge_count = edges.iter().count();

        // Build adjacency for analysis
        let mut adjacency: HashMap<Entity, Vec<Entity>> = HashMap::new();
        for edge in edges.iter() {
            adjacency.entry(edge.source).or_default().push(edge.target);
        }

        let has_cycles = detect_cycles(&adjacency);
        let connected_components = count_components(&nodes, &edges);

        metrics_events.send(GraphMetricsEvent {
            node_count,
            edge_count,
            connected_components,
            has_cycles,
        });
    }
}

// Algorithm implementations

fn find_shortest_path(
    start: Entity,
    end: Entity,
    edges: &Query<&Edge>,
) -> Option<Vec<Entity>> {
    // Build adjacency list
    let mut adjacency: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for edge in edges.iter() {
        adjacency.entry(edge.source).or_default().push(edge.target);
    }

    // BFS for shortest path
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut parent: HashMap<Entity, Entity> = HashMap::new();

    queue.push_back(start);
    visited.insert(start);

    while let Some(current) = queue.pop_front() {
        if current == end {
            // Reconstruct path
            let mut path = vec![end];
            let mut node = end;

            while let Some(&p) = parent.get(&node) {
                path.push(p);
                node = p;
            }

            path.reverse();
            return Some(path);
        }

        if let Some(neighbors) = adjacency.get(&current) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    parent.insert(neighbor, current);
                    queue.push_back(neighbor);
                }
            }
        }
    }

    None
}

fn calculate_centrality(
    nodes: &Query<(Entity, &NodeId)>,
    edges: &Query<&Edge>,
) -> HashMap<Entity, f32> {
    let mut centrality = HashMap::new();

    // Simple degree centrality
    for (entity, _) in nodes.iter() {
        let mut degree = 0;

        for edge in edges.iter() {
            if edge.source == entity || edge.target == entity {
                degree += 1;
            }
        }

        centrality.insert(entity, degree as f32);
    }

    // Normalize
    let max_degree = centrality.values().cloned().fold(0.0, f32::max);
    if max_degree > 0.0 {
        for value in centrality.values_mut() {
            *value /= max_degree;
        }
    }

    centrality
}

fn topological_sort(
    nodes: &Query<(Entity, &NodeId)>,
    edges: &Query<&Edge>,
) -> Result<Vec<Entity>, ()> {
    // Build adjacency and in-degree
    let mut adjacency: HashMap<Entity, Vec<Entity>> = HashMap::new();
    let mut in_degree: HashMap<Entity, usize> = HashMap::new();

    for (entity, _) in nodes.iter() {
        in_degree.insert(entity, 0);
        adjacency.insert(entity, Vec::new());
    }

    for edge in edges.iter() {
        adjacency.get_mut(&edge.source).unwrap().push(edge.target);
        *in_degree.get_mut(&edge.target).unwrap() += 1;
    }

    // Kahn's algorithm
    let mut queue = VecDeque::new();
    let mut result = Vec::new();

    // Find nodes with no incoming edges
    for (entity, &degree) in &in_degree {
        if degree == 0 {
            queue.push_back(*entity);
        }
    }

    while let Some(node) = queue.pop_front() {
        result.push(node);

        if let Some(neighbors) = adjacency.get(&node) {
            for &neighbor in neighbors {
                let degree = in_degree.get_mut(&neighbor).unwrap();
                *degree -= 1;

                if *degree == 0 {
                    queue.push_back(neighbor);
                }
            }
        }
    }

    if result.len() == nodes.iter().count() {
        Ok(result)
    } else {
        Err(()) // Cycle detected
    }
}

fn calculate_force_directed_layout(
    nodes: &Query<(Entity, &Transform), With<NodeId>>,
    edges: &Query<&Edge>,
) -> HashMap<Entity, Vec3> {
    let mut positions = HashMap::new();
    let mut forces: HashMap<Entity, Vec3> = HashMap::new();

    // Initialize positions
    for (entity, transform) in nodes.iter() {
        positions.insert(entity, transform.translation);
        forces.insert(entity, Vec3::ZERO);
    }

    // Parameters
    const REPULSION_STRENGTH: f32 = 100.0;
    const ATTRACTION_STRENGTH: f32 = 0.1;
    const DAMPING: f32 = 0.9;
    const ITERATIONS: usize = 50;

    for _ in 0..ITERATIONS {
        // Clear forces
        for force in forces.values_mut() {
            *force = Vec3::ZERO;
        }

        // Repulsion between all nodes
        let entities: Vec<Entity> = nodes.iter().map(|(e, _)| e).collect();
        for i in 0..entities.len() {
            for j in (i + 1)..entities.len() {
                let pos_i = positions[&entities[i]];
                let pos_j = positions[&entities[j]];

                let diff = pos_i - pos_j;
                let distance = diff.length().max(1.0);
                let force = diff.normalize() * (REPULSION_STRENGTH / (distance * distance));

                *forces.get_mut(&entities[i]).unwrap() += force;
                *forces.get_mut(&entities[j]).unwrap() -= force;
            }
        }

        // Attraction along edges
        for edge in edges.iter() {
            let pos_source = positions[&edge.source];
            let pos_target = positions[&edge.target];

            let diff = pos_target - pos_source;
            let distance = diff.length();
            let force = diff.normalize() * (distance * ATTRACTION_STRENGTH);

            *forces.get_mut(&edge.source).unwrap() += force;
            *forces.get_mut(&edge.target).unwrap() -= force;
        }

        // Apply forces
        for (entity, pos) in positions.iter_mut() {
            let force = forces[entity] * DAMPING;
            *pos += force;
            pos.y = 0.0; // Keep on XZ plane
        }
    }

    positions
}

fn calculate_hierarchical_layout(
    nodes: &Query<(Entity, &Transform), With<NodeId>>,
    edges: &Query<&Edge>,
) -> HashMap<Entity, Vec3> {
    let mut positions = HashMap::new();

    // Find root nodes (no incoming edges)
    let mut has_incoming = HashSet::new();
    for edge in edges.iter() {
        has_incoming.insert(edge.target);
    }

    let mut roots = Vec::new();
    for (entity, _) in nodes.iter() {
        if !has_incoming.contains(&entity) {
            roots.push(entity);
        }
    }

    // Build adjacency
    let mut children: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for edge in edges.iter() {
        children.entry(edge.source).or_default().push(edge.target);
    }

    // Layout by levels
    let mut level = 0;
    let mut current_level = roots;
    let level_height = 5.0;
    let node_spacing = 3.0;

    while !current_level.is_empty() {
        let y = -(level as f32) * level_height;
        let total_width = (current_level.len() as f32 - 1.0) * node_spacing;

        for (i, &entity) in current_level.iter().enumerate() {
            let x = (i as f32) * node_spacing - total_width / 2.0;
            positions.insert(entity, Vec3::new(x, y, 0.0));
        }

        // Get next level
        let mut next_level = Vec::new();
        for &entity in &current_level {
            if let Some(entity_children) = children.get(&entity) {
                next_level.extend(entity_children);
            }
        }

        current_level = next_level;
        level += 1;
    }

    positions
}

fn calculate_circular_layout(
    nodes: &Query<(Entity, &Transform), With<NodeId>>,
) -> HashMap<Entity, Vec3> {
    let mut positions = HashMap::new();
    let entities: Vec<Entity> = nodes.iter().map(|(e, _)| e).collect();

    let radius = 10.0 * (entities.len() as f32).sqrt();

    for (i, &entity) in entities.iter().enumerate() {
        let angle = (i as f32 / entities.len() as f32) * std::f32::consts::TAU;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;

        positions.insert(entity, Vec3::new(x, 0.0, z));
    }

    positions
}

fn calculate_grid_layout(
    nodes: &Query<(Entity, &Transform), With<NodeId>>,
) -> HashMap<Entity, Vec3> {
    let mut positions = HashMap::new();
    let entities: Vec<Entity> = nodes.iter().map(|(e, _)| e).collect();

    let grid_size = (entities.len() as f32).sqrt().ceil() as usize;
    let spacing = 5.0;

    for (i, &entity) in entities.iter().enumerate() {
        let row = i / grid_size;
        let col = i % grid_size;

        let x = (col as f32 - grid_size as f32 / 2.0) * spacing;
        let z = (row as f32 - grid_size as f32 / 2.0) * spacing;

        positions.insert(entity, Vec3::new(x, 0.0, z));
    }

    positions
}

fn detect_cycles(adjacency: &HashMap<Entity, Vec<Entity>>) -> bool {
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    for &node in adjacency.keys() {
        if !visited.contains(&node) {
            if has_cycle_dfs(node, adjacency, &mut visited, &mut rec_stack) {
                return true;
            }
        }
    }

    false
}

fn has_cycle_dfs(
    node: Entity,
    adjacency: &HashMap<Entity, Vec<Entity>>,
    visited: &mut HashSet<Entity>,
    rec_stack: &mut HashSet<Entity>,
) -> bool {
    visited.insert(node);
    rec_stack.insert(node);

    if let Some(neighbors) = adjacency.get(&node) {
        for &neighbor in neighbors {
            if !visited.contains(&neighbor) {
                if has_cycle_dfs(neighbor, adjacency, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack.contains(&neighbor) {
                return true;
            }
        }
    }

    rec_stack.remove(&node);
    false
}

fn count_components(
    nodes: &Query<&NodeId>,
    edges: &Query<&Edge>,
) -> usize {
    // Implementation would be similar to find_connected_components in validation.rs
    // For now, return 1 as placeholder
    1
}
