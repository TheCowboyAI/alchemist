use bevy::prelude::*;
use super::events::{DomainEvent, EventPayload, Cid};
use super::store::EventStore;
use crate::contexts::graph_management::domain::{
    Node as GraphNode, Edge, NodeIdentity, EdgeIdentity, NodeContent,
    EdgeRelationship, GraphIdentity, SpatialPosition, NodeBundle, EdgeBundle
};
use uuid::Uuid;
use std::collections::HashMap;
use serde_json::Value as JsonValue;

/// Replays events from the Merkle DAG to reconstruct state
pub struct EventReplayer;

/// Reconstructed state from event replay
#[derive(Debug, Clone)]
pub struct ReplayedState {
    pub nodes: HashMap<NodeIdentity, GraphNode>,
    pub edges: HashMap<EdgeIdentity, Edge>,
}

impl EventReplayer {
    /// Replay all events for a specific aggregate (graph)
    pub fn replay_graph(
        event_store: &EventStore,
        graph_id: Uuid,
        commands: &mut Commands,
    ) -> Result<(), String> {
        let events = event_store.get_events_for_aggregate(graph_id)?;

        for event in events {
            Self::apply_event(&event, event_store, commands)?;
        }

        Ok(())
    }

    /// Replay events from a specific point in the DAG
    pub fn replay_from_cid(
        event_store: &EventStore,
        start_cid: &Cid,
        max_depth: usize,
        commands: &mut Commands,
    ) -> Result<(), String> {
        let events = event_store.traverse_from(start_cid, max_depth)?;

        // Sort by sequence to ensure proper ordering
        let mut sorted_events = events;
        sorted_events.sort_by_key(|e| e.sequence);

        for event in sorted_events {
            Self::apply_event(&event, event_store, commands)?;
        }

        Ok(())
    }

    /// Apply a single event to reconstruct state
    fn apply_event(
        event: &DomainEvent,
        event_store: &EventStore,
        commands: &mut Commands,
    ) -> Result<(), String> {
        // Get the payload from the object store
        let payload = event_store.get_event_payload(event)?
            .ok_or_else(|| format!("Payload not found for event {}", event.id))?;

        match event.event_type.as_str() {
            "GraphCreated" => Self::apply_graph_created(event, &payload, commands),
            "NodeAdded" => Self::apply_node_added(event, &payload, commands),
            "EdgeConnected" => Self::apply_edge_connected(event, &payload, commands),
            "NodeRemoved" => Self::apply_node_removed(event, &payload, commands),
            "NodeMoved" => Self::apply_node_moved(event, &payload, commands),
            _ => {
                warn!("Unknown event type: {}", event.event_type);
                Ok(())
            }
        }
    }

    fn apply_graph_created(
        event: &DomainEvent,
        _payload: &EventPayload,
        commands: &mut Commands,
    ) -> Result<(), String> {
        // Create a marker entity for the graph
        commands.spawn((
            GraphIdentity(event.aggregate_id),
            Name::new(format!("Graph_{}", event.aggregate_id)),
        ));
        Ok(())
    }

    fn apply_node_added(
        event: &DomainEvent,
        payload: &EventPayload,
        commands: &mut Commands,
    ) -> Result<(), String> {
        let data = &payload.data;

        let node_id = data["node_id"]
            .as_str()
            .and_then(|s| s.parse::<Uuid>().ok())
            .ok_or("Invalid node_id")?;

        let label = data["content"]["label"]
            .as_str()
            .unwrap_or("Unnamed")
            .to_string();

        let category = data["content"]["category"]
            .as_str()
            .unwrap_or("default")
            .to_string();

        let x = data["position"]["x"].as_f64().unwrap_or(0.0) as f32;
        let y = data["position"]["y"].as_f64().unwrap_or(0.0) as f32;
        let z = data["position"]["z"].as_f64().unwrap_or(0.0) as f32;

        let properties = data["content"]["properties"]
            .as_object()
            .map(|map| {
                map.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<HashMap<String, serde_json::Value>>()
            })
            .unwrap_or_default();

        let node_identity = NodeIdentity(node_id);
        let position = SpatialPosition::at_3d(x, y, z);
        let content = NodeContent {
            label,
            category,
            properties,
        };

        let node = GraphNode {
            identity: node_identity,
            graph: GraphIdentity(event.aggregate_id),
            content: content.clone(),
            position,
        };

        commands.spawn(NodeBundle {
            node,
            identity: node_identity,
            position,
            content,
            transform: Transform::from_translation(Vec3::new(x, y, z)),
            global_transform: GlobalTransform::default(),
        });

        Ok(())
    }

