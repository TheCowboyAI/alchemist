//! Visual Demo of CIM-Viz-Bevy
//!
//! This demo shows how to use the cim-viz-bevy library to create
//! an interactive graph visualization with Bevy.
//!
//! Run with: cargo run --example visual_demo --features bevy/dynamic_linking

use bevy::prelude::*;
use cim_viz_bevy::*;
use cim_contextgraph::{NodeId, EdgeId, ContextGraphId as GraphId};
use std::collections::HashMap;

/// Animation component for smooth transitions
#[derive(Component)]
struct AnimatedTransition {
    start_position: Vec3,
    target_position: Vec3,
    progress: f32,
    duration: f32,
}

/// Marker for the graph camera
#[derive(Component)]
struct GraphCamera;

/// Camera controller component
#[derive(Component, Default)]
struct CameraController {
    pub default_position: Vec3,
    pub default_target: Vec3,
}

/// Material handle component
#[derive(Component)]
struct NodeMaterial(Handle<StandardMaterial>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CIM Graph Visualization Demo".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        // Add our visualization plugin
        .add_plugins(CimVizPlugin::default())
        .add_plugins(CimVizDebugPlugin)
        // Demo-specific resources
        .insert_resource(DemoState::default())
        .insert_resource(NodeEntityMap::default())
        // Setup
        .add_systems(Startup, (setup_scene, create_demo_graph))
        // Visualization systems
        .add_systems(Update, (
            handle_node_creation,
            handle_edge_creation,
            handle_node_removal,
            handle_mouse_interaction,
            update_node_positions,
            animate_nodes,
            handle_keyboard_input,
            camera_controls,
            update_info_text,
            handle_domain_events,
        ))
        .run();
}

/// Demo state tracking
#[derive(Resource, Default)]
struct DemoState {
    current_graph_id: Option<GraphId>,
    selected_node: Option<(Entity, NodeId)>,
    hovering_node: Option<(Entity, NodeId)>,
    node_count: usize,
    edge_count: usize,
}

/// Map from NodeId to Entity for quick lookups
#[derive(Resource, Default)]
struct NodeEntityMap {
    map: HashMap<NodeId, Entity>,
}

/// Marker for info text
#[derive(Component)]
struct InfoText;

/// Setup the 3D scene
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    let camera_pos = Vec3::new(0.0, 15.0, 30.0);
    let camera_target = Vec3::ZERO;

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(camera_pos).looking_at(camera_target, Vec3::Y),
        GraphCamera,
        CameraController {
            default_position: camera_pos,
            default_target: camera_target,
        },
    ));

    // Lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, -0.5, 0.0)),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
        affects_lightmapped_meshes: false,
    });

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.2),
            ..default()
        })),
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));

    // UI Text
    commands.spawn((
        Text::new("CIM Graph Demo\nPress SPACE to add nodes\nClick nodes to select\nPress D to delete selected\nPress R to reset graph"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        InfoText,
    ));
}

/// Create an initial demo graph
fn create_demo_graph(
    mut demo_state: ResMut<DemoState>,
    bridge: Res<CategoricalBridge>,
) {
    let graph_id = GraphId::new();
    demo_state.current_graph_id = Some(graph_id);

    // Create a small initial graph through the bridge
    let positions = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::new(-5.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, -5.0),
    ];

    // Send events to create nodes
    let sender = bridge.domain_sender();
    let mut node_ids = Vec::new();

    for (i, pos) in positions.iter().enumerate() {
        let node_id = NodeId::new();
        node_ids.push(node_id);

        let event = DomainEvent::NodeAdded {
            graph_id,
            node_id,
            position: Some(*pos),
            metadata: serde_json::json!({
                "name": format!("Node {}", i + 1),
                "type": if i == 0 { "Central" } else { "Peripheral" }
            }),
        };

        let _ = sender.send(event);
    }

    // Create edges from center to periphery
    for i in 1..node_ids.len() {
        let event = DomainEvent::EdgeAdded {
            graph_id,
            edge_id: EdgeId::new(),
            source: node_ids[0],
            target: node_ids[i],
            metadata: serde_json::json!({
                "weight": 1.0,
                "type": "Connection"
            }),
        };

        let _ = sender.send(event);
    }
}

