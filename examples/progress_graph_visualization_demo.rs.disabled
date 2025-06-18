//! Progress Graph Visualization Demo
//!
//! This demo demonstrates the complete CIM pipeline with project progress data:
//! 1. Load progress.json from the CIM project 
//! 2. Convert progress data to a ContextGraph representation
//! 3. Visualize with enhanced Bevy rendering including:
//!    - Different edge colors for different relationship types
//!    - Text labels on nodes showing their names
//!    - Click interaction to display rich metadata
//!    - Proper spatial layout based on project timeline
//!
//! ## Mermaid Graph - Progress Visualization Architecture
//! 
//! ```mermaid
//! graph TD
//!     A[progress.json] -->|load| B[ProgressData]
//!     B -->|convert| C[ContextGraph]
//!     C -->|visualize| D[Bevy ECS Components]
//!     D -->|render| E[Interactive 3D Graph]
//!     
//!     F[User Click] -->|ray casting| G[Node Selection]
//!     G -->|display| H[Metadata Panel]
//!     
//!     subgraph "Edge Types"
//!         I[leads_to - Blue]
//!         J[enables - Green] 
//!         K[implemented_by - Purple]
//!         L[sequence - Orange]
//!     end
//!     
//!     subgraph "Node Data"
//!         M[Status: completed/in-progress]
//!         N[Dates: created/completed]
//!         O[Description & Details]
//!         P[Progress Percentage]
//!     end
//! ```

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
            update_node_animations,
        ))
        .run();
}

