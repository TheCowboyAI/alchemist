//! Progress Graph Visualization Demo
//!
//! Loads progress.json, converts to contextgraph, and displays with:
//! - Different edge colors for relationship types  
//! - Text labels on nodes
//! - Click interaction for metadata display

use bevy::prelude::*;
use cim_contextgraph::{ContextGraph, NodeId as ContextNodeId, EdgeId as ContextEdgeId};
use std::collections::HashMap;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use serde::{Deserialize, Serialize};
use std::fs;

fn main() {
    println!("🚀 Starting Progress Graph Visualization Demo");
    
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CIM - Progress Graph Visualization".into(),
                resolution: (1400.0, 900.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 200.0,
        })
        .insert_resource(ProgressGraphData::default())
        .insert_resource(SelectedNode::default())
        .add_systems(Startup, (
            setup_camera,
            setup_ui,
            load_and_visualize_progress,
        ))
        .add_systems(Update, (
            camera_controller,
            update_info_panel,
            handle_node_interactions,
        ))
        .run();
}

// Data structures for progress.json
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
    updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProgressNode {
    id: String,
    label: String,
    #[serde(rename = "type")]
    node_type: String,
    position: Position3D,
    data: ProgressNodeData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Position3D {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProgressNodeData {
    status: String,
    #[serde(default)]
    progress: Option<u32>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    completed_date: Option<String>,
    description: String,
    #[serde(default)]
    details: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProgressEdge {
    id: String,
    source: String,
    target: String,
    relationship: String,
    label: String,
}

// ECS Components
#[derive(Component)]
struct ProgressNodeComponent {
    node_id: String,
    label: String,
    node_type: String,
    data: ProgressNodeData,
}

#[derive(Component)]
struct ProgressEdgeComponent {
    edge_id: String,
    relationship: String,
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

// Resources
#[derive(Resource, Default)]
struct ProgressGraphData {
    node_count: usize,
    edge_count: usize,
    project_info: Option<String>,
}

#[derive(Resource, Default)]
struct SelectedNode {
    node_data: Option<ProgressNodeData>,
    node_label: Option<String>,
}

// Edge color mapping
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
        Camera3dBundle {
            transform: Transform::from_xyz(50.0, 20.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraController {
            sensitivity: 150.0,
            speed: 25.0,
        },
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn setup_ui(mut commands: Commands) {
    // Info panel
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(350.0),
                height: Val::Px(400.0),
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::linear_rgba(0.0, 0.0, 0.0, 0.8).into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Loading Progress Graph...",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                InfoPanel,
            ));
        });

    // Metadata panel (initially hidden)
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(400.0),
                height: Val::Px(500.0),
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                top: Val::Px(10.0),
                padding: UiRect::all(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                display: Display::None,
                ..default()
            },
            background_color: Color::linear_rgba(0.1, 0.1, 0.1, 0.9).into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Click a node to view metadata",
                    TextStyle {
                        font_size: 14.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                MetadataPanel,
            ));
        });
}

