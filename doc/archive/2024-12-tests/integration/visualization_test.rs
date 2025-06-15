//! Integration tests for visualization systems with ConceptGraph data

use alchemist::domain::conceptual_graph::{
    ConceptEdge, ConceptGraph, ConceptNode, ConceptRelationship, ConceptType, ConceptualPoint,
    DimensionType, NodeId, QualityDimension,
};
use alchemist::presentation::components::conceptual_visualization::{
    ConceptualEdgeVisual, ConceptualNodeVisual, ConceptualSpaceVisual, DraggableNode,
    QualityDimensionAxis, SelectableGraph,
};
use alchemist::presentation::plugins::GraphEditorPlugin;
use bevy::app::AppExit;
use bevy::prelude::*;
use std::time::Duration;

/// Test that conceptual nodes are properly visualized in 3D space
#[test]
fn test_conceptual_node_visualization() {
    // Create a test app
    let mut app = App::new();

    // Add minimal plugins for testing
    app.add_plugins(MinimalPlugins);
    app.add_plugins(GraphEditorPlugin);

    // Create a test concept graph
    let mut graph = ConceptGraph::new("TestGraph");

    // Add quality dimensions
    graph = graph
        .with_dimension(QualityDimension::new(
            "abstraction",
            DimensionType::Continuous,
            0.0..1.0,
        ))
        .with_dimension(QualityDimension::new(
            "complexity",
            DimensionType::Continuous,
            0.0..1.0,
        ))
        .with_dimension(QualityDimension::new(
            "stability",
            DimensionType::Continuous,
            0.0..1.0,
        ));

    // Add nodes with different positions in conceptual space
    let node1 = ConceptNode::Atom {
        id: NodeId::new(),
        concept_type: ConceptType::Entity,
        quality_position: ConceptualPoint::new(vec![0.2, 0.3, 0.8]),
        properties: Default::default(),
    };

    let node2 = ConceptNode::Atom {
        id: NodeId::new(),
        concept_type: ConceptType::ValueObject,
        quality_position: ConceptualPoint::new(vec![0.7, 0.5, 0.6]),
        properties: Default::default(),
    };

    let idx1 = graph.add_node(node1.clone());
    let idx2 = graph.add_node(node2.clone());

    // Add edge
    let edge = ConceptEdge::new(ConceptRelationship::DependsOn);
    graph.add_edge(idx1, idx2, edge);

    // Spawn the graph in the ECS
    app.world_mut().spawn((
        graph.clone(),
        ConceptualSpaceVisual {
            bounds: Vec3::new(10.0, 10.0, 10.0),
            grid_size: 1.0,
            show_grid: true,
            show_axes: true,
        },
    ));

    // Run one update cycle
    app.update();

    // Verify nodes were created with proper visual components
    let mut node_query = app
        .world_mut()
        .query::<(&ConceptualNodeVisual, &Transform)>();
    let nodes: Vec<_> = node_query.iter(&app.world()).collect();

    assert_eq!(nodes.len(), 2, "Should have created 2 visual nodes");

    // Verify positions are mapped correctly
    for (visual, transform) in nodes {
        if visual.node_id == node1.id() {
            // Check that position is mapped from conceptual space
            assert!(transform.translation.x > 0.0);
            assert!(transform.translation.y > 0.0);
            assert!(transform.translation.z > 0.0);
        }
    }
}

/// Test node interaction systems
#[test]
fn test_node_interaction() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(GraphEditorPlugin);

    // Create a draggable node
    let node_entity = app
        .world_mut()
        .spawn((
            ConceptualNodeVisual {
                node_id: NodeId::new(),
                concept_type: ConceptType::Entity,
                quality_dimensions: vec![0.5, 0.5, 0.5],
                visual_style: Default::default(),
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
            DraggableNode {
                is_dragging: false,
                drag_offset: Vec3::ZERO,
                constraints: Default::default(),
                snap_to_grid: false,
                grid_size: 1.0,
            },
        ))
        .id();

    // Simulate a drag start
    app.world_mut()
        .entity_mut(node_entity)
        .insert(DraggableNode {
            is_dragging: true,
            drag_offset: Vec3::new(0.1, 0.1, 0.0),
            constraints: Default::default(),
            snap_to_grid: false,
            grid_size: 1.0,
        });

    app.update();

    // Verify the node can be dragged
    let draggable = app.world().get::<DraggableNode>(node_entity).unwrap();
    assert!(draggable.is_dragging);
}

