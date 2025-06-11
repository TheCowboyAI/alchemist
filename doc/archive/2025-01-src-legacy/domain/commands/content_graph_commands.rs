use crate::domain::{
    aggregates::content_graph::NodeContent,
    value_objects::{EdgeId, GraphId, NodeId, Position3D, RelatedBy},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Commands for ContentGraph operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentGraphCommand {
    /// Create a new content graph
    CreateGraph {
        graph_id: GraphId,
    },

    /// Add content to the graph
    AddContent {
        node_id: NodeId,
        content: NodeContent,
        position: Position3D,
        metadata: HashMap<String, serde_json::Value>,
    },

    /// Remove content from the graph
    RemoveContent {
        node_id: NodeId,
    },

    /// Establish a relationship between content nodes
    EstablishRelationship {
        edge_id: EdgeId,
        source: NodeId,
        target: NodeId,
        relationship: RelatedBy,
    },

    /// Remove a relationship
    RemoveRelationship {
        edge_id: EdgeId,
    },

    /// Discover relationships based on similarity
    DiscoverRelationships {
        threshold: f64,
    },

    /// Update semantic clusters
    UpdateSemanticClusters,

    /// Calculate business metrics
    CalculateMetrics,

    /// Define a named view (selection of nodes)
    DefineView {
        name: String,
        description: String,
        node_ids: Vec<NodeId>,
    },

    /// Update a view
    UpdateView {
        name: String,
        node_ids: Vec<NodeId>,
    },

    /// Remove a view
    RemoveView {
        name: String,
    },
}
