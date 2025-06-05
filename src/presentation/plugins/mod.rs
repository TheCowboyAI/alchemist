//! Bevy Plugins for the Presentation Layer

use crate::application::command_handlers::process_commands;
use crate::application::{CommandEvent, EventNotification};
use crate::presentation::components::*;
use crate::domain::events::{DomainEvent, GraphEvent};
use crate::domain::value_objects::{NodeId, EdgeId, GraphId, Position3D};
use bevy::prelude::*;
use tracing::info;

/// Main plugin for the graph editor
pub struct GraphEditorPlugin;

impl Plugin for GraphEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register events
            .add_event::<CommandEvent>()
            .add_event::<EventNotification>()
            // Add systems
            .add_systems(Startup, (setup_camera, setup_lighting))
            .add_systems(
                Update,
                (
                    // Command processing
                    process_commands,
                    // Event handling
                    handle_domain_events,
                    // Visualization updates
                    update_edge_positions,
                ),
            );
    }
}

/// Setup basic 3D camera
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Setup basic lighting
fn setup_lighting(mut commands: Commands) {
    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
        affects_lightmapped_meshes: false,
    });

    // Directional light
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Handle domain events and update the world
fn handle_domain_events(
    mut commands: Commands,
    mut events: EventReader<EventNotification>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.read() {
        info!("Received domain event: {:?}", event.event);

        match &event.event {
            DomainEvent::Graph(graph_event) => match graph_event {
                GraphEvent::GraphCreated { id, metadata } => {
                    create_graph_visualization(&mut commands, *id, &metadata.name);

                    // Create some demo nodes
                    create_demo_nodes(&mut commands, &mut meshes, &mut materials, *id);
                }
                _ => {}
            },
            _ => {}
        }
    }
}

/// Create the graph container
fn create_graph_visualization(commands: &mut Commands, graph_id: GraphId, name: &str) {
    commands.spawn((
        GraphContainer {
            graph_id,
            name: name.to_string(),
        },
        Transform::default(),
        Visibility::default(),
    ));

    info!("Created graph visualization for: {}", name);
}

/// Create demo nodes for visualization
fn create_demo_nodes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    graph_id: GraphId,
) {
    // Node mesh and materials
    let node_mesh = meshes.add(Sphere::new(0.5));
    let node_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.5, 0.8),
        metallic: 0.3,
        perceptual_roughness: 0.6,
        ..default()
    });

    // Edge material
    let edge_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.6, 0.6),
        metallic: 0.1,
        perceptual_roughness: 0.8,
        ..default()
    });

    // Create a few demo nodes
    let positions = vec![
        Vec3::new(-3.0, 0.0, 0.0),
        Vec3::new(3.0, 0.0, 0.0),
        Vec3::new(0.0, 3.0, 0.0),
        Vec3::new(0.0, -3.0, 0.0),
    ];

    let mut node_entities = Vec::new();

    for (i, pos) in positions.iter().enumerate() {
        let node_id = NodeId::new();

        let entity = commands.spawn((
            GraphNode {
                node_id,
                graph_id,
            },
            NodeLabel {
                text: format!("Node {}", i + 1),
            },
            Mesh3d(node_mesh.clone()),
            MeshMaterial3d(node_material.clone()),
            Transform::from_translation(*pos),
        )).id();

        node_entities.push((node_id, entity));
    }

    // Create edges between nodes
    let edge_connections = vec![
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (0, 2),
    ];

    for (source_idx, target_idx) in edge_connections {
        if let (Some((source_id, _)), Some((target_id, _))) =
            (node_entities.get(source_idx), node_entities.get(target_idx)) {

            let edge_id = EdgeId::new();

            // Create a simple cylinder for the edge
            let edge_mesh = meshes.add(Cylinder::new(0.05, 1.0));

            commands.spawn((
                GraphEdge {
                    edge_id,
                    graph_id,
                    source: *source_id,
                    target: *target_id,
                },
                Mesh3d(edge_mesh),
                MeshMaterial3d(edge_material.clone()),
                Transform::default(),
            ));
        }
    }

    info!("Created demo nodes and edges for graph");
}

/// Update edge positions to connect nodes
fn update_edge_positions(
    node_query: Query<(&GraphNode, &Transform), Without<GraphEdge>>,
    mut edge_query: Query<(&GraphEdge, &mut Transform), Without<GraphNode>>,
) {
    for (edge, mut edge_transform) in edge_query.iter_mut() {
        // Find source and target positions
        let mut source_pos = None;
        let mut target_pos = None;

        for (node, transform) in node_query.iter() {
            if node.node_id == edge.source {
                source_pos = Some(transform.translation);
            }
            if node.node_id == edge.target {
                target_pos = Some(transform.translation);
            }
        }

        if let (Some(source), Some(target)) = (source_pos, target_pos) {
            // Calculate edge position and rotation
            let midpoint = (source + target) / 2.0;
            let direction = target - source;
            let distance = direction.length();

            if distance > 0.0 {
                let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());

                edge_transform.translation = midpoint;
                edge_transform.rotation = rotation;
                edge_transform.scale = Vec3::new(1.0, distance, 1.0);
            }
        }
    }
}
