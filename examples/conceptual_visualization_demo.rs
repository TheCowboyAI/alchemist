//! Demo of conceptual graph visualization with interactive manipulation

use alchemist::domain::conceptual_graph::{
    ConceptGraph, ConceptNode, ConceptEdge, ConceptType, ConceptRelationship,
    ConceptualPoint, QualityDimension, DimensionType, NodeId,
};
use alchemist::presentation::components::conceptual_visualization::{
    ConceptualNodeVisual, ConceptualEdgeVisual, ConceptualSpaceVisual,
    QualityDimensionAxis, VisualStyle,
};
use alchemist::presentation::plugins::GraphEditorPlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GraphEditorPlugin)
        .add_systems(Startup, setup_demo_graph)
        .add_systems(Update, rotate_camera)
        .run();
}

fn setup_demo_graph(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 4.0)),
    ));

    // Create a conceptual graph with DDD concepts
    let mut graph = ConceptGraph::new("Domain-Driven Design Concepts");

    // Define quality dimensions for DDD concepts
    graph = graph
        .with_dimension(QualityDimension::new("abstraction", DimensionType::Continuous, 0.0..1.0))
        .with_dimension(QualityDimension::new("complexity", DimensionType::Continuous, 0.0..1.0))
        .with_dimension(QualityDimension::new("coupling", DimensionType::Continuous, 0.0..1.0));

    // Create DDD concept nodes
    let entity_node = ConceptNode::Atom {
        id: NodeId::new(),
        concept_type: ConceptType::Entity,
        quality_position: ConceptualPoint::new(vec![0.7, 0.5, 0.3]), // High abstraction, medium complexity, low coupling
        properties: Default::default(),
    };

    let value_object_node = ConceptNode::Atom {
        id: NodeId::new(),
        concept_type: ConceptType::ValueObject,
        quality_position: ConceptualPoint::new(vec![0.6, 0.3, 0.1]), // Medium-high abstraction, low complexity, very low coupling
        properties: Default::default(),
    };

    let aggregate_node = ConceptNode::Atom {
        id: NodeId::new(),
        concept_type: ConceptType::Aggregate,
        quality_position: ConceptualPoint::new(vec![0.8, 0.7, 0.5]), // High abstraction, high complexity, medium coupling
        properties: Default::default(),
    };

    let policy_node = ConceptNode::Atom {
        id: NodeId::new(),
        concept_type: ConceptType::Policy,
        quality_position: ConceptualPoint::new(vec![0.5, 0.6, 0.2]), // Medium abstraction, medium-high complexity, low coupling
        properties: Default::default(),
    };

    let event_node = ConceptNode::Atom {
        id: NodeId::new(),
        concept_type: ConceptType::Event,
        quality_position: ConceptualPoint::new(vec![0.4, 0.4, 0.1]), // Medium-low abstraction, medium complexity, very low coupling
        properties: Default::default(),
    };

    // Add nodes to graph
    let entity_idx = graph.add_node(entity_node.clone());
    let value_idx = graph.add_node(value_object_node.clone());
    let aggregate_idx = graph.add_node(aggregate_node.clone());
    let policy_idx = graph.add_node(policy_node.clone());
    let event_idx = graph.add_node(event_node.clone());

    // Add relationships
    graph.add_edge(aggregate_idx, entity_idx, ConceptEdge::new(ConceptRelationship::PartOf));
    graph.add_edge(aggregate_idx, value_idx, ConceptEdge::new(ConceptRelationship::PartOf));
    graph.add_edge(policy_idx, aggregate_idx, ConceptEdge::new(ConceptRelationship::DependsOn));
    graph.add_edge(aggregate_idx, event_idx, ConceptEdge::new(ConceptRelationship::Triggers));

    // Spawn the conceptual space visualization
    commands.spawn((
        graph,
        ConceptualSpaceVisual {
            bounds: Vec3::new(20.0, 20.0, 20.0),
            grid_size: 2.0,
            show_grid: true,
            show_axes: true,
        },
    ));

    // Create quality dimension axes
    let axis_colors = [
        Color::srgb(1.0, 0.0, 0.0), // Red for abstraction
        Color::srgb(0.0, 1.0, 0.0), // Green for complexity
        Color::srgb(0.0, 0.0, 1.0), // Blue for coupling
    ];

    let dimension_names = ["Abstraction", "Complexity", "Coupling"];

    for (index, (color, name)) in axis_colors.iter().zip(dimension_names.iter()).enumerate() {
        commands.spawn((
            QualityDimensionAxis {
                dimension: QualityDimension::new(*name, DimensionType::Continuous, 0.0..1.0),
                axis_index: index,
                color: *color,
                length: 20.0,
                show_labels: true,
                label_interval: 0.2,
            },
            Transform::from_translation(Vec3::new(-10.0, -10.0, -10.0)),
            GlobalTransform::default(),
        ));
    }

    // Add a ground plane for reference
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(40.0, 40.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, -10.0, 0.0),
    ));

    // Add UI instructions
    commands.spawn((
        Text::new(
            "Conceptual Graph Visualization Demo\n\
            Controls:\n\
            - Left Click: Select node\n\
            - Ctrl+Click: Multi-select\n\
            - Drag: Move nodes\n\
            - Mouse Wheel: Zoom\n\
            - Right Drag: Rotate camera\n\
            - Tab: Switch tools\n\
            - Space: Toggle grid\n\
            - Escape: Clear selection"
        ),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

fn rotate_camera(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Camera3d>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.pressed(KeyCode::KeyQ) {
        for mut transform in query.iter_mut() {
            transform.rotate_around(
                Vec3::ZERO,
                Quat::from_rotation_y(time.delta_secs() * 0.5),
            );
        }
    }
    if keyboard.pressed(KeyCode::KeyE) {
        for mut transform in query.iter_mut() {
            transform.rotate_around(
                Vec3::ZERO,
                Quat::from_rotation_y(-time.delta_secs() * 0.5),
            );
        }
    }
}
