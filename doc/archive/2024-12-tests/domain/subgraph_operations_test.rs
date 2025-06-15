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

#[test]
fn test_subgraph_state_transitions() {
    // Test state representations
    let expanded = SubgraphState::Expanded;
    let collapsed = SubgraphState::Collapsed;

    assert_eq!(format!("{}", expanded), "Expanded");
    assert_eq!(format!("{}", collapsed), "Collapsed");

    // Test transitioning state
    let transitioning = SubgraphState::Transitioning {
        progress: 0.75,
        from: Box::new(SubgraphState::Expanded),
        to: Box::new(SubgraphState::Collapsed),
    };

    let display = format!("{}", transitioning);
    assert!(display.contains("75.0%"));
    assert!(display.contains("Expanded"));
    assert!(display.contains("Collapsed"));
}

#[test]
fn test_subgraph_metadata_builder() {
    let metadata = SubgraphMetadataBuilder::new("Test Module")
        .description("A test subgraph module")
        .tag("test")
        .tag("module")
        .icon(IconType::Module)
        .author("Test Author")
        .property("complexity", serde_json::json!(5))
        .build();

    assert_eq!(metadata.name, "Test Module");
    assert_eq!(
        metadata.description,
        Some("A test subgraph module".to_string())
    );
    assert_eq!(metadata.tags.len(), 2);
    assert!(metadata.tags.contains(&"test".to_string()));
    assert!(metadata.tags.contains(&"module".to_string()));
    assert_eq!(metadata.icon, Some(IconType::Module));
    assert_eq!(metadata.author, Some("Test Author".to_string()));
    assert_eq!(metadata.version, 1);
    assert!(metadata.validate().is_ok());
}

#[test]
fn test_subgraph_style_creation() {
    let style = SubgraphStyle {
        base_color: Color {
            r: 0.2,
            g: 0.4,
            b: 0.8,
            a: 1.0,
        },
        border_style: BorderStyle::Dashed {
            width: 2.0,
            dash_length: 5.0,
            gap_length: 3.0,
        },
        fill_pattern: FillPattern::Gradient {
            start_color: Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            end_color: Color {
                r: 0.4,
                g: 0.5,
                b: 0.6,
                a: 1.0,
            },
            angle: 45.0,
        },
        glow_intensity: 0.5,
        opacity: 0.9,
    };

    // Test that all fields are accessible
    assert_eq!(style.base_color.r, 0.2);
    assert_eq!(style.glow_intensity, 0.5);
    assert_eq!(style.opacity, 0.9);
}

#[test]
fn test_subgraph_events() {
    let graph_id = GraphId::new();
    let subgraph_id = SubgraphId::new();
    let node_ids = vec![NodeId::new(), NodeId::new(), NodeId::new()];
    let position = Position3D::new(10.0, 20.0, 0.0);

    // Test collapse event
    let collapse_event = SubgraphOperationEvent::SubgraphCollapsed {
        graph_id,
        subgraph_id,
        collapsed_at: position,
        contained_nodes: node_ids.clone(),
        collapse_strategy: CollapseStrategy::Centroid,
        timestamp: Utc::now(),
    };

    assert_eq!(collapse_event.graph_id(), graph_id);

    // Test expand event
    let expand_event = SubgraphOperationEvent::SubgraphExpanded {
        graph_id,
        subgraph_id,
        expansion_layout: LayoutStrategy::Circular {
            radius: 5.0,
            start_angle: 0.0,
        },
        node_positions: vec![(node_ids[0], Position3D::new(0.0, 0.0, 0.0))],
        timestamp: Utc::now(),
    };

    assert_eq!(expand_event.graph_id(), graph_id);

    // Test that we can get the primary subgraph ID
    if let Some(primary_id) = expand_event.primary_subgraph_id() {
        assert_eq!(primary_id, subgraph_id);
    }
}

#[test]
fn test_subgraph_commands() {
    let graph_id = GraphId::new();
    let subgraph_id = SubgraphId::new();

    // Test collapse command
    let collapse_cmd = SubgraphOperationCommand::CollapseSubgraph {
        graph_id,
        subgraph_id,
        collapse_strategy: CollapseStrategy::MostConnected,
        animate: true,
        duration_ms: Some(300),
    };

    assert!(collapse_cmd.validate().is_ok());
    assert_eq!(collapse_cmd.graph_id(), graph_id);

    // Test invalid duration
    let invalid_cmd = SubgraphOperationCommand::CollapseSubgraph {
        graph_id,
        subgraph_id,
        collapse_strategy: CollapseStrategy::Centroid,
        animate: true,
        duration_ms: Some(0),
    };

    assert!(invalid_cmd.validate().is_err());

    // Test merge command builder
    let merge_cmd = MergeSubgraphsBuilder::new(graph_id, "Merged Module")
        .add_source(SubgraphId::new())
        .add_source(SubgraphId::new())
        .strategy(MergeStrategy::OptimizeConnections)
        .preserve_metadata(false)
        .build();

    assert!(merge_cmd.is_ok());
}

