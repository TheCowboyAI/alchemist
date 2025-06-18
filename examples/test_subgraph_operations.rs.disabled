//! Test program to verify subgraph operations functionality

use chrono::Utc;
use ia::domain::{
    commands::{AnalysisDepth, MergeSubgraphsBuilder, SubgraphOperationCommand},
    events::SubgraphOperationEvent,
    services::{SubgraphAnalyzer, SubgraphLayoutCalculator},
    value_objects::{
        BorderStyle, ClusteringAlgorithm, CollapseStrategy, Color, EdgeId, FillPattern, GraphId,
        IconType, LayoutDirection, LayoutStrategy, MergeStrategy, NodeId, OptimizationType,
        Position3D, SplitCriteria, SubgraphAnalysis, SubgraphId, SubgraphMetadata,
        SubgraphMetadataBuilder, SubgraphState, SubgraphStatistics, SubgraphStyle, SubgraphType,
        SuggestedOperation,
    },
};
use petgraph::graph::Graph;
use std::collections::{HashMap, HashSet};

fn main() {
    println!("Testing Subgraph Operations...\n");

    // Test 1: Value Objects
    test_value_objects();

    // Test 2: Events
    test_events();

    // Test 3: Commands
    test_commands();

    // Test 4: Domain Services
    test_domain_services();

    println!("\nAll tests completed successfully!");
}

fn test_value_objects() {
    println!("=== Testing Value Objects ===");

    // Test SubgraphState
    let expanded = SubgraphState::Expanded;
    let collapsed = SubgraphState::Collapsed {
        collapsed_position: Position3D::new(10.0, 20.0, 0.0),
        original_layout: Box::new(LayoutStrategy::ForceDirected {
            iterations: 100,
            spring_strength: 0.1,
            repulsion_strength: 0.2,
        }),
    };

    println!("✓ SubgraphState created: {:?}", collapsed);

    // Test SubgraphMetadata
    let metadata = SubgraphMetadataBuilder::new("Test Subgraph")
        .with_description("A test subgraph for verification")
        .with_icon(IconType::Module)
        .with_tag("test")
        .with_tag("example")
        .build();

    println!(
        "✓ SubgraphMetadata created: {} with {} tags",
        metadata.name,
        metadata.tags.len()
    );

    // Test SubgraphStyle
    let style = SubgraphStyle {
        border_color: Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        fill_color: Color {
            r: 0.8,
            g: 0.8,
            b: 0.8,
            a: 0.5,
        },
        border_style: BorderStyle::Dashed,
        border_width: 2.0,
        fill_pattern: FillPattern::Gradient,
        label_style: Default::default(),
    };

    println!("✓ SubgraphStyle created with dashed border");

    // Test SubgraphStatistics
    let stats = SubgraphStatistics {
        node_count: 10,
        edge_count: 15,
        internal_edges: 12,
        external_edges: 3,
        depth: 2,
        density: 0.27,
        clustering_coefficient: 0.6,
        average_degree: 3.0,
    };

    println!(
        "✓ SubgraphStatistics: {} nodes, {} edges, density: {:.2}",
        stats.node_count, stats.edge_count, stats.density
    );
}

fn test_events() {
    println!("\n=== Testing Events ===");

    let graph_id = GraphId::new();
    let subgraph_id = SubgraphId::new();

    // Test SubgraphCollapsed event
    let collapse_event = SubgraphOperationEvent::SubgraphCollapsed {
        graph_id,
        subgraph_id,
        collapsed_at: Position3D::new(50.0, 50.0, 0.0),
        contained_nodes: vec![NodeId::new(), NodeId::new(), NodeId::new()],
        collapse_strategy: CollapseStrategy::Centroid,
        timestamp: Utc::now(),
    };

    println!("✓ SubgraphCollapsed event created with {} nodes", 3);

    // Test SubgraphAnalyzed event
    let analysis = SubgraphAnalysis {
        statistics: SubgraphStatistics {
            node_count: 5,
            edge_count: 4,
            internal_edges: 3,
            external_edges: 1,
            depth: 2,
            density: 0.4,
            clustering_coefficient: 0.6,
            average_degree: 1.6,
        },
        cohesion_score: 0.8,
        coupling_score: 0.2,
        complexity_score: 0.5,
        suggested_operations: vec![SuggestedOperation::Optimize {
            reason: "High complexity".to_string(),
            optimization_type: OptimizationType::SimplifyStructure,
            confidence: 0.7,
        }],
    };

    let analyze_event = SubgraphOperationEvent::SubgraphAnalyzed {
        graph_id,
        subgraph_id,
        analysis,
        timestamp: Utc::now(),
    };

    println!("✓ SubgraphAnalyzed event created with cohesion: 0.8");

    // Test event helper methods
    assert_eq!(collapse_event.graph_id(), graph_id);
    assert_eq!(collapse_event.primary_subgraph_id(), Some(subgraph_id));
    println!("✓ Event helper methods work correctly");
}

