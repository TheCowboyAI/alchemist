//! Tests for ConceptGraph
use cim_conceptgraph::{ConceptGraph, ConceptId, ConceptualPoint, SemanticRelationship};

#[test]
fn test_concept_graph_creation() {
    let graph = ConceptGraph::new("Test Graph");

    assert_eq!(graph.nodes().len(), 0);
    assert_eq!(graph.edges().len(), 0);
}

#[test]
fn test_add_concepts_and_relationships() {
    let mut graph = ConceptGraph::new("Test Graph");

    // Create concepts
    let concept1 = ConceptId::new();
    let position1 = ConceptualPoint {
        coordinates: vec![0.0, 0.0, 0.0],
    };
    let node1 = graph.add_concept(
        concept1,
        position1,
        "Concept 1".to_string(),
    );

    let concept2 = ConceptId::new();
    let position2 = ConceptualPoint {
        coordinates: vec![1.0, 1.0, 1.0],
    };
    let node2 = graph.add_concept(
        concept2,
        position2,
        "Concept 2".to_string(),
    );

    // Add relationship
    let edge_id = graph.connect_concepts(
        node1,
        node2,
        SemanticRelationship::Similarity,
        0.8,
    ).unwrap();

    // Verify
    assert_eq!(graph.nodes().len(), 2);
    assert_eq!(graph.edges().len(), 1);

    let edges = graph.edges();
    assert_eq!(edges[0].0, edge_id); // Verify the edge ID matches
    assert_eq!(edges[0].1.relationship_type, SemanticRelationship::Similarity);
    assert_eq!(edges[0].1.strength, 0.8);
}
