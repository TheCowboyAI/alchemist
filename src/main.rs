//! Information Alchemist - Main Entry Point

use bevy::prelude::*;
use tracing::info;
use ia::application::CommandEvent;
use ia::infrastructure::event_bridge::{EventBridge, EventBridgePlugin};
use ia::infrastructure::nats::{NatsClient, NatsConfig};
use ia::presentation::plugins::{GraphEditorPlugin, ConceptualGraphPlugin, WorkflowDesignerPlugin};
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
fn setup(commands: EventWriter<CommandEvent>) {
    info!("Starting Information Alchemist");

    // Don't create a graph automatically - let the import system handle it
    info!("Keyboard shortcuts are ready - check the console for instructions");
    info!("Press 'I' to import a graph or use mouse buttons as shown");
    info!("Press 'W' to open the Workflow Designer");
}
