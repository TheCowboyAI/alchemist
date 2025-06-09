//! Subject-based event routing for reliable event delivery

use bevy::prelude::*;
use crossbeam_channel::{bounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info, warn};

use crate::domain::events::{
    DomainEvent, GraphEvent, NodeEvent, EdgeEvent, WorkflowEvent, SubgraphEvent, ContextBridgeEvent, MetricContextEvent, RuleContextEvent
};
use crate::domain::value_objects::AggregateId;

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
        let subject = event_to_subject(&event);
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

fn graph_event_to_subject(event: &GraphEvent) -> String {
    match event {
        GraphEvent::GraphCreated { .. } => "event.graph.created".to_string(),
        GraphEvent::GraphDeleted { .. } => "event.graph.deleted".to_string(),
        GraphEvent::GraphRenamed { .. } => "event.graph.renamed".to_string(),
        GraphEvent::GraphTagged { .. } => "event.graph.tagged".to_string(),
        GraphEvent::GraphUntagged { .. } => "event.graph.untagged".to_string(),
        GraphEvent::GraphUpdated { .. } => "event.graph.updated".to_string(),
        GraphEvent::GraphImportRequested { .. } => "event.graph.import_requested".to_string(),
        GraphEvent::GraphImportCompleted { .. } => "event.graph.import_completed".to_string(),
        GraphEvent::GraphImportFailed { .. } => "event.graph.import_failed".to_string(),
    }
}

fn node_event_to_subject(event: &NodeEvent) -> String {
    match event {
        NodeEvent::NodeAdded { .. } => "event.node.added".to_string(),
        NodeEvent::NodeRemoved { .. } => "event.node.removed".to_string(),
        NodeEvent::NodeUpdated { .. } => "event.node.updated".to_string(),
        NodeEvent::NodeMoved { .. } => "event.node.moved".to_string(),
        NodeEvent::NodeContentChanged { .. } => "event.node.content_changed".to_string(),
    }
}

fn edge_event_to_subject(event: &EdgeEvent) -> String {
    match event {
        EdgeEvent::EdgeConnected { .. } => "event.edge.connected".to_string(),
        EdgeEvent::EdgeRemoved { .. } => "event.edge.removed".to_string(),
        EdgeEvent::EdgeUpdated { .. } => "event.edge.updated".to_string(),
        EdgeEvent::EdgeReversed { .. } => "event.edge.reversed".to_string(),
    }
}

fn subgraph_event_to_subject(event: &SubgraphEvent) -> String {
    match event {
        SubgraphEvent::SubgraphCreated { .. } => "event.subgraph.created".to_string(),
        SubgraphEvent::SubgraphRemoved { .. } => "event.subgraph.removed".to_string(),
        SubgraphEvent::SubgraphMoved { .. } => "event.subgraph.moved".to_string(),
        SubgraphEvent::NodeAddedToSubgraph { .. } => "event.subgraph.node_added".to_string(),
        SubgraphEvent::NodeRemovedFromSubgraph { .. } => "event.subgraph.node_removed".to_string(),
    }
}

fn workflow_event_to_subject(event: &WorkflowEvent) -> String {
    match event {
        WorkflowEvent::WorkflowCreated { .. } => "event.workflow.created".to_string(),
        WorkflowEvent::StepAdded { .. } => "event.workflow.step_added".to_string(),
        WorkflowEvent::StepsConnected { .. } => "event.workflow.steps_connected".to_string(),
        WorkflowEvent::WorkflowValidated { .. } => "event.workflow.validated".to_string(),
        WorkflowEvent::WorkflowStarted { .. } => "event.workflow.started".to_string(),
        WorkflowEvent::StepCompleted { .. } => "event.workflow.step_completed".to_string(),
        WorkflowEvent::WorkflowPaused { .. } => "event.workflow.paused".to_string(),
        WorkflowEvent::WorkflowResumed { .. } => "event.workflow.resumed".to_string(),
        WorkflowEvent::WorkflowFailed { .. } => "event.workflow.failed".to_string(),
        WorkflowEvent::WorkflowCompleted { .. } => "event.workflow.completed".to_string(),
    }
}

