//! Subject-based event routing for reliable event delivery

use bevy::prelude::*;
use crossbeam_channel::{bounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info, warn};

use crate::domain::events::DomainEvent;
use crate::domain::value_objects::{AggregateId, EventId};

/// Subject-based event router for Bevy
#[derive(Resource)]
pub struct SubjectRouter {
    /// Subject pattern to channel mapping
    routes: Arc<RwLock<HashMap<String, SubjectChannel>>>,

    /// Global event sequence for ordering
    global_sequence: Arc<RwLock<u64>>,

    /// Per-aggregate sequences for ordering within aggregates
    aggregate_sequences: Arc<RwLock<HashMap<AggregateId, u64>>>,

    /// Configuration
    config: RouterConfig,
}

/// Configuration for the subject router
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// Maximum events per channel
    pub channel_capacity: usize,

    /// Enable sequence number tracking
    pub track_sequences: bool,

    /// Enable dead letter queue
    pub enable_dlq: bool,

    /// Maximum retries before DLQ
    pub max_retries: u32,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            channel_capacity: 10_000,
            track_sequences: true,
            enable_dlq: true,
            max_retries: 3,
        }
    }
}

/// A subject-specific channel
pub struct SubjectChannel {
    /// Subject pattern (e.g., "event.graph.>")
    pub pattern: String,

    /// Sender for this subject
    pub sender: Sender<RoutedEvent>,

    /// Receiver for this subject
    pub receiver: Receiver<RoutedEvent>,

    /// Number of subscribers
    pub subscriber_count: usize,

    /// Statistics
    pub stats: ChannelStats,
}

/// Statistics for a channel
#[derive(Default, Debug, Clone)]
pub struct ChannelStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub messages_dropped: u64,
    pub last_error: Option<String>,
}

/// Event with routing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutedEvent {
    /// The domain event
    pub event: DomainEvent,

    /// Subject this event was routed to
    pub subject: String,

    /// Global sequence number
    pub global_sequence: u64,

    /// Aggregate-specific sequence
    pub aggregate_sequence: u64,

    /// Retry count
    pub retry_count: u32,

    /// Timestamp when routed
    pub routed_at: std::time::SystemTime,
}

