use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin};
use std::collections::HashMap;
use uuid::Uuid;

// Import the new modules
mod camera;
mod events;
mod graph;
mod graph_core;
mod graph_patterns;
mod json_loader;
mod models;
mod theming;
mod unified_graph_editor;

use camera::CameraViewportPlugin;
use graph_core::{CreateEdgeEvent, CreateNodeEvent, DomainNodeType, GraphPlugin};
use graph_patterns::{GraphPattern, generate_pattern};
use json_loader::{
    FileOperationState, JsonGraphData, JsonNode, JsonPosition, LoadJsonFileEvent,
    SaveJsonFileEvent, json_to_base_graph, load_json_file, save_json_file,
};

#[derive(Resource, Default)]
struct NodeCounter(u32);

#[derive(Resource)]
struct PendingEdges {
    edges: Vec<PendingEdgeData>,
    node_count: usize,
    processed_nodes: usize,
}

struct PendingEdgeData {
    id: Uuid,
    source_uuid: Uuid,
    target_uuid: Uuid,
    edge_type: graph_core::DomainEdgeType,
    labels: Vec<String>,
    properties: std::collections::HashMap<String, String>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Alchemist Graph Editor".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: false,
        })
        // Add our custom plugins
        .add_plugins(CameraViewportPlugin)
        .add_plugins(GraphPlugin)
        // Resources
        .init_resource::<NodeCounter>()
        .init_resource::<FileOperationState>()
        // Events
        .add_event::<LoadJsonFileEvent>()
        .add_event::<SaveJsonFileEvent>()
        // Setup systems
        .add_systems(Startup, (setup, setup_file_scanner))
        .add_systems(
            Update,
            (
                // UI and input - run after EGUI initialization
                ui_system,
                debug_camera_system,
                keyboard_commands_system,
            )
                .chain()
                .after(bevy_egui::EguiPreUpdateSet::InitContexts),
        )
        .add_systems(
            Update,
            (
                // File operations and graph manipulation
                handle_load_json_file,
                handle_save_json_file,
                process_pending_edges,
            )
                .chain()
                .after(bevy_egui::EguiPreUpdateSet::InitContexts),
        )
        .add_systems(Last, track_node_despawns)
        .run();
}

/// Debug system to log camera state
fn debug_camera_system(
    camera_query: Query<&camera::GraphViewCamera>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyD) {
        if let Ok(camera) = camera_query.single() {
            info!("Camera Debug Info:");
            info!("  View Mode: {:?}", camera.view_mode);
            info!("  Transition Active: {}", camera.transition.active);
        }
    }

    // Log input state when mouse buttons are pressed
    if mouse_input.just_pressed(MouseButton::Left) {
        info!("Left mouse button pressed");
    }
    if mouse_input.just_pressed(MouseButton::Right) {
        info!("Right mouse button pressed");
    }
    if mouse_input.just_pressed(MouseButton::Middle) {
        info!("Middle mouse button pressed");
    }
}

