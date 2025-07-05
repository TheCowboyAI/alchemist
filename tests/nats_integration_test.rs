//! NATS Integration Tests
//! 
//! These tests verify that the CIM system works correctly with NATS messaging
//! 
//! Prerequisites:
//! - NATS server running on localhost:4222
//! - Run with: nats-server -js
//! 
//! To run these tests:
//! cargo test --test nats_integration_test -- --ignored --show-output

use cim_domain::{DomainResult, GraphId, NodeId, DomainError};
use async_nats::{Client, jetstream};
use serde::{Serialize, Deserialize};
use std::time::Duration;
use futures::StreamExt;

/// Test event structure that mimics domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestDomainEvent {
    event_id: String,
    event_type: String,
    aggregate_id: String,
    payload: serde_json::Value,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Helper to check if NATS is available
async fn nats_available() -> bool {
    match async_nats::connect("nats://localhost:4222").await {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[tokio::test]
#[ignore = "Requires NATS server running"]
async fn test_nats_connection() -> DomainResult<()> {
    if !nats_available().await {
        eprintln!("⚠️  NATS server not available, skipping test");
        return Ok(());
    }

    let client = async_nats::connect("nats://localhost:4222")
        .await
        .map_err(|e| DomainError::generic(format!("Failed to connect: {}", e)))?;

    // Test basic pub/sub
    let mut subscriber = client.subscribe("test.connection").await
        .map_err(|e| DomainError::generic(e.to_string()))?;

    client.publish("test.connection", "Hello NATS".into()).await
        .map_err(|e| DomainError::generic(e.to_string()))?;

    let msg = tokio::time::timeout(Duration::from_secs(1), subscriber.next())
        .await
        .map_err(|_| DomainError::generic("Timeout waiting for message".to_string()))?
        .ok_or_else(|| DomainError::generic("No message received".to_string()))?;

    assert_eq!(msg.payload, "Hello NATS");
    println!("✅ NATS connection test passed");

    Ok(())
}

#[tokio::test]
#[ignore = "Requires NATS server running"]
async fn test_jetstream_event_store() -> DomainResult<()> {
    if !nats_available().await {
        eprintln!("⚠️  NATS server not available, skipping test");
        return Ok(());
    }

    let client = async_nats::connect("nats://localhost:4222").await
        .map_err(|e| DomainError::generic(e.to_string()))?;

    let jetstream = jetstream::new(client);

    // Create or update test stream
    let stream_config = jetstream::stream::Config {
        name: "TEST_EVENTS".to_string(),
        subjects: vec!["events.test.>".to_string()],
        retention: jetstream::stream::RetentionPolicy::Limits,
        max_messages: 10_000,
        max_age: Duration::from_secs(3600), // 1 hour
        ..Default::default()
    };

    // Create or get existing stream
    let mut stream = match jetstream.get_stream("TEST_EVENTS").await {
        Ok(s) => s,
        Err(_) => jetstream.create_stream(stream_config).await
            .map_err(|e| DomainError::generic(format!("Failed to create stream: {}", e)))?,
    };

    // Publish test events
    let graph_id = GraphId::new();
    let node_id = NodeId::new();

    let events = vec![
        TestDomainEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: "GraphCreated".to_string(),
            aggregate_id: graph_id.to_string(),
            payload: serde_json::json!({
                "name": "Test Graph",
                "graph_type": "Workflow"
            }),
            timestamp: chrono::Utc::now(),
        },
        TestDomainEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: "NodeAdded".to_string(),
            aggregate_id: graph_id.to_string(),
            payload: serde_json::json!({
                "node_id": node_id.to_string(),
                "node_type": "Process",
                "position": { "x": 0.0, "y": 0.0, "z": 0.0 }
            }),
            timestamp: chrono::Utc::now(),
        },
    ];

    // Publish events
    for event in &events {
        let subject = format!("events.test.{}", event.aggregate_id);
        let payload = serde_json::to_vec(&event)
            .map_err(|e| DomainError::generic(e.to_string()))?;

        jetstream.publish(subject, payload.into()).await
            .map_err(|e| DomainError::generic(format!("Failed to publish: {}", e)))?;
    }

    // Verify events were stored
    let info = stream.info().await
        .map_err(|e| DomainError::generic(e.to_string()))?;
    
    // At least our 2 events should be there (might have more from previous runs)
    assert!(info.state.messages >= 2);
    println!("✅ JetStream event store test passed - {} events stored", info.state.messages);

    Ok(())
}

