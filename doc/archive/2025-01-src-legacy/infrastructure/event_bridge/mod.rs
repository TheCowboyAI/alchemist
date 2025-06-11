//! Event Bridge - Async/Sync bridge between NATS and Bevy ECS

pub mod subject_router;
pub mod event_sequencer;
pub mod workflow_bridge;

pub use subject_router::{SubjectRouter, SubjectRouterPlugin, SubjectConsumer, RoutedEvent};
pub use event_sequencer::{EventSequencer, EventSequencerPlugin};
pub use workflow_bridge::{WorkflowEventBridge, WorkflowEventBridgePlugin};

use crate::domain::commands::Command;
use crate::domain::events::DomainEvent;
use crate::infrastructure::nats::{NatsClient, NatsError};
use bevy::prelude::*;
use crossbeam_channel::{
    Receiver as CrossbeamReceiver, Sender as CrossbeamSender, TryRecvError, bounded,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tracing::{error, info, warn};

/// Bridge between async NATS and sync Bevy ECS
#[derive(Resource)]
pub struct EventBridge {
    /// Commands from Bevy to NATS (sync -> async)
    command_tx: CrossbeamSender<BridgeCommand>,
    command_rx: Arc<Mutex<CrossbeamReceiver<BridgeCommand>>>,

    /// Events from NATS to Bevy (async -> sync)
    event_tx: UnboundedSender<BridgeEvent>,
    event_rx: Arc<Mutex<UnboundedReceiver<BridgeEvent>>>,

    /// Crossbeam channel for Bevy to receive events
    bevy_event_tx: CrossbeamSender<BridgeEvent>,
    bevy_event_rx: CrossbeamReceiver<BridgeEvent>,

    /// Runtime for async operations
    runtime: Arc<Runtime>,
}

/// Commands sent from Bevy to NATS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeCommand {
    /// Publish a domain event to NATS
    PublishEvent(DomainEvent),

    /// Execute a command that may generate events
    ExecuteCommand(Command),

    /// Subscribe to a NATS subject
    Subscribe(String),

    /// Unsubscribe from a NATS subject
    Unsubscribe(String),
}

/// Events received from NATS for Bevy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeEvent {
    /// Domain event received from NATS
    DomainEvent(DomainEvent),

    /// Connection status changed
    ConnectionStatus(ConnectionStatus),

    /// Error occurred
    Error(String),

    /// Workflow started
    WorkflowStarted {
        workflow_id: crate::domain::value_objects::WorkflowId,
        instance_id: uuid::Uuid,
    },

    /// Workflow step started
    WorkflowStepStarted {
        step_id: crate::domain::value_objects::StepId,
    },

    /// Workflow step completed
    WorkflowStepCompleted {
        step_id: crate::domain::value_objects::StepId,
        duration: f32,
    },

    /// Workflow step failed
    WorkflowStepFailed {
        step_id: crate::domain::value_objects::StepId,
        error: String,
    },

    /// Workflow completed
    WorkflowCompleted {
        workflow_id: crate::domain::value_objects::WorkflowId,
        instance_id: uuid::Uuid,
        total_duration: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Reconnecting,
}

impl EventBridge {
    /// Create a new event bridge
    pub fn new() -> Self {
        // Create channels
        let (command_tx, command_rx) = bounded(1000);
        let (event_tx, event_rx) = unbounded_channel();
        let (bevy_event_tx, bevy_event_rx) = bounded(1000);

        // Create runtime for async operations
        let runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime"),
        );

