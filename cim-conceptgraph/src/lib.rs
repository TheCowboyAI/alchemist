//! ConceptGraph - Composition of ContextGraph with Conceptual Spaces
//!
//! This module provides a semantic graph that combines:
//! - ContextGraph (from cim-contextgraph) for graph structure and operations
//! - Conceptual positioning and semantic relationships
//!
//! A ConceptGraph represents domain concepts as nodes positioned in conceptual space,
//! with relationships that have both graph-theoretic and semantic properties.

use cim_contextgraph::{
    ContextGraph, NodeId, EdgeId,
    GraphError,
};
// TODO: Use these imports once cim-domain-conceptualspaces types are compatible
// use cim_domain_conceptualspaces::{
//     ConceptId, ConceptualPoint, ConvexRegion,
// };
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Temporary types until cim-domain-conceptualspaces types are compatible

/// Temporary ConceptId
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConceptId(Uuid);

impl ConceptId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Temporary ConceptualPoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptualPoint {
    pub coordinates: Vec<f32>,
}

/// Temporary ConvexRegion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvexRegion {
    pub center: ConceptualPoint,
    pub radius: f32,
}

impl ConvexRegion {
    pub fn contains(&self, point: &ConceptualPoint) -> bool {
        let distance = self.center.euclidean_distance(point);
        distance <= self.radius
    }
}

/// Node type for ConceptGraph - stores concept information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptNode {
    /// Unique identifier for this concept
    pub concept_id: ConceptId,
    /// Position in conceptual space
    pub position: ConceptualPoint,
    /// Human-readable label
    pub label: String,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Edge type for ConceptGraph - represents semantic relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptEdge {
    /// Type of semantic relationship
    pub relationship_type: SemanticRelationship,
    /// Strength of the relationship (0.0 to 1.0)
    pub strength: f32,
    /// Additional properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Types of semantic relationships between concepts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SemanticRelationship {
    /// Similarity based on conceptual distance
    Similarity,
    /// Hierarchical relationship (is-a)
    Hierarchy,
    /// Part-whole relationship (part-of)
    Meronymy,
    /// Causal relationship
    Causality,
    /// Custom relationship type
    Custom(String),
}

