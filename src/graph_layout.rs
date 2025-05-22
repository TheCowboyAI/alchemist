use bevy::prelude::*;
use crate::graph::{AlchemistGraph, GraphNode, GraphEdge};
use crate::graph_editor_3d::{GraphEditor3D, UpdateGraph3DEvent};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Parameters for the force-directed layout algorithm
#[derive(Clone, Debug)]
pub struct ForceDirectedParams {
    /// Spring constant for Hooke's law (attraction force)
    pub spring_constant: f32,
    /// Optimal distance between connected nodes
    pub optimal_distance: f32,
    /// Coulomb constant for repulsion force
    pub repulsion_constant: f32,
    /// Damping factor to prevent oscillation
    pub damping: f32,
    /// Maximum force magnitude to apply in a single step (prevents extreme movements)
    pub max_force: f32,
    /// Minimum distance to prevent division by zero
    pub min_distance: f32,
}

impl Default for ForceDirectedParams {
    fn default() -> Self {
        Self {
            spring_constant: 0.3,
            optimal_distance: 3.0,
            repulsion_constant: 20.0,
            damping: 0.8,
            max_force: 10.0,
            min_distance: 0.1,
        }
    }
}

/// Resource to control the layout behavior
#[derive(Resource)]
pub struct GraphLayoutController {
    pub enabled: bool,
    pub params: ForceDirectedParams,
    pub iterations_per_update: usize,
    pub iteration_count: usize,
    pub max_iterations: usize,
    pub stabilization_threshold: f32,
    pub temperature: f32,
}

impl Default for GraphLayoutController {
    fn default() -> Self {
        Self {
            enabled: true,
            params: ForceDirectedParams::default(),
            iterations_per_update: 5,
            iteration_count: 0,
            max_iterations: 500,
            stabilization_threshold: 0.01,
            temperature: 0.9,
        }
    }
}

/// Event to request a layout update
#[derive(Event)]
pub struct LayoutUpdateEvent;

/// Plugin for graph layout
pub struct GraphLayoutPlugin;

impl Plugin for GraphLayoutPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<GraphLayoutController>()
            .add_event::<LayoutUpdateEvent>()
            .add_systems(Update, (
                handle_layout_update,
                apply_force_directed_layout.run_if(|res: Option<Res<GraphEditor3D>>| res.is_some()),
            ));
    }
}

/// System to handle layout update events
fn handle_layout_update(
    mut event_reader: EventReader<LayoutUpdateEvent>,
    mut controller: ResMut<GraphLayoutController>,
) {
    for _ in event_reader.read() {
        // Reset iteration counter and temperature when a new layout update is triggered
        controller.iteration_count = 0;
        controller.temperature = 0.9;
    }
}

/// System that applies the force-directed layout algorithm
fn apply_force_directed_layout(
    mut graph_editor: ResMut<GraphEditor3D>,
    mut controller: ResMut<GraphLayoutController>,
    mut update_events: EventWriter<UpdateGraph3DEvent>,
    _time: Res<Time>,
) {
    if !controller.enabled || 
       controller.iteration_count >= controller.max_iterations ||
       controller.temperature < controller.stabilization_threshold {
        return;
    }

    // Process multiple iterations per frame
    let mut total_movement = 0.0;
    for _ in 0..controller.iterations_per_update {
        // Skip if we've reached max iterations
        if controller.iteration_count >= controller.max_iterations {
            break;
        }

        total_movement += force_directed_iteration(&mut graph_editor.graph, &controller.params, controller.temperature);
        
        // Cool down the system
        controller.temperature *= 0.99;
        controller.iteration_count += 1;

        // If movement is very small, consider the layout stable
        if total_movement < controller.stabilization_threshold {
            break;
        }
    }

    // Update the visualization
    update_events.write(UpdateGraph3DEvent);
}

