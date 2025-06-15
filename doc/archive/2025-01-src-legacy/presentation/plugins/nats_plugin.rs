//! NATS plugin for Bevy
//!
//! Provides NATS messaging integration for the presentation layer

use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender, bounded};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::{debug, error, info, warn};

use crate::infrastructure::nats::NatsConfig;

/// NATS plugin for Bevy
pub struct NatsPlugin {
    config: NatsConfig,
}

impl NatsPlugin {
    /// Create a new NATS plugin with default configuration
    pub fn new() -> Self {
        Self {
            config: NatsConfig::default(),
        }
    }

    /// Create a new NATS plugin with custom configuration
    pub fn with_config(config: NatsConfig) -> Self {
        Self { config }
    }
}

impl Plugin for NatsPlugin {
    fn build(&self, app: &mut App) {
        // Create channels for async/sync communication
        let (_event_tx, event_rx) = bounded(1000);
        let (command_tx, _command_rx) = bounded(1000);

        // Create runtime for async operations
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");

        // Clone config for the async task
        let config = self.config.clone();

        // Spawn NATS connection task
        runtime.spawn(async move {
            match crate::infrastructure::nats::connect(config).await {
                Ok(_client) => {
                    info!("Connected to NATS");
                    // TODO: Set up subscriptions and message handling
                }
                Err(e) => {
                    error!("Failed to connect to NATS: {}", e);
                }
            }
        });

        // Add resources
        app.insert_resource(NatsRuntime(Arc::new(runtime)))
            .insert_resource(EventChannel { rx: event_rx })
            .insert_resource(CommandChannel { tx: command_tx })
            .add_systems(Update, process_nats_events);
    }
}

/// Resource holding the Tokio runtime
#[derive(Resource)]
struct NatsRuntime(Arc<Runtime>);

/// Channel for receiving events from NATS
#[derive(Resource)]
struct EventChannel {
    rx: Receiver<NatsEvent>,
}

/// Channel for sending commands to NATS
#[derive(Resource)]
struct CommandChannel {
    tx: Sender<NatsCommand>,
}

/// Events received from NATS
#[derive(Debug, Clone)]
enum NatsEvent {
    Connected,
    Disconnected,
    Message { subject: String, payload: Vec<u8> },
}

/// Commands to send via NATS
#[derive(Debug, Clone)]
enum NatsCommand {
    Publish { subject: String, payload: Vec<u8> },
    Subscribe { subject: String },
}

/// System to process NATS events
fn process_nats_events(event_channel: Res<EventChannel>, _commands: Commands) {
    // Process all pending events
    while let Ok(event) = event_channel.rx.try_recv() {
        match event {
            NatsEvent::Connected => {
                info!("NATS connected");
                // TODO: Handle connection event
            }
            NatsEvent::Disconnected => {
                warn!("NATS disconnected");
                // TODO: Handle disconnection event
            }
            NatsEvent::Message { subject, payload } => {
                debug!("Received message on {}: {} bytes", subject, payload.len());
                // TODO: Route message to appropriate handler
            }
        }
    }
}