fn test_commands() {
    println!("\n=== Testing Commands ===");

    let graph_id = GraphId::new();
    let subgraph_id = SubgraphId::new();

    // Test CollapseSubgraph command
    let collapse_cmd = SubgraphOperationCommand::CollapseSubgraph {
        graph_id,
        subgraph_id,
        strategy: CollapseStrategy::WeightedCenter,
    };

    println!("✓ CollapseSubgraph command created with weighted center strategy");

    // Test MergeSubgraphs command using builder
    let merge_cmd = MergeSubgraphsBuilder::new(graph_id)
        .add_source(SubgraphId::new())
        .add_source(SubgraphId::new())
        .with_strategy(MergeStrategy::Union)
        .with_new_name("Merged Subgraph")
        .build();

    match merge_cmd {
        SubgraphOperationCommand::MergeSubgraphs {
            source_subgraphs, ..
        } => {
            println!(
                "✓ MergeSubgraphs command created with {} sources",
                source_subgraphs.len()
            );
        }
        _ => panic!("Wrong command type"),
    }

    // Test AnalyzeSubgraph command
    let analyze_cmd = SubgraphOperationCommand::AnalyzeSubgraph {
        graph_id,
        subgraph_id,
    };

    println!("✓ AnalyzeSubgraph command created");

    // Test command helper methods
    assert_eq!(collapse_cmd.command_type(), "CollapseSubgraph");
    assert_eq!(collapse_cmd.graph_id(), graph_id);
    println!("✓ Command helper methods work correctly");
}

fn test_domain_services() {
    println!("\n=== Testing Domain Services ===");

    // Create a test graph
    let mut graph = Graph::new();
    let n1 = graph.add_node(NodeId::new());
    let n2 = graph.add_node(NodeId::new());
    let n3 = graph.add_node(NodeId::new());
    let n4 = graph.add_node(NodeId::new());

    graph.add_edge(n1, n2, EdgeId::new());
    graph.add_edge(n2, n3, EdgeId::new());
    graph.add_edge(n3, n1, EdgeId::new());
    graph.add_edge(n3, n4, EdgeId::new());

    let mut subgraph_nodes = HashSet::new();
    subgraph_nodes.insert(*graph.node_weight(n1).unwrap());
    subgraph_nodes.insert(*graph.node_weight(n2).unwrap());
    subgraph_nodes.insert(*graph.node_weight(n3).unwrap());

    // Test SubgraphAnalyzer
    let analyzer = SubgraphAnalyzer::new();
    let analysis = analyzer.analyze_subgraph(&graph, &subgraph_nodes);

    println!("✓ SubgraphAnalyzer results:");
    println!("  - Cohesion: {:.2}", analysis.cohesion_score);
    println!("  - Coupling: {:.2}", analysis.coupling_score);
    println!("  - Complexity: {:.2}", analysis.complexity_score);
    println!("  - Suggestions: {}", analysis.suggested_operations.len());

    // Test SubgraphLayoutCalculator
    let layout_calc = SubgraphLayoutCalculator::new();

    let mut node_positions_map = HashMap::new();
    node_positions_map.insert(
        *graph.node_weight(n1).unwrap(),
        Position3D::new(0.0, 0.0, 0.0),
    );
    node_positions_map.insert(
        *graph.node_weight(n2).unwrap(),
        Position3D::new(10.0, 0.0, 0.0),
    );
    node_positions_map.insert(
        *graph.node_weight(n3).unwrap(),
        Position3D::new(5.0, 8.66, 0.0),
    );

    let collapsed_pos = layout_calc.calculate_collapsed_position(
        &node_positions_map,
        &CollapseStrategy::Centroid,
        None,
    );

    println!(
        "✓ SubgraphLayoutCalculator collapse position: ({:.2}, {:.2}, {:.2})",
        collapsed_pos.x, collapsed_pos.y, collapsed_pos.z
    );

    let layout_strategy = LayoutStrategy::Circular {
        radius: 20.0,
        start_angle: 0.0,
    };
    let expanded_layout = layout_calc.calculate_expansion_layout(
        &subgraph_nodes.iter().cloned().collect::<Vec<_>>(),
        collapsed_pos,
        &layout_strategy,
        None,
        None,
    );

    println!(
        "✓ SubgraphLayoutCalculator expanded {} nodes in circular layout",
        expanded_layout.len()
    );
}
