//! Debug import pipeline

use bevy::prelude::*;
use ia::application::{CommandEvent, EventNotification};
use ia::domain::{
    commands::{Command, GraphCommand, ImportSource, ImportOptions},
    events::{DomainEvent, GraphEvent},
    value_objects::GraphId,
};
use ia::presentation::{
    events::{ImportRequestEvent, ImportResultEvent},
    plugins::GraphEditorPlugin,
};
use std::collections::HashMap;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

fn main() {
    // Set up logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("Starting import debug application");

    let mut app = App::new();

    // Add default plugins
    app.add_plugins(DefaultPlugins);

    // Add our graph editor plugin
    app.add_plugins(GraphEditorPlugin);

    // Add debug systems
    app.add_systems(Update, (
        debug_command_events,
        debug_event_notifications,
        debug_import_requests,
        debug_import_results,
    ));

    // Add startup system to trigger import
    app.add_systems(Startup, trigger_test_import);

    app.run();
}

fn trigger_test_import(mut commands: EventWriter<CommandEvent>) {
    info!("=== TRIGGERING TEST IMPORT ===");

    let graph_id = GraphId::new();

    // First create a graph
    commands.write(CommandEvent {
        command: Command::Graph(GraphCommand::CreateGraph {
            id: graph_id,
            name: "Debug Import Graph".to_string(),
            metadata: HashMap::new(),
        }),
    });

    // Then trigger import
    commands.write(CommandEvent {
        command: Command::Graph(GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::File {
                path: "examples/data/sample_graph.json".to_string(),
            },
            format: "arrows_app".to_string(),
            options: ImportOptions::default(),
        }),
    });

    info!("Sent CreateGraph and ImportGraph commands for graph: {:?}", graph_id);
}

fn debug_command_events(mut events: EventReader<CommandEvent>) {
    for event in events.read() {
        info!("DEBUG: CommandEvent received: {:?}", event.command);
    }
}

fn debug_event_notifications(mut events: EventReader<EventNotification>) {
    for event in events.read() {
        info!("DEBUG: EventNotification received: {:?}", event.event.event_type());

        if let DomainEvent::Graph(GraphEvent::GraphImportRequested { .. }) = &event.event {
            info!("  -> This is a GraphImportRequested event!");
        }
    }
}

fn debug_import_requests(mut events: EventReader<ImportRequestEvent>) {
    for event in events.read() {
        info!("DEBUG: ImportRequestEvent received: {:?}", event.event.event_type());
    }
}

fn debug_import_results(mut events: EventReader<ImportResultEvent>) {
    for event in events.read() {
        info!("DEBUG: ImportResultEvent received: {:?}", event.event.event_type());
    }
}
