//! Test import visibility

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
                title: "Import Visibility Test".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GraphEditorPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (monitor_nodes, check_visibility))
        .run();
}

fn setup(mut commands: EventWriter<CommandEvent>) {
    println!("\n=== IMPORT VISIBILITY TEST ===\n");

    // Create a graph
    let graph_id = GraphId::new();
    println!("Creating graph with ID: {:?}", graph_id);

    commands.write(CommandEvent {
        command: Command::Graph(GraphCommand::CreateGraph {
            id: graph_id,
            name: "Test Graph".to_string(),
            metadata: HashMap::new(),
        }),
    });

    println!("Press 'I' to import nodes\n");
}

fn monitor_nodes(
    nodes: Query<(Entity, &GraphNode, Option<&Transform>, Option<&Visibility>)>,
    mut last_count: Local<usize>,
) {
    let current_count = nodes.iter().count();
    if current_count != *last_count {
        println!("\n[NODE UPDATE] Total nodes: {}", current_count);

        for (entity, node, transform, visibility) in nodes.iter() {
            println!("  Entity {:?}:", entity);
            println!("    - Node ID: {:?}", node.node_id);
            println!("    - Graph ID: {:?}", node.graph_id);

            if let Some(t) = transform {
                println!("    - Position: ({:.2}, {:.2}, {:.2})", t.translation.x, t.translation.y, t.translation.z);
                println!("    - Scale: ({:.2}, {:.2}, {:.2})", t.scale.x, t.scale.y, t.scale.z);
            } else {
                println!("    - NO TRANSFORM!");
            }

            if let Some(v) = visibility {
                println!("    - Visibility: {:?}", v);
            } else {
                println!("    - NO VISIBILITY!");
            }
        }

        *last_count = current_count;
    }
}

fn check_visibility(
    keyboard: Res<ButtonInput<KeyCode>>,
    transforms: Query<(Entity, &Transform), With<GraphNode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyV) {
        println!("\n[VISIBILITY CHECK]");
        for (entity, transform) in transforms.iter() {
            println!("Entity {:?}: pos=({:.2}, {:.2}, {:.2}), scale=({:.2}, {:.2}, {:.2})",
                entity,
                transform.translation.x, transform.translation.y, transform.translation.z,
                transform.scale.x, transform.scale.y, transform.scale.z
            );
        }
    }
}
