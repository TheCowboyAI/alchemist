//! Bridge between Bevy UI events and NATS JetStream
//!
//! This plugin ensures that all UI events are properly published to NATS
//! with correlation IDs, causation IDs, and proper subjects.

use bevy::prelude::*;
use crate::simple_agent::{AgentQuestionEvent, AgentResponseEvent, AgentErrorEvent};
use async_nats::jetstream;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::{info, error};

/// Resource for NATS connection
#[derive(Resource)]
pub struct NatsConnection {
    client: async_nats::Client,
    jetstream: jetstream::Context,
    runtime: Arc<Runtime>,
}

impl NatsConnection {
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
}

/// Event wrapper with metadata for NATS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsEvent<T> {
    pub event_id: Uuid,
    pub event_type: String,
    pub timestamp: String,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
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
                app.add_systems(Update, (
                    publish_question_events,
                    publish_response_events,
                    publish_error_events,
                ));
            }
            Err(e) => {
                error!("Failed to connect to NATS: {}. Events will not be persisted.", e);
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
        tracker.active_correlations.insert(
            event.question.clone(),
            correlation_id,
        );
        
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
                    info!("Published question event {} to NATS (seq: {})", 
                        event_id, ack.await.unwrap().sequence);
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
        let correlation_id = tracker.active_correlations
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
                    info!("Published response event {} to NATS (seq: {})", 
                        event_id, ack.await.unwrap().sequence);
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
        let correlation_id = tracker.active_correlations
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
                    info!("Published error event {} to NATS (seq: {})", 
                        event_id, ack.await.unwrap().sequence);
                }
                Err(e) => {
                    error!("Failed to publish error event: {}", e);
                }
            }
        });
    }
} 