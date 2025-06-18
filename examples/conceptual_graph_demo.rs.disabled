//! Demo of the Conceptual Graph Composition System
//!
//! This example demonstrates how to create and compose concept graphs
//! using Applied Category Theory principles.

use ia::domain::conceptual_graph::{
    CategoryType, CompositionBuilder, ConceptEdge, ConceptGraph, ConceptId, ConceptNode,
    ConceptRelationship, ConceptType, ConceptualPoint, DimensionType, EnrichmentType,
    GraphComposer, NodeId, ProductType, QualityDimension,
};
use std::collections::HashMap;

fn main() {
    println!("=== Conceptual Graph Composition Demo ===\n");

    // 1. Create a simple concept graph for "User"
    let user_graph = create_user_concept();
    println!(
        "Created User concept with {} nodes",
        user_graph.node_count()
    );

    // 2. Create a concept graph for "Email"
    let email_graph = create_email_concept();
    println!(
        "Created Email concept with {} nodes",
        email_graph.node_count()
    );

    // 3. Compose them together
    let composed = compose_concepts(user_graph, email_graph);
    println!(
        "\nComposed graph has {} nodes and {} edges",
        composed.node_count(),
        composed.edge_count()
    );

    // 4. Demonstrate quality dimensions and conceptual space
    demonstrate_conceptual_space();

    // 5. Show category theory structures
    demonstrate_categories();
}

fn create_user_concept() -> ConceptGraph {
    let mut graph = ConceptGraph::new("User")
        .with_category(CategoryType::Database)
        .with_dimension(QualityDimension::new(
            "authority",
            DimensionType::Continuous,
            0.0..1.0,
        ))
        .with_dimension(QualityDimension::new(
            "activity",
            DimensionType::Continuous,
            0.0..100.0,
        ));

    // Add nodes
    let user_node = ConceptNode::Atom {
        id: NodeId::new(),
        concept_type: ConceptType::Entity,
        properties: {
            let mut props = HashMap::new();
            props.insert("has_identity".to_string(), serde_json::json!(true));
            props
        },
    };

    let name_node = ConceptNode::Atom {
        id: NodeId::new(),
        concept_type: ConceptType::ValueObject,
        properties: {
            let mut props = HashMap::new();
            props.insert("type".to_string(), serde_json::json!("string"));
            props.insert("required".to_string(), serde_json::json!(true));
            props
        },
    };

    let user_idx = graph.add_node(user_node);
    let name_idx = graph.add_node(name_node);

    // Add edge
    let edge = ConceptEdge::new(ConceptRelationship::PartOf);
    graph.add_edge(name_idx, user_idx, edge);

    graph
}

fn create_email_concept() -> ConceptGraph {
    let mut graph = ConceptGraph::new("Email")
        .with_category(CategoryType::Simple)
        .with_dimension(QualityDimension::new(
            "validity",
            DimensionType::Binary,
            0.0..1.0,
        ));

    // Add email node
    let email_node = ConceptNode::Atom {
        id: NodeId::new(),
        concept_type: ConceptType::ValueObject,
        properties: {
            let mut props = HashMap::new();
            props.insert("format".to_string(), serde_json::json!("email"));
            props.insert("pattern".to_string(), serde_json::json!("^[^@]+@[^@]+$"));
            props
        },
    };

    graph.add_node(email_node);
    graph
}

fn compose_concepts(user: ConceptGraph, email: ConceptGraph) -> ConceptGraph {
    // Use the composition builder
    let result = CompositionBuilder::new()
        .with_base(user)
        .embed(email)
        .build()
        .expect("Composition failed");

    println!("\nComposition successful!");
    println!("Result category: {:?}", result.category);
    println!("Quality dimensions: {}", result.quality_dimensions.len());

    result
}

fn demonstrate_conceptual_space() {
    println!("\n=== Conceptual Space Demo ===");

    let dimensions = vec![
        QualityDimension::new("complexity", DimensionType::Continuous, 0.0..10.0),
        QualityDimension::new("performance", DimensionType::Continuous, 0.0..100.0),
        QualityDimension::new("security", DimensionType::Ordinal, 0.0..5.0),
    ];

    // Create points in conceptual space
    let simple_fast = ConceptualPoint::new(vec![2.0, 90.0, 3.0]);
    let complex_secure = ConceptualPoint::new(vec![8.0, 60.0, 5.0]);

    // Calculate distance
    let distance = simple_fast.distance_to(&complex_secure, &dimensions);
    println!("Distance between concepts: {:.2}", distance);

    // Show dimension properties
    for dim in &dimensions {
        println!(
            "Dimension '{}': {:?}, metric: {:?}",
            dim.name, dim.dimension_type, dim.metric
        );
    }
}

fn demonstrate_categories() {
    println!("\n=== Category Theory Structures ===");

    // Show different category types
    let categories = vec![
        CategoryType::Order,
        CategoryType::Database,
        CategoryType::Monoidal,
        CategoryType::Topos,
        CategoryType::Enriched {
            enrichment: EnrichmentType::Metric,
        },
    ];

    for cat in categories {
        println!("\n{:?}:", cat);
        println!("  Description: {}", cat.description());
        println!(
            "  Supports parallel composition: {}",
            cat.supports_parallel_composition()
        );
        println!("  Has logic: {}", cat.has_logic());
        println!("  Has ordering: {}", cat.has_ordering());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concept_creation() {
        let graph = create_user_concept();
        assert_eq!(graph.name, "User");
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_composition() {
        let user = create_user_concept();
        let email = create_email_concept();

        let composed = compose_concepts(user, email);
        assert!(composed.node_count() > 2);
    }
}
