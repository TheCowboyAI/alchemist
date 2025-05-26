use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use uuid::Uuid;

// Import the new modular structure
mod components;
mod resources;
mod events;
mod bundles;
mod system_sets;

// Import the existing modules
mod camera;
mod events_old; // Rename to avoid conflict
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
use graph_core::{CreateNodeEvent, DomainNodeType};
use json_loader::{
    FileOperationState, LoadJsonFileEvent, SaveJsonFileEvent,
    load_json_file, save_json_file, json_to_base_graph,
    JsonGraphData, JsonNode, JsonPosition,
};
use resources::DpiScaling;
use system_sets::configure_system_sets;



fn main() {
    let mut app = App::new();

    app
        // Add DefaultPlugins first (includes basic Bevy functionality)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Alchemist Graph Editor".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        // Add EguiPlugin early so it initializes before other plugins
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: false,
        });

    // Configure system sets after basic plugins
    configure_system_sets(&mut app);

    app
        // Add our custom plugins
        .add_plugins(CameraViewportPlugin)
        .add_plugins(graph_core::GraphPlugin)
        .add_plugins(ui_panels::UiPanelsPlugin)
        .init_resource::<DpiScaling>()
        .init_resource::<FileOperationState>()
        .init_resource::<theming::AlchemistTheme>()
        .insert_resource(EdgeCreationTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
        .add_event::<LoadJsonFileEvent>()
        .add_event::<SaveJsonFileEvent>()
        .add_systems(
            Startup,
            (
                setup,
                setup_demo_graph,
                setup_file_scanner,
            ),
        )
        // Add setup_dpi_scaling separately to ensure it runs after EguiPlugin
        .add_systems(
            Startup,
            setup_dpi_scaling.after(bevy_egui::EguiStartupSet::InitContexts),
        )
        .add_systems(
            Update,
            (
                debug_camera_system,
                handle_load_json_file,
                handle_save_json_file,
                track_node_despawns,
                create_demo_edges_after_delay,
            ),
        )
        // Systems that use EguiContexts must run after initialization
        .add_systems(
            Update,
            (
                update_dpi_scaling,
                keyboard_commands_system,
            )
                .after(bevy_egui::EguiPreUpdateSet::InitContexts),
        )
        .run();
}

/// Setup DPI scaling on startup
fn setup_dpi_scaling(
    mut dpi_scaling: ResMut<DpiScaling>,
    mut contexts: EguiContexts,
    windows: Query<&Window>,
) {
    if let Ok(window) = windows.single() {
        let window_scale_factor = window.scale_factor();
        let resolution = &window.resolution;

        // Get egui's native pixels per point which might be different
        let egui_scale = contexts.ctx_mut().native_pixels_per_point().unwrap_or(1.0);

        // Use the higher of the two scale factors
        let detected_scale = window_scale_factor.max(egui_scale);

        dpi_scaling.scale_factor = detected_scale;

        let base_size = dpi_scaling.base_font_size;

        debug!("DPI Detection:");
        debug!("  Window scale factor: {}", window_scale_factor);
        debug!("  Egui native pixels per point: {}", egui_scale);
        debug!("  Window resolution: {}x{}", resolution.width(), resolution.height());
        debug!("  Using scale factor: {}", detected_scale);
        debug!("Applied DPI scaling: {}x (base font: {}pt -> {}pt)",
            detected_scale, base_size, base_size * detected_scale);

        // Apply initial scaling
        theming::apply_base16_theme(contexts.ctx_mut(), &theming::Base16Theme::tokyo_night());
        apply_dpi_scaling(&dpi_scaling, contexts.ctx_mut());
    }
}

/// Update DPI scaling when window scale factor changes
fn update_dpi_scaling(
    mut dpi_scaling: ResMut<DpiScaling>,
    mut contexts: EguiContexts,
    windows: Query<&Window, Changed<Window>>,
    theme: Res<theming::AlchemistTheme>,
) {
    // Only check for DPI changes if the window actually changed
    if let Ok(window) = windows.single() {
        let new_scale_factor = window.scale_factor();
        if (new_scale_factor - dpi_scaling.scale_factor).abs() > 0.01 {
            dpi_scaling.scale_factor = new_scale_factor;
            debug!("DPI scale factor changed to: {}", dpi_scaling.scale_factor);

            // Only reapply scaling if theme is not currently changing
            // The theme system will handle reapplying DPI scaling after theme changes
            if !theme.theme_changed {
                apply_dpi_scaling(&dpi_scaling, contexts.ctx_mut());
                debug!("Reapplied DPI scaling due to window change");
            }
        }
    }
}