fn context_bridge_event_to_subject(event: &ContextBridgeEvent) -> String {
    match event {
        ContextBridgeEvent::BridgeCreated { .. } => "event.context_bridge.created".to_string(),
        ContextBridgeEvent::TranslationRuleAdded { .. } => "event.context_bridge.rule_added".to_string(),
        ContextBridgeEvent::TranslationRuleRemoved { .. } => "event.context_bridge.rule_removed".to_string(),
        ContextBridgeEvent::ConceptTranslated { .. } => "event.context_bridge.concept_translated".to_string(),
        ContextBridgeEvent::BridgeDeleted { .. } => "event.context_bridge.deleted".to_string(),
        ContextBridgeEvent::MappingTypeUpdated { .. } => "event.context_bridge.mapping_updated".to_string(),
        ContextBridgeEvent::TranslationFailed { .. } => "event.context_bridge.translation_failed".to_string(),
    }
}

fn metric_context_event_to_subject(event: &MetricContextEvent) -> String {
    match event {
        MetricContextEvent::MetricContextCreated { .. } => "event.metric_context.created".to_string(),
        MetricContextEvent::DistanceSet { .. } => "event.metric_context.distance_set".to_string(),
        MetricContextEvent::ShortestPathCalculated { .. } => "event.metric_context.path_calculated".to_string(),
        MetricContextEvent::NearestNeighborsFound { .. } => "event.metric_context.neighbors_found".to_string(),
        MetricContextEvent::ConceptsClustered { .. } => "event.metric_context.concepts_clustered".to_string(),
        MetricContextEvent::ConceptsWithinRadiusFound { .. } => "event.metric_context.radius_search".to_string(),
        MetricContextEvent::MetricPropertiesUpdated { .. } => "event.metric_context.properties_updated".to_string(),
    }
}

fn rule_context_event_to_subject(event: &RuleContextEvent) -> String {
    match event {
        RuleContextEvent::RuleContextCreated { .. } => "event.rule_context.created".to_string(),
        RuleContextEvent::RuleAdded { .. } => "event.rule_context.rule_added".to_string(),
        RuleContextEvent::RuleRemoved { .. } => "event.rule_context.rule_removed".to_string(),
        RuleContextEvent::RuleEnabledChanged { .. } => "event.rule_context.rule_enabled_changed".to_string(),
        RuleContextEvent::RulesEvaluated { .. } => "event.rule_context.rules_evaluated".to_string(),
        RuleContextEvent::ComplianceChecked { .. } => "event.rule_context.compliance_checked".to_string(),
        RuleContextEvent::FactsInferred { .. } => "event.rule_context.facts_inferred".to_string(),
        RuleContextEvent::ImpactAnalyzed { .. } => "event.rule_context.impact_analyzed".to_string(),
        RuleContextEvent::RulePriorityUpdated { .. } => "event.rule_context.priority_updated".to_string(),
        RuleContextEvent::FactAdded { .. } => "event.rule_context.fact_added".to_string(),
        RuleContextEvent::FactRemoved { .. } => "event.rule_context.fact_removed".to_string(),
        RuleContextEvent::RuleActionsExecuted { .. } => "event.rule_context.actions_executed".to_string(),
        RuleContextEvent::RulesValidated { .. } => "event.rule_context.rules_validated".to_string(),
        RuleContextEvent::RulesExported { .. } => "event.rule_context.rules_exported".to_string(),
        RuleContextEvent::RulesImported { .. } => "event.rule_context.rules_imported".to_string(),
        RuleContextEvent::RuleViolated { .. } => "event.rule_context.rule_violated".to_string(),
        RuleContextEvent::RuleExecutionFailed { .. } => "event.rule_context.execution_failed".to_string(),
        RuleContextEvent::CircularDependencyDetected { .. } => "event.rule_context.circular_dependency".to_string(),
    }
}

/// Maps domain events to NATS subjects
pub fn event_to_subject(event: &DomainEvent) -> String {
    match event {
        DomainEvent::Graph(e) => graph_event_to_subject(e),
        DomainEvent::Node(e) => node_event_to_subject(e),
        DomainEvent::Edge(e) => edge_event_to_subject(e),
        DomainEvent::Subgraph(e) => subgraph_event_to_subject(e),
        DomainEvent::Workflow(e) => workflow_event_to_subject(e),
        DomainEvent::ContextBridge(e) => context_bridge_event_to_subject(e),
        DomainEvent::MetricContext(e) => metric_context_event_to_subject(e),
        DomainEvent::RuleContext(e) => rule_context_event_to_subject(e),
    }
}
