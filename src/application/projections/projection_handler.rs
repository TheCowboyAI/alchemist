//! Projection Handler
//!
//! Subscribes to domain events and updates projections accordingly.
//! This is the bridge between the event store and read models.

use crate::domain::events::DomainEvent;
use crate::infrastructure::event_store::EventStore;
use crate::application::projections::{Projection, GraphSummaryProjection};
use async_nats::jetstream;
use bevy::prelude::*;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use futures::StreamExt;
use tracing::{error, warn};
use anyhow::Result;

/// Manages projection updates from event streams
pub struct ProjectionHandler {
    event_store: Arc<dyn EventStore>,
    graph_summary: Arc<RwLock<GraphSummaryProjection>>,
    consumer: Option<jetstream::consumer::Consumer<jetstream::consumer::pull::Config>>,
}

impl ProjectionHandler {
    pub fn new(
        event_store: Arc<dyn EventStore>,
        graph_summary: Arc<RwLock<GraphSummaryProjection>>,
    ) -> Self {
        Self {
            event_store,
            graph_summary,
            consumer: None,
        }
    }

    /// Start consuming events and updating projections
    pub async fn start(&mut self, jetstream: jetstream::Context) -> Result<()> {
        // Create or get the events stream
        let stream = jetstream.get_or_create_stream(jetstream::stream::Config {
            name: "EVENTS".to_string(),
            subjects: vec!["events.>".to_string()],
            retention: jetstream::stream::RetentionPolicy::Limits,
            storage: jetstream::stream::StorageType::File,
            ..Default::default()
        }).await?;

        // Create a durable consumer for projections
        let consumer_config = jetstream::consumer::pull::Config {
            name: Some("projection-consumer".to_string()),
            durable_name: Some("projection-consumer".to_string()),
            deliver_policy: jetstream::consumer::DeliverPolicy::All,
            ack_policy: jetstream::consumer::AckPolicy::Explicit,
            ..Default::default()
        };

        let consumer = stream.create_consumer(consumer_config).await?;
        self.consumer = Some(consumer.clone());

        // Start processing events in background
        let graph_summary = self.graph_summary.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::process_events(consumer, graph_summary).await {
                error!("Projection handler error: {}", e);
            }
        });

        Ok(())
    }

    /// Process events from the consumer
    async fn process_events(
        consumer: jetstream::consumer::Consumer<jetstream::consumer::pull::Config>,
        graph_summary: Arc<RwLock<GraphSummaryProjection>>,
    ) -> Result<()> {
        let mut messages = consumer.messages().await?;

        while let Some(message) = messages.next().await {
            let message = message?;

            // Parse event
            if let Ok(event) = serde_json::from_slice::<DomainEvent>(&message.payload) {
                let sequence = message.info()
                    .map_err(|e| anyhow::anyhow!("Failed to get message info: {}", e))?
                    .stream_sequence;

                // Update projections
                {
                    let mut projection = graph_summary.write().await;
                    if let Err(e) = projection.handle_event(event.clone(), sequence).await {
                        error!("Failed to update graph summary projection: {}", e);
                        // Continue processing other events
                    }
                }

                // TODO: Update other projections as they are implemented

                // Acknowledge message after successful processing
                message.ack().await
                    .map_err(|e| anyhow::anyhow!("Failed to acknowledge message: {}", e))?;
            } else {
                warn!("Failed to parse event from message");
                // Still acknowledge to avoid reprocessing
                message.ack().await
                    .map_err(|e| anyhow::anyhow!("Failed to acknowledge message: {}", e))?;
            }
        }

        Ok(())
    }

    /// Replay events from a specific sequence number
    pub async fn replay_from(&mut self, start_sequence: u64, jetstream: jetstream::Context) -> Result<()> {
        // Get the events stream
        let stream = jetstream.get_stream("EVENTS").await?;

        let consumer_name = format!("replay-consumer-{}", uuid::Uuid::new_v4());
        let replay_config = jetstream::consumer::pull::Config {
            name: Some(consumer_name.clone()),
            deliver_policy: jetstream::consumer::DeliverPolicy::ByStartSequence { start_sequence },
            ack_policy: jetstream::consumer::AckPolicy::None, // No acks needed for replay
            ..Default::default()
        };

        let replay_consumer = stream.create_consumer(replay_config).await?;
        let mut messages = replay_consumer.messages().await?;

        // Reset projections before replay
        {
            let mut projection = self.graph_summary.write().await;
            *projection = GraphSummaryProjection::new();
        }

        // Replay events
        while let Some(message) = messages.next().await {
            let message = message?;
            let sequence = message.info()
                .map_err(|e| anyhow::anyhow!("Failed to get message info: {}", e))?
                .stream_sequence;

            if let Ok(event) = serde_json::from_slice::<DomainEvent>(&message.payload) {
                let mut projection = self.graph_summary.write().await;
                projection.handle_event(event, sequence).await?;
            }
        }

        // Clean up replay consumer - use the stream to delete it
        stream.delete_consumer(&consumer_name).await?;

        Ok(())
    }

    /// Get a read-only reference to the graph summary projection
    pub fn graph_summary(&self) -> Arc<RwLock<GraphSummaryProjection>> {
        self.graph_summary.clone()
    }
}

/// Bevy plugin for integrating projections
pub struct ProjectionPlugin;

impl Plugin for ProjectionPlugin {
    fn build(&self, app: &mut App) {
        // Initialize projections as resources
        app.insert_resource(GraphSummaryProjection::new());

        // TODO: Add systems to sync projections with Bevy state
        app.add_systems(Update, sync_graph_summary_to_bevy);
    }
}

/// System to sync GraphSummaryProjection to Bevy resources
fn sync_graph_summary_to_bevy(
    projection: Res<GraphSummaryProjection>,
    mut graph_projection: ResMut<crate::application::projections::GraphProjection>,
) {
    // Update GraphProjection resource from GraphSummaryProjection
    for summary in projection.get_all_summaries() {
        if let Some(graph_state) = graph_projection.graphs.get_mut(&summary.id) {
            // Update metadata if changed
            graph_state.name = summary.metadata.name.clone();
            graph_state.description = summary.metadata.tags.first().cloned();
        } else {
            // Create new graph state
            use crate::application::projections::GraphState;

            let state = GraphState {
                id: summary.id,
                name: summary.metadata.name.clone(),
                description: summary.metadata.tags.first().cloned(),
                nodes: HashMap::new(),
                edges: HashMap::new(),
            };

            graph_projection.graphs.insert(summary.id, state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::events::GraphEvent;
    use crate::domain::value_objects::{GraphMetadata, GraphId};

    #[tokio::test]
    async fn test_projection_handler_updates() {
        let graph_summary = Arc::new(RwLock::new(GraphSummaryProjection::new()));
        let graph_id = GraphId::new();

        // Create test event
        let event = DomainEvent::Graph(GraphEvent::GraphCreated {
            id: graph_id,
            metadata: GraphMetadata {
                name: "Test Graph".to_string(),
                description: None,
                tags: vec![],
                properties: HashMap::new(),
            },
        });

        // Update projection
        {
            let mut projection = graph_summary.write().await;
            projection.handle_event(event, 1).await.unwrap();
        }

        // Verify update
        {
            let projection = graph_summary.read().await;
            let summary = projection.get_summary(&graph_id).unwrap();
            assert_eq!(summary.metadata.name, "Test Graph");
            assert_eq!(summary.node_count, 0);
            assert_eq!(summary.edge_count, 0);
        }
    }
}
