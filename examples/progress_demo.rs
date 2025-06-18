use bevy::prelude::*;
use bevy::input::mouse::{MouseWheel, MouseMotion};
use cim_contextgraph::ContextGraph;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Progress Graph Demo - Enhanced".into(),
                resolution: (1600.0, 1000.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 100.0,
            affects_lightmapped_meshes: true,
        })
        .insert_resource(ProgressGraphData::default())
        .insert_resource(SelectedNode::default())
        .insert_resource(EventLog::default())
        .insert_resource(SearchState::default())
        .insert_resource(FilterState::default())
        .insert_resource(PathVisualization::default())
        .insert_resource(GraphAnimationState {
            nodes_to_spawn: Vec::new(),
            edges_to_spawn: Vec::new(),
            node_spawn_index: 0,
            edge_spawn_index: 0,
            layout_iterations: 0,
            node_positions: HashMap::new(),
            node_entities: HashMap::new(),
            spawn_timer: 0.0,
            layout_timer: 0.0,
            phase: AnimationPhase::SpawningNodes,
        })
        .add_systems(Startup, (setup_camera, setup_ui, load_progress_graph))
        .add_systems(Update, (
            animate_graph_construction,
            animate_node_movement,
            animate_fade_in,
            animate_edge_growth,
            update_event_log_panel,
        ))
        .add_systems(Update, (
            camera_controller,
            update_info_panel,
            handle_node_clicks,
            highlight_connected_edges,
            handle_search_input,
            apply_filters,
            visualize_paths,
            update_search_panel,
            highlight_selected_nodes,
        ))
        .add_systems(Update, (
            handle_reset_input,
        ))
        .run();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProgressData {
    metadata: ProgressMetadata,
    nodes: Vec<ProgressNode>,
    edges: Vec<ProgressEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProgressMetadata {
    name: String,
    description: String,
    version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProgressNode {
    id: String,
    #[serde(alias = "name")]
    label: Option<String>,
    #[serde(rename = "type")]
    node_type: Option<String>,
    #[serde(default)]
    position: Option<Position3D>,
    #[serde(default)]
    data: Option<ProgressNodeData>,
    // Handle alternative structure fields
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    dependencies: Option<Vec<String>>,
    #[serde(default)]
    outputs: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Position3D {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProgressNodeData {
    #[serde(default)]
    status: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    details: Vec<String>,
    #[serde(default)]
    progress: Option<f32>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    completed_date: Option<String>,
    #[serde(default)]
    target_date: Option<String>,
    #[serde(default)]
    week: Option<f32>,
    #[serde(default)]
    parent: Option<String>,
    #[serde(default)]
    references: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProgressEdge {
    id: String,
    source: String,
    target: String,
    relationship: String,
    label: Option<String>,
}

// ECS Components
#[derive(Component, Debug, Clone)]
struct ProgressNodeComponent {
    node_id: String,
    label: String,
    node_type: String,
    data: ProgressNodeData,
}

#[derive(Component, Debug, Clone)]
struct ProgressEdgeComponent {
    edge_id: String,
    edge_type: String,
    label: String,
}

#[derive(Component)]
struct CameraController {
    sensitivity: f32,
    speed: f32,
}

#[derive(Component)]
struct InfoPanel;

#[derive(Component)]
struct MetadataPanel;

#[derive(Component)]
struct Clickable;

#[derive(Component)]
struct EventLogPanel;

// Animation components
#[derive(Component)]
struct AnimatedNode {
    target_position: Vec3,
    spawn_time: f32,
    animation_duration: f32,
}

#[derive(Component)]
struct AnimatedEdge {
    spawn_time: f32,
    animation_duration: f32,
    target_scale: Vec3,
}

#[derive(Component)]
struct FadingIn {
    start_time: f32,
    duration: f32,
    start_alpha: f32,
    end_alpha: f32,
}

// New component to track edge connections
#[derive(Component)]
struct EdgeConnection {
    source_entity: Entity,
    target_entity: Entity,
}

// Search and filter components
#[derive(Component)]
struct SearchPanel;

#[derive(Component)]
struct FilterPanel;

#[derive(Component)]
struct Highlighted;

#[derive(Component)]
struct PathNode {
    path_id: u32,
    order: usize,
}

#[derive(Component)]
struct PathEdge {
    path_id: u32,
}

// Animation state
#[derive(Resource)]
struct GraphAnimationState {
    nodes_to_spawn: Vec<(String, Vec3, ProgressNodeComponent)>,
    edges_to_spawn: Vec<(String, String, ProgressEdgeComponent)>,
    node_spawn_index: usize,
    edge_spawn_index: usize,
    layout_iterations: usize,
    node_positions: HashMap<String, Vec3>,
    node_entities: HashMap<String, Entity>,
    spawn_timer: f32,
    layout_timer: f32,
    phase: AnimationPhase,
}

#[derive(PartialEq)]
enum AnimationPhase {
    SpawningNodes,
    AnimatingLayout,
    SpawningEdges,
    Complete,
}

// Resources
#[derive(Resource, Default)]
struct ProgressGraphData {
    node_count: usize,
    edge_count: usize,
    project_info: Option<String>,
    graph: Option<ContextGraph<serde_json::Value, serde_json::Value>>,
}

#[derive(Resource, Default)]
struct SelectedNode {
    node_data: Option<ProgressNodeData>,
    node_label: Option<String>,
    selected_entity: Option<Entity>,
}

#[derive(Resource)]
struct EventLog {
    messages: Vec<String>,
    max_messages: usize,
}

impl Default for EventLog {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            max_messages: 20,
        }
    }
}

impl EventLog {
    fn add_message(&mut self, message: String) {
        self.messages.push(format!("[{}] {}", 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() % 1000,
            message
        ));
        
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
        
        println!("{}", message); // Also print to console
    }
    
    fn get_display_text(&self) -> String {
        if self.messages.is_empty() {
            "📝 Event Log\n\nNo events yet...".to_string()
        } else {
            format!("📝 Event Log\n\n{}", self.messages.join("\n"))
        }
    }
}

// Search and filter resources
#[derive(Resource, Default)]
struct SearchState {
    is_active: bool,
    query: String,
    matching_nodes: HashSet<String>,
}

#[derive(Resource)]
struct FilterState {
    node_types: HashSet<String>,
    statuses: HashSet<String>,
    min_progress: Option<f32>,
    max_progress: Option<f32>,
    show_all: bool,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            node_types: HashSet::new(),
            statuses: HashSet::new(),
            min_progress: None,
            max_progress: None,
            show_all: true, // Show all nodes by default
        }
    }
}

impl FilterState {
    fn is_filtered(&self, node_data: &ProgressNodeData) -> bool {
        if self.show_all {
            return true;
        }
        
        // Check node type filter
        if !self.node_types.is_empty() {
            // For now, we'll check based on status since we don't have node_type in ProgressNodeData
            // This is a simplified version
            return self.statuses.contains(&node_data.status);
        }
        
        // Check status filter
        if !self.statuses.is_empty() && !self.statuses.contains(&node_data.status) {
            return false;
        }
        
        // Check progress filter
        if let Some(progress) = node_data.progress {
            if let Some(min) = self.min_progress {
                if progress < min {
                    return false;
                }
            }
            if let Some(max) = self.max_progress {
                if progress > max {
                    return false;
                }
            }
        }
        
        true
    }
}

// Path visualization
#[derive(Resource)]
struct PathVisualization {
    is_active: bool,
    start_node: Option<Entity>,
    end_node: Option<Entity>,
    current_path: Vec<Entity>,
    path_edges: Vec<Entity>,
}

impl Default for PathVisualization {
    fn default() -> Self {
        Self {
            is_active: false,
            start_node: None,
            end_node: None,
            current_path: Vec::new(),
            path_edges: Vec::new(),
        }
    }
}

impl PathVisualization {
    fn clear_path(&mut self) {
        self.current_path.clear();
        self.path_edges.clear();
        self.start_node = None;
        self.end_node = None;
    }

    fn next_node(&self, _current: Option<Entity>) -> Option<Entity> {
        // Path traversal logic would go here
        None
    }
}

// Color mapping functions
fn get_edge_color(relationship: &str) -> Color {
    match relationship {
        "sequence" => Color::linear_rgb(1.0, 0.5, 0.0), // Orange
        "leads_to" => Color::linear_rgb(0.0, 0.5, 1.0), // Blue  
        "enables" => Color::linear_rgb(0.0, 1.0, 0.0),  // Green
        "implemented_by" => Color::linear_rgb(0.8, 0.0, 1.0), // Purple
        "expanded_to" | "expanded_by" => Color::linear_rgb(1.0, 1.0, 0.0), // Yellow
        "corrected_by" | "refactored_by" => Color::linear_rgb(1.0, 0.0, 0.0), // Red
        "requires" | "required" => Color::linear_rgb(0.0, 1.0, 1.0), // Cyan
        "triggers" | "continues_to" => Color::linear_rgb(1.0, 0.0, 1.0), // Magenta
        _ => Color::linear_rgb(0.7, 0.7, 0.7), // Gray
    }
}

fn get_node_color(node_type: &str, status: &str) -> Color {
    match (node_type, status) {
        (_, "completed") => Color::linear_rgb(0.0, 0.8, 0.0), // Green
        (_, "in-progress") => Color::linear_rgb(1.0, 1.0, 0.0), // Yellow
        ("milestone", _) => Color::linear_rgb(0.0, 0.5, 1.0), // Blue
        ("phase", _) => Color::linear_rgb(1.0, 0.0, 0.5), // Pink
        ("task", _) => Color::linear_rgb(0.5, 0.5, 0.5), // Gray
        _ => Color::linear_rgb(0.8, 0.8, 0.8), // Light gray
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 3000.0, 8000.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraController {
            sensitivity: 2.0,
            speed: 2000.0,
        },
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn setup_ui(mut commands: Commands, mut event_log: ResMut<EventLog>) {
    event_log.add_message("🚀 Starting Progress Graph Demo".to_string());
    event_log.add_message("🎨 Setting up UI components".to_string());

    // Info panel (left side)
    commands
        .spawn((
            Node {
                width: Val::Px(350.0),
                height: Val::Px(400.0),
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::linear_rgba(0.0, 0.0, 0.0, 0.8)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Loading Progress Graph..."),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                InfoPanel,
            ));
        });

    // Search & Filter Panel (top right)
    commands
        .spawn((
            Node {
                width: Val::Px(300.0),
                height: Val::Px(200.0),
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                top: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::linear_rgba(0.1, 0.0, 0.1, 0.8)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("🔍 Search & Filter\n\nPress 'S' to search\nPress 'F' to filter\nPress 'P' for paths"),
                TextColor(Color::linear_rgb(1.0, 0.8, 1.0)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                SearchPanel,
            ));
        });

    // Event Log Panel (bottom left)
    commands
        .spawn((
            Node {
                width: Val::Px(350.0),
                height: Val::Px(250.0),
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                bottom: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::linear_rgba(0.0, 0.1, 0.0, 0.8)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("📝 Event Log\n\nInitializing..."),
                TextColor(Color::linear_rgb(0.0, 1.0, 0.0)),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                EventLogPanel,
            ));
        });

    // Metadata panel (right side, initially hidden)
    commands
        .spawn((
            Node {
                width: Val::Px(400.0),
                height: Val::Px(500.0),
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                top: Val::Px(220.0),
                padding: UiRect::all(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::linear_rgba(0.1, 0.1, 0.1, 0.9)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Click a node to view metadata"),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                MetadataPanel,
            ));
        });

    event_log.add_message("✅ UI setup complete".to_string());
}

fn load_progress_graph(
    mut progress_data: ResMut<ProgressGraphData>,
    mut event_log: ResMut<EventLog>,
    mut animation_state: ResMut<GraphAnimationState>,
) {
    event_log.add_message("📖 Loading progress.json...".to_string());

    let progress_json = match fs::read_to_string("doc/progress/progress.json") {
        Ok(content) => {
            event_log.add_message(format!("✅ Read {} bytes from progress.json", content.len()));
            content
        },
        Err(e) => {
            event_log.add_message(format!("❌ Failed to read progress.json: {}", e));
            return;
        }
    };

    let progress: ProgressData = match serde_json::from_str::<ProgressData>(&progress_json) {
        Ok(data) => {
            event_log.add_message("✅ Successfully parsed JSON data".to_string());
            event_log.add_message(format!("📋 Project: {} v{}", data.metadata.name, data.metadata.version));
            data
        },
        Err(e) => {
            event_log.add_message(format!("❌ Failed to parse progress.json: {}", e));
            return;
        }
    };

    event_log.add_message(format!("📊 Found {} nodes and {} edges", progress.nodes.len(), progress.edges.len()));

    progress_data.node_count = progress.nodes.len();
    progress_data.edge_count = progress.edges.len();
    progress_data.project_info = Some(format!(
        "{}\nVersion: {}",
        progress.metadata.name,
        progress.metadata.version
    ));

    event_log.add_message(format!("🏗️ Creating ContextGraph: {}", progress.metadata.name));
    let mut context_graph = ContextGraph::new("Progress Graph");

    // Prepare nodes for animation
    event_log.add_message("🌟 Preparing graph layout for animation...".to_string());
    
    let mut node_positions = HashMap::new();
    let mut node_types = HashMap::new();
    let mut node_statuses = HashMap::new();
    
    // Group nodes by type and status for better positioning
    let mut milestones = Vec::new();
    let mut phases = Vec::new();
    let mut tasks = Vec::new();
    
    for node in &progress.nodes {
        let label = node.label.as_ref()
            .unwrap_or(&node.id)
            .clone();
        
        let node_type = node.node_type.as_ref()
            .unwrap_or(&"unknown".to_string())
            .clone();
            
        let node_data = if let Some(data) = &node.data {
            data.clone()
        } else {
            ProgressNodeData {
                status: node.status.clone().unwrap_or("unknown".to_string()),
                description: node.description.clone().unwrap_or("No description".to_string()),
                details: vec![],
                progress: None,
                date: None,
                completed_date: None,
                target_date: None,
                week: None,
                parent: None,
                references: node.outputs.clone(),
            }
        };

        node_types.insert(node.id.clone(), node_type.clone());
        node_statuses.insert(node.id.clone(), node_data.status.clone());
        
        // Use existing position if available, otherwise we'll calculate it
        if let Some(pos) = &node.position {
            node_positions.insert(node.id.clone(), Vec3::new(pos.x * 0.01, pos.y * 0.01, pos.z * 0.01));
        }
        
        // Group by type for layout
        match node_type.as_str() {
            "milestone" => milestones.push((node.id.clone(), label, node_data)),
            "phase" => phases.push((node.id.clone(), label, node_data)),
            _ => tasks.push((node.id.clone(), label, node_data)),
        }
    }
    
    // Create proper graph layout using edge relationships and semantic weights
    let mut edge_weights = HashMap::new();
    
    // First pass: Calculate edge weights based on relationship types
    event_log.add_message("🔗 Calculating semantic edge weights...".to_string());
    for edge in &progress.edges {
        let weight = match edge.relationship.as_str() {
            "sequence" => 1.0,        // Strong sequential relationship
            "leads_to" => 0.8,        // Strong causal relationship  
            "enables" => 0.7,         // Medium enablement relationship
            "implements" => 0.9,      // Strong implementation relationship
            "depends_on" => 0.8,      // Strong dependency
            "triggers" => 0.6,        // Medium trigger relationship
            "requires" => 0.7,        // Medium requirement relationship
            "expanded_to" => 0.5,     // Weaker expansion relationship
            "corrected_by" => 0.4,    // Correction relationship
            _ => 0.3,                 // Default weak relationship
        };
        edge_weights.insert((edge.source.clone(), edge.target.clone()), weight);
    }
    event_log.add_message(format!("📊 Calculated {} edge weights", edge_weights.len()));
    
    // Initialize node positions using semantic clustering
    let center = Vec3::ZERO;
    let radius = 2000.0;
    
    // Group nodes by type for initial positioning
    for (i, (id, _label, data)) in milestones.iter().enumerate() {
        let angle = (i as f32 / milestones.len() as f32) * 2.0 * std::f32::consts::PI;
        
        // Use data to influence positioning - completed milestones get priority positioning
        let radius_modifier = if data.status == "completed" { 0.8 } else { 1.0 };
        let height_modifier = if data.status == "completed" { 1.2 } else { 1.0 };
        
        let pos = Vec3::new(
            center.x + radius * radius_modifier * angle.cos(),
            center.y + 1000.0 * height_modifier, // Higher for milestones, even higher for completed
            center.z + radius * radius_modifier * angle.sin(),
        );
        node_positions.insert(id.clone(), pos);
        
        event_log.add_message(format!("📍 Positioned milestone: {} ({})", id, data.status));
    }
    
    for (i, (id, _label, data)) in phases.iter().enumerate() {
        let angle = (i as f32 / phases.len() as f32) * 2.0 * std::f32::consts::PI;
        
        // Use data to influence positioning - completed phases closer to center
        let radius_modifier = if data.status == "completed" { 0.5 } else { 0.7 };
        let height_modifier = if data.status == "in-progress" { 0.5 } else { 0.0 };
        
        let pos = Vec3::new(
            center.x + (radius * radius_modifier) * angle.cos(),
            center.y + 200.0 * height_modifier, // Slight elevation for in-progress phases
            center.z + (radius * radius_modifier) * angle.sin(),
        );
        node_positions.insert(id.clone(), pos);
        
        event_log.add_message(format!("📍 Positioned phase: {} ({})", id, data.status));
    }
    
    for (i, (id, _label, data)) in tasks.iter().enumerate() {
        let angle = (i as f32 / tasks.len() as f32) * 2.0 * std::f32::consts::PI;
        
        // Use data to influence positioning - tasks with higher progress closer to center
        let progress_factor = data.progress.unwrap_or(0.0) / 100.0;
        let radius_modifier = 0.4 + (1.0 - progress_factor) * 0.3; // Higher progress = closer to center
        let height_modifier = if data.status == "in-progress" { -0.3 } else { -1.0 };
        
        let pos = Vec3::new(
            center.x + (radius * radius_modifier) * angle.cos(),
            center.y + 500.0 * height_modifier, // Lower for tasks, less low for in-progress
            center.z + (radius * radius_modifier) * angle.sin(),
        );
        node_positions.insert(id.clone(), pos);
        
        event_log.add_message(format!("📍 Positioned task: {} ({}, {:.0}% progress)", 
            id, data.status, data.progress.unwrap_or(0.0)));
    }
    
    // Apply force-directed layout algorithm using edge weights
    event_log.add_message("⚡ Applying force-directed layout with semantic weights...".to_string());
    let iterations = 50;
    let k = 500.0; // Ideal edge length
    let c1 = 2.0;  // Repulsion constant
    let c2 = 1.0;  // Attraction constant
    
    for iteration in 0..iterations {
        let mut forces = HashMap::new();
        
        // Initialize forces
        for id in node_positions.keys() {
            forces.insert(id.clone(), Vec3::ZERO);
        }
        
        // Calculate repulsive forces between all nodes
        for (id1, pos1) in &node_positions {
            for (id2, pos2) in &node_positions {
                if id1 != id2 {
                    let diff = *pos1 - *pos2;
                    let distance = diff.length().max(1.0);
                    let repulsion = diff.normalize() * (c1 * k * k) / distance;
                    *forces.get_mut(id1).unwrap() += repulsion;
                }
            }
        }
        
        // Calculate attractive forces for connected nodes using edge weights
        for ((source, target), weight) in &edge_weights {
            if let (Some(pos1), Some(pos2)) = (node_positions.get(source), node_positions.get(target)) {
                let diff = *pos2 - *pos1;
                let distance = diff.length().max(1.0);
                let ideal_distance = k / weight; // Closer for stronger relationships
                let attraction = diff.normalize() * c2 * weight * (distance - ideal_distance);
                
                *forces.get_mut(source).unwrap() += attraction;
                *forces.get_mut(target).unwrap() -= attraction;
            }
        }
        
        // Apply forces with cooling
        let cooling = 1.0 - (iteration as f32 / iterations as f32);
        let max_displacement = 100.0 * cooling;
        
        for (id, force) in forces {
            if let Some(pos) = node_positions.get_mut(&id) {
                let displacement = force.clamp_length_max(max_displacement);
                *pos += displacement;
            }
        }
    }
    
    event_log.add_message("✅ Force-directed layout complete using edge semantics".to_string());
    
    // Prepare node data for animation
    let mut nodes_to_spawn = Vec::new();
    for node in progress.nodes.iter().take(50) {
        if let Some(position) = node_positions.get(&node.id) {
            let label = node.label.as_ref().unwrap_or(&node.id).clone();
            let node_type = node_types.get(&node.id).unwrap_or(&"unknown".to_string()).clone();
            let status = node_statuses.get(&node.id).unwrap_or(&"unknown".to_string()).clone();
            
            let node_data = if let Some(data) = &node.data {
                data.clone()
            } else {
                ProgressNodeData {
                    status: status.clone(),
                    description: node.description.clone().unwrap_or("No description".to_string()),
                    details: vec![],
                    progress: None,
                    date: None,
                    completed_date: None,
                    target_date: None,
                    week: None,
                    parent: None,
                    references: node.outputs.clone(),
                }
            };

            let component = ProgressNodeComponent {
                node_id: node.id.clone(),
                label: label.clone(),
                node_type: node_type.clone(),
                data: node_data.clone(),
            };
            
            nodes_to_spawn.push((node.id.clone(), *position, component));
            
            context_graph.add_node(serde_json::json!({
                "id": node.id,
                "label": label,
                "type": node_type,
                "status": node_data.status,
                "description": node_data.description
            }));
        }
    }
    
    // Prepare edge data for animation
    let mut edges_to_spawn = Vec::new();
    for edge in &progress.edges {
        let component = ProgressEdgeComponent {
            edge_id: edge.id.clone(),
            edge_type: edge.relationship.clone(),
            label: edge.label.clone().unwrap_or_else(|| edge.relationship.clone()),
        };
        edges_to_spawn.push((edge.source.clone(), edge.target.clone(), component));
    }
    
    // Set up animation state
    animation_state.nodes_to_spawn = nodes_to_spawn;
    animation_state.edges_to_spawn = edges_to_spawn;
    animation_state.node_positions = node_positions;
    animation_state.phase = AnimationPhase::SpawningNodes;
    
    progress_data.graph = Some(context_graph);
    event_log.add_message("🎬 Animation prepared! Starting graph construction...".to_string());
    event_log.add_message("💡 Use WASD + mouse to navigate, right-click and drag to look around".to_string());
}

fn update_event_log_panel(
    mut text_query: Query<&mut Text, With<EventLogPanel>>,
    event_log: Res<EventLog>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        **text = event_log.get_display_text();
    }
}