/// Setup the scene with camera, lights, and initial graph nodes
fn setup(mut commands: Commands, mut create_node_events: EventWriter<CreateNodeEvent>) {
    // Spawn camera with our GraphViewCamera component
    commands.spawn((Camera3d::default(), camera::GraphViewCamera::default()));

    // Add lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            affects_lightmapped_mesh_diffuse: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ambient light for better visibility
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
        affects_lightmapped_meshes: true,
    });

    // Create some initial test nodes
    create_node_events.send(CreateNodeEvent {
        id: Uuid::new_v4(),
        position: Vec3::new(0.0, 0.0, 0.0),
        domain_type: DomainNodeType::Process,
        name: "Central Node".to_string(),
        labels: vec!["process".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
    });

    create_node_events.send(CreateNodeEvent {
        id: Uuid::new_v4(),
        position: Vec3::new(5.0, 0.0, 0.0),
        domain_type: DomainNodeType::Decision,
        name: "Decision Node".to_string(),
        labels: vec!["decision".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
    });

    create_node_events.send(CreateNodeEvent {
        id: Uuid::new_v4(),
        position: Vec3::new(-5.0, 0.0, 0.0),
        domain_type: DomainNodeType::Event,
        name: "Event Node".to_string(),
        labels: vec!["event".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
    });

    create_node_events.send(CreateNodeEvent {
        id: Uuid::new_v4(),
        position: Vec3::new(0.0, 0.0, 5.0),
        domain_type: DomainNodeType::Storage,
        name: "Storage Node".to_string(),
        labels: vec!["storage".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
    });

    create_node_events.send(CreateNodeEvent {
        id: Uuid::new_v4(),
        position: Vec3::new(0.0, 0.0, -5.0),
        domain_type: DomainNodeType::Interface,
        name: "Interface Node".to_string(),
        labels: vec!["interface".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
    });
}

/// Setup file scanner
fn setup_file_scanner(mut file_state: ResMut<FileOperationState>) {
    file_state.scan_models_directory();
}

/// Enhanced UI system with graph patterns and file loading
fn ui_system(
    mut contexts: EguiContexts,
    mut graph_state: ResMut<graph_core::GraphState>,
    camera_query: Query<&camera::GraphViewCamera>,
    mut create_node_events: EventWriter<CreateNodeEvent>,
    mut create_edge_events: EventWriter<CreateEdgeEvent>,
    mut load_json_events: EventWriter<LoadJsonFileEvent>,
    mut save_json_events: EventWriter<SaveJsonFileEvent>,
    mut file_state: ResMut<FileOperationState>,
    node_query: Query<(Entity, &graph_core::GraphNode)>,
    edge_query: Query<Entity, With<graph_core::GraphEdge>>,
    mut commands: Commands,
) {
    // Left panel with controls
    egui::SidePanel::left("controls")
        .default_width(300.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Graph Editor Controls");

            ui.separator();

            // View mode info
            if let Ok(camera) = camera_query.single() {
                ui.label("View Mode:");
                match camera.view_mode {
                    camera::ViewMode::ThreeD(_) => {
                        ui.label("🎲 3D View");
                        ui.label("Controls:");
                        ui.label("• Right Mouse: Orbit");
                        ui.label("• Middle Mouse: Pan (with Shift)");
                        ui.label("• Scroll: Zoom");
                        ui.label("• Tab/V: Switch to 2D");
                    }
                    camera::ViewMode::TwoD(_) => {
                        ui.label("📄 2D View");
                        ui.label("Controls:");
                        ui.label("• Middle Mouse: Pan");
                        ui.label("• Scroll: Zoom");
                        ui.label("• Tab/V: Switch to 3D");
                    }
                }
            }

            ui.separator();

            // Graph stats
            ui.label(format!("Nodes: {}", graph_state.node_count));
            ui.label(format!("Edges: {}", graph_state.edge_count));

            ui.separator();

            // Selected/Hovered Node Info
            if let Some(hovered_entity) = graph_state.hovered_entity {
                if let Ok((_, node)) = node_query.get(hovered_entity) {
                    ui.heading("Hovered Node:");
                    ui.label(format!("Name: {}", node.name));
                    ui.label(format!("Type: {:?}", node.domain_type));

                    if !node.labels.is_empty() {
                        ui.label("Labels:");
                        for label in &node.labels {
                            ui.label(format!("  • {}", label));
                        }
                    }

                    if !node.properties.is_empty() {
                        ui.label("Properties:");
                        for (key, value) in &node.properties {
                            ui.label(format!("  • {}: {}", key, value));
                        }
                    }
                    ui.separator();
                }
            }

            // Selected nodes info
            if !graph_state.selected_nodes.is_empty() {
                ui.heading(format!(
                    "Selected: {} nodes",
                    graph_state.selected_nodes.len()
                ));
                for entity in &graph_state.selected_nodes {
                    if let Ok((_, node)) = node_query.get(*entity) {
                        ui.collapsing(format!("📌 {}", node.name), |ui| {
                            ui.label(format!("Type: {:?}", node.domain_type));

                            if !node.labels.is_empty() {
                                ui.label("Labels:");
                                for label in &node.labels {
                                    ui.label(format!("  • {}", label));
                                }
                            }

                            if !node.properties.is_empty() {
                                ui.label("Properties:");
                                for (key, value) in &node.properties {
                                    ui.label(format!("  • {}: {}", key, value));
                                }
                            }
                        });
                    }
                }
                ui.separator();
            }

            // Graph Patterns section
            ui.heading("📐 Graph Patterns");

            ui.horizontal(|ui| {
                if ui.button("⭐ Star").clicked() {
                    add_pattern_to_graph(
                        GraphPattern::Star { points: 6 },
                        &mut create_node_events,
                        &mut create_edge_events,
                        &node_query,
                    );
                }
                if ui.button("🌳 Tree").clicked() {
                    add_pattern_to_graph(
                        GraphPattern::Tree {
                            branch_factor: 3,
                            depth: 3,
                        },
                        &mut create_node_events,
                        &mut create_edge_events,
                        &node_query,
                    );
                }
            });

            ui.horizontal(|ui| {
                if ui.button("🔄 Cycle").clicked() {
                    add_pattern_to_graph(
                        GraphPattern::Cycle { nodes: 5 },
                        &mut create_node_events,
                        &mut create_edge_events,
                        &node_query,
                    );
                }
                if ui.button("🔗 Complete").clicked() {
                    add_pattern_to_graph(
                        GraphPattern::Complete { nodes: 4 },
                        &mut create_node_events,
                        &mut create_edge_events,
                        &node_query,
                    );
                }
            });

            ui.horizontal(|ui| {
                if ui.button("📊 DAG").clicked() {
                    add_pattern_to_graph(
                        GraphPattern::DirectedAcyclicGraph {
                            levels: 3,
                            nodes_per_level: 2,
                        },
                        &mut create_node_events,
                        &mut create_edge_events,
                        &node_query,
                    );
                }
                if ui.button("🤖 Moore").clicked() {
                    add_pattern_to_graph(
                        GraphPattern::MooreMachine,
                        &mut create_node_events,
                        &mut create_edge_events,
                        &node_query,
                    );
                }
            });

            ui.horizontal(|ui| {
                if ui.button("🔷 Grid").clicked() {
                    add_pattern_to_graph(
                        GraphPattern::Grid {
                            width: 3,
                            height: 3,
                        },
                        &mut create_node_events,
                        &mut create_edge_events,
                        &node_query,
                    );
                }
                if ui.button("🎭 Bipartite").clicked() {
                    add_pattern_to_graph(
                        GraphPattern::Bipartite {
                            left_nodes: 3,
                            right_nodes: 3,
                            edge_density: 0.7,
                        },
                        &mut create_node_events,
                        &mut create_edge_events,
                        &node_query,
                    );
                }
            });

            ui.separator();

            // File operations
            ui.heading("📁 File Operations");

            if let Some(current_file) = &file_state.current_file_path {
                ui.label(format!(
                    "Current: {}",
                    current_file.split('/').last().unwrap_or("unknown")
                ));
            } else {
                ui.label("No file loaded");
            }

            ui.separator();

            // Available files
            ui.label("Available JSON files:");
            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    if file_state.available_files.is_empty() {
                        ui.label("No JSON files found");
                        if ui.button("🔄 Refresh").clicked() {
                            file_state.scan_models_directory();
                        }
                    } else {
                        for file_path in file_state.available_files.clone() {
                            let file_name = file_path.split('/').last().unwrap_or("unknown");
                            if ui.button(format!("📂 {}", file_name)).clicked() {
                                load_json_events.send(LoadJsonFileEvent {
                                    file_path: file_path.clone(),
                                });
                            }
                        }
                    }
                });

            ui.separator();

            // Save options
            ui.horizontal(|ui| {
                if ui.button("💾 Save").clicked() {
                    if let Some(current_file) = &file_state.current_file_path {
                        save_json_events.send(SaveJsonFileEvent {
                            file_path: current_file.clone(),
                        });
                    }
                }

                if ui.button("💾 Save As...").clicked() {
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let new_file = format!("assets/models/graph_{}.json", timestamp);
                    save_json_events.send(SaveJsonFileEvent {
                        file_path: new_file,
                    });
                }
            });

            ui.separator();

            // Add node button
            if ui.button("Add Random Node").clicked() {
                let counter = graph_state.node_count as f32;
                let angle = counter * 0.618; // Golden ratio for nice distribution
                let radius = 5.0 + (counter * 0.5).min(10.0);
                let x = angle.cos() * radius;
                let z = angle.sin() * radius;

                create_node_events.send(CreateNodeEvent {
                    id: Uuid::new_v4(),
                    position: Vec3::new(x, 0.0, z),
                    domain_type: match (counter as u32) % 5 {
                        0 => DomainNodeType::Process,
                        1 => DomainNodeType::Decision,
                        2 => DomainNodeType::Event,
                        3 => DomainNodeType::Storage,
                        _ => DomainNodeType::Interface,
                    },
                    name: format!("Node {}", graph_state.node_count + 1),
                    labels: vec!["process".to_string()],
                    properties: std::collections::HashMap::new(),
                    subgraph_id: None,
                });
            }

            // Clear graph button
            if ui.button("🗑️ Clear Graph").clicked() {
                // Clear the graph directly instead of sending an event
                for entity in &node_query {
                    commands.entity(entity.0).despawn_recursive();
                }
                for entity in &edge_query {
                    commands.entity(entity).despawn_recursive();
                }

                // Reset graph state
                graph_state.node_count = 0;
                graph_state.edge_count = 0;
                graph_state.selected_nodes.clear();
                graph_state.selected_edges.clear();
                graph_state.hovered_entity = None;

                info!("Graph cleared from UI");
            }

            ui.separator();

            // Help text
            ui.label("Keyboard shortcuts:");
            ui.label("• H: Show help");
            ui.label("• Ctrl+K: Clear graph");
            ui.label("• Ctrl+N: Add node");
        });
}

/// Helper function to add a pattern to the graph
fn add_pattern_to_graph(
    pattern: GraphPattern,
    create_node_events: &mut EventWriter<CreateNodeEvent>,
    _create_edge_events: &mut EventWriter<CreateEdgeEvent>,
    _existing_nodes: &Query<(Entity, &graph_core::GraphNode)>,
) {
    let pattern_graph = generate_pattern(pattern);

    info!(
        "Generating pattern with {} nodes",
        pattern_graph.nodes.len()
    );

    // Calculate offset to avoid overlapping with existing nodes
    let offset = Vec3::new(10.0, 0.0, 10.0);

    // Map old UUIDs to new entities
    let mut id_to_entity: std::collections::HashMap<Uuid, Entity> =
        std::collections::HashMap::new();

    // Create a simple layout for nodes if they don't have positions
    let node_count = pattern_graph.nodes.len();
    let mut node_index = 0;

    // First pass: create all nodes
    for (old_id, node) in &pattern_graph.nodes {
        let new_id = Uuid::new_v4();

        // Calculate position - try to get from properties first
        let position = if let (Some(x_str), Some(y_str)) =
            (node.properties.get("x_pos"), node.properties.get("y_pos"))
        {
            if let (Ok(x), Ok(y)) = (x_str.parse::<f32>(), y_str.parse::<f32>()) {
                Vec3::new(x / 20.0 + offset.x, 0.0, y / 20.0 + offset.z)
            } else {
                // Parse failed, use default layout
                let angle = node_index as f32 * 2.0 * std::f32::consts::PI / node_count as f32;
                let radius = 5.0;
                Vec3::new(
                    radius * angle.cos() + offset.x,
                    0.0,
                    radius * angle.sin() + offset.z,
                )
            }
        } else {
            // No position properties, arrange in a circle
            let angle = node_index as f32 * 2.0 * std::f32::consts::PI / node_count as f32;
            let radius = 5.0;
            Vec3::new(
                radius * angle.cos() + offset.x,
                0.0,
                radius * angle.sin() + offset.z,
            )
        };

        node_index += 1;

        info!(
            "Creating pattern node '{}' at position {:?}",
            node.name, position
        );

        create_node_events.send(CreateNodeEvent {
            id: new_id,
            position,
            domain_type: DomainNodeType::Process, // Default type
            name: node.name.clone(),
            labels: node.labels.clone(),
            properties: node.properties.clone(),
            subgraph_id: None,
        });

        // We'll need to wait for nodes to be created before we can get their entities
        // For now, just track the mapping
        id_to_entity.insert(*old_id, Entity::PLACEHOLDER);
    }

    info!(
        "Pattern generation complete, sent {} node events",
        node_index
    );

    // Note: Edge creation would need to happen after nodes are created
    // This is a limitation we'd need to address with a better event system
}

/// Keyboard commands system for clear controls
fn keyboard_commands_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut create_node_events: EventWriter<CreateNodeEvent>,
    mut commands: Commands,
    node_query: Query<Entity, With<graph_core::GraphNode>>,
    edge_query: Query<Entity, With<graph_core::GraphEdge>>,
    mut graph_state: ResMut<graph_core::GraphState>,
) {
    // Clear graph with Ctrl+K
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyK) {
        // Clear the graph directly
        for entity in &node_query {
            commands.entity(entity).despawn_recursive();
        }
        for entity in &edge_query {
            commands.entity(entity).despawn_recursive();
        }

        // Reset graph state
        graph_state.node_count = 0;
        graph_state.edge_count = 0;
        graph_state.selected_nodes.clear();
        graph_state.selected_edges.clear();
        graph_state.hovered_entity = None;

        info!("Clear graph command triggered");
    }

    // Add node at origin with Ctrl+N
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyN) {
        create_node_events.send(CreateNodeEvent {
            id: Uuid::new_v4(),
            position: Vec3::ZERO,
            domain_type: DomainNodeType::Process,
            name: "New Node".to_string(),
            labels: vec!["process".to_string()],
            properties: std::collections::HashMap::new(),
            subgraph_id: None,
        });
        info!("Add node command triggered");
    }

    // Print help with H
    if keyboard.just_pressed(KeyCode::KeyH) {
        info!("=== Keyboard Commands ===");
        info!("Tab/V: Switch between 2D/3D view");
        info!("Ctrl+K: Clear graph");
        info!("Ctrl+N: Add new node at origin");
        info!("D: Debug camera info");
        info!("H: Show this help");
        info!("=== Mouse Controls ===");
        info!("3D: Right-click drag to orbit, Middle+Shift to pan, Scroll to zoom");
        info!("2D: Middle-click drag to pan, Scroll to zoom");
    }
}

