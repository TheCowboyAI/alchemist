//! End-to-End Integration Tests
//!
//! These tests verify the complete flow from Bevy commands through NATS
//! to event store and finally to projection updates.

use ia::domain::aggregates::GraphAggregate;
use ia::domain::commands::{Command, GraphCommand, NodeCommand, EdgeCommand};
use ia::domain::events::{DomainEvent, GraphEvent, NodeEvent, EdgeEvent};
use ia::domain::value_objects::{GraphId, NodeId, EdgeId, Position3D, GraphMetadata, EdgeRelationship};
use ia::infrastructure::event_store::{DistributedEventStore, EventStore};
use ia::infrastructure::event_bridge::{EventBridge, BridgeCommand};
use ia::infrastructure::nats::{NatsClient, NatsConfig};
use ia::application::command_handlers::{GraphCommandHandler, CommandHandler};
use ia::application::projections::{GraphSummaryProjection, Projection, ProjectionHandler};
use crate::fixtures::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use bevy::prelude::*;

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_complete_end_to_end_flow() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup infrastructure
    let (client, jetstream) = connect_test_nats().await?;
    let nats_client = Arc::new(NatsClient::new(client.clone(), Some(jetstream.clone())).await?);
    let event_store = Arc::new(DistributedEventStore::new(jetstream.clone()).await?);
    let event_bridge = EventBridge::new();

    // 2. Setup projections
    let graph_summary = Arc::new(RwLock::new(GraphSummaryProjection::new()));
    let mut projection_handler = ProjectionHandler::new(
        event_store.clone(),
        graph_summary.clone(),
    );

    // Start projection handler
    projection_handler.start(jetstream.clone()).await?;

    // 3. Start event bridge
    event_bridge.start(nats_client.clone()).await?;

        // Test data
    let graph_id = GraphId::new();
    let node1_id = NodeId::new();
    let node2_id = NodeId::new();
    let edge_id = EdgeId::new();

    // 4. Create graph using command handler
    let handler = GraphCommandHandler::new(event_store.clone());
    let create_graph_cmd = GraphCommand::CreateGraph {
        id: graph_id,
        name: "E2E Test Graph".to_string(),
    };

        handler.handle(Command::Graph(create_graph_cmd)).await?;

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(200)).await;

    // 5. Verify projection was updated
    {
        let projection = graph_summary.read().await;
        let summary = projection.get_summary(&graph_id)
            .expect("Graph should exist in projection");
        assert_eq!(summary.metadata.name, "E2E Test Graph");
        assert_eq!(summary.node_count, 0);
        assert_eq!(summary.edge_count, 0);
    }

    // 6. Add nodes through command handler (direct approach)

    let add_node1_cmd = GraphCommand::Node(NodeCommand::AddNode {
        graph_id,
        node_id: node1_id,
        content: "Node 1".to_string(),
        position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
        metadata: Default::default(),
    });

    let events = handler.handle(add_node1_cmd).await?;
    assert_eq!(events.len(), 1);

    let add_node2_cmd = GraphCommand::Node(NodeCommand::AddNode {
        graph_id,
        node_id: node2_id,
        content: "Node 2".to_string(),
        position: Position3D { x: 10.0, y: 0.0, z: 0.0 },
        metadata: Default::default(),
    });

    handler.handle(add_node2_cmd).await?;

    // Wait for projection updates
    tokio::time::sleep(Duration::from_millis(200)).await;

    // 7. Verify node count in projection
    {
        let projection = graph_summary.read().await;
        let summary = projection.get_summary(&graph_id).unwrap();
        assert_eq!(summary.node_count, 2);
        assert_eq!(summary.edge_count, 0);
    }

    // 8. Connect nodes with edge
    let connect_edge_cmd = GraphCommand::Edge(EdgeCommand::ConnectEdge {
        graph_id,
        edge_id,
        source: node1_id,
        target: node2_id,
        relationship: EdgeRelationship::new("connected"),
    });

    handler.handle(connect_edge_cmd).await?;

    // Wait for projection update
    tokio::time::sleep(Duration::from_millis(200)).await;

    // 9. Verify edge count in projection
    {
        let projection = graph_summary.read().await;
        let summary = projection.get_summary(&graph_id).unwrap();
        assert_eq!(summary.node_count, 2);
        assert_eq!(summary.edge_count, 1);
    }

    // 10. Test projection statistics
    {
        let projection = graph_summary.read().await;
        let (graph_count, total_nodes, total_edges) = projection.get_total_stats();
        assert_eq!(graph_count, 1);
        assert_eq!(total_nodes, 2);
        assert_eq!(total_edges, 1);

        // Test recent graphs
        let recent = projection.get_recent_graphs(10);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].id, graph_id);
    }

    // 11. Remove a node (should cascade delete edge)
    let remove_node_cmd = GraphCommand::Node(NodeCommand::RemoveNode {
        graph_id,
        node_id: node1_id,
    });

    let remove_events = handler.handle(remove_node_cmd).await?;

    // Should have both node and edge removal events
    assert!(remove_events.iter().any(|e| matches!(e, DomainEvent::Node(NodeEvent::NodeRemoved { .. }))));
    assert!(remove_events.iter().any(|e| matches!(e, DomainEvent::Edge(EdgeEvent::EdgeRemoved { .. }))));

    // Wait for projection update
    tokio::time::sleep(Duration::from_millis(200)).await;

    // 12. Verify final state
    {
        let projection = graph_summary.read().await;
        let summary = projection.get_summary(&graph_id).unwrap();
        assert_eq!(summary.node_count, 1);
        assert_eq!(summary.edge_count, 0);
    }

    // 13. Test projection replay
    let checkpoint = {
        let projection = graph_summary.read().await;
        projection.get_checkpoint().await.unwrap_or(0)
    };

    // Replay from beginning
    projection_handler.replay_from(1, jetstream.clone()).await?;

    // Wait for replay to complete
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify state after replay
    {
        let projection = graph_summary.read().await;
        let summary = projection.get_summary(&graph_id).unwrap();
        assert_eq!(summary.node_count, 1);
        assert_eq!(summary.edge_count, 0);

        // Checkpoint should be at least as high as before
        assert!(projection.get_checkpoint().await.unwrap_or(0) >= checkpoint);
    }

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_concurrent_graph_operations() -> Result<(), Box<dyn std::error::Error>> {
    let (client, jetstream) = connect_test_nats().await?;
    let nats_client = Arc::new(NatsClient::new(client.clone(), Some(jetstream.clone())).await?);
    let event_store = Arc::new(DistributedEventStore::new(jetstream.clone()).await?);

    // Setup projections
    let graph_summary = Arc::new(RwLock::new(GraphSummaryProjection::new()));
    let mut projection_handler = ProjectionHandler::new(
        event_store.clone(),
        graph_summary.clone(),
    );

    projection_handler.start(jetstream.clone()).await?;

    // Create multiple graphs concurrently
    let handler = Arc::new(GraphCommandHandler::new(event_store.clone()));
    let mut handles = Vec::new();

    for i in 0..10 {
        let handler_clone = handler.clone();
        let handle = tokio::spawn(async move {
            let graph_id = GraphId::new();
            let cmd = GraphCommand::CreateGraph {
                id: graph_id,
                metadata: GraphMetadata {
                    name: format!("Concurrent Graph {}", i),
                    description: Some(format!("Graph created concurrently #{}", i)),
                    tags: vec!["concurrent".to_string()],
                    properties: Default::default(),
                },
            };

            handler_clone.handle(cmd).await?;

            // Add some nodes
            for j in 0..5 {
                let node_cmd = GraphCommand::Node(NodeCommand::AddNode {
                    graph_id,
                    node_id: NodeId::new(),
                    content: format!("Node {}-{}", i, j),
                    position: Position3D {
                        x: j as f32 * 10.0,
                        y: i as f32 * 10.0,
                        z: 0.0,
                    },
                    metadata: Default::default(),
                });

                handler_clone.handle(node_cmd).await?;
            }

            Ok::<GraphId, Box<dyn std::error::Error + Send + Sync>>(graph_id)
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    let graph_ids: Vec<GraphId> = futures::future::try_join_all(handles).await?;

    // Wait for projections to catch up
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Verify all graphs were created and have correct node counts
    {
        let projection = graph_summary.read().await;
        let (graph_count, total_nodes, total_edges) = projection.get_total_stats();

        assert_eq!(graph_count, 10);
        assert_eq!(total_nodes, 50); // 10 graphs * 5 nodes each
        assert_eq!(total_edges, 0);

        // Verify each graph individually
        for graph_id in &graph_ids {
            let summary = projection.get_summary(graph_id).unwrap();
            assert_eq!(summary.node_count, 5);
        }
    }

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_error_recovery_and_consistency() -> Result<(), Box<dyn std::error::Error>> {
    let (client, jetstream) = connect_test_nats().await?;
    let event_store = Arc::new(DistributedEventStore::new(jetstream.clone()).await?);
    let handler = GraphCommandHandler::new(event_store.clone());

    // Setup projections
    let graph_summary = Arc::new(RwLock::new(GraphSummaryProjection::new()));
    let mut projection_handler = ProjectionHandler::new(
        event_store.clone(),
        graph_summary.clone(),
    );

    projection_handler.start(jetstream.clone()).await?;

    let graph_id = GraphId::new();

    // Create graph
    let create_cmd = GraphCommand::CreateGraph {
        id: graph_id,
        metadata: GraphMetadata {
            name: "Error Recovery Test".to_string(),
            description: None,
            tags: vec![],
            properties: Default::default(),
        },
    };

    handler.handle(create_cmd).await?;

    // Try to add node to non-existent graph (should fail)
    let invalid_cmd = GraphCommand::Node(NodeCommand::AddNode {
        graph_id: GraphId::new(), // Different ID
        node_id: NodeId::new(),
        content: "Invalid".to_string(),
        position: Position3D::default(),
        metadata: Default::default(),
    });

    let result = handler.handle(invalid_cmd).await;
    assert!(result.is_err());

    // Wait for any projection updates
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify projection state is still consistent
    {
        let projection = graph_summary.read().await;
        let summary = projection.get_summary(&graph_id).unwrap();
        assert_eq!(summary.node_count, 0); // No nodes should have been added
    }

    // Add valid nodes
    for i in 0..3 {
        let cmd = GraphCommand::Node(NodeCommand::AddNode {
            graph_id,
            node_id: NodeId::new(),
            content: format!("Valid Node {}", i),
            position: Position3D::default(),
            metadata: Default::default(),
        });

        handler.handle(cmd).await?;
    }

    // Wait for projection updates
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify correct count
    {
        let projection = graph_summary.read().await;
        let summary = projection.get_summary(&graph_id).unwrap();
        assert_eq!(summary.node_count, 3);
    }

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_projection_performance() -> Result<(), Box<dyn std::error::Error>> {
    let (client, jetstream) = connect_test_nats().await?;
    let event_store = Arc::new(DistributedEventStore::new(jetstream.clone()).await?);
    let handler = GraphCommandHandler::new(event_store.clone());

    // Setup projections
    let graph_summary = Arc::new(RwLock::new(GraphSummaryProjection::new()));
    let mut projection_handler = ProjectionHandler::new(
        event_store.clone(),
        graph_summary.clone(),
    );

    projection_handler.start(jetstream.clone()).await?;

    // Create a large graph
    let graph_id = GraphId::new();
    let create_cmd = GraphCommand::CreateGraph {
        id: graph_id,
        metadata: GraphMetadata {
            name: "Performance Test Graph".to_string(),
            description: Some("Testing projection performance with many nodes".to_string()),
            tags: vec!["performance".to_string()],
            properties: Default::default(),
        },
    };

    handler.handle(create_cmd).await?;

    // Add many nodes
    let start = std::time::Instant::now();
    let node_count = 1000;

    for i in 0..node_count {
        let cmd = GraphCommand::Node(NodeCommand::AddNode {
            graph_id,
            node_id: NodeId::new(),
            content: format!("Node {}", i),
            position: Position3D {
                x: (i % 100) as f32,
                y: (i / 100) as f32,
                z: 0.0,
            },
            metadata: Default::default(),
        });

        handler.handle(cmd).await?;
    }

    let command_duration = start.elapsed();
    println!("Time to process {} node commands: {:?}", node_count, command_duration);

    // Wait for projections to catch up
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Measure query performance
    let query_start = std::time::Instant::now();
    {
        let projection = graph_summary.read().await;
        let summary = projection.get_summary(&graph_id).unwrap();
        assert_eq!(summary.node_count, node_count);

        let stats = projection.get_total_stats();
        assert_eq!(stats.1, node_count); // Total nodes
    }
    let query_duration = query_start.elapsed();

    println!("Time to query projection: {:?}", query_duration);
    assert!(query_duration < Duration::from_millis(10), "Query should be fast");

    Ok(())
}
