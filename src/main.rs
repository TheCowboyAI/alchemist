//! Information Alchemist - Main Entry Point

use bevy::prelude::*;
use ia::presentation::plugins::GraphEditorPlugin;
use ia::application::CommandEvent;
use ia::domain::commands::{Command, GraphCommand};
use ia::domain::value_objects::GraphId;
use tracing::info;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GraphEditorPlugin)
        .add_systems(Startup, setup)
        .run();
}

/// Create an initial graph to demonstrate the system
fn setup(mut commands: EventWriter<CommandEvent>) {
    info!("Starting Information Alchemist");

    // Send a test command
    info!("Creating initial graph through command");
    let graph_id = GraphId::new();

    commands.write(CommandEvent {
        command: Command::Graph(GraphCommand::CreateGraph {
            id: graph_id,
            name: "Test Graph".to_string(),
        }),
    });

    info!("Initial graph creation command sent with id: {:?}", graph_id);
}