/// Handle file loading
fn handle_load_json_file(
    mut events: EventReader<LoadJsonFileEvent>,
    mut create_node_events: EventWriter<CreateNodeEvent>,
    mut file_state: ResMut<FileOperationState>,
    mut commands: Commands,
    node_query: Query<(Entity, &graph_core::GraphNode)>,
    edge_query: Query<Entity, With<graph_core::GraphEdge>>,
    mut graph_state: ResMut<graph_core::GraphState>,
) {
    for event in events.read() {
        info!("=== START LOADING FILE: {} ===", event.file_path);

        // Clear the graph immediately (synchronously)
        info!("Clearing existing graph before loading new data...");

        // Despawn all nodes
        let node_count = node_query.iter().count();
        for (entity, _) in &node_query {
            commands.entity(entity).despawn_recursive();
        }

        // Despawn all edges
        let edge_count = edge_query.iter().count();
        for entity in &edge_query {
            commands.entity(entity).despawn_recursive();
        }

        info!("Cleared {} nodes and {} edges", node_count, edge_count);

        // Reset graph state
        graph_state.node_count = 0;
        graph_state.edge_count = 0;
        graph_state.selected_nodes.clear();
        graph_state.selected_edges.clear();
        graph_state.hovered_entity = None;

        // Try to load the JSON file
        match load_graph_from_json(&event.file_path) {
            Ok((nodes, edges)) => {
                info!(
                    "Successfully parsed {} nodes and {} edges from JSON",
                    nodes.len(),
                    edges.len()
                );

                // We need to collect the UUID to Entity mapping after nodes are created
                // Store the edge data to process after nodes are spawned
                commands.insert_resource(PendingEdges {
                    edges,
                    node_count: nodes.len(),
                    processed_nodes: 0,
                });

                // Create nodes from loaded data
                for node_data in nodes {
                    info!(
                        "Creating node: {} at position {:?}",
                        node_data.name, node_data.position
                    );
                    create_node_events.send(node_data);
                }

                file_state.current_file_path = Some(event.file_path.clone());
                info!("Successfully loaded graph from {}", event.file_path);
                info!("=== COMPLETED LOADING FILE ===");
                info!("Graph state after loading - Nodes: {}, Edges: {}",
                    graph_state.node_count, graph_state.edge_count);
            }
            Err(e) => {
                warn!("Failed to load file {}: {}", event.file_path, e);
            }
        }
    }
}

