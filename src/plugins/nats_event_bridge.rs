//! Bridge between Bevy UI events and NATS `JetStream`
//!
//! This plugin ensures that all UI events are properly published to NATS
//! with correlation IDs, causation IDs, and proper subjects.

use crate::simple_agent::{AgentErrorEvent, AgentQuestionEvent, AgentResponseEvent};
use async_nats::jetstream;
use bevy::prelude::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::{error, info};
use uuid::Uuid;

/// Resource for NATS connection
#[derive(Resource)]
pub struct NatsConnection {
    client: async_nats::Client,
    jetstream: jetstream::Context,
    runtime: Arc<Runtime>,
}

impl NatsConnection {
    /// Creates a new NATS connection with `JetStream` context
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = async_nats::connect("nats://localhost:4222").await?;
        let jetstream = jetstream::new(client.clone());

        // Ensure stream exists
        let stream_config = jetstream::stream::Config {
            name: "CIM-UI-EVENTS".to_string(),
            subjects: vec!["cim.ui.>".to_string()],
            retention: jetstream::stream::RetentionPolicy::Limits,
            ..Default::default()
        };

        match jetstream.get_or_create_stream(stream_config).await {
            Ok(_) => info!("CIM-UI-EVENTS stream ready"),
            Err(e) => error!("Failed to create stream: {}", e),
        }

        Ok(Self {
            client,
            jetstream,
            runtime: Arc::new(Runtime::new()?),
        })
    }

    /// Publishes a message to a core NATS subject (non-JetStream)
    pub async fn publish_core(
        &self,
        subject: &str,
        payload: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.client
            .publish(subject.to_string(), payload.into())
            .await?;
        Ok(())
    }

    /// Subscribes to a core NATS subject for real-time updates
    pub async fn subscribe_core(
        &self,
        subject: &str,
    ) -> Result<async_nats::Subscriber, Box<dyn std::error::Error>> {
        Ok(self.client.subscribe(subject.to_string()).await?)
    }

    /// Gets the server info to check connection status
    #[must_use] pub fn server_info(&self) -> async_nats::ServerInfo {
        // Return the server info directly
        self.client.server_info()
    }

    /// Flushes all pending messages to ensure they're sent
    pub async fn flush(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.client.flush().await?;
        Ok(())
    }
}

/// Event wrapper with metadata for NATS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsEvent<T> {
    /// Unique identifier for this event
    pub event_id: Uuid,
    /// Type name of the event
    pub event_type: String,
    /// ISO 8601 timestamp when the event occurred
    pub timestamp: String,
    /// Correlation ID to track related events
    pub correlation_id: Uuid,
    /// ID of the event that caused this one (None for root events)
    pub causation_id: Option<Uuid>,
    /// The actual event payload
    pub payload: T,
}

/// Tracks correlation between questions and responses
#[derive(Resource, Default)]
pub struct CorrelationTracker {
    active_correlations: std::collections::HashMap<String, Uuid>,
}

/// Plugin to bridge UI events to NATS
pub struct NatsEventBridgePlugin;

impl Plugin for NatsEventBridgePlugin {
    fn build(&self, app: &mut App) {
        // Try to connect to NATS
        let runtime = Runtime::new().expect("Failed to create runtime");
        let nats_result = runtime.block_on(NatsConnection::new());

        match nats_result {
            Ok(connection) => {
                info!("NATS connection established for event bridge");
                app.insert_resource(connection);
                app.init_resource::<CorrelationTracker>();
                app.add_systems(
                    Update,
                    (
                        publish_question_events,
                        publish_response_events,
                        publish_error_events,
                    ),
                );
            }
            Err(e) => {
                error!(
                    "Failed to connect to NATS: {}. Events will not be persisted.",
                    e
                );
            }
        }
    }
}

/// Publish question events to NATS
fn publish_question_events(
    mut events: EventReader<AgentQuestionEvent>,
    nats: Option<Res<NatsConnection>>,
    mut tracker: ResMut<CorrelationTracker>,
) {
    let Some(nats) = nats else { return };

    for event in events.read() {
        let correlation_id = Uuid::new_v4();
        let event_id = Uuid::new_v4();

        // Track correlation for response matching
        tracker
            .active_correlations
            .insert(event.question.clone(), correlation_id);

        let nats_event = NatsEvent {
            event_id,
            event_type: "AgentQuestionEvent".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            correlation_id,
            causation_id: None, // Questions are root events
            payload: serde_json::json!({
                "question": event.question,
            }),
        };

        let subject = "cim.ui.agent.question";
        let payload = serde_json::to_vec(&nats_event).unwrap_or_default();

        let jetstream = nats.jetstream.clone();
        nats.runtime.spawn(async move {
            match jetstream.publish(subject, payload.into()).await {
                Ok(ack) => {
                    info!(
                        "Published question event {} to NATS (seq: {})",
                        event_id,
                        ack.await.unwrap().sequence
                    );
                }
                Err(e) => {
                    error!("Failed to publish question event: {}", e);
                }
            }
        });
    }
}

/// Publish response events to NATS
fn publish_response_events(
    mut events: EventReader<AgentResponseEvent>,
    nats: Option<Res<NatsConnection>>,
    tracker: Res<CorrelationTracker>,
) {
    let Some(nats) = nats else { return };

    for event in events.read() {
        let event_id = Uuid::new_v4();

        // Try to find correlation ID from recent questions
        let correlation_id = tracker
            .active_correlations
            .values()
            .next()
            .copied()
            .unwrap_or_else(Uuid::new_v4);

        let nats_event = NatsEvent {
            event_id,
            event_type: "AgentResponseEvent".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            correlation_id,
            causation_id: Some(correlation_id), // Response is caused by question
            payload: serde_json::json!({
                "response": event.response,
            }),
        };

        let subject = "cim.ui.agent.response";
        let payload = serde_json::to_vec(&nats_event).unwrap_or_default();

        let jetstream = nats.jetstream.clone();
        nats.runtime.spawn(async move {
            match jetstream.publish(subject, payload.into()).await {
                Ok(ack) => {
                    info!(
                        "Published response event {} to NATS (seq: {})",
                        event_id,
                        ack.await.unwrap().sequence
                    );
                }
                Err(e) => {
                    error!("Failed to publish response event: {}", e);
                }
            }
        });
    }
}

/// Publish error events to NATS
fn publish_error_events(
    mut events: EventReader<AgentErrorEvent>,
    nats: Option<Res<NatsConnection>>,
    tracker: Res<CorrelationTracker>,
) {
    let Some(nats) = nats else { return };

    for event in events.read() {
        let event_id = Uuid::new_v4();

        // Try to find correlation ID
        let correlation_id = tracker
            .active_correlations
            .values()
            .next()
            .copied()
            .unwrap_or_else(Uuid::new_v4);

        let nats_event = NatsEvent {
            event_id,
            event_type: "AgentErrorEvent".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            correlation_id,
            causation_id: Some(correlation_id),
            payload: serde_json::json!({
                "error": event.error,
            }),
        };

        let subject = "cim.ui.agent.error";
        let payload = serde_json::to_vec(&nats_event).unwrap_or_default();

        let jetstream = nats.jetstream.clone();
        nats.runtime.spawn(async move {
            match jetstream.publish(subject, payload.into()).await {
                Ok(ack) => {
                    info!(
                        "Published error event {} to NATS (seq: {})",
                        event_id,
                        ack.await.unwrap().sequence
                    );
                }
                Err(e) => {
                    error!("Failed to publish error event: {}", e);
                }
            }
        });
    }
}
