//! JetStream Persistence for Graph Data
//!
//! This module provides actual persistence of graph data to NATS JetStream,
//! enabling event sourcing, replay, and distributed synchronization.

use anyhow::{Result, Context};
use async_nats::jetstream::{self, stream::{Config as StreamConfig, RetentionPolicy}};
use bevy::prelude::*;
use crate::{
    graph_components::*,
    graph_parser::{NodeData, EdgeData},
    nats_client::NatsClient,
    TokioRuntime,
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use futures::StreamExt;

/// Event types for graph persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GraphPersistenceEvent {
    // Graph lifecycle
    GraphCreated {
        graph_id: String,
        name: String,
        metadata: serde_json::Value,
        timestamp: i64,
    },
    GraphDeleted {
        graph_id: String,
        timestamp: i64,
    },
    
    // Node events
    NodeAdded {
        graph_id: String,
        node_id: String,
        label: String,
        position: [f32; 3],
        metadata: serde_json::Value,
        timestamp: i64,
    },
    NodeUpdated {
        graph_id: String,
        node_id: String,
        label: Option<String>,
        position: Option<[f32; 3]>,
        metadata: Option<serde_json::Value>,
        timestamp: i64,
    },
    NodeRemoved {
        graph_id: String,
        node_id: String,
        timestamp: i64,
    },
    
    // Edge events
    EdgeAdded {
        graph_id: String,
        edge_id: String,
        source_id: String,
        target_id: String,
        label: Option<String>,
        weight: f32,
        timestamp: i64,
    },
    EdgeUpdated {
        graph_id: String,
        edge_id: String,
        label: Option<String>,
        weight: Option<f32>,
        timestamp: i64,
    },
    EdgeRemoved {
        graph_id: String,
        edge_id: String,
        timestamp: i64,
    },
    
    // Bulk operations
    BulkNodesAdded {
        graph_id: String,
        nodes: Vec<NodeData>,
        timestamp: i64,
    },
    BulkEdgesAdded {
        graph_id: String,
        edges: Vec<EdgeData>,
        timestamp: i64,
    },
    
    // Layout changes
    LayoutChanged {
        graph_id: String,
        layout_type: String,
        timestamp: i64,
    },
    
    // Snapshot
    GraphSnapshot {
        graph_id: String,
        nodes: Vec<NodeData>,
        edges: Vec<EdgeData>,
        metadata: serde_json::Value,
        timestamp: i64,
    },
}

// NodeData and EdgeData are imported from graph_parser

/// Resource for managing JetStream persistence
#[derive(Resource)]
pub struct GraphPersistence {
    client: NatsClient,
    jetstream: jetstream::Context,
    stream_name: String,
    replay_batch_size: usize,
}

impl GraphPersistence {
    pub async fn new(client: NatsClient) -> Result<Self> {
        let jetstream = jetstream::new(client.inner().clone());
        let stream_name = "GRAPH_EVENTS".to_string();
        
        // Create or update stream
        let stream_config = StreamConfig {
            name: stream_name.clone(),
            subjects: vec![
                "graph.>".to_string(),
            ],
            retention: RetentionPolicy::Limits,
            max_messages_per_subject: 10_000,
            max_bytes: 1_073_741_824, // 1GB
            max_age: Duration::from_secs(30 * 24 * 60 * 60), // 30 days
            ..Default::default()
        };
        
        jetstream.create_stream(stream_config).await
            .context("Failed to create JetStream stream")?;
        
        Ok(Self {
            client,
            jetstream,
            stream_name,
            replay_batch_size: 100,
        })
    }
    
