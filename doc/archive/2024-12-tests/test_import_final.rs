//! Final import test - verify nodes are visible after import

use bevy::prelude::*;
use ia::application::{CommandEvent, EventNotification};
use ia::domain::{
    commands::{Command, GraphCommand},
    events::DomainEvent,
    value_objects::GraphId,
};
use ia::presentation::components::GraphNode;
use ia::presentation::plugins::GraphEditorPlugin;
use std::collections::HashMap;

fn main() {
    println!("\n=== FINAL IMPORT TEST ===");
    println!("Press 'I' to import nodes - they should be VISIBLE now!");
    println!("Press 'C' to count visible nodes\n");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GraphEditorPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (monitor_events, count_nodes))
        .run();
}

fn setup(mut commands: EventWriter<CommandEvent>) {
    // Create initial graph
    let graph_id = GraphId::new();
    commands.write(CommandEvent {
        command: Command::Graph(GraphCommand::CreateGraph {
            id: graph_id,
            name: "Import Test Graph".to_string(),
            metadata: HashMap::new(),
        }),
    });
}

fn monitor_events(mut events: EventReader<EventNotification>) {
    for event in events.read() {
        match &event.event {
            DomainEvent::Node(_) => {
                println!("✓ Node event: {:?}", event.event.event_type());
            }
            DomainEvent::Edge(_) => {
                println!("✓ Edge event: {:?}", event.event.event_type());
            }
            DomainEvent::Graph(_) => {
                println!("✓ Graph event: {:?}", event.event.event_type());
            }
            _ => {}
        }
    }
}

fn count_nodes(
    keyboard: Res<ButtonInput<KeyCode>>,
    nodes: Query<(Entity, &GraphNode, &Transform, &Visibility)>,
) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        println!("\n=== NODE COUNT ===");
        let total = nodes.iter().count();
        let visible = nodes
            .iter()
            .filter(|(_, _, transform, vis)| {
                transform.scale.length() > 0.01 && matches!(vis, Visibility::Visible)
            })
            .count();

        println!("Total nodes: {}", total);
        println!("Visible nodes: {}", visible);

        for (entity, node, transform, vis) in nodes.iter() {
            println!("\nEntity {:?}:", entity);
            println!("  Node ID: {:?}", node.node_id);
            println!(
                "  Position: ({:.2}, {:.2}, {:.2})",
                transform.translation.x, transform.translation.y, transform.translation.z
            );
            println!("  Scale: {:.2}", transform.scale.x);
            println!("  Visibility: {:?}", vis);
        }
    }
}
