//! Complete Graph Visualization Demo with Bevy
//!
//! This example demonstrates:
//! - Loading graphs from files
//! - Real-time graph visualization
//! - Interactive node selection
//! - Connected component highlighting
//! - JetStream persistence
//! - Dynamic graph updates

use alchemist::{
    graph_plugin::{AlchemistGraphPlugin, GraphLoadRequest, GraphSaveRequest, GraphExportFormat},
    graph_components::*,
    graph_algorithms::ComponentMembership,
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Alchemist Graph Visualization".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_plugins(AlchemistGraphPlugin::default())
        .insert_resource(UiState::default())
        .add_systems(Startup, setup_demo_graph)
        .add_systems(Update, (
            ui_system,
            handle_keyboard_input,
            camera_control_system,
            highlight_component_system,
        ))
        .run();
}

#[derive(Resource, Default)]
struct UiState {
    file_path: String,
    show_components: bool,
    selected_layout: LayoutType,
    export_format: GraphExportFormat,
}

fn setup_demo_graph(
    mut commands: Commands,
    mut graph_ops: EventWriter<GraphOperationEvent>,
) {
    // Create a demo graph with multiple components
    let graph_id = "demo_graph".to_string();
    
    // Component 1: Triangle
    for i in 0..3 {
        graph_ops.send(GraphOperationEvent {
            graph_id: graph_id.clone(),
            operation: GraphOperation::CreateNode {
                id: format!("node_{}", i),
                label: format!("Node {}", i),
                position: Vec3::new(
                    i as f32 * 3.0,
                    0.0,
                    0.0,
                ),
            },
        });
    }
    
    // Connect triangle
    graph_ops.send(GraphOperationEvent {
        graph_id: graph_id.clone(),
        operation: GraphOperation::CreateEdge {
            id: "edge_0".to_string(),
            source_id: "node_0".to_string(),
            target_id: "node_1".to_string(),
            label: Some("connects".to_string()),
        },
    });
    
    graph_ops.send(GraphOperationEvent {
        graph_id: graph_id.clone(),
        operation: GraphOperation::CreateEdge {
            id: "edge_1".to_string(),
            source_id: "node_1".to_string(),
            target_id: "node_2".to_string(),
            label: Some("connects".to_string()),
        },
    });
    
    graph_ops.send(GraphOperationEvent {
        graph_id: graph_id.clone(),
        operation: GraphOperation::CreateEdge {
            id: "edge_2".to_string(),
            source_id: "node_2".to_string(),
            target_id: "node_0".to_string(),
            label: Some("connects".to_string()),
        },
    });
    
    // Component 2: Chain
    for i in 3..7 {
        graph_ops.send(GraphOperationEvent {
            graph_id: graph_id.clone(),
            operation: GraphOperation::CreateNode {
                id: format!("node_{}", i),
                label: format!("Node {}", i),
                position: Vec3::new(
                    -3.0,
                    0.0,
                    (i - 3) as f32 * 3.0,
                ),
            },
        });
    }
    
    // Connect chain
    for i in 3..6 {
        graph_ops.send(GraphOperationEvent {
            graph_id: graph_id.clone(),
            operation: GraphOperation::CreateEdge {
                id: format!("edge_{}", i),
                source_id: format!("node_{}", i),
                target_id: format!("node_{}", i + 1),
                label: Some("follows".to_string()),
            },
        });
    }
    
    // Isolated node
    graph_ops.send(GraphOperationEvent {
        graph_id: graph_id.clone(),
        operation: GraphOperation::CreateNode {
            id: "isolated".to_string(),
            label: "Isolated Node".to_string(),
            position: Vec3::new(10.0, 0.0, 0.0),
        },
    });
}

