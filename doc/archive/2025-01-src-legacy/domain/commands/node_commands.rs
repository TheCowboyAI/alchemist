//! Node-related commands

use crate::domain::value_objects::{GraphId, NodeContent, NodeId, Position3D};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeCommand {
    AddNode {
        graph_id: GraphId,
        node_id: NodeId,
        content: NodeContent,
        position: Position3D,
    },
    RemoveNode {
        graph_id: GraphId,
        node_id: NodeId,
    },
    UpdateNode {
        graph_id: GraphId,
        node_id: NodeId,
        content: NodeContent,
    },
    MoveNode {
        graph_id: GraphId,
        node_id: NodeId,
        position: Position3D,
    },
    SelectNode {
        graph_id: GraphId,
        node_id: NodeId,
    },
    DeselectNode {
        graph_id: GraphId,
        node_id: NodeId,
    },
}

impl NodeCommand {
    pub fn command_type(&self) -> &'static str {
        match self {
            NodeCommand::AddNode { .. } => "AddNode",
            NodeCommand::RemoveNode { .. } => "RemoveNode",
            NodeCommand::UpdateNode { .. } => "UpdateNode",
            NodeCommand::MoveNode { .. } => "MoveNode",
            NodeCommand::SelectNode { .. } => "SelectNode",
            NodeCommand::DeselectNode { .. } => "DeselectNode",
        }
    }

    pub fn graph_id(&self) -> GraphId {
        match self {
            NodeCommand::AddNode { graph_id, .. }
            | NodeCommand::RemoveNode { graph_id, .. }
            | NodeCommand::UpdateNode { graph_id, .. }
            | NodeCommand::MoveNode { graph_id, .. }
            | NodeCommand::SelectNode { graph_id, .. }
            | NodeCommand::DeselectNode { graph_id, .. } => *graph_id,
        }
    }
}