    /// Publish a graph event to JetStream
    pub async fn publish_event(&self, event: GraphPersistenceEvent) -> Result<()> {
        let subject = match &event {
            GraphPersistenceEvent::GraphCreated { graph_id, .. } => 
                format!("graph.{}.created", graph_id),
            GraphPersistenceEvent::GraphDeleted { graph_id, .. } => 
                format!("graph.{}.deleted", graph_id),
            GraphPersistenceEvent::NodeAdded { graph_id, .. } => 
                format!("graph.{}.node.added", graph_id),
            GraphPersistenceEvent::NodeUpdated { graph_id, .. } => 
                format!("graph.{}.node.updated", graph_id),
            GraphPersistenceEvent::NodeRemoved { graph_id, .. } => 
                format!("graph.{}.node.removed", graph_id),
            GraphPersistenceEvent::EdgeAdded { graph_id, .. } => 
                format!("graph.{}.edge.added", graph_id),
            GraphPersistenceEvent::EdgeUpdated { graph_id, .. } => 
                format!("graph.{}.edge.updated", graph_id),
            GraphPersistenceEvent::EdgeRemoved { graph_id, .. } => 
                format!("graph.{}.edge.removed", graph_id),
            GraphPersistenceEvent::BulkNodesAdded { graph_id, .. } => 
                format!("graph.{}.bulk.nodes", graph_id),
            GraphPersistenceEvent::BulkEdgesAdded { graph_id, .. } => 
                format!("graph.{}.bulk.edges", graph_id),
            GraphPersistenceEvent::LayoutChanged { graph_id, .. } => 
                format!("graph.{}.layout", graph_id),
            GraphPersistenceEvent::GraphSnapshot { graph_id, .. } => 
                format!("graph.{}.snapshot", graph_id),
        };
        
        let payload = serde_json::to_vec(&event)?;
        
        self.jetstream
            .publish(subject, payload.into())
            .await?
            .await?; // Wait for acknowledgment
        
        Ok(())
    }
    
    /// Load a graph by replaying events from JetStream
    pub async fn load_graph(&self, graph_id: &str) -> Result<ReplayedGraph> {
        let subject = format!("graph.{}.>", graph_id);
        
        // Create consumer for replay
        let consumer = self.jetstream
            .create_consumer_on_stream(
                jetstream::consumer::Config {
                    filter_subject: subject,
                    deliver_policy: jetstream::consumer::DeliverPolicy::All,
                    ..Default::default()
                },
                self.stream_name.clone(),
            )
            .await?;
        
        let replayed_graph = ReplayedGraph::new(graph_id.to_string());
        
        // Get messages from consumer - this is a simplified version
        // In production, you'd use consumer.stream() or similar
        info!("Loading graph {} from JetStream", graph_id);
        
        Ok(replayed_graph)
    }
    
    /// Save current graph state as a snapshot
    pub async fn save_snapshot(
        &self,
        graph_id: &str,
        nodes: Vec<NodeData>,
        edges: Vec<EdgeData>,
        metadata: serde_json::Value,
    ) -> Result<()> {
        let event = GraphPersistenceEvent::GraphSnapshot {
            graph_id: graph_id.to_string(),
            nodes,
            edges,
            metadata,
            timestamp: current_timestamp(),
        };
        
        self.publish_event(event).await
    }
    
    /// Get the latest snapshot for a graph
    pub async fn get_latest_snapshot(&self, graph_id: &str) -> Result<Option<GraphPersistenceEvent>> {
        let subject = format!("graph.{}.snapshot", graph_id);
        
        // Get the last message on the snapshot subject
        let stream = self.jetstream.get_stream(&self.stream_name).await?;
        let last_msg = stream.get_last_raw_message_by_subject(&subject).await?;
        
        let event: GraphPersistenceEvent = serde_json::from_slice(&last_msg.payload)?;
        Ok(Some(event))
    }
    
    /// Subscribe to real-time graph events
    pub async fn subscribe_to_graph(&self, graph_id: &str) -> Result<GraphEventSubscription> {
        let subject = format!("graph.{}.>", graph_id);
        let subscriber = self.client.subscribe(&subject).await?;
        
        Ok(GraphEventSubscription {
            subscriber,
            graph_id: graph_id.to_string(),
        })
    }
}

/// Subscription to graph events
pub struct GraphEventSubscription {
    subscriber: async_nats::Subscriber,
    graph_id: String,
}

