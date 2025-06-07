//! Integration test that would have caught the missing import functionality

use bevy::prelude::*;
use ia::application::{CommandEvent, EventNotification};
use ia::domain::{
    commands::{Command, GraphCommand, ImportSource},
    events::{DomainEvent, GraphEvent},
    value_objects::GraphId,
};
use ia::presentation::plugins::GraphEditorPlugin;

#[test]
fn test_import_command_actually_imports() {
    // This test would have caught that ImportGraph commands were not being processed
    let mut app = App::new();

    // Add minimal plugins and our plugin
    app.add_plugins(MinimalPlugins);
    app.add_plugins(GraphEditorPlugin);

    // Send an import command
    app.world_mut().send_event(CommandEvent {
        command: Command::Graph(GraphCommand::ImportGraph {
            graph_id: GraphId::new(),
            source: ImportSource::Content {
                content: r#"{
                    "nodes": [{
                        "id": "test-node",
                        "position": {"x": 0, "y": 0, "z": 0},
                        "caption": "Test Node"
                    }],
                    "relationships": []
                }"#.to_string(),
                format: ia::domain::commands::ImportFormat::ArrowsApp,
            },
            options: ia::domain::commands::ImportOptions {
                merge_behavior: ia::domain::commands::MergeBehavior::Replace,
                id_prefix: None,
                position_offset: None,
                layout_algorithm: None,
            },
        }),
    });

    // Update the app to process the command
    app.update();

    // Check that a GraphImportRequested event was generated
    let mut import_requested = false;
    let events = app.world().resource::<Events<EventNotification>>();
    let mut reader = events.get_reader();

    for event in reader.read(events) {
        if let DomainEvent::Graph(GraphEvent::GraphImportRequested { .. }) = &event.event {
            import_requested = true;
            break;
        }
    }

    assert!(import_requested, "ImportGraph command should generate GraphImportRequested event");

    // Update again to process the import request
    app.update();

    // Check that nodes were actually created
    let node_query = app.world().query::<&ia::presentation::components::NodeEntity>();
    let node_count = node_query.iter(app.world()).count();

    assert!(node_count > 0, "Import should create at least one node entity");
}

#[test]
fn test_import_events_are_processed() {
    // This test verifies that GraphImportRequested events are actually processed
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(GraphEditorPlugin);

    // Directly send a GraphImportRequested event
    app.world_mut().send_event(EventNotification {
        event: DomainEvent::Graph(GraphEvent::GraphImportRequested {
            graph_id: GraphId::new(),
            source: ia::domain::events::ImportSource::Content {
                content: r#"{
                    "nodes": [{
                        "id": "direct-test",
                        "position": {"x": 10, "y": 20, "z": 0},
                        "caption": "Direct Test"
                    }],
                    "relationships": []
                }"#.to_string(),
                format: ia::domain::events::ImportFormat::ArrowsApp,
            },
            options: ia::domain::events::ImportOptions {
                merge_behavior: ia::domain::events::MergeBehavior::Replace,
                id_prefix: None,
                position_offset: None,
                layout_algorithm: None,
            },
        }),
    });

    // Process the event
    app.update();
    app.update(); // Second update to ensure all systems run

    // Check that import completed event was generated
    let mut import_completed = false;
    let events = app.world().resource::<Events<EventNotification>>();
    let mut reader = events.get_reader();

    for event in reader.read(events) {
        if let DomainEvent::Graph(GraphEvent::GraphImportCompleted { .. }) = &event.event {
            import_completed = true;
            break;
        }
    }

    assert!(import_completed, "GraphImportRequested should generate GraphImportCompleted event");
}

#[test]
fn test_keyboard_shortcut_triggers_import() {
    // This test would verify that keyboard shortcuts actually trigger imports
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(GraphEditorPlugin);

    // Simulate Ctrl+M press (Mermaid import)
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ControlLeft);
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyM);

    // Update to process input
    app.update();

    // Check that a command was generated
    let mut command_sent = false;
    let events = app.world().resource::<Events<CommandEvent>>();
    let mut reader = events.get_reader();

    for event in reader.read(events) {
        if let Command::Graph(GraphCommand::ImportGraph { .. }) = &event.command {
            command_sent = true;
            break;
        }
    }

    assert!(command_sent, "Keyboard shortcut should trigger import command");
}
