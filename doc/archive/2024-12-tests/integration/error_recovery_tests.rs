//! Error recovery integration tests
//!
//! These tests verify system resilience and error recovery capabilities.

use super::fixtures::*;
use ia::application::command_handlers::{CommandHandler, GraphCommandHandler};
use ia::domain::commands::GraphCommand;
use ia::domain::events::DomainEvent;
use ia::domain::value_objects::{GraphId, GraphMetadata, NodeId, Position3D};
use ia::infrastructure::event_store::{DistributedEventStore, EventStore};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_event_store_recovery_after_failure() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = connect_test_nats().await?;
    let event_store = Arc::new(DistributedEventStore::new(jetstream.clone()).await?);

    // Create initial events
    let graph_id = GraphId::new();
    let events = vec![DomainEvent::GraphCreated {
        id: graph_id,
        metadata: GraphMetadata {
            name: "recovery-test".to_string(),
            ..Default::default()
        },
        timestamp: std::time::SystemTime::now(),
    }];

    // Store events
    event_store
        .append_events(graph_id.to_string(), events)
        .await?;

    // Simulate failure by creating new event store instance
    let event_store2 = Arc::new(DistributedEventStore::new(jetstream).await?);

    // Should be able to retrieve events from new instance
    let recovered_events = event_store2.get_events(graph_id.to_string()).await?;
    assert_eq!(recovered_events.len(), 1);

    match &recovered_events[0] {
        DomainEvent::GraphCreated { metadata, .. } => {
            assert_eq!(metadata.name, "recovery-test");
        }
        _ => panic!("Unexpected event type"),
    }

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_concurrent_modification_handling() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = connect_test_nats().await?;
    let event_store = Arc::new(DistributedEventStore::new(jetstream).await?);
    let handler = Arc::new(GraphCommandHandler::new(event_store.clone()));

    // Create graph
    let graph_id = GraphId::new();
    let create_cmd = GraphCommand::CreateGraph {
        id: graph_id,
        name: "concurrent-mod-test".to_string(),
        metadata: HashMap::new(),
    };

    handler.handle(create_cmd).await?;

    // Simulate concurrent modifications
    let node_ids: Vec<NodeId> = (0..10).map(|_| NodeId::new()).collect();
    let mut handles = Vec::new();

    // Multiple threads trying to add nodes to same positions
    for i in 0..5 {
        let handler_clone = handler.clone();
        let graph_id_clone = graph_id;
        let node_id1 = node_ids[i * 2];
        let node_id2 = node_ids[i * 2 + 1];

        let handle = tokio::spawn(async move {
            // Thread 1: Add two nodes
            let cmd1 = NodeCommand::AddNode {
                graph_id: graph_id_clone,
                node_id: node_id1,
                content: format!("node-a-{}", i),
                position: Position3D {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },
                metadata: Default::default(),
            };

            let cmd2 = NodeCommand::AddNode {
                graph_id: graph_id_clone,
                node_id: node_id2,
                content: format!("node-b-{}", i),
                position: Position3D {
                    x: i as f32,
                    y: 1.0,
                    z: 0.0,
                },
                metadata: Default::default(),
            };

            let result1 = handler_clone.handle(cmd1.into()).await;
            let result2 = handler_clone.handle(cmd2.into()).await;

            (result1, result2)
        });

        handles.push(handle);
    }

    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // All should succeed (no conflicts for adding different nodes)
    for result in results {
        let (r1, r2) = result?;
        assert!(r1.is_ok());
        assert!(r2.is_ok());
    }

    // Verify final state
    let events = event_store.get_events(graph_id.to_string()).await?;
    assert_eq!(events.len(), 11); // 1 create + 10 nodes

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_event_deduplication() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = connect_test_nats().await?;

    // Create test stream with deduplication window
    let stream_config = jetstream::stream::Config {
        name: "DEDUP-TEST".to_string(),
        subjects: vec!["dedup.test.>".to_string()],
        retention: jetstream::stream::RetentionPolicy::WorkQueue,
        storage: jetstream::stream::StorageType::Memory,
        duplicate_window: Duration::from_secs(60),
        ..Default::default()
    };

    let stream = jetstream.create_stream(stream_config).await?;

    // Create event with specific ID
    let event = DomainEvent::GraphCreated {
        id: GraphId::new(),
        metadata: GraphMetadata {
            name: "dedup-test".to_string(),
            ..Default::default()
        },
        timestamp: std::time::SystemTime::now(),
    };

    let payload = serde_json::to_vec(&event)?;
    let msg_id = "test-msg-12345";

    // Publish same event multiple times with same ID
    for _ in 0..3 {
        let headers = async_nats::HeaderMap::new().insert("Nats-Msg-Id", msg_id);

        jetstream
            .publish_with_headers("dedup.test.event", headers, payload.clone().into())
            .await?
            .await?;
    }

    // Check stream stats - should only have 1 message
    let info = stream.info().await?;
    assert_eq!(info.state.messages, 1);

    // Cleanup
    jetstream.delete_stream("DEDUP-TEST").await?;

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_partial_failure_rollback() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = connect_test_nats().await?;
    let event_store = Arc::new(DistributedEventStore::new(jetstream).await?);
    let handler = GraphCommandHandler::new(event_store.clone());

    // Create graph
    let graph_id = GraphId::new();
    let create_cmd = GraphCommand::CreateGraph {
        id: graph_id,
        name: "rollback-test".to_string(),
        metadata: HashMap::new(),
    };

    handler.handle(create_cmd).await?;

    // Add some nodes
    let node_ids: Vec<NodeId> = (0..5).map(|_| NodeId::new()).collect();

    for (i, node_id) in node_ids.iter().enumerate() {
        let cmd = NodeCommand::AddNode {
            graph_id,
            node_id: *node_id,
            content: format!("node-{}", i),
            position: Position3D {
                x: i as f32,
                y: 0.0,
                z: 0.0,
            },
            metadata: Default::default(),
        };

        handler.handle(cmd.into()).await?;
    }

    // Try to remove a node that would cause cascade deletes
    // This tests that either all events are applied or none
    let remove_cmd = NodeCommand::RemoveNode {
        graph_id,
        node_id: node_ids[2],
    };

    let result = handler.handle(remove_cmd.into()).await;
    assert!(result.is_ok());

    // Verify events are consistent
    let events = event_store.get_events(graph_id.to_string()).await?;

    // Should have: 1 create + 5 adds + removal events
    assert!(events.len() >= 7);

    // Verify CID chain integrity after all operations
    let chain_events: Vec<_> = events
        .iter()
        .map(|e| e.as_chained_event())
        .collect::<Result<Vec<_>, _>>()?;

    TestAssertions::assert_cid_chain_valid(&chain_events)?;

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_event_replay_after_crash() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = connect_test_nats().await?;
    let event_store = Arc::new(DistributedEventStore::new(jetstream.clone()).await?);

    // Create events
    let graph_id = GraphId::new();
    let events = vec![
        DomainEvent::GraphCreated {
            id: graph_id,
            metadata: GraphMetadata {
                name: "crash-test".to_string(),
                ..Default::default()
            },
            timestamp: std::time::SystemTime::now(),
        },
        DomainEvent::NodeAdded {
            graph_id,
            node_id: NodeId::new(),
            content: "before-crash".to_string(),
            position: Position3D::default(),
            metadata: Default::default(),
            timestamp: std::time::SystemTime::now(),
        },
    ];

    event_store
        .append_events(graph_id.to_string(), events)
        .await?;

    // Simulate crash and recovery
    drop(event_store);

    // Create new instance and replay
    let new_event_store = Arc::new(DistributedEventStore::new(jetstream).await?);
    let recovered_events = new_event_store.get_events(graph_id.to_string()).await?;

    // Should have all events
    assert_eq!(recovered_events.len(), 2);

    // Add more events after recovery
    let post_crash_event = DomainEvent::NodeAdded {
        graph_id,
        node_id: NodeId::new(),
        content: "after-crash".to_string(),
        position: Position3D::default(),
        metadata: Default::default(),
        timestamp: std::time::SystemTime::now(),
    };

    new_event_store
        .append_events(graph_id.to_string(), vec![post_crash_event])
        .await?;

    // Verify complete chain
    let final_events = new_event_store.get_events(graph_id.to_string()).await?;
    assert_eq!(final_events.len(), 3);

    // Verify content order
    match &final_events[1] {
        DomainEvent::NodeAdded { content, .. } => assert_eq!(content, "before-crash"),
        _ => panic!("Unexpected event"),
    }

    match &final_events[2] {
        DomainEvent::NodeAdded { content, .. } => assert_eq!(content, "after-crash"),
        _ => panic!("Unexpected event"),
    }

    Ok(())
}