#[tokio::test]
#[ignore = "Requires NATS server running"]
async fn test_event_replay() -> DomainResult<()> {
    if !nats_available().await {
        eprintln!("⚠️  NATS server not available, skipping test");
        return Ok(());
    }

    let client = async_nats::connect("nats://localhost:4222").await
        .map_err(|e| DomainError::generic(e.to_string()))?;

    let jetstream = jetstream::new(client);

    // Ensure stream exists
    let stream_name = "REPLAY_TEST";
    let stream_config = jetstream::stream::Config {
        name: stream_name.to_string(),
        subjects: vec!["replay.test.>".to_string()],
        retention: jetstream::stream::RetentionPolicy::Limits,
        ..Default::default()
    };

    // Create or get existing stream
    let _ = match jetstream.get_stream(stream_name).await {
        Ok(s) => s,
        Err(_) => jetstream.create_stream(stream_config).await
            .map_err(|e| DomainError::generic(e.to_string()))?,
    };

    // Publish sequence of events
    let aggregate_id = GraphId::new();
    let events_to_publish = 5;

    for i in 0..events_to_publish {
        let event = TestDomainEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: "TestEvent".to_string(),
            aggregate_id: aggregate_id.to_string(),
            payload: serde_json::json!({ "sequence": i }),
            timestamp: chrono::Utc::now(),
        };

        let subject = format!("replay.test.{}", aggregate_id);
        let payload = serde_json::to_vec(&event)
            .map_err(|e| DomainError::generic(e.to_string()))?;

        jetstream.publish(subject, payload.into()).await
            .map_err(|e| DomainError::generic(e.to_string()))?;
    }

    // Create consumer for replay
    let consumer_config = jetstream::consumer::pull::Config {
        name: Some("replay_consumer".to_string()),
        deliver_policy: jetstream::consumer::DeliverPolicy::All,
        ..Default::default()
    };

    let stream = jetstream.get_stream(stream_name).await
        .map_err(|e| DomainError::generic(e.to_string()))?;

    let consumer = stream.create_consumer(consumer_config).await
        .map_err(|e| DomainError::generic(e.to_string()))?;

    // Replay events
    let mut replayed_count = 0;
    let mut messages = consumer.messages().await
        .map_err(|e| DomainError::generic(e.to_string()))?;

    while let Ok(Some(msg)) = tokio::time::timeout(
        Duration::from_millis(100),
        messages.next()
    ).await {
        match msg {
            Ok(message) => {
                let event: TestDomainEvent = serde_json::from_slice(&message.payload)
                    .map_err(|e| DomainError::generic(e.to_string()))?;

                println!("Replayed event {}: {}", replayed_count, event.event_type);
                replayed_count += 1;

                message.ack().await
                    .map_err(|e| DomainError::generic(e.to_string()))?;
            }
            Err(e) => {
                return Err(DomainError::generic(format!("Message error: {}", e)));
            }
        }
    }

    assert_eq!(replayed_count, events_to_publish);
    println!("✅ Event replay test passed - {} events replayed", replayed_count);

    Ok(())
}