/// A single iteration of the force-directed layout algorithm
fn force_directed_iteration(
    graph: &mut AlchemistGraph,
    params: &ForceDirectedParams,
    temperature: f32,
) -> f32 {
    let node_ids: Vec<Uuid> = graph.nodes.keys().cloned().collect();
    let node_count = node_ids.len();
    
    // Initialize a map to store the forces for each node
    let mut forces: HashMap<Uuid, Vec3> = HashMap::new();
    for id in &node_ids {
        forces.insert(*id, Vec3::ZERO);
    }

    // Calculate repulsive forces (Coulomb's law) between all pairs of nodes
    for i in 0..node_count {
        for j in i+1..node_count {
            let id1 = node_ids[i];
            let id2 = node_ids[j];
            
            let pos1 = get_node_position(graph, id1);
            let pos2 = get_node_position(graph, id2);
            
            let direction = pos1 - pos2;
            let distance = direction.length().max(params.min_distance);
            let normalized_direction = direction.normalize_or_zero();
            
            // Coulomb's law: F = k * q1 * q2 / r^2
            // We simplify by assuming all nodes have the same charge (q1 = q2 = 1)
            let repulsion_force = params.repulsion_constant / (distance * distance);
            let force = normalized_direction * repulsion_force;
            
            // Apply repulsive force to both nodes (equal and opposite)
            if let Some(f) = forces.get_mut(&id1) {
                *f += force;
            }
            if let Some(f) = forces.get_mut(&id2) {
                *f -= force;
            }
        }
    }

    // Calculate attractive forces (Hooke's law) between connected nodes
    for (_, edge) in &graph.edges {
        let source_id = edge.source;
        let target_id = edge.target;
        
        let source_pos = get_node_position(graph, source_id);
        let target_pos = get_node_position(graph, target_id);
        
        let direction = source_pos - target_pos;
        let distance = direction.length().max(params.min_distance);
        let normalized_direction = direction.normalize_or_zero();
        
        // Hooke's law: F = -k * (x - r)
        // Where k is spring constant, x is current distance, r is optimal distance
        let displacement = distance - params.optimal_distance;
        let spring_force = params.spring_constant * edge.weight * displacement;
        let force = normalized_direction * spring_force;
        
        // Apply attractive force (spring pulls nodes together if they're too far apart,
        // or pushes them away if they're too close)
        if let Some(f) = forces.get_mut(&source_id) {
            *f -= force;
        }
        if let Some(f) = forces.get_mut(&target_id) {
            *f += force;
        }
    }

    // Apply forces to update node positions
    let mut total_movement = 0.0;
    for id in &node_ids {
        if let Some(force) = forces.get(id) {
            // Limit force to prevent extreme movements
            let clamped_force = if force.length() > params.max_force {
                force.normalize() * params.max_force
            } else {
                *force
            };
            
            // Apply damping and temperature
            let movement = clamped_force * params.damping * temperature;
            total_movement += movement.length();
            
            // Update position
            let current_pos = get_node_position(graph, *id);
            let new_pos = current_pos + movement;
            
            // Store new position in graph
            set_node_position(graph, *id, new_pos);
        }
    }

    total_movement / node_count as f32
}

/// Helper function to get a node's 3D position
fn get_node_position(graph: &AlchemistGraph, id: Uuid) -> Vec3 {
    if let Some(pos) = graph.node_positions.get(&id) {
        // Convert 2D position to 3D
        Vec3::new(pos.x, 0.0, pos.y)
    } else {
        // Default position if none exists
        Vec3::ZERO
    }
}

/// Helper function to set a node's position in the graph
fn set_node_position(graph: &mut AlchemistGraph, id: Uuid, position: Vec3) {
    // Convert 3D position back to 2D for storage
    graph.node_positions.insert(id, egui::Pos2::new(position.x, position.z));
}

/// Function to apply an initial layout to a specific graph pattern
pub fn apply_initial_layout(graph: &mut AlchemistGraph, pattern_name: &str) {
    match pattern_name {
        "binary_tree" => apply_tree_layout(graph),
        "small_star" => apply_star_layout(graph),
        "pentagon" | "hexagon" | "octagon" => apply_polygon_layout(graph),
        "cycle" => apply_cycle_layout(graph),
        "small_grid" => apply_grid_layout(graph),
        "simple_dfa" | "simple_nfa" => apply_automaton_layout(graph),
        "small_dag" => apply_dag_layout(graph),
        "basic_bipartite" => apply_bipartite_layout(graph),
        // For other patterns, use a basic circular layout
        _ => apply_circular_layout(graph),
    }
}

