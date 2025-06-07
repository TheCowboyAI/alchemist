//! Integration tests for graph import functionality

use bevy::prelude::*;
use ia::application::{CommandEvent, EventNotification};
use ia::domain::{
    commands::{Command, GraphCommand, ImportSource, ImportOptions},
    commands::graph_commands::MergeBehavior,
    events::{DomainEvent, GraphEvent, NodeEvent},
    value_objects::{GraphId, NodeId},
};
use ia::presentation::plugins::GraphPlugin;
use ia::presentation::systems::process_graph_import_requests;

#[test]
fn test_import_graph_full_flow() {
    // Create a Bevy app with necessary plugins and systems
    let mut app = App::new();

    // Add minimal plugins needed for testing
    app.add_plugins(MinimalPlugins);
    app.add_event::<CommandEvent>();
    app.add_event::<EventNotification>();

    // Add the systems we need
    app.add_systems(Update, (
        ia::application::command_handlers::process_commands,
        process_graph_import_requests,
    ).chain());

    // Create test content
    let test_content = r#"{
        "nodes": [{
            "id": "node1",
            "position": {"x": 0, "y": 0, "z": 0},
            "caption": "Test Node",
            "labels": ["SIMPLE"],
            "properties": {"key": "value"},
            "style": {}
        }],
        "relationships": []
    }"#;

    // Send an ImportGraph command
    let graph_id = GraphId::new();
    app.world_mut().send_event(CommandEvent {
        command: Command::Graph(GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::InlineContent {
                content: test_content.to_string(),
            },
            format: "arrows_app".to_string(),
            options: ImportOptions {
                merge_behavior: MergeBehavior::AlwaysCreate,
                id_prefix: None,
                position_offset: None,
                mapping: None,
                validate: true,
                max_nodes: None,
            },
        }),
    });

    // Run the app to process the command
    app.update();

    // Check that events were generated
    let events = app.world().resource::<Events<EventNotification>>();
    let mut reader = events.get_cursor();
    let event_count = reader.read(events).count();

    // We expect at least:
    // 1. GraphImportRequested event (from command handler)
    // 2. NodeAdded event (from import processor)
    // 3. GraphImportCompleted event (from import processor)
    assert!(event_count >= 3, "Expected at least 3 events, got {}", event_count);
}

#[test]
fn test_import_creates_entities() {
    // This test would verify that entities are created in the ECS
    // However, since the current implementation only generates events
    // and doesn't create entities directly, this test documents
    // the missing functionality

    // TODO: The presentation layer should have systems that:
    // 1. Listen for NodeAdded events
    // 2. Create Bevy entities with appropriate components
    // 3. Listen for EdgeConnected events
    // 4. Create edge entities

    panic!("Entity creation from import events is not implemented!");
}

#[test]
fn test_import_with_edges() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<CommandEvent>();
    app.add_event::<EventNotification>();

    app.add_systems(Update, (
        ia::application::command_handlers::process_commands,
        process_graph_import_requests,
    ).chain());

    // Create test content with edges
    let test_content = r#"{
        "nodes": [
            {
                "id": "node1",
                "position": {"x": 0, "y": 0, "z": 0},
                "caption": "Node 1",
                "labels": ["SIMPLE"],
                "properties": {},
                "style": {}
            },
            {
                "id": "node2",
                "position": {"x": 100, "y": 0, "z": 0},
                "caption": "Node 2",
                "labels": ["SIMPLE"],
                "properties": {},
                "style": {}
            }
        ],
        "relationships": [{
            "id": "edge1",
            "fromId": "node1",
            "toId": "node2",
            "type": "CONNECTS",
            "properties": {},
            "style": {}
        }]
    }"#;

    let graph_id = GraphId::new();
    app.world_mut().send_event(CommandEvent {
        command: Command::Graph(GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::InlineContent {
                content: test_content.to_string(),
            },
            format: "arrows_app".to_string(),
            options: ImportOptions {
                merge_behavior: MergeBehavior::AlwaysCreate,
                id_prefix: None,
                position_offset: None,
                mapping: None,
                validate: true,
                max_nodes: None,
            },
        }),
    });

    app.update();

    // Verify events were generated
    let events = app.world().resource::<Events<EventNotification>>();
    let mut reader = events.get_cursor();
    let generated_events: Vec<_> = reader.read(events).collect();

    // Should have:
    // 1. GraphImportRequested
    // 2. NodeAdded for node1
    // 3. NodeAdded for node2
    // 4. EdgeConnected for edge1
    // 5. GraphImportCompleted
    assert!(generated_events.len() >= 5, "Expected at least 5 events, got {}", generated_events.len());

    // Count event types
    let node_added_count = generated_events.iter()
        .filter(|e| matches!(&e.event, DomainEvent::Node(NodeEvent::NodeAdded { .. })))
        .count();
    assert_eq!(node_added_count, 2, "Expected 2 NodeAdded events");

    let edge_connected_count = generated_events.iter()
        .filter(|e| matches!(&e.event, DomainEvent::Edge(_)))
        .count();
    assert_eq!(edge_connected_count, 1, "Expected 1 EdgeConnected event");
}