/// Handle file saving
fn handle_save_json_file(
    mut events: EventReader<SaveJsonFileEvent>,
    node_query: Query<(&graph_core::GraphNode, &Transform)>,
    edge_query: Query<&graph_core::GraphEdge>,
    mut file_state: ResMut<FileOperationState>,
) {
    for event in events.read() {
        info!("Saving file: {}", event.file_path);

        // Collect graph data
        let mut nodes = Vec::new();
        for (node, transform) in &node_query {
            nodes.push((node.clone(), transform.translation));
        }

        let mut edges = Vec::new();
        for edge in &edge_query {
            edges.push(edge.clone());
        }

        match save_graph_to_json(&event.file_path, nodes, edges) {
            Ok(_) => {
                file_state.current_file_path = Some(event.file_path.clone());
                info!("Successfully saved graph to {}", event.file_path);
            }
            Err(e) => {
                warn!("Failed to save file {}: {}", event.file_path, e);
            }
        }
    }
}

/// Process pending edges after nodes are created
fn process_pending_edges(
    mut commands: Commands,
    pending_edges: Option<ResMut<PendingEdges>>,
    node_query: Query<(Entity, &graph_core::GraphNode)>,
    mut create_edge_events: EventWriter<CreateEdgeEvent>,
) {
    if let Some(mut pending) = pending_edges {
        // Check if we have all nodes created
        let current_node_count = node_query.iter().count();

        if current_node_count >= pending.node_count && !pending.edges.is_empty() {
            info!("Processing {} pending edges", pending.edges.len());

            // Build UUID to Entity mapping
            let mut uuid_to_entity = std::collections::HashMap::new();
            for (entity, node) in &node_query {
                uuid_to_entity.insert(node.id, entity);
            }

            // Create edges
            for edge_data in &pending.edges {
                if let (Some(&source_entity), Some(&target_entity)) = (
                    uuid_to_entity.get(&edge_data.source_uuid),
                    uuid_to_entity.get(&edge_data.target_uuid),
                ) {
                    create_edge_events.send(CreateEdgeEvent {
                        id: edge_data.id,
                        source: source_entity,
                        target: target_entity,
                        edge_type: edge_data.edge_type.clone(),
                        labels: edge_data.labels.clone(),
                        properties: edge_data.properties.clone(),
                    });
                } else {
                    warn!(
                        "Failed to find entities for edge {:?} -> {:?}",
                        edge_data.source_uuid, edge_data.target_uuid
                    );
                }
            }

            // Clear the pending edges
            pending.edges.clear();
            commands.remove_resource::<PendingEdges>();
        }
    }
}

