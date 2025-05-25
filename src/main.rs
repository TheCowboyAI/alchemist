use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin};
use std::collections::HashMap;
use uuid::Uuid;

// Import the new modules
mod camera;
mod events;
mod graph;
mod graph_core;
mod graph_layout;
mod graph_patterns;
mod json_loader;
mod models;
mod theming;
mod ui_panels;
mod unified_graph_editor;

use camera::CameraViewportPlugin;
use graph_core::{CreateEdgeEvent, CreateNodeEvent, DomainNodeType, GraphPlugin};
use graph_layout::GraphLayoutPlugin;
use graph_patterns::{GraphPattern, generate_pattern};
use json_loader::{
    FileOperationState, JsonGraphData, JsonNode, JsonPosition, LoadJsonFileEvent,
    SaveJsonFileEvent, json_to_base_graph, load_json_file, save_json_file,
};
use ui_panels::UiPanelsPlugin;

#[derive(Resource, Default)]
struct NodeCounter(u32);

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
        .add_plugins(GraphLayoutPlugin)
        .add_plugins(UiPanelsPlugin)
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
                debug_camera_system.after(bevy_egui::EguiPreUpdateSet::InitContexts),
                keyboard_commands_system.after(bevy_egui::EguiPreUpdateSet::InitContexts),
                handle_load_json_file.after(bevy_egui::EguiPreUpdateSet::InitContexts),
                handle_save_json_file.after(bevy_egui::EguiPreUpdateSet::InitContexts),
            ),
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

    // Create some initial test nodes with IDs we can reference
    let center_id = Uuid::new_v4();
    let decision_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let storage_id = Uuid::new_v4();
    let interface_id = Uuid::new_v4();

    create_node_events.send(CreateNodeEvent {
        id: center_id,
        position: Vec3::new(0.0, 0.0, 0.0),
        domain_type: DomainNodeType::Process,
        name: "Central Node".to_string(),
        labels: vec!["process".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
        color: None,
    });

    create_node_events.send(CreateNodeEvent {
        id: decision_id,
        position: Vec3::new(5.0, 0.0, 0.0),
        domain_type: DomainNodeType::Decision,
        name: "Decision Node".to_string(),
        labels: vec!["decision".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
        color: None,
    });

    create_node_events.send(CreateNodeEvent {
        id: event_id,
        position: Vec3::new(-5.0, 0.0, 0.0),
        domain_type: DomainNodeType::Event,
        name: "Event Node".to_string(),
        labels: vec!["event".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
        color: None,
    });

    create_node_events.send(CreateNodeEvent {
        id: storage_id,
        position: Vec3::new(0.0, 0.0, 5.0),
        domain_type: DomainNodeType::Storage,
        name: "Storage Node".to_string(),
        labels: vec!["storage".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
        color: None,
    });

    create_node_events.send(CreateNodeEvent {
        id: interface_id,
        position: Vec3::new(0.0, 0.0, -5.0),
        domain_type: DomainNodeType::Interface,
        name: "Interface Node".to_string(),
        labels: vec!["event".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
        color: None,
    });
}

/// Setup file scanner
fn setup_file_scanner(mut file_state: ResMut<FileOperationState>) {
    file_state.scan_models_directory();
}

/// Keyboard commands system for clear controls
fn keyboard_commands_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut create_node_events: EventWriter<CreateNodeEvent>,
    mut commands: Commands,
    node_query: Query<Entity, With<graph_core::GraphNode>>,
    mut graph_state: ResMut<graph_core::GraphState>,
) {
    // Clear graph with Ctrl+K
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyK) {
        // Despawn all nodes
        for entity in &node_query {
            commands.entity(entity).despawn_recursive();
        }

        // Reset graph state
        graph_state.node_count = 0;
        graph_state.edge_count = 0;
        graph_state.selected_nodes.clear();
        graph_state.hovered_entity = None;

        // Clear the GraphData resource
        commands.insert_resource(graph_core::GraphData::default());

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
            color: None,
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
    mut deferred_edge_events: EventWriter<graph_core::DeferredEdgeEvent>,
    mut file_state: ResMut<FileOperationState>,
    mut commands: Commands,
    node_query: Query<(Entity, &graph_core::GraphNode)>,
    mut graph_state: ResMut<graph_core::GraphState>,
) {
    for event in events.read() {
        info!("Loading file: {}", event.file_path);

        // Try to load the JSON file first before clearing
        match load_graph_from_json(&event.file_path) {
            Ok((nodes, edges)) => {
                info!(
                    "Successfully parsed {} nodes and {} edges from JSON",
                    nodes.len(),
                    edges.len()
                );

                // Despawn all nodes
                for (entity, _) in &node_query {
                    commands.entity(entity).despawn_recursive();
                }

                // Reset graph state
                graph_state.node_count = 0;
                graph_state.edge_count = 0;
                graph_state.selected_nodes.clear();
                graph_state.hovered_entity = None;

                // Clear the GraphData resource
                commands.insert_resource(graph_core::GraphData::default());

                // Create nodes from loaded data
                for node_data in nodes {
                    create_node_events.send(node_data);
                }

                // Send deferred edge events
                for edge_event in edges {
                    deferred_edge_events.send(edge_event);
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
    graph_data: Res<graph_core::GraphData>,
    mut file_state: ResMut<FileOperationState>,
) {
    for event in events.read() {
        info!("Saving file: {}", event.file_path);

        // Collect graph data
        let mut nodes = Vec::new();
        for (node, transform) in &node_query {
            nodes.push((node.clone(), transform.translation));
        }

        // Get edges from GraphData
        // Note: Edge saving would require access to GraphData
        // For now, we'll skip edges
        for (_edge_idx, edge_data, _source_idx, _target_idx) in graph_data.edges() {
            // Convert EdgeData to the format expected by save function
            // For now, just log that we'd save edges
            info!("Would save edge {:?}", edge_data.id);
        }

        match save_graph_to_json(&event.file_path, nodes) {
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

/// System to track when nodes are despawned
fn track_node_despawns(
    mut removed_nodes: RemovedComponents<graph_core::GraphNode>,
) {
    let removed_node_count: Vec<_> = removed_nodes.read().collect();

    // Only log if it seems like an unexpected removal (small numbers)
    if !removed_node_count.is_empty() && removed_node_count.len() < 5 {
        info!(
            "Nodes removed: {} nodes despawned",
            removed_node_count.len()
        );
    }
}

/// Create a demo graph with various node types
fn setup_demo_graph(
    mut commands: Commands,
    mut create_node_events: EventWriter<CreateNodeEvent>,
    mut deferred_edge_events: EventWriter<graph_core::DeferredEdgeEvent>,
) {
    info!("Setting up demo graph");

    // Create node IDs
    let center_id = Uuid::new_v4();
    let decision_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let storage_id = Uuid::new_v4();
    let interface_id = Uuid::new_v4();

    // Create nodes
    create_node_events.send(CreateNodeEvent {
        id: center_id,
        position: Vec3::new(0.0, 0.0, 0.0),
        domain_type: DomainNodeType::Process,
        name: "Central Node".to_string(),
        labels: vec!["process".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
        color: None,
    });

    create_node_events.send(CreateNodeEvent {
        id: decision_id,
        position: Vec3::new(5.0, 0.0, 0.0),
        domain_type: DomainNodeType::Decision,
        name: "Decision Node".to_string(),
        labels: vec!["decision".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
        color: None,
    });

    create_node_events.send(CreateNodeEvent {
        id: event_id,
        position: Vec3::new(-5.0, 0.0, 0.0),
        domain_type: DomainNodeType::Event,
        name: "Event Node".to_string(),
        labels: vec!["event".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
        color: None,
    });

    create_node_events.send(CreateNodeEvent {
        id: storage_id,
        position: Vec3::new(0.0, 0.0, 5.0),
        domain_type: DomainNodeType::Storage,
        name: "Storage Node".to_string(),
        labels: vec!["storage".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
        color: None,
    });

    create_node_events.send(CreateNodeEvent {
        id: interface_id,
        position: Vec3::new(0.0, 0.0, -5.0),
        domain_type: DomainNodeType::Interface,
        name: "Interface Node".to_string(),
        labels: vec!["interface".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
        color: None,
    });

    // Create deferred edges using the new system
    deferred_edge_events.send(graph_core::DeferredEdgeEvent {
        id: Uuid::new_v4(),
        source_uuid: center_id,
        target_uuid: decision_id,
        edge_type: graph_core::DomainEdgeType::DataFlow,
        labels: vec!["test".to_string()],
        properties: std::collections::HashMap::new(),
        retry_count: 0,
    });

    deferred_edge_events.send(graph_core::DeferredEdgeEvent {
        id: Uuid::new_v4(),
        source_uuid: center_id,
        target_uuid: event_id,
        edge_type: graph_core::DomainEdgeType::DataFlow,
        labels: vec!["test".to_string()],
        properties: std::collections::HashMap::new(),
        retry_count: 0,
    });

    deferred_edge_events.send(graph_core::DeferredEdgeEvent {
        id: Uuid::new_v4(),
        source_uuid: center_id,
        target_uuid: storage_id,
        edge_type: graph_core::DomainEdgeType::DataFlow,
        labels: vec!["test".to_string()],
        properties: std::collections::HashMap::new(),
        retry_count: 0,
    });

    deferred_edge_events.send(graph_core::DeferredEdgeEvent {
        id: Uuid::new_v4(),
        source_uuid: center_id,
        target_uuid: interface_id,
        edge_type: graph_core::DomainEdgeType::DataFlow,
        labels: vec!["test".to_string()],
        properties: std::collections::HashMap::new(),
        retry_count: 0,
    });
}

// Simple JSON load/save functions
fn load_graph_from_json(
    path: &str,
) -> Result<(Vec<CreateNodeEvent>, Vec<graph_core::DeferredEdgeEvent>), String> {
    // Load the JSON file
    let json_data = load_json_file(path)?;

    // Convert to base graph
    let base_graph = json_to_base_graph(json_data)?;

    // Convert to events
    let mut node_events = Vec::new();
    let mut edge_events = Vec::new();

    // Create node events
    for (uuid, node) in &base_graph.graph.nodes {
        let mut position = base_graph
            .node_positions
            .get(uuid)
            .copied()
            .unwrap_or(Vec3::ZERO);

        // Lift the node up by its radius so it sits on the ground plane
        position.y += 0.5;

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

        // Get color from properties if available
        let mut properties_without_color = node.properties.clone();
        let color_str = properties_without_color.remove("node-color");

        node_events.push(CreateNodeEvent {
            id: *uuid,
            position,
            domain_type,
            name: node.name.clone(),
            labels: node.labels.clone(),
            properties: properties_without_color,
            subgraph_id: None,
            color: color_str,
        });
    }

    // Create deferred edge events
    for (_, edge) in &base_graph.graph.edges {
        edge_events.push(graph_core::DeferredEdgeEvent {
            id: edge.id,
            source_uuid: edge.source,
            target_uuid: edge.target,
            edge_type: graph_core::DomainEdgeType::DataFlow,
            labels: edge.labels.clone(),
            properties: edge.properties.clone(),
            retry_count: 0,
        });
    }

    Ok((node_events, edge_events))
}

fn save_graph_to_json(
    path: &str,
    nodes: Vec<(graph_core::GraphNode, Vec3)>,
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

    // Note: Edge saving would require access to GraphData
    // For now, we'll skip edges

    let json_data = JsonGraphData {
        nodes: json_nodes,
        relationships: json_relationships,
        style: Default::default(),
    };

    save_json_file(path, &json_data)?;

    Ok(())
}
