//! Information Alchemist - Main Entry Point

use bevy::prelude::*;
use ia::application::CommandEvent;
use ia::domain::commands::{Command, GraphCommand};
use ia::domain::value_objects::GraphId;
use ia::infrastructure::event_bridge::{EventBridge, EventBridgePlugin};
use ia::infrastructure::nats::{NatsClient, NatsConfig};
use ia::presentation::plugins::GraphEditorPlugin;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::info;

fn main() {
    // Create tokio runtime for NATS
    let runtime = Arc::new(
        Runtime::new().expect("Failed to create tokio runtime")
    );

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
        .add_plugins(GraphEditorPlugin);

    // Only add event bridge if NATS is connected
    if let Some(client) = nats_client {
        app.add_plugins(EventBridgePlugin);
        app.add_systems(Startup, move |bridge: Res<EventBridge>| {
            bridge.start(client.clone());
            info!("Event bridge started with NATS connection");
        });
    }

    app.add_systems(Startup, setup)
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

    info!(
        "Initial graph creation command sent with id: {:?}",
        graph_id
    );
}
