//! Edge-related domain events

use crate::domain::value_objects::{EdgeId, EdgeRelationship, GraphId, NodeId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    EdgeAdded {
        graph_id: GraphId,
        edge_id: EdgeId,
        source: NodeId,
        target: NodeId,
        metadata: HashMap<String, serde_json::Value>,
    },
    EdgeRemoved {
        graph_id: GraphId,
        edge_id: EdgeId,
    },
    EdgeMetadataUpdated {
        graph_id: GraphId,
        edge_id: EdgeId,
        key: String,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
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
            EdgeEvent::EdgeAdded { .. } => "EdgeAdded",
            EdgeEvent::EdgeRemoved { .. } => "EdgeRemoved",
            EdgeEvent::EdgeMetadataUpdated { .. } => "EdgeMetadataUpdated",
        }
    }

    pub fn graph_id(&self) -> GraphId {
        match self {
            EdgeEvent::EdgeConnected { graph_id, .. }
            | EdgeEvent::EdgeDisconnected { graph_id, .. }
            | EdgeEvent::EdgeUpdated { graph_id, .. }
            | EdgeEvent::EdgeSelected { graph_id, .. }
            | EdgeEvent::EdgeDeselected { graph_id, .. }
            | EdgeEvent::EdgeAdded { graph_id, .. }
            | EdgeEvent::EdgeRemoved { graph_id, .. }
            | EdgeEvent::EdgeMetadataUpdated { graph_id, .. } => *graph_id,
        }
    }

    pub fn edge_id(&self) -> EdgeId {
        match self {
            EdgeEvent::EdgeConnected { edge_id, .. }
            | EdgeEvent::EdgeDisconnected { edge_id, .. }
            | EdgeEvent::EdgeUpdated { edge_id, .. }
            | EdgeEvent::EdgeSelected { edge_id, .. }
            | EdgeEvent::EdgeDeselected { edge_id, .. }
            | EdgeEvent::EdgeAdded { edge_id, .. }
            | EdgeEvent::EdgeRemoved { edge_id, .. }
            | EdgeEvent::EdgeMetadataUpdated { edge_id, .. } => *edge_id,
        }
    }
}
