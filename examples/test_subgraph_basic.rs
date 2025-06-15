//! Basic test to verify subgraph operations work

use chrono::Utc;
use ia::domain::{
    commands::SubgraphOperationCommand,
    events::SubgraphOperationEvent,
    services::{SubgraphAnalyzer, SubgraphLayoutCalculator},
    value_objects::{
        CollapseStrategy, EdgeId, GraphId, LayoutStrategy, NodeId, Position3D, SubgraphAnalysis,
        SubgraphId, SubgraphState, SubgraphStatistics, SuggestedOperation,
    },
};
use petgraph::graph::Graph;
use std::collections::{HashMap, HashSet};

fn main() {
    println!("=== Basic Subgraph Operations Test ===\n");

    // Test 1: Basic Value Objects
    println!("1. Testing Value Objects:");
    let graph_id = GraphId::new();
    let subgraph_id = SubgraphId::new();
    println!("   ✓ Created GraphId: {}", graph_id);
    println!("   ✓ Created SubgraphId: {}", subgraph_id);

    let expanded = SubgraphState::Expanded;
    let collapsed = SubgraphState::Collapsed;
    let transitioning = SubgraphState::Transitioning {
        progress: 0.5,
        from: Box::new(SubgraphState::Expanded),
        to: Box::new(SubgraphState::Collapsed),
    };
    println!("   ✓ Created SubgraphState variants");
    println!("     - Expanded: {}", expanded);
    println!("     - Collapsed: {}", collapsed);
    println!("     - Transitioning: {}", transitioning);

    // Test 2: Events
    println!("\n2. Testing Events:");
    let event = SubgraphOperationEvent::SubgraphCollapsed {
        graph_id,
        subgraph_id,
        collapsed_at: Position3D::new(50.0, 50.0, 0.0),
        contained_nodes: vec![NodeId::new(), NodeId::new()],
        collapse_strategy: CollapseStrategy::Centroid,
        timestamp: Utc::now(),
    };
    println!("   ✓ Created SubgraphCollapsed event");
    println!("   ✓ Event graph_id: {}", event.graph_id());
    println!(
        "   ✓ Event primary_subgraph_id: {:?}",
        event.primary_subgraph_id()
    );

    // Test 3: Commands
    println!("\n3. Testing Commands:");
    let command = SubgraphOperationCommand::CollapseSubgraph {
        graph_id,
        subgraph_id,
        strategy: CollapseStrategy::WeightedCenter,
    };
    println!("   ✓ Created CollapseSubgraph command");
    println!("   ✓ Command type: {}", command.command_type());
    println!("   ✓ Command graph_id: {}", command.graph_id());

    // Test 4: Domain Services
    println!("\n4. Testing Domain Services:");

    // Create a simple graph
    let mut graph = Graph::new();
    let n1 = graph.add_node(NodeId::new());
    let n2 = graph.add_node(NodeId::new());
    let n3 = graph.add_node(NodeId::new());

    graph.add_edge(n1, n2, EdgeId::new());
    graph.add_edge(n2, n3, EdgeId::new());

    let mut subgraph_nodes = HashSet::new();
    subgraph_nodes.insert(*graph.node_weight(n1).unwrap());
    subgraph_nodes.insert(*graph.node_weight(n2).unwrap());

    // Test analyzer
    let analyzer = SubgraphAnalyzer::new();
    let analysis = analyzer.analyze_subgraph(&graph, &subgraph_nodes);
    println!("   ✓ SubgraphAnalyzer created and analyzed");
    println!("     - Nodes: {}", analysis.statistics.node_count);
    println!("     - Edges: {}", analysis.statistics.edge_count);
    println!("     - Cohesion: {:.2}", analysis.cohesion_score);

    // Test layout calculator
    let layout_calc = SubgraphLayoutCalculator::new();
    let mut positions = HashMap::new();
    positions.insert(
        *graph.node_weight(n1).unwrap(),
        Position3D::new(0.0, 0.0, 0.0),
    );
    positions.insert(
        *graph.node_weight(n2).unwrap(),
        Position3D::new(10.0, 0.0, 0.0),
    );

    let collapsed_pos =
        layout_calc.calculate_collapsed_position(&positions, &CollapseStrategy::Centroid, None);
    println!("   ✓ SubgraphLayoutCalculator calculated collapse position");
    println!(
        "     - Position: ({:.2}, {:.2}, {:.2})",
        collapsed_pos.x, collapsed_pos.y, collapsed_pos.z
    );

    println!("\n✅ All basic tests passed!");
}
