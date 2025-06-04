//! Edge Commands

use serde::{Deserialize, Serialize};
use crate::domain::value_objects::{GraphId, EdgeId, NodeId, EdgeRelationship};

/// Commands for Edge operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeCommand {
    /// Create a new edge
    CreateEdge {
        graph_id: GraphId,
        edge_id: EdgeId,
        source_id: NodeId,
        target_id: NodeId,
        relationship: EdgeRelationship,
    },
    /// Update edge relationship
    UpdateEdgeRelationship {
        graph_id: GraphId,
        edge_id: EdgeId,
        relationship: EdgeRelationship,
    },
    /// Delete an edge
    DeleteEdge {
        graph_id: GraphId,
        edge_id: EdgeId,
    },
}
