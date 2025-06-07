//! Test import flow to debug the issue

use bevy::prelude::*;
use ia::application::CommandEvent;
use ia::domain::{
    commands::{Command, GraphCommand, ImportSource, ImportOptions, graph_commands::MergeBehavior},
    value_objects::{GraphId, Position3D},
    services::ImportFormat,
};
use ia::presentation::plugins::GraphEditorPlugin;
use ia::presentation::components::GraphContainer;
use ia::presentation::systems::{import_file_to_graph, ImportState};
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GraphEditorPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, test_import)
        .run();
}

fn setup(mut commands: EventWriter<CommandEvent>) {
    println!("\n=== TESTING IMPORT FLOW ===\n");

    // Create a graph
    let graph_id = GraphId::new();
    println!("1. Creating graph with ID: {:?}", graph_id);

    commands.write(CommandEvent {
        command: Command::Graph(GraphCommand::CreateGraph {
            id: graph_id,
            name: "Test Graph".to_string(),
            metadata: HashMap::new(),
        }),
    });

    println!("2. Graph created. Press 'T' to test import.\n");
}

fn test_import(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: EventWriter<CommandEvent>,
    graph_query: Query<&GraphContainer>,
    mut import_state: ResMut<ImportState>,
    mut tested: Local<bool>,
) {
    if keyboard.just_pressed(KeyCode::KeyT) && !*tested {
        println!("\n3. 'T' pressed - testing import");

        // Call the import function directly
        import_file_to_graph(
            &mut commands,
            &graph_query,
            "examples/data/sample_graph.json",
            ImportFormat::ArrowsApp,
            &mut import_state,
        );

        *tested = true;
    }
}