fn ui_system(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut load_events: EventWriter<GraphLoadRequest>,
    mut save_events: EventWriter<GraphSaveRequest>,
    mut graph_ops: EventWriter<GraphOperationEvent>,
    nodes: Query<&GraphNode>,
    edges: Query<&GraphEdge>,
    components: Query<&ComponentMembership>,
) {
    egui::SidePanel::left("control_panel")
        .default_width(300.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Graph Visualization Control");
            
            ui.separator();
            
            // Graph stats
            ui.label(format!("Nodes: {}", nodes.iter().count()));
            ui.label(format!("Edges: {}", edges.iter().count()));
            
            let component_count = components.iter()
                .map(|c| c.component_id)
                .collect::<std::collections::HashSet<_>>()
                .len();
            ui.label(format!("Components: {}", component_count));
            
            ui.separator();
            
            // File operations
            ui.collapsing("File Operations", |ui| {
                ui.horizontal(|ui| {
                    ui.label("File path:");
                    ui.text_edit_singleline(&mut ui_state.file_path);
                });
                
                ui.horizontal(|ui| {
                    if ui.button("Load Graph").clicked() {
                        load_events.send(GraphLoadRequest {
                            file_path: ui_state.file_path.clone(),
                            graph_id: Some("demo_graph".to_string()),
                        });
                    }
                    
                    if ui.button("Save Graph").clicked() {
                        save_events.send(GraphSaveRequest {
                            graph_id: "demo_graph".to_string(),
                            file_path: ui_state.file_path.clone(),
                            format: ui_state.export_format,
                        });
                    }
                });
                
                ui.label("Export format:");
                ui.radio_value(&mut ui_state.export_format, GraphExportFormat::Json, "JSON");
                ui.radio_value(&mut ui_state.export_format, GraphExportFormat::Cytoscape, "Cytoscape");
                ui.radio_value(&mut ui_state.export_format, GraphExportFormat::Graphviz, "Graphviz DOT");
                ui.radio_value(&mut ui_state.export_format, GraphExportFormat::Gexf, "GEXF");
            });
            
            ui.separator();
            
            // Layout controls
            ui.collapsing("Layout", |ui| {
                if ui.button("Force-Directed Layout").clicked() {
                    graph_ops.send(GraphOperationEvent {
                        graph_id: "demo_graph".to_string(),
                        operation: GraphOperation::ApplyLayout {
                            layout_type: LayoutType::ForceDirected,
                        },
                    });
                }
                
                if ui.button("Hierarchical Layout").clicked() {
                    graph_ops.send(GraphOperationEvent {
                        graph_id: "demo_graph".to_string(),
                        operation: GraphOperation::ApplyLayout {
                            layout_type: LayoutType::Hierarchical,
                        },
                    });
                }
                
                if ui.button("Circular Layout").clicked() {
                    graph_ops.send(GraphOperationEvent {
                        graph_id: "demo_graph".to_string(),
                        operation: GraphOperation::ApplyLayout {
                            layout_type: LayoutType::Circular,
                        },
                    });
                }
                
                if ui.button("Grid Layout").clicked() {
                    graph_ops.send(GraphOperationEvent {
                        graph_id: "demo_graph".to_string(),
                        operation: GraphOperation::ApplyLayout {
                            layout_type: LayoutType::Grid,
                        },
                    });
                }
            });
            
            ui.separator();
            
            // Component visualization
            ui.checkbox(&mut ui_state.show_components, "Highlight Components");
            
            ui.separator();
            
            // Graph operations
            ui.collapsing("Graph Operations", |ui| {
                if ui.button("Add Random Node").clicked() {
                    let id = format!("random_{}", uuid::Uuid::new_v4());
                    graph_ops.send(GraphOperationEvent {
                        graph_id: "demo_graph".to_string(),
                        operation: GraphOperation::CreateNode {
                            id: id.clone(),
                            label: "Random Node".to_string(),
                            position: Vec3::new(
                                rand::random::<f32>() * 20.0 - 10.0,
                                0.0,
                                rand::random::<f32>() * 20.0 - 10.0,
                            ),
                        },
                    });
                }
                
                if ui.button("Clear Graph").clicked() {
                    graph_ops.send(GraphOperationEvent {
                        graph_id: "demo_graph".to_string(),
                        operation: GraphOperation::Clear,
                    });
                }
            });
            
            ui.separator();
            
            // Instructions
            ui.collapsing("Controls", |ui| {
                ui.label("Camera:");
                ui.label("• WASD - Move horizontally");
                ui.label("• Q/E - Move up/down");
                ui.label("• Mouse drag - Rotate");
                ui.label("");
                ui.label("Selection:");
                ui.label("• Click - Select node");
                ui.label("• Shift+Click - Multi-select");
                ui.label("• Delete - Remove selected");
            });
        });
}

fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut graph_ops: EventWriter<GraphOperationEvent>,
    selected: Query<(Entity, &GraphNode), With<SelectedNode>>,
) {
    if keyboard.just_pressed(KeyCode::Delete) {
        for (entity, node) in selected.iter() {
            graph_ops.send(GraphOperationEvent {
                graph_id: node.graph_id.clone(),
                operation: GraphOperation::DeleteNode { entity },
            });
        }
    }
}

fn camera_control_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    let mut camera_transform = camera.single_mut();
    let speed = 10.0 * time.delta_seconds();
    
    // Movement
    if keyboard.pressed(KeyCode::KeyW) {
        let forward = camera_transform.forward();
        camera_transform.translation += forward * speed;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        let forward = camera_transform.forward();
        camera_transform.translation -= forward * speed;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        let right = camera_transform.right();
        camera_transform.translation -= right * speed;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        let right = camera_transform.right();
        camera_transform.translation += right * speed;
    }
    if keyboard.pressed(KeyCode::KeyQ) {
        camera_transform.translation.y -= speed;
    }
    if keyboard.pressed(KeyCode::KeyE) {
        camera_transform.translation.y += speed;
    }
}

fn highlight_component_system(
    ui_state: Res<UiState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    nodes: Query<(&Handle<StandardMaterial>, Option<&ComponentMembership>), With<GraphNode>>,
) {
    if !ui_state.show_components {
        return;
    }
    
    for (material_handle, membership) in nodes.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            if let Some(membership) = membership {
                // Color based on component ID
                let hue = (membership.component_id as f32 * 137.5) % 360.0;
                material.base_color = Color::hsl(hue, 0.7, 0.5);
                
                // Make articulation points brighter
                if membership.is_articulation_point {
                    material.emissive = Color::hsl(hue, 0.7, 0.3);
                }
            }
        }
    }
}