fn camera_controller(
    mut camera_query: Query<(&mut Transform, &CameraController), With<Camera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut mouse_motion: EventReader<MouseMotion>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, controller)) = camera_query.single_mut() {
        let mut translation_delta = Vec3::ZERO;
        
        // Check for speed modifiers (only Ctrl for slow movement)
        let speed_multiplier = if keyboard_input.pressed(KeyCode::ControlLeft) || keyboard_input.pressed(KeyCode::ControlRight) {
            0.3 // Slower movement with ctrl
        } else {
            1.0 // Normal speed
        };
        
        let effective_speed = controller.speed * speed_multiplier;

        // WASD movement with fixed delta time
        if keyboard_input.pressed(KeyCode::KeyW) {
            let forward = transform.forward().as_vec3();
            translation_delta += forward * effective_speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            let back = transform.back().as_vec3();
            translation_delta += back * effective_speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            let left = transform.left().as_vec3();
            translation_delta += left * effective_speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            let right = transform.right().as_vec3();
            translation_delta += right * effective_speed * time.delta_secs();
        }

        // Apply movement
        transform.translation += translation_delta;

        // Mouse wheel zoom with clamping
        for wheel in mouse_wheel.read() {
            let zoom_delta = wheel.y * 200.0; // Much faster zoom for massive space
            let forward = transform.forward().as_vec3(); // Store forward vector first
            transform.translation += forward * zoom_delta;
            
            // Clamp camera distance to prevent going too close or too far
            let distance_from_origin = transform.translation.length();
            if distance_from_origin < 1000.0 { // Much larger minimum distance
                transform.translation = transform.translation.normalize() * 1000.0;
            } else if distance_from_origin > 20000.0 { // Much larger maximum distance
                transform.translation = transform.translation.normalize() * 20000.0;
            }
        }

        // Mouse look (only when right mouse button is held) with sensitivity control
        if mouse_button_input.pressed(MouseButton::Right) {
            for delta in mouse_motion.read() {
                // Clamp mouse delta to prevent excessive rotation
                let clamped_delta_x = delta.delta.x.clamp(-10.0, 10.0);
                let clamped_delta_y = delta.delta.y.clamp(-10.0, 10.0);
                
                // Yaw (horizontal rotation)
                let yaw = -clamped_delta_x * 0.001 * controller.sensitivity;
                transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(yaw));
                
                // Pitch (vertical rotation) - limit to prevent flipping
                let right = (transform.rotation * Vec3::X).normalize();
                let current_up = transform.rotation * Vec3::Y;
                let pitch = -clamped_delta_y * 0.001 * controller.sensitivity;
                
                // Limit pitch to prevent camera flipping
                if current_up.y > 0.1 || pitch < 0.0 {
                    let pitch_rotation = Quat::from_axis_angle(right, pitch);
                    transform.rotation = pitch_rotation * transform.rotation;
                }
            }
        }
    }
}

