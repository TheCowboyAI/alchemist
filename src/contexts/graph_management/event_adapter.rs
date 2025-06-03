use super::events::*;
use crate::contexts::graph_management::domain::*;
use crate::contexts::event_store::{DomainEvent, events::EventPayload, EventStore, DomainEventOccurred};
use bevy::prelude::*;
use serde_json::json;
use std::time::SystemTime;

/// Adapter to convert graph management events into domain events for the Merkle DAG
pub struct GraphEventAdapter;

impl GraphEventAdapter {
    /// Convert a GraphCreated event to a domain event
    pub fn graph_created_to_domain_event(
        event: &GraphCreated,
        event_store: &EventStore,
    ) -> Result<DomainEvent, String> {
        let payload = EventPayload {
            data: json!({
                "graph_id": event.graph.0,
                "metadata": event.metadata,
                "timestamp": event.timestamp,
            }),
            created_at: SystemTime::now(),
        };

        event_store.append_with_payload(
            event.graph.0,
            "GraphCreated".to_string(),
            payload,
        )
    }

    /// Convert a NodeAdded event to a domain event
    pub fn node_added_to_domain_event(
        event: &NodeAdded,
        event_store: &EventStore,
    ) -> Result<DomainEvent, String> {
        let payload = EventPayload {
            data: json!({
                "node_id": event.node.0,
                "content": {
                    "label": event.content.label,
                    "category": event.content.category,
                    "properties": event.content.properties,
                },
                "position": {
                    "x": event.position.coordinates_3d.x,
                    "y": event.position.coordinates_3d.y,
                    "z": event.position.coordinates_3d.z,
                },
            }),
            created_at: SystemTime::now(),
        };

        event_store.append_with_payload(
            event.graph.0,
            "NodeAdded".to_string(),
            payload,
        )
    }

    /// Convert an EdgeConnected event to a domain event
    pub fn edge_connected_to_domain_event(
        event: &EdgeConnected,
        event_store: &EventStore,
    ) -> Result<DomainEvent, String> {
        let payload = EventPayload {
            data: json!({
                "edge_id": event.edge.0,
                "source": event.relationship.source.0,
                "target": event.relationship.target.0,
                "edge_type": event.relationship.category,
                "properties": event.relationship.properties,
            }),
            created_at: SystemTime::now(),
        };

        event_store.append_with_payload(
            event.graph.0,
            "EdgeConnected".to_string(),
            payload,
        )
    }

    /// Convert a NodeRemoved event to a domain event
    pub fn node_removed_to_domain_event(
        event: &NodeRemoved,
        event_store: &EventStore,
    ) -> Result<DomainEvent, String> {
        let payload = EventPayload {
            data: json!({
                "node_id": event.node.0,
            }),
            created_at: SystemTime::now(),
        };

        event_store.append_with_payload(
            event.graph.0,
            "NodeRemoved".to_string(),
            payload,
        )
    }

    /// Convert a NodeMoved event to a domain event
    pub fn node_moved_to_domain_event(
        event: &NodeMoved,
        event_store: &EventStore,
    ) -> Result<DomainEvent, String> {
        let payload = EventPayload {
            data: json!({
                "node_id": event.node.0,
                "from_position": {
                    "x": event.from_position.coordinates_3d.x,
                    "y": event.from_position.coordinates_3d.y,
                    "z": event.from_position.coordinates_3d.z,
                },
                "to_position": {
                    "x": event.to_position.coordinates_3d.x,
                    "y": event.to_position.coordinates_3d.y,
                    "z": event.to_position.coordinates_3d.z,
                },
            }),
            created_at: SystemTime::now(),
        };

        event_store.append_with_payload(
            event.graph.0,
            "NodeMoved".to_string(),
            payload,
        )
    }
}

/// System to capture graph events and store them in the Merkle DAG
pub fn capture_graph_events(
    event_store: Res<EventStore>,
    mut domain_events: EventWriter<DomainEventOccurred>,
    mut graph_created: EventReader<GraphCreated>,
    mut node_added: EventReader<NodeAdded>,
    mut edge_connected: EventReader<EdgeConnected>,
    mut node_removed: EventReader<NodeRemoved>,
    mut node_moved: EventReader<NodeMoved>,
) {
    // Process GraphCreated events
    for event in graph_created.read() {
        match GraphEventAdapter::graph_created_to_domain_event(event, &*event_store) {
            Ok(domain_event) => {
                domain_events.write(DomainEventOccurred(domain_event));
            }
            Err(e) => {
                error!("Failed to convert GraphCreated event: {}", e);
            }
        }
    }

    // Process NodeAdded events
    for event in node_added.read() {
        match GraphEventAdapter::node_added_to_domain_event(event, &*event_store) {
            Ok(domain_event) => {
                domain_events.write(DomainEventOccurred(domain_event));
            }
            Err(e) => {
                error!("Failed to convert NodeAdded event: {}", e);
            }
        }
    }

    // Process EdgeConnected events
    for event in edge_connected.read() {
        match GraphEventAdapter::edge_connected_to_domain_event(event, &*event_store) {
            Ok(domain_event) => {
                domain_events.write(DomainEventOccurred(domain_event));
            }
            Err(e) => {
                error!("Failed to convert EdgeConnected event: {}", e);
            }
        }
    }

    // Process NodeRemoved events
    for event in node_removed.read() {
        match GraphEventAdapter::node_removed_to_domain_event(event, &*event_store) {
            Ok(domain_event) => {
                domain_events.write(DomainEventOccurred(domain_event));
            }
            Err(e) => {
                error!("Failed to convert NodeRemoved event: {}", e);
            }
        }
    }

    // Process NodeMoved events
    for event in node_moved.read() {
        match GraphEventAdapter::node_moved_to_domain_event(event, &*event_store) {
            Ok(domain_event) => {
                domain_events.write(DomainEventOccurred(domain_event));
            }
            Err(e) => {
                error!("Failed to convert NodeMoved event: {}", e);
            }
        }
    }
}