/// Apply a tree layout pattern
fn apply_tree_layout(graph: &mut AlchemistGraph) {
    // Find the root node (has no incoming edges)
    let mut incoming_edges: HashMap<Uuid, usize> = HashMap::new();
    for (_, edge) in &graph.edges {
        *incoming_edges.entry(edge.target).or_insert(0) += 1;
    }
    
    let root_candidates: Vec<Uuid> = graph.nodes.keys()
        .filter(|id| !incoming_edges.contains_key(id))
        .cloned()
        .collect();
    
    if let Some(root_id) = root_candidates.first() {
        // Map each node to its level in the tree
        let mut node_levels: HashMap<Uuid, usize> = HashMap::new();
        node_levels.insert(*root_id, 0);
        
        // Position based on level (breadth-first assignment)
        let mut current_level = 0;
        let mut current_level_nodes = vec![*root_id];
        
        while !current_level_nodes.is_empty() {
            let mut next_level_nodes = Vec::new();
            let level_width = current_level_nodes.len() as f32;
            
            for (i, node_id) in current_level_nodes.iter().enumerate() {
                // Position horizontally based on position in level
                let x = (i as f32 - (level_width - 1.0) / 2.0) * 3.0;
                let y = current_level as f32 * -3.0; // Negative y to go downward
                
                graph.node_positions.insert(*node_id, egui::Pos2::new(x, y));
                
                // Find children
                for (_, edge) in &graph.edges {
                    if edge.source == *node_id {
                        node_levels.insert(edge.target, current_level + 1);
                        next_level_nodes.push(edge.target);
                    }
                }
            }
            
            current_level_nodes = next_level_nodes;
            current_level += 1;
        }
    }
}

/// Apply a star layout pattern
fn apply_star_layout(graph: &mut AlchemistGraph) {
    // Find the center node (has the most connections)
    let mut connection_count: HashMap<Uuid, usize> = HashMap::new();
    for (_, edge) in &graph.edges {
        *connection_count.entry(edge.source).or_insert(0) += 1;
        *connection_count.entry(edge.target).or_insert(0) += 1;
    }
    
    if let Some((center_id, _)) = connection_count.iter().max_by_key(|(_, count)| *count) {
        // Position center node at origin
        graph.node_positions.insert(*center_id, egui::Pos2::ZERO);
        
        // Position other nodes in a circle around the center
        let mut peripheral_nodes: Vec<Uuid> = graph.nodes.keys()
            .filter(|id| *id != center_id)
            .cloned()
            .collect();
            
        let count = peripheral_nodes.len() as f32;
        let radius = 5.0;
        
        for (i, node_id) in peripheral_nodes.iter().enumerate() {
            let angle = (i as f32 / count) * std::f32::consts::TAU;
            let x = radius * angle.cos();
            let y = radius * angle.sin();
            
            graph.node_positions.insert(*node_id, egui::Pos2::new(x, y));
        }
    }
}

/// Apply a polygon layout pattern
fn apply_polygon_layout(graph: &mut AlchemistGraph) {
    let nodes: Vec<Uuid> = graph.nodes.keys().cloned().collect();
    let count = nodes.len() as f32;
    let radius = 5.0;
    
    for (i, node_id) in nodes.iter().enumerate() {
        let angle = (i as f32 / count) * std::f32::consts::TAU;
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        
        graph.node_positions.insert(*node_id, egui::Pos2::new(x, y));
    }
}

/// Apply a cycle layout pattern
fn apply_cycle_layout(graph: &mut AlchemistGraph) {
    // Same as polygon layout
    apply_polygon_layout(graph);
}

/// Apply a grid layout pattern
fn apply_grid_layout(graph: &mut AlchemistGraph) {
    let nodes: Vec<Uuid> = graph.nodes.keys().cloned().collect();
    
    // Try to determine grid dimensions from the pattern
    let node_count = nodes.len();
    let width = (node_count as f32).sqrt().round() as usize;
    let height = (node_count + width - 1) / width;
    
    let cell_size = 3.0;
    let x_offset = (width as f32 - 1.0) * cell_size / 2.0;
    let y_offset = (height as f32 - 1.0) * cell_size / 2.0;
    
    for (i, node_id) in nodes.iter().enumerate() {
        let row = i / width;
        let col = i % width;
        
        let x = (col as f32 * cell_size) - x_offset;
        let y = (row as f32 * cell_size) - y_offset;
        
        graph.node_positions.insert(*node_id, egui::Pos2::new(x, y));
    }
}

/// Apply a automaton layout pattern
fn apply_automaton_layout(graph: &mut AlchemistGraph) {
    // Similar to circular layout but with starting state at the left
    let nodes: Vec<Uuid> = graph.nodes.keys().cloned().collect();
    let count = nodes.len() as f32;
    let radius = 5.0;
    
    // Find start state (typically has a label indicating it's the start)
    let mut start_state = nodes[0];
    for (id, node) in &graph.nodes {
        if node.labels.contains(&"start".to_string()) {
            start_state = *id;
            break;
        }
    }
    
    // Position start state at the left
    graph.node_positions.insert(start_state, egui::Pos2::new(-radius, 0.0));
    
    // Position other states in a semicircle
    let other_nodes: Vec<Uuid> = nodes.into_iter()
        .filter(|id| *id != start_state)
        .collect();
    
    let other_count = other_nodes.len() as f32;
    for (i, node_id) in other_nodes.iter().enumerate() {
        // Distribute in 3/4 of a circle
        let angle = std::f32::consts::PI * 0.25 + 
                    (i as f32 / other_count) * std::f32::consts::PI * 1.5;
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        
        graph.node_positions.insert(*node_id, egui::Pos2::new(x, y));
    }
}

