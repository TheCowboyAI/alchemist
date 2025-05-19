use eframe::egui;
use rand::Rng;
use uuid::Uuid;

mod graph;
mod workflow_editor;
mod ecs;
mod events;
mod graph_patterns;
mod models;

use workflow_editor::WorkflowEditor;
use ecs::GraphSystem;
use events::{EventStream, GraphEvent, Command, CreateNodeCommand, CreateEdgeCommand, Model};
use graph::AlchemistGraph;
use graph_patterns::{GraphPattern, generate_pattern, PatternCatalog, PatternCategory};

// Graph settings structure to replace unsafe static variables
#[derive(Clone, Debug)]
struct GraphSettings {
    node_size: f32,
    node_color: egui::Color32,
    edge_thickness: f32,
    edge_color: egui::Color32,
    show_arrows: bool,
    show_node_labels: bool,
    show_node_properties: bool,
}

impl Default for GraphSettings {
    fn default() -> Self {
        Self {
            node_size: 20.0,
            node_color: egui::Color32::from_rgb(50, 150, 220),
            edge_thickness: 2.0,
            edge_color: egui::Color32::GRAY,
            show_arrows: true,
            show_node_labels: true,
            show_node_properties: true,
        }
    }
}

// Add a helper struct for 3D positions
#[derive(Clone, Debug, Copy)]
struct Position3D {
    x: f32,
    y: f32,
    z: f32,
}

impl Position3D {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    fn from_pos2(pos: egui::Pos2, z: f32) -> Self {
        Self { x: pos.x, y: pos.y, z }
    }
    
    fn to_pos2(&self) -> egui::Pos2 {
        egui::Pos2::new(self.x, self.y)
    }
    
    fn distance(&self, other: &Position3D) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
    
    fn direction_to(&self, other: &Position3D) -> (f32, f32, f32) {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dz = other.z - self.z;
        let length = self.distance(other);
        if length > 0.001 {
            (dx / length, dy / length, dz / length)
        } else {
            (0.0, 0.0, 0.0)
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("Alchemist - Information Graph Workflows"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Alchemist",
        options,
        Box::new(|cc| {
            // Set larger text styles
            let mut style = (*cc.egui_ctx.style()).clone();
            style.text_styles = [
                (egui::TextStyle::Heading, egui::FontId::proportional(32.0)),
                (egui::TextStyle::Body, egui::FontId::proportional(22.0)),
                (egui::TextStyle::Monospace, egui::FontId::monospace(20.0)),
                (egui::TextStyle::Button, egui::FontId::proportional(22.0)),
                (egui::TextStyle::Small, egui::FontId::proportional(18.0)),
            ].into();
            cc.egui_ctx.set_style(style);
            
            Ok(Box::new(AlchemistApp::new(cc)))
        }),
    )
}

struct AlchemistApp {
    name: String,
    age: u32,
    color: egui::Color32,
    show_extra_panel: bool,
    random_value: f64,
    current_view: ViewType,
    workflow_editor: WorkflowEditor,
    graph_system: GraphSystem,
    event_stream: EventStream,
    information_graph: AlchemistGraph,
    layout_type: LayoutType,
    label_filter: String,
    show_all_nodes: bool,
    show_all_edges: bool,
    applying_force_layout: bool,
    layout_iterations: usize,
    node_forces: std::collections::HashMap<Uuid, egui::Vec2>,
    graph_settings: GraphSettings,
    selected_pattern: PatternType,
    tree_branch_factor: usize,
    tree_depth: usize,
    star_points: usize,
    cycle_nodes: usize,
    complete_nodes: usize,
    grid_width: usize,
    grid_height: usize,
    random_nodes: usize,
    random_edge_probability: f32,
    polygon_sides: usize,
    bipartite_left: usize,
    bipartite_right: usize,
    bipartite_density: f32,
    dag_levels: usize,
    dag_nodes_per_level: usize,
    // New fields for automata
    automaton_states: usize,
    automaton_alphabet_size: usize,
    automaton_is_deterministic: bool,
    pattern_catalog: PatternCatalog,
    selected_catalog_pattern: String,
    selected_category: PatternCategory,
    // Physics parameters for force-directed layout
    physics_coulomb_constant: f32,
    physics_hooke_constant: f32,
    physics_damping: f32,
    physics_natural_length: f32,
    physics_max_iterations: usize,
    // Add view mode
    view_mode: ViewMode,
    // 3D specific parameters
    camera_rotation_x: f32,
    camera_rotation_y: f32,
    camera_distance: f32,
    // Add these fields after the existing camera parameters
    camera_pan_x: f32,
    camera_pan_y: f32,
}

#[derive(PartialEq, Clone, Copy)]
enum ViewType {
    Main,
    Workflow,
    Settings,
    Events,
}

#[derive(PartialEq, Clone, Copy)]
enum LayoutType {
    Circular,
    ForceDirected,
    Hierarchical,
}

#[derive(PartialEq, Clone, Copy)]
enum ViewMode {
    TwoDimensional,
    ThreeDimensional,
}

#[derive(PartialEq, Clone, Copy)]
enum PatternType {
    Tree,
    Star,
    Cycle,
    Complete,
    Grid,
    Random,
    RegularPolygon,
    // Replace StateMachine with the new automata types
    MooreMachine,
    MealyMachine,
    FiniteAutomaton,
    DirectedAcyclicGraph,
    Bipartite,
}

impl AlchemistApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize the graph system
        let mut graph_system = GraphSystem::new();
        let event_stream = EventStream::new();
        let information_graph = AlchemistGraph::new();
        
        // Create a workflow editor
        let workflow_editor = WorkflowEditor::new();
        
        // Import the workflow graph into the ECS system
        graph_system.import_graph(&workflow_editor.workflow.graph);
        
        Self {
            name: "Alchemist".to_string(),
            age: 42,
            color: egui::Color32::LIGHT_BLUE,
            show_extra_panel: false,
            random_value: rand::random(),
            current_view: ViewType::Main,
            workflow_editor,
            graph_system,
            event_stream,
            information_graph,
            layout_type: LayoutType::Circular,
            label_filter: String::new(),
            show_all_nodes: true,
            show_all_edges: true,
            applying_force_layout: false,
            layout_iterations: 0,
            node_forces: std::collections::HashMap::new(),
            graph_settings: GraphSettings::default(),
            selected_pattern: PatternType::Tree,
            tree_branch_factor: 2,
            tree_depth: 3,
            star_points: 8,
            cycle_nodes: 6,
            complete_nodes: 5,
            grid_width: 4,
            grid_height: 4,
            random_nodes: 10,
            random_edge_probability: 0.3,
            polygon_sides: 6,
            bipartite_left: 3,
            bipartite_right: 3,
            bipartite_density: 0.7,
            dag_levels: 3,
            dag_nodes_per_level: 2,
            // Initialize new fields
            automaton_states: 4,
            automaton_alphabet_size: 2,
            automaton_is_deterministic: true,
            pattern_catalog: PatternCatalog::new(),
            selected_catalog_pattern: "binary_tree".to_string(),
            selected_category: PatternCategory::Basic,
            // Physics parameters with defaults
            physics_coulomb_constant: 500000.0,
            physics_hooke_constant: 0.02,
            physics_damping: 0.9,
            physics_natural_length: 100.0,
            physics_max_iterations: 100,
            // Start in 3D mode by default
            view_mode: ViewMode::ThreeDimensional,
            // 3D specific parameters with optimal values for state machine visibility
            camera_rotation_x: 1.8, // Top-down view angle
            camera_rotation_y: -0.4, // Slight rotation
            camera_distance: 2000.0, // Increased distance for better overview
            // Initialize new fields
            camera_pan_x: 0.0,
            camera_pan_y: 0.0,
        }
    }
    
    // Helper method to execute a command and update the information graph from events
    fn execute_command(&mut self, command: &dyn Command) {
        // Execute the command to get events
        let events = command.execute();
        
        // Add all events to the event stream and apply to our models
        for event in events {
            if let Some(graph_event) = event.as_any().downcast_ref::<GraphEvent>() {
                // Apply the event to our graph model
                self.information_graph.apply_event(graph_event);
                
                // Apply the event to our ECS system
                self.graph_system.apply_event(graph_event);
                
                // Add to event stream
                self.event_stream.append(graph_event.clone());
            }
        }
    }
    
    // Add a method to apply force-directed layout
    fn apply_force_directed_layout(&mut self, node_positions: &mut std::collections::HashMap<Uuid, egui::Pos2>) {
        // Reset forces
        self.node_forces.clear();
        
        // Initialize forces for all nodes
        for id in self.information_graph.nodes.keys() {
            self.node_forces.insert(*id, egui::Vec2::ZERO);
        }
        
        // Physical constants (now from the struct)
        let coulomb_constant = self.physics_coulomb_constant;
        let hooke_constant = self.physics_hooke_constant;
        let damping = self.physics_damping;
        let min_distance = 20.0;  // Still hardcoded as it's a safety parameter
        let natural_length = self.physics_natural_length;
        let max_movement = 5.0;   // Still hardcoded for stability
        
        // Apply Coulomb's Law (repulsive forces between all nodes)
        let node_ids: Vec<Uuid> = self.information_graph.nodes.keys().cloned().collect();
        for i in 0..node_ids.len() {
            for j in (i+1)..node_ids.len() {
                let id1 = node_ids[i];
                let id2 = node_ids[j];
                
                if let (Some(pos1), Some(pos2)) = (node_positions.get(&id1), node_positions.get(&id2)) {
                    let delta = *pos2 - *pos1;
                    let distance = delta.length().max(min_distance);
                    
                    // Coulomb's Law: F = k * q1 * q2 / r^2
                    // Assuming all nodes have the same charge (q1 = q2 = 1)
                    let force_magnitude = coulomb_constant / (distance * distance);
                    let force = delta.normalized() * force_magnitude;
                    
                    // Check for fixed nodes
                    let node1_fixed = self.information_graph.nodes.get(&id1)
                        .map(|n| n.properties.contains_key("fixed_position"))
                        .unwrap_or(false);
                    
                    let node2_fixed = self.information_graph.nodes.get(&id2)
                        .map(|n| n.properties.contains_key("fixed_position"))
                        .unwrap_or(false);
                    
                    // Apply repulsive forces
                    if !node1_fixed {
                        if let Some(force1) = self.node_forces.get_mut(&id1) {
                            *force1 -= force;
                        }
                    }
                    
                    if !node2_fixed {
                        if let Some(force2) = self.node_forces.get_mut(&id2) {
                            *force2 += force;
                        }
                    }
                }
            }
        }
        
        // Apply Hooke's Law (attractive forces for connected nodes)
        for (_, edge) in &self.information_graph.edges {
            if let (Some(pos1), Some(pos2)) = (node_positions.get(&edge.source), node_positions.get(&edge.target)) {
                let delta = *pos2 - *pos1;
                let distance = delta.length().max(min_distance);
                
                // Calculate displacement from equilibrium
                let displacement = distance - natural_length;
                
                // Hooke's Law: F = -k * x
                // Where x is the displacement from equilibrium
                let force_magnitude = hooke_constant * displacement;
                let force = delta.normalized() * force_magnitude;
                
                // Check for fixed nodes
                let source_fixed = self.information_graph.nodes.get(&edge.source)
                    .map(|n| n.properties.contains_key("fixed_position"))
                    .unwrap_or(false);
                
                let target_fixed = self.information_graph.nodes.get(&edge.target)
                    .map(|n| n.properties.contains_key("fixed_position"))
                    .unwrap_or(false);
                
                // Apply attractive force
                if !source_fixed {
                    if let Some(force1) = self.node_forces.get_mut(&edge.source) {
                        *force1 += force;
                    }
                }
                
                if !target_fixed {
                    if let Some(force2) = self.node_forces.get_mut(&edge.target) {
                        *force2 -= force;
                    }
                }
            }
        }
        
        // Apply forces to update positions
        for (id, force) in &self.node_forces {
            // Skip fixed nodes
            let node_fixed = self.information_graph.nodes.get(id)
                .map(|n| n.properties.contains_key("fixed_position"))
                .unwrap_or(false);
                
            if !node_fixed {
                if let Some(pos) = node_positions.get_mut(id) {
                    // Apply force with damping, limiting maximum movement
                    let movement = force.normalized() * force.length().min(max_movement) * damping;
                    pos.x += movement.x;
                    pos.y += movement.y;
                    
                    // Optional: Add boundary constraints if needed
                    // pos.x = pos.x.clamp(some_min_x, some_max_x);
                    // pos.y = pos.y.clamp(some_min_y, some_max_y);
                }
            }
        }
        
        // Keep track of iterations
        self.layout_iterations += 1;
        if self.layout_iterations >= self.physics_max_iterations {
            self.applying_force_layout = false;
            self.layout_iterations = 0;
        }
    }
    