// ===== Data Structures =====

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProgressData {
    metadata: ProgressMetadata,
    nodes: Vec<ProgressNode>,
    edges: Vec<ProgressEdge>,
    milestones: Vec<Milestone>,
    current_focus: CurrentFocus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProgressMetadata {
    name: String,
    description: String,
    created: String,
    updated: String,
    version: String,
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
    #[serde(default)]
    week: Option<u32>,
    #[serde(default)]
    parent: Option<String>,
    #[serde(default)]
    references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProgressEdge {
    id: String,
    source: String,
    target: String,
    relationship: String,
    label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Milestone {
    name: String,
    phase: String,
    status: String,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    target_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CurrentFocus {
    task: String,
    status: String,
    details: Vec<String>,
}

// ===== ECS Components =====

#[derive(Component)]
struct ProgressGraphEntity;

#[derive(Component)]
struct ProgressNode {
    node_id: String,
    label: String,
    node_type: String,
    data: ProgressNodeData,
}

#[derive(Component)]
struct ProgressEdge {
    edge_id: String,
    source_entity: Entity,
    target_entity: Entity,
    relationship: String,
    label: String,
}

#[derive(Component)]
struct NodeLabel;

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
struct NodeAnimation {
    scale_factor: f32,
    pulse_speed: f32,
}

// ===== Resources =====

#[derive(Resource, Default)]
struct ProgressGraphData {
    node_count: usize,
    edge_count: usize,
    project_info: Option<String>,
    graph: Option<ContextGraph>,
}

#[derive(Resource, Default)]
struct SelectedNode {
    entity: Option<Entity>,
    node_data: Option<ProgressNodeData>,
    node_label: Option<String>,
}

// ===== Edge Type Colors =====

fn get_edge_color(relationship: &str) -> Color {
    match relationship {
        "sequence" => Color::linear_rgb(1.0, 0.5, 0.0), // Orange - sequential flow
        "leads_to" => Color::linear_rgb(0.0, 0.5, 1.0), // Blue - causation
        "enables" => Color::linear_rgb(0.0, 1.0, 0.0),  // Green - enablement
        "implemented_by" => Color::linear_rgb(0.8, 0.0, 1.0), // Purple - implementation
        "expanded_to" | "expanded_by" => Color::linear_rgb(1.0, 1.0, 0.0), // Yellow - expansion
        "corrected_by" | "refactored_by" => Color::linear_rgb(1.0, 0.0, 0.0), // Red - correction
        "requires" | "required" => Color::linear_rgb(0.0, 1.0, 1.0), // Cyan - dependency
        "triggers" | "continues_to" => Color::linear_rgb(1.0, 0.0, 1.0), // Magenta - triggering
        _ => Color::linear_rgb(0.7, 0.7, 0.7), // Gray - other relationships
    }
}

fn get_node_color(node_type: &str, status: &str) -> Color {
    match (node_type, status) {
        (_, "completed") => Color::linear_rgb(0.0, 0.8, 0.0), // Green for completed
        (_, "in-progress") => Color::linear_rgb(1.0, 1.0, 0.0), // Yellow for in-progress
        ("milestone", _) => Color::linear_rgb(0.0, 0.5, 1.0), // Blue for milestones
        ("phase", _) => Color::linear_rgb(1.0, 0.0, 0.5), // Pink for phases
        ("task", _) => Color::linear_rgb(0.5, 0.5, 0.5), // Gray for tasks
        _ => Color::linear_rgb(0.8, 0.8, 0.8), // Light gray default
    }
}

// ===== Systems =====

fn setup_camera(mut commands: Commands) {
    // Spawn camera with enhanced controller
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

    // Add directional light for better 3D visualization
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
                display: Display::None, // Hidden by default
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

    // Load progress.json
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

    // Update progress data
    progress_data.node_count = progress.nodes.len();
    progress_data.edge_count = progress.edges.len();
    progress_data.project_info = Some(format!(
        "{}\nVersion: {}\nLast Updated: {}",
        progress.metadata.name,
        progress.metadata.version,
        progress.metadata.updated
    ));

    // Create context graph
    let mut context_graph = ContextGraph::new();
    let mut node_entity_map = HashMap::new();

    // Create node meshes and materials
    let sphere_mesh = meshes.add(Sphere::new(0.8));
    let cube_mesh = meshes.add(Cuboid::new(1.2, 1.2, 1.2));
    let cylinder_mesh = meshes.add(Cylinder::new(0.6, 1.0));

    // Spawn nodes
    for node in &progress.nodes {
        let position = Vec3::new(
            node.position.x * 0.1, // Scale down positions
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
            ProgressNode {
                node_id: node.id.clone(),
                label: node.label.clone(),
                node_type: node.node_type.clone(),
                data: node.data.clone(),
            },
            Clickable,
            NodeAnimation {
                scale_factor: 1.0,
                pulse_speed: 2.0,
            },
        )).id();

        // Add text label above node
        commands.spawn((
            Text2dBundle {
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
            },
            NodeLabel,
        ));

        node_entity_map.insert(node.id.clone(), entity);
        
        // Add to context graph
        context_graph.add_node(ContextNodeId::new(&node.id), serde_json::json!({
            "label": node.label,
            "type": node.node_type,
            "status": node.data.status,
            "description": node.data.description
        }));
    }

    // Create edge materials
    let mut edge_materials = HashMap::new();
    
    // Spawn edges
    for edge in &progress.edges {
        if let (Some(&source_entity), Some(&target_entity)) = 
            (node_entity_map.get(&edge.source), node_entity_map.get(&edge.target)) {
            
            // Get source and target positions
            let source_pos = commands.entity(source_entity).get::<Transform>()
                .map(|t| t.translation).unwrap_or_default();
            let target_pos = commands.entity(target_entity).get::<Transform>()
                .map(|t| t.translation).unwrap_or_default();

            // Calculate edge properties
            let direction = target_pos - source_pos;
            let length = direction.length();
            let midpoint = source_pos + direction * 0.5;

            // Get or create material for this relationship type
            let material = edge_materials.entry(edge.relationship.clone()).or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: get_edge_color(&edge.relationship),
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })
            }).clone();

            // Create edge as a thin cylinder
            let edge_entity = commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Cylinder::new(0.05, length)),
                    material,
                    transform: Transform::from_translation(midpoint)
                        .looking_at(target_pos, Vec3::Y)
                        .with_rotation(
                            Transform::from_translation(midpoint)
                                .looking_at(target_pos, Vec3::Y)
                                .rotation
                                * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
                        ),
                    ..default()
                },
                ProgressEdge {
                    edge_id: edge.id.clone(),
                    source_entity,
                    target_entity,
                    relationship: edge.relationship.clone(),
                    label: edge.label.clone(),
                },
            )).id();

            // Add to context graph
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

    progress_data.graph = Some(context_graph);
    
    println!("🎨 Visualized {} nodes and {} edges with contextgraph integration", 
             progress_data.node_count, progress_data.edge_count);
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

        // Mouse look (right mouse button)
        let is_right_mouse_pressed = mouse_button_input.pressed(MouseButton::Right);
        
        for event in mouse_motion_events.read() {
            if is_right_mouse_pressed {
                rotation_delta += event.delta * controller.sensitivity * 0.001;
            }
        }

        // Keyboard movement
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

        // Mouse wheel zoom
        for event in mouse_wheel_events.read() {
            let zoom_speed = controller.speed * 3.0;
            translation_delta += transform.forward().as_vec3() * event.y * zoom_speed;
        }

        // Apply rotation
        if rotation_delta.length_squared() > 0.0 {
            let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            let new_yaw = yaw - rotation_delta.x;
            let new_pitch = (pitch - rotation_delta.y).clamp(-1.5, 1.5);
            transform.rotation = Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, 0.0);
        }

        // Apply translation
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
                "Progress Graph Visualization\n\n{}\n\nNodes: {}\nEdges: {}\n\nControls:\n- WASD: Move camera\n- Right click + drag: Look around\n- Mouse wheel: Zoom in/out\n- Left click: Select node for metadata\n\nEdge Colors:\n🔵 leads_to (Blue)\n🟢 enables (Green)\n🟣 implemented_by (Purple)\n🟠 sequence (Orange)\n🟡 expanded (Yellow)\n🔴 corrected (Red)\n🩵 requires (Cyan)\n🩷 triggers (Magenta)\n⚪ other (Gray)",
                project_info,
                progress_data.node_count,
                progress_data.edge_count
            );
        }
    }
}