/// Apply a DAG layout
fn apply_dag_layout(graph: &mut AlchemistGraph) {
    // Find nodes with no incoming edges (sources)
    let mut incoming_edges: HashMap<Uuid, usize> = HashMap::new();
    for (_, edge) in &graph.edges {
        *incoming_edges.entry(edge.target).or_insert(0) += 1;
    }
    
    let sources: Vec<Uuid> = graph.nodes.keys()
        .filter(|id| !incoming_edges.contains_key(id))
        .cloned()
        .collect();
    
    if !sources.is_empty() {
        // Assign each node to a layer based on longest path from a source
        let mut layers: HashMap<Uuid, usize> = HashMap::new();
        let mut visited = HashSet::new();
        
        let mut current_layer = 0;
        let mut current_nodes = sources.clone();
        
        while !current_nodes.is_empty() {
            let mut next_nodes = Vec::new();
            
            for node_id in &current_nodes {
                if visited.contains(node_id) {
                    continue;
                }
                
                visited.insert(*node_id);
                layers.insert(*node_id, current_layer);
                
                // Find successors
                for (_, edge) in &graph.edges {
                    if edge.source == *node_id {
                        next_nodes.push(edge.target);
                    }
                }
            }
            
            current_nodes = next_nodes;
            current_layer += 1;
        }
        
        // Count nodes in each layer
        let mut layer_counts: HashMap<usize, usize> = HashMap::new();
        for &layer in layers.values() {
            *layer_counts.entry(layer).or_insert(0) += 1;
        }
        
        // Position nodes by layer
        let mut layer_positions: HashMap<usize, usize> = HashMap::new();
        for (node_id, layer) in &layers {
            let position = *layer_positions.entry(*layer).or_insert(0);
            let count = *layer_counts.get(layer).unwrap_or(&1) as f32;
            let x = (position as f32 - (count - 1.0) / 2.0) * 3.0;
            let y = *layer as f32 * 3.0;
            
            graph.node_positions.insert(*node_id, egui::Pos2::new(x, y));
            
            *layer_positions.entry(*layer).or_insert(0) += 1;
        }
    }
}

/// Apply a bipartite layout
fn apply_bipartite_layout(graph: &mut AlchemistGraph) {
    // Try to identify the two sets of nodes
    let mut left_nodes = Vec::new();
    let mut right_nodes = Vec::new();
    
    // Use a simple heuristic: nodes that only have outgoing edges are left nodes,
    // nodes that only have incoming edges are right nodes
    let mut incoming = HashSet::new();
    let mut outgoing = HashSet::new();
    
    for (_, edge) in &graph.edges {
        incoming.insert(edge.target);
        outgoing.insert(edge.source);
    }
    
    for node_id in graph.nodes.keys() {
        if outgoing.contains(node_id) && !incoming.contains(node_id) {
            left_nodes.push(*node_id);
        } else if incoming.contains(node_id) && !outgoing.contains(node_id) {
            right_nodes.push(*node_id);
        } else if left_nodes.len() <= right_nodes.len() {
            // If we can't determine, balance the sets
            left_nodes.push(*node_id);
        } else {
            right_nodes.push(*node_id);
        }
    }
    
    // Position nodes in two columns
    let left_count = left_nodes.len() as f32;
    let right_count = right_nodes.len() as f32;
    
    for (i, node_id) in left_nodes.iter().enumerate() {
        let y = (i as f32 - (left_count - 1.0) / 2.0) * 2.0;
        graph.node_positions.insert(*node_id, egui::Pos2::new(-4.0, y));
    }
    
    for (i, node_id) in right_nodes.iter().enumerate() {
        let y = (i as f32 - (right_count - 1.0) / 2.0) * 2.0;
        graph.node_positions.insert(*node_id, egui::Pos2::new(4.0, y));
    }
}

/// Apply a simple circular layout for unknown patterns
fn apply_circular_layout(graph: &mut AlchemistGraph) {
    let nodes: Vec<Uuid> = graph.nodes.keys().cloned().collect();
    let count = nodes.len() as f32;
    let radius = count.sqrt() * 1.5; // Scale radius based on node count
    
    for (i, node_id) in nodes.iter().enumerate() {
        let angle = (i as f32 / count) * std::f32::consts::TAU;
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        
        graph.node_positions.insert(*node_id, egui::Pos2::new(x, y));
    }
} 