//! Edge-related domain events

use serde::{Deserialize, Serialize};
use crate::domain::value_objects::{GraphId, EdgeId, NodeId, EdgeRelationship};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeEvent {
    EdgeConnected {
        graph_id: GraphId,
        edge_id: EdgeId,
        source: NodeId,
        target: NodeId,
        relationship: EdgeRelationship,
    },
    EdgeDisconnected {
        graph_id: GraphId,
        edge_id: EdgeId,
        source: NodeId,
        target: NodeId,
    },
    EdgeUpdated {
        graph_id: GraphId,
        edge_id: EdgeId,
        old_relationship: EdgeRelationship,
        new_relationship: EdgeRelationship,
    },
    EdgeSelected {
        graph_id: GraphId,
        edge_id: EdgeId,
    },
    EdgeDeselected {
        graph_id: GraphId,
        edge_id: EdgeId,
    },
}

impl EdgeEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            EdgeEvent::EdgeConnected { .. } => "EdgeConnected",
            EdgeEvent::EdgeDisconnected { .. } => "EdgeDisconnected",
            EdgeEvent::EdgeUpdated { .. } => "EdgeUpdated",
            EdgeEvent::EdgeSelected { .. } => "EdgeSelected",
            EdgeEvent::EdgeDeselected { .. } => "EdgeDeselected",
        }
    }

    pub fn graph_id(&self) -> GraphId {
        match self {
            EdgeEvent::EdgeConnected { graph_id, .. } |
            EdgeEvent::EdgeDisconnected { graph_id, .. } |
            EdgeEvent::EdgeUpdated { graph_id, .. } |
            EdgeEvent::EdgeSelected { graph_id, .. } |
            EdgeEvent::EdgeDeselected { graph_id, .. } => *graph_id,
        }
    }

    pub fn edge_id(&self) -> EdgeId {
        match self {
            EdgeEvent::EdgeConnected { edge_id, .. } |
            EdgeEvent::EdgeDisconnected { edge_id, .. } |
            EdgeEvent::EdgeUpdated { edge_id, .. } |
            EdgeEvent::EdgeSelected { edge_id, .. } |
            EdgeEvent::EdgeDeselected { edge_id, .. } => *edge_id,
        }
    }
}
