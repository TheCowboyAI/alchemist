//! Example of consuming subject-based events in Bevy systems

use bevy::prelude::*;
use crate::infrastructure::event_bridge::{SubjectRouter, SubjectConsumer, RoutedEvent};
use crate::domain::events::DomainEvent;
use crate::presentation::components::{GraphNode, GraphEdge, SubgraphRegion};
use tracing::info;

/// Example system that consumes graph-related events
pub fn graph_event_consumer_system(
    router: Res<SubjectRouter>,
    mut consumers: Query<&mut GraphEventConsumer>,
    mut commands: Commands,
) {
    for mut consumer in consumers.iter_mut() {
        // Poll events from all subscribed subjects
        let events = consumer.subject_consumer.poll_events();

        // Process events in order
        for routed_event in events {
            match &routed_event.event {
                DomainEvent::Node(crate::domain::events::NodeEvent::NodeAdded { graph_id, node_id, position, .. }) => {
                    info!(
                        "Received NodeAdded on subject '{}' (seq: {})",
                        routed_event.subject,
                        routed_event.global_sequence
                    );

                    // Handle the event - spawn visual representation
                    commands.spawn((
                        GraphNode {
                            node_id: *node_id,
                            graph_id: *graph_id,
                        },
                        Transform::from_translation((*position).into()),
                        // ... other components
                    ));
                }

                DomainEvent::Edge(crate::domain::events::EdgeEvent::EdgeConnected { graph_id, edge_id, source, target, .. }) => {
                    info!(
                        "Received EdgeConnected on subject '{}' (seq: {})",
                        routed_event.subject,
                        routed_event.global_sequence
                    );

                    // Handle edge creation
                    // ... implementation
                }

                _ => {
                    // Handle other event types
                }
            }

            // Update last processed sequence
            consumer.last_global_sequence = routed_event.global_sequence;
            consumer.events_processed += 1;
        }
    }
}

/// Component for entities that consume graph events
#[derive(Component)]
pub struct GraphEventConsumer {
    pub subject_consumer: SubjectConsumer,
    pub last_global_sequence: u64,
    pub events_processed: u64,
}

impl GraphEventConsumer {
    /// Create a consumer for specific graph events
    pub fn new_for_graph(router: &SubjectRouter, graph_id: &str) -> Result<Self, String> {
        let patterns = vec![
            format!("event.graph.{}.>", graph_id),  // All events for this graph
            "event.graph.*.created".to_string(),    // All graph creation events
        ];

        let subject_consumer = SubjectConsumer::new(router, patterns)?;

        Ok(Self {
            subject_consumer,
            last_global_sequence: 0,
            events_processed: 0,
        })
    }

    /// Create a consumer for all node events across graphs
    pub fn new_for_nodes(router: &SubjectRouter) -> Result<Self, String> {
        let patterns = vec![
            "event.graph.*.node.>".to_string(),  // All node events
        ];

        let subject_consumer = SubjectConsumer::new(router, patterns)?;

        Ok(Self {
            subject_consumer,
            last_global_sequence: 0,
            events_processed: 0,
        })
    }
}

/// System to create specialized consumers
pub fn setup_event_consumers(
    mut commands: Commands,
    router: Res<SubjectRouter>,
) {
    // Create a consumer for all graph events
    if let Ok(consumer) = GraphEventConsumer::new_for_nodes(&router) {
        commands.spawn((
            consumer,
            Name::new("NodeEventConsumer"),
        ));
    }

    // You can create multiple consumers with different patterns
    let patterns = vec![
        "event.graph.*.subgraph.>".to_string(),  // All subgraph events
    ];

    if let Ok(subject_consumer) = SubjectConsumer::new(&router, patterns) {
        commands.spawn((
            GraphEventConsumer {
                subject_consumer,
                last_global_sequence: 0,
                events_processed: 0,
            },
            Name::new("SubgraphEventConsumer"),
        ));
    }
}

/// System to monitor event consumer health
pub fn monitor_event_consumers(
    consumers: Query<(&Name, &GraphEventConsumer)>,
    time: Res<Time>,
) {
    static mut LAST_CHECK: Option<f32> = None;

    let current_time = time.elapsed_secs();

    unsafe {
        if let Some(last) = LAST_CHECK {
            if current_time - last < 5.0 {
                return;  // Only check every 5 seconds
            }
        }
        LAST_CHECK = Some(current_time);
    }

    for (name, consumer) in consumers.iter() {
        info!(
            "{}: Processed {} events, last sequence: {}",
            name,
            consumer.events_processed,
            consumer.last_global_sequence
        );
    }
}

/// Plugin to demonstrate subject-based event consumption
pub struct EventConsumerExamplePlugin;

impl Plugin for EventConsumerExamplePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_event_consumers)
            .add_systems(
                Update,
                (
                    graph_event_consumer_system,
                    monitor_event_consumers,
                ),
            );
    }
}
