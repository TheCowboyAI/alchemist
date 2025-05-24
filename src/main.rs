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
mod models;
mod json_loader;

use camera::CameraViewportPlugin;
use graph_core::{CreateEdgeEvent, CreateNodeEvent, DomainNodeType, GraphPlugin};
use graph_patterns::{GraphPattern, generate_pattern};
use json_loader::{load_json_file, save_json_file, json_to_base_graph, base_graph_to_json, FileOperationState, JsonGraphData, JsonNode, JsonRelationship, JsonPosition};

#[derive(Resource, Default)]
struct NodeCounter(u32);

#[derive(Event)]
pub struct LoadJsonFileEvent {
    pub file_path: String,
}

#[derive(Event)]
pub struct SaveJsonFileEvent {
    pub file_path: String,
}

#[derive(Event)]
pub struct ClearGraphEvent;

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
        .add_event::<ClearGraphEvent>()
        // Setup systems
        .add_systems(Startup, (setup, setup_file_scanner))
        .add_systems(Update, (
            ui_system,
            debug_camera_system,
            keyboard_commands_system,
            handle_load_json_file,
            handle_save_json_file,
            handle_clear_graph,
        ))
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
        subgraph_id: None,
    });

    create_node_events.send(CreateNodeEvent {
        id: Uuid::new_v4(),
        position: Vec3::new(5.0, 0.0, 0.0),
        domain_type: DomainNodeType::Decision,
        name: "Decision Node".to_string(),
        subgraph_id: None,
    });

    create_node_events.send(CreateNodeEvent {
        id: Uuid::new_v4(),
        position: Vec3::new(-5.0, 0.0, 0.0),
        domain_type: DomainNodeType::Event,
        name: "Event Node".to_string(),
        subgraph_id: None,
    });

    create_node_events.send(CreateNodeEvent {
        id: Uuid::new_v4(),
        position: Vec3::new(0.0, 0.0, 5.0),
        domain_type: DomainNodeType::Storage,
        name: "Storage Node".to_string(),
        subgraph_id: None,
    });

    create_node_events.send(CreateNodeEvent {
        id: Uuid::new_v4(),
        position: Vec3::new(0.0, 0.0, -5.0),
        domain_type: DomainNodeType::Interface,
        name: "Interface Node".to_string(),
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
    graph_state: Res<graph_core::GraphState>,
    camera_query: Query<&camera::GraphViewCamera>,
    mut create_node_events: EventWriter<CreateNodeEvent>,
    mut create_edge_events: EventWriter<CreateEdgeEvent>,
    mut load_json_events: EventWriter<LoadJsonFileEvent>,
    mut save_json_events: EventWriter<SaveJsonFileEvent>,
    mut file_state: ResMut<FileOperationState>,
    node_query: Query<(Entity, &graph_core::GraphNode)>,
    mut clear_events: EventWriter<ClearGraphEvent>,
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
                    subgraph_id: None,
                });
            }

            // Clear graph button
            if ui.button("🗑️ Clear Graph").clicked() {
                clear_events.send(ClearGraphEvent);
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
    create_edge_events: &mut EventWriter<CreateEdgeEvent>,
    existing_nodes: &Query<(Entity, &graph_core::GraphNode)>,
) {
    let pattern_graph = generate_pattern(pattern);

    // Calculate offset to avoid overlapping with existing nodes
    let offset = Vec3::new(10.0, 0.0, 10.0);

    // Map old UUIDs to new entities
    let mut id_to_entity: std::collections::HashMap<Uuid, Entity> =
        std::collections::HashMap::new();

    // First pass: create all nodes
    for (old_id, node) in &pattern_graph.nodes {
        let new_id = Uuid::new_v4();

        // Calculate position with offset
        let pos = if let (Some(x_str), Some(y_str)) =
            (node.properties.get("x_pos"), node.properties.get("y_pos"))
        {
            if let (Ok(x), Ok(y)) = (x_str.parse::<f32>(), y_str.parse::<f32>()) {
                Vec3::new(x / 20.0 + offset.x, 0.0, y / 20.0 + offset.z)
            } else {
                offset
            }
        } else {
            offset
        };

        create_node_events.send(CreateNodeEvent {
            id: new_id,
            position: pos,
            domain_type: DomainNodeType::Process, // Default type
            name: node.name.clone(),
            subgraph_id: None,
        });

        // We'll need to wait for nodes to be created before we can get their entities
        // For now, just track the mapping
        id_to_entity.insert(*old_id, Entity::PLACEHOLDER);
    }

    // Note: Edge creation would need to happen after nodes are created
    // This is a limitation we'd need to address with a better event system
}