fn update_info_panel(
    mut text_query: Query<&mut Text, With<InfoPanel>>,
    progress_data: Res<ProgressGraphData>,
    search_state: Res<SearchState>,
    filter_state: Res<FilterState>,
    path_vis: Res<PathVisualization>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        if let Some(ref project_info) = progress_data.project_info {
            let search_status = if search_state.is_active { "🔍 ACTIVE" } else { "🔍 Inactive" };
            let filter_status = if filter_state.show_all { "🔽 Show All" } else { "🔼 Filtered" };
            let path_status = if path_vis.is_active { "🛤️ ACTIVE" } else { "🛤️ Inactive" };

            **text = format!(
                "Progress Graph Visualization - Enhanced\n\n{}\n\nNodes: {}\nEdges: {}\n\n📋 CONTROLS:\n- WASD: Move camera\n- Ctrl + WASD: Slow movement\n- Mouse wheel: Zoom\n- Right click + drag: Look around\n- Left click: Select node\n- R: Reset animation\n\n🔍 SEARCH & FILTER:\n- S: Toggle search mode ({})\n- F: Toggle filter ({})\n- 1,2,3: Quick filters (in search mode)\n\n🛤️ PATH VISUALIZATION:\n- P: Toggle path mode ({})\n- Space: Demo path highlight\n\n🎨 EDGE COLORS:\n🔵 leads_to (Blue)\n🟢 enables (Green)\n🟣 implemented_by (Purple)\n🟠 sequence (Orange)\n🟡 expanded (Yellow)\n🔴 corrected (Red)\n🩵 requires (Cyan)\n🩷 triggers (Magenta)\n⚪ other (Gray)",
                project_info,
                progress_data.node_count,
                progress_data.edge_count,
                search_status,
                filter_status,
                path_status
            );
        }
    }
}

