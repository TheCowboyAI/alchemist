//! Implementation of EventStore trait for DistributedEventStore

use super::{EventStore, EventStoreError};
use crate::domain::events::{
    DomainEvent, NodeEvent, EdgeEvent, WorkflowEvent, SubgraphEvent, ContextBridgeEvent, MetricContextEvent, RuleContextEvent
};
use async_nats::jetstream::{self, stream::Config as StreamConfig};
use async_trait::async_trait;

use uuid::Uuid;
use futures::StreamExt;

/// Wrapper around JetStream for EventStore trait implementation
pub struct DistributedEventStore {
    jetstream: jetstream::Context,
    stream_name: String,
}

impl DistributedEventStore {
    /// Create a new distributed event store
    pub async fn new(jetstream: jetstream::Context) -> Result<Self, EventStoreError> {
        let stream_name = "EVENT-STORE".to_string();

        // Create stream configuration
        let stream_config = StreamConfig {
            name: stream_name.clone(),
            subjects: vec!["events.>".to_string()],
            retention: jetstream::stream::RetentionPolicy::Limits,
            storage: jetstream::stream::StorageType::File,
            max_age: std::time::Duration::from_secs(365 * 24 * 60 * 60),
            duplicate_window: std::time::Duration::from_secs(120),
            ..Default::default()
        };

        // Create or update the stream
        match jetstream.create_stream(stream_config).await {
            Ok(_) => {},
            Err(e) if e.to_string().contains("already exists") => {},
            Err(e) => return Err(EventStoreError::Storage(e.to_string())),
        }

        Ok(Self {
            jetstream,
            stream_name,
        })
    }
}

#[async_trait]
impl EventStore for DistributedEventStore {
    async fn append_events(&self, aggregate_id: String, events: Vec<DomainEvent>) -> Result<(), EventStoreError> {
        for event in events {
            // Determine subject based on event type
            let subject = format!("events.{}.{}", aggregate_id, event.event_type());

            // Serialize event
            let payload = serde_json::to_vec(&event)
                .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;

            // Publish to JetStream
            self.jetstream
                .publish(subject, payload.into())
                .await
                .map_err(|e| EventStoreError::Storage(e.to_string()))?
                .await
                .map_err(|e| EventStoreError::Storage(e.to_string()))?;
        }

        Ok(())
    }

    async fn get_events(&self, aggregate_id: String) -> Result<Vec<DomainEvent>, EventStoreError> {
        // Create subject filter for this aggregate
        let subject = format!("events.{aggregate_id}.>");

        // Get stream handle
        let stream = self.jetstream
            .get_stream(&self.stream_name)
            .await
            .map_err(|e| EventStoreError::Storage(e.to_string()))?;

        // Create consumer for reading events
        let consumer = stream
            .create_consumer(jetstream::consumer::pull::Config {
                filter_subject: subject,
                ..Default::default()
            })
            .await
            .map_err(|e| EventStoreError::Storage(e.to_string()))?;

        // Fetch messages
        let mut messages = consumer
            .fetch()
            .max_messages(10000)
            .messages()
            .await
            .map_err(|e| EventStoreError::Storage(e.to_string()))?;

        let mut events = Vec::new();

        while let Some(Ok(message)) = messages.next().await {
            // Deserialize event
            match serde_json::from_slice::<DomainEvent>(&message.payload) {
                Ok(event) => {
                    events.push(event);
                }
                Err(e) => {
                    tracing::warn!("Failed to deserialize event: {}", e);
                }
            }

            // Acknowledge message
            message.ack().await
                .map_err(|e| EventStoreError::Storage(e.to_string()))?;
        }

        Ok(events)
    }