// Simple JSON load/save functions
fn load_graph_from_json(
    path: &str,
) -> Result<(Vec<CreateNodeEvent>, Vec<PendingEdgeData>), String> {
    // Load the JSON file
    let json_data = load_json_file(path)?;

    // Convert to base graph
    let base_graph = json_to_base_graph(json_data)?;

    // Convert to events
    let mut node_events = Vec::new();
    let mut edge_data = Vec::new();

    // Create node events
    for (uuid, node) in &base_graph.graph.nodes {
        let position = base_graph
            .node_positions
            .get(uuid)
            .copied()
            .unwrap_or(Vec3::ZERO);

        // Determine domain type from labels or use the first label as a hint
        let domain_type = if node.labels.contains(&"decision".to_string()) {
            DomainNodeType::Decision
        } else if node.labels.contains(&"event".to_string()) {
            DomainNodeType::Event
        } else if node.labels.contains(&"storage".to_string()) {
            DomainNodeType::Storage
        } else if node.labels.contains(&"interface".to_string()) {
            DomainNodeType::Interface
        } else {
            DomainNodeType::Process
        };

        node_events.push(CreateNodeEvent {
            id: *uuid,
            position,
            domain_type,
            name: node.name.clone(),
            labels: node.labels.clone(),
            properties: node.properties.clone(),
            subgraph_id: None,
        });
    }

    // Create edge data
    for (_, edge) in &base_graph.graph.edges {
        edge_data.push(PendingEdgeData {
            id: edge.id,
            source_uuid: edge.source,
            target_uuid: edge.target,
            edge_type: graph_core::DomainEdgeType::DataFlow,
            labels: edge.labels.clone(),
            properties: edge.properties.clone(),
        });
    }

    Ok((node_events, edge_data))
}