/// Keyboard commands system for clear controls
fn keyboard_commands_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut clear_events: EventWriter<ClearGraphEvent>,
    mut create_node_events: EventWriter<CreateNodeEvent>,
) {
    // Clear graph with Ctrl+K
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyK) {
        clear_events.send(ClearGraphEvent);
        info!("Clear graph command triggered");
    }

    // Add node at origin with Ctrl+N
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyN) {
        create_node_events.send(CreateNodeEvent {
            id: Uuid::new_v4(),
            position: Vec3::ZERO,
            domain_type: DomainNodeType::Process,
            name: "New Node".to_string(),
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
    mut commands: Commands,
    mut clear_events: EventWriter<ClearGraphEvent>,
    mut create_node_events: EventWriter<CreateNodeEvent>,
    mut file_state: ResMut<FileOperationState>,
) {
    for event in events.read() {
        info!("Loading file: {}", event.file_path);

        // First clear the graph
        clear_events.send(ClearGraphEvent);

        // Try to load the JSON file
        match load_graph_from_json(&event.file_path) {
            Ok((nodes, _edges)) => {
                // Create nodes from loaded data
                for node_data in nodes {
                    create_node_events.send(node_data);
                }

                file_state.current_file_path = Some(event.file_path.clone());
                info!("Successfully loaded graph from {}", event.file_path);
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
        let nodes: Vec<_> = node_query.iter()
            .map(|(node, transform)| (node.clone(), transform.translation))
            .collect();

        let edges: Vec<_> = edge_query.iter().cloned().collect();

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

/// Handle clearing the graph
fn handle_clear_graph(
    mut events: EventReader<ClearGraphEvent>,
    mut commands: Commands,
    node_query: Query<Entity, With<graph_core::GraphNode>>,
    edge_query: Query<Entity, With<graph_core::GraphEdge>>,
    mut graph_state: ResMut<graph_core::GraphState>,
) {
    for _ in events.read() {
        // Despawn all nodes
        for entity in &node_query {
            commands.entity(entity).despawn_recursive();
        }

        // Despawn all edges
        for entity in &edge_query {
            commands.entity(entity).despawn_recursive();
        }

        // Reset graph state
        graph_state.node_count = 0;
        graph_state.edge_count = 0;
        graph_state.selected_nodes.clear();
        graph_state.selected_edges.clear();
        graph_state.hovered_entity = None;

        info!("Graph cleared");
    }
}

// Simple JSON load/save functions
fn load_graph_from_json(path: &str) -> Result<(Vec<CreateNodeEvent>, Vec<graph_core::CreateEdgeEvent>), String> {
    // Load the JSON file
    let json_data = load_json_file(path)?;

    // Convert to base graph
    let base_graph = json_to_base_graph(json_data)?;

    // Convert to events
    let mut node_events = Vec::new();
    let mut edge_events = Vec::new();

    // Create node events
    for (uuid, node) in &base_graph.graph.nodes {
        let position = base_graph.node_positions.get(uuid)
            .copied()
            .unwrap_or(Vec3::ZERO);

        // Determine domain type from labels
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
            subgraph_id: None,
        });
    }

    // Create edge events
    for (_, edge) in &base_graph.graph.edges {
        edge_events.push(graph_core::CreateEdgeEvent {
            id: edge.id,
            source: Entity::PLACEHOLDER, // Will need to map UUIDs to entities
            target: Entity::PLACEHOLDER, // Will need to map UUIDs to entities
            edge_type: graph_core::DomainEdgeType::DataFlow,
        });
    }

    Ok((node_events, edge_events))
}

fn save_graph_to_json(
    path: &str,
    nodes: Vec<(graph_core::GraphNode, Vec3)>,
    edges: Vec<graph_core::GraphEdge>
) -> Result<(), String> {
    // Create JSON data structure directly
    let mut json_nodes = Vec::new();
    let mut json_relationships = Vec::new();
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
            labels: vec![format!("{:?}", node.domain_type).to_lowercase()],
            properties: std::collections::HashMap::new(),
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