fn handle_node_clicks(
    node_query: Query<(Entity, &ProgressNodeComponent, &Transform), With<Clickable>>,
    camera_query: Query<(&Transform, &Camera), With<Camera>>,
    windows: Query<&Window>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut selected_node: ResMut<SelectedNode>,
    mut metadata_panel_query: Query<(&mut Node, &mut Text), With<MetadataPanel>>,
    mut event_log: ResMut<EventLog>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Ok(window) = windows.single() {
            if let Some(cursor_position) = window.cursor_position() {
                if let Ok((camera_transform, camera)) = camera_query.single() {
                    // Convert cursor position to normalized device coordinates
                    let window_size = Vec2::new(window.width(), window.height());
                    let ndc = (cursor_position / window_size) * 2.0 - Vec2::ONE;
                    let ndc = Vec2::new(ndc.x, -ndc.y); // Flip Y coordinate
                    
                    // Create ray from camera in world space
                    let ray_origin = camera_transform.translation;
                    let ray_direction = {
                        // Create direction vector from camera through cursor position
                        let forward = -camera_transform.forward().as_vec3();
                        let right = camera_transform.right().as_vec3();
                        let up = camera_transform.up().as_vec3();
                        
                        // Apply field of view and aspect ratio (simplified)
                        let fov_factor = 1.0; // Could be calculated from camera projection
                        let aspect_ratio = window_size.x / window_size.y;
                        
                        (forward + right * ndc.x * fov_factor * aspect_ratio + up * ndc.y * fov_factor).normalize()
                    };

                    // Find closest intersected node
                    let mut closest_distance = f32::INFINITY;
                    let mut selected_entity = None;

                    for (entity, node_component, transform) in node_query.iter() {
                        let node_position = transform.translation;
                        let to_node = node_position - ray_origin;
                        
                        // Project onto ray direction to find closest point on ray
                        let projection_length = to_node.dot(ray_direction);
                        if projection_length > 0.0 {
                            let closest_point_on_ray = ray_origin + ray_direction * projection_length;
                            let distance_to_ray = (node_position - closest_point_on_ray).length();
                            
                            // Use a selection radius of 50 units
                            if distance_to_ray < 50.0 && projection_length < closest_distance {
                                closest_distance = projection_length;
                                selected_entity = Some((entity, node_component));
                            }
                        }
                    }

                    if let Some((entity, node_component)) = selected_entity {
                        selected_node.selected_entity = Some(entity);
                        selected_node.node_label = Some(node_component.label.clone());
                        
                        event_log.add_message(format!("🎯 Selected: {}", node_component.label));
                        
                        // Update metadata panel
                        if let Ok((mut style, mut text)) = metadata_panel_query.single_mut() {
                            style.display = Display::Flex;
                            
                            **text = format!(
                                "📊 Node Details\n\n🏷️ Label: {}\n📋 Type: {}\n📈 Status: {}\n⏰ Created: {}\n📝 Description: {}\n🔗 Details: {}\n📚 References: {}",
                                node_component.label,
                                node_component.node_type,
                                node_component.data.status,
                                node_component.data.date.as_ref().unwrap_or(&"Unknown".to_string()),
                                node_component.data.description,
                                if node_component.data.details.is_empty() { 
                                    "No details".to_string() 
                                } else { 
                                    node_component.data.details.join(", ") 
                                },
                                node_component.data.references.as_ref()
                                    .map(|refs| if refs.is_empty() { 
                                        "None".to_string() 
                                    } else { 
                                        refs.join(", ") 
                                    })
                                    .unwrap_or_else(|| "None".to_string())
                            );
                        }
                    } else {
                        // Clear selection
                        selected_node.selected_entity = None;
                        selected_node.node_label = None;
                        
                        if let Ok((mut style, _)) = metadata_panel_query.single_mut() {
                            style.display = Display::None;
                        }
                    }
                }
            }
        }
    }
}

