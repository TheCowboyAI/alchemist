//! Node-related domain events

use crate::domain::value_objects::{GraphId, NodeId, Position3D};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeEvent {
    NodeAdded {
        graph_id: GraphId,
        node_id: NodeId,
        position: Position3D,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    },
    NodeRemoved {
        graph_id: GraphId,
        node_id: NodeId,
    },
    // NOTE: To change a node's position (value object), you must:
    // 1. NodeRemoved (remove the node at old position)
    // 2. NodeAdded (add the node at new position)
    // This follows DDD principles where value objects are immutable
    NodeMetadataUpdated {
        graph_id: GraphId,
        node_id: NodeId,
        key: String,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
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
            NodeEvent::NodeMetadataUpdated { .. } => "NodeMetadataUpdated",
            NodeEvent::NodeSelected { .. } => "NodeSelected",
            NodeEvent::NodeDeselected { .. } => "NodeDeselected",
        }
    }

    pub fn graph_id(&self) -> GraphId {
        match self {
            NodeEvent::NodeAdded { graph_id, .. }
            | NodeEvent::NodeRemoved { graph_id, .. }
            | NodeEvent::NodeMetadataUpdated { graph_id, .. }
            | NodeEvent::NodeSelected { graph_id, .. }
            | NodeEvent::NodeDeselected { graph_id, .. } => *graph_id,
        }
    }

    pub fn node_id(&self) -> NodeId {
        match self {
            NodeEvent::NodeAdded { node_id, .. }
            | NodeEvent::NodeRemoved { node_id, .. }
            | NodeEvent::NodeMetadataUpdated { node_id, .. }
            | NodeEvent::NodeSelected { node_id, .. }
            | NodeEvent::NodeDeselected { node_id, .. } => *node_id,
        }
    }
}
