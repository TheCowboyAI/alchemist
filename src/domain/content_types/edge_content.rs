//! Edge content type for CIM-IPLD

use crate::domain::value_objects::{EdgeId, NodeId};
use cim_ipld::{ContentType, TypedContent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Content representing a graph edge for IPLD storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeIPLDContent {
    /// Unique identifier for the edge
    pub id: EdgeId,
    /// Source node ID
    pub source: NodeId,
    /// Target node ID
    pub target: NodeId,
    /// Edge label
    pub label: String,
    /// Edge type
    pub edge_type: EdgeIPLDType,
    /// Weight or strength of the connection
    pub weight: f64,
    /// Custom properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Types of edges in the graph for IPLD
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EdgeIPLDType {
    /// Standard directed edge
    Directed,
    /// Bidirectional edge
    Bidirectional,
    /// Semantic similarity edge
    Similarity,
    /// Dependency edge
    Dependency,
    /// Workflow sequence edge
    Sequence,
    /// Conditional edge
    Conditional,
    /// Custom edge type
    Custom(String),
}

impl TypedContent for EdgeIPLDContent {
    const CODEC: u64 = 0x300102;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x300102);
}

impl EdgeIPLDContent {
    /// Create a new edge content
    pub fn new(
        id: EdgeId,
        source: NodeId,
        target: NodeId,
        label: String,
    ) -> Self {
        Self {
            id,
            source,
            target,
            label,
            edge_type: EdgeIPLDType::Directed,
            weight: 1.0,
            properties: HashMap::new(),
        }
    }

    /// Set the edge type
    pub fn with_type(mut self, edge_type: EdgeIPLDType) -> Self {
        self.edge_type = edge_type;
        self
    }

    /// Set the edge weight
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    /// Add a custom property
    pub fn add_property(&mut self, key: String, value: serde_json::Value) {
        self.properties.insert(key, value);
    }

    /// Get a property value
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }

    /// Check if this is a bidirectional edge
    pub fn is_bidirectional(&self) -> bool {
        matches!(self.edge_type, EdgeIPLDType::Bidirectional)
    }

    /// Check if this is a workflow edge
    pub fn is_workflow(&self) -> bool {
        matches!(self.edge_type, EdgeIPLDType::Sequence | EdgeIPLDType::Conditional)
    }

    /// Get the reverse edge (for bidirectional edges)
    pub fn reverse(&self) -> Option<Self> {
        if self.is_bidirectional() {
            let mut reversed = self.clone();
            reversed.source = self.target;
            reversed.target = self.source;
            Some(reversed)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_edge_content_creation() {
        let id = EdgeId(Uuid::new_v4());
        let source = NodeId(Uuid::new_v4());
        let target = NodeId(Uuid::new_v4());
        let label = "Test Edge".to_string();
        let edge = EdgeIPLDContent::new(id, source, target, label.clone());

        assert_eq!(edge.label, label);
        assert_eq!(edge.edge_type, EdgeIPLDType::Directed);
        assert_eq!(edge.weight, 1.0);
        assert!(edge.properties.is_empty());
    }

    #[test]
    fn test_edge_content_with_type() {
        let id = EdgeId(Uuid::new_v4());
        let source = NodeId(Uuid::new_v4());
        let target = NodeId(Uuid::new_v4());
        let label = "Similarity Edge".to_string();
        let edge = EdgeIPLDContent::new(id, source, target, label)
            .with_type(EdgeIPLDType::Similarity)
            .with_weight(0.85);

        assert_eq!(edge.edge_type, EdgeIPLDType::Similarity);
        assert_eq!(edge.weight, 0.85);
    }

    #[test]
    fn test_edge_content_bidirectional() {
        let id = EdgeId(Uuid::new_v4());
        let source = NodeId(Uuid::new_v4());
        let target = NodeId(Uuid::new_v4());
        let label = "Bidirectional Edge".to_string();
        let edge = EdgeIPLDContent::new(id, source, target, label)
            .with_type(EdgeIPLDType::Bidirectional);

        assert!(edge.is_bidirectional());

        let reversed = edge.reverse().unwrap();
        assert_eq!(reversed.source, edge.target);
        assert_eq!(reversed.target, edge.source);
    }

    #[test]
    fn test_edge_content_cid() {
        let id = EdgeId(Uuid::new_v4());
        let source = NodeId(Uuid::new_v4());
        let target = NodeId(Uuid::new_v4());
        let label = "Test Edge".to_string();
        let edge = EdgeIPLDContent::new(id, source, target, label);

        let cid = edge.calculate_cid().unwrap();
        assert!(!cid.to_string().is_empty());
    }
}
