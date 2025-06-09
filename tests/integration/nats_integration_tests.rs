//! NATS integration tests
//!
//! These tests verify NATS-specific functionality including
//! event publishing, consumption, and the Bevy-NATS bridge.

use ia::domain::events::{DomainEvent, GraphEvent, NodeEvent};
use ia::domain::value_objects::{GraphId, NodeId, Position3D, GraphMetadata};
use ia::infrastructure::event_bridge::{EventBridge, BridgeCommand, BridgeEvent};
use ia::infrastructure::nats::{NatsClient, NatsConfig};
use super::fixtures::*;
use async_nats::jetstream;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_nats_event_publishing_and_consumption() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = connect_test_nats().await?;

    // Create test stream
    let stream_config = jetstream::stream::Config {
        name: "TEST-EVENTS".to_string(),
        subjects: vec!["events.test.>".to_string()],
        retention: jetstream::stream::RetentionPolicy::WorkQueue,
        storage: jetstream::stream::StorageType::Memory,
        ..Default::default()
    };

    let stream = jetstream.create_stream(stream_config).await?;

    // Create consumer
    let consumer_config = jetstream::consumer::pull::Config {
        name: Some("test-consumer".to_string()),
        ..Default::default()
    };

    let mut consumer = stream.create_consumer(consumer_config).await?;

    // Publish test event
    let event = DomainEvent::GraphCreated {
        id: GraphId::new(),
        metadata: GraphMetadata {
            name: "nats-test-graph".to_string(),
            ..Default::default()
        },
        timestamp: std::time::SystemTime::now(),
    };

    let payload = serde_json::to_vec(&event)?;
    jetstream.publish("events.test.graph.created", payload.into())
        .await?
        .await?;

    // Consume event
    let mut messages = consumer.messages().await?;
    let message = tokio::time::timeout(
        Duration::from_secs(1),
        messages.next()
    ).await??;

    let message = message?;
    let received_event: DomainEvent = serde_json::from_slice(&message.payload)?;

    // Verify event
    match received_event {
        DomainEvent::GraphCreated { metadata, .. } => {
            assert_eq!(metadata.name, "nats-test-graph");
        }
        _ => panic!("Unexpected event type"),
    }

    message.ack().await?;

    // Cleanup
    jetstream.delete_stream("TEST-EVENTS").await?;

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_event_bridge_bidirectional_flow() -> Result<(), Box<dyn std::error::Error>> {
    let event_bridge = EventBridge::new();

    // Start the bridge
    let (client, jetstream) = connect_test_nats().await?;
    let nats_client = Arc::new(NatsClient::new(client, Some(jetstream)).await?);

    event_bridge.start(nats_client.clone()).await?;

    // Test command flow (Bevy → NATS)
    let test_command = BridgeCommand::PublishEvent {
        subject: "test.command.flow".to_string(),
        event: DomainEvent::NodeAdded {
            graph_id: GraphId::new(),
            node_id: NodeId::new(),
            content: "bridge-test-node".to_string(),
            position: Position3D { x: 1.0, y: 2.0, z: 3.0 },
            metadata: Default::default(),
            timestamp: std::time::SystemTime::now(),
        },
    };

    event_bridge.send_command(test_command).await?;

    // Give time for async processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test event flow (NATS → Bevy)
    // Create a receiver for bridge events
    let mut event_receiver = event_bridge.subscribe_events();

    // Publish event to NATS that bridge should pick up
    let incoming_event = DomainEvent::GraphCreated {
        id: GraphId::new(),
        metadata: GraphMetadata {
            name: "incoming-graph".to_string(),
            ..Default::default()
        },
        timestamp: std::time::SystemTime::now(),
    };

    nats_client.publish("events.graph.created", &incoming_event).await?;

    // Receive event through bridge
    let bridge_event = tokio::time::timeout(
        Duration::from_secs(1),
        event_receiver.recv()
    ).await??;

    match bridge_event {
        Some(BridgeEvent::EventReceived { event, .. }) => {
            match event {
                DomainEvent::GraphCreated { metadata, .. } => {
                    assert_eq!(metadata.name, "incoming-graph");
                }
                _ => panic!("Unexpected event type"),
            }
        }
        _ => panic!("Expected EventReceived"),
    }

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_nats_reconnection_handling() -> Result<(), Box<dyn std::error::Error>> {
    let config = NatsConfig {
        url: "nats://localhost:4222".to_string(),
        jetstream_enabled: true,
        max_reconnects: Some(5),
        reconnect_delay_ms: 100,
        ..Default::default()
    };

    // Connect initially
    let client = NatsClient::connect(config.clone()).await?;

    // Verify connection is healthy
    assert!(client.health_check().await.is_ok());

    // Publish test event
    let event = DomainEvent::NodeAdded {
        graph_id: GraphId::new(),
        node_id: NodeId::new(),
        content: "reconnection-test".to_string(),
        position: Position3D::default(),
        metadata: Default::default(),
        timestamp: std::time::SystemTime::now(),
    };

    client.publish("test.reconnection", &event).await?;

    // Note: Full reconnection test would require stopping/starting NATS server
    // which is beyond the scope of unit tests

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_jetstream_stream_management() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = connect_test_nats().await?;

    // Create multiple streams for different event types
    let streams = vec![
        ("GRAPH-EVENTS", vec!["events.graph.>".to_string()]),
        ("NODE-EVENTS", vec!["events.node.>".to_string()]),
        ("EDGE-EVENTS", vec!["events.edge.>".to_string()]),
    ];

    for (name, subjects) in &streams {
        let config = jetstream::stream::Config {
            name: name.to_string(),
            subjects: subjects.clone(),
            retention: jetstream::stream::RetentionPolicy::Limits,
            max_messages: 10000,
            max_age: Duration::from_days(7),
            storage: jetstream::stream::StorageType::File,
            duplicate_window: Duration::from_secs(120),
            ..Default::default()
        };

        jetstream.create_stream(config).await?;
    }

    // Verify streams exist
    for (name, _) in &streams {
        let stream = jetstream.get_stream(name).await?;
        let info = stream.info().await?;
        assert_eq!(info.config.name, *name);
    }

    // Publish events to appropriate streams
    let graph_event = DomainEvent::GraphCreated {
        id: GraphId::new(),
        metadata: GraphMetadata::default(),
        timestamp: std::time::SystemTime::now(),
    };

    let node_event = DomainEvent::NodeAdded {
        graph_id: GraphId::new(),
        node_id: NodeId::new(),
        content: "test".to_string(),
        position: Position3D::default(),
        metadata: Default::default(),
        timestamp: std::time::SystemTime::now(),
    };

    // Publish to correct subjects
    let graph_payload = serde_json::to_vec(&graph_event)?;
    jetstream.publish("events.graph.created", graph_payload.into()).await?.await?;

    let node_payload = serde_json::to_vec(&node_event)?;
    jetstream.publish("events.node.added", node_payload.into()).await?.await?;

    // Verify messages in streams
    let graph_stream = jetstream.get_stream("GRAPH-EVENTS").await?;
    let graph_info = graph_stream.info().await?;
    assert_eq!(graph_info.state.messages, 1);

    let node_stream = jetstream.get_stream("NODE-EVENTS").await?;
    let node_info = node_stream.info().await?;
    assert_eq!(node_info.state.messages, 1);

    // Cleanup
    for (name, _) in &streams {
        jetstream.delete_stream(name).await?;
    }

    Ok(())
}

#[tokio::test]
#[ignore = "requires running NATS server"]
async fn test_event_ordering_guarantees() -> Result<(), Box<dyn std::error::Error>> {
    let (_client, jetstream) = connect_test_nats().await?;

    // Create ordered stream
    let stream_config = jetstream::stream::Config {
        name: "ORDERED-EVENTS".to_string(),
        subjects: vec!["ordered.>".to_string()],
        retention: jetstream::stream::RetentionPolicy::WorkQueue,
        storage: jetstream::stream::StorageType::Memory,
        ..Default::default()
    };

    let stream = jetstream.create_stream(stream_config).await?;

    // Create ordered consumer
    let consumer = stream.create_consumer(jetstream::consumer::pull::OrderedConfig {
        name: Some("ordered-consumer".to_string()),
        ..Default::default()
    }).await?;

    // Publish events in order
    let graph_id = GraphId::new();
    let events = vec![
        DomainEvent::GraphCreated {
            id: graph_id,
            metadata: GraphMetadata::default(),
            timestamp: std::time::SystemTime::now(),
        },
        DomainEvent::NodeAdded {
            graph_id,
            node_id: NodeId::new(),
            content: "node-1".to_string(),
            position: Position3D::default(),
            metadata: Default::default(),
            timestamp: std::time::SystemTime::now(),
        },
        DomainEvent::NodeAdded {
            graph_id,
            node_id: NodeId::new(),
            content: "node-2".to_string(),
            position: Position3D::default(),
            metadata: Default::default(),
            timestamp: std::time::SystemTime::now(),
        },
    ];

    // Publish all events
    for (i, event) in events.iter().enumerate() {
        let payload = serde_json::to_vec(event)?;
        jetstream.publish(
            format!("ordered.event.{}", i),
            payload.into()
        ).await?.await?;
    }

    // Consume in order
    let mut messages = consumer.messages().await?;
    let mut received_events = Vec::new();

    for _ in 0..events.len() {
        let message = tokio::time::timeout(
            Duration::from_secs(1),
            messages.next()
        ).await???;

        let event: DomainEvent = serde_json::from_slice(&message.payload)?;
        received_events.push(event);
        message.ack().await?;
    }

    // Verify order is preserved
    assert_eq!(received_events.len(), events.len());

    // First should be GraphCreated
    assert!(matches!(received_events[0], DomainEvent::GraphCreated { .. }));

    // Next two should be NodeAdded with correct content
    match &received_events[1] {
        DomainEvent::NodeAdded { content, .. } => assert_eq!(content, "node-1"),
        _ => panic!("Expected NodeAdded"),
    }

    match &received_events[2] {
        DomainEvent::NodeAdded { content, .. } => assert_eq!(content, "node-2"),
        _ => panic!("Expected NodeAdded"),
    }

    // Cleanup
    jetstream.delete_stream("ORDERED-EVENTS").await?;

    Ok(())
}