fn handle_node_interactions(
    mut commands: Commands,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera>>,
    node_query: Query<(Entity, &ProgressNode, &Transform), With<Clickable>>,
    windows: Query<&Window>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut selected_node: ResMut<SelectedNode>,
    mut metadata_panel_query: Query<(&mut Style, &mut Text), With<MetadataPanel>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Ok(window) = windows.single() {
            if let Some(cursor_position) = window.cursor_position() {
                if let Ok((camera, camera_transform)) = camera_query.single() {
                    // Simple distance-based selection (could be improved with ray casting)
                    let mut closest_node = None;
                    let mut closest_distance = f32::INFINITY;

                    for (entity, progress_node, transform) in node_query.iter() {
                        // Project 3D position to screen space (simplified)
                        let world_pos = transform.translation;
                        let distance_to_camera = (world_pos - camera_transform.translation()).length();
                        
                        // Simple selection based on camera distance and rough screen position
                        if distance_to_camera < closest_distance {
                            closest_distance = distance_to_camera;
                            closest_node = Some((entity, progress_node));
                        }
                    }

                    if let Some((entity, progress_node)) = closest_node {
                        // Update selected node
                        selected_node.entity = Some(entity);
                        selected_node.node_data = Some(progress_node.data.clone());
                        selected_node.node_label = Some(progress_node.label.clone());

                        // Update metadata panel
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

                            let date_text = if let Some(ref date) = progress_node.data.completed_date {
                                format!("\nCompleted: {}", date)
                            } else if let Some(ref date) = progress_node.data.date {
                                format!("\nDate: {}", date)
                            } else {
                                String::new()
                            };

                            let details_text = if !progress_node.data.details.is_empty() {
                                format!("\n\nDetails:\n{}", progress_node.data.details.join("\n• "))
                            } else {
                                String::new()
                            };

                            let references_text = if !progress_node.data.references.is_empty() {
                                format!("\n\nReferences:\n{}", progress_node.data.references.join("\n• "))
                            } else {
                                String::new()
                            };

                            text.sections[0].value = format!(
                                "📊 NODE METADATA\n\n{} {}\n\nType: {}\nStatus: {} {}{}{}\n\nDescription:\n{}{}{}\n\n[Click elsewhere to close]",
                                status_emoji,
                                progress_node.label,
                                progress_node.node_type,
                                status_emoji,
                                progress_node.data.status,
                                progress_text,
                                date_text,
                                progress_node.data.description,
                                details_text,
                                references_text
                            );
                        }

                        println!("🖱️  Selected node: {} ({})", progress_node.label, progress_node.data.status);
                    }
                }
            }
        }
    }

    // Close metadata panel on escape
    if mouse_button_input.just_pressed(MouseButton::Right) {
        selected_node.entity = None;
        if let Ok((mut style, _)) = metadata_panel_query.single_mut() {
            style.display = Display::None;
        }
    }
}

fn update_node_animations(
    time: Res<Time>,
    mut node_query: Query<(&mut Transform, &mut NodeAnimation, &ProgressNode)>,
    selected_node: Res<SelectedNode>,
) {
    for (mut transform, mut animation, progress_node) in node_query.iter_mut() {
        // Pulse animation for selected node
        if let Some(selected_entity) = selected_node.entity {
            // Simple ID-based comparison (in real implementation, use Entity comparison)
            if selected_node.node_label.as_ref() == Some(&progress_node.label) {
                let pulse = 1.0 + 0.3 * (time.elapsed_seconds() * animation.pulse_speed).sin();
                animation.scale_factor = pulse;
            } else {
                animation.scale_factor = 1.0;
            }
        } else {
            animation.scale_factor = 1.0;
        }

        // Apply scale
        transform.scale = Vec3::splat(animation.scale_factor);
    }
} 