//! Graph Summary Projection
//!
//! Maintains summary statistics and metadata for all graphs in the system.
//! This projection is optimized for quick lookups of graph information.

use crate::domain::events::{DomainEvent, GraphEvent};
use crate::domain::value_objects::{GraphId, GraphMetadata};
use async_trait::async_trait;
use bevy::prelude::*;
use std::collections::HashMap;
use std::time::{SystemTime, Instant};

/// Summary information about a graph
#[derive(Debug, Clone)]
pub struct GraphSummary {
    pub id: GraphId,
    pub metadata: GraphMetadata,
    pub node_count: usize,
    pub edge_count: usize,
    pub created_at: SystemTime,
    pub last_modified: SystemTime,
    pub last_event_sequence: u64,
}

/// Projection that maintains graph summaries
#[derive(Resource)]
pub struct GraphSummaryProjection {
    summaries: HashMap<GraphId, GraphSummary>,
    last_checkpoint: Option<u64>,
    last_update: Instant,
}

impl Default for GraphSummaryProjection {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur in the projection
#[derive(Debug, thiserror::Error)]
pub enum ProjectionError {
    #[error("Graph not found: {0}")]
    GraphNotFound(GraphId),

    #[error("Invalid event sequence: expected {expected}, got {actual}")]
    InvalidSequence { expected: u64, actual: u64 },

    #[error("Checkpoint save failed: {0}")]
    CheckpointFailed(String),
}

#[async_trait]
impl Projection for GraphSummaryProjection {
    type Event = DomainEvent;
    type Error = ProjectionError;

        async fn handle_event(&mut self, event: Self::Event, sequence: u64) -> Result<(), Self::Error> {
        use crate::domain::events::{NodeEvent, EdgeEvent};
        let timestamp = SystemTime::now(); // Events don't carry timestamps in current structure

        match event {
            DomainEvent::Graph(GraphEvent::GraphCreated { id, metadata }) => {
                let summary = GraphSummary {
                    id,
                    metadata,
                    node_count: 0,
                    edge_count: 0,
                    created_at: timestamp,
                    last_modified: timestamp,
                    last_event_sequence: sequence,
                };
                self.summaries.insert(id, summary);
            }

            DomainEvent::Graph(GraphEvent::GraphDeleted { id }) => {
                self.summaries.remove(&id);
            }

            DomainEvent::Graph(GraphEvent::GraphRenamed { id, new_name, .. }) => {
                if let Some(summary) = self.summaries.get_mut(&id) {
                    summary.metadata.name = new_name;
                    summary.last_modified = timestamp;
                    summary.last_event_sequence = sequence;
                }
            }

            DomainEvent::Graph(GraphEvent::GraphTagged { id, tag }) => {
                if let Some(summary) = self.summaries.get_mut(&id) {
                    if !summary.metadata.tags.contains(&tag) {
                        summary.metadata.tags.push(tag);
                    }
                    summary.last_modified = timestamp;
                    summary.last_event_sequence = sequence;
                }
            }

            DomainEvent::Graph(GraphEvent::GraphUntagged { id, tag }) => {
                if let Some(summary) = self.summaries.get_mut(&id) {
                    summary.metadata.tags.retain(|t| t != &tag);
                    summary.last_modified = timestamp;
                    summary.last_event_sequence = sequence;
                }
            }

            DomainEvent::Node(NodeEvent::NodeAdded { graph_id, .. }) => {
                if let Some(summary) = self.summaries.get_mut(&graph_id) {
                    summary.node_count += 1;
                    summary.last_modified = timestamp;
                    summary.last_event_sequence = sequence;
                }
            }

            DomainEvent::Node(NodeEvent::NodeRemoved { graph_id, .. }) => {
                if let Some(summary) = self.summaries.get_mut(&graph_id) {
                    summary.node_count = summary.node_count.saturating_sub(1);
                    summary.last_modified = timestamp;
                    summary.last_event_sequence = sequence;
                }
            }

            DomainEvent::Edge(EdgeEvent::EdgeAdded { graph_id, .. }) |
            DomainEvent::Edge(EdgeEvent::EdgeConnected { graph_id, .. }) => {
                if let Some(summary) = self.summaries.get_mut(&graph_id) {
                    summary.edge_count += 1;
                    summary.last_modified = timestamp;
                    summary.last_event_sequence = sequence;
                }
            }

            DomainEvent::Edge(EdgeEvent::EdgeRemoved { graph_id, .. }) |
            DomainEvent::Edge(EdgeEvent::EdgeDisconnected { graph_id, .. }) => {
                if let Some(summary) = self.summaries.get_mut(&graph_id) {
                    summary.edge_count = summary.edge_count.saturating_sub(1);
                    summary.last_modified = timestamp;
                    summary.last_event_sequence = sequence;
                }
            }

            _ => {
                // Other events don't affect graph summaries
            }
        }

        self.last_checkpoint = Some(sequence);
        self.last_update = Instant::now();
        Ok(())
    }