        Self {
            command_tx,
            command_rx: Arc::new(Mutex::new(command_rx)),
            event_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
            bevy_event_tx,
            bevy_event_rx,
            runtime,
        }
    }

    /// Send a command from Bevy to NATS
    pub fn send_command(&self, command: BridgeCommand) -> Result<(), String> {
        self.command_tx
            .send(command)
            .map_err(|e| format!("Failed to send command: {e}"))
    }

    /// Receive events in Bevy (non-blocking)
    pub fn receive_events(&self) -> Vec<BridgeEvent> {
        let mut events = Vec::new();

        // Collect all available events without blocking
        while let Ok(event) = self.bevy_event_rx.try_recv() {
            events.push(event);
        }

        events
    }

    /// Start the bridge with a NATS client
    pub fn start(&self, nats_client: NatsClient) {
        let command_rx: Arc<Mutex<CrossbeamReceiver<BridgeCommand>>> = Arc::clone(&self.command_rx);
        let event_tx = self.event_tx.clone();
        let bevy_event_tx = self.bevy_event_tx.clone();
        let event_rx = Arc::clone(&self.event_rx);

        // Spawn async task to handle commands
        self.runtime.spawn(async move {
            info!("Event bridge started");

            // Spawn task to forward events from async to sync
            tokio::spawn(async move {
                let mut event_rx = event_rx.lock().await;

                while let Some(event) = event_rx.recv().await {
                    if let Err(e) = bevy_event_tx.send(event) {
                        error!("Failed to forward event to Bevy: {}", e);
                    }
                }
            });

            // Main command processing loop
            loop {
                let command_rx = command_rx.lock().await;

                match command_rx.try_recv() {
                    Ok(command) => {
                        drop(command_rx); // Release lock before processing

                        match command {
                            BridgeCommand::PublishEvent(event) => {
                                if let Err(e) = publish_event(&nats_client, event).await {
                                    error!("Failed to publish event: {}", e);
                                    let _ = event_tx.send(BridgeEvent::Error(e.to_string()));
                                }
                            }
                            BridgeCommand::ExecuteCommand(cmd) => {
                                // TODO: Process command and generate events
                                warn!("Command execution not yet implemented: {:?}", cmd);
                            }
                            BridgeCommand::Subscribe(subject) => {
                                if let Err(e) =
                                    subscribe_to_subject(&nats_client, &subject, event_tx.clone())
                                        .await
                                {
                                    error!("Failed to subscribe to {}: {}", subject, e);
                                    let _ = event_tx.send(BridgeEvent::Error(e.to_string()));
                                }
                            }
                            BridgeCommand::Unsubscribe(subject) => {
                                warn!("Unsubscribe not yet implemented for: {}", subject);
                            }
                        }
                    }
                    Err(TryRecvError::Empty) => {
                        // No commands available, sleep briefly
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    }
                    Err(TryRecvError::Disconnected) => {
                        error!("Command channel disconnected");
                        break;
                    }
                }
            }
        });
    }
}

/// Publish a domain event to NATS
async fn publish_event(client: &NatsClient, event: DomainEvent) -> Result<(), NatsError> {
    let subject = format!("events.{}", event.event_type());
    let payload = serde_json::to_vec(&event)?;

    client.publish(&subject, payload).await?;
    info!("Published event to {}: {:?}", subject, event.event_type());

    Ok(())
}

/// Subscribe to a NATS subject and forward events
async fn subscribe_to_subject(
    client: &NatsClient,
    subject: &str,
    event_tx: UnboundedSender<BridgeEvent>,
) -> Result<(), NatsError> {
    let mut subscriber = client.subscribe(subject).await?;
    info!("Subscribed to NATS subject: {}", subject);

    // Spawn task to handle incoming messages
    tokio::spawn(async move {
        while let Some(message) = subscriber.next().await {
            match serde_json::from_slice::<DomainEvent>(&message.payload) {
                Ok(event) => {
                    if let Err(e) = event_tx.send(BridgeEvent::DomainEvent(event)) {
                        error!("Failed to send event to bridge: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    warn!("Failed to deserialize event: {}", e);
                }
            }
        }
    });

    Ok(())
}

/// Bevy plugin for the event bridge
pub struct EventBridgePlugin;

impl Plugin for EventBridgePlugin {
    fn build(&self, app: &mut App) {
        // Create and insert the event bridge
        let bridge = EventBridge::new();
        app.insert_resource(bridge);

        // Add systems to process bridge events
        app.add_systems(Update, (process_bridge_events, send_events_to_nats));
    }
}

/// System to process events from NATS
fn process_bridge_events(
    bridge: Res<EventBridge>,
    mut event_writer: EventWriter<crate::application::EventNotification>,
) {
    let events = bridge.receive_events();

    for bridge_event in events {
        match bridge_event {
            BridgeEvent::DomainEvent(event) => {
                event_writer.write(crate::application::EventNotification { event });
            }
            BridgeEvent::ConnectionStatus(status) => {
                info!("NATS connection status: {:?}", status);
            }
            BridgeEvent::Error(error) => {
                error!("Bridge error: {}", error);
            }
        }
    }
}

/// System to send events to NATS
fn send_events_to_nats(
    bridge: Res<EventBridge>,
    mut event_reader: EventReader<crate::application::EventNotification>,
) {
    for notification in event_reader.read() {
        if let Err(e) = bridge.send_command(BridgeCommand::PublishEvent(notification.event.clone()))
        {
            error!("Failed to send event to NATS: {}", e);
        }
    }
}

impl Default for EventBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
