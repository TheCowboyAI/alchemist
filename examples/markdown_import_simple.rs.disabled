//! Simple Markdown Import Demo
//!
//! Press 'M' to import a markdown file with Mermaid diagrams

use bevy::prelude::*;
use ia::{
    application::CommandEvent,
    domain::{
        commands::{
            Command, GraphCommand, ImportOptions, ImportSource, graph_commands::MergeBehavior,
        },
        value_objects::{GraphId, Position3D},
    },
    presentation::{
        components::{GraphContainer, GraphNode},
        plugins::GraphEditorPlugin,
    },
};
use std::collections::HashMap;
use tracing::info;

fn main() {
    println!("=== Markdown Import Demo ===");
    println!();
    println!("Controls:");
    println!("  M - Import markdown file");
    println!("  Ctrl+D - Cycle through DDD markdown files");
    println!("  V - Debug: Show node visibility");
    println!("  ESC - Exit");
    println!();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GraphEditorPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_markdown_import, debug_node_visibility))
        .run();
}

fn setup(mut commands: Commands, mut event_writer: EventWriter<CommandEvent>) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
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

    println!("Graph created with ID: {:?}", graph_id);
}

fn handle_markdown_import(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut event_writer: EventWriter<CommandEvent>,
    graph_query: Query<&GraphContainer>,
    mut file_index: Local<usize>,
) {
    // Add debug logging for any key press
    if keyboard.get_just_pressed().len() > 0 {
        info!(
            "Keys pressed: {:?}",
            keyboard.get_just_pressed().collect::<Vec<_>>()
        );
    }

    if keyboard.just_pressed(KeyCode::KeyM) {
        info!("M key detected - importing markdown file");
        if let Ok(container) = graph_query.single() {
            println!("\n📄 Importing KECO_DDD_Core_Model.md...");

            event_writer.write(CommandEvent {
                command: Command::Graph(GraphCommand::ImportGraph {
                    graph_id: container.graph_id,
                    source: ImportSource::File {
                        path: "assets/keco/KECO_DDD_Core_Model.md".to_string(),
                    },
                    format: "mermaid".to_string(),
                    options: ImportOptions {
                        merge_behavior: MergeBehavior::MergePreferImported,
                        id_prefix: Some("core".to_string()),
                        position_offset: Some(Position3D {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        }),
                        mapping: None,
                        validate: true,
                        max_nodes: Some(1000),
                    },
                }),
            });

            println!("✅ Import command sent");
        }
    }

    if keyboard.just_pressed(KeyCode::KeyD) && keyboard.pressed(KeyCode::ControlLeft) {
        info!("Ctrl+D detected - cycling through files");
        if let Ok(container) = graph_query.single() {
            let ddd_files = [
                ("assets/keco/KECO_DDD_Core_Model.md", "core"),
                ("assets/keco/KECO_DDD_LoanOriginationContext.md", "loan"),
                (
                    "assets/keco/KECO_DDD_UnderwritingContext.md",
                    "underwriting",
                ),
                ("assets/keco/KECO_DDD_DocumentContext.md", "document"),
                ("assets/keco/KECO_DDD_ClosingContext.md", "closing"),
            ];

            let (file_path, prefix) = ddd_files[*file_index % ddd_files.len()];
            *file_index += 1;

            println!("\n📄 Importing {}...", file_path);

            // Calculate offset for each import
            let offset_x = (*file_index as f32 - 1.0) * 30.0;

            event_writer.write(CommandEvent {
                command: Command::Graph(GraphCommand::ImportGraph {
                    graph_id: container.graph_id,
                    source: ImportSource::File {
                        path: file_path.to_string(),
                    },
                    format: "mermaid".to_string(),
                    options: ImportOptions {
                        merge_behavior: MergeBehavior::MergePreferImported,
                        id_prefix: Some(format!("{}_{}", prefix, *file_index)),
                        position_offset: Some(Position3D {
                            x: offset_x,
                            y: 0.0,
                            z: 0.0,
                        }),
                        mapping: None,
                        validate: true,
                        max_nodes: Some(1000),
                    },
                }),
            });

            println!("✅ Import command sent (offset: {})", offset_x);
        }
    }
}

fn debug_node_visibility(
    nodes: Query<(Entity, &GraphNode, &Transform, Option<&Visibility>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyV) {
        println!("\n=== Node Visibility Debug ===");
        let node_count = nodes.iter().count();
        println!("Total nodes in scene: {}", node_count);

        for (entity, node, transform, visibility) in nodes.iter() {
            let vis_status = if let Some(vis) = visibility {
                format!("{:?}", vis)
            } else {
                "No Visibility component".to_string()
            };

            println!("Node {:?}:", node.node_id);
            println!("  Entity: {:?}", entity);
            println!("  Position: {:?}", transform.translation);
            println!("  Visibility: {}", vis_status);
        }

        if node_count == 0 {
            println!("No nodes found in the scene!");
        }
        println!("=========================\n");
    }
}