fn save_graph_to_json(
    path: &str,
    nodes: Vec<(graph_core::GraphNode, Vec3)>,
    edges: Vec<graph_core::GraphEdge>,
) -> Result<(), String> {
    // Create JSON data structure directly
    let mut json_nodes = Vec::new();
    let json_relationships = Vec::new();
    let mut id_counter = 1;
    let mut uuid_to_string = std::collections::HashMap::new();

    // Convert nodes
    for (node, position) in nodes {
        let string_id = format!("n{}", id_counter);
        uuid_to_string.insert(node.id, string_id.clone());
        id_counter += 1;

        let json_node = JsonNode {
            id: string_id,
            position: JsonPosition {
                x: position.x * 100.0,
                y: position.z * 100.0, // Use Z as Y for 2D representation
            },
            caption: node.name.clone(),
            labels: node.labels.clone(),
            properties: node.properties.clone(),
            style: std::collections::HashMap::new(),
        };

        json_nodes.push(json_node);
    }

    // Note: Edge saving would require entity-to-UUID mapping
    // For now, we'll skip edges

    let json_data = JsonGraphData {
        nodes: json_nodes,
        relationships: json_relationships,
        style: Default::default(),
    };

    save_json_file(path, &json_data)?;

    Ok(())
}

/// System to track when nodes are despawned
fn track_node_despawns(
    mut removed_nodes: RemovedComponents<graph_core::GraphNode>,
    mut removed_edges: RemovedComponents<graph_core::GraphEdge>,
) {
    let removed_node_count: Vec<_> = removed_nodes.read().collect();
    let removed_edge_count: Vec<_> = removed_edges.read().collect();

    if !removed_node_count.is_empty() || !removed_edge_count.is_empty() {
        warn!(
            "NODES/EDGES REMOVED: {} nodes, {} edges despawned. Stack trace:",
            removed_node_count.len(),
            removed_edge_count.len()
        );
        // This will help us identify what's clearing the graph
        if removed_node_count.len() > 5 {
            info!("Graph cleared");
        }
    }
}