/// Handle node creation events from the domain
fn handle_node_creation(
    mut commands: Commands,
    mut create_events: EventReader<CreateNodeVisual>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut node_map: ResMut<NodeEntityMap>,
    mut demo_state: ResMut<DemoState>,
) {
    for event in create_events.read() {
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.7, 0.3),
            metallic: 0.3,
            perceptual_roughness: 0.7,
            ..default()
        });

        let entity = commands.spawn((
            // Use the bundle from cim-viz-bevy
            NodeVisualBundle::new(event.node_id, event.graph_id, event.position),
            // Add mesh for rendering
            Mesh3d(meshes.add(Sphere::new(0.5).mesh().ico(3).unwrap())),
            MeshMaterial3d(material.clone()),
            // Store material handle
            NodeMaterial(material),
            // Add animation component
            AnimatedTransition {
                start_position: event.position + Vec3::Y * 10.0,
                target_position: event.position,
                progress: 0.0,
                duration: 1.0,
            },
        )).id();

        node_map.map.insert(event.node_id, entity);
        demo_state.node_count += 1;
    }
}

/// Handle edge creation events
fn handle_edge_creation(
    mut commands: Commands,
    mut create_events: EventReader<CreateEdgeVisual>,
    node_map: Res<NodeEntityMap>,
    mut demo_state: ResMut<DemoState>,
) {
    for event in create_events.read() {
        if let (Some(&source_entity), Some(&target_entity)) = (
            node_map.map.get(&event.source_id),
            node_map.map.get(&event.target_id),
        ) {
            commands.spawn(
                EdgeVisualBundle::new(event.edge_id, event.graph_id, source_entity, target_entity)
            );
            demo_state.edge_count += 1;
        }
    }
}

/// Handle node removal
fn handle_node_removal(
    mut commands: Commands,
    mut remove_events: EventReader<RemoveNodeVisual>,
    node_map: Res<NodeEntityMap>,
    mut demo_state: ResMut<DemoState>,
) {
    for event in remove_events.read() {
        if let Some(&entity) = node_map.map.get(&event.node_id) {
            commands.entity(entity).despawn();
            demo_state.node_count = demo_state.node_count.saturating_sub(1);
        }
    }
}

/// Handle mouse interaction
fn handle_mouse_interaction(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<GraphCamera>>,
    nodes: Query<(Entity, &NodeVisual, &Transform, &NodeMaterial)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut demo_state: ResMut<DemoState>,
    mut node_click_events: EventWriter<NodeClicked>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera.single() else { return };

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            let mut hovered_node = None;

            // Check for node intersection
            for (entity, node_visual, transform, node_material) in nodes.iter() {
                // Simple sphere intersection test
                let sphere_center = transform.translation;
                let sphere_radius = 0.6; // Slightly larger than visual radius

                // Find closest point on ray to sphere center
                let ray_origin = ray.origin;
                let ray_direction = ray.direction.as_vec3();
                let to_sphere = sphere_center - ray_origin;
                let t = to_sphere.dot(ray_direction).max(0.0);
                let closest_point = ray_origin + ray_direction * t;
                let distance = (closest_point - sphere_center).length();

                if distance < sphere_radius {
                    hovered_node = Some((entity, node_visual.node_id));

                    // Update material for hover effect
                    if let Some(material) = materials.get_mut(&node_material.0) {
                        if demo_state.hovering_node != hovered_node {
                            material.base_color = Color::srgb(0.4, 0.8, 0.4);
                        }
                    }

                    // Handle click
                    if mouse_button.just_pressed(MouseButton::Left) {
                        demo_state.selected_node = Some((entity, node_visual.node_id));

                        node_click_events.write(NodeClicked {
                            entity,
                            node_id: node_visual.node_id,
                            graph_id: node_visual.graph_id,
                            world_position: transform.translation,
                        });

                        // Update material for selection
                        if let Some(material) = materials.get_mut(&node_material.0) {
                            material.base_color = Color::srgb(0.8, 0.4, 0.4);
                        }
                    }

                    break;
                }
            }

            // Reset hover state for non-hovered nodes
            if demo_state.hovering_node != hovered_node {
                if let Some((old_entity, _)) = demo_state.hovering_node {
                    if let Ok((_, _, _, node_material)) = nodes.get(old_entity) {
                        if let Some(material) = materials.get_mut(&node_material.0) {
                            if demo_state.selected_node.map(|(e, _)| e) != Some(old_entity) {
                                material.base_color = Color::srgb(0.3, 0.7, 0.3);
                            }
                        }
                    }
                }
                demo_state.hovering_node = hovered_node;
            }
        }
    }
}

