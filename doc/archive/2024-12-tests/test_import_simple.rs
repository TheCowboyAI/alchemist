//! Simple import test with detailed logging

use bevy::prelude::*;
use ia::application::{CommandEvent, EventNotification};
use ia::domain::{
    commands::{Command, GraphCommand, ImportOptions, ImportSource, graph_commands::MergeBehavior},
    events::DomainEvent,
    value_objects::{GraphId, Position3D},
};
use ia::presentation::plugins::GraphEditorPlugin;
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Import Test".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GraphEditorPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (log_events, log_nodes))
        .run();
}

fn setup(mut commands: EventWriter<CommandEvent>) {
    println!("\n=== IMPORT TEST STARTING ===\n");

    // Create a graph first
    let graph_id = GraphId::new();
    println!("1. Creating graph: {:?}", graph_id);

    commands.write(CommandEvent {
        command: Command::Graph(GraphCommand::CreateGraph {
            id: graph_id,
            name: "Test Graph".to_string(),
            metadata: HashMap::new(),
        }),
    });

    // Import sample data
    println!("2. Importing sample_graph.json into graph: {:?}", graph_id);

    commands.write(CommandEvent {
        command: Command::Graph(GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::File {
                path: "examples/data/sample_graph.json".to_string(),
            },
            format: "arrows_app".to_string(),
            options: ImportOptions {
                merge_behavior: MergeBehavior::AlwaysCreate,
                id_prefix: Some("test".to_string()),
                position_offset: Some(Position3D {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
                mapping: None,
                validate: true,
                max_nodes: Some(1000),
            },
        }),
    });

    println!("3. Commands sent!\n");
}

fn log_events(mut events: EventReader<EventNotification>) {
    for event in events.read() {
        println!("EVENT: {:?}", event.event.event_type());

        match &event.event {
            DomainEvent::Graph(graph_event) => {
                println!("  Graph Event: {:?}", graph_event);
            }
            DomainEvent::Node(node_event) => {
                println!("  Node Event: {:?}", node_event);
            }
            DomainEvent::Edge(edge_event) => {
                println!("  Edge Event: {:?}", edge_event);
            }
            _ => {}
        }
    }
}

fn log_nodes(nodes: Query<&ia::presentation::components::GraphNode>, mut logged: Local<bool>) {
    let count = nodes.iter().count();
    if count > 0 && !*logged {
        println!("\nNODES IN SCENE: {}", count);
        for node in nodes.iter() {
            println!("  - Node: {:?} in Graph: {:?}", node.node_id, node.graph_id);
        }
        *logged = true;
    }
}
