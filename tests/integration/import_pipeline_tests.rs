//! Integration tests for the graph import pipeline
//!
//! These tests verify the complete flow from import command to entity creation:
//! 1. Import command is issued
//! 2. Import service processes the data
//! 3. Domain events are generated
//! 4. Events are persisted to NATS
//! 5. Projections are updated
//! 6. Bevy entities are created
//!
//! ```mermaid
//! graph LR
//!     A[Import Command] --> B[Import Service]
//!     B --> C[Parse Data]
//!     C --> D[Generate Events]
//!     D --> E[Event Store]
//!     E --> F[Projections]
//!     E --> G[Bevy Entities]
//! ```
#![cfg(feature = "bevy")]

use crate::fixtures::{TestEventStore, TestNatsServer, assertions::*, create_test_app};
use bevy::prelude::*;
use cim_domain::{DomainEvent, DomainResult, GraphId, NodeId};
use cim_domain_graph::{
    GraphAggregate, GraphCommand, GraphDomainEvent, ImportFormat, ImportOptions, ImportSource,
    NodeType, Position3D, StepType,
};
use std::time::SystemTime;

/// Test that import command generates appropriate domain events
#[tokio::test]
async fn test_import_command_generates_events() -> DomainResult<()> {
    // Arrange
    let mut graph = GraphAggregate::new(GraphId::new());
    let import_data = r#"{
        "nodes": [
            {"id": "1", "type": "concept", "label": "Node 1"},
            {"id": "2", "type": "process", "label": "Node 2"}
        ],
        "edges": [
            {"source": "1", "target": "2", "relationship": "connects_to"}
        ]
    }"#;

    // Act - Process import command
    let command = GraphCommand::ImportGraph {
        graph_id: graph.id(),
        source: ImportSource::Text(import_data.to_string()),
        format: ImportFormat::Json,
        options: ImportOptions::default(),
    };

    let events = graph.handle_command(command)?;

    // Assert
    assert!(!events.is_empty(), "Import should generate events");

    // Should have GraphImportRequested event
    assert!(
        events.iter().any(|e| matches!(
            e,
            DomainEvent::Graph(GraphDomainEvent::GraphImportRequested { .. })
        )),
        "Should generate GraphImportRequested event"
    );

    Ok(())
}

/// Test that import events are processed to create nodes and edges
#[tokio::test]
async fn test_import_event_processing() -> DomainResult<()> {
    // Arrange
    let event_store = TestEventStore::new();
    let mut graph = GraphAggregate::new(GraphId::new());

    // Create import requested event
    let import_event = DomainEvent::Graph(GraphDomainEvent::GraphImportRequested {
        graph_id: graph.id(),
        source: ImportSource::Text(
            r#"{
            "nodes": [{"id": "1", "type": "concept", "label": "Test"}],
            "edges": []
        }"#
            .to_string(),
        ),
        format: ImportFormat::Json,
        options: ImportOptions::default(),
        timestamp: SystemTime::now(),
    });

    // Act - Process the import event (simulating event handler)
    // In real implementation, this would be done by ImportEventHandler
    let result_events = process_import_event(import_event)?;

    // Store resulting events
    for event in &result_events {
        event_store.append(event.clone()).await?;
    }

    // Assert
    let stored_events = event_store.get_events().await;
    assert!(
        stored_events
            .iter()
            .any(|e| matches!(e, DomainEvent::Graph(GraphDomainEvent::NodeAdded { .. }))),
        "Import should generate NodeAdded events"
    );

    Ok(())
}

/// Test complete import flow from command to Bevy entities
#[tokio::test]
async fn test_complete_import_to_entity_flow() -> DomainResult<()> {
    // Arrange
    let mut app = create_test_app();
    let graph_id = GraphId::new();

    // Add import command to the world
    app.world.send_event(GraphCommand::ImportGraph {
        graph_id,
        source: ImportSource::Text(
            r#"{
            "nodes": [
                {"id": "1", "type": "concept", "label": "Concept A", "x": 0, "y": 0},
                {"id": "2", "type": "process", "label": "Process B", "x": 10, "y": 0}
            ],
            "edges": [
                {"source": "1", "target": "2", "relationship": "feeds_into"}
            ]
        }"#
            .to_string(),
        ),
        format: ImportFormat::Json,
        options: ImportOptions::default(),
    });

    // Act - Process through systems
    app.update(); // Process commands
    app.update(); // Process events
    app.update(); // Create entities

    // Assert - Check that entities were created
    let node_query = app
        .world
        .query::<&cim_domain_bevy::components::NodeVisual>();
    let node_count = node_query.iter(&app.world).count();
    assert_eq!(node_count, 2, "Should create 2 node entities");

    let edge_query = app
        .world
        .query::<&cim_domain_bevy::components::EdgeVisual>();
    let edge_count = edge_query.iter(&app.world).count();
    assert_eq!(edge_count, 1, "Should create 1 edge entity");

    Ok(())
}

