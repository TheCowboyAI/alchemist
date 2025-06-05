//! Node content type for CIM-IPLD

use crate::domain::value_objects::{NodeId, Position3D};
use cim_ipld::{ContentType, TypedContent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Content representing a graph node for IPLD storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIPLDContent {
    /// Unique identifier for the node
    pub id: NodeId,
    /// Node label
    pub label: String,
    /// Visual position in 3D space
    pub position: Position3D,
    /// Optional conceptual space coordinates
    pub conceptual_coordinates: Option<Vec<f64>>,
    /// Custom properties
    pub properties: HashMap<String, serde_json::Value>,
    /// Node type/category
    pub node_type: NodeIPLDType,
}

/// Types of nodes in the graph for IPLD
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeIPLDType {
    /// Standard graph node
    Standard,
    /// Concept node in conceptual space
    Concept,
    /// Workflow step node
    WorkflowStep,
    /// Decision point node
    Decision,
    /// Integration node
    Integration,
    /// Custom node type
    Custom(String),
}

impl TypedContent for NodeIPLDContent {
    const CODEC: u64 = 0x300101;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x300101);
}

impl NodeIPLDContent {
    /// Create a new node content
    pub fn new(id: NodeId, label: String, position: Position3D) -> Self {
        Self {
            id,
            label,
            position,
            conceptual_coordinates: None,
            properties: HashMap::new(),
            node_type: NodeIPLDType::Standard,
        }
    }

    /// Set the node type
    pub fn with_type(mut self, node_type: NodeIPLDType) -> Self {
        self.node_type = node_type;
        self
    }

    /// Set conceptual coordinates
    pub fn with_conceptual_coordinates(mut self, coordinates: Vec<f64>) -> Self {
        self.conceptual_coordinates = Some(coordinates);
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

    /// Check if this is a concept node
    pub fn is_concept(&self) -> bool {
        matches!(self.node_type, NodeIPLDType::Concept)
    }

    /// Check if this is a workflow node
    pub fn is_workflow(&self) -> bool {
        matches!(self.node_type, NodeIPLDType::WorkflowStep | NodeIPLDType::Decision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_node_content_creation() {
        let id = NodeId(Uuid::new_v4());
        let label = "Test Node".to_string();
        let position = Position3D::new(1.0, 2.0, 3.0);
        let node = NodeIPLDContent::new(id, label.clone(), position);

        assert_eq!(node.label, label);
        assert_eq!(node.node_type, NodeIPLDType::Standard);
        assert!(node.conceptual_coordinates.is_none());
        assert!(node.properties.is_empty());
    }

    #[test]
    fn test_node_content_with_type() {
        let id = NodeId(Uuid::new_v4());
        let label = "Concept Node".to_string();
        let position = Position3D::new(0.0, 0.0, 0.0);
        let node = NodeIPLDContent::new(id, label, position)
            .with_type(NodeIPLDType::Concept)
            .with_conceptual_coordinates(vec![0.5, 0.7, 0.3]);

        assert!(node.is_concept());
        assert!(node.conceptual_coordinates.is_some());
    }

    #[test]
    fn test_node_content_properties() {
        let id = NodeId(Uuid::new_v4());
        let label = "Test Node".to_string();
        let position = Position3D::new(0.0, 0.0, 0.0);
        let mut node = NodeIPLDContent::new(id, label, position);

        node.add_property("color".to_string(), serde_json::json!("#FF0000"));
        node.add_property("weight".to_string(), serde_json::json!(1.5));

        assert_eq!(
            node.get_property("color"),
            Some(&serde_json::json!("#FF0000"))
        );
        assert_eq!(node.get_property("weight"), Some(&serde_json::json!(1.5)));
    }

    #[test]
    fn test_node_content_cid() {
        let id = NodeId(Uuid::new_v4());
        let label = "Test Node".to_string();
        let position = Position3D::new(0.0, 0.0, 0.0);
        let node = NodeIPLDContent::new(id, label, position);

        let cid = node.calculate_cid().unwrap();
        assert!(!cid.to_string().is_empty());
    }
}
