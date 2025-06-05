//! Edge-related commands

use crate::domain::value_objects::{EdgeId, EdgeRelationship, GraphId, NodeId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeCommand {
    ConnectEdge {
        graph_id: GraphId,
        edge_id: EdgeId,
        source: NodeId,
        target: NodeId,
        relationship: EdgeRelationship,
    },
    DisconnectEdge {
        graph_id: GraphId,
        edge_id: EdgeId,
    },
    UpdateEdge {
        graph_id: GraphId,
        edge_id: EdgeId,
        relationship: EdgeRelationship,
    },
    SelectEdge {
        graph_id: GraphId,
        edge_id: EdgeId,
    },
    DeselectEdge {
        graph_id: GraphId,
        edge_id: EdgeId,
    },
}

impl EdgeCommand {
    pub fn command_type(&self) -> &'static str {
        match self {
            EdgeCommand::ConnectEdge { .. } => "ConnectEdge",
            EdgeCommand::DisconnectEdge { .. } => "DisconnectEdge",
            EdgeCommand::UpdateEdge { .. } => "UpdateEdge",
            EdgeCommand::SelectEdge { .. } => "SelectEdge",
            EdgeCommand::DeselectEdge { .. } => "DeselectEdge",
        }
    }

    pub fn graph_id(&self) -> GraphId {
        match self {
            EdgeCommand::ConnectEdge { graph_id, .. }
            | EdgeCommand::DisconnectEdge { graph_id, .. }
            | EdgeCommand::UpdateEdge { graph_id, .. }
            | EdgeCommand::SelectEdge { graph_id, .. }
            | EdgeCommand::DeselectEdge { graph_id, .. } => *graph_id,
        }
    }
}
