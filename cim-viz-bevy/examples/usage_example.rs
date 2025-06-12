//! Example of how to use cim-viz-bevy in a Bevy application
//!
//! This example demonstrates how a consuming Bevy app would:
//! 1. Add the CimVizPlugin
//! 2. Implement systems to handle visualization events
//! 3. Connect to a domain layer through the bridge

use bevy::prelude::*;
use cim_viz_bevy::*;
use cim_contextgraph::{NodeId, EdgeId, ContextGraphId as GraphId};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add the CIM visualization plugin
        .add_plugins(CimVizPlugin::default())
        // Optional: Add debug plugin
        .add_plugins(CimVizDebugPlugin)
        // Add app-specific visualization systems
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_node_creation,
            handle_node_removal,
            handle_edge_creation,
            handle_mouse_clicks,
            update_node_positions,
        ))
        .run();
}

/// Basic setup for the visualization
fn setup(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        GraphCamera,
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 4.0)),
    ));
}

/// System to handle node creation events from the domain
fn handle_node_creation(
    mut commands: Commands,
    mut create_events: EventReader<CreateNodeVisual>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in create_events.read() {
        info!("Creating visual for node: {:?}", event.node_id);

        // Spawn the visual representation
        commands.spawn((
            // Use the bundle from cim-viz-bevy
            NodeVisualBundle::new(event.node_id, event.graph_id, event.position),
            // Add mesh for rendering
            Mesh3d(meshes.add(Sphere::new(0.5).mesh())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.7, 0.3),
                ..default()
            })),
        ));
    }
}

/// System to handle node removal events
fn handle_node_removal(
    mut commands: Commands,
    mut remove_events: EventReader<RemoveNodeVisual>,
    nodes: Query<(Entity, &NodeVisual)>,
) {
    for event in remove_events.read() {
        // Find and despawn the entity
        for (entity, node) in nodes.iter() {
            if node.node_id == event.node_id {
                info!("Removing visual for node: {:?}", event.node_id);
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

/// System to handle edge creation events
fn handle_edge_creation(
    mut commands: Commands,
    mut create_events: EventReader<CreateEdgeVisual>,
    nodes: Query<(Entity, &NodeVisual)>,
) {
    for event in create_events.read() {
        // Find source and target entities
        let mut source_entity = None;
        let mut target_entity = None;

        for (entity, node) in nodes.iter() {
            if node.node_id == event.source_id {
                source_entity = Some(entity);
            }
            if node.node_id == event.target_id {
                target_entity = Some(entity);
            }
        }

        if let (Some(source), Some(target)) = (source_entity, target_entity) {
            info!("Creating edge visual between {:?} and {:?}", event.source_id, event.target_id);

            // Spawn edge visual (simplified - real app would render a line/curve)
            commands.spawn(
                EdgeVisualBundle::new(event.edge_id, event.graph_id, source, target)
            );
        }
    }
}

/// System to handle mouse clicks and generate domain events
fn handle_mouse_clicks(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<GraphCamera>>,
    nodes: Query<(Entity, &NodeVisual, &Transform)>,
    mut node_click_events: EventWriter<NodeClicked>,
    mut background_click_events: EventWriter<BackgroundClicked>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera.get_single() else {
        return;
    };

    if let Some(cursor_position) = window.cursor_position() {
        // Convert screen coordinates to world ray
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            let origin = ray.origin;
            let direction = ray.direction;

            // Simple sphere intersection test
            let mut clicked_node = None;
            let mut min_distance = f32::MAX;

            for (entity, node_visual, transform) in nodes.iter() {
                let sphere_center = transform.translation;
                let sphere_radius = 0.5; // Match the sphere mesh size

                // Ray-sphere intersection
                let oc = origin - sphere_center;
                let a = direction.dot(direction);
                let b = 2.0 * oc.dot(direction);
                let c = oc.dot(oc) - sphere_radius * sphere_radius;
                let discriminant = b * b - 4.0 * a * c;

                if discriminant >= 0.0 {
                    let sqrt_discriminant = discriminant.sqrt();
                    let t = (-b - sqrt_discriminant) / (2.0 * a);
                    if t > 0.0 && t < min_distance {
                        min_distance = t;
                        clicked_node = Some((entity, node_visual, origin + direction * t));
                    }
                }
            }

            if let Some((entity, node_visual, hit_point)) = clicked_node {
                // Send node clicked event
                node_click_events.send(NodeClicked {
                    entity,
                    node_id: node_visual.node_id,
                    graph_id: node_visual.graph_id,
                    world_position: hit_point,
                });
            } else {
                // Send background clicked event
                let ground_t = -origin.y / direction.y;
                if ground_t > 0.0 {
                    let ground_point = origin + direction * ground_t;
                    background_click_events.send(BackgroundClicked {
                        world_position: ground_point,
                    });
                }
            }
        }
    }
}

/// System to handle position updates from domain
fn update_node_positions(
    mut position_events: EventReader<NodePositionChanged>,
    mut nodes: Query<(&NodeVisual, &mut Transform)>,
) {
    for event in position_events.read() {
        for (node, mut transform) in nodes.iter_mut() {
            if node.node_id == event.node_id {
                info!("Updating position for node: {:?}", event.node_id);
                transform.translation = event.new_position;
            }
        }
    }
}

/// Example of connecting to a domain layer (would be in a separate module/crate)
fn domain_connection_example(bridge: Res<CategoricalBridge>) {
    // Simulate receiving events from domain
    let domain_event = DomainEvent::NodeAdded {
        graph_id: GraphId::new(),
        node_id: NodeId::new(),
        position: Some(Vec3::new(5.0, 0.0, 5.0)),
        metadata: serde_json::json!({
            "type": "Person",
            "name": "Alice"
        }),
    };

    // In a real app, this would come from an async domain layer
    let sender = bridge.domain_sender();
    let _ = sender.send(domain_event);
}