// Animation systems
fn animate_graph_construction(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut animation_state: ResMut<GraphAnimationState>,
    mut event_log: ResMut<EventLog>,
    time: Res<Time>,
) {
    animation_state.spawn_timer += time.delta_secs();
    animation_state.layout_timer += time.delta_secs();
    
    match animation_state.phase {
        AnimationPhase::SpawningNodes => {
            // Spawn nodes much faster (10 per second instead of 2)
            if animation_state.spawn_timer > 0.1 && animation_state.node_spawn_index < animation_state.nodes_to_spawn.len() {
                let (node_id, position, component) = animation_state.nodes_to_spawn[animation_state.node_spawn_index].clone();
                
                // Use the get_node_color function instead of hardcoded colors
                let node_color = get_node_color(&component.node_type, &component.data.status);
                
                // Create meshes based on node type with much larger sizes for massive distances
                let (mesh, material) = match component.node_type.as_str() {
                    "milestone" => {
                        let mesh = meshes.add(Sphere::new(50.0)); // Much larger for massive distances
                        let material = materials.add(StandardMaterial {
                            base_color: node_color,
                            metallic: 0.3,
                            perceptual_roughness: 0.5,
                            ..default()
                        });
                        (mesh, material)
                    },
                    "phase" => {
                        let mesh = meshes.add(Cuboid::new(60.0, 60.0, 60.0)); // Much larger
                        let material = materials.add(StandardMaterial {
                            base_color: node_color,
                            metallic: 0.2,
                            perceptual_roughness: 0.6,
                            ..default()
                        });
                        (mesh, material)
                    },
                    _ => { // tasks and others
                        let mesh = meshes.add(Cylinder::new(30.0, 50.0)); // Much larger
                        let material = materials.add(StandardMaterial {
                            base_color: node_color,
                            metallic: 0.1,
                            perceptual_roughness: 0.7,
                            ..default()
                        });
                        (mesh, material)
                    }
                };

                // Spawn node with basic components first
                let entity = commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(position).with_scale(Vec3::splat(0.1)),
                )).id();
                
                // Add remaining components separately
                commands.entity(entity).insert(component.clone());
                commands.entity(entity).insert(Clickable);

                // Add animation components separately
                commands.entity(entity).insert((
                    AnimatedNode {
                        target_position: position,
                        spawn_time: time.elapsed_secs(),
                        animation_duration: 0.3,
                    },
                    FadingIn {
                        start_time: time.elapsed_secs(),
                        duration: 0.2,
                        start_alpha: 0.0,
                        end_alpha: 1.0,
                    },
                ));

                // Add text label with much larger size for massive distances
                commands.spawn((
                    Text2d::new(&component.label),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                    TextFont {
                        font_size: 120.0, // Much larger for massive distances
                        ..default()
                    },
                    Transform::from_translation(position + Vec3::new(0.0, 80.0, 0.0)), // Larger offset for bigger nodes
                    FadingIn {
                        start_time: time.elapsed_secs() + 0.1,
                        duration: 0.2,
                        start_alpha: 0.0,
                        end_alpha: 1.0,
                    },
                ));

                animation_state.node_entities.insert(node_id.clone(), entity);
                animation_state.node_spawn_index += 1;
                animation_state.spawn_timer = 0.0;
                
                if animation_state.node_spawn_index % 5 == 0 {
                    event_log.add_message(format!("🌟 Spawned {} nodes...", animation_state.node_spawn_index));
                }
            }
            
            // Move to next phase faster
            if animation_state.node_spawn_index >= animation_state.nodes_to_spawn.len() {
                animation_state.phase = AnimationPhase::SpawningEdges;
                animation_state.spawn_timer = 0.0;
                event_log.add_message("🔗 Starting edge animation phase...".to_string());
            }
        },
        
        AnimationPhase::AnimatingLayout => {
            // Skip force-directed layout since it causes overlap - go straight to completion
            animation_state.phase = AnimationPhase::Complete;
            event_log.add_message("🎉 Graph construction complete!".to_string());
            event_log.add_message("👆 Click nodes to view details".to_string());
            event_log.add_message("🔄 Press 'R' to reset animation".to_string());
        },
        
        AnimationPhase::SpawningEdges => {
            // Spawn edges faster (20 per second)
            if animation_state.spawn_timer > 0.05 && animation_state.edge_spawn_index < animation_state.edges_to_spawn.len() {
                let (source_id, target_id, component) = &animation_state.edges_to_spawn[animation_state.edge_spawn_index];
                
                if let (Some(&source_entity), Some(&target_entity)) = 
                    (animation_state.node_entities.get(source_id), animation_state.node_entities.get(target_id)) {
                    
                    if let (Some(source_pos), Some(target_pos)) = 
                        (animation_state.node_positions.get(source_id), animation_state.node_positions.get(target_id)) {
                        
                        let distance = source_pos.distance(*target_pos);
                        let center = (*source_pos + *target_pos) / 2.0;
                        let direction = (*target_pos - *source_pos).normalize();
                        let rotation = Quat::from_rotation_arc(Vec3::Y, direction);

                        // Create very thick cylinder for edge visibility across massive distances
                        let edge_mesh = meshes.add(Cylinder::new(15.0, distance)); // Much thicker for massive distances
                        
                        // Use the get_edge_color function instead of hardcoded colors
                        let edge_color = get_edge_color(&component.edge_type);
                        
                        let edge_material = materials.add(StandardMaterial {
                            base_color: edge_color,
                            metallic: 0.1,
                            perceptual_roughness: 0.8,
                            alpha_mode: AlphaMode::Opaque, // Make fully opaque
                            ..default()
                        });
                        
                        let edge_entity = commands.spawn((
                            Mesh3d(edge_mesh),
                            MeshMaterial3d(edge_material),
                            Transform::from_translation(center)
                                .with_rotation(rotation)
                                .with_scale(Vec3::new(0.0, 1.0, 0.0)),
                        )).id();
                        
                        // Add edge component separately
                        commands.entity(edge_entity).insert(component.clone());
                        
                        // Add animation component separately
                        commands.entity(edge_entity).insert(AnimatedEdge {
                            spawn_time: time.elapsed_secs(),
                            animation_duration: 0.3,
                            target_scale: Vec3::ONE,
                        });

                        // Create a visual connection by storing references to source and target entities
                        commands.entity(edge_entity).insert(EdgeConnection {
                            source_entity,
                            target_entity,
                        });
                        
                        event_log.add_message(format!("🔗 Connected {} -> {} via {}", 
                            source_id, target_id, component.edge_type));
                    }
                }
                
                animation_state.edge_spawn_index += 1;
                animation_state.spawn_timer = 0.0;
                
                if animation_state.edge_spawn_index % 10 == 0 {
                    event_log.add_message(format!("⚡ Spawned {} edges...", animation_state.edge_spawn_index));
                }
            }
            
            // Move to layout phase when all edges are spawned
            if animation_state.edge_spawn_index >= animation_state.edges_to_spawn.len() {
                animation_state.phase = AnimationPhase::AnimatingLayout;
                animation_state.layout_timer = 0.0;
                event_log.add_message("🎯 Starting force-directed layout animation...".to_string());
            }
        },
        
        AnimationPhase::Complete => {
            // Animation finished
        }
    }
}

