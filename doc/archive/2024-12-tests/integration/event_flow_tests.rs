//! Event flow integration tests
//!
//! These tests verify the complete flow from command submission
//! through event storage to projection updates.

use super::fixtures::*;
use ia::application::command_handlers::{CommandHandler, GraphCommandHandler};
use ia::domain::aggregates::Graph;
use ia::domain::commands::{EdgeCommand, GraphCommand, NodeCommand};
use ia::domain::events::{DomainEvent, EdgeEvent, GraphEvent, NodeEvent};
use ia::domain::value_objects::{EdgeId, GraphId, GraphMetadata, NodeId, Position3D};
use ia::infrastructure::event_bridge::EventBridge;
use ia::infrastructure::event_store::{DistributedEventStore, EventStore};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_complete_command_to_projection_flow() -> Result<(), Box<dyn std::error::Error>> {
    // Setup
    let (client, jetstream) = connect_test_nats().await?;
    let event_store = Arc::new(DistributedEventStore::new(jetstream.clone()).await?);
    let event_bridge = EventBridge::new();

    // Create test stream for projections
    let test_stream = TestEventStream::new("projection-test").await?;

    // Test data
    let graph_id = GraphId::new();
    let node_id = NodeId::new();

    // Step 1: Submit CreateGraph command
    let create_cmd = Command::Graph(GraphCommand::CreateGraph {
        id: graph_id,
        name: "integration-test-graph".to_string(),
        metadata: {
            let mut map = HashMap::new();
            map.insert(
                "description".to_string(),
                serde_json::json!("Testing command to projection flow"),
            );
            map
        },
    });

    // Process command through handler
    let handler = GraphCommandHandler::new(event_store.clone());
    let events = handler.handle(create_cmd).await?;

    // Verify event was created
    assert_eq!(events.len(), 1);
    match &events[0] {
        DomainEvent::Graph(GraphEvent::GraphCreated { id, metadata }) => {
            assert_eq!(*id, graph_id);
            assert_eq!(metadata.name, "integration-test-graph");
        }
        _ => panic!("Expected GraphCreated event"),
    }

    // Step 2: Verify event was stored
    let stored_events = event_store.get_events(graph_id.to_string()).await?;
    assert_eq!(stored_events.len(), 1);

    // Step 3: Submit AddNode command
    let add_node_cmd = NodeCommand::AddNode {
        graph_id,
        node_id,
        content: "test-node".to_string(),
        position: Position3D {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        metadata: Default::default(),
    };

    // Process through event bridge (simulating Bevy → NATS flow)
    event_bridge.send_command(add_node_cmd.into()).await?;

    // Wait for async processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Step 4: Verify complete event chain
    let all_events = event_store.get_events(graph_id.to_string()).await?;
    assert_eq!(all_events.len(), 2);

    // Verify CID chain integrity
    let chain_events: Vec<_> = all_events
        .iter()
        .map(|e| e.as_chained_event())
        .collect::<Result<Vec<_>, _>>()?;

    TestAssertions::assert_cid_chain_valid(&chain_events)?;

    // Step 5: Verify projection would be updated (simulated)
    // In a real system, this would check the actual projection state
    let mut consumer = test_stream.create_consumer("projection-consumer").await?;

    // Publish events to projection stream
    for event in &all_events {
        test_stream.publish_event("graph.events", event).await?;
    }

    // Verify projection received events
    let projection_event: DomainEvent =
        TestAssertions::assert_event_published(&mut consumer, Duration::from_secs(1)).await?;

    match projection_event {
        DomainEvent::GraphCreated { .. } => {
            // Projection would update its state here
        }
        _ => panic!("Unexpected event type in projection"),
    }

    // Cleanup
    test_stream.cleanup().await?;

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_multi_aggregate_event_flow() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = connect_test_nats().await?;
    let event_store = Arc::new(DistributedEventStore::new(jetstream.clone()).await?);

    // Create multiple graphs
    let graph_ids: Vec<GraphId> = (0..3).map(|_| GraphId::new()).collect();
    let handler = GraphCommandHandler::new(event_store.clone());

    // Submit commands for each graph
    for (i, graph_id) in graph_ids.iter().enumerate() {
        let cmd = GraphCommand::CreateGraph {
            id: *graph_id,
            metadata: {
                let mut map = HashMap::new();
                map.insert(
                    "name".to_string(),
                    serde_json::json!(format!("graph-{}", i)),
                );
                map
            },
        };

        handler.handle(cmd).await?;
    }

    // Add nodes to each graph
    for graph_id in &graph_ids {
        for j in 0..5 {
            let node_cmd = NodeCommand::AddNode {
                graph_id: *graph_id,
                node_id: NodeId::new(),
                content: format!("node-{}", j),
                position: Position3D {
                    x: j as f32,
                    y: 0.0,
                    z: 0.0,
                },
                metadata: Default::default(),
            };

            // Process command
            let events = handler.handle(node_cmd.into()).await?;
            assert!(!events.is_empty());
        }
    }

    // Verify each aggregate has correct number of events
    for graph_id in &graph_ids {
        let events = event_store.get_events(graph_id.to_string()).await?;
        assert_eq!(events.len(), 6); // 1 create + 5 nodes
    }

    // Verify event isolation between aggregates
    let graph1_events = event_store.get_events(graph_ids[0].to_string()).await?;
    let graph2_events = event_store.get_events(graph_ids[1].to_string()).await?;

    // Events should be completely separate
    for event in &graph1_events {
        assert!(!graph2_events.contains(event));
    }

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_complex_graph_operations_flow() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = connect_test_nats().await?;
    let event_store = Arc::new(DistributedEventStore::new(jetstream.clone()).await?);
    let handler = GraphCommandHandler::new(event_store.clone());

    // Create graph
    let graph_id = GraphId::new();
    let create_cmd = GraphCommand::CreateGraph {
        id: graph_id,
        metadata: {
            let mut map = HashMap::new();
            map.insert("name".to_string(), serde_json::json!("complex-graph"));
            map
        },
    };

    handler.handle(create_cmd).await?;

    // Add multiple nodes
    let node_ids: Vec<NodeId> = (0..10).map(|_| NodeId::new()).collect();

    for (i, node_id) in node_ids.iter().enumerate() {
        let cmd = NodeCommand::AddNode {
            graph_id,
            node_id: *node_id,
            content: format!("node-{}", i),
            position: Position3D {
                x: (i as f32) * 2.0,
                y: 0.0,
                z: 0.0,
            },
            metadata: Default::default(),
        };

        handler.handle(cmd.into()).await?;
    }

    // Connect nodes with edges
    for i in 0..node_ids.len() - 1 {
        let edge_cmd = EdgeCommand::ConnectNodes {
            graph_id,
            edge_id: EdgeId::new(),
            source: node_ids[i],
            target: node_ids[i + 1],
            relationship: format!("edge-{}", i),
            metadata: Default::default(),
        };

        handler.handle(edge_cmd.into()).await?;
    }

    // Remove a node (should cascade delete edges)
    let remove_cmd = NodeCommand::RemoveNode {
        graph_id,
        node_id: node_ids[5],
    };

    let remove_events = handler.handle(remove_cmd.into()).await?;

    // Should have node removed event and edge removed events
    let node_removed_count = remove_events
        .iter()
        .filter(|e| matches!(e, DomainEvent::NodeRemoved { .. }))
        .count();
    let edge_removed_count = remove_events
        .iter()
        .filter(|e| matches!(e, DomainEvent::EdgeRemoved { .. }))
        .count();

    assert_eq!(node_removed_count, 1);
    assert!(edge_removed_count >= 2); // At least the edges connected to node 5

    // Verify final state through event replay
    let all_events = event_store.get_events(graph_id.to_string()).await?;

    // Replay events to reconstruct aggregate
    let mut aggregate = Graph::new(graph_id);
    for event in &all_events {
        aggregate.apply_event(event)?;
    }

    // Verify aggregate state
    assert_eq!(aggregate.nodes.len(), 9); // 10 - 1 removed
    assert!(aggregate.edges.len() < 9); // Some edges were removed

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_concurrent_command_processing() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = connect_test_nats().await?;
    let event_store = Arc::new(DistributedEventStore::new(jetstream.clone()).await?);
    let handler = Arc::new(GraphCommandHandler::new(event_store.clone()));

    // Create graph
    let graph_id = GraphId::new();
    let create_cmd = GraphCommand::CreateGraph {
        id: graph_id,
        metadata: {
            let mut map = HashMap::new();
            map.insert("name".to_string(), serde_json::json!("concurrent-test"));
            map
        },
    };

    handler.handle(create_cmd).await?;

    // Submit multiple commands concurrently
    let mut handles = Vec::new();

    for i in 0..20 {
        let handler_clone = handler.clone();
        let graph_id_clone = graph_id;

        let handle = tokio::spawn(async move {
            let cmd = NodeCommand::AddNode {
                graph_id: graph_id_clone,
                node_id: NodeId::new(),
                content: format!("concurrent-node-{}", i),
                position: Position3D {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },
                metadata: Default::default(),
            };

            handler_clone.handle(cmd.into()).await
        });

        handles.push(handle);
    }

    // Wait for all commands to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // Verify all succeeded
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }

    // Verify all events were stored
    let events = event_store.get_events(graph_id.to_string()).await?;
    assert_eq!(events.len(), 21); // 1 create + 20 nodes

    // Verify CID chain integrity despite concurrent processing
    let chain_events: Vec<_> = events
        .iter()
        .map(|e| e.as_chained_event())
        .collect::<Result<Vec<_>, _>>()?;

    TestAssertions::assert_cid_chain_valid(&chain_events)?;

    Ok(())
}