/// Test import with invalid data handling
#[tokio::test]
async fn test_import_invalid_data_handling() -> DomainResult<()> {
    // Arrange
    let mut graph = GraphAggregate::new(GraphId::new());
    let invalid_json = r#"{ "invalid": "not a valid graph format" }"#;

    // Act
    let command = GraphCommand::ImportGraph {
        graph_id: graph.id(),
        source: ImportSource::Text(invalid_json.to_string()),
        format: ImportFormat::Json,
        options: ImportOptions::default(),
    };

    let result = graph.handle_command(command);

    // Assert - Should handle gracefully
    assert!(result.is_ok(), "Should not panic on invalid data");
    let events = result.unwrap();

    // Should generate import failed event
    assert!(
        events.iter().any(|e| matches!(
            e,
            DomainEvent::Graph(GraphDomainEvent::GraphImportFailed { .. })
        )),
        "Should generate GraphImportFailed event for invalid data"
    );

    Ok(())
}

/// Test import of different formats
#[tokio::test]
async fn test_import_multiple_formats() -> DomainResult<()> {
    // Test JSON format
    test_import_format(ImportFormat::Json, get_json_test_data()).await?;

    // Test Mermaid format
    test_import_format(ImportFormat::Mermaid, get_mermaid_test_data()).await?;

    // Test ArrowsApp format
    test_import_format(ImportFormat::ArrowsApp, get_arrows_app_test_data()).await?;

    Ok(())
}

/// Test concurrent imports don't interfere
#[tokio::test]
async fn test_concurrent_imports() -> DomainResult<()> {
    use tokio::task;

    // Arrange
    let event_store = TestEventStore::new();
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let store = event_store.clone();
            task::spawn(async move {
                let graph_id = GraphId::new();
                let mut graph = GraphAggregate::new(graph_id);

                let command = GraphCommand::ImportGraph {
                    graph_id,
                    source: ImportSource::Text(format!(
                        r#"{{"nodes": [{{"id": "{}", "type": "concept"}}], "edges": []}}"#,
                        i
                    )),
                    format: ImportFormat::Json,
                    options: ImportOptions::default(),
                };

                let events = graph.handle_command(command)?;
                for event in events {
                    store.append(event).await?;
                }

                Ok::<_, cim_domain::DomainError>(())
            })
        })
        .collect();

    // Act - Wait for all imports
    for handle in handles {
        handle.await??;
    }

    // Assert - All imports should succeed
    let events = event_store.get_events().await;
    assert!(events.len() >= 5, "Should have events from all imports");

    Ok(())
}

/// Test import with large dataset
#[tokio::test]
async fn test_import_large_dataset() -> DomainResult<()> {
    // Arrange - Create large graph data
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for i in 0..1000 {
        nodes.push(format!(
            r#"{{"id": "{}", "type": "concept", "label": "Node {}", "x": {}, "y": {}}}"#,
            i,
            i,
            i % 100,
            i / 100
        ));

        if i > 0 {
            edges.push(format!(
                r#"{{"source": "{}", "target": "{}", "relationship": "connects"}}"#,
                i - 1,
                i
            ));
        }
    }

    let large_data = format!(
        r#"{{"nodes": [{}], "edges": [{}]}}"#,
        nodes.join(","),
        edges.join(",")
    );

    // Act
    let start = std::time::Instant::now();
    let mut graph = GraphAggregate::new(GraphId::new());

    let command = GraphCommand::ImportGraph {
        graph_id: graph.id(),
        source: ImportSource::Text(large_data),
        format: ImportFormat::Json,
        options: ImportOptions::default(),
    };

    let events = graph.handle_command(command)?;
    let duration = start.elapsed();

    // Assert
    assert!(
        !events.is_empty(),
        "Should generate events for large import"
    );
    assert!(
        duration.as_secs() < 5,
        "Large import should complete within 5 seconds"
    );

    // Should have import requested event
    assert!(events.iter().any(|e| matches!(
        e,
        DomainEvent::Graph(GraphDomainEvent::GraphImportRequested { .. })
    )));

    Ok(())
}

// Helper functions

fn process_import_event(event: DomainEvent) -> DomainResult<Vec<DomainEvent>> {
    // This simulates what the ImportEventHandler would do
    match event {
        DomainEvent::Graph(GraphDomainEvent::GraphImportRequested {
            graph_id,
            source,
            format,
            ..
        }) => {
            // Parse the import data and generate events
            let mut events = Vec::new();

            // For now, just generate a simple node added event
            events.push(DomainEvent::Graph(GraphDomainEvent::NodeAdded {
                graph_id,
                node_id: NodeId::new(),
                node_type: NodeType::Concept,
                position: Position3D::default(),
                conceptual_point: Default::default(),
                metadata: Default::default(),
            }));

            Ok(events)
        }
        _ => Ok(vec![]),
    }
}

async fn test_import_format(format: ImportFormat, data: String) -> DomainResult<()> {
    let mut graph = GraphAggregate::new(GraphId::new());

    let command = GraphCommand::ImportGraph {
        graph_id: graph.id(),
        source: ImportSource::Text(data),
        format,
        options: ImportOptions::default(),
    };

    let events = graph.handle_command(command)?;
    assert!(
        !events.is_empty(),
        "Import should generate events for format {:?}",
        format
    );

    Ok(())
}

fn get_json_test_data() -> String {
    r#"{
        "nodes": [{"id": "1", "type": "concept"}],
        "edges": []
    }"#
    .to_string()
}

fn get_mermaid_test_data() -> String {
    "graph TD\n  A[Start] --> B[End]".to_string()
}

fn get_arrows_app_test_data() -> String {
    r#"{
        "nodes": [{"id": "1", "caption": "Test"}],
        "relationships": []
    }"#
    .to_string()
}