    async fn get_checkpoint(&self) -> Option<u64> {
        self.last_checkpoint
    }

    async fn save_checkpoint(&mut self, sequence: u64) -> Result<(), Self::Error> {
        // In a real implementation, this would persist to storage
        self.last_checkpoint = Some(sequence);
        Ok(())
    }
}

impl GraphSummaryProjection {
    pub fn new() -> Self {
        Self {
            summaries: HashMap::new(),
            last_checkpoint: None,
            last_update: Instant::now(),
        }
    }

    /// Get a summary for a specific graph
    pub fn get_summary(&self, graph_id: &GraphId) -> Option<&GraphSummary> {
        self.summaries.get(graph_id)
    }

    /// Get all graph summaries
    pub fn get_all_summaries(&self) -> Vec<&GraphSummary> {
        self.summaries.values().collect()
    }

    /// Get summaries sorted by last modified time
    pub fn get_recent_graphs(&self, limit: usize) -> Vec<&GraphSummary> {
        let mut summaries: Vec<_> = self.summaries.values().collect();
        summaries.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        summaries.truncate(limit);
        summaries
    }

    /// Get total statistics across all graphs
    pub fn get_total_stats(&self) -> (usize, usize, usize) {
        let graph_count = self.summaries.len();
        let total_nodes: usize = self.summaries.values().map(|s| s.node_count).sum();
        let total_edges: usize = self.summaries.values().map(|s| s.edge_count).sum();
        (graph_count, total_nodes, total_edges)
    }

    /// Get time since last update
    pub fn time_since_update(&self) -> std::time::Duration {
        self.last_update.elapsed()
    }
}

/// Trait for projections in our system
#[async_trait]
pub trait Projection: Send + Sync {
    type Event;
    type Error: std::error::Error;

    /// Handle an event and update the projection state
    async fn handle_event(&mut self, event: Self::Event, sequence: u64) -> Result<(), Self::Error>;

    /// Get the last processed event sequence number
    async fn get_checkpoint(&self) -> Option<u64>;

    /// Save a checkpoint for recovery
    async fn save_checkpoint(&mut self, sequence: u64) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{NodeId, Position3D};

    #[tokio::test]
    async fn test_graph_summary_projection() {
        let mut projection = GraphSummaryProjection::new();
        let graph_id = GraphId::new();

                // Create graph
        let create_event = DomainEvent::Graph(GraphEvent::GraphCreated {
            id: graph_id,
            metadata: GraphMetadata::new("Test Graph".to_string()),
        });

        projection.handle_event(create_event, 1).await.unwrap();

        // Verify summary created
        let summary = projection.get_summary(&graph_id).unwrap();
        assert_eq!(summary.metadata.name, "Test Graph");
        assert_eq!(summary.node_count, 0);
        assert_eq!(summary.edge_count, 0);

        // Add nodes
        for i in 0..5 {
            let node_event = DomainEvent::Node(crate::domain::events::NodeEvent::NodeAdded {
                graph_id,
                node_id: NodeId::new(),
                position: Position3D::default(),
                metadata: Default::default(),
            });
            projection.handle_event(node_event, 2 + i).await.unwrap();
        }

        // Verify node count
        let summary = projection.get_summary(&graph_id).unwrap();
        assert_eq!(summary.node_count, 5);

        // Check checkpoint
        assert_eq!(projection.get_checkpoint().await, Some(6));
    }
}