    // Add a method to apply 3D force-directed layout
    fn apply_3d_force_directed_layout(&mut self, node_positions_3d: &mut std::collections::HashMap<Uuid, Position3D>) {
        // Reset forces
        let mut node_forces_3d = std::collections::HashMap::new();
        
        // Initialize forces for all nodes
        for id in self.information_graph.nodes.keys() {
            node_forces_3d.insert(*id, (0.0, 0.0, 0.0));
        }
        
        // Physical constants (from struct)
        let coulomb_constant = self.physics_coulomb_constant;
        let hooke_constant = self.physics_hooke_constant;
        let damping = self.physics_damping;
        let min_distance = 20.0;
        let natural_length = self.physics_natural_length;
        let max_movement = 5.0;
        
        // Apply Coulomb's Law (repulsive forces) in 3D
        let node_ids: Vec<Uuid> = self.information_graph.nodes.keys().cloned().collect();
        for i in 0..node_ids.len() {
            for j in (i+1)..node_ids.len() {
                let id1 = node_ids[i];
                let id2 = node_ids[j];
                
                if let (Some(pos1), Some(pos2)) = (node_positions_3d.get(&id1), node_positions_3d.get(&id2)) {
                    let distance = pos1.distance(pos2);
                    let dist_squared = (distance * distance).max(min_distance * min_distance);
                    
                    // Calculate force magnitude (Coulomb's Law)
                    let force_magnitude = coulomb_constant / dist_squared;
                    
                    // Calculate direction vector
                    let (dir_x, dir_y, dir_z) = pos1.direction_to(pos2);
                    
                    // Calculate force components
                    let force_x = dir_x * force_magnitude;
                    let force_y = dir_y * force_magnitude;
                    let force_z = dir_z * force_magnitude;
                    
                    // Check for fixed nodes
                    let node1_fixed = self.information_graph.nodes.get(&id1)
                        .map(|n| n.properties.contains_key("fixed_position"))
                        .unwrap_or(false);
                    
                    let node2_fixed = self.information_graph.nodes.get(&id2)
                        .map(|n| n.properties.contains_key("fixed_position"))
                        .unwrap_or(false);
                    
                    // Apply repulsive forces
                    if !node1_fixed {
                        let (fx, fy, fz) = node_forces_3d.get(&id1).unwrap_or(&(0.0, 0.0, 0.0));
                        node_forces_3d.insert(id1, (fx - force_x, fy - force_y, fz - force_z));
                    }
                    
                    if !node2_fixed {
                        let (fx, fy, fz) = node_forces_3d.get(&id2).unwrap_or(&(0.0, 0.0, 0.0));
                        node_forces_3d.insert(id2, (fx + force_x, fy + force_y, fz + force_z));
                    }
                }
            }
        }
        
        // Apply Hooke's Law (attractive forces) in 3D for connected nodes
        for (_, edge) in &self.information_graph.edges {
            if let (Some(pos1), Some(pos2)) = (node_positions_3d.get(&edge.source), node_positions_3d.get(&edge.target)) {
                let distance = pos1.distance(pos2);
                
                // Calculate displacement from equilibrium
                let displacement = distance - natural_length;
                
                // Calculate force magnitude (Hooke's Law)
                let force_magnitude = hooke_constant * displacement;
                
                // Calculate direction vector
                let (dir_x, dir_y, dir_z) = pos1.direction_to(pos2);
                
                // Calculate force components
                let force_x = dir_x * force_magnitude;
                let force_y = dir_y * force_magnitude;
                let force_z = dir_z * force_magnitude;
                
                // Check for fixed nodes
                let source_fixed = self.information_graph.nodes.get(&edge.source)
                    .map(|n| n.properties.contains_key("fixed_position"))
                    .unwrap_or(false);
                
                let target_fixed = self.information_graph.nodes.get(&edge.target)
                    .map(|n| n.properties.contains_key("fixed_position"))
                    .unwrap_or(false);
                
                // Apply attractive forces
                if !source_fixed {
                    let (fx, fy, fz) = node_forces_3d.get(&edge.source).unwrap_or(&(0.0, 0.0, 0.0));
                    node_forces_3d.insert(edge.source, (fx + force_x, fy + force_y, fz + force_z));
                }
                
                if !target_fixed {
                    let (fx, fy, fz) = node_forces_3d.get(&edge.target).unwrap_or(&(0.0, 0.0, 0.0));
                    node_forces_3d.insert(edge.target, (fx - force_x, fy - force_y, fz - force_z));
                }
            }
        }
        
        // Apply forces to update positions
        for (id, (force_x, force_y, force_z)) in &node_forces_3d {
            // Skip fixed nodes
            let node_fixed = self.information_graph.nodes.get(id)
                .map(|n| n.properties.contains_key("fixed_position"))
                .unwrap_or(false);
                
            if !node_fixed {
                if let Some(pos) = node_positions_3d.get_mut(id) {
                    // Calculate force magnitude
                    let force_magnitude = (force_x * force_x + force_y * force_y + force_z * force_z).sqrt();
                    
                    // Apply damping and limit movement
                    let scale = (force_magnitude.min(max_movement) / force_magnitude.max(0.001)) * damping;
                    
                    // Update position
                    pos.x += force_x * scale;
                    pos.y += force_y * scale;
                    pos.z += force_z * scale;
                }
            }
        }
        
        // Keep track of iterations
        self.layout_iterations += 1;
        if self.layout_iterations >= self.physics_max_iterations {
            self.applying_force_layout = false;
            self.layout_iterations = 0;
        }
    }
    
    // Update the project_3d_to_2d method for better perspective projection
    fn project_3d_to_2d(&self, pos: &Position3D, rect: &egui::Rect) -> egui::Pos2 {
        // Center of the viewport
        let center_x = rect.center().x;
        let center_y = rect.center().y;
        
        // Apply camera panning to the position
        let adjusted_x = pos.x + self.camera_pan_x;
        let adjusted_y = pos.y + self.camera_pan_y;
        let adjusted_z = pos.z;
        
        // Apply rotations using proper transformation matrices
        let cos_x = self.camera_rotation_x.cos();
        let sin_x = self.camera_rotation_x.sin();
        let cos_y = self.camera_rotation_y.cos();
        let sin_y = self.camera_rotation_y.sin();
        
        // First rotate around X-axis (pitch)
        let y_rot = adjusted_y * cos_x - adjusted_z * sin_x;
        let z_rot_x = adjusted_y * sin_x + adjusted_z * cos_x;
        
        // Then rotate around Y-axis (yaw)
        let x_rot = adjusted_x * cos_y + z_rot_x * sin_y;
        let z_rot = -adjusted_x * sin_y + z_rot_x * cos_y;
        
        // Apply perspective projection with the camera distance factor
        // Make sure z is never behind the camera
        let z_distance = self.camera_distance + z_rot;
        let scale = self.camera_distance / z_distance.max(0.1);
        
        // Calculate projected coordinates
        let projected_x = center_x + x_rot * scale;
        let projected_y = center_y + y_rot * scale;
        
        egui::Pos2::new(projected_x, projected_y)
    }
    
    fn apply_hierarchical_layout(&mut self, node_positions: &mut std::collections::HashMap<Uuid, egui::Pos2>, rect: egui::Rect) {
        // Maps to track node levels and positions within levels
        let mut node_levels = std::collections::HashMap::new();
        let mut nodes_per_level = std::collections::HashMap::new();
        
        // First, assign levels to nodes (0 for nodes with no incoming edges, etc.)
        let mut nodes_with_no_incoming = Vec::new();
        let mut incoming_edges_count = std::collections::HashMap::new();
        
        // Initialize incoming edges count
        for id in self.information_graph.nodes.keys() {
            incoming_edges_count.insert(*id, 0);
        }
        
        // Count incoming edges
        for (_, edge) in &self.information_graph.edges {
            if let Some(count) = incoming_edges_count.get_mut(&edge.target) {
                *count += 1;
            }
        }
        
        // Find nodes with no incoming edges (level 0)
        for (id, count) in &incoming_edges_count {
            if *count == 0 {
                nodes_with_no_incoming.push(*id);
                node_levels.insert(*id, 0);
                
                // Initialize level 0 counter
                nodes_per_level.entry(0).or_insert(0);
                *nodes_per_level.get_mut(&0).unwrap() += 1;
            }
        }
        
        // Breadth-first traversal to assign levels
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        
        // Start with nodes having no incoming edges
        for id in nodes_with_no_incoming {
            queue.push_back(id);
            visited.insert(id);
        }
        
        // If no entry points found, start with any node
        if queue.is_empty() && !self.information_graph.nodes.is_empty() {
            let first_node = *self.information_graph.nodes.keys().next().unwrap();
            queue.push_back(first_node);
            visited.insert(first_node);
            node_levels.insert(first_node, 0);
            nodes_per_level.entry(0).or_insert(0);
            *nodes_per_level.get_mut(&0).unwrap() += 1;
        }
        
        // Process queue
        while let Some(node_id) = queue.pop_front() {
            let current_level = *node_levels.get(&node_id).unwrap();
            
            // Find all outgoing edges
            for (_, edge) in &self.information_graph.edges {
                if edge.source == node_id {
                    let target_id = edge.target;
                    
                    // If not visited yet, assign level and add to queue
                    if !visited.contains(&target_id) {
                        let next_level = current_level + 1;
                        node_levels.insert(target_id, next_level);
                        
                        // Update counter for this level
                        nodes_per_level.entry(next_level).or_insert(0);
                        *nodes_per_level.get_mut(&next_level).unwrap() += 1;
                        
                        queue.push_back(target_id);
                        visited.insert(target_id);
                    }
                }
            }
        }
        
        // Assign positions based on levels
        let mut positions_in_level = std::collections::HashMap::new();
        let level_height = rect.height() / (nodes_per_level.len().max(1) as f32 + 1.0);
        
        for (id, level) in &node_levels {
            // Initialize position counter for this level if not exists
            positions_in_level.entry(*level).or_insert(0);
            let position = *positions_in_level.get(level).unwrap();
            
            // Calculate horizontal position
            let level_width = rect.width() / (nodes_per_level.get(level).unwrap_or(&1) + 1) as f32;
            let x = rect.left() + level_width * (position + 1) as f32;
            
            // Calculate vertical position
            let y = rect.top() + level_height * (*level + 1) as f32;
            
            // Update position
            node_positions.insert(*id, egui::Pos2::new(x, y));
            
            // Increment position counter for this level
            *positions_in_level.get_mut(level).unwrap() += 1;
        }
        
        // Assign positions for any nodes not visited (isolated)
        let default_level = nodes_per_level.len();
        for id in self.information_graph.nodes.keys() {
            if !node_levels.contains_key(id) {
                // Place at bottom level
                let x = rect.left() + rect.width() / 2.0;
                let y = rect.top() + level_height * (default_level + 1) as f32;
                node_positions.insert(*id, egui::Pos2::new(x, y));
            }
        }
    }
    