    async fn store(&self, event: DomainEvent) -> Result<(), EventStoreError> {
        // Legacy method - extract aggregate ID from event
        use crate::domain::events::{GraphEvent};

        let aggregate_id = match &event {
            DomainEvent::Graph(graph_event) => match graph_event {
                GraphEvent::GraphCreated { id, .. } => id.to_string(),
                GraphEvent::GraphDeleted { id } => id.to_string(),
                GraphEvent::GraphRenamed { id, .. } => id.to_string(),
                GraphEvent::GraphTagged { id, .. } => id.to_string(),
                GraphEvent::GraphUntagged { id, .. } => id.to_string(),
                GraphEvent::GraphUpdated { graph_id, .. } => graph_id.to_string(),
                GraphEvent::GraphImportRequested { graph_id, .. } => graph_id.to_string(),
                GraphEvent::GraphImportCompleted { graph_id, .. } => graph_id.to_string(),
                GraphEvent::GraphImportFailed { graph_id, .. } => graph_id.to_string(),
            },
            DomainEvent::Node(node_event) => match node_event {
                NodeEvent::NodeAdded { graph_id, .. } => graph_id.to_string(),
                NodeEvent::NodeRemoved { graph_id, .. } => graph_id.to_string(),
                NodeEvent::NodeUpdated { graph_id, .. } => graph_id.to_string(),
                NodeEvent::NodeMoved { graph_id, .. } => graph_id.to_string(),
                NodeEvent::NodeContentChanged { graph_id, .. } => graph_id.to_string(),
            },
            DomainEvent::Edge(edge_event) => match edge_event {
                EdgeEvent::EdgeConnected { graph_id, .. } => graph_id.to_string(),
                EdgeEvent::EdgeRemoved { graph_id, .. } => graph_id.to_string(),
                EdgeEvent::EdgeUpdated { graph_id, .. } => graph_id.to_string(),
                EdgeEvent::EdgeReversed { graph_id, .. } => graph_id.to_string(),
            },
            DomainEvent::Workflow(workflow_event) => match workflow_event {
                WorkflowEvent::WorkflowCreated(event) => event.workflow_id.to_string(),
                WorkflowEvent::StepAdded(event) => event.workflow_id.to_string(),
                WorkflowEvent::StepsConnected(event) => event.workflow_id.to_string(),
                WorkflowEvent::WorkflowValidated(event) => event.workflow_id.to_string(),
                WorkflowEvent::WorkflowStarted(event) => event.workflow_id.to_string(),
                WorkflowEvent::StepCompleted(event) => event.workflow_id.to_string(),
                WorkflowEvent::WorkflowPaused(event) => event.workflow_id.to_string(),
                WorkflowEvent::WorkflowResumed(event) => event.workflow_id.to_string(),
                WorkflowEvent::WorkflowCompleted(event) => event.workflow_id.to_string(),
                WorkflowEvent::WorkflowFailed(event) => event.workflow_id.to_string(),
            },
            DomainEvent::Subgraph(subgraph_event) => match subgraph_event {
                SubgraphEvent::SubgraphCreated { graph_id, .. } => graph_id.to_string(),
                SubgraphEvent::SubgraphRemoved { graph_id, .. } => graph_id.to_string(),
                SubgraphEvent::SubgraphMoved { graph_id, .. } => graph_id.to_string(),
                SubgraphEvent::NodeAddedToSubgraph { graph_id, .. } => graph_id.to_string(),
                SubgraphEvent::NodeRemovedFromSubgraph { graph_id, .. } => graph_id.to_string(),
            },
            DomainEvent::SubgraphOperation(op_event) => op_event.graph_id().to_string(),
            DomainEvent::ContextBridge(context_bridge_event) => match context_bridge_event {
                ContextBridgeEvent::BridgeCreated { bridge_id, .. } => bridge_id.to_string(),
                ContextBridgeEvent::TranslationRuleAdded { bridge_id, .. } => bridge_id.to_string(),
                ContextBridgeEvent::TranslationRuleRemoved { bridge_id, .. } => bridge_id.to_string(),
                ContextBridgeEvent::ConceptTranslated { bridge_id, .. } => bridge_id.to_string(),
                ContextBridgeEvent::TranslationFailed { bridge_id, .. } => bridge_id.to_string(),
                ContextBridgeEvent::BridgeDeleted { bridge_id, .. } => bridge_id.to_string(),
                ContextBridgeEvent::MappingTypeUpdated { bridge_id, .. } => bridge_id.to_string(),
            },
            DomainEvent::MetricContext(metric_context_event) => match metric_context_event {
                MetricContextEvent::MetricContextCreated { context_id, .. } => context_id.to_string(),
                MetricContextEvent::DistanceSet { context_id, .. } => context_id.to_string(),
                MetricContextEvent::ShortestPathCalculated { context_id, .. } => context_id.to_string(),
                MetricContextEvent::NearestNeighborsFound { context_id, .. } => context_id.to_string(),
                MetricContextEvent::ConceptsClustered { context_id, .. } => context_id.to_string(),
                MetricContextEvent::ConceptsWithinRadiusFound { context_id, .. } => context_id.to_string(),
                MetricContextEvent::MetricPropertiesUpdated { context_id, .. } => context_id.to_string(),
            },
            DomainEvent::RuleContext(rule_event) => match rule_event {
                RuleContextEvent::RuleContextCreated { context_id, .. } => context_id.to_string(),
                RuleContextEvent::RuleAdded { context_id, .. } => context_id.to_string(),
                RuleContextEvent::RuleRemoved { context_id, .. } => context_id.to_string(),
                RuleContextEvent::RuleEnabledChanged { context_id, .. } => context_id.to_string(),
                RuleContextEvent::RulesEvaluated { context_id, .. } => context_id.to_string(),
                RuleContextEvent::ComplianceChecked { context_id, .. } => context_id.to_string(),
                RuleContextEvent::FactsInferred { context_id, .. } => context_id.to_string(),
                RuleContextEvent::ImpactAnalyzed { context_id, .. } => context_id.to_string(),
                RuleContextEvent::RulePriorityUpdated { context_id, .. } => context_id.to_string(),
                RuleContextEvent::FactAdded { context_id, .. } => context_id.to_string(),
                RuleContextEvent::FactRemoved { context_id, .. } => context_id.to_string(),
                RuleContextEvent::RuleActionsExecuted { context_id, .. } => context_id.to_string(),
                RuleContextEvent::RulesValidated { context_id, .. } => context_id.to_string(),
                RuleContextEvent::RulesExported { context_id, .. } => context_id.to_string(),
                RuleContextEvent::RulesImported { context_id, .. } => context_id.to_string(),
                RuleContextEvent::RuleViolated { context_id, .. } => context_id.to_string(),
                RuleContextEvent::RuleExecutionFailed { context_id, .. } => context_id.to_string(),
                RuleContextEvent::CircularDependencyDetected { context_id, .. } => context_id.to_string(),
            },
        };

        self.append_events(aggregate_id, vec![event]).await
    }

    async fn load_events(&self, aggregate_id: Uuid) -> Result<Vec<DomainEvent>, EventStoreError> {
        self.get_events(aggregate_id.to_string()).await
    }

    async fn load_all_events(&self) -> Result<Vec<DomainEvent>, EventStoreError> {
        // Get all events from the stream
        let stream = self.jetstream
            .get_stream(&self.stream_name)
            .await
            .map_err(|e| EventStoreError::Storage(e.to_string()))?;

        let consumer = stream
            .create_consumer(jetstream::consumer::pull::Config {
                ..Default::default()
            })
            .await
            .map_err(|e| EventStoreError::Storage(e.to_string()))?;

        let mut messages = consumer
            .fetch()
            .max_messages(100000)
            .messages()
            .await
            .map_err(|e| EventStoreError::Storage(e.to_string()))?;

        let mut events = Vec::new();

        while let Some(Ok(message)) = messages.next().await {
            match serde_json::from_slice::<DomainEvent>(&message.payload) {
                Ok(event) => events.push(event),
                Err(e) => tracing::warn!("Failed to deserialize event: {}", e),
            }

            message.ack().await
                .map_err(|e| EventStoreError::Storage(e.to_string()))?;
        }

        Ok(events)
    }
}
