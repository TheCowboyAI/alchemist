//! Edge value objects
//!
//! Edges are value objects within the graph context.
//! They represent relationships between nodes.

use crate::shared::types::{EdgeId, NodeId};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Edge data within a graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub edge_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Edge data for creation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeData {
    pub source: NodeId,
    pub target: NodeId,
    pub edge_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl EdgeData {
    pub fn new(source: NodeId, target: NodeId, edge_type: String) -> Self {
        Self {
            source,
            target,
            edge_type,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}
