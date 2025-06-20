//! Node Commands

use crate::domain::value_objects::{GraphId, NodeContent, NodeId, Position3D};
use serde::{Deserialize, Serialize};

/// Commands for Node operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeCommand {
    /// Create a new node
    CreateNode {
        graph_id: GraphId,
        node_id: NodeId,
        position: Position3D,
        content: NodeContent,
    },
    /// Update node position
    UpdateNodePosition {
        graph_id: GraphId,
        node_id: NodeId,
        position: Position3D,
    },
    /// Update node content
    UpdateNodeContent {
        graph_id: GraphId,
        node_id: NodeId,
        content: NodeContent,
    },
    /// Delete a node
    DeleteNode { graph_id: GraphId, node_id: NodeId },
}
