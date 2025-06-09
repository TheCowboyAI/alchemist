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

#[test]
fn test_complete_import_to_render_pipeline() {
    // User Story: US9 - Import/Export
    // Acceptance Criteria: Imported graphs should be fully rendered with visual entities
    // Test Purpose: Validates the complete pipeline from import command to rendered entities
    // Expected Behavior: Import command → events → Bevy entities with all required components

    use bevy::app::App;
    use ia::presentation::plugins::GraphEditorPlugin;
    use ia::presentation::components::{GraphNode, GraphEdge, NodeLabel};
    use ia::application::CommandEvent;
    use ia::domain::commands::{Command, GraphCommand, ImportSource, ImportOptions};
    use ia::domain::value_objects::GraphId;
    use bevy::prelude::*;

    // Set headless mode
    unsafe {
        std::env::set_var("BEVY_HEADLESS", "1");
    }

    // Given - A full Bevy app with all systems
    let mut app = App::new();

    // Add minimal plugins needed for rendering
    app.add_plugins((
        bevy::MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
    ));

    // Add our graph editor plugin
    app.add_plugins(GraphEditorPlugin);

    // Add command event
    app.add_event::<CommandEvent>();

    // Create a test graph JSON
    let test_json = r#"{
        "nodes": [
            {
                "id": "n1",
                "position": {"x": 0, "y": 0},
                "caption": "Node One",
                "labels": ["TestNode"],
                "properties": {"test": "value1"}
            },
            {
                "id": "n2",
                "position": {"x": 100, "y": 100},
                "caption": "Node Two",
                "labels": ["TestNode"],
                "properties": {"test": "value2"}
            }
        ],
        "relationships": [
            {
                "id": "r1",
                "fromId": "n1",
                "toId": "n2",
                "type": "CONNECTS",
                "properties": {}
            }
        ]
    }"#;

    // When - Send import command
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

    // Process multiple updates to allow event propagation
    for _ in 0..5 {
        app.update();
    }

    // Then - Verify nodes were created with all required components
    let nodes_with_components: Vec<_> = app.world_mut()
        .query::<(&GraphNode, &Transform, &NodeLabel)>()
        .iter(&app.world())
        .collect();

    assert_eq!(nodes_with_components.len(), 2,
        "Should have created 2 nodes with all visual components, but found {}",
        nodes_with_components.len());

    // Verify node labels
    let labels: Vec<String> = nodes_with_components.iter()
        .map(|(_, _, label)| label.text.clone())
        .collect();

    assert!(labels.contains(&"Node One".to_string()), "Should have Node One");
    assert!(labels.contains(&"Node Two".to_string()), "Should have Node Two");

    // Verify edges were created
    let edges: Vec<_> = app.world_mut()
        .query::<(&GraphEdge, &Transform)>()
        .iter(&app.world())
        .collect();

    assert_eq!(edges.len(), 1,
        "Should have created 1 edge with visual components, but found {}",
        edges.len());

    // Verify the edge connects the right nodes
    let (edge, _) = &edges[0];

    // Collect node entities first to avoid borrow issues
    let node_entities: Vec<(Entity, NodeId)> = app.world()
        .query::<(Entity, &GraphNode)>()
        .iter(&app.world())
        .map(|(e, node)| (e, node.node_id))
        .collect();

    assert_eq!(node_entities.len(), 2, "Should have 2 node entities");

    // The edge should reference valid node entities
    let source_exists = node_entities.iter().any(|(e, _)| *e == edge.source);
    let target_exists = node_entities.iter().any(|(e, _)| *e == edge.target);

    assert!(source_exists, "Edge source should reference a valid node entity");
    assert!(target_exists, "Edge target should reference a valid node entity");
}

#[test]
fn test_import_mermaid_to_render_pipeline() {
    // User Story: US9 - Import/Export
    // Acceptance Criteria: Mermaid diagrams should be imported and rendered
    // Test Purpose: Validates Mermaid import through the complete pipeline
    // Expected Behavior: Mermaid text → parsed → events → rendered entities

    use bevy::app::App;
    use ia::presentation::plugins::GraphEditorPlugin;
    use ia::presentation::components::GraphNode;
    use ia::application::CommandEvent;
    use ia::domain::commands::{Command, GraphCommand, ImportSource, ImportOptions};
    use ia::domain::value_objects::GraphId;
    use bevy::prelude::*;

    // Set headless mode
    unsafe {
        std::env::set_var("BEVY_HEADLESS", "1");
    }

    // Given - A full Bevy app
    let mut app = App::new();

    app.add_plugins((
        bevy::MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
    ));

    app.add_plugins(GraphEditorPlugin);
    app.add_event::<CommandEvent>();

    // Mermaid diagram
    let mermaid = r#"
graph TD
    A[Start] --> B{Decision}
    B --> C[Yes]
    B --> D[No]
"#;

    // When - Import Mermaid
    let graph_id = GraphId::new();
    app.world_mut().send_event(CommandEvent {
        command: Command::Graph(GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::InlineContent {
                content: mermaid.to_string(),
            },
            format: "mermaid".to_string(),
            options: ImportOptions::default(),
        }),
    });

    // Process updates
    for _ in 0..5 {
        app.update();
    }

    // Then - Verify rendering
    let nodes: Vec<_> = app.world_mut()
        .query::<&GraphNode>()
        .iter(&app.world())
        .collect();

    assert_eq!(nodes.len(), 4,
        "Mermaid diagram should create 4 nodes (A, B, C, D), but found {}",
        nodes.len());
}
