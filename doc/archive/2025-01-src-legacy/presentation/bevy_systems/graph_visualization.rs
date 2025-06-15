//! Graph visualization systems

use super::event_animation::{GraphCommand, ScheduledCommand};
use crate::domain::value_objects::NodeId;
use crate::presentation::components::{GraphEdge, GraphNode};
use bevy::prelude::*;
use std::time::Duration;
use tracing::info;

/// Setup 3D scene with camera and lighting
pub fn setup_3d_scene(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

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

/// Create demo graph with K7 complete graph
pub fn create_demo_graph(mut event_writer: EventWriter<ScheduledCommand>) {
    // Create K7 complete graph - 7 nodes arranged in a circle
    let num_nodes = 7;
    let radius = 4.0;
    let animation_duration = 15.0;
    let nodes_duration = 5.0;

    // Store node IDs for edge creation
    let mut node_ids = Vec::new();

    // Create nodes
    for i in 0..num_nodes {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / (num_nodes as f32);
        let x = radius * angle.cos();
        let z = radius * angle.sin();
        let position = Vec3::new(x, 0.0, z);

        let node_id = NodeId::new();
        let delay = Duration::from_secs_f32((i as f32 / num_nodes as f32) * nodes_duration);

        // Schedule node creation
        event_writer.write(ScheduledCommand {
            delay,
            command: GraphCommand::SpawnNode {
                node_id,
                position,
                label: format!("Node {}", i + 1),
            },
        });

        node_ids.push(node_id);
    }

    // Note: For the demo, we'll create edges based on node positions after nodes are created
    // In a real system, you'd track the entities as they're created

    info!(
        "Scheduled K7 complete graph creation with {} nodes over {} seconds",
        num_nodes, animation_duration
    );
}

/// System to draw edges between nodes
pub fn draw_edges(
    mut gizmos: Gizmos,
    edges: Query<&GraphEdge>,
    nodes: Query<(Entity, &Transform), With<GraphNode>>,
) {
    for edge in edges.iter() {
        // Find source and target positions
        let mut source_pos = None;
        let mut target_pos = None;

        for (entity, transform) in nodes.iter() {
            if entity == edge.source {
                source_pos = Some(transform.translation);
            }
            if entity == edge.target {
                target_pos = Some(transform.translation);
            }
        }

        if let (Some(source), Some(target)) = (source_pos, target_pos) {
            gizmos.line(source, target, Color::srgb(0.6, 0.6, 0.6));
        }
    }
}
