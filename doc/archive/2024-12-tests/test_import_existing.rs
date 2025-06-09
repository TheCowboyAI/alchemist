//! Test import into existing graph

use bevy::prelude::*;
use ia::application::{CommandEvent, EventNotification};
use ia::domain::{
    commands::{Command, GraphCommand, ImportSource, ImportOptions, graph_commands::MergeBehavior},
    events::DomainEvent,
    value_objects::{GraphId, Position3D},
};
use ia::presentation::plugins::GraphEditorPlugin;
use ia::presentation::components::{GraphContainer, GraphNode};
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Import Existing Test".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GraphEditorPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (log_events, monitor_nodes, trigger_import))
        .run();
}

fn setup(mut commands: EventWriter<CommandEvent>) {
    println!("\n=== IMPORT INTO EXISTING GRAPH TEST ===\n");

    // Create initial graph
    let graph_id = GraphId::new();
    println!("1. Creating initial graph: {:?}", graph_id);

    commands.write(CommandEvent {
        command: Command::Graph(GraphCommand::CreateGraph {
            id: graph_id,
            name: "Main Graph".to_string(),
            metadata: HashMap::new(),
        }),
    });

    println!("2. Initial graph created. Press 'I' to import into it.\n");
}

fn trigger_import(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: EventWriter<CommandEvent>,
    graph_query: Query<&GraphContainer>,
    mut imported: Local<bool>,
) {
    if keyboard.just_pressed(KeyCode::KeyI) && !*imported {
        println!("\n3. 'I' pressed - importing into existing graph");

        if let Ok(container) = graph_query.get_single() {
            println!("   Found existing graph: {:?}", container.graph_id);

            commands.write(CommandEvent {
                command: Command::Graph(GraphCommand::ImportGraph {
                    graph_id: container.graph_id,
                    source: ImportSource::File {
                        path: "examples/data/sample_graph.json".to_string(),
                    },
                    format: "arrows_app".to_string(),
                    options: ImportOptions {
                        merge_behavior: MergeBehavior::MergePreferImported,
                        id_prefix: Some("test".to_string()),
                        position_offset: Some(Position3D { x: 5.0, y: 0.0, z: 0.0 }),
                        mapping: None,
                        validate: true,
                        max_nodes: Some(1000),
                    },
                }),
            });

            println!("   Import command sent to graph: {:?}", container.graph_id);
            *imported = true;
        } else {
            println!("   ERROR: No graph container found!");
        }
    }
}

fn log_events(mut events: EventReader<EventNotification>) {
    for event in events.read() {
        match &event.event {
            DomainEvent::Graph(graph_event) => {
                println!("GRAPH EVENT: {:?}", graph_event);
            }
            DomainEvent::Node(node_event) => {
                println!("NODE EVENT: {:?}", node_event);
            }
            DomainEvent::Edge(edge_event) => {
                println!("EDGE EVENT: {:?}", edge_event);
            }
            _ => {}
        }
    }
}

fn monitor_nodes(
    nodes: Query<(&GraphNode, &Transform)>,
    mut last_count: Local<usize>,
) {
    let current_count = nodes.iter().count();
    if current_count != *last_count {
        println!("\nNODE COUNT CHANGED: {} -> {}", *last_count, current_count);

        if current_count > 0 {
            println!("Current nodes:");
            for (node, transform) in nodes.iter() {
                println!("  - Node: {:?} at position ({:.2}, {:.2}, {:.2})",
                    node.node_id,
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z
                );
            }

            // Check if all nodes belong to the same graph
            let graph_ids: std::collections::HashSet<_> = nodes.iter()
                .map(|(node, _)| node.graph_id)
                .collect();

            if graph_ids.len() == 1 {
                println!("✓ All nodes belong to the same graph!");
            } else {
                println!("✗ Nodes belong to {} different graphs!", graph_ids.len());
            }
        }

        *last_count = current_count;
    }
}
