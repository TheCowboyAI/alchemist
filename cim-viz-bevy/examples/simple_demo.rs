//! Simple Demo of CIM-Viz-Bevy
//!
//! A minimal example showing how to visualize a graph with cim-viz-bevy.
//!
//! Run with: cargo run --example simple_demo --package cim-viz-bevy

use bevy::prelude::*;
use cim_viz_bevy::*;
use cim_contextgraph::{NodeId, EdgeId, ContextGraphId as GraphId};
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CimVizPlugin::default())
        .insert_resource(NodeMap::default())
        .insert_resource(GraphCreated(false))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_node_creation,
            handle_edge_creation,
            render_edges,
            create_initial_graph,
        ))
        .run();
}

#[derive(Resource, Default)]
struct NodeMap {
    nodes: HashMap<NodeId, Entity>,
}

#[derive(Resource)]
struct GraphCreated(bool);

fn setup(
    mut commands: Commands,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // Info text
    commands.spawn((
        Text::new("Simple CIM Graph Demo\nNodes and edges will appear automatically"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

fn create_initial_graph(
    bridge: Res<CategoricalBridge>,
    mut created: ResMut<GraphCreated>,
) {
    if created.0 {
        return;
    }
    created.0 = true;

    let graph_id = GraphId::new();
    let sender = bridge.domain_sender();

    // Create a simple triangle of nodes
    let node1 = NodeId::new();
    let node2 = NodeId::new();
    let node3 = NodeId::new();

    // Send node creation events
    let _ = sender.send(DomainEvent::NodeAdded {
        graph_id,
        node_id: node1,
        position: Some(Vec3::new(-3.0, 0.0, 0.0)),
        metadata: serde_json::json!({"name": "Node 1"}),
    });

    let _ = sender.send(DomainEvent::NodeAdded {
        graph_id,
        node_id: node2,
        position: Some(Vec3::new(3.0, 0.0, 0.0)),
        metadata: serde_json::json!({"name": "Node 2"}),
    });

    let _ = sender.send(DomainEvent::NodeAdded {
        graph_id,
        node_id: node3,
        position: Some(Vec3::new(0.0, 0.0, -3.0)),
        metadata: serde_json::json!({"name": "Node 3"}),
    });

    // Create edges
    let _ = sender.send(DomainEvent::EdgeAdded {
        graph_id,
        edge_id: EdgeId::new(),
        source: node1,
        target: node2,
        metadata: serde_json::json!({"label": "Edge 1-2"}),
    });

    let _ = sender.send(DomainEvent::EdgeAdded {
        graph_id,
        edge_id: EdgeId::new(),
        source: node2,
        target: node3,
        metadata: serde_json::json!({"label": "Edge 2-3"}),
    });

    let _ = sender.send(DomainEvent::EdgeAdded {
        graph_id,
        edge_id: EdgeId::new(),
        source: node3,
        target: node1,
        metadata: serde_json::json!({"label": "Edge 3-1"}),
    });

    info!("Created initial graph with 3 nodes and 3 edges");
}

fn handle_node_creation(
    mut commands: Commands,
    mut events: EventReader<CreateNodeVisual>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut node_map: ResMut<NodeMap>,
) {
    for event in events.read() {
        info!("Creating visual for node {:?} at {:?}", event.node_id, event.position);

        let entity = commands.spawn((
            NodeVisualBundle::new(event.node_id, event.graph_id, event.position),
            Mesh3d(meshes.add(Sphere::new(0.5).mesh())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.7, 0.9),
                ..default()
            })),
        )).id();

        node_map.nodes.insert(event.node_id, entity);
    }
}

fn handle_edge_creation(
    mut commands: Commands,
    mut events: EventReader<CreateEdgeVisual>,
    node_map: Res<NodeMap>,
) {
    for event in events.read() {
        if let (Some(&source_entity), Some(&target_entity)) = (
            node_map.nodes.get(&event.source_id),
            node_map.nodes.get(&event.target_id),
        ) {
            info!("Creating edge visual between {:?} and {:?}", event.source_id, event.target_id);

            commands.spawn(
                EdgeVisualBundle::new(event.edge_id, event.graph_id, source_entity, target_entity)
            );
        }
    }
}

fn render_edges(
    mut gizmos: Gizmos,
    edges: Query<&EdgeVisual>,
    nodes: Query<&Transform>,
) {
    for edge in edges.iter() {
        if let (Ok(source_transform), Ok(target_transform)) = (
            nodes.get(edge.source_entity),
            nodes.get(edge.target_entity),
        ) {
            gizmos.line(
                source_transform.translation,
                target_transform.translation,
                Color::srgb(0.6, 0.6, 0.6),
            );
        }
    }
}
