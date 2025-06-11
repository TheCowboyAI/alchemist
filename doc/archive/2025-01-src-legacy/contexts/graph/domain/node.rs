//! Node value objects
//!
//! Nodes are value objects within the graph context.
//! They have no identity outside of their containing graph.

use crate::shared::types::NodeId;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Node data within a graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub node_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Node data for creation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeData {
    pub node_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl NodeData {
    pub fn new(node_type: String) -> Self {
        Self {
            node_type,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}