/// Handle keyboard input
fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut demo_state: ResMut<DemoState>,
    bridge: Res<CategoricalBridge>,
    mut node_map: ResMut<NodeEntityMap>,
    mut commands: Commands,
    edges: Query<(Entity, &EdgeVisual)>,
    nodes: Query<(Entity, &NodeVisual)>,
) {
    // Debug: Check if any key is pressed
    for key in keyboard.get_just_pressed() {
        println!("⌨️  Key pressed: {:?}", key);
    }

    if keyboard.just_pressed(KeyCode::Space) {
        println!("🚀 SPACE key - Adding new node");
        // Add a new node at a random position
        if let Some(graph_id) = demo_state.current_graph_id {
            let pos = Vec3::new(
                (rand::random::<f32>() - 0.5) * 20.0,
                0.0,
                (rand::random::<f32>() - 0.5) * 20.0,
            );

            let node_id = NodeId::new();
            println!("  📤 Sending command: AddNode {{ node_id: {:?}, position: {:?} }}", node_id, pos);

            let event = DomainEvent::NodeAdded {
                graph_id,
                node_id,
                position: Some(pos),
                metadata: serde_json::json!({
                    "type": "dynamic",
                    "created_at": chrono::Utc::now().to_rfc3339()
                }),
            };

            bridge.send_domain_event(event);
        }
    }

    if keyboard.just_pressed(KeyCode::KeyD) {
        println!("🗑️  D key - Deleting selected node");
        // Delete selected node
        if let Some(selected_node_id) = demo_state.selected_node {
            if let Some(graph_id) = demo_state.current_graph_id {
                println!("  📤 Sending command: RemoveNode {{ node_id: {:?} }}", selected_node_id);

                // Send remove event
                let event = DomainEvent::NodeRemoved {
                    graph_id,
                    node_id: selected_node_id,
                };
                bridge.send_domain_event(event);

                // Also remove connected edges
                for (edge_entity, edge_visual) in edges.iter() {
                    if edge_visual.source == selected_node_id || edge_visual.target == selected_node_id {
                        println!("  📤 Sending command: RemoveEdge {{ edge_id: {:?} }}", edge_visual.edge_id);
                        let edge_event = DomainEvent::EdgeRemoved {
                            graph_id,
                            edge_id: edge_visual.edge_id,
                        };
                        bridge.send_domain_event(edge_event);
                    }
                }

                // Clear selection
                demo_state.selected_node = None;
            }
        }
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        println!("🔄 R key - Resetting entire graph");
        if let Some(graph_id) = demo_state.current_graph_id {
            // Remove all existing nodes and edges
            println!("  🧹 Clearing all nodes and edges...");
            for (entity, node_visual) in nodes.iter() {
                println!("    📤 Sending command: RemoveNode {{ node_id: {:?} }}", node_visual.node_id);
                let event = DomainEvent::NodeRemoved {
                    graph_id,
                    node_id: node_visual.node_id,
                };
                bridge.send_domain_event(event);
                commands.entity(entity).despawn();
            }

            for (entity, edge_visual) in edges.iter() {
                println!("    📤 Sending command: RemoveEdge {{ edge_id: {:?} }}", edge_visual.edge_id);
                let event = DomainEvent::EdgeRemoved {
                    graph_id,
                    edge_id: edge_visual.edge_id,
                };
                bridge.send_domain_event(event);
                commands.entity(entity).despawn();
            }

            // Clear state
            node_map.map.clear();
            demo_state.node_count = 0;
            demo_state.edge_count = 0;
            demo_state.selected_node = None;

            // Recreate initial graph
            println!("  🏗️  Recreating initial graph...");
            let positions = vec![
                Vec3::new(-5.0, 0.0, 0.0),
                Vec3::new(5.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, -5.0),
                Vec3::new(-3.0, 0.0, 3.0),
                Vec3::new(3.0, 0.0, 3.0),
            ];

            let mut node_ids = Vec::new();
            for (i, pos) in positions.iter().enumerate() {
                let node_id = NodeId::new();
                node_ids.push(node_id);

                println!("    📤 Sending command: AddNode {{ node_id: {:?}, position: {:?} }}", node_id, pos);
                let event = DomainEvent::NodeAdded {
                    graph_id,
                    node_id,
                    position: Some(*pos),
                    metadata: serde_json::json!({
                        "type": "initial",
                        "index": i
                    }),
                };
                bridge.send_domain_event(event);
            }

            // Create edges
            let edge_pairs = vec![
                (0, 1), (1, 2), (2, 0), (0, 3), (1, 4), (3, 4)
            ];

            for (source_idx, target_idx) in edge_pairs {
                let edge_id = EdgeId::new();
                println!("    📤 Sending command: AddEdge {{ edge_id: {:?}, source: {:?}, target: {:?} }}",
                    edge_id, node_ids[source_idx], node_ids[target_idx]);

                let event = DomainEvent::EdgeAdded {
                    graph_id,
                    edge_id,
                    source: node_ids[source_idx],
                    target: node_ids[target_idx],
                    relationship: EdgeRelationship::Connected,
                };
                bridge.send_domain_event(event);
            }
        }
    }
}

