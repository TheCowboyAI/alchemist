//! Simple Markdown Import Demo
//!
//! This demo shows markdown import functionality without NATS dependency.
//!
//! Controls:
//!   M - Import next markdown file
//!   C - Clear current graph
//!   ESC - Exit

use bevy::prelude::*;
use ia::{
    application::CommandEvent,
    domain::{
        commands::{Command, GraphCommand},
        value_objects::{GraphId, NodeId},
    },
    presentation::{
        components::{GraphContainer, GraphEdge, GraphNode},
        plugins::GraphEditorPlugin,
    },
};
use std::collections::HashMap;

fn main() {
    println!("=== Simple Markdown Import Demo ===");
    println!();
    println!("Controls:");
    println!("  M - Import next markdown file");
    println!("  C - Clear current graph");
    println!("  ESC - Exit");
    println!();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GraphEditorPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_markdown_import,
                handle_clear,
                display_stats,
                simple_camera_controls,
                debug_node_positions,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, mut event_writer: EventWriter<CommandEvent>) {
    // Camera - positioned further back to see the full graph
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(200.0, 300.0, 500.0).looking_at(Vec3::new(200.0, 0.0, 0.0), Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // Create initial graph
    let graph_id = GraphId::new();
    event_writer.write(CommandEvent {
        command: Command::Graph(GraphCommand::CreateGraph {
            id: graph_id,
            name: "Markdown Import Demo".to_string(),
            metadata: HashMap::new(),
        }),
    });

    println!("✅ Setup complete. Graph ID: {:?}", graph_id);
    println!("\nPress 'M' to import markdown files!");
}

fn handle_markdown_import(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut event_writer: EventWriter<CommandEvent>,
    mut file_index: Local<usize>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        let files = vec![
            "assets/models/KECO_DDD_Core_Model.md",
            "assets/models/KECO_DDD_LoanOriginationContext.md",
            "assets/models/KECO_DDD_UnderwritingContext.md",
            "assets/models/KECO_DDD_DocumentContext.md",
            "assets/models/KECO_DDD_ClosingContext.md",
        ];

        let file_path = files[*file_index % files.len()];
        *file_index += 1;

        println!("\n📁 Importing markdown file: {}", file_path);

        let graph_id = GraphId::new();
        event_writer.write(CommandEvent {
            command: Command::Graph(GraphCommand::ImportFromFile {
                graph_id,
                file_path: file_path.to_string(),
                format: "mermaid".to_string(),
            }),
        });

        println!("✅ Import command sent for graph: {:?}", graph_id);
    }
}

fn handle_clear(
    keyboard: Res<ButtonInput<KeyCode>>,
    graph_query: Query<&GraphContainer>,
    mut event_writer: EventWriter<CommandEvent>,
) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        if let Ok(container) = graph_query.single() {
            println!("\n🗑️  Clearing graph...");

            event_writer.write(CommandEvent {
                command: Command::Graph(GraphCommand::ClearGraph {
                    graph_id: container.graph_id,
                }),
            });

            println!("✅ Graph cleared");
        }
    }
}

fn display_stats(nodes: Query<&GraphNode>, edges: Query<&GraphEdge>, time: Res<Time>) {
    if time.elapsed_secs() as u32 % 5 == 0 && time.delta_secs() > 0.0 {
        let node_count = nodes.iter().count();
        let edge_count = edges.iter().count();

        println!("\n📊 Stats: {} nodes, {} edges", node_count, edge_count);
    }
}

fn simple_camera_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = camera_query.single_mut() {
        let speed = 200.0 * time.delta_secs();

        // WASD for horizontal movement
        if keyboard.pressed(KeyCode::KeyW) {
            transform.translation.z -= speed;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            transform.translation.z += speed;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            transform.translation.x -= speed;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            transform.translation.x += speed;
        }

        // Q/E for vertical movement
        if keyboard.pressed(KeyCode::KeyQ) {
            transform.translation.y -= speed;
        }
        if keyboard.pressed(KeyCode::KeyE) {
            transform.translation.y += speed;
        }

        // Always look at the center of the graph area
        transform.look_at(Vec3::new(200.0, 0.0, 0.0), Vec3::Y);
    }
}

fn debug_node_positions(
    nodes: Query<(&GraphNode, &Transform), Changed<Transform>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        println!("\n=== Node Positions Debug ===");
        let mut nodes_by_subgraph: HashMap<String, Vec<(NodeId, Vec3)>> = HashMap::new();

        for (node, transform) in nodes.iter() {
            // For now, just group by Y position to see if they're in layers
            let y_group = format!("Y={:.0}", transform.translation.y);
            nodes_by_subgraph
                .entry(y_group)
                .or_insert_with(Vec::new)
                .push((node.node_id, transform.translation));
        }

        for (group, positions) in nodes_by_subgraph.iter() {
            println!("\n{}: {} nodes", group, positions.len());
            for (node_id, pos) in positions {
                println!(
                    "  Node {:?}: ({:.1}, {:.1}, {:.1})",
                    node_id, pos.x, pos.y, pos.z
                );
            }
        }

        println!("\nTotal nodes: {}", nodes.iter().count());
        println!("Press 'P' again to refresh positions");
    }
}
