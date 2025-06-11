use crate::domain::value_objects::{GraphId, NodeId, Position3D};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeEvent {
    NodeAdded {
        graph_id: GraphId,
        node_id: NodeId,
        metadata: HashMap<String, serde_json::Value>,
        position: Position3D,
    },
    NodeRemoved {
        graph_id: GraphId,
        node_id: NodeId,
    },
    NodeUpdated {
        graph_id: GraphId,
        node_id: NodeId,
        new_position: Option<Position3D>,
        new_content: Option<serde_json::Value>,
    },
    NodeMoved {
        graph_id: GraphId,
        node_id: NodeId,
        old_position: Position3D,
        new_position: Position3D,
    },
    NodeContentChanged {
        graph_id: GraphId,
        node_id: NodeId,
        old_content: serde_json::Value,
        new_content: serde_json::Value,
    },
}