fn load_and_visualize_progress(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut progress_data: ResMut<ProgressGraphData>,
) {
    println!("📖 Loading progress.json...");

    let progress_json = match fs::read_to_string("doc/progress/progress.json") {
        Ok(content) => content,
        Err(e) => {
            println!("❌ Failed to load progress.json: {}", e);
            return;
        }
    };

    let progress: ProgressData = match serde_json::from_str(&progress_json) {
        Ok(data) => data,
        Err(e) => {
            println!("❌ Failed to parse progress.json: {}", e);
            return;
        }
    };

    println!("✅ Loaded {} nodes and {} edges", progress.nodes.len(), progress.edges.len());

    progress_data.node_count = progress.nodes.len();
    progress_data.edge_count = progress.edges.len();
    progress_data.project_info = Some(format!(
        "{}\nVersion: {}\nLast Updated: {}",
        progress.metadata.name,
        progress.metadata.version,
        progress.metadata.updated
    ));

    let mut context_graph = ContextGraph::new();
    let mut node_entity_map = HashMap::new();

    let sphere_mesh = meshes.add(Sphere::new(0.8));
    let cube_mesh = meshes.add(Cuboid::new(1.2, 1.2, 1.2));
    let cylinder_mesh = meshes.add(Cylinder::new(0.6, 1.0));

    // Spawn nodes
    for node in &progress.nodes {
        let position = Vec3::new(
            node.position.x * 0.1,
            node.position.y * 0.1, 
            node.position.z * 0.1,
        );

        let (mesh, material) = match node.node_type.as_str() {
            "milestone" => (
                sphere_mesh.clone(),
                materials.add(StandardMaterial {
                    base_color: get_node_color(&node.node_type, &node.data.status),
                    metallic: 0.3,
                    perceptual_roughness: 0.5,
                    ..default()
                })
            ),
            "phase" => (
                cube_mesh.clone(),
                materials.add(StandardMaterial {
                    base_color: get_node_color(&node.node_type, &node.data.status),
                    metallic: 0.2,
                    perceptual_roughness: 0.6,
                    ..default()
                })
            ),
            _ => (
                cylinder_mesh.clone(),
                materials.add(StandardMaterial {
                    base_color: get_node_color(&node.node_type, &node.data.status),
                    metallic: 0.1,
                    perceptual_roughness: 0.7,
                    ..default()
                })
            ),
        };

        let entity = commands.spawn((
            PbrBundle {
                mesh,
                material,
                transform: Transform::from_translation(position),
                ..default()
            },
            ProgressNodeComponent {
                node_id: node.id.clone(),
                label: node.label.clone(),
                node_type: node.node_type.clone(),
                data: node.data.clone(),
            },
            Clickable,
        )).id();

        // Add text label
        commands.spawn(Text2dBundle {
            text: Text::from_section(
                &node.label,
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_translation(position + Vec3::new(0.0, 2.0, 0.0)),
            ..default()
        });

        node_entity_map.insert(node.id.clone(), entity);
        
        context_graph.add_node(ContextNodeId::new(&node.id), serde_json::json!({
            "label": node.label,
            "type": node.node_type,
            "status": node.data.status,
            "description": node.data.description
        }));
    }

    // Spawn edges
    for edge in &progress.edges {
        if let (Some(&source_entity), Some(&target_entity)) = 
            (node_entity_map.get(&edge.source), node_entity_map.get(&edge.target)) {
            
            // Get positions (simplified - in real implementation would query components)
            let source_pos = Vec3::ZERO; // Would get from entity transform
            let target_pos = Vec3::new(1.0, 0.0, 0.0); // Would get from entity transform

            let direction = target_pos - source_pos;
            let length = direction.length().max(0.1);
            let midpoint = source_pos + direction * 0.5;

            let material = materials.add(StandardMaterial {
                base_color: get_edge_color(&edge.relationship),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            });

            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Cylinder::new(0.05, length)),
                    material,
                    transform: Transform::from_translation(midpoint),
                    ..default()
                },
                ProgressEdgeComponent {
                    edge_id: edge.id.clone(),
                    relationship: edge.relationship.clone(),
                    label: edge.label.clone(),
                },
            ));

            context_graph.add_edge(
                ContextEdgeId::new(&edge.id),
                ContextNodeId::new(&edge.source),
                ContextNodeId::new(&edge.target),
                serde_json::json!({
                    "relationship": edge.relationship,
                    "label": edge.label
                })
            );
        }
    }
    
    println!("🎨 Visualized progress graph with contextgraph integration");
}