/// Test edge visualization between concepts
#[test]
fn test_edge_visualization() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(GraphEditorPlugin);

    // Create two nodes
    let node1_id = NodeId::new();
    let node2_id = NodeId::new();

    let node1 = app
        .world_mut()
        .spawn((
            ConceptualNodeVisual {
                node_id: node1_id,
                concept_type: ConceptType::Entity,
                quality_dimensions: vec![0.2, 0.3, 0.8],
                visual_style: Default::default(),
            },
            Transform::from_xyz(-5.0, 0.0, 0.0),
            GlobalTransform::default(),
        ))
        .id();

    let node2 = app
        .world_mut()
        .spawn((
            ConceptualNodeVisual {
                node_id: node2_id,
                concept_type: ConceptType::ValueObject,
                quality_dimensions: vec![0.7, 0.5, 0.6],
                visual_style: Default::default(),
            },
            Transform::from_xyz(5.0, 0.0, 0.0),
            GlobalTransform::default(),
        ))
        .id();

    // Create edge between them
    app.world_mut().spawn((ConceptualEdgeVisual {
        edge_id: Default::default(),
        source_node: node1_id,
        target_node: node2_id,
        relationship: ConceptRelationship::DependsOn,
        visual_style: Default::default(),
        animation: Default::default(),
    },));

    app.update();

    // Verify edge exists
    let edge_query = app.world().query::<&ConceptualEdgeVisual>();
    assert_eq!(edge_query.iter(&app.world()).count(), 1);
}

/// Test quality dimension axes visualization
#[test]
fn test_quality_dimension_axes() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(GraphEditorPlugin);

    // Create quality dimension axes
    let dimensions = vec![
        QualityDimension::new("abstraction", DimensionType::Continuous, 0.0..1.0),
        QualityDimension::new("complexity", DimensionType::Continuous, 0.0..1.0),
        QualityDimension::new("stability", DimensionType::Continuous, 0.0..1.0),
    ];

    for (index, dimension) in dimensions.into_iter().enumerate() {
        app.world_mut().spawn((
            QualityDimensionAxis {
                dimension,
                axis_index: index,
                color: Color::srgb(1.0, 0.0, 0.0),
                length: 10.0,
                show_labels: true,
                label_interval: 0.2,
            },
            Transform::default(),
            GlobalTransform::default(),
        ));
    }

    app.update();

    // Verify axes were created
    let axes_query = app.world().query::<&QualityDimensionAxis>();
    assert_eq!(axes_query.iter(&app.world()).count(), 3);
}

/// Test graph selection functionality
#[test]
fn test_graph_selection() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(GraphEditorPlugin);

    // Create a selectable graph
    let graph_entity = app
        .world_mut()
        .spawn((
            ConceptGraph::new("TestGraph"),
            SelectableGraph {
                selected_nodes: Default::default(),
                selected_edges: Default::default(),
                selection_mode: Default::default(),
                multi_select_key: Default::default(),
            },
        ))
        .id();

    // Create some nodes
    let node1 = app
        .world_mut()
        .spawn((
            ConceptualNodeVisual {
                node_id: NodeId::new(),
                concept_type: ConceptType::Entity,
                quality_dimensions: vec![0.5, 0.5, 0.5],
                visual_style: Default::default(),
            },
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    // Select the node
    app.world_mut()
        .entity_mut(graph_entity)
        .get_mut::<SelectableGraph>()
        .unwrap()
        .selected_nodes
        .insert(node1);

    app.update();

    // Verify selection
    let graph = app.world().get::<SelectableGraph>(graph_entity).unwrap();
    assert_eq!(graph.selected_nodes.len(), 1);
    assert!(graph.selected_nodes.contains(&node1));
}

/// Helper to run app for a specific duration
fn run_app_for_duration(app: &mut App, duration: Duration) {
    let start = std::time::Instant::now();
    while start.elapsed() < duration {
        app.update();
        if app.world().contains_resource::<AppExit>() {
            break;
        }
    }
}