/// Animate nodes dropping in
fn animate_nodes(
    mut query: Query<(&mut Transform, &mut AnimatedTransition)>,
    time: Res<Time>,
) {
    for (mut transform, mut transition) in query.iter_mut() {
        if transition.progress < 1.0 {
            transition.progress += time.delta_secs() / transition.duration;
            transition.progress = transition.progress.min(1.0);

            // Smooth easing
            let t = ease_out_cubic(transition.progress);
            transform.translation = transition.start_position.lerp(transition.target_position, t);
        }
    }
}

/// Update node positions from domain events
fn update_node_positions(
    mut position_events: EventReader<NodePositionChanged>,
    mut nodes: Query<(&NodeVisual, &mut Transform)>,
) {
    for event in position_events.read() {
        for (node, mut transform) in nodes.iter_mut() {
            if node.node_id == event.node_id {
                transform.translation = event.new_position;
            }
        }
    }
}

/// Update info text
fn update_info_text(
    mut text_query: Query<&mut Text, With<InfoText>>,
    demo_state: Res<DemoState>,
) {
    if demo_state.is_changed() {
        if let Ok(mut text) = text_query.single_mut() {
            text.0 = format!(
                "CIM Graph Demo\n\
                Nodes: {}\n\
                Edges: {}\n\
                Selected: {}\n\n\
                Press SPACE to add nodes\n\
                Click nodes to select\n\
                Press D to delete selected\n\
                Press R to reset graph",
                demo_state.node_count,
                demo_state.edge_count,
                if demo_state.selected_node.is_some() { "Yes" } else { "No" }
            );
        }
    }
}

/// Easing function for smooth animation
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

// Add rand for demo purposes
mod rand {
    pub fn random<T>() -> T
    where
        rand::distributions::Standard: rand::distributions::Distribution<T>
    {
        use rand::Rng;
        rand::thread_rng().gen()
    }
}

/// Camera control system
fn camera_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<(&mut Transform, &CameraController), With<GraphCamera>>,
) {
    // Reset camera on R key
    if keyboard.just_pressed(KeyCode::KeyR) {
        println!("R key pressed - attempting camera reset");

        match camera_query.single_mut() {
            Ok((mut transform, controller)) => {
                println!("Found camera - current position: {:?}", transform.translation);
                println!("Resetting to position: {:?}", controller.default_position);

                // Create new transform to ensure it's properly updated
                let new_transform = Transform::from_translation(controller.default_position)
                    .looking_at(controller.default_target, Vec3::Y);

                *transform = new_transform;

                println!("Camera reset complete - new position: {:?}", transform.translation);
            }
            Err(e) => {
                println!("Failed to get camera: {:?}", e);
            }
        }
    }

    // You can add more camera controls here (WASD movement, etc.)
}

/// Handle domain events from the bridge
fn handle_domain_events(
    mut events: EventReader<DomainEvent>,
    mut create_node_events: EventWriter<CreateNodeVisual>,
    mut remove_node_events: EventWriter<RemoveNodeVisual>,
    mut create_edge_events: EventWriter<CreateEdgeVisual>,
    mut remove_edge_events: EventWriter<RemoveEdgeVisual>,
) {
    for event in events.read() {
        // Log every domain event
        println!("📨 Domain Event: {:?}", event);

        match event {
            DomainEvent::NodeAdded { graph_id, node_id, position, metadata } => {
                println!("  → Creating visual for node {:?} at {:?}", node_id, position);
                create_node_events.write(CreateNodeVisual {
                    graph_id: *graph_id,
                    node_id: *node_id,
                    position: position.unwrap_or(Vec3::ZERO),
                    metadata: metadata.clone(),
                });
            }
            DomainEvent::NodeRemoved { graph_id, node_id } => {
                println!("  → Removing visual for node {:?}", node_id);
                remove_node_events.write(RemoveNodeVisual {
                    graph_id: *graph_id,
                    node_id: *node_id,
                });
            }
            DomainEvent::EdgeAdded { graph_id, edge_id, source, target, relationship } => {
                println!("  → Creating edge visual {:?} between {:?} and {:?}", edge_id, source, target);
                create_edge_events.write(CreateEdgeVisual {
                    graph_id: *graph_id,
                    edge_id: *edge_id,
                    source: *source,
                    target: *target,
                    relationship: relationship.clone(),
                });
            }
            DomainEvent::EdgeRemoved { graph_id, edge_id } => {
                println!("  → Removing edge visual {:?}", edge_id);
                remove_edge_events.write(RemoveEdgeVisual {
                    graph_id: *graph_id,
                    edge_id: *edge_id,
                });
            }
        }
    }
}