impl SubjectRouter {
    pub fn new(config: RouterConfig) -> Self {
        Self {
            routes: Arc::new(RwLock::new(HashMap::new())),
            global_sequence: Arc::new(RwLock::new(0)),
            aggregate_sequences: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Register a subject pattern
    pub fn register_subject(&self, pattern: &str) -> Result<Receiver<RoutedEvent>, String> {
        let mut routes = self.routes.write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;

        if routes.contains_key(pattern) {
            let channel = routes.get_mut(pattern).unwrap();
            channel.subscriber_count += 1;
            return Ok(channel.receiver.clone());
        }

        let (sender, receiver) = bounded(self.config.channel_capacity);

        let channel = SubjectChannel {
            pattern: pattern.to_string(),
            sender,
            receiver: receiver.clone(),
            subscriber_count: 1,
            stats: ChannelStats::default(),
        };

        routes.insert(pattern.to_string(), channel);
        info!("Registered subject pattern: {}", pattern);

        Ok(receiver)
    }

    /// Route an event to appropriate subjects
    pub fn route_event(&self, event: DomainEvent) -> Result<Vec<String>, String> {
        let subject = self.event_to_subject(&event);
        let mut routed_to = Vec::new();

        // Get sequences
        let global_seq = self.increment_global_sequence()?;
        let aggregate_seq = self.increment_aggregate_sequence(event.aggregate_id())?;

        let routed_event = RoutedEvent {
            event: event.clone(),
            subject: subject.clone(),
            global_sequence: global_seq,
            aggregate_sequence: aggregate_seq,
            retry_count: 0,
            routed_at: std::time::SystemTime::now(),
        };

        // Route to matching subjects
        let routes = self.routes.read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;

        for (pattern, channel) in routes.iter() {
            if self.matches_pattern(&subject, pattern) {
                match channel.sender.try_send(routed_event.clone()) {
                    Ok(_) => {
                        routed_to.push(pattern.clone());
                    }
                    Err(e) => {
                        warn!("Failed to route to {}: {}", pattern, e);
                        // TODO: Handle DLQ if enabled
                    }
                }
            }
        }

        if routed_to.is_empty() {
            warn!("No subscribers for subject: {}", subject);
        }

        Ok(routed_to)
    }

    /// Convert event to subject
    fn event_to_subject(&self, event: &DomainEvent) -> String {
        match event {
            DomainEvent::Node(node_event) => match node_event {
                crate::domain::events::NodeEvent::NodeAdded { graph_id, .. } =>
                    format!("event.graph.{}.node.added", graph_id),
                crate::domain::events::NodeEvent::NodeRemoved { graph_id, .. } =>
                    format!("event.graph.{}.node.removed", graph_id),
                crate::domain::events::NodeEvent::NodeUpdated { graph_id, .. } =>
                    format!("event.graph.{}.node.updated", graph_id),
                crate::domain::events::NodeEvent::NodeMoved { graph_id, .. } =>
                    format!("event.graph.{}.node.moved", graph_id),
                crate::domain::events::NodeEvent::NodeContentChanged { graph_id, .. } =>
                    format!("event.graph.{}.node.content_changed", graph_id),
            },
            DomainEvent::Edge(edge_event) => match edge_event {
                crate::domain::events::EdgeEvent::EdgeConnected { graph_id, .. } =>
                    format!("event.graph.{}.edge.connected", graph_id),
                crate::domain::events::EdgeEvent::EdgeRemoved { graph_id, .. } =>
                    format!("event.graph.{}.edge.removed", graph_id),
                crate::domain::events::EdgeEvent::EdgeUpdated { graph_id, .. } =>
                    format!("event.graph.{}.edge.updated", graph_id),
                crate::domain::events::EdgeEvent::EdgeReversed { graph_id, .. } =>
                    format!("event.graph.{}.edge.reversed", graph_id),
            },
            DomainEvent::Graph(graph_event) => match graph_event {
                crate::domain::events::GraphEvent::GraphCreated { id, .. } =>
                    format!("event.graph.{}.created", id),
                crate::domain::events::GraphEvent::GraphDeleted { id } =>
                    format!("event.graph.{}.deleted", id),
                crate::domain::events::GraphEvent::GraphRenamed { id, .. } =>
                    format!("event.graph.{}.renamed", id),
                crate::domain::events::GraphEvent::GraphTagged { id, .. } =>
                    format!("event.graph.{}.tagged", id),
                crate::domain::events::GraphEvent::GraphUntagged { id, .. } =>
                    format!("event.graph.{}.untagged", id),
                crate::domain::events::GraphEvent::GraphUpdated { graph_id, .. } =>
                    format!("event.graph.{}.updated", graph_id),
                crate::domain::events::GraphEvent::GraphImportRequested { graph_id, .. } =>
                    format!("event.graph.{}.import_requested", graph_id),
                crate::domain::events::GraphEvent::GraphImportCompleted { graph_id, .. } =>
                    format!("event.graph.{}.import_completed", graph_id),
                crate::domain::events::GraphEvent::GraphImportFailed { graph_id, .. } =>
                    format!("event.graph.{}.import_failed", graph_id),
            },
            DomainEvent::Workflow(workflow_event) => match workflow_event {
                crate::domain::events::WorkflowEvent::WorkflowCreated(evt) =>
                    format!("event.workflow.{}.created", evt.workflow_id),
                crate::domain::events::WorkflowEvent::StepAdded(evt) =>
                    format!("event.workflow.{}.step_added", evt.workflow_id),
                crate::domain::events::WorkflowEvent::StepsConnected(evt) =>
                    format!("event.workflow.{}.steps_connected", evt.workflow_id),
                crate::domain::events::WorkflowEvent::WorkflowValidated(evt) =>
                    format!("event.workflow.{}.validated", evt.workflow_id),
                crate::domain::events::WorkflowEvent::WorkflowStarted(evt) =>
                    format!("event.workflow.{}.started", evt.workflow_id),
                crate::domain::events::WorkflowEvent::StepCompleted(evt) =>
                    format!("event.workflow.{}.step_completed", evt.workflow_id),
                crate::domain::events::WorkflowEvent::WorkflowPaused(evt) =>
                    format!("event.workflow.{}.paused", evt.workflow_id),
                crate::domain::events::WorkflowEvent::WorkflowResumed(evt) =>
                    format!("event.workflow.{}.resumed", evt.workflow_id),
                crate::domain::events::WorkflowEvent::WorkflowCompleted(evt) =>
                    format!("event.workflow.{}.completed", evt.workflow_id),
                crate::domain::events::WorkflowEvent::WorkflowFailed(evt) =>
                    format!("event.workflow.{}.failed", evt.workflow_id),
            },
            DomainEvent::Subgraph(subgraph_event) => match subgraph_event {
                crate::domain::events::SubgraphEvent::SubgraphCreated { graph_id, subgraph_id, .. } =>
                    format!("event.graph.{}.subgraph.{}.created", graph_id, subgraph_id),
                crate::domain::events::SubgraphEvent::SubgraphRemoved { graph_id, subgraph_id } =>
                    format!("event.graph.{}.subgraph.{}.removed", graph_id, subgraph_id),
                crate::domain::events::SubgraphEvent::SubgraphMoved { graph_id, subgraph_id, .. } =>
                    format!("event.graph.{}.subgraph.{}.moved", graph_id, subgraph_id),
                crate::domain::events::SubgraphEvent::NodeAddedToSubgraph { graph_id, subgraph_id, .. } =>
                    format!("event.graph.{}.subgraph.{}.node_added", graph_id, subgraph_id),
                crate::domain::events::SubgraphEvent::NodeRemovedFromSubgraph { graph_id, subgraph_id, .. } =>
                    format!("event.graph.{}.subgraph.{}.node_removed", graph_id, subgraph_id),
            },
        }
    }

    /// Check if subject matches pattern (supports wildcards)
    fn matches_pattern(&self, subject: &str, pattern: &str) -> bool {
        let subject_parts: Vec<&str> = subject.split('.').collect();
        let pattern_parts: Vec<&str> = pattern.split('.').collect();

        if pattern_parts.is_empty() || subject_parts.is_empty() {
            return false;
        }

        // Check for full wildcard
        if pattern_parts.last() == Some(&">") {
            let prefix_parts = &pattern_parts[..pattern_parts.len() - 1];
            return subject_parts.starts_with(prefix_parts);
        }

        // Check exact match with single wildcards
        if pattern_parts.len() != subject_parts.len() {
            return false;
        }

        for (s, p) in subject_parts.iter().zip(pattern_parts.iter()) {
            if p != &"*" && s != p {
                return false;
            }
        }

        true
    }

    fn increment_global_sequence(&self) -> Result<u64, String> {
        let mut seq = self.global_sequence.write()
            .map_err(|e| format!("Failed to acquire sequence lock: {}", e))?;
        *seq += 1;
        Ok(*seq)
    }

    fn increment_aggregate_sequence(&self, aggregate_id: AggregateId) -> Result<u64, String> {
        let mut sequences = self.aggregate_sequences.write()
            .map_err(|e| format!("Failed to acquire sequence lock: {}", e))?;
        let seq = sequences.entry(aggregate_id).or_insert(0);
        *seq += 1;
        Ok(*seq)
    }

    /// Get statistics for all channels
    pub fn get_stats(&self) -> HashMap<String, ChannelStats> {
        self.routes.read()
            .ok()
            .map(|routes| {
                routes.iter()
                    .map(|(k, v)| (k.clone(), v.stats.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Plugin for subject-based routing
pub struct SubjectRouterPlugin;

impl Plugin for SubjectRouterPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SubjectRouter::new(RouterConfig::default()))
            .add_systems(Update, route_domain_events);
    }
}

/// System to route domain events
fn route_domain_events(
    router: Res<SubjectRouter>,
    mut event_reader: EventReader<crate::application::EventNotification>,
) {
    for notification in event_reader.read() {
        match router.route_event(notification.event.clone()) {
            Ok(subjects) => {
                debug!("Routed event to {} subjects", subjects.len());
            }
            Err(e) => {
                error!("Failed to route event: {}", e);
            }
        }
    }
}

/// Consumer for subject-based events
#[derive(Component)]
pub struct SubjectConsumer {
    pub patterns: Vec<String>,
    pub receivers: Vec<Receiver<RoutedEvent>>,
}

impl SubjectConsumer {
    pub fn new(router: &SubjectRouter, patterns: Vec<String>) -> Result<Self, String> {
        let mut receivers = Vec::new();

        for pattern in &patterns {
            let receiver = router.register_subject(pattern)?;
            receivers.push(receiver);
        }

        Ok(Self { patterns, receivers })
    }

    /// Poll for events (non-blocking)
    pub fn poll_events(&self) -> Vec<RoutedEvent> {
        let mut events = Vec::new();

        for receiver in &self.receivers {
            while let Ok(event) = receiver.try_recv() {
                events.push(event);
            }
        }

        // Sort by global sequence to maintain order
        events.sort_by_key(|e| e.global_sequence);
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matching() {
        let router = SubjectRouter::new(RouterConfig::default());

        // Exact match
        assert!(router.matches_pattern("event.graph.node", "event.graph.node"));

        // Single wildcard
        assert!(router.matches_pattern("event.graph.node", "event.*.node"));
        assert!(router.matches_pattern("event.graph.node", "event.graph.*"));

        // Full wildcard
        assert!(router.matches_pattern("event.graph.node.added", "event.graph.>"));
        assert!(router.matches_pattern("event.graph.node", "event.>"));

        // No match
        assert!(!router.matches_pattern("event.graph.node", "event.graph.edge"));
        assert!(!router.matches_pattern("event.graph", "event.graph.node"));
    }
}