#[test]
fn test_subgraph_analyzer_cohesion() {
    let analyzer = SubgraphAnalyzer::new();
    let mut graph = Graph::new();

    // Create a triangle (fully connected)
    let n1 = graph.add_node(NodeId::new());
    let n2 = graph.add_node(NodeId::new());
    let n3 = graph.add_node(NodeId::new());

    graph.add_edge(n1, n2, EdgeId::new());
    graph.add_edge(n2, n3, EdgeId::new());
    graph.add_edge(n3, n1, EdgeId::new());

    let subgraph_nodes: HashSet<NodeId> = graph.node_weights().cloned().collect();

    // Test cohesion - should be 1.0 for fully connected triangle
    let cohesion = analyzer.analyze_cohesion(&graph, &subgraph_nodes);
    assert_eq!(cohesion, 1.0);

    // Test with disconnected node
    let n4 = graph.add_node(NodeId::new());
    let mut larger_subgraph = subgraph_nodes.clone();
    larger_subgraph.insert(*graph.node_weight(n4).unwrap());

    let cohesion2 = analyzer.analyze_cohesion(&graph, &larger_subgraph);
    assert!(cohesion2 < 1.0); // Should be less than 1.0
}

#[test]
fn test_subgraph_analyzer_coupling() {
    let analyzer = SubgraphAnalyzer::new();
    let mut graph = Graph::new();

    // Create two connected components
    let n1 = graph.add_node(NodeId::new());
    let n2 = graph.add_node(NodeId::new());
    let n3 = graph.add_node(NodeId::new());
    let n4 = graph.add_node(NodeId::new());

    // Internal edges
    graph.add_edge(n1, n2, EdgeId::new());

    // External edge
    graph.add_edge(n2, n3, EdgeId::new());
    graph.add_edge(n3, n4, EdgeId::new());

    let mut subgraph_nodes = HashSet::new();
    subgraph_nodes.insert(*graph.node_weight(n1).unwrap());
    subgraph_nodes.insert(*graph.node_weight(n2).unwrap());

    let coupling = analyzer.analyze_coupling(&graph, &subgraph_nodes);

    // Should have some coupling due to external edge
    assert!(coupling > 0.0);
    assert!(coupling < 1.0);
}

#[test]
fn test_subgraph_analyzer_full_analysis() {
    let analyzer = SubgraphAnalyzer::new();
    let mut graph = Graph::new();

    // Create a more complex graph
    let nodes: Vec<_> = (0..5).map(|_| graph.add_node(NodeId::new())).collect();

    // Create some edges
    graph.add_edge(nodes[0], nodes[1], EdgeId::new());
    graph.add_edge(nodes[1], nodes[2], EdgeId::new());
    graph.add_edge(nodes[2], nodes[3], EdgeId::new());
    graph.add_edge(nodes[3], nodes[4], EdgeId::new());
    graph.add_edge(nodes[4], nodes[0], EdgeId::new()); // Make it a cycle

    let subgraph_nodes: HashSet<NodeId> = graph.node_weights().cloned().collect();

    let analysis = analyzer.analyze_subgraph(&graph, &subgraph_nodes);

    // Verify analysis results
    assert_eq!(analysis.statistics.node_count, 5);
    assert_eq!(analysis.statistics.edge_count, 5);
    assert!(analysis.cohesion_score > 0.0);
    assert!(analysis.cohesion_score <= 1.0);
    assert_eq!(analysis.coupling_score, 0.0); // No external edges
    assert!(analysis.complexity_score > 0.0);
    assert!(analysis.complexity_score <= 1.0);
}

#[test]
fn test_layout_calculator_centroid() {
    let calculator = SubgraphLayoutCalculator::new();

    let mut positions = HashMap::new();
    positions.insert(NodeId::new(), Position3D::new(0.0, 0.0, 0.0));
    positions.insert(NodeId::new(), Position3D::new(4.0, 0.0, 0.0));
    positions.insert(NodeId::new(), Position3D::new(2.0, 3.0, 0.0));

    let centroid =
        calculator.calculate_collapsed_position(&positions, &CollapseStrategy::Centroid, None);

    assert_eq!(centroid.x, 2.0);
    assert!((centroid.y - 1.0).abs() < 0.01);
    assert_eq!(centroid.z, 0.0);
}

#[test]
fn test_layout_calculator_circular() {
    let calculator = SubgraphLayoutCalculator::new();

    let nodes = vec![NodeId::new(), NodeId::new(), NodeId::new(), NodeId::new()];
    let center = Position3D::new(0.0, 0.0, 0.0);

    let positions = calculator.calculate_expansion_layout(
        &nodes,
        center,
        &LayoutStrategy::Circular {
            radius: 5.0,
            start_angle: 0.0,
        },
        None,
        None,
    );

    assert_eq!(positions.len(), 4);

    // Verify all nodes are at the correct radius
    for (_, pos) in positions.iter() {
        let dist = (pos.x * pos.x + pos.y * pos.y).sqrt();
        assert!((dist - 5.0).abs() < 0.01);
    }
}