fn camera_controller(
    mut camera_query: Query<(&mut Transform, &CameraController), With<Camera>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, controller)) = camera_query.single_mut() {
        let mut rotation_delta = Vec2::ZERO;
        let mut translation_delta = Vec3::ZERO;

        let is_right_mouse_pressed = mouse_button_input.pressed(MouseButton::Right);
        
        for event in mouse_motion_events.read() {
            if is_right_mouse_pressed {
                rotation_delta += event.delta * controller.sensitivity * 0.001;
            }
        }

        if keyboard_input.pressed(KeyCode::KeyW) {
            translation_delta -= transform.forward().as_vec3() * controller.speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            translation_delta += transform.forward().as_vec3() * controller.speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            translation_delta -= transform.right().as_vec3() * controller.speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            translation_delta += transform.right().as_vec3() * controller.speed * time.delta_secs();
        }

        for event in mouse_wheel_events.read() {
            let zoom_speed = controller.speed * 3.0;
            translation_delta += transform.forward().as_vec3() * event.y * zoom_speed;
        }

        if rotation_delta.length_squared() > 0.0 {
            let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            let new_yaw = yaw - rotation_delta.x;
            let new_pitch = (pitch - rotation_delta.y).clamp(-1.5, 1.5);
            transform.rotation = Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, 0.0);
        }

        if translation_delta.length_squared() > 0.0 {
            transform.translation += translation_delta;
        }
    }
}

fn update_info_panel(
    mut text_query: Query<&mut Text, With<InfoPanel>>,
    progress_data: Res<ProgressGraphData>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        if let Some(ref project_info) = progress_data.project_info {
            text.sections[0].value = format!(
                "Progress Graph Visualization\n\n{}\n\nNodes: {}\nEdges: {}\n\nControls:\n- WASD: Move camera\n- Right click + drag: Look around\n- Mouse wheel: Zoom\n- Left click: Select node\n\nEdge Colors:\n🔵 leads_to\n🟢 enables\n🟣 implemented_by\n🟠 sequence\n🟡 expanded\n🔴 corrected\n🩵 requires\n🩷 triggers",
                project_info,
                progress_data.node_count,
                progress_data.edge_count
            );
        }
    }
}

fn handle_node_interactions(
    node_query: Query<(Entity, &ProgressNodeComponent, &Transform), With<Clickable>>,
    windows: Query<&Window>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut selected_node: ResMut<SelectedNode>,
    mut metadata_panel_query: Query<(&mut Style, &mut Text), With<MetadataPanel>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Ok(window) = windows.single() {
            if let Some(_cursor_position) = window.cursor_position() {
                // Simplified selection - select first node for demo
                if let Some((entity, progress_node, _transform)) = node_query.iter().next() {
                    selected_node.node_data = Some(progress_node.data.clone());
                    selected_node.node_label = Some(progress_node.label.clone());

                    if let Ok((mut style, mut text)) = metadata_panel_query.single_mut() {
                        style.display = Display::Flex;
                        
                        let status_emoji = match progress_node.data.status.as_str() {
                            "completed" => "✅",
                            "in-progress" => "🔄",
                            _ => "⏳",
                        };

                        let progress_text = if let Some(progress) = progress_node.data.progress {
                            format!("\nProgress: {}%", progress)
                        } else {
                            String::new()
                        };

                        let details_text = if !progress_node.data.details.is_empty() {
                            format!("\n\nDetails:\n• {}", progress_node.data.details.join("\n• "))
                        } else {
                            String::new()
                        };

                        text.sections[0].value = format!(
                            "📊 NODE METADATA\n\n{} {}\n\nType: {}\nStatus: {} {}{}\n\nDescription:\n{}{}\n\n[Right-click to close]",
                            status_emoji,
                            progress_node.label,
                            progress_node.node_type,
                            status_emoji,
                            progress_node.data.status,
                            progress_text,
                            progress_node.data.description,
                            details_text
                        );
                    }

                    println!("🖱️ Selected node: {}", progress_node.label);
                }
            }
        }
    }

    if mouse_button_input.just_pressed(MouseButton::Right) {
        if let Ok((mut style, _)) = metadata_panel_query.single_mut() {
            style.display = Display::None;
        }
    }
} 