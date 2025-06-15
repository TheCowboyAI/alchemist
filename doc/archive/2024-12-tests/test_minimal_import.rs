//! Minimal test to debug import pipeline

use bevy::prelude::*;
use ia::application::{CommandEvent, EventNotification};
use ia::domain::{
    commands::{Command, GraphCommand, ImportOptions, ImportSource},
    events::{DomainEvent, GraphEvent},
    value_objects::GraphId,
};
use ia::presentation::events::{ImportRequestEvent, ImportResultEvent};
use ia::presentation::systems::{
    forward_import_requests, forward_import_results, process_graph_import_requests,
};
use std::collections::HashMap;

fn main() {
    println!("Starting minimal import test");

    App::new()
        .add_plugins(MinimalPlugins)
        .add_event::<CommandEvent>()
        .add_event::<EventNotification>()
        .add_event::<ImportRequestEvent>()
        .add_event::<ImportResultEvent>()
        .add_systems(Startup, send_test_command)
        .add_systems(
            Update,
            (
                log_commands,
                process_test_commands,
                log_event_notifications,
                forward_import_requests,
                log_import_requests,
                process_graph_import_requests,
                log_import_results,
                forward_import_results,
                log_final_events,
            )
                .chain(),
        )
        .run();
}

fn send_test_command(mut commands: EventWriter<CommandEvent>) {
    println!("Sending test import command");

    let graph_id = GraphId::new();

    // First create a graph
    commands.write(CommandEvent {
        command: Command::Graph(GraphCommand::CreateGraph {
            id: graph_id,
            name: "Test Graph".to_string(),
            metadata: HashMap::new(),
        }),
    });

    // Then send import command
    commands.write(CommandEvent {
        command: Command::Graph(GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::InlineContent {
                content: r#"{"nodes": [{"id": "1", "position": {"x": 0, "y": 0}, "style": {}, "labels": ["Node"], "properties": {}}], "relationships": []}"#.to_string(),
            },
            format: "arrows_app".to_string(),
            options: ImportOptions::default(),
        }),
    });

    println!("Commands sent!");
}

fn log_commands(mut events: EventReader<CommandEvent>) {
    for event in events.read() {
        println!(
            "1. LOG_COMMANDS: Received command: {:?}",
            event.command.command_type()
        );
    }
}

fn process_test_commands(
    mut commands: EventReader<CommandEvent>,
    mut events: EventWriter<EventNotification>,
) {
    for cmd in commands.read() {
        println!(
            "2. PROCESS_COMMANDS: Processing command: {:?}",
            cmd.command.command_type()
        );

        match &cmd.command {
            Command::Graph(GraphCommand::ImportGraph {
                graph_id,
                source,
                format,
                options,
            }) => {
                println!("2. PROCESS_COMMANDS: Creating GraphImportRequested event");
                events.write(EventNotification {
                    event: DomainEvent::Graph(GraphEvent::GraphImportRequested {
                        graph_id: *graph_id,
                        source: source.clone(),
                        format: format.clone(),
                        options: options.clone(),
                    }),
                });
            }
            Command::Graph(GraphCommand::CreateGraph { id, .. }) => {
                println!("2. PROCESS_COMMANDS: Creating GraphCreated event");
                events.write(EventNotification {
                    event: DomainEvent::Graph(GraphEvent::GraphCreated {
                        id: *id,
                        metadata: Default::default(),
                    }),
                });
            }
            _ => {}
        }
    }
}

fn log_event_notifications(mut events: EventReader<EventNotification>) {
    for event in events.read() {
        println!(
            "3. LOG_EVENT_NOTIFICATIONS: Received event: {:?}",
            event.event.event_type()
        );
    }
}

fn log_import_requests(mut events: EventReader<ImportRequestEvent>) {
    for event in events.read() {
        println!(
            "5. LOG_IMPORT_REQUESTS: Received import request: {:?}",
            event.event.event_type()
        );
    }
}

fn log_import_results(mut events: EventReader<ImportResultEvent>) {
    for event in events.read() {
        println!(
            "7. LOG_IMPORT_RESULTS: Received import result: {:?}",
            event.event.event_type()
        );
    }
}

fn log_final_events(mut events: EventReader<EventNotification>) {
    for event in events.read() {
        println!(
            "9. LOG_FINAL_EVENTS: Final event: {:?}",
            event.event.event_type()
        );
    }
}