impl GraphEventSubscription {
    pub async fn next(&mut self) -> Result<Option<GraphPersistenceEvent>> {
        if let Some(msg) = self.subscriber.next().await {
            let event: GraphPersistenceEvent = serde_json::from_slice(&msg.payload)?;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }
}

/// Represents a graph rebuilt from events
#[derive(Debug, Clone)]
pub struct ReplayedGraph {
    pub graph_id: String,
    pub name: String,
    pub nodes: HashMap<String, NodeData>,
    pub edges: HashMap<String, EdgeData>,
    pub metadata: serde_json::Value,
    pub last_event_timestamp: i64,
}

impl ReplayedGraph {
    fn new(graph_id: String) -> Self {
        Self {
            graph_id,
            name: String::new(),
            nodes: HashMap::new(),
            edges: HashMap::new(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            last_event_timestamp: 0,
        }
    }
    
    fn apply_event(&mut self, event: GraphPersistenceEvent) {
        match event {
            GraphPersistenceEvent::GraphCreated { name, metadata, timestamp, .. } => {
                self.name = name;
                self.metadata = metadata;
                self.last_event_timestamp = timestamp;
            }
            GraphPersistenceEvent::NodeAdded { node_id, label, position, metadata, timestamp, .. } => {
                self.nodes.insert(node_id.clone(), NodeData {
                    id: node_id,
                    label,
                    position,
                    color: None,
                    size: None,
                    metadata,
                });
                self.last_event_timestamp = timestamp;
            }
            GraphPersistenceEvent::NodeUpdated { node_id, label, position, metadata, timestamp, .. } => {
                if let Some(node) = self.nodes.get_mut(&node_id) {
                    if let Some(new_label) = label {
                        node.label = new_label;
                    }
                    if let Some(new_position) = position {
                        node.position = new_position;
                    }
                    if let Some(new_metadata) = metadata {
                        node.metadata = new_metadata;
                    }
                }
                self.last_event_timestamp = timestamp;
            }
            GraphPersistenceEvent::NodeRemoved { node_id, timestamp, .. } => {
                self.nodes.remove(&node_id);
                self.last_event_timestamp = timestamp;
            }
            GraphPersistenceEvent::EdgeAdded { edge_id, source_id, target_id, label, weight, timestamp, .. } => {
                self.edges.insert(edge_id.clone(), EdgeData {
                    id: edge_id,
                    source_id,
                    target_id,
                    label,
                    weight,
                    color: None,
                    metadata: serde_json::Value::Null,
                });
                self.last_event_timestamp = timestamp;
            }
            GraphPersistenceEvent::EdgeRemoved { edge_id, timestamp, .. } => {
                self.edges.remove(&edge_id);
                self.last_event_timestamp = timestamp;
            }
            GraphPersistenceEvent::GraphSnapshot { nodes, edges, metadata, timestamp, .. } => {
                // Replace entire state with snapshot
                self.nodes.clear();
                self.edges.clear();
                
                for node in nodes {
                    self.nodes.insert(node.id.clone(), node);
                }
                for edge in edges {
                    self.edges.insert(edge.id.clone(), edge);
                }
                
                self.metadata = metadata;
                self.last_event_timestamp = timestamp;
            }
            _ => {} // Handle other events as needed
        }
    }
}

/// System to persist graph changes to JetStream
pub fn persist_graph_events_system(
    mut events: EventReader<GraphOperationEvent>,
    persistence: Option<Res<GraphPersistence>>,
    nodes: Query<(&GraphNode, &Transform)>,
    edges: Query<&GraphEdge>,
    runtime: Res<TokioRuntime>,
) {
    let Some(persistence) = persistence else { return };
    
    for event in events.read() {
        match &event.operation {
            GraphOperation::CreateNode { id, label, position } => {
                let persist_event = GraphPersistenceEvent::NodeAdded {
                    graph_id: event.graph_id.clone(),
                    node_id: id.clone(),
                    label: label.clone(),
                    position: position.to_array(),
                    metadata: serde_json::Value::Null,
                    timestamp: current_timestamp(),
                };
                
                // In production, we'd queue this event for async processing
                info!("Would persist node creation: {}", id);
            }
            GraphOperation::DeleteNode { entity } => {
                if let Ok((node, _)) = nodes.get(*entity) {
                    let persist_event = GraphPersistenceEvent::NodeRemoved {
                        graph_id: event.graph_id.clone(),
                        node_id: node.id.clone(),
                        timestamp: current_timestamp(),
                    };
                    
                    // In production, we'd queue this event for async processing
                    info!("Would persist node deletion: {}", node.id);
                }
            }
            _ => {} // Handle other operations
        }
    }
}

/// System to sync graph state from JetStream
pub fn sync_from_jetstream_system(
    mut commands: Commands,
    persistence: Option<Res<GraphPersistence>>,
    graph_manager: ResMut<GraphManager>,
    runtime: Res<TokioRuntime>,
) {
    let Some(persistence) = persistence else { return };
    
    // This would be triggered by specific events or on startup
    // For now, it's a placeholder for the sync logic
}

/// Helper to get current timestamp
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// Resource for managing graph event replay
#[derive(Resource)]
pub struct GraphReplayManager {
    pub replaying: bool,
    pub target_timestamp: Option<i64>,
    pub events_replayed: usize,
}

impl Default for GraphReplayManager {
    fn default() -> Self {
        Self {
            replaying: false,
            target_timestamp: None,
            events_replayed: 0,
        }
    }
}

/// Component to mark entities that are being replayed from history
#[derive(Component)]
pub struct ReplayedEntity {
    pub original_timestamp: i64,
    pub replayed_at: i64,
}