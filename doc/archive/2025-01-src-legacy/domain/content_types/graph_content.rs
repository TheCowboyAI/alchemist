//! Graph content type for CIM-IPLD

use crate::domain::value_objects::{GraphId, GraphMetadata};
use cim_ipld::{ContentType, TypedContent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Content representing a complete graph structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphContent {
    /// Unique identifier for the graph
    pub id: GraphId,
    /// Graph metadata
    pub metadata: GraphMetadata,
    /// Node IDs in the graph
    pub nodes: Vec<String>,
    /// Edge definitions (source -> targets)
    pub edges: HashMap<String, Vec<String>>,
    /// Optional conceptual space coordinates for the graph
    pub conceptual_position: Option<ConceptualPosition>,
}

/// Position in conceptual space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptualPosition {
    /// Coordinates in n-dimensional conceptual space
    pub coordinates: Vec<f64>,
    /// Dimension labels
    pub dimensions: Vec<String>,
}

impl TypedContent for GraphContent {
    const CODEC: u64 = 0x300100;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x300100);
}

impl GraphContent {
    /// Create a new graph content
    pub fn new(id: GraphId, metadata: GraphMetadata) -> Self {
        Self {
            id,
            metadata,
            nodes: Vec::new(),
            edges: HashMap::new(),
            conceptual_position: None,
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node_id: String) {
        if !self.nodes.contains(&node_id) {
            self.nodes.push(node_id);
        }
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, source: String, target: String) {
        self.edges.entry(source).or_default().push(target);
    }

    /// Set the conceptual position
    pub fn set_conceptual_position(&mut self, coordinates: Vec<f64>, dimensions: Vec<String>) {
        self.conceptual_position = Some(ConceptualPosition {
            coordinates,
            dimensions,
        });
    }

    /// Get the number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges
    pub fn edge_count(&self) -> usize {
        self.edges.values().map(|targets| targets.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_graph_content_creation() {
        let id = GraphId::from(Uuid::new_v4());
        let metadata = GraphMetadata::new("Test Graph".to_string());
        let graph = GraphContent::new(id, metadata);

        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
        assert!(graph.conceptual_position.is_none());
    }

    #[test]
    fn test_graph_content_cid() {
        let id = GraphId::from(Uuid::new_v4());
        let metadata = GraphMetadata::new("Test Graph".to_string());
        let graph = GraphContent::new(id, metadata);

        let cid = graph.calculate_cid().unwrap();
        assert!(!cid.to_string().is_empty());
    }

    #[test]
    fn test_graph_content_serialization() {
        let id = GraphId::from(Uuid::new_v4());
        let metadata = GraphMetadata::new("Test Graph".to_string());
        let mut graph = GraphContent::new(id, metadata);

        graph.add_node("node1".to_string());
        graph.add_node("node2".to_string());
        graph.add_edge("node1".to_string(), "node2".to_string());

        let bytes = graph.to_bytes().unwrap();
        let deserialized = GraphContent::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.node_count(), 2);
        assert_eq!(deserialized.edge_count(), 1);
    }
}
