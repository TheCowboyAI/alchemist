//! Integration tests for graph import functionality
//! These tests verify the complete flow from import command to rendered entities

use bevy::prelude::*;
use ia::application::{CommandEvent, command_handlers};
use ia::domain::{
    commands::{Command, GraphCommand, ImportOptions, ImportSource},
    value_objects::{GraphId, NodeId},
};
use ia::presentation::components::{GraphEdge, GraphNode};
use std::time::Duration;

/// Test that import commands are properly processed and create visible entities
#[test]
fn test_import_graph_creates_entities() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_event::<CommandEvent>()
        .add_systems(Update, command_handlers::process_commands)
        .add_systems(Update, verify_import_creates_entities);

    // Act - Send import command
    let graph_id = GraphId::new();
    app.world.send_event(CommandEvent {
        command: Command::Graph(GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::File {
                path: "examples/data/sample_graph.json".to_string(),
            },
            format: "arrows_app".to_string(),
            options: ImportOptions::default(),
        }),
    });

    // Update the app to process the command
    app.update();

    // Allow time for async processing
    std::thread::sleep(Duration::from_millis(100));
    app.update();

    // Assert - Check that nodes were created
    let node_count = app.world.query::<&GraphNode>().iter(&app.world).count();

    assert!(
        node_count > 0,
        "Import command should create graph nodes, but found 0 nodes"
    );

    // Check that edges were created
    let edge_count = app.world.query::<&GraphEdge>().iter(&app.world).count();

    assert!(
        edge_count > 0,
        "Import command should create graph edges, but found 0 edges"
    );
}

/// Test that import commands emit proper events
#[test]
fn test_import_graph_emits_events() {
    // This test should verify that GraphImportRequested events are properly handled
    // and result in NodeAdded/EdgeAdded events

    // TODO: Implement when event handling is properly wired up
    panic!(
        "Import event handling not implemented - GraphImportRequested events are not processed!"
    );
}

/// Test that imported nodes have proper visual components
#[test]
fn test_imported_nodes_have_visual_components() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::asset::AssetPlugin::default())
        .add_plugins(bevy::render::RenderPlugin::default())
        .add_event::<CommandEvent>()
        .add_systems(Update, command_handlers::process_commands);

    // Send import command
    let graph_id = GraphId::new();
    app.world.send_event(CommandEvent {
        command: Command::Graph(GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::File {
                path: "examples/data/sample_graph.json".to_string(),
            },
            format: "arrows_app".to_string(),
            options: ImportOptions::default(),
        }),
    });

    app.update();
    std::thread::sleep(Duration::from_millis(100));
    app.update();

    // Check that nodes have required visual components
    let visual_nodes = app
        .world
        .query::<(&GraphNode, &Transform)>()
        .iter(&app.world)
        .count();

    assert!(
        visual_nodes > 0,
        "Imported nodes should have visual components (Transform, Mesh3d), but found 0 complete nodes"
    );
}

/// Helper system to verify import results
fn verify_import_creates_entities(nodes: Query<&GraphNode>, edges: Query<&GraphEdge>) {
    let node_count = nodes.iter().count();
    let edge_count = edges.iter().count();

    if node_count > 0 || edge_count > 0 {
        println!(
            "Import created {} nodes and {} edges",
            node_count, edge_count
        );
    }
}

#[cfg(test)]
mod import_service_tests {
    use super::*;
    use ia::domain::services::graph_import::{GraphImportService, ImportFormat};

    #[test]
    fn test_import_service_integration() {
        // Test that the import service correctly parses and returns data
        let service = GraphImportService::new();
        let json = r#"{
            "nodes": [{
                "id": "n1",
                "position": {"x": 0, "y": 0},
                "caption": "Test Node",
                "labels": ["TestType"],
                "properties": {"name": "Test"}
            }],
            "relationships": []
        }"#;

        let result = service.import_from_json(json, ImportFormat::ArrowsApp);
        assert!(result.is_ok(), "Import service should parse valid JSON");

        let graph = result.unwrap();
        assert_eq!(graph.nodes.len(), 1, "Should import one node");
        assert_eq!(graph.nodes[0].label, "Test Node", "Node label should match");
    }

    #[test]
    fn test_import_command_handler_processes_import() {
        // This test should verify the command handler actually processes ImportGraph commands
        // Currently it just returns None for ImportGraph!

        panic!(
            "ImportGraph command handler not implemented - returns None instead of processing import!"
        );
    }
}