    fn apply_circular_layout(&mut self, node_positions: &mut std::collections::HashMap<Uuid, egui::Pos2>, rect: egui::Rect) {
        let count = self.information_graph.nodes.len();
        if count == 0 {
            return;
        }
        
        let center_x = rect.center().x;
        let center_y = rect.center().y;
        let radius = (rect.width().min(rect.height()) * 0.4).max(100.0);
        
        let angle_step = 2.0 * std::f32::consts::PI / count as f32;
        
        for (i, (id, _)) in self.information_graph.nodes.iter().enumerate() {
            let angle = i as f32 * angle_step;
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            node_positions.insert(*id, egui::Pos2::new(x, y));
        }
    }
    
    // Completely rewrite the fit_camera_to_all_nodes method for better centering
    fn fit_camera_to_all_nodes(&mut self, node_positions_3d: &std::collections::HashMap<Uuid, Position3D>) {
        if node_positions_3d.is_empty() {
            return;
        }
        
        // Find the bounding box of all nodes
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut min_z = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut max_z = f32::MIN;
        
        for (_, pos) in node_positions_3d {
            min_x = min_x.min(pos.x);
            min_y = min_y.min(pos.y);
            min_z = min_z.min(pos.z);
            max_x = max_x.max(pos.x);
            max_y = max_y.max(pos.y);
            max_z = max_z.max(pos.z);
        }
        
        // Calculate the center of the bounding box
        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        let _center_z = (min_z + max_z) / 2.0; // Not used but calculated for completeness
        
        // Calculate the dimensions of the bounding box
        let width = (max_x - min_x).max(1.0);
        let height = (max_y - min_y).max(1.0);
        let depth = (max_z - min_z).max(1.0);
        
        // Calculate the dimension that needs the most space
        let max_dimension = width.max(height).max(depth);
        
        // Set camera distance based on the maximum dimension
        // Use a multiplier to ensure all nodes are visible with margin
        let distance_multiplier = 1.5;
        self.camera_distance = max_dimension * distance_multiplier;
        
        // Ensure distance is within reasonable bounds
        self.camera_distance = self.camera_distance.max(200.0).min(2000.0);
        
        // Reset camera panning to center
        self.camera_pan_x = -center_x;
        self.camera_pan_y = -center_y;
        
        // Set camera to a good default viewing angle
        self.camera_rotation_x = 0.3; // Slight tilt
        self.camera_rotation_y = 0.4; // Slight rotation
    }
    
    // Fix the unused variable warning in focus_on_node method
    fn focus_on_node(&mut self, node_id: Uuid, node_positions_3d: &std::collections::HashMap<Uuid, Position3D>) {
        if let Some(_pos) = node_positions_3d.get(&node_id) {
            // Reset camera rotation to a good viewing angle
            self.camera_rotation_x = 0.2;
            self.camera_rotation_y = 0.3;
            
            // Adjust camera distance to focus on the node (closer than the fit all view)
            self.camera_distance = 300.0;
            
            // Reset panning
            self.camera_pan_x = 0.0;
            self.camera_pan_y = 0.0;
        }
    }
    

    
    // Improve the redistribute_nodes_in_3d method for better spacing
    fn redistribute_nodes_in_3d(&mut self, node_positions: &std::collections::HashMap<Uuid, egui::Pos2>, node_positions_3d: &mut std::collections::HashMap<Uuid, Position3D>) {
        // First ensure all nodes have a 3D position
        for (id, pos_2d) in node_positions {
            if !node_positions_3d.contains_key(id) {
                // New node, create a 3D position
                node_positions_3d.insert(*id, Position3D::from_pos2(*pos_2d, 0.0));
            }
        }
        
        // Find connected components in the graph to create layers
        let mut visited = std::collections::HashSet::new();
        let mut component_assignments = std::collections::HashMap::new();
        let mut component_counter = 0;
        
        // Use DFS to find connected components
        for source_id in self.information_graph.nodes.keys() {
            if !visited.contains(source_id) {
                // Start a new component
                component_counter += 1;
                let mut stack = vec![*source_id];
                visited.insert(*source_id);
                component_assignments.insert(*source_id, component_counter);
                
                while let Some(node_id) = stack.pop() {
                    // Find all connected nodes
                    for (_, edge) in &self.information_graph.edges {
                        if edge.source == node_id && !visited.contains(&edge.target) {
                            stack.push(edge.target);
                            visited.insert(edge.target);
                            component_assignments.insert(edge.target, component_counter);
                        }
                        
                        if edge.target == node_id && !visited.contains(&edge.source) {
                            stack.push(edge.source);
                            visited.insert(edge.source);
                            component_assignments.insert(edge.source, component_counter);
                        }
                    }
                }
            }
        }
        
        // Determine graph center and size
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        
        for (_, pos) in node_positions {
            min_x = min_x.min(pos.x);
            max_x = max_x.max(pos.x);
            min_y = min_y.min(pos.y);
            max_y = max_y.max(pos.y);
            sum_x += pos.x;
            sum_y += pos.y;
        }
        
        // Calculate center and size
        let center_x = if node_positions.is_empty() { 0.0 } else { sum_x / node_positions.len() as f32 };
        let center_y = if node_positions.is_empty() { 0.0 } else { sum_y / node_positions.len() as f32 };
        let graph_width = (max_x - min_x).max(1.0);
        let graph_height = (max_y - min_y).max(1.0);
        let graph_size = graph_width.max(graph_height);
        
        // Use random generator for consistent offsets
        let mut rng = rand::rng();
        
        // For each node, adjust its 3D position
        for (id, _) in &self.information_graph.nodes {
            if let Some(pos_2d) = node_positions.get(id) {
                if let Some(pos_3d) = node_positions_3d.get_mut(id) {
                    // Normalize position relative to center (creates better spread)
                    let normalized_x = (pos_2d.x - center_x) / graph_size * 3.0; // Triple the normalization scale
                    let normalized_y = (pos_2d.y - center_y) / graph_size * 3.0;
                    
                    // Base X and Y on 2D layout with increased spacing
                    pos_3d.x = normalized_x * 400.0; // Double the spacing from 200 to 400
                    pos_3d.y = normalized_y * 400.0;
                    
                    // Component separation - give each component a different Z-depth
                    let component = component_assignments.get(id).unwrap_or(&1);
                    let component_z_offset = (*component as f32 - 1.0) * 250.0; // Increase vertical separation from 150 to 250
                    
                    // Count connections for node
                    let mut connection_count = 0;
                    for (_, edge) in &self.information_graph.edges {
                        if edge.source == *id || edge.target == *id {
                            connection_count += 1;
                        }
                    }
                    
                    // Nodes with more connections get pushed forward
                    let connectivity_z_offset = connection_count as f32 * 30.0; // Double from 15 to 30
                    
                    // Add randomness for visual separation and to prevent overlaps
                    let random_offset = rng.random_range(-50.0..50.0); // Increase from -30..30 to -50..50
                    
                    // Combine all Z factors
                    pos_3d.z = component_z_offset + connectivity_z_offset + random_offset;
                }
            }
        }
        
        // Additional pass to separate nodes based on label/type
        let mut node_types = std::collections::HashMap::new();
        
        // Group nodes by labels
        for (id, node) in &self.information_graph.nodes {
            if !node.labels.is_empty() {
                let primary_label = &node.labels[0];
                node_types.entry(primary_label.clone())
                    .or_insert_with(Vec::new)
                    .push(*id);
            }
        }
        
        // Adjust nodes of the same type to have similar Z values with extra offset
        for (label, nodes) in node_types {
            // Assign different z_offset based on node type
            let z_offset = match label.as_str() {
                "test" => 100.0,    // Doubled from 50 to 100
                "source" => -200.0, // Doubled from -100 to -200
                "target" => 200.0,  // Doubled from 100 to 200
                "process" => 0.0,
                "state" => 150.0,   // Doubled from 70 to 150
                "initial" => -100.0,// Doubled from -50 to -100
                "terminal" => 200.0,// Doubled from 100 to 200
                _ => 0.0,
            };
            
            for id in nodes {
                if let Some(pos_3d) = node_positions_3d.get_mut(&id) {
                    pos_3d.z += z_offset;
                }
            }
        }
    }

    // Add a new method to set optimal initial camera parameters
    fn initialize_optimal_camera_settings(&mut self, node_positions_3d: &std::collections::HashMap<Uuid, Position3D>) {
        // Set default camera angles for a good initial view
        self.camera_rotation_x = 1.8;   // Top-down view angle
        self.camera_rotation_y = -0.4;  // Slight rotation
        
        // Reset panning
        self.camera_pan_x = 0.0;
        self.camera_pan_y = 0.0;
        
        // Set a reasonable default distance
        self.camera_distance = 2000.0;  // Increased distance for better overview
        
        // If we have nodes, optimize the view to see them all
        if !node_positions_3d.is_empty() {
            self.fit_camera_to_all_nodes(node_positions_3d);
        }
    }
}

