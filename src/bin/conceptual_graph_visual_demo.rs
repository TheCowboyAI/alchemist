//! Visual Demo of the Conceptual Graph Composition System
//!
//! This demo shows:
//! - Conceptual graphs with quality dimensions
//! - Visual positioning based on conceptual space
//! - Graph composition operations
//! - Category theory structures in action

use bevy::prelude::*;
use ia::domain::conceptual_graph::{
    ConceptGraph, ConceptNode, ConceptEdge, ConceptRelationship,
    ConceptType, NodeId, ConceptId,
    QualityDimension, DimensionType, ConceptualPoint,
    CategoryType, EnrichmentType,
    GraphComposer, CompositionBuilder, ProductType,
};
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_graphs)
        .run();
}



#[derive(Component)]
struct GraphMarker;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 20.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Create conceptual graphs
    let user_graph = create_user_graph();
    let email_graph = create_email_graph();

    // Visualize User graph (blue sphere on left)
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(3.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.6, 1.0),
            ..default()
        })),
        Transform::from_xyz(-10.0, 0.0, 0.0),
        GraphMarker,
    ));

    // Visualize Email graph (orange sphere on right)
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(2.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.6, 0.2),
            ..default()
        })),
        Transform::from_xyz(10.0, 0.0, 0.0),
        GraphMarker,
    ));

    // Compose and visualize (purple sphere in center)
    let composed = CompositionBuilder::new()
        .with_base(user_graph)
        .embed(email_graph)
        .build()
        .expect("Composition failed");

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(4.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.2, 1.0),
            ..default()
        })),
        Transform::from_xyz(0.0, 5.0, 0.0),
        GraphMarker,
    ));

    // Info text
    commands.spawn((
        Text::new("Conceptual Graph Visual Demo\n\nBlue = User Graph\nOrange = Email Graph\nPurple = Composed Graph"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        },
    ));

    println!("Composed graph has {} nodes", composed.node_count());
}

fn create_user_graph() -> ConceptGraph {
    let mut graph = ConceptGraph::new("User")
        .with_category(CategoryType::Database)
        .with_dimension(QualityDimension::new(
            "authority",
            DimensionType::Continuous,
            0.0..1.0,
        ));

    let user_node = ConceptNode::Atom {
        id: NodeId::new(),
        concept_type: ConceptType::Entity,
        quality_position: ConceptualPoint::new(vec![0.0, 0.0, 0.0]),
        properties: HashMap::new(),
    };

    let profile_node = ConceptNode::Atom {
        id: NodeId::new(),
        concept_type: ConceptType::ValueObject,
        quality_position: ConceptualPoint::new(vec![1.0, 0.0, 0.0]),
        properties: HashMap::new(),
    };

    let user_idx = graph.add_node(user_node);
    let profile_idx = graph.add_node(profile_node);

    graph.add_edge(
        profile_idx,
        user_idx,
        ConceptEdge::new(ConceptRelationship::PartOf),
    );

    graph
}

fn create_email_graph() -> ConceptGraph {
    let mut graph = ConceptGraph::new("Email")
        .with_category(CategoryType::Simple)
        .with_dimension(QualityDimension::new(
            "validity",
            DimensionType::Binary,
            0.0..1.0,
        ));

    let email_node = ConceptNode::Atom {
        id: NodeId::new(),
        concept_type: ConceptType::ValueObject,
        quality_position: ConceptualPoint::new(vec![0.5, 1.0, 0.0]),
        properties: HashMap::new(),
    };

    graph.add_node(email_node);
    graph
}

fn rotate_graphs(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<GraphMarker>>,
) {
    for mut transform in query.iter_mut() {
        transform.rotate_y(time.delta_secs() * 0.5);
    }
}


