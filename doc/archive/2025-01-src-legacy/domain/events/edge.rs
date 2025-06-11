use crate::domain::value_objects::{GraphId, NodeId, EdgeId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EdgeEvent {
    EdgeConnected {
        graph_id: GraphId,
        edge_id: EdgeId,
        source: NodeId,
        target: NodeId,
        relationship: String,
    },
    EdgeRemoved {
        graph_id: GraphId,
        edge_id: EdgeId,
    },
    EdgeUpdated {
        graph_id: GraphId,
        edge_id: EdgeId,
        new_properties: HashMap<String, serde_json::Value>,
    },
    EdgeReversed {
        graph_id: GraphId,
        edge_id: EdgeId,
        old_source: NodeId,
        old_target: NodeId,
        new_source: NodeId,
        new_target: NodeId,
    },
}