impl eframe::App for AlchemistApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Add initialization code to setup 3D view on first run
        // This will run only once when the app starts
        if ctx.input(|i| i.time <= 0.2) && self.view_mode == ViewMode::ThreeDimensional && self.current_view == ViewType::Main {
            // On first run, set up 3D positions and camera
            let node_positions_id = egui::Id::new("node_positions");
            let node_positions_3d_id = egui::Id::new("node_positions_3d");
            
            // Get 2D node positions if they exist
            let node_positions: std::collections::HashMap<Uuid, egui::Pos2> = 
                ctx.memory_mut(|mem| mem.data.get_persisted(node_positions_id))
                .unwrap_or_default();
                
            // Create or get 3D node positions
            let mut node_positions_3d: std::collections::HashMap<Uuid, Position3D> = 
                if self.view_mode == ViewMode::ThreeDimensional {
                    ctx.memory_mut(|mem| mem.data.get_persisted(node_positions_3d_id))
                    .unwrap_or_default()
                } else {
                    std::collections::HashMap::new()
                };
                
            // Make sure all nodes have 3D positions
            if !self.information_graph.nodes.is_empty() {
                self.redistribute_nodes_in_3d(&node_positions, &mut node_positions_3d);
                self.initialize_optimal_camera_settings(&node_positions_3d);
                
                // Store the 3D positions
                ctx.memory_mut(|mem| mem.data.insert_persisted(node_positions_3d_id, node_positions_3d));
                
                // Request a repaint to show the updated view
                ctx.request_repaint();
            }
        }
        
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Alchemist - Information Graph Workflows");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Add 2D/3D toggle only for the main view
                    if self.current_view == ViewType::Main {
                        let view_mode_text = match self.view_mode {
                            ViewMode::TwoDimensional => "Switch to 3D",
                            ViewMode::ThreeDimensional => "Switch to 2D",
                        };
                        
                        if ui.button(view_mode_text).clicked() {
                            let new_mode = match self.view_mode {
                                ViewMode::TwoDimensional => ViewMode::ThreeDimensional,
                                ViewMode::ThreeDimensional => ViewMode::TwoDimensional,
                            };
                            
                            if new_mode == ViewMode::ThreeDimensional && self.view_mode == ViewMode::TwoDimensional {
                                // Switching from 2D to 3D - redistribute nodes
                                let node_positions_id = egui::Id::new("node_positions");
                                let node_positions_3d_id = egui::Id::new("node_positions_3d");
                                
                                let node_positions: std::collections::HashMap<Uuid, egui::Pos2> = 
                                    ctx.memory_mut(|mem| mem.data.get_persisted(node_positions_id))
                                    .unwrap_or_default();
                                    
                                let mut node_positions_3d: std::collections::HashMap<Uuid, Position3D> = 
                                    ctx.memory_mut(|mem| mem.data.get_persisted(node_positions_3d_id))
                                    .unwrap_or_default();
                                    
                                self.redistribute_nodes_in_3d(&node_positions, &mut node_positions_3d);
                                
                                // Initialize optimal camera settings for the 3D view
                                self.initialize_optimal_camera_settings(&node_positions_3d);
                                
                                // Store the updated 3D positions
                                ctx.memory_mut(|mem| mem.data.insert_persisted(node_positions_3d_id, node_positions_3d));
                            }
                            
                            self.view_mode = new_mode;
                        }
                    }
                    
                    if ui.button("Generate Random").clicked() {
                        self.random_value = rand::rng().random_range(0.0..1.0);
                    }
                    
                    if ui.button("Toggle Extra Panel").clicked() {
                        self.show_extra_panel = !self.show_extra_panel;
                    }
                });
            });
            
            ui.horizontal(|ui| {
                if ui.selectable_label(self.current_view == ViewType::Main, "Main View").clicked() {
                    self.current_view = ViewType::Main;
                }
                if ui.selectable_label(self.current_view == ViewType::Workflow, "Workflow Editor").clicked() {
                    self.current_view = ViewType::Workflow;
                }
                if ui.selectable_label(self.current_view == ViewType::Settings, "Settings").clicked() {
                    self.current_view = ViewType::Settings;
                }
                if ui.selectable_label(self.current_view == ViewType::Events, "Events").clicked() {
                    self.current_view = ViewType::Events;
                }
            });
        });
        
        match self.current_view {
            ViewType::Main => self.show_main_view(ctx),
            ViewType::Workflow => self.show_workflow_view(ctx),
            ViewType::Settings => self.show_settings_view(ctx),
            ViewType::Events => self.show_events_view(ctx),
        }
        
        if self.show_extra_panel {
            egui::Window::new("Detail")
                .default_pos(egui::pos2(300.0, 300.0))
                .min_width(350.0)
                .show(ctx, |ui| {
                    let selected_node_id = egui::Id::new("selected_node");
                    let selected_node: Option<Uuid> = 
                        ctx.memory_mut(|mem| mem.data.get_persisted(selected_node_id))
                        .unwrap_or_default();
                        
                    if let Some(node_id) = selected_node {
                        if let Some(node) = self.information_graph.nodes.get(&node_id) {
                            ui.heading(&node.name);
                            ui.label(format!("ID: {}", node_id));
                            
                            ui.add_space(10.0);
                            ui.separator();
                            
                            ui.strong("Labels:");
                            if node.labels.is_empty() {
                                ui.label("None");
                            } else {
                                for label in &node.labels {
                                    ui.label(format!("• {}", label));
                                }
                            }
                            
                            ui.add_space(5.0);
                            ui.separator();
                            
                            ui.strong("Properties:");
                            if node.properties.is_empty() {
                                ui.label("None");
                            } else {
                                // Use a ScrollArea for properties in case there are many
                                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                                    for (key, value) in &node.properties {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("• {}:", key));
                                            ui.label(value);
                                        });
                                    }
                                });
                            }
                            
                            ui.add_space(5.0);
                            ui.separator();
                            
                            // Count and show connected edges
                            let mut incoming_edges = 0;
                            let mut outgoing_edges = 0;
                            
                            for (_, edge) in &self.information_graph.edges {
                                if edge.source == node_id {
                                    outgoing_edges += 1;
                                }
                                if edge.target == node_id {
                                    incoming_edges += 1;
                                }
                            }
                            
                            ui.strong("Connectivity:");
                            ui.label(format!("• Incoming Edges: {}", incoming_edges));
                            ui.label(format!("• Outgoing Edges: {}", outgoing_edges));
                            ui.label(format!("• Total Connections: {}", incoming_edges + outgoing_edges));
                            
                            if self.view_mode == ViewMode::ThreeDimensional {
                                // Show 3D position if in 3D mode
                                let node_positions_3d_id = egui::Id::new("node_positions_3d");
                                if let Some(positions_3d) = ctx.memory_mut(|mem| mem.data.get_persisted::<std::collections::HashMap<Uuid, Position3D>>(node_positions_3d_id)) {
                                    if let Some(pos_3d) = positions_3d.get(&node_id) {
                                        ui.add_space(5.0);
                                        ui.separator();
                                        ui.strong("3D Position:");
                                        ui.label(format!("X: {:.2}, Y: {:.2}, Z: {:.2}", pos_3d.x, pos_3d.y, pos_3d.z));
                                    }
                                }
                            }
                        } else {
                            ui.heading("Node Not Found");
                            ui.label("The selected node no longer exists in the graph.");
                        }
                    } else {
                        ui.heading("No Node Selected");
                        ui.label("Select a node in the graph to view its details.");
                    }
                    
                    ui.add_space(10.0);
                    ui.separator();
                    
                    if ui.button("Close").clicked() {
                        self.show_extra_panel = false;
                    }
                });
        }
    }
}

impl AlchemistApp {
    fn show_main_view(&mut self, ctx: &egui::Context) {
        // Define IDs for persistent storage
        let node_positions_id = egui::Id::new("node_positions");
        let node_positions_3d_id = egui::Id::new("node_positions_3d");
        let selected_node_id = egui::Id::new("selected_node");
        
        // Get persistent node positions and selection
        let mut node_positions: std::collections::HashMap<Uuid, egui::Pos2> = 
            ctx.memory_mut(|mem| mem.data.get_persisted(node_positions_id))
            .unwrap_or_default();
            
        let mut node_positions_3d: std::collections::HashMap<Uuid, Position3D> = 
            if self.view_mode == ViewMode::ThreeDimensional {
                ctx.memory_mut(|mem| mem.data.get_persisted(node_positions_3d_id))
                .unwrap_or_default()
            } else {
                std::collections::HashMap::new()
            };
            
        let mut selected_node: Option<Uuid> = 
            ctx.memory_mut(|mem| mem.data.get_persisted(selected_node_id))
            .unwrap_or_default();
        
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Graph Settings");
            ui.add(egui::Separator::default());
            ui.add_space(10.0);
            
            // Node Appearance Settings
            ui.collapsing("Node Appearance", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Node Size:");
                    let mut size = self.graph_settings.node_size;
                    if ui.add(egui::Slider::new(&mut size, 10.0..=50.0)).changed() {
                        self.graph_settings.node_size = size;
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Node Color:");
                    let mut color = self.graph_settings.node_color;
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        self.graph_settings.node_color = color;
                    }
                });
                