fn animate_node_movement(
    mut node_query: Query<(&mut Transform, &AnimatedNode)>,
    time: Res<Time>,
) {
    for (mut transform, animated) in node_query.iter_mut() {
        let elapsed = time.elapsed_secs() - animated.spawn_time;
        let progress = (elapsed / animated.animation_duration).clamp(0.0, 1.0);
        
        // Smooth easing function
        let eased_progress = progress * progress * (3.0 - 2.0 * progress);
        
        // Scale animation (grow from small to normal size)
        let scale = 0.1 + eased_progress * 0.9;
        transform.scale = Vec3::splat(scale);
    }
}

fn animate_fade_in(
    mut text_query: Query<&mut TextColor, With<FadingIn>>,
    fade_query: Query<(Entity, &FadingIn)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, fade) in fade_query.iter() {
        let elapsed = time.elapsed_secs() - fade.start_time;
        let progress = (elapsed / fade.duration).clamp(0.0, 1.0);
        
        let current_alpha = fade.start_alpha + (fade.end_alpha - fade.start_alpha) * progress;
        
        // Update text color alpha
        if let Ok(mut text_color) = text_query.get_mut(entity) {
            text_color.0 = text_color.0.with_alpha(current_alpha);
        }
        
        // Remove fade component when complete
        if progress >= 1.0 {
            commands.entity(entity).remove::<FadingIn>();
        }
    }
}