/// Apply DPI scaling to egui context
fn apply_dpi_scaling(dpi_scaling: &DpiScaling, ctx: &mut egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Use manual override if available, otherwise use detected scale
    let scale = dpi_scaling.manual_override.unwrap_or(dpi_scaling.scale_factor);
    let base_size = dpi_scaling.base_font_size;

    // Apply scaling to all text styles
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(base_size * scale, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(base_size * scale, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new((base_size * 1.5) * scale, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new((base_size * 0.83) * scale, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new((base_size * 0.92) * scale, egui::FontFamily::Monospace),
    );

    // Scale spacing and padding proportionally
    let spacing_scale = scale.max(1.0); // Don't make spacing smaller than default
    style.spacing.item_spacing = egui::vec2(8.0 * spacing_scale, 6.0 * spacing_scale);
    style.spacing.button_padding = egui::vec2(12.0 * spacing_scale, 6.0 * spacing_scale);
    style.spacing.menu_margin = egui::Margin::same((8.0 * spacing_scale) as i8);
    style.spacing.indent = 18.0 * spacing_scale;

    // Scale window rounding
    let rounding_scale = scale.min(2.0); // Don't make rounding too large
    style.visuals.window_corner_radius = egui::CornerRadius::same((6.0 * rounding_scale) as u8);
    style.visuals.menu_corner_radius = egui::CornerRadius::same((4.0 * rounding_scale) as u8);
    style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same((3.0 * rounding_scale) as u8);
    style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same((3.0 * rounding_scale) as u8);
    style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same((3.0 * rounding_scale) as u8);
    style.visuals.widgets.active.corner_radius = egui::CornerRadius::same((3.0 * rounding_scale) as u8);

    ctx.set_style(style);

    let override_text = if dpi_scaling.manual_override.is_some() { " (manual)" } else { "" };
    info!("Applied DPI scaling: {}x{} (base font: {}pt -> {}pt)",
          scale, override_text, dpi_scaling.base_font_size, dpi_scaling.base_font_size * scale);
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
fn setup(mut commands: Commands) {
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

    // Note: Nodes are created in setup_demo_graph
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
    mut dpi_scaling: ResMut<DpiScaling>,
    mut contexts: EguiContexts,
    theme: Res<theming::AlchemistTheme>,
) {
    // Clear graph with Ctrl+K
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyK) {
        // Despawn all nodes
        for entity in &node_query {
            commands.entity(entity).despawn();
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
        create_node_events.write(CreateNodeEvent {
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

    // DPI scaling controls
    if keyboard.pressed(KeyCode::ControlLeft) {
        let mut dpi_changed = false;

        // Increase DPI scaling with Ctrl+Plus
        if keyboard.just_pressed(KeyCode::Equal) || keyboard.just_pressed(KeyCode::NumpadAdd) {
            let current_scale = dpi_scaling.manual_override.unwrap_or(dpi_scaling.scale_factor);
            let new_scale = (current_scale + 0.25).min(3.0);
            dpi_scaling.manual_override = Some(new_scale);
            dpi_changed = true;
            info!("Increased DPI scaling to: {}x", new_scale);
        }

        // Decrease DPI scaling with Ctrl+Minus
        if keyboard.just_pressed(KeyCode::Minus) || keyboard.just_pressed(KeyCode::NumpadSubtract) {
            let current_scale = dpi_scaling.manual_override.unwrap_or(dpi_scaling.scale_factor);
            let new_scale = (current_scale - 0.25).max(0.5);
            dpi_scaling.manual_override = Some(new_scale);
            dpi_changed = true;
            info!("Decreased DPI scaling to: {}x", new_scale);
        }

        // Reset DPI scaling with Ctrl+0
        if keyboard.just_pressed(KeyCode::Digit0) || keyboard.just_pressed(KeyCode::Numpad0) {
            dpi_scaling.manual_override = None;
            dpi_changed = true;
            info!("Reset DPI scaling to auto-detected: {}x", dpi_scaling.scale_factor);
        }

        // Apply DPI changes immediately
        if dpi_changed {
            theming::apply_base16_theme(contexts.ctx_mut(), &theme.current_theme);
            apply_dpi_scaling(&dpi_scaling, contexts.ctx_mut());
        }
    }

    // Print help with H
    if keyboard.just_pressed(KeyCode::KeyH) {
        info!("=== Keyboard Commands ===");
        info!("Tab/V: Switch between 2D/3D view");
        info!("Ctrl+K: Clear graph");
        info!("Ctrl+N: Add new node at origin");
        info!("Ctrl++: Increase DPI scaling");
        info!("Ctrl+-: Decrease DPI scaling");
        info!("Ctrl+0: Reset DPI scaling to auto");
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
                    commands.entity(entity).despawn();
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
                    create_node_events.write(node_data);
                }

                // Send deferred edge events
                for edge_event in edges {
                    deferred_edge_events.write(edge_event);
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
    _commands: Commands,
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
    create_node_events.write(CreateNodeEvent {
        id: center_id,
        position: Vec3::new(0.0, 0.0, 0.0),
        domain_type: DomainNodeType::Process,
        name: "Central Node".to_string(),
        labels: vec!["process".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
        color: None,
    });

    create_node_events.write(CreateNodeEvent {
        id: decision_id,
        position: Vec3::new(5.0, 0.0, 0.0),
        domain_type: DomainNodeType::Decision,
        name: "Decision Node".to_string(),
        labels: vec!["decision".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
        color: None,
    });

    create_node_events.write(CreateNodeEvent {
        id: event_id,
        position: Vec3::new(-5.0, 0.0, 0.0),
        domain_type: DomainNodeType::Event,
        name: "Event Node".to_string(),
        labels: vec!["event".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
        color: None,
    });

    create_node_events.write(CreateNodeEvent {
        id: storage_id,
        position: Vec3::new(0.0, 0.0, 5.0),
        domain_type: DomainNodeType::Storage,
        name: "Storage Node".to_string(),
        labels: vec!["storage".to_string()],
        properties: std::collections::HashMap::new(),
        subgraph_id: None,
        color: None,
    });

    create_node_events.write(CreateNodeEvent {
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
    info!("Creating edges for demo graph");

    deferred_edge_events.write(graph_core::DeferredEdgeEvent {
        id: Uuid::new_v4(),
        source_uuid: center_id,
        target_uuid: decision_id,
        edge_type: graph_core::DomainEdgeType::DataFlow,
        labels: vec!["test".to_string()],
        properties: std::collections::HashMap::new(),
        retry_count: 0,
    });

    deferred_edge_events.write(graph_core::DeferredEdgeEvent {
        id: Uuid::new_v4(),
        source_uuid: center_id,
        target_uuid: event_id,
        edge_type: graph_core::DomainEdgeType::DataFlow,
        labels: vec!["test".to_string()],
        properties: std::collections::HashMap::new(),
        retry_count: 0,
    });

    deferred_edge_events.write(graph_core::DeferredEdgeEvent {
        id: Uuid::new_v4(),
        source_uuid: center_id,
        target_uuid: storage_id,
        edge_type: graph_core::DomainEdgeType::DataFlow,
        labels: vec!["test".to_string()],
        properties: std::collections::HashMap::new(),
        retry_count: 0,
    });

    deferred_edge_events.write(graph_core::DeferredEdgeEvent {
        id: Uuid::new_v4(),
        source_uuid: center_id,
        target_uuid: interface_id,
        edge_type: graph_core::DomainEdgeType::DataFlow,
        labels: vec!["test".to_string()],
        properties: std::collections::HashMap::new(),
        retry_count: 0,
    });

    info!("Demo graph setup complete: 5 nodes, 4 edges");
}

/// Timer resource to delay edge creation
#[derive(Resource)]
struct EdgeCreationTimer(Timer);

/// System to create edges after nodes are ready
fn create_demo_edges_after_delay(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<EdgeCreationTimer>,
    graph_data: Res<graph_core::GraphData>,
    node_query: Query<(Entity, &graph_core::GraphNode)>,
) {
    timer.0.tick(time.delta());

    // Only run once when timer finishes and we have all nodes
    if timer.0.just_finished() && graph_data.node_count() >= 5 && !timer.0.paused() {
        info!("Creating edges directly after delay");

        // Find the central node
        let central_node = node_query.iter()
            .find(|(_, node)| node.name == "Central Node");

        if let Some((central_entity, _central_node)) = central_node {
            // Collect all edges for the central node
            let mut outgoing_edges = graph_core::OutgoingEdges::default();

            for (entity, node) in &node_query {
                if entity != central_entity {
                    // Add edge to the collection
                    outgoing_edges.edges.push(graph_core::OutgoingEdge {
                        id: Uuid::new_v4(),
                        target: entity,
                        edge_type: graph_core::DomainEdgeType::DataFlow,
                        labels: vec!["demo".to_string()],
                        properties: std::collections::HashMap::new(),
                    });
                    info!("Added edge from Central Node to {}", node.name);
                }
            }

            let edge_count = outgoing_edges.edges.len();
            info!("Created {} edges total", edge_count);

            // Add all edges at once
            commands.entity(central_entity).insert(outgoing_edges);

            // Pause the timer so it doesn't run again
            timer.0.pause();
        }
    }
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