#[tokio::test]
#[ignore = "Requires NATS server running"]
async fn test_cross_domain_event_flow() -> DomainResult<()> {
    if !nats_available().await {
        eprintln!("⚠️  NATS server not available, skipping test");
        return Ok(());
    }

    let client = async_nats::connect("nats://localhost:4222").await
        .map_err(|e| DomainError::generic(e.to_string()))?;

    // Set up subscribers for different domains
    let mut graph_sub = client.subscribe("domain.graph.>").await
        .map_err(|e| DomainError::generic(e.to_string()))?;

    let mut workflow_sub = client.subscribe("domain.workflow.>").await
        .map_err(|e| DomainError::generic(e.to_string()))?;

    // Simulate graph domain publishing event
    let graph_id = GraphId::new();
    let graph_event = TestDomainEvent {
        event_id: uuid::Uuid::new_v4().to_string(),
        event_type: "WorkflowGraphCreated".to_string(),
        aggregate_id: graph_id.to_string(),
        payload: serde_json::json!({
            "name": "Approval Process",
            "graph_type": "Workflow"
        }),
        timestamp: chrono::Utc::now(),
    };

    let payload = serde_json::to_vec(&graph_event)
        .map_err(|e| DomainError::generic(e.to_string()))?;

    client.publish("domain.graph.created", payload.clone().into()).await
        .map_err(|e| DomainError::generic(e.to_string()))?;

    // Graph domain receives its own event
    let graph_msg = tokio::time::timeout(Duration::from_secs(1), graph_sub.next())
        .await
        .map_err(|_| DomainError::generic("Timeout".to_string()))?
        .ok_or_else(|| DomainError::generic("No message".to_string()))?;

    let received_event: TestDomainEvent = serde_json::from_slice(&graph_msg.payload)
        .map_err(|e| DomainError::generic(e.to_string()))?;

    assert_eq!(received_event.event_type, "WorkflowGraphCreated");

    // Workflow domain reacts by creating workflow
    let workflow_event = TestDomainEvent {
        event_id: uuid::Uuid::new_v4().to_string(),
        event_type: "WorkflowCreated".to_string(),
        aggregate_id: format!("workflow-{}", graph_id),
        payload: serde_json::json!({
            "graph_id": graph_id.to_string(),
            "name": "Approval Process"
        }),
        timestamp: chrono::Utc::now(),
    };

    let workflow_payload = serde_json::to_vec(&workflow_event)
        .map_err(|e| DomainError::generic(e.to_string()))?;

    client.publish("domain.workflow.created", workflow_payload.into()).await
        .map_err(|e| DomainError::generic(e.to_string()))?;

    // Workflow domain receives event
    let workflow_msg = tokio::time::timeout(Duration::from_secs(1), workflow_sub.next())
        .await
        .map_err(|_| DomainError::generic("Timeout".to_string()))?
        .ok_or_else(|| DomainError::generic("No message".to_string()))?;

    let workflow_received: TestDomainEvent = serde_json::from_slice(&workflow_msg.payload)
        .map_err(|e| DomainError::generic(e.to_string()))?;

    assert_eq!(workflow_received.event_type, "WorkflowCreated");

    println!("✅ Cross-domain event flow test passed");

    Ok(())
}

