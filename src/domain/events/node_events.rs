//! Node-related domain events

use serde::{Deserialize, Serialize};
use crate::domain::value_objects::{GraphId, NodeId, NodeContent, Position3D};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeEvent {
    NodeAdded {
        graph_id: GraphId,
        node_id: NodeId,
        content: NodeContent,
        position: Position3D,
    },
    NodeRemoved {
        graph_id: GraphId,
        node_id: NodeId,
    },
    NodeUpdated {
        graph_id: GraphId,
        node_id: NodeId,
        old_content: NodeContent,
        new_content: NodeContent,
    },
    NodeMoved {
        graph_id: GraphId,
        node_id: NodeId,
        old_position: Position3D,
        new_position: Position3D,
    },
    NodeSelected {
        graph_id: GraphId,
        node_id: NodeId,
    },
    NodeDeselected {
        graph_id: GraphId,
        node_id: NodeId,
    },
}

impl NodeEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            NodeEvent::NodeAdded { .. } => "NodeAdded",
            NodeEvent::NodeRemoved { .. } => "NodeRemoved",
            NodeEvent::NodeUpdated { .. } => "NodeUpdated",
            NodeEvent::NodeMoved { .. } => "NodeMoved",
            NodeEvent::NodeSelected { .. } => "NodeSelected",
            NodeEvent::NodeDeselected { .. } => "NodeDeselected",
        }
    }

    pub fn graph_id(&self) -> GraphId {
        match self {
            NodeEvent::NodeAdded { graph_id, .. } |
            NodeEvent::NodeRemoved { graph_id, .. } |
            NodeEvent::NodeUpdated { graph_id, .. } |
            NodeEvent::NodeMoved { graph_id, .. } |
            NodeEvent::NodeSelected { graph_id, .. } |
            NodeEvent::NodeDeselected { graph_id, .. } => *graph_id,
        }
    }

    pub fn node_id(&self) -> NodeId {
        match self {
            NodeEvent::NodeAdded { node_id, .. } |
            NodeEvent::NodeRemoved { node_id, .. } |
            NodeEvent::NodeUpdated { node_id, .. } |
            NodeEvent::NodeMoved { node_id, .. } |
            NodeEvent::NodeSelected { node_id, .. } |
            NodeEvent::NodeDeselected { node_id, .. } => *node_id,
        }
    }
}