                ui.checkbox(&mut self.graph_settings.show_node_labels, "Show Node Labels");
                ui.checkbox(&mut self.graph_settings.show_node_properties, "Show Node Properties");
            });
            
            // Edge Appearance Settings
            ui.collapsing("Edge Appearance", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Edge Thickness:");
                    let mut thickness = self.graph_settings.edge_thickness;
                    if ui.add(egui::Slider::new(&mut thickness, 1.0..=10.0)).changed() {
                        self.graph_settings.edge_thickness = thickness;
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Edge Color:");
                    let mut color = self.graph_settings.edge_color;
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        self.graph_settings.edge_color = color;
                    }
                });
                
                let mut show_arrows = self.graph_settings.show_arrows;
                if ui.checkbox(&mut show_arrows, "Show Arrows").changed() {
                    self.graph_settings.show_arrows = show_arrows;
                }
            });
            
            // Layout Settings
            ui.collapsing("Layout", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Layout Type:");
                    ui.selectable_value(&mut self.layout_type, LayoutType::Circular, "Circular");
                    ui.selectable_value(&mut self.layout_type, LayoutType::ForceDirected, "Force Directed");
                    ui.selectable_value(&mut self.layout_type, LayoutType::Hierarchical, "Hierarchical");
                });
                
                // Physics parameters for force-directed layout
                if self.layout_type == LayoutType::ForceDirected {
                    ui.add_space(10.0);
                    ui.label("Physics Parameters:");
                    
                    ui.horizontal(|ui| {
                        ui.label("Repulsion Strength:");
                        ui.add(egui::Slider::new(&mut self.physics_coulomb_constant, 100000.0..=1000000.0).logarithmic(true));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Spring Strength:");
                        ui.add(egui::Slider::new(&mut self.physics_hooke_constant, 0.001..=0.1).logarithmic(true));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Natural Edge Length:");
                        ui.add(egui::Slider::new(&mut self.physics_natural_length, 50.0..=300.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Damping:");
                        ui.add(egui::Slider::new(&mut self.physics_damping, 0.5..=0.99));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Max Iterations:");
                        ui.add(egui::Slider::new(&mut self.physics_max_iterations, 10..=500));
                    });
                    
                    if ui.button("Reset Physics Parameters").clicked() {
                        self.physics_coulomb_constant = 500000.0;
                        self.physics_hooke_constant = 0.02;
                        self.physics_damping = 0.9;
                        self.physics_natural_length = 100.0;
                        self.physics_max_iterations = 100;
                    }
                }
                
                if ui.button("Apply Layout").clicked() {
                    match self.layout_type {
                        LayoutType::Circular => {
                            // Clear positions to trigger recalculation with circular layout
                            ctx.memory_mut(|mem| mem.data.remove::<std::collections::HashMap<Uuid, egui::Pos2>>(node_positions_id));
                        },
                        LayoutType::ForceDirected => {
                            // Start force-directed layout
                            self.applying_force_layout = true;
                            self.layout_iterations = 0;
                        },
                        LayoutType::Hierarchical => {
                            // Clear positions to trigger recalculation with hierarchical layout
                            ctx.memory_mut(|mem| mem.data.remove::<std::collections::HashMap<Uuid, egui::Pos2>>(node_positions_id));
                            // The hierarchical layout will be applied in the drawing code
                            self.layout_type = LayoutType::Hierarchical;
                        }
                    }
                }
                
                if ui.button("Reset Layout").clicked() {
                    // Clear node positions to trigger recalculation
                    ctx.memory_mut(|mem| mem.data.remove::<std::collections::HashMap<Uuid, egui::Pos2>>(node_positions_id));
                }
            });
            
            // Filter Settings
            ui.collapsing("Filters", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Filter by Label:");
                    ui.text_edit_singleline(&mut self.label_filter);
                });
                
                ui.checkbox(&mut self.show_all_nodes, "Show All Nodes");
                ui.checkbox(&mut self.show_all_edges, "Show All Edges");
                
                if ui.button("Apply Filters").clicked() {
                    // Logic to apply filters
                }
            });
            
            // Show entity counts from the ECS system
            ui.add_space(10.0);
            ui.separator();
            ui.heading("System Status");
            ui.label(format!("ECS Entities: {}", self.graph_system.registry.entity_count()));
            ui.label(format!("Event Count: {}", self.event_stream.len()));
            
            // Add a new section for graph patterns
            ui.collapsing("Graph Patterns", |ui| {
                // Tab view for custom patterns vs catalog
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.selected_pattern, PatternType::Tree, "Tree");
                    ui.selectable_value(&mut self.selected_pattern, PatternType::Star, "Star");
                    ui.selectable_value(&mut self.selected_pattern, PatternType::Cycle, "Cycle");
                });
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.selected_pattern, PatternType::Complete, "Complete");
                    ui.selectable_value(&mut self.selected_pattern, PatternType::Grid, "Grid");
                    ui.selectable_value(&mut self.selected_pattern, PatternType::Random, "Random");
                });
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.selected_pattern, PatternType::RegularPolygon, "Polygon");
                    ui.selectable_value(&mut self.selected_pattern, PatternType::MooreMachine, "Moore Machine");
                });
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.selected_pattern, PatternType::MealyMachine, "Mealy Machine");
                    ui.selectable_value(&mut self.selected_pattern, PatternType::FiniteAutomaton, "Finite Automaton");
                });
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.selected_pattern, PatternType::DirectedAcyclicGraph, "DAG");
                    ui.selectable_value(&mut self.selected_pattern, PatternType::Bipartite, "Bipartite");
                });
                
                // Show pattern-specific parameters
                match self.selected_pattern {
                    PatternType::Tree => {
                        ui.horizontal(|ui| {
                            ui.label("Branch Factor:");
                            ui.add(egui::Slider::new(&mut self.tree_branch_factor, 1..=5));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Depth:");
                            ui.add(egui::Slider::new(&mut self.tree_depth, 1..=5));
                        });
                    },
                    PatternType::Star => {
                        ui.horizontal(|ui| {
                            ui.label("Points:");
                            ui.add(egui::Slider::new(&mut self.star_points, 3..=20));
                        });
                    },
                    PatternType::Cycle => {
                        ui.horizontal(|ui| {
                            ui.label("Nodes:");
                            ui.add(egui::Slider::new(&mut self.cycle_nodes, 3..=20));
                        });
                    },
                    PatternType::Complete => {
                        ui.horizontal(|ui| {
                            ui.label("Nodes:");
                            ui.add(egui::Slider::new(&mut self.complete_nodes, 3..=10));
                            ui.label("(caution: generates many edges)");
                        });
                    },
                    PatternType::Grid => {
                        ui.horizontal(|ui| {
                            ui.label("Width:");
                            ui.add(egui::Slider::new(&mut self.grid_width, 2..=10));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Height:");
                            ui.add(egui::Slider::new(&mut self.grid_height, 2..=10));
                        });
                    },
                    PatternType::Random => {
                        ui.horizontal(|ui| {
                            ui.label("Nodes:");
                            ui.add(egui::Slider::new(&mut self.random_nodes, 3..=20));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Edge Probability:");
                            ui.add(egui::Slider::new(&mut self.random_edge_probability, 0.1..=0.9));
                        });
                    },
                    PatternType::RegularPolygon => {
                        ui.horizontal(|ui| {
                            ui.label("Sides:");
                            ui.add(egui::Slider::new(&mut self.polygon_sides, 3..=12));
                        });
                    },
                    PatternType::MooreMachine => {
                        ui.label("Creates a Moore machine with outputs on states.");
                    },
                    PatternType::MealyMachine => {
                        ui.label("Creates a Mealy machine with outputs on transitions.");
                    },
                    PatternType::FiniteAutomaton => {
                        ui.horizontal(|ui| {
                            ui.label("States:");
                            ui.add(egui::Slider::new(&mut self.automaton_states, 2..=10));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Alphabet Size:");
                            ui.add(egui::Slider::new(&mut self.automaton_alphabet_size, 1..=5));
                        });
                        ui.checkbox(&mut self.automaton_is_deterministic, "Deterministic (DFA)");
                        if !self.automaton_is_deterministic {
                            ui.label("Non-deterministic (NFA) - may have multiple transitions per input");
                        }
                    },
                    PatternType::DirectedAcyclicGraph => {
                        ui.horizontal(|ui| {
                            ui.label("Levels:");
                            ui.add(egui::Slider::new(&mut self.dag_levels, 2..=5));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Nodes per Level:");
                            ui.add(egui::Slider::new(&mut self.dag_nodes_per_level, 1..=5));
                        });
                    },
                    PatternType::Bipartite => {
                        ui.horizontal(|ui| {
                            ui.label("Left Set Nodes:");
                            ui.add(egui::Slider::new(&mut self.bipartite_left, 1..=8));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Right Set Nodes:");
                            ui.add(egui::Slider::new(&mut self.bipartite_right, 1..=8));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Edge Density:");
                            ui.add(egui::Slider::new(&mut self.bipartite_density, 0.1..=1.0));
                        });
                    },
                }
                
                if ui.button("Generate Custom Pattern").clicked() {
                    // Create the selected pattern
                    let pattern = match self.selected_pattern {
                        PatternType::Tree => GraphPattern::Tree { 
                            branch_factor: self.tree_branch_factor, 
                            depth: self.tree_depth 
                        },
                        PatternType::Star => GraphPattern::Star { 
                            points: self.star_points 
                        },
                        PatternType::Cycle => GraphPattern::Cycle { 
                            nodes: self.cycle_nodes 
                        },
                        PatternType::Complete => GraphPattern::Complete { 
                            nodes: self.complete_nodes 
                        },
                        PatternType::Grid => GraphPattern::Grid { 
                            width: self.grid_width, 
                            height: self.grid_height 
                        },
                        PatternType::Random => GraphPattern::Random { 
                            nodes: self.random_nodes, 
                            edge_probability: self.random_edge_probability 
                        },
                        PatternType::RegularPolygon => GraphPattern::RegularPolygon {
                            sides: self.polygon_sides
                        },
                        PatternType::MooreMachine => GraphPattern::MooreMachine,
                        PatternType::MealyMachine => GraphPattern::MealyMachine,
                        PatternType::FiniteAutomaton => GraphPattern::FiniteAutomaton {
                            states: self.automaton_states,
                            alphabet_size: self.automaton_alphabet_size,
                            is_deterministic: self.automaton_is_deterministic
                        },
                        PatternType::DirectedAcyclicGraph => GraphPattern::DirectedAcyclicGraph {
                            levels: self.dag_levels,
                            nodes_per_level: self.dag_nodes_per_level
                        },
                        PatternType::Bipartite => GraphPattern::Bipartite {
                            left_nodes: self.bipartite_left,
                            right_nodes: self.bipartite_right,
                            edge_density: self.bipartite_density
                        },
                    };
                    
                    // Generate the pattern and replace the current graph
                    self.information_graph = generate_pattern(pattern);
                    
                    // Clear the node positions to trigger layout recalculation
                    ctx.memory_mut(|mem| mem.data.remove::<std::collections::HashMap<Uuid, egui::Pos2>>(egui::Id::new("node_positions")));
                    
                    // Apply the selected layout
                    match self.layout_type {
                        LayoutType::ForceDirected => {
                            // Start force-directed layout
                            self.applying_force_layout = true;
                            self.layout_iterations = 0;
                        },
                        _ => {} // Other layouts will be applied automatically
                    }
                }
            });
            
            // Add a Pattern Catalog section
            ui.collapsing("Pattern Catalog", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Category:");
                    
                    if ui.selectable_label(self.selected_category == PatternCategory::Basic, "Basic").clicked() {
                        self.selected_category = PatternCategory::Basic;
                    }
                    if ui.selectable_label(self.selected_category == PatternCategory::Structural, "Structural").clicked() {
                        self.selected_category = PatternCategory::Structural;
                    }
                });
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.selected_category == PatternCategory::Algorithmic, "Algorithmic").clicked() {
                        self.selected_category = PatternCategory::Algorithmic;
                    }
                    if ui.selectable_label(self.selected_category == PatternCategory::Modeling, "Modeling").clicked() {
                        self.selected_category = PatternCategory::Modeling;
                    }
                });
                
                ui.separator();
                
                // Display patterns in the selected category
                let patterns_in_category = self.pattern_catalog.get_keys_by_category(self.selected_category);
                for &pattern_key in &patterns_in_category {
                    let pattern = self.pattern_catalog.get_pattern(pattern_key).unwrap();
                    if ui.selectable_label(self.selected_catalog_pattern == pattern_key, pattern.name()).clicked() {
                        self.selected_catalog_pattern = pattern_key.to_string();
                    }
                }
                
                // Show pattern description if available
                if let Some(pattern) = self.pattern_catalog.get_pattern(&self.selected_catalog_pattern) {
                    ui.label(pattern.description());
                }
                
                if ui.button("Generate from Catalog").clicked() {
                    if let Some(pattern) = self.pattern_catalog.get_pattern(&self.selected_catalog_pattern) {
                        // Generate the pattern and replace the current graph
                        self.information_graph = generate_pattern(pattern.clone());
                        
                        // Clear the node positions to trigger layout recalculation
                        ctx.memory_mut(|mem| mem.data.remove::<std::collections::HashMap<Uuid, egui::Pos2>>(egui::Id::new("node_positions")));
                        
                        // Apply the selected layout
                        match self.layout_type {
                            LayoutType::ForceDirected => {
                                // Start force-directed layout
                                self.applying_force_layout = true;
                                self.layout_iterations = 0;
                            },
                            _ => {} // Other layouts will be applied automatically
                        }
                    }
                }
            });
            
            // Add 3D camera controls if in 3D mode
            if self.view_mode == ViewMode::ThreeDimensional && self.current_view == ViewType::Main {
                ui.collapsing("3D Camera Controls", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("X Rotation:");
                        ui.add(egui::Slider::new(&mut self.camera_rotation_x, -std::f32::consts::PI..=std::f32::consts::PI).step_by(0.01));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Y Rotation:");
                        ui.add(egui::Slider::new(&mut self.camera_rotation_y, -std::f32::consts::PI..=std::f32::consts::PI).step_by(0.01));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Camera Distance:");
                        ui.add(egui::Slider::new(&mut self.camera_distance, 100.0..=2000.0).step_by(10.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Pan X:");
                        ui.add(egui::Slider::new(&mut self.camera_pan_x, -2000.0..=2000.0).step_by(10.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Pan Y:");
                        ui.add(egui::Slider::new(&mut self.camera_pan_y, -2000.0..=2000.0).step_by(10.0));
                    });
                    
                    ui.horizontal(|ui| {
                        if ui.button("Reset Camera").clicked() {
                            self.camera_rotation_x = 0.0;
                            self.camera_rotation_y = 0.0;
                            self.camera_distance = 500.0;
                            self.camera_pan_x = 0.0;
                            self.camera_pan_y = 0.0;
                        }
                        
                        if ui.button("Fit to View").clicked() {
                            let node_positions_3d_id = egui::Id::new("node_positions_3d");
                            let node_positions_3d: std::collections::HashMap<Uuid, Position3D> = 
                                ctx.memory_mut(|mem| mem.data.get_persisted(node_positions_3d_id))
                                .unwrap_or_default();
                                
                            self.fit_camera_to_all_nodes(&node_positions_3d);
                        }
                        
                        // Add focus on selected node button
                        if let Some(node_id) = selected_node {
                            if ui.button("Focus Selected").clicked() {
                                let node_positions_3d_id = egui::Id::new("node_positions_3d");
                                let node_positions_3d: std::collections::HashMap<Uuid, Position3D> = 
                                    ctx.memory_mut(|mem| mem.data.get_persisted(node_positions_3d_id))
                                    .unwrap_or_default();
                                    
                                self.focus_on_node(node_id, &node_positions_3d);
                            }
                        }
                    });
                    
                    // Add a new row for the redistribute button
                    ui.horizontal(|ui| {
                        if ui.button("Redistribute in 3D").clicked() {
                            let node_positions_id = egui::Id::new("node_positions");
                            let node_positions_3d_id = egui::Id::new("node_positions_3d");
                            
                            let node_positions: std::collections::HashMap<Uuid, egui::Pos2> = 
                                ctx.memory_mut(|mem| mem.data.get_persisted(node_positions_id))
                                .unwrap_or_default();
                                
                            let mut node_positions_3d: std::collections::HashMap<Uuid, Position3D> = 
                                ctx.memory_mut(|mem| mem.data.get_persisted(node_positions_3d_id))
                                .unwrap_or_default();
                                
                            self.redistribute_nodes_in_3d(&node_positions, &mut node_positions_3d);
                            
                            // Store the updated 3D positions
                            ctx.memory_mut(|mem| mem.data.insert_persisted(node_positions_3d_id, node_positions_3d));
                        }
                    });
                });
            }
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Information Graph Visualization");
            
            // Graph control toolbar
            ui.horizontal(|ui| {
                if ui.button("Generate Test Nodes").clicked() {
                    // Example of using the Command pattern to create nodes
                    let cmd = CreateNodeCommand {
                        name: format!("Test Node {}", rand::rng().random_range(1..100)),
                        labels: vec!["test".to_string(), "node".to_string()],
                    };
                    
                    // Execute the command using our helper method
                    self.execute_command(&cmd);
                    
                    // Create a random connection between nodes if we have at least 2 nodes
                    let nodes: Vec<_> = self.information_graph.nodes.keys().cloned().collect();
                    if nodes.len() >= 2 {
                        let mut rng = rand::rng();
                        let source_idx = rng.random_range(0..nodes.len());
                        let mut target_idx = rng.random_range(0..nodes.len());
                        
                        // Make sure source and target are different
                        while target_idx == source_idx {
                            target_idx = rng.random_range(0..nodes.len());
                        }
                        
                        let edge_cmd = CreateEdgeCommand {
                            source: nodes[source_idx],
                            target: nodes[target_idx],
                            labels: vec!["connection".to_string()],
                        };
                        
                        self.execute_command(&edge_cmd);
                    }
                }
                
                if ui.button("Update Graph System").clicked() {
                    // Run the position update system
                    self.graph_system.update_positions();
                }
                
                if ui.button("Synchronize Systems").clicked() {
                    // Export the ECS data to a graph, which will replace our information graph
                    self.information_graph = self.graph_system.export_to_graph();
                }
                
                if ui.button("Reset Layout").clicked() {
                    // Clear node positions to trigger recalculation
                    ctx.memory_mut(|mem| mem.data.remove::<std::collections::HashMap<Uuid, egui::Pos2>>(node_positions_id));
                }
                
                ui.separator();
                
                if ui.button("Clear Graph").clicked() {
                    // Clear the entire graph
                    self.information_graph = AlchemistGraph::new();
                }
                
                if ui.button("Export Graph").clicked() {
                    // Placeholder for export functionality
                    // Would save the graph to a file in a real implementation
                }
                
                if ui.button("Import Graph").clicked() {
                    // Placeholder for import functionality
                    // Would load a graph from a file in a real implementation
                }
            });
            
            ui.add_space(5.0);
            
            // Display information about our graph
            ui.horizontal(|ui| {
                ui.strong(format!("Nodes: {}", self.information_graph.nodes.len()));
                ui.strong(format!("Edges: {}", self.information_graph.edges.len()));
                
                if let Some(node_id) = selected_node {
                    ui.separator();
                    ui.strong(format!("Selected: {}", 
                        self.information_graph.nodes.get(&node_id)
                            .map(|n| n.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string())
                    ));
                }
            });
            
            ui.separator();
            
            // Draw the graph visualization
            let rect = ui.available_rect_before_wrap();
            let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
            let painter = ui.painter();
            
            // Calculate positions for nodes based on selected layout and view mode
            if !self.information_graph.nodes.is_empty() {
                // Remove any positions for nodes that no longer exist
                node_positions.retain(|id, _| self.information_graph.nodes.contains_key(id));
                node_positions_3d.retain(|id, _| self.information_graph.nodes.contains_key(id));
                
                // Apply force-directed layout if active
                if self.applying_force_layout {
                    if self.view_mode == ViewMode::ThreeDimensional {
                        self.apply_3d_force_directed_layout(&mut node_positions_3d);
                        
                        // Update 2D positions for compatibility
                        for (id, pos_3d) in &node_positions_3d {
                            node_positions.insert(*id, self.project_3d_to_2d(pos_3d, &rect));
                        }
                    } else {
                        self.apply_force_directed_layout(&mut node_positions);
                        
                        // Update 3D positions for compatibility
                        for (id, pos_2d) in &node_positions {
                            if !node_positions_3d.contains_key(id) {
                                node_positions_3d.insert(*id, Position3D::from_pos2(*pos_2d, 0.0));
                            } else {
                                let pos_3d = node_positions_3d.get_mut(id).unwrap();
                                pos_3d.x = pos_2d.x;
                                pos_3d.y = pos_2d.y;
                            }
                        }
                    }
                    
                    // Request a repaint to animate the layout
                    ctx.request_repaint();
                }
                
                // Handle initialization of positions for new nodes
                if node_positions.len() < self.information_graph.nodes.len() || 
                   node_positions_3d.len() < self.information_graph.nodes.len() {
                    // Check for nodes with fixed positions from their properties
                    for (id, node) in &self.information_graph.nodes {
                        // Handle 2D positions first
                        if !node_positions.contains_key(id) {
                            if node.properties.contains_key("fixed_position") && 
                               node.properties.contains_key("x_pos") &&
                               node.properties.contains_key("y_pos") {
                                
                                // Convert stored position from strings to floats
                                if let (Ok(x), Ok(y)) = (
                                    node.properties.get("x_pos").unwrap().parse::<f32>(),
                                    node.properties.get("y_pos").unwrap().parse::<f32>()
                                ) {
                                    // Apply the fixed position, centered in the available rect
                                    let center_x = rect.center().x;
                                    let center_y = rect.center().y;
                                    node_positions.insert(*id, egui::Pos2::new(center_x + x, center_y + y));
                                }
                            }
                        }
                        
                        // Now handle 3D positions
                        if !node_positions_3d.contains_key(id) {
                            if node.properties.contains_key("fixed_position") && 
                               node.properties.contains_key("x_pos") &&
                               node.properties.contains_key("y_pos") {
                                
                                // Get z-position if specified, otherwise default to 0.0
                                let z = if node.properties.contains_key("z_pos") {
                                    node.properties.get("z_pos").unwrap().parse::<f32>().unwrap_or(0.0)
                                } else {
                                    0.0
                                };
                                
                                // Convert stored position from strings to floats
                                if let (Ok(x), Ok(y)) = (
                                    node.properties.get("x_pos").unwrap().parse::<f32>(),
                                    node.properties.get("y_pos").unwrap().parse::<f32>()
                                ) {
                                    // Apply the fixed 3D position
                                    let center_x = rect.center().x;
                                    let center_y = rect.center().y;
                                    node_positions_3d.insert(*id, Position3D::new(center_x + x, center_y + y, z));
                                }
                            }
                        }
                    }
                }
                
                // For the remaining nodes without fixed positions, apply the selected layout
                let missing_nodes = self.information_graph.nodes.len() - node_positions.len();
                if missing_nodes > 0 {
                    match self.layout_type {
                        LayoutType::Circular => {
                            self.apply_circular_layout(&mut node_positions, rect);
                        },
                        LayoutType::ForceDirected => {
                            // Initialize with circular layout then let force layout take over
                            let mut temp_positions = std::collections::HashMap::new();
                            self.apply_circular_layout(&mut temp_positions, rect);
                            
                            // Copy positions for new nodes
                            for (id, pos) in temp_positions {
                                if !node_positions.contains_key(&id) {
                                    node_positions.insert(id, pos);
                                }
                            }
                            
                            // Start force-directed layout if not already running
                            if !self.applying_force_layout {
                                self.applying_force_layout = true;
                                self.layout_iterations = 0;
                            }
                        },
                        LayoutType::Hierarchical => {
                            self.apply_hierarchical_layout(&mut node_positions, rect);
                        }
                    }
                }
                
                // Initialize 3D positions for nodes that don't have them
                for (id, _) in &self.information_graph.nodes {
                    if !node_positions_3d.contains_key(id) {
                        if let Some(pos_2d) = node_positions.get(id) {
                            // Create 3D position from 2D with random Z
                            let random_z = rand::rng().random_range(-50.0..50.0);
                            node_positions_3d.insert(*id, Position3D::from_pos2(*pos_2d, random_z));
                        }
                    }
                }
                
                // Draw edges
                for (_, edge) in &self.information_graph.edges {
                    let (src_pos, dst_pos) = if self.view_mode == ViewMode::ThreeDimensional {
                        if let (Some(src_pos_3d), Some(dst_pos_3d)) = (
                            node_positions_3d.get(&edge.source), 
                            node_positions_3d.get(&edge.target)
                        ) {
                            // Project 3D positions to 2D for rendering
                            (
                                self.project_3d_to_2d(src_pos_3d, &rect),
                                self.project_3d_to_2d(dst_pos_3d, &rect)
                            )
                        } else {
                            continue;
                        }
                    } else {
                        if let (Some(src_pos), Some(dst_pos)) = (
                            node_positions.get(&edge.source), 
                            node_positions.get(&edge.target)
                        ) {
                            (*src_pos, *dst_pos)
                        } else {
                            continue;
                        }
                    };
                    
                    // Determine edge color based on labels
                    let edge_type_color = if edge.labels.contains(&"connection".to_string()) {
                        self.graph_settings.edge_color
                    } else if edge.labels.contains(&"dependency".to_string()) {
                        egui::Color32::from_rgb(200, 100, 100)
                    } else if edge.labels.contains(&"flow".to_string()) {
                        egui::Color32::from_rgb(100, 200, 100)
                    } else {
                        self.graph_settings.edge_color
                    };
                    
                    // Calculate control point for curved edge
                    let mid_point = egui::Pos2::lerp(&src_pos, dst_pos, 0.5);
                    let normal = egui::Vec2::new(-(dst_pos.y - src_pos.y), dst_pos.x - src_pos.x).normalized();
                    
                    // Make self-loops more visible
                    let curve_strength = if edge.source == edge.target {
                        50.0
                    } else {
                        20.0
                    };
                    
                    let control_point = mid_point + normal * curve_strength;
                    
                    // Draw a quadratic bezier curve for the edge
                    let steps = 20; // Number of line segments to approximate the curve
                    let mut prev_point = src_pos;
                    
                    for i in 1..=steps {
                        let t = i as f32 / steps as f32;
                        
                        // Quadratic Bezier curve calculation using proper references
                        let p0 = prev_point.lerp(control_point, t);
                        let p1 = control_point.lerp(dst_pos, t);
                        let curve_point = p0.lerp(p1, t);
                        
                        // Draw line segment of the curve using tuples converted to array
                        painter.line_segment(
                            (prev_point, curve_point).into(),
                            egui::Stroke::new(self.graph_settings.edge_thickness, edge_type_color)
                        );
                        
                        prev_point = curve_point;
                    }
                    
                    // Draw arrowhead if enabled
                    if self.graph_settings.show_arrows {
                        // Calculate direction vector for arrow at the end of the curve
                        let last_segment = dst_pos - prev_point;
                        let dir = last_segment.normalized();
                        let arrow_size = 10.0;
                        let arrow_pos = dst_pos - dir * self.graph_settings.node_size; // offset by node radius
                        
                        // Draw arrowhead lines
                        let perpendicular = egui::Vec2::new(-dir.y, dir.x);
                        let arrow_left = arrow_pos - perpendicular * arrow_size - dir * arrow_size;
                        let arrow_right = arrow_pos + perpendicular * arrow_size - dir * arrow_size;
                        
                        painter.line_segment(
                            (arrow_pos, arrow_left).into(),
                            egui::Stroke::new(self.graph_settings.edge_thickness, edge_type_color)
                        );
                        painter.line_segment(
                            (arrow_pos, arrow_right).into(),
                            egui::Stroke::new(self.graph_settings.edge_thickness, edge_type_color)
                        );
                    }
                }
                
                // Check for node dragging and clicks
                let mut hovered_node: Option<Uuid> = None;
                let mut clicked_node: Option<Uuid> = None;
                
                // Draw the nodes and check for interactions
                for (id, node) in &self.information_graph.nodes {
                    let pos = if self.view_mode == ViewMode::ThreeDimensional {
                        if let Some(pos_3d) = node_positions_3d.get(id) {
                            // Project 3D position to 2D for rendering
                            self.project_3d_to_2d(pos_3d, &rect)
                        } else {
                            continue;
                        }
                    } else {
                        if let Some(pos_2d) = node_positions.get(id) {
                            *pos_2d
                        } else {
                            continue;
                        }
                    };
                    
                    // In 3D mode, calculate node size based on z-position for depth effect
                    let node_size = if self.view_mode == ViewMode::ThreeDimensional {
                        if let Some(pos_3d) = node_positions_3d.get(id) {
                            // Scale size based on z distance (farther = smaller)
                            let z_factor = self.camera_distance / (self.camera_distance + pos_3d.z).max(0.1);
                            self.graph_settings.node_size * z_factor
                        } else {
                            self.graph_settings.node_size
                        }
                    } else {
                        self.graph_settings.node_size
                    };
                    
                    // Determine node color based on selection
                    let current_node_color = if Some(*id) == selected_node {
                        egui::Color32::from_rgb(220, 100, 100) // Highlighted color
                    } else {
                        self.graph_settings.node_color // Default color from settings
                    };
                    
                    // Draw node circle
                    painter.circle_filled(pos, node_size, current_node_color);
                    
                    // Draw node text (name) if enabled
                    if self.graph_settings.show_node_labels {
                        painter.text(
                            pos,
                            egui::Align2::CENTER_CENTER,
                            &node.name,
                            egui::FontId::proportional(24.0),
                            egui::Color32::WHITE
                        );
                    }
                    
                    // Draw node properties if enabled
                    if self.graph_settings.show_node_properties && node.properties.len() > 0 {
                        let props_text = node.properties.iter()
                            .take(3) // Show at most 3 properties
                            .map(|(k, v)| format!("{}: {}", k, v))
                            .collect::<Vec<_>>()
                            .join("\n");
                            
                        let props_pos = pos + egui::vec2(0.0, node_size + 5.0);
                        painter.text(
                            props_pos,
                            egui::Align2::CENTER_TOP,
                            props_text,
                            egui::FontId::proportional(16.0),
                            egui::Color32::LIGHT_GRAY
                        );
                    }
                    
                    // Check if mouse is over this node
                    let node_rect = egui::Rect::from_center_size(
                        pos, 
                        egui::Vec2::new(node_size * 2.0, node_size * 2.0)
                    );
                    
                    // Check for hover and clicks
                    if node_rect.contains(response.hover_pos().unwrap_or_default()) {
                        hovered_node = Some(*id);
                        
                        // Show tooltip on hover
                        ui.ctx().debug_painter().text(
                            response.hover_pos().unwrap_or_default() + egui::vec2(15.0, 15.0),
                            egui::Align2::LEFT_TOP,
                            format!("{}\nID: {}\nProperties: {}\nLabels: {}", 
                                node.name, 
                                id, 
                                node.properties.len(), 
                                node.labels.join(", ")
                            ),
                            egui::FontId::proportional(18.0),
                            egui::Color32::WHITE,
                        );
                        
                        if response.clicked() {
                            clicked_node = Some(*id);
                        }
                    }
                    
                    // Handle dragging
                    if response.dragged() && response.hover_pos().is_some() {
                        if let Some(hover_pos) = response.hover_pos() {
                            if node_rect.contains(hover_pos) || Some(*id) == selected_node {
                                let delta = response.drag_delta();
                                
                                if self.view_mode == ViewMode::ThreeDimensional {
                                    // In 3D mode, update the 3D position
                                    if let Some(pos_3d) = node_positions_3d.get_mut(id) {
                                        // Translate in screen space
                                        pos_3d.x += delta.x;
                                        pos_3d.y += delta.y;
                                        
                                        // Update 2D position for compatibility
                                        let new_pos_2d = self.project_3d_to_2d(pos_3d, &rect);
                                        node_positions.insert(*id, new_pos_2d);
                                    }
                                } else {
                                    // Update 2D position
                                    let mut curr_pos = pos;
                                    curr_pos.x += delta.x;
                                    curr_pos.y += delta.y;
                                    node_positions.insert(*id, curr_pos);
                                    
                                    // Update 3D position for compatibility
                                    if let Some(pos_3d) = node_positions_3d.get_mut(id) {
                                        pos_3d.x = curr_pos.x;
                                        pos_3d.y = curr_pos.y;
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Update selected node if a node was clicked
                if let Some(node_id) = clicked_node {
                    if selected_node == Some(node_id) {
                        selected_node = None; // Deselect if clicked again
                        self.show_extra_panel = false; // Hide the detail panel when deselecting
                    } else {
                        selected_node = Some(node_id); // Select new node
                        self.show_extra_panel = true; // Show the detail panel
                    }
                } else if response.clicked() && hovered_node.is_none() {
                    // Clicked empty space, deselect
                    selected_node = None;
                    self.show_extra_panel = false; // Hide the detail panel when deselecting
                }
            } else {
                // Show a message if there are no nodes
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "No nodes in graph - Click 'Generate Test Nodes' to create some",
                    egui::FontId::proportional(32.0),
                    egui::Color32::WHITE
                );
            }
            
            // Show detailed information about selected node
            if let Some(node_id) = selected_node {
                if let Some(node) = self.information_graph.nodes.get(&node_id) {
                    ui.separator();
                    ui.heading(format!("Selected Node: {}", node.name));
                    
                    // Add horizontal layout with ID and View Details button
                    ui.horizontal(|ui| {
                        ui.label(format!("ID: {}", node_id));
                        
                        // Add a button to open the detailed view panel
                        if !self.show_extra_panel && ui.button("View Details").clicked() {
                            self.show_extra_panel = true;
                        }
                    });
                    
                    ui.label("Properties:");
                    for (key, value) in &node.properties {
                        ui.horizontal(|ui| {
                            ui.label(format!("• {}: {}", key, value));
                        });
                    }
                    
                    ui.label(format!("Labels: {}", node.labels.join(", ")));
                    ui.label(format!("Radius: {:.2}", node.radius));
                    
                    // Show connected edges
                    ui.collapsing("Connected Edges", |ui| {
                        let mut has_edges = false;
                        
                        ui.label("Outgoing:");
                        for (_, edge) in &self.information_graph.edges {
                            if edge.source == node_id {
                                has_edges = true;
                                if let Some(target) = self.information_graph.nodes.get(&edge.target) {
                                    ui.label(format!("→ {} ({})", target.name, edge.target));
                                }
                            }
                        }
                        
                        ui.label("Incoming:");
                        for (_, edge) in &self.information_graph.edges {
                            if edge.target == node_id {
                                has_edges = true;
                                if let Some(source) = self.information_graph.nodes.get(&edge.source) {
                                    ui.label(format!("← {} ({})", source.name, edge.source));
                                }
                            }
                        }
                        
                        if !has_edges {
                            ui.label("No connected edges");
                        }
                    });
                }
            }
            
            // Handle keyboard controls for 3D mode
            if self.view_mode == ViewMode::ThreeDimensional && self.current_view == ViewType::Main && selected_node.is_some() {
                // Get input from keyboard for Z-axis manipulation
                if ui.input(|i| i.key_pressed(egui::Key::PageUp)) {
                    // Move selected node forward (increase Z)
                    if let Some(node_id) = selected_node {
                        if let Some(pos_3d) = node_positions_3d.get_mut(&node_id) {
                            pos_3d.z += 10.0;
                            // Update the 2D projection
                            let new_pos_2d = self.project_3d_to_2d(pos_3d, &rect);
                            node_positions.insert(node_id, new_pos_2d);
                        }
                    }
                }
                
                if ui.input(|i| i.key_pressed(egui::Key::PageDown)) {
                    // Move selected node backward (decrease Z)
                    if let Some(node_id) = selected_node {
                        if let Some(pos_3d) = node_positions_3d.get_mut(&node_id) {
                            pos_3d.z -= 10.0;
                            // Update the 2D projection
                            let new_pos_2d = self.project_3d_to_2d(pos_3d, &rect);
                            node_positions.insert(node_id, new_pos_2d);
                        }
                    }
                }
                
                // Rotate camera with arrow keys
                if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                    self.camera_rotation_y -= 0.1;
                }
                
                if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                    self.camera_rotation_y += 0.1;
                }
                
                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    self.camera_rotation_x -= 0.1;
                }
                
                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    self.camera_rotation_x += 0.1;
                }
                
                // Add camera panning with WASD keys
                if ui.input(|i| i.key_pressed(egui::Key::W)) {
                    self.camera_pan_y -= 50.0;
                }
                
                if ui.input(|i| i.key_pressed(egui::Key::S)) {
                    self.camera_pan_y += 50.0;
                }
                
                if ui.input(|i| i.key_pressed(egui::Key::A)) {
                    self.camera_pan_x -= 50.0;
                }
                
                if ui.input(|i| i.key_pressed(egui::Key::D)) {
                    self.camera_pan_x += 50.0;
                }
                
                // Zoom with Q and E keys
                if ui.input(|i| i.key_pressed(egui::Key::Q)) {
                    self.camera_distance += 20.0;
                    self.camera_distance = self.camera_distance.min(1000.0);
                }
                
                if ui.input(|i| i.key_pressed(egui::Key::E)) {
                    self.camera_distance -= 20.0;
                    self.camera_distance = self.camera_distance.max(100.0);
                }
                
                // Reset camera with Home key including panning
                if ui.input(|i| i.key_pressed(egui::Key::Home)) {
                    self.camera_rotation_x = 0.0;
                    self.camera_rotation_y = 0.0;
                    self.camera_pan_x = 0.0;
                    self.camera_pan_y = 0.0;
                }

                // Add a keyboard shortcut to focus on selected node with F key
                if ui.input(|i| i.key_pressed(egui::Key::F)) && selected_node.is_some() {
                    if let Some(node_id) = selected_node {
                        let node_positions_3d_id = egui::Id::new("node_positions_3d");
                        let node_positions_3d: std::collections::HashMap<Uuid, Position3D> = 
                            ctx.memory_mut(|mem| mem.data.get_persisted(node_positions_3d_id))
                            .unwrap_or_default();
                            
                        self.focus_on_node(node_id, &node_positions_3d);
                    }
                }
            }
            
            // In 3D mode, add a help panel
            if self.view_mode == ViewMode::ThreeDimensional && self.current_view == ViewType::Main {
                egui::Window::new("3D Controls Help")
                    .default_pos(egui::pos2(rect.right() - 250.0, rect.top() + 50.0))
                    .show(ctx, |ui| {
                        ui.label("3D Node Controls:");
                        ui.label("• Page Up/Down: Move node in Z-axis");
                        ui.label("• Click + Drag: Move node in X-Y plane");
                        ui.separator();
                        ui.label("Camera Controls:");
                        ui.label("• Arrow Keys: Rotate camera");
                        ui.label("• W/A/S/D: Pan camera");
                        ui.label("• Q/E: Zoom out/in");
                        ui.label("• Home: Reset camera");
                        ui.label("• F: Focus on selected node");
                        ui.separator();
                        ui.label("Special Functions:");
                        ui.label("• Fit to View: Adjust camera to see all nodes");
                        ui.label("• Focus Selected: Zoom to selected node");
                        ui.label("• Redistribute in 3D: Organize nodes in layers");
                        ui.separator();
                        ui.label("Tips:");
                        ui.label("• Use Redistribute in 3D to better visualize");
                        ui.label("  graph structure and connected components");
                        ui.label("• Nodes with more connections are placed");
                        ui.label("  closer to the front (higher Z value)");
                    });
            }
        });
        
        // Store node positions and selection for the next frame
        ctx.memory_mut(|mem| mem.data.insert_persisted(node_positions_id, node_positions));
        ctx.memory_mut(|mem| mem.data.insert_persisted(node_positions_3d_id, node_positions_3d));
        ctx.memory_mut(|mem| mem.data.insert_persisted(selected_node_id, selected_node));
    }
    
    fn show_workflow_view(&mut self, ctx: &egui::Context) {
        // Left panel - Pattern catalog
        egui::SidePanel::left("workflow_left_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Workflow Templates");
                ui.add_space(10.0);
                
                if ui.button("New Workflow").clicked() {
                    self.workflow_editor = WorkflowEditor::new();
                }
                
                if ui.button("Example Workflow").clicked() {
                    self.workflow_editor = WorkflowEditor::new();
                    self.workflow_editor.workflow.create_example_workflow();
                }
                
                ui.add_space(5.0);
                ui.separator();
                
                ui.collapsing("ETL Workflows", |ui| {
                    if ui.button("Data Ingestion Pipeline").clicked() {
                        self.workflow_editor = WorkflowEditor::new();
                        // Create a data ingestion workflow template
                        let workflow = &mut self.workflow_editor.workflow;
                        
                        let source = workflow.graph.add_node("Data Source", vec!["workflow".to_string()]);
                        workflow.graph.add_property(source, "type".to_string(), "database".to_string());
                        
                        let extract = workflow.graph.add_node("Extract", vec!["workflow".to_string(), "process".to_string()]);
                        workflow.graph.add_property(extract, "sql".to_string(), "SELECT * FROM data".to_string());
                        
                        let transform = workflow.graph.add_node("Transform", vec!["workflow".to_string(), "process".to_string()]);
                        workflow.graph.add_property(transform, "operation".to_string(), "clean".to_string());
                        
                        let load = workflow.graph.add_node("Load", vec!["workflow".to_string(), "destination".to_string()]);
                        workflow.graph.add_property(load, "target".to_string(), "data_warehouse".to_string());
                        
                        workflow.graph.add_edge(source, extract, vec!["data_flow".to_string()]);
                        workflow.graph.add_edge(extract, transform, vec!["data_flow".to_string()]);
                        workflow.graph.add_edge(transform, load, vec!["data_flow".to_string()]);
                        
                        workflow.sync_to_snarl();
                    }
                });
                
                ui.collapsing("Decision Trees", |ui| {
                    if ui.button("Simple Decision Process").clicked() {
                        self.workflow_editor = WorkflowEditor::new();
                        // Create a decision tree workflow template
                        let workflow = &mut self.workflow_editor.workflow;
                        
                        let start = workflow.graph.add_node("Start", vec!["workflow".to_string(), "start".to_string()]);
                        
                        let decision1 = workflow.graph.add_node("Decision 1", vec!["workflow".to_string(), "decision".to_string()]);
                        workflow.graph.add_property(decision1, "condition".to_string(), "value > 10".to_string());
                        
                        let process_true = workflow.graph.add_node("Process A", vec!["workflow".to_string(), "process".to_string()]);
                        let process_false = workflow.graph.add_node("Process B", vec!["workflow".to_string(), "process".to_string()]);
                        
                        let end = workflow.graph.add_node("End", vec!["workflow".to_string(), "end".to_string()]);
                        
                        workflow.graph.add_edge(start, decision1, vec!["flow".to_string()]);
                        workflow.graph.add_edge(decision1, process_true, vec!["true".to_string()]);
                        workflow.graph.add_edge(decision1, process_false, vec!["false".to_string()]);
                        workflow.graph.add_edge(process_true, end, vec!["flow".to_string()]);
                        workflow.graph.add_edge(process_false, end, vec!["flow".to_string()]);
                        
                        workflow.sync_to_snarl();
                    }
                });
            });
        
        // Main content area - Workflow editor
        egui::CentralPanel::default().show(ctx, |ui| {
            // Pass control to the workflow editor UI
            self.workflow_editor.ui(ui);
        });
        
        // Right panel - Node details and properties
        egui::SidePanel::right("workflow_right_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Node Details");
                ui.add_space(10.0);
                
                if let Some(node_id) = self.workflow_editor.selected_node {
                    if let Some(node) = self.workflow_editor.workflow.graph.get_node(node_id) {
                        ui.label(format!("Node: {}", node.name));
                        ui.add_space(5.0);
                        
                        ui.label("Labels:");
                        for label in &node.labels {
                            ui.label(format!("• {}", label));
                        }
                        
                        ui.add_space(5.0);
                        ui.label("Properties:");
                        
                        for (key, value) in &node.properties {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}:", key));
                                ui.label(value);
                            });
                        }
                        
                        ui.add_space(10.0);
                        ui.separator();
                        
                        // Add property form
                        ui.collapsing("Add Property", |ui| {
                            static mut NEW_PROP_KEY: String = String::new();
                            static mut NEW_PROP_VALUE: String = String::new();
                            
                            let key_ref = unsafe { &mut NEW_PROP_KEY };
                            let value_ref = unsafe { &mut NEW_PROP_VALUE };
                            
                            ui.horizontal(|ui| {
                                ui.label("Key:");
                                ui.text_edit_singleline(key_ref);
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Value:");
                                ui.text_edit_singleline(value_ref);
                            });
                            
                            if ui.button("Add").clicked() && !key_ref.is_empty() {
                                self.workflow_editor.workflow.graph.add_property(
                                    node_id, 
                                    key_ref.clone(), 
                                    value_ref.clone()
                                );
                                *key_ref = String::new();
                                *value_ref = String::new();
                            }
                        });
                    } else {
                        ui.label("No node selected");
                    }
                } else {
                    ui.label("No node selected");
                }
            });
    }
    
    fn show_settings_view(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Alchemist Settings");
            ui.label("Configure the application settings here.");
            
            ui.add_space(20.0);
            
            ui.collapsing("About Graphs in Alchemist", |ui| {
                ui.label("Alchemist represents the universe as a hypergraph.");
                ui.label("A hypergraph is a special graph that allows an edge to connect more than two nodes.");
                ui.add_space(10.0);
                ui.label("Graph Features:");
                ui.label("• Nodes represent things (nouns)");
                ui.label("• Edges represent relationships (actions/behaviors)");
                ui.label("• Edges have direction");
                ui.label("• Nodes and edges have properties and labels");
                ui.label("• Nodes have a radius");
                ui.label("• Edges have a weight");
            });
            
            ui.collapsing("About Workflows", |ui| {
                ui.label("Workflows are a special type of graph representing sequential processes.");
                ui.label("Use the Workflow Editor to create and edit workflow graphs.");
            });
            
            ui.collapsing("About ECS", |ui| {
                ui.label("Entity Component System Architecture:");
                ui.label("• Entities are identifiable objects with unique identifiers");
                ui.label("• Components are values (collections of data structures)");
                ui.label("• Systems provide behavior and functionality");
                ui.add_space(10.0);
                ui.label("The system maintains a separation of data and behavior.");
            });
            
            ui.collapsing("About Events", |ui| {
                ui.label("The event system follows the Command-Event pattern:");
                ui.label("• Commands produce Events");
                ui.label("• Events are applied to models");
                ui.label("• The event stream is append-only and sequential");
                ui.label("• Models are reconstructed by replaying events");
            });
        });
    }
    
    fn show_events_view(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Event Stream");
            ui.label(format!("Total events: {}", self.event_stream.len()));
            
            ui.add_space(10.0);
            
            if ui.button("Replay All Events").clicked() {
                // Clear and reconstruct the models from events
                self.information_graph = AlchemistGraph::new();
                
                // Re-create the graph system
                self.graph_system = GraphSystem::new();
                
                // Replay all events in order
                for event in self.event_stream.get_events() {
                    if let Some(graph_event) = event.as_any().downcast_ref::<GraphEvent>() {
                        self.information_graph.apply_event(graph_event);
                        self.graph_system.apply_event(graph_event);
                    }
                }
            }
            
            // Display events in a scrollable list
            egui::ScrollArea::vertical().show(ui, |ui| {
                for event in self.event_stream.get_events() {
                    ui.group(|ui| {
                        ui.label(format!("Type: {}", event.event_type()));
                        if let Some(entity_id) = event.entity_id() {
                            ui.label(format!("Entity: {}", entity_id));
                        }
                        ui.label(format!("Timestamp: {}", event.timestamp()));
                        
                        // Try to downcast to GraphEvent to get payload
                        if let Some(graph_event) = event.as_any().downcast_ref::<GraphEvent>() {
                            ui.label("Payload:");
                            for (key, value) in &graph_event.payload {
                                ui.label(format!("  {}: {}", key, value));
                            }
                        }
                    });
                    ui.add_space(5.0);
                }
            });
        });
    }
}