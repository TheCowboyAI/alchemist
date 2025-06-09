//! Information Alchemist - Main Entry Point

use bevy::prelude::*;
use tracing::info;
use ia::application::CommandEvent;
use ia::infrastructure::event_bridge::{EventBridge, EventBridgePlugin};
use ia::infrastructure::nats::{NatsClient, NatsConfig};
use ia::presentation::plugins::{GraphEditorPlugin, ConceptualGraphPlugin, WorkflowDesignerPlugin, GraphPlugin};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn main() {
    // Create tokio runtime for NATS
    let runtime = Arc::new(Runtime::new().expect("Failed to create tokio runtime"));

    // Initialize NATS client (optional - will work without NATS server)
    let nats_client = runtime.block_on(async {
        match NatsClient::new(NatsConfig::localhost()).await {
            Ok(client) => {
                info!("Successfully connected to NATS");
                Some(client)
            }
            Err(e) => {
                info!("Running without NATS connection: {}", e);
                None
            }
        }
    });

    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(GraphPlugin)
        .add_plugins(GraphEditorPlugin)
        .add_plugins(ConceptualGraphPlugin)
        .add_plugins(WorkflowDesignerPlugin);

    // Only add event bridge if NATS is connected
    if let Some(client) = nats_client {
        app.add_plugins(EventBridgePlugin);
        app.add_systems(Startup, move |bridge: Res<EventBridge>| {
            bridge.start(client.clone());
            info!("Event bridge started with NATS connection");
        });
    }

    app.add_systems(Startup, setup).run();
}

/// Create an initial graph to demonstrate the system
fn setup(mut commands: EventWriter<CommandEvent>) {
    info!("Starting Information Alchemist");

    // Create a graph and import sample data after a short delay
    info!("Creating initial graph and importing sample data...");

    // Create a new graph
    let graph_id = ia::domain::value_objects::GraphId::new();
    commands.write(CommandEvent {
        command: ia::domain::commands::Command::Graph(
            ia::domain::commands::GraphCommand::CreateGraph {
                id: graph_id,
                name: "Demo Graph".to_string(),
                metadata: std::collections::HashMap::new(),
            }
        ),
    });

    // Import sample data
    commands.write(CommandEvent {
        command: ia::domain::commands::Command::Graph(
            ia::domain::commands::GraphCommand::ImportFromFile {
                graph_id,
                file_path: "examples/data/sample_graph.json".to_string(),
                format: "ArrowsApp".to_string(),
            }
        ),
    });

    info!("Keyboard shortcuts are ready:");
    info!("  Press 'I' to import more data");
    info!("  Press 'W' to open the Workflow Designer");
    info!("  Press 'V' for select tool, 'M' for move tool");
    info!("  Press 'N' to create nodes, 'E' to create edges");
}
