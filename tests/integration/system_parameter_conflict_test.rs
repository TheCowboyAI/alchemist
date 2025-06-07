//! Tests for system parameter conflicts in the import flow

use bevy::prelude::*;
use ia::application::{CommandEvent, EventNotification};
use ia::presentation::events::{ImportResultEvent, ImportRequestEvent};
use ia::presentation::systems::{
    forward_import_requests,
    process_graph_import_requests,
    forward_import_results,
};
use ia::presentation::plugins::GraphEditorPlugin;

#[test]
fn test_no_system_parameter_conflicts() {
    // This test verifies that all systems can run without parameter conflicts
    let mut app = App::new();

    // Add minimal plugins
    app.add_plugins(MinimalPlugins);

    // Add all the events that the real app uses
    app.add_event::<CommandEvent>();
    app.add_event::<EventNotification>();
    app.add_event::<ImportResultEvent>();
    app.add_event::<ImportRequestEvent>();

    // Add all the systems that could conflict
    app.add_systems(Update, (
        // Systems that read EventNotification
        ia::application::command_handlers::process_commands,
        forward_import_requests,

        // Systems that process import requests
        process_graph_import_requests,

        // Systems that forward results
        forward_import_results,
    ));

    // This should not panic with system parameter conflicts
    app.update();
}

#[test]
fn test_full_plugin_no_conflicts() {
    // Test the full plugin setup
    let mut app = App::new();

    // Add minimal plugins needed
    app.add_plugins(MinimalPlugins);

    // Add the full GraphEditorPlugin
    app.add_plugins(GraphEditorPlugin);

    // This should not panic with system parameter conflicts
    app.update();
}

#[test]
fn test_event_flow_with_all_systems() {
    // Test that events flow correctly through all systems
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_event::<CommandEvent>();
    app.add_event::<EventNotification>();
    app.add_event::<ImportResultEvent>();
    app.add_event::<ImportRequestEvent>();

    // Add systems in the correct order
    app.add_systems(Update, (
        ia::application::command_handlers::process_commands,
        forward_import_requests,
        process_graph_import_requests,
        forward_import_results,
    ).chain());

    // Send a command
    app.world_mut().send_event(CommandEvent {
        command: ia::domain::commands::Command::Graph(
            ia::domain::commands::GraphCommand::ImportGraph {
                graph_id: ia::domain::value_objects::GraphId::new(),
                source: ia::domain::commands::ImportSource::InlineContent {
                    content: r#"{"nodes": [], "relationships": []}"#.to_string(),
                },
                format: "arrows_app".to_string(),
                options: Default::default(),
            }
        ),
    });

    // Update should not panic
    app.update();

    // Verify events were forwarded
    let import_requests = app.world().resource::<Events<ImportRequestEvent>>();
    let mut reader = import_requests.get_cursor();
    let request_count = reader.read(import_requests).count();
    assert!(request_count > 0, "ImportRequestEvent should have been forwarded");
}

#[test]
fn test_system_ordering_prevents_conflicts() {
    // Test that proper system ordering prevents conflicts
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_event::<EventNotification>();
    app.add_event::<ImportRequestEvent>();

    // These systems would conflict if run in parallel
    // but should work when properly ordered
    app.add_systems(Update, (
        // First: Systems that write to EventNotification
        |mut writer: EventWriter<EventNotification>| {
            writer.send(EventNotification {
                event: ia::domain::events::DomainEvent::Graph(
                    ia::domain::events::GraphEvent::GraphCreated {
                        graph_id: ia::domain::value_objects::GraphId::new(),
                        name: "Test".to_string(),
                        metadata: Default::default(),
                    }
                ),
            });
        },
        // Then: Systems that read from EventNotification
        forward_import_requests,
        // Finally: Systems that process forwarded events
        process_graph_import_requests,
    ).chain());

    // Should not panic
    app.update();
}