fn animate_edge_growth(
    mut edge_query: Query<(&mut Transform, &AnimatedEdge)>,
    time: Res<Time>,
) {
    for (mut transform, animated) in edge_query.iter_mut() {
        let elapsed = time.elapsed_secs() - animated.spawn_time;
        let progress = (elapsed / animated.animation_duration).clamp(0.0, 1.0);
        
        // Smooth easing
        let eased_progress = progress * progress * (3.0 - 2.0 * progress);
        
        // Scale from 0 to target scale
        let current_scale = Vec3::lerp(Vec3::new(0.0, 1.0, 0.0), animated.target_scale, eased_progress);
        transform.scale = current_scale;
    }
}

// Reset system
fn handle_reset_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    entities: Query<Entity, Or<(With<ProgressNodeComponent>, With<ProgressEdgeComponent>, With<Text2d>)>>,
    mut animation_state: ResMut<GraphAnimationState>,
    mut event_log: ResMut<EventLog>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        event_log.add_message("🔄 Resetting animation...".to_string());
        
        // Despawn all graph entities
        for entity in entities.iter() {
            commands.entity(entity).despawn();
        }
        
        // Reset animation state
        animation_state.phase = AnimationPhase::SpawningNodes;
        animation_state.node_spawn_index = 0;
        animation_state.edge_spawn_index = 0;
        animation_state.spawn_timer = 0.0;
        animation_state.layout_timer = 0.0;
        animation_state.layout_iterations = 0;
        animation_state.node_entities.clear();
        
        // Reset camera position
        if let Ok(mut camera_transform) = camera_query.single_mut() {
            *camera_transform = Transform::from_xyz(0.0, 3000.0, 8000.0).looking_at(Vec3::ZERO, Vec3::Y);
        }
        
        event_log.add_message("✅ Animation reset! Starting fresh...".to_string());
    }
}

fn highlight_connected_edges(
    selected_node: Res<SelectedNode>,
    edge_query: Query<(Entity, &ProgressEdgeComponent, &EdgeConnection)>,
    node_query: Query<(Entity, &ProgressNodeComponent)>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut event_log: ResMut<EventLog>,
) {
    // Only highlight when a node is actually selected
    if let Some(ref selected_label) = selected_node.node_label {
        // Find the selected node entity
        let selected_entity = node_query.iter()
            .find(|(_, component)| component.label == *selected_label)
            .map(|(entity, _)| entity);
        
        if let Some(selected_entity) = selected_entity {
            let mut highlighted_count = 0;
            
            // Highlight all edges connected to the selected node
            for (edge_entity, edge_component, edge_connection) in edge_query.iter() {
                let is_connected = edge_connection.source_entity == selected_entity 
                    || edge_connection.target_entity == selected_entity;
                
                if is_connected {
                    // Get the material handle and brighten it
                    if let Ok(material_handle) = material_query.get(edge_entity) {
                        if let Some(material) = materials.get_mut(&material_handle.0) {
                            // Use Bevy v0.16 color API - create brighter color
                            let current_color = material.base_color;
                            
                            // Convert to linear RGB, brighten, then convert back
                            let brightened = match current_color {
                                Color::LinearRgba(rgba) => {
                                    Color::LinearRgba(LinearRgba {
                                        red: (rgba.red + 0.3).min(1.0),
                                        green: (rgba.green + 0.3).min(1.0),
                                        blue: (rgba.blue + 0.3).min(1.0),
                                        alpha: rgba.alpha,
                                    })
                                },
                                Color::Srgba(srgba) => {
                                    // Convert sRGB to linear RGB for calculations
                                    Color::LinearRgba(LinearRgba {
                                        red: (srgba.red.powf(2.2) + 0.3).min(1.0),
                                        green: (srgba.green.powf(2.2) + 0.3).min(1.0),
                                        blue: (srgba.blue.powf(2.2) + 0.3).min(1.0),
                                        alpha: srgba.alpha,
                                    })
                                },
                                _ => {
                                    // For other color spaces, convert to linear RGB first
                                    let linear = current_color.to_linear();
                                    Color::LinearRgba(LinearRgba {
                                        red: (linear.red + 0.3).min(1.0),
                                        green: (linear.green + 0.3).min(1.0),
                                        blue: (linear.blue + 0.3).min(1.0),
                                        alpha: linear.alpha,
                                    })
                                }
                            };
                            
                            material.base_color = brightened;
                            material.emissive = brightened.to_linear();
                            highlighted_count += 1;
                        }
                    }
                }
            }
            
            if highlighted_count > 0 {
                event_log.add_message(format!("✨ Highlighted {} connected edges", highlighted_count));
            }
        }
    } else {
        // Reset all edge colors when nothing is selected
        for (_edge_entity, edge_component, _edge_connection) in edge_query.iter() {
            // Reset colors based on edge type
            if let Ok(material_handle) = material_query.get(_edge_entity) {
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    material.base_color = get_edge_color(&edge_component.edge_type);
                    material.emissive = LinearRgba::BLACK;
                }
            }
        }
    }
}

