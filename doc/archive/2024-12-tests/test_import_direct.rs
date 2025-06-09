//! Direct test of import functionality

use bevy::prelude::*;
use ia::application::CommandEvent;
use ia::domain::{
    commands::{Command, GraphCommand, ImportSource, ImportOptions, graph_commands::MergeBehavior},
    value_objects::{GraphId, Position3D},
};
use ia::presentation::plugins::GraphEditorPlugin;
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GraphEditorPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, check_for_nodes)
        .run();
}

fn setup(mut commands: EventWriter<CommandEvent>) {
    println!("Starting direct import test");

    // First create a graph
    let graph_id = GraphId::new();
    println!("Creating graph with ID: {:?}", graph_id);

    commands.write(CommandEvent {
        command: Command::Graph(GraphCommand::CreateGraph {
            id: graph_id,
            name: "Import Test Graph".to_string(),
            metadata: HashMap::new(),
        }),
    });

    // Then import into it
    println!("Sending import command for graph: {:?}", graph_id);

    commands.write(CommandEvent {
        command: Command::Graph(GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::File {
                path: "examples/data/sample_graph.json".to_string(),
            },
            format: "arrows_app".to_string(),
            options: ImportOptions {
                merge_behavior: MergeBehavior::AlwaysCreate,
                id_prefix: Some("imported".to_string()),
                position_offset: Some(Position3D {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
                mapping: None,
                validate: true,
                max_nodes: Some(10000),
            },
        }),
    });

    println!("Commands sent!");
}

fn check_for_nodes(
    nodes: Query<&ia::presentation::components::GraphNode>,
    time: Res<Time>,
    mut last_count: Local<usize>,
) {
    let current_count = nodes.iter().count();
    if current_count != *last_count {
        println!("[{:.2}s] Node count changed: {} -> {}",
            time.elapsed_secs(),
            *last_count,
            current_count
        );

        if current_count > 0 {
            println!("Current nodes:");
            for node in nodes.iter() {
                println!("  Node ID: {:?}, Graph ID: {:?}", node.node_id, node.graph_id);
            }
        }

        *last_count = current_count;
    }
}