impl std::fmt::Display for SemanticRelationship {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Similarity => write!(f, "similarity"),
            Self::Hierarchy => write!(f, "hierarchy"),
            Self::Meronymy => write!(f, "meronymy"),
            Self::Causality => write!(f, "causality"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Error types for ConceptGraph operations
#[derive(Debug, thiserror::Error)]
pub enum ConceptGraphError {
    #[error("Graph error: {0}")]
    GraphError(#[from] GraphError),

    #[error("Concept not found: {0}")]
    ConceptNotFound(NodeId),

    #[error("Invalid semantic relationship")]
    InvalidRelationship,

    #[error("Region not found: {0}")]
    RegionNotFound(String),
}

/// Result type for ConceptGraph operations
pub type Result<T> = std::result::Result<T, ConceptGraphError>;

/// Extension trait for ConceptualPoint to add euclidean_distance method
pub trait ConceptualPointExt {
    fn euclidean_distance(&self, other: &ConceptualPoint) -> f32;
}

impl ConceptualPointExt for ConceptualPoint {
    fn euclidean_distance(&self, other: &ConceptualPoint) -> f32 {
        self.coordinates.iter()
            .zip(&other.coordinates)
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

/// A graph that combines structural and conceptual representations
pub struct ConceptGraph {
    /// The underlying context graph
    graph: ContextGraph<ConceptNode, ConceptEdge>,
    /// Named regions in conceptual space
    regions: HashMap<String, ConvexRegion>,
    /// Concept ID to Node ID mapping
    concept_map: HashMap<ConceptId, NodeId>,
}

impl ConceptGraph {
    /// Create a new ConceptGraph
    pub fn new(name: &str) -> Self {
        Self {
            graph: ContextGraph::new(name),
            regions: HashMap::new(),
            concept_map: HashMap::new(),
        }
    }

    /// Add a concept to the graph
    pub fn add_concept(
        &mut self,
        concept_id: ConceptId,
        position: ConceptualPoint,
        label: String,
    ) -> NodeId {
        let node = ConceptNode {
            concept_id,
            position,
            label,
            metadata: HashMap::new(),
        };

        let _node_id = self.graph.add_node(node);
        self.concept_map.insert(concept_id, _node_id);
        _node_id
    }

    /// Connect two concepts with a semantic relationship
    pub fn connect_concepts(
        &mut self,
        source: NodeId,
        target: NodeId,
        relationship: SemanticRelationship,
        strength: f32,
    ) -> Result<EdgeId> {
        let edge = ConceptEdge {
            relationship_type: relationship,
            strength: strength.clamp(0.0, 1.0),
            properties: HashMap::new(),
        };

        self.graph.add_edge(source, target, edge)
            .map_err(ConceptGraphError::from)
    }

    /// Add a named region to the conceptual space
    pub fn add_region(&mut self, name: String, region: ConvexRegion) {
        self.regions.insert(name, region);
    }

    /// Find concepts within a region
    pub fn concepts_in_region(&self, region_name: &str) -> Result<Vec<NodeId>> {
        let region = self.regions.get(region_name)
            .ok_or_else(|| ConceptGraphError::RegionNotFound(region_name.to_string()))?;

        let mut nodes_in_region = Vec::new();
        for (node_id, node_entry) in self.graph.get_all_nodes() {
            if region.contains(&node_entry.value.position) {
                nodes_in_region.push(node_id);
            }
        }

        Ok(nodes_in_region)
    }

    /// Find semantically similar concepts
    pub fn find_similar_concepts(
        &self,
        node_id: NodeId,
        threshold: f32,
    ) -> Result<Vec<(NodeId, f32)>> {
        let node = self.graph.get_node(node_id)
            .ok_or_else(|| ConceptGraphError::ConceptNotFound(node_id))?;
        let position = &node.value.position;

        let mut similar_nodes = Vec::new();
        for (other_node_id, other_node_entry) in self.graph.get_all_nodes() {
            if other_node_id != node_id {
                let distance = position.euclidean_distance(&other_node_entry.value.position);
                if distance <= threshold {
                    similar_nodes.push((other_node_id, distance));
                }
            }
        }

        similar_nodes.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        Ok(similar_nodes)
    }

    /// Get a concept by its ConceptId
    pub fn get_concept_by_id(&self, concept_id: ConceptId) -> Option<&ConceptNode> {
        self.concept_map.get(&concept_id)
            .and_then(|node_id| self.graph.get_node(*node_id))
            .map(|entry| &entry.value)
    }

    /// Get all nodes
    pub fn nodes(&self) -> Vec<(NodeId, ConceptNode)> {
        let mut result = Vec::new();
        for (node_id, node_entry) in self.graph.get_all_nodes() {
            result.push((node_id, node_entry.value.clone()));
        }
        result
    }

    /// Get all edges
    pub fn edges(&self) -> Vec<(EdgeId, ConceptEdge)> {
        let mut result = Vec::new();
        for (edge_id, edge_entry) in self.graph.get_all_edges() {
            result.push((edge_id, edge_entry.value.clone()));
        }
        result
    }

    /// Get a node by its NodeId
    pub fn get_node(&self, node_id: NodeId) -> Option<&ConceptNode> {
        self.graph.get_node(node_id).map(|entry| &entry.value)
    }
    
    /// Get an edge by its EdgeId
    pub fn get_edge(&self, edge_id: EdgeId) -> Option<(&ConceptEdge, NodeId, NodeId)> {
        self.graph.get_edge(edge_id).map(|entry| (&entry.value, entry.source, entry.target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concept_graph_creation() {
        let graph = ConceptGraph::new("Test Graph");
        assert_eq!(graph.nodes().len(), 0);
        assert_eq!(graph.edges().len(), 0);
    }

    #[test]
    fn test_add_concept() {
        let mut graph = ConceptGraph::new("Test Graph");

        let concept_id = ConceptId::new();
        let position = ConceptualPoint {
            coordinates: vec![1.0, 2.0, 3.0],
        };

        let node_id = graph.add_concept(
            concept_id,
            position.clone(),
            "Test Concept".to_string(),
        );

        assert_eq!(graph.nodes().len(), 1);

        // Verify the node was added with the correct ID
        let nodes = graph.nodes();
        assert_eq!(nodes[0].0, node_id);
        
        // Verify we can retrieve the node by its node_id
        let node = graph.get_node(node_id).unwrap();
        assert_eq!(node.concept_id, concept_id);

        let concept = graph.get_concept_by_id(concept_id).unwrap();
        assert_eq!(concept.label, "Test Concept");
        assert_eq!(concept.position.coordinates, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_connect_concepts() {
        let mut graph = ConceptGraph::new("Test Graph");

        let concept1 = ConceptId::new();
        let concept2 = ConceptId::new();

        let node1 = graph.add_concept(
            concept1,
            ConceptualPoint { coordinates: vec![0.0, 0.0] },
            "Concept 1".to_string(),
        );

        let node2 = graph.add_concept(
            concept2,
            ConceptualPoint { coordinates: vec![1.0, 1.0] },
            "Concept 2".to_string(),
        );

        let edge_id = graph.connect_concepts(
            node1,
            node2,
            SemanticRelationship::Similarity,
            0.8,
        ).unwrap();

        assert_eq!(graph.edges().len(), 1);

        // Verify the edge was created with the correct ID
        let edges = graph.edges();
        assert_eq!(edges[0].0, edge_id);
        
        // Verify we can retrieve the edge by its edge_id
        let (edge, source, target) = graph.get_edge(edge_id).unwrap();
        assert_eq!(edge.relationship_type, SemanticRelationship::Similarity);
        assert_eq!(edge.strength, 0.8);
        
        // Verify the edge connects the correct nodes
        assert_eq!(source, node1);
        assert_eq!(target, node2);
    }

    #[test]
    fn test_find_similar_concepts() {
        let mut graph = ConceptGraph::new("Test Graph");

        // Add three concepts in a line
        let c1 = graph.add_concept(
            ConceptId::new(),
            ConceptualPoint { coordinates: vec![0.0, 0.0] },
            "C1".to_string(),
        );

        let c2 = graph.add_concept(
            ConceptId::new(),
            ConceptualPoint { coordinates: vec![1.0, 0.0] },
            "C2".to_string(),
        );

        let c3 = graph.add_concept(
            ConceptId::new(),
            ConceptualPoint { coordinates: vec![3.0, 0.0] },
            "C3".to_string(),
        );

        // Find concepts similar to c1 within distance 2.0
        let similar = graph.find_similar_concepts(c1, 2.0).unwrap();

        assert_eq!(similar.len(), 1);
        assert_eq!(similar[0].0, c2);
        assert_eq!(similar[0].1, 1.0);
        
        // Find concepts similar to c2 within distance 2.5
        let similar_to_c2 = graph.find_similar_concepts(c2, 2.5).unwrap();
        assert_eq!(similar_to_c2.len(), 2);
        
        // c1 should be closer than c3
        assert_eq!(similar_to_c2[0].0, c1);
        assert_eq!(similar_to_c2[0].1, 1.0);
        assert_eq!(similar_to_c2[1].0, c3);
        assert_eq!(similar_to_c2[1].1, 2.0);
        
        // Verify c3 is too far from c1
        let similar_to_c1_large = graph.find_similar_concepts(c1, 5.0).unwrap();
        assert_eq!(similar_to_c1_large.len(), 2);
        assert!(similar_to_c1_large.iter().any(|(node, _)| *node == c3));
    }

    #[test]
    fn test_regions() {
        let mut graph = ConceptGraph::new("Test Graph");

        // Create a region
        let region = ConvexRegion {
            center: ConceptualPoint { coordinates: vec![0.0, 0.0] },
            radius: 2.0,
        };

        graph.add_region("TestRegion".to_string(), region);

        // Add concepts inside and outside the region
        let inside = graph.add_concept(
            ConceptId::new(),
            ConceptualPoint { coordinates: vec![1.0, 1.0] },
            "Inside".to_string(),
        );

        let outside = graph.add_concept(
            ConceptId::new(),
            ConceptualPoint { coordinates: vec![5.0, 5.0] },
            "Outside".to_string(),
        );

        let concepts_in_region = graph.concepts_in_region("TestRegion").unwrap();

        assert_eq!(concepts_in_region.len(), 1);
        assert_eq!(concepts_in_region[0], inside);
        
        // Verify the outside node is not in the region
        assert!(!concepts_in_region.contains(&outside));
        
        // Create a larger region that contains both nodes
        let large_region = ConvexRegion {
            center: ConceptualPoint { coordinates: vec![0.0, 0.0] },
            radius: 10.0,
        };
        graph.add_region("LargeRegion".to_string(), large_region);
        
        let concepts_in_large_region = graph.concepts_in_region("LargeRegion").unwrap();
        assert_eq!(concepts_in_large_region.len(), 2);
        assert!(concepts_in_large_region.contains(&inside));
        assert!(concepts_in_large_region.contains(&outside));
    }
}
