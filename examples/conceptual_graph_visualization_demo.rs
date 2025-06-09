//! Conceptual Graph Visualization Demo
//!
//! This example demonstrates the visualization of conceptual graphs with quality dimensions.

use bevy::prelude::*;
use ia::{
    domain::conceptual_graph::{
        ConceptGraph, ConceptNode as DomainConceptNode, ConceptType,
        ConceptEdge as DomainConceptEdge, QualityDimension, DimensionType,
        DistanceMetric, ConceptualPoint, NodeId,
    },
    presentation::{
        components::{
            ConceptualSpaceVisual, SpaceId, QualityDimensionAxis,
            SpaceBounds, GridSettings,
        },
        systems::{ConceptualVisualizationPlugin, ConceptNodeEntity},
    },
};
use std::ops::Range;
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ConceptualVisualizationPlugin)
        .add_systems(Startup, (setup_camera, setup_lighting, setup_conceptual_space))
        .add_systems(Startup, create_demo_concepts.after(setup_conceptual_space))
        .run();
}

fn setup_camera(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, 10.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn setup_lighting(mut commands: Commands) {
    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
    });

    // Directional light
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4,
            std::f32::consts::FRAC_PI_4,
            0.0,
        )),
    ));
}

fn setup_conceptual_space(mut commands: Commands) {
    // Create quality dimensions
    let dimensions = vec![
        QualityDimensionAxis {
            dimension: QualityDimension::new(
                "Complexity",
                DimensionType::Continuous,
                0.0..10.0,
            ),
            axis_direction: Vec3::X,
            scale: 1.0,
            color: Color::srgb(1.0, 0.0, 0.0),
            show_labels: true,
            label_entities: vec![],
        },
        QualityDimensionAxis {
            dimension: QualityDimension::new(
                "Abstraction",
                DimensionType::Continuous,
                0.0..10.0,
            ),
            axis_direction: Vec3::Y,
            scale: 1.0,
            color: Color::srgb(0.0, 1.0, 0.0),
            show_labels: true,
            label_entities: vec![],
        },
        QualityDimensionAxis {
            dimension: QualityDimension::new(
                "Performance",
                DimensionType::Continuous,
                0.0..10.0,
            ),
            axis_direction: Vec3::Z,
            scale: 1.0,
            color: Color::srgb(0.0, 0.0, 1.0),
            show_labels: true,
            label_entities: vec![],
        },
    ];

    // Create conceptual space
    commands.spawn(ConceptualSpaceVisual {
        space_id: SpaceId::new(),
        dimensions,
        origin: Vec3::ZERO,
        bounds: SpaceBounds {
            min: Vec3::new(-10.0, -2.0, -10.0),
            max: Vec3::new(10.0, 10.0, 10.0),
        },
        grid_settings: GridSettings {
            visible: true,
            spacing: 1.0,
            color: Color::srgba(0.5, 0.5, 0.5, 0.3),
            line_width: 0.01,
            subdivisions: 20,
        },
    });
}

fn create_demo_concepts(mut commands: Commands) {
    // Create some example concepts at different positions in quality space

    // Low complexity, low abstraction, high performance - "Simple Algorithm"
    commands.spawn(ConceptNodeEntity {
        node: DomainConceptNode::Atom {
            id: NodeId::new(),
            concept_type: ConceptType::Atom,
            quality_position: ConceptualPoint::new(vec![2.0, 2.0, 8.0]),
            properties: HashMap::new(),
        },
    });

    // Medium complexity, high abstraction, medium performance - "Design Pattern"
    commands.spawn(ConceptNodeEntity {
        node: DomainConceptNode::Composite {
            id: NodeId::new(),
            quality_position: ConceptualPoint::new(vec![5.0, 8.0, 5.0]),
            subgraph: Box::new(ConceptGraph::new("Design Pattern")),
        },
    });

    // High complexity, medium abstraction, low performance - "Complex Implementation"
    commands.spawn(ConceptNodeEntity {
        node: DomainConceptNode::Function {
            id: NodeId::new(),
            quality_position: ConceptualPoint::new(vec![8.0, 5.0, 2.0]),
            input_type: ConceptType::Entity,
            output_type: ConceptType::ValueObject,
            implementation: ia::domain::conceptual_graph::FunctionImpl::BuiltIn("transform".to_string()),
        },
    });

    // Low complexity, high abstraction, high performance - "Pure Function"
    commands.spawn(ConceptNodeEntity {
        node: DomainConceptNode::Function {
            id: NodeId::new(),
            quality_position: ConceptualPoint::new(vec![2.0, 8.0, 8.0]),
            input_type: ConceptType::ValueObject,
            output_type: ConceptType::ValueObject,
            implementation: ia::domain::conceptual_graph::FunctionImpl::BuiltIn("map".to_string()),
        },
    });

    // Medium all dimensions - "Balanced Solution"
    commands.spawn(ConceptNodeEntity {
        node: DomainConceptNode::Composite {
            id: NodeId::new(),
            quality_position: ConceptualPoint::new(vec![5.0, 5.0, 5.0]),
            subgraph: Box::new(ConceptGraph::new("Balanced Solution")),
        },
    });

    println!("Created 5 demo concepts in 3D quality space:");
    println!("- Simple Algorithm (low complexity, low abstraction, high performance)");
    println!("- Design Pattern (medium complexity, high abstraction, medium performance)");
    println!("- Complex Implementation (high complexity, medium abstraction, low performance)");
    println!("- Pure Function (low complexity, high abstraction, high performance)");
    println!("- Balanced Solution (medium in all dimensions)");
    println!("\nUse mouse to rotate camera and observe the 3D conceptual space!");
}