#[test]
fn test_layout_calculator_grid() {
    let calculator = SubgraphLayoutCalculator::new();

    let nodes: Vec<NodeId> = (0..6).map(|_| NodeId::new()).collect();
    let center = Position3D::new(0.0, 0.0, 0.0);

    let positions = calculator.calculate_expansion_layout(
        &nodes,
        center,
        &LayoutStrategy::Grid {
            columns: 3,
            spacing: 2.0,
        },
        None,
        None,
    );

    assert_eq!(positions.len(), 6);

    // Verify grid structure (should be 2 rows x 3 columns)
    let y_values: HashSet<i32> = positions
        .values()
        .map(|p| (p.y / 2.0).round() as i32)
        .collect();
    assert_eq!(y_values.len(), 2); // 2 rows

    let x_values: HashSet<i32> = positions
        .values()
        .map(|p| (p.x / 2.0).round() as i32)
        .collect();
    assert_eq!(x_values.len(), 3); // 3 columns
}

#[test]
fn test_layout_calculator_geometric() {
    let calculator = SubgraphLayoutCalculator::new();

    // Test with 6 nodes - should form a hexagon
    let nodes: Vec<NodeId> = (0..6).map(|_| NodeId::new()).collect();
    let center = Position3D::new(0.0, 0.0, 0.0);

    let positions = calculator.calculate_expansion_layout(
        &nodes,
        center,
        &LayoutStrategy::Geometric { spacing: 2.0 },
        None,
        None,
    );

    assert_eq!(positions.len(), 6);

    // Verify all nodes are equidistant from center
    let distances: Vec<f32> = positions
        .values()
        .map(|p| (p.x * p.x + p.y * p.y).sqrt())
        .collect();

    let first_dist = distances[0];
    for dist in &distances[1..] {
        assert!((dist - first_dist).abs() < 0.01);
    }
}

#[test]
fn test_split_criteria() {
    // Test geometric line split
    let line_split = SplitCriteria::GeometricLine {
        start: Position3D::new(0.0, 0.0, 0.0),
        end: Position3D::new(10.0, 10.0, 0.0),
    };

    // Test connectivity split
    let connectivity_split = SplitCriteria::Connectivity {
        min_cut: true,
        max_components: 3,
    };

    // Test clustering split
    let clustering_split = SplitCriteria::Clustering {
        algorithm: ClusteringAlgorithm::KMeans,
        num_clusters: 4,
    };

    // Just verify we can create these
    match line_split {
        SplitCriteria::GeometricLine { start, end } => {
            assert_eq!(start.x, 0.0);
            assert_eq!(end.x, 10.0);
        }
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_suggested_operations() {
    let split_op = SuggestedOperation::Split {
        reason: "Low cohesion detected".to_string(),
        criteria: SplitCriteria::Connectivity {
            min_cut: true,
            max_components: 2,
        },
        confidence: 0.85,
    };

    let optimize_op = SuggestedOperation::Optimize {
        reason: "High complexity".to_string(),
        optimization_type: OptimizationType::SimplifyStructure,
        confidence: 0.7,
    };

    // Verify we can pattern match on these
    match split_op {
        SuggestedOperation::Split { confidence, .. } => {
            assert_eq!(confidence, 0.85);
        }
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_hierarchical_layout() {
    let calculator = SubgraphLayoutCalculator::new();

    let nodes: Vec<NodeId> = (0..9).map(|_| NodeId::new()).collect();
    let center = Position3D::new(0.0, 0.0, 0.0);

    let positions = calculator.calculate_expansion_layout(
        &nodes,
        center,
        &LayoutStrategy::Hierarchical {
            direction: LayoutDirection::TopToBottom,
            layer_spacing: 5.0,
            node_spacing: 3.0,
        },
        None,
        None,
    );

    assert_eq!(positions.len(), 9);

    // Verify nodes are arranged in layers
    let y_values: HashSet<i32> = positions
        .values()
        .map(|p| (p.y / 5.0).round() as i32)
        .collect();

    // Should have multiple layers
    assert!(y_values.len() > 1);
}

#[test]
fn test_force_directed_layout() {
    let calculator = SubgraphLayoutCalculator::new();

    // Create a simple graph for force-directed layout
    let mut graph = Graph::new();
    let n1 = graph.add_node(NodeId::new());
    let n2 = graph.add_node(NodeId::new());
    let n3 = graph.add_node(NodeId::new());

    graph.add_edge(n1, n2, ());
    graph.add_edge(n2, n3, ());

    let nodes: Vec<NodeId> = graph.node_weights().cloned().collect();
    let center = Position3D::new(0.0, 0.0, 0.0);

    let positions = calculator.calculate_expansion_layout(
        &nodes,
        center,
        &LayoutStrategy::ForceDirected {
            iterations: 10,
            spring_strength: 0.1,
            repulsion_strength: 100.0,
        },
        Some(&graph),
        None,
    );

    assert_eq!(positions.len(), 3);

    // Verify nodes have spread out from initial positions
    for (_, pos) in positions.iter() {
        let dist = (pos.x * pos.x + pos.y * pos.y).sqrt();
        assert!(dist > 0.1); // Should have moved from center
    }
}