#[tokio::test]
#[ignore = "Requires NATS server running"]
async fn test_concurrent_publishers() -> DomainResult<()> {
    if !nats_available().await {
        eprintln!("⚠️  NATS server not available, skipping test");
        return Ok(());
    }

    let client = async_nats::connect("nats://localhost:4222").await
        .map_err(|e| DomainError::generic(e.to_string()))?;

    let jetstream = jetstream::new(client.clone());

    // Create stream for concurrent test
    let stream_config = jetstream::stream::Config {
        name: "CONCURRENT_TEST".to_string(),
        subjects: vec!["concurrent.>".to_string()],
        ..Default::default()
    };

    // Create or get existing stream
    let _ = match jetstream.get_stream("CONCURRENT_TEST").await {
        Ok(s) => s,
        Err(_) => jetstream.create_stream(stream_config).await
            .map_err(|e| DomainError::generic(e.to_string()))?,
    };

    // Spawn multiple publishers
    let publisher_count = 10;
    let events_per_publisher = 100;
    let mut handles = Vec::new();

    for publisher_id in 0..publisher_count {
        let js = jetstream.clone();
        let handle = tokio::spawn(async move {
            for event_num in 0..events_per_publisher {
                let event = TestDomainEvent {
                    event_id: uuid::Uuid::new_v4().to_string(),
                    event_type: "ConcurrentEvent".to_string(),
                    aggregate_id: format!("publisher-{}", publisher_id),
                    payload: serde_json::json!({
                        "publisher": publisher_id,
                        "event": event_num
                    }),
                    timestamp: chrono::Utc::now(),
                };

                let payload = serde_json::to_vec(&event).unwrap();
                let subject = format!("concurrent.publisher.{}", publisher_id);

                js.publish(subject, payload.into()).await.unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all publishers
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all events were published
    let mut stream = jetstream.get_stream("CONCURRENT_TEST").await
        .map_err(|e| DomainError::generic(e.to_string()))?;

    let info = stream.info().await
        .map_err(|e| DomainError::generic(e.to_string()))?;

    let expected_total = publisher_count * events_per_publisher;
    assert_eq!(info.state.messages, expected_total as u64);

    println!("✅ Concurrent publishers test passed - {} events from {} publishers", 
             expected_total, publisher_count);

    Ok(())
}

#[tokio::test]
#[ignore = "Requires NATS server running"]
async fn test_event_deduplication() -> DomainResult<()> {
    if !nats_available().await {
        eprintln!("⚠️  NATS server not available, skipping test");
        return Ok(());
    }

    let client = async_nats::connect("nats://localhost:4222").await
        .map_err(|e| DomainError::generic(e.to_string()))?;

    let jetstream = jetstream::new(client);

    // Create stream with deduplication window
    let stream_config = jetstream::stream::Config {
        name: "DEDUP_TEST".to_string(),
        subjects: vec!["dedup.>".to_string()],
        duplicate_window: Duration::from_secs(60),
        ..Default::default()
    };

    // Create or get existing stream
    let mut stream = match jetstream.get_stream("DEDUP_TEST").await {
        Ok(s) => s,
        Err(_) => jetstream.create_stream(stream_config).await
            .map_err(|e| DomainError::generic(e.to_string()))?,
    };

    // Publish same event multiple times with same ID
    let event_id = uuid::Uuid::new_v4().to_string();
    let event = TestDomainEvent {
        event_id: event_id.clone(),
        event_type: "DedupTest".to_string(),
        aggregate_id: GraphId::new().to_string(),
        payload: serde_json::json!({"test": "deduplication"}),
        timestamp: chrono::Utc::now(),
    };

    let payload = serde_json::to_vec(&event)
        .map_err(|e| DomainError::generic(e.to_string()))?;

    // Publish 3 times with same message ID
    for _ in 0..3 {
        let mut headers = async_nats::HeaderMap::new();
        headers.insert("Nats-Msg-Id", event_id.as_str());

        jetstream.publish_with_headers("dedup.test", headers, payload.clone().into()).await
            .map_err(|e| DomainError::generic(e.to_string()))?;
    }

    // Check that only one message was stored
    let info = stream.info().await
        .map_err(|e| DomainError::generic(e.to_string()))?;

    assert_eq!(info.state.messages, 1);
    println!("✅ Event deduplication test passed - only 1 of 3 duplicate events stored");

    Ok(())
}

/// Helper function to clean up test streams
async fn cleanup_test_streams() -> DomainResult<()> {
    if !nats_available().await {
        return Ok(());
    }

    let client = async_nats::connect("nats://localhost:4222").await
        .map_err(|e| DomainError::generic(e.to_string()))?;

    let jetstream = jetstream::new(client);

    let test_streams = vec![
        "TEST_EVENTS",
        "REPLAY_TEST",
        "CONCURRENT_TEST",
        "DEDUP_TEST",
    ];

    for stream_name in test_streams {
        let _ = jetstream.delete_stream(stream_name).await;
    }

    Ok(())
}

#[tokio::test]
#[ignore = "Requires NATS server running"]
async fn test_cleanup() -> DomainResult<()> {
    cleanup_test_streams().await?;
    println!("✅ Test streams cleaned up");
    Ok(())
} 