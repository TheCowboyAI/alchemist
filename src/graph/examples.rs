//! Example systems demonstrating graph usage
//!
//! These systems show how to create and manipulate graphs using
//! the event-driven architecture.

use bevy::prelude::*;
use crate::graph::{components::*, events::*};
use std::collections::HashMap;

/// Example system that creates a simple graph on startup
pub fn create_example_graph(
    mut commands: Commands,
    mut graph_created_events: EventWriter<GraphCreatedEvent>,
    mut node_added_events: EventWriter<NodeAddedEvent>,
    mut edge_created_events: EventWriter<EdgeCreatedEvent>,
) {
    // Create a new graph
    let graph_id = GraphId::new();
    let metadata = GraphMetadata {
        name: "Example Knowledge Graph".to_string(),
        description: "A simple graph demonstrating the component structure".to_string(),
        domain_type: "knowledge".to_string(),
        ..Default::default()
    };

    // Spawn the graph entity
    commands.spawn(GraphBundle {
        graph: Graph,
        id: graph_id,
        metadata: metadata.clone(),
    });

    // Fire the graph created event
    graph_created_events.write(GraphCreatedEvent {
        graph_id,
        metadata,
    });

    // Create some nodes
    let node1_id = NodeId::new();
    let node2_id = NodeId::new();
    let node3_id = NodeId::new();

    // Add first node - "Rust"
    let mut node1_props = HashMap::new();
    node1_props.insert("label".to_string(), serde_json::json!("Rust"));
    node1_props.insert("type".to_string(), serde_json::json!("Technology"));
    node1_props.insert("description".to_string(), serde_json::json!("Systems programming language"));

    node_added_events.write(NodeAddedEvent {
        graph_id,
        node_id: node1_id,
        position: Vec3::new(-5.0, 0.0, 0.0),
        properties: node1_props,
    });

    // Add second node - "Bevy"
    let mut node2_props = HashMap::new();
    node2_props.insert("label".to_string(), serde_json::json!("Bevy"));
    node2_props.insert("type".to_string(), serde_json::json!("Framework"));
    node2_props.insert("description".to_string(), serde_json::json!("Game engine built in Rust"));

    node_added_events.write(NodeAddedEvent {
        graph_id,
        node_id: node2_id,
        position: Vec3::new(5.0, 0.0, 0.0),
        properties: node2_props,
    });

    // Add third node - "ECS"
    let mut node3_props = HashMap::new();
    node3_props.insert("label".to_string(), serde_json::json!("ECS"));
    node3_props.insert("type".to_string(), serde_json::json!("Pattern"));
    node3_props.insert("description".to_string(), serde_json::json!("Entity Component System"));

    node_added_events.write(NodeAddedEvent {
        graph_id,
        node_id: node3_id,
        position: Vec3::new(0.0, 5.0, 0.0),
        properties: node3_props,
    });

    // Create edges
    let edge1_id = EdgeId::new();
    let mut edge1_props = HashMap::new();
    edge1_props.insert("relationship".to_string(), serde_json::json!("implements"));

    edge_created_events.write(EdgeCreatedEvent {
        graph_id,
        edge_id: edge1_id,
        source: node2_id,
        target: node1_id,
        weight: 1.0,
        properties: edge1_props,
    });

    let edge2_id = EdgeId::new();
    let mut edge2_props = HashMap::new();
    edge2_props.insert("relationship".to_string(), serde_json::json!("uses"));

    edge_created_events.write(EdgeCreatedEvent {
        graph_id,
        edge_id: edge2_id,
        source: node2_id,
        target: node3_id,
        weight: 1.0,
        properties: edge2_props,
    });

    info!("Example graph created with ID: {:?}", graph_id);
}

/// System that listens to graph events and spawns the corresponding entities
pub fn handle_graph_events(
    mut commands: Commands,
    mut node_added_events: EventReader<NodeAddedEvent>,
    mut edge_created_events: EventReader<EdgeCreatedEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Handle node creation
    for event in node_added_events.read() {
        // Log node creation
        let label = event.properties.get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        info!("Creating node '{}' at position {:?}", label, event.position);

        // Create the visual representation
        let node_bundle = NodeBundle {
            node: GraphNode {
                graph_id: event.graph_id,
                position: event.position,
                properties: event.properties.clone(),
            },
            id: event.node_id,
            state: ElementState::Normal,
            selectable: Selectable,
            draggable: Draggable,
            transform: Transform::from_translation(event.position),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
        };

        // Spawn the node entity with a sphere mesh
        commands.spawn((
            node_bundle,
            Mesh3d(meshes.add(Sphere::new(0.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.5, 0.8),
                ..default()
            })),
        ));

        info!("Spawned node entity: {:?} at {:?}", event.node_id, event.position);
    }

    // Handle edge creation
    for event in edge_created_events.read() {
        info!("Creating edge from {:?} to {:?}", event.source, event.target);

        // For now, just create the edge entity without visual representation
        // Visual representation will be added in the rendering phase
        let edge_bundle = EdgeBundle {
            edge: GraphEdge {
                graph_id: event.graph_id,
                source: event.source,
                target: event.target,
                weight: event.weight,
                properties: event.properties.clone(),
            },
            id: event.edge_id,
            state: ElementState::Normal,
            selectable: Selectable,
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
        };

        commands.spawn(edge_bundle);

        info!("Spawned edge entity: {:?}", event.edge_id);
    }
}
