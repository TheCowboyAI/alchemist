//! Simple test to verify import-to-render pipeline

use bevy::prelude::*;
use ia::application::{CommandEvent, EventNotification};
use ia::domain::{
    commands::{Command, GraphCommand, ImportSource, ImportOptions},
    events::{DomainEvent, GraphEvent},
    value_objects::GraphId,
};
use ia::presentation::{
    components::GraphNode,
    events::ImportRequestEvent,
    systems::{forward_import_requests, process_graph_import_requests},
};

#[test]
fn test_import_creates_graph_nodes() {
    // Set headless mode
    unsafe {
        std::env::set_var("BEVY_HEADLESS", "1");
    }

    // Create minimal app
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Add events
    app.add_event::<CommandEvent>();
    app.add_event::<EventNotification>();
    app.add_event::<ImportRequestEvent>();

    // Add only the systems we need to test
    app.add_systems(Update, (
        // Process commands to generate GraphImportRequested events
        ia::application::command_handlers::process_commands,
        // Forward import requests
        forward_import_requests,
        // Process imports
        process_graph_import_requests,
    ).chain());

    // Create simple test JSON
    let test_json = r#"{
        "nodes": [{
            "id": "n1",
            "position": {"x": 0, "y": 0},
            "caption": "Test Node",
            "labels": ["Test"],
            "properties": {}
        }],
        "relationships": []
    }"#;

    // Send import command
    let graph_id = GraphId::new();
    app.world_mut().send_event(CommandEvent {
        command: Command::Graph(GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::InlineContent {
                content: test_json.to_string(),
            },
            format: "arrows_app".to_string(),
            options: ImportOptions::default(),
        }),
    });

    // Process the command
    app.update();

    // Check if GraphImportRequested event was generated
    let import_requested_events: Vec<_> = app.world()
        .resource::<Events<EventNotification>>()
        .iter_current_update_events()
        .filter(|e| matches!(&e.event, DomainEvent::Graph(GraphEvent::GraphImportRequested { .. })))
        .collect();

    assert_eq!(import_requested_events.len(), 1,
        "Should have generated 1 GraphImportRequested event, but found {}",
        import_requested_events.len());

    // Process another update to handle the import
    app.update();

    // Check if any events were generated from the import
    let all_events: Vec<_> = app.world()
        .resource::<Events<EventNotification>>()
        .iter_current_update_events()
        .collect();

    println!("Events generated after import: {}", all_events.len());
    for event in &all_events {
        println!("  Event: {:?}", event.event.event_type());
    }

    // The import processor should have generated NodeAdded events
    let node_added_events: Vec<_> = all_events.iter()
        .filter(|e| matches!(&e.event, DomainEvent::Node(_)))
        .collect();

    assert!(node_added_events.len() > 0,
        "Import should have generated NodeAdded events, but found 0");
}