    fn apply_edge_connected(
        event: &DomainEvent,
        payload: &EventPayload,
        commands: &mut Commands,
    ) -> Result<(), String> {
        let data = &payload.data;

        let edge_id = data["edge_id"]
            .as_str()
            .and_then(|s| s.parse::<Uuid>().ok())
            .ok_or("Invalid edge_id")?;

        let source_id = data["source"]
            .as_str()
            .and_then(|s| s.parse::<Uuid>().ok())
            .ok_or("Invalid source")?;

        let target_id = data["target"]
            .as_str()
            .and_then(|s| s.parse::<Uuid>().ok())
            .ok_or("Invalid target")?;

        let edge_type = data["edge_type"]
            .as_str()
            .unwrap_or("default")
            .to_string();

        let properties = data["properties"]
            .as_object()
            .map(|map| {
                map.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<HashMap<String, serde_json::Value>>()
            })
            .unwrap_or_default();

        let edge_identity = EdgeIdentity(edge_id);
        let relationship = EdgeRelationship {
            source: NodeIdentity(source_id),
            target: NodeIdentity(target_id),
            category: edge_type,
            strength: 1.0, // Default strength
            properties,
        };

        let edge = Edge {
            identity: edge_identity,
            graph: GraphIdentity(event.aggregate_id),
            relationship: relationship.clone(),
        };

        commands.spawn(EdgeBundle {
            edge,
            identity: edge_identity,
            relationship,
        });

        Ok(())
    }

    fn apply_node_removed(
        _event: &DomainEvent,
        payload: &EventPayload,
        _commands: &mut Commands,
    ) -> Result<(), String> {
        let data = &payload.data;

        let node_id = data["node_id"]
            .as_str()
            .and_then(|s| s.parse::<Uuid>().ok())
            .ok_or("Invalid node_id")?;

        // In a real implementation, we'd need to query for the entity
        // with this NodeIdentity and despawn it
        // For now, we'll just log it
        info!("Node removed: {}", node_id);

        // This would require entity tracking or a query system
        // commands.entity(entity).despawn_recursive();

        Ok(())
    }

    fn apply_node_moved(
        _event: &DomainEvent,
        payload: &EventPayload,
        _commands: &mut Commands,
    ) -> Result<(), String> {
        let data = &payload.data;

        let node_id = data["node_id"]
            .as_str()
            .and_then(|s| s.parse::<Uuid>().ok())
            .ok_or("Invalid node_id")?;

        let to_x = data["to_position"]["x"].as_f64().unwrap_or(0.0) as f32;
        let to_y = data["to_position"]["y"].as_f64().unwrap_or(0.0) as f32;
        let to_z = data["to_position"]["z"].as_f64().unwrap_or(0.0) as f32;

        // In a real implementation, we'd need to query for the entity
        // with this NodeIdentity and update its Transform
        info!("Node {} moved to ({}, {}, {})", node_id, to_x, to_y, to_z);

        // This would require entity tracking or a query system
        // if let Ok((entity, mut transform)) = query.get_single_mut() {
        //     transform.translation = Vec3::new(to_x, to_y, to_z);
        // }

        Ok(())
    }
}

/// System to handle replay requests
#[derive(Event)]
pub struct ReplayGraphRequest {
    pub graph_id: Uuid,
}

#[derive(Event)]
pub struct ReplayFromCidRequest {
    pub start_cid: Cid,
    pub max_depth: usize,
}

pub fn handle_replay_requests(
    event_store: Res<EventStore>,
    mut commands: Commands,
    mut graph_requests: EventReader<ReplayGraphRequest>,
    mut cid_requests: EventReader<ReplayFromCidRequest>,
) {
    for request in graph_requests.read() {
        if let Err(e) = EventReplayer::replay_graph(&*event_store, request.graph_id, &mut commands) {
            error!("Failed to replay graph {}: {}", request.graph_id, e);
        }
    }

    for request in cid_requests.read() {
        if let Err(e) = EventReplayer::replay_from_cid(
            &*event_store,
            &request.start_cid,
            request.max_depth,
            &mut commands
        ) {
            error!("Failed to replay from CID {:?}: {}", request.start_cid, e);
        }
    }
}