fn handle_search_input(
    mut search_state: ResMut<SearchState>,
    mut filter_state: ResMut<FilterState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut event_log: ResMut<EventLog>,
    node_query: Query<(Entity, &ProgressNodeComponent)>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        search_state.is_active = !search_state.is_active;
        if search_state.is_active {
            event_log.add_message("🔍 Search mode activated - ESC to exit".to_string());
        } else {
            event_log.add_message("🔍 Search mode deactivated".to_string());
        }
    }

    if keyboard_input.just_pressed(KeyCode::KeyF) {
        filter_state.show_all = !filter_state.show_all;
        if filter_state.show_all {
            event_log.add_message("🔽 Showing all nodes".to_string());
        } else {
            // Apply basic filters
            filter_state.statuses.insert("completed".to_string());
            filter_state.statuses.insert("in-progress".to_string());
            event_log.add_message("🔼 Applying status filters".to_string());
        }
    }

    if keyboard_input.just_pressed(KeyCode::Digit1) && search_state.is_active {
        // Quick filter: show only completed
        filter_state.statuses.clear();
        filter_state.statuses.insert("completed".to_string());
        event_log.add_message("✅ Filtering: Completed only".to_string());
    }

    if keyboard_input.just_pressed(KeyCode::Digit2) && search_state.is_active {
        // Quick filter: show only in-progress
        filter_state.statuses.clear();
        filter_state.statuses.insert("in-progress".to_string());
        event_log.add_message("🔄 Filtering: In-progress only".to_string());
    }

    if keyboard_input.just_pressed(KeyCode::Digit3) && search_state.is_active {
        // Quick filter: show milestones only
        filter_state.node_types.clear();
        filter_state.node_types.insert("milestone".to_string());
        event_log.add_message("🎯 Filtering: Milestones only".to_string());
    }
}

fn apply_filters(
    filter_state: Res<FilterState>,
    search_state: Res<SearchState>,
    mut node_query: Query<(Entity, &ProgressNodeComponent, &mut Visibility)>,
    mut event_log: ResMut<EventLog>,
) {
    if !filter_state.is_changed() && !search_state.is_changed() {
        return;
    }

    let mut visible_count = 0;
    let mut hidden_count = 0;

    for (entity, node_component, mut visibility) in node_query.iter_mut() {
        let should_show = if filter_state.show_all {
            true
        } else {
            filter_state.is_filtered(&node_component.data) &&
            (filter_state.node_types.is_empty() || 
             filter_state.node_types.contains(&node_component.node_type))
        };

        if should_show {
            *visibility = Visibility::Visible;
            visible_count += 1;
        } else {
            *visibility = Visibility::Hidden;
            hidden_count += 1;
        }
    }

    if filter_state.is_changed() {
        event_log.add_message(format!("👁️ Filter applied: {} visible, {} hidden", 
            visible_count, hidden_count));
    }
}

fn visualize_paths(
    mut path_vis: ResMut<PathVisualization>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    node_query: Query<(Entity, &ProgressNodeComponent, &Transform)>,
    edge_query: Query<(Entity, &ProgressEdgeComponent, &EdgeConnection)>,
    mut commands: Commands,
    mut event_log: ResMut<EventLog>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        path_vis.is_active = !path_vis.is_active;
        if path_vis.is_active {
            event_log.add_message("🛤️ Path visualization activated - click two nodes".to_string());
        } else {
            event_log.add_message("🛤️ Path visualization deactivated".to_string());
            // Clear any existing path highlights
            for (entity, _, _) in node_query.iter() {
                commands.entity(entity).remove::<Highlighted>();
            }
        }
    }

    if path_vis.is_active && keyboard_input.just_pressed(KeyCode::Space) {
        // Demo: highlight a simple path through the first few nodes
        let nodes: Vec<_> = node_query.iter().take(5).collect();
        
        for (i, (entity, node_component, _)) in nodes.iter().enumerate() {
            commands.entity(*entity).insert(Highlighted);
            commands.entity(*entity).insert(PathNode {
                path_id: 1,
                order: i,
            });
        }

        event_log.add_message(format!("🛤️ Highlighted demo path with {} nodes", nodes.len()));
    }
}

fn update_search_panel(
    mut text_query: Query<&mut Text, With<SearchPanel>>,
    search_state: Res<SearchState>,
    filter_state: Res<FilterState>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        let filter_info = if filter_state.show_all {
            "All nodes visible".to_string()
        } else {
            format!("Filters active: {} types, {} statuses", 
                filter_state.node_types.len(),
                filter_state.statuses.len())
        };

        **text = format!(
            "🔍 Search & Filter\n\nPress 'S' to toggle search\nPress 'F' to toggle filters\nPress 'P' for path mode\n\nQuick filters (in search mode):\n1 - Completed only\n2 - In-progress only\n3 - Milestones only\n\nStatus: {}\n{}",
            if search_state.is_active { "SEARCH MODE" } else { "Normal" },
            filter_info
        );
    }
}

fn highlight_selected_nodes(
    selected_node: Res<SelectedNode>,
    node_query: Query<(Entity, &ProgressNodeComponent)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut event_log: ResMut<EventLog>,
) {
    if let Some(ref selected_label) = selected_node.node_label {
        let selected_entity = node_query.iter()
            .find(|(_, component)| component.label == *selected_label)
            .map(|(entity, _)| entity);
        
        if let Some(selected_entity) = selected_entity {
            let mut highlighted_count = 0;
            
            for (entity, node_component) in node_query.iter() {
                if entity == selected_entity {
                    if let Ok(material_handle) = material_query.get(entity) {
                        if let Some(material) = materials.get_mut(&material_handle.0) {
                            material.base_color = Color::linear_rgb(1.0, 0.0, 0.0);
                            material.emissive = Color::linear_rgb(1.0, 0.0, 0.0).to_linear() * 0.5;
                            highlighted_count += 1;
                        }
                    }
                } else {
                    if let Ok(material_handle) = material_query.get(entity) {
                        if let Some(material) = materials.get_mut(&material_handle.0) {
                            material.base_color = Color::linear_rgb(0.8, 0.8, 0.8);
                            material.emissive = Color::linear_rgb(0.8, 0.8, 0.8).to_linear() * 0.5;
                        }
                    }
                }
            }
            
            if highlighted_count > 0 {
                event_log.add_message(format!("💡 Highlighted {} selected nodes", highlighted_count));
            }
        }
    }
} 