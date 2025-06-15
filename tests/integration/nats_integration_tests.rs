//! NATS messaging integration tests

use crate::fixtures::{TestNatsServer, assertions::*};
use async_nats::jetstream;
use cim_domain::{DomainEvent, DomainResult};

#[tokio::test]
async fn test_nats_event_publishing() -> DomainResult<()> {
    // Arrange
    let nats = TestNatsServer::start().await?;
    let subject = "test.events.graph.node.added";

    // Create a test event
    let event = DomainEvent::from(cim_domain_graph::GraphDomainEvent::NodeAdded {
        graph_id: cim_domain::GraphId::new(),
        node_id: cim_domain::NodeId::new(),
        node_type: cim_domain_graph::NodeType::WorkflowStep {
            step_type: cim_domain_graph::StepType::Process,
        },
        position: cim_domain_graph::Position3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        conceptual_point: cim_domain_graph::ConceptualPoint::default(),
        metadata: Default::default(),
    });

    // Act - Publish event
    let payload = serde_json::to_vec(&event)
        .map_err(|e| cim_domain::DomainError::Serialization(e.to_string()))?;

    nats.jetstream()
        .publish(subject, payload.into())
        .await
        .map_err(|e| cim_domain::DomainError::Infrastructure(format!("Failed to publish: {}", e)))?
        .await
        .map_err(|e| {
            cim_domain::DomainError::Infrastructure(format!("Publish not acknowledged: {}", e))
        })?;

    // Assert - Try to consume the event
    let consumer = nats
        .jetstream()
        .create_consumer(
            "TEST-EVENTS",
            jetstream::consumer::pull::Config {
                durable_name: Some("test-consumer".to_string()),
                filter_subject: subject.to_string(),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| {
            cim_domain::DomainError::Infrastructure(format!("Failed to create consumer: {}", e))
        })?;

    let mut messages = consumer
        .fetch()
        .max_messages(1)
        .expires(std::time::Duration::from_secs(1))
        .messages()
        .await
        .map_err(|e| {
            cim_domain::DomainError::Infrastructure(format!("Failed to fetch messages: {}", e))
        })?;

    let received = messages.next().await;
    assert!(received.is_some(), "Expected to receive published event");

    // Cleanup
    nats.cleanup().await?;

    Ok(())
}

#[tokio::test]
async fn test_nats_event_subscription() -> DomainResult<()> {
    // Arrange
    let nats = TestNatsServer::start().await?;
    let subject = "test.events.graph.>";

    // Create subscription
    let mut subscriber = nats.client().subscribe(subject).await.map_err(|e| {
        cim_domain::DomainError::Infrastructure(format!("Failed to subscribe: {}", e))
    })?;

    // Act - Publish multiple events
    let events = vec![
        ("test.events.graph.node.added", "NodeAdded"),
        ("test.events.graph.edge.added", "EdgeAdded"),
        ("test.events.graph.node.removed", "NodeRemoved"),
    ];

    for (subj, event_type) in &events {
        let payload = format!(r#"{{"event_type":"{}"}}"#, event_type);
        nats.client()
            .publish(*subj, payload.into())
            .await
            .map_err(|e| {
                cim_domain::DomainError::Infrastructure(format!("Failed to publish: {}", e))
            })?;
    }

    // Assert - Receive all events
    let mut received_count = 0;
    let timeout = tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while let Some(msg) = subscriber.next().await {
            received_count += 1;
            if received_count >= events.len() {
                break;
            }
        }
    })
    .await;

    assert!(timeout.is_ok(), "Timeout waiting for messages");
    assert_eq!(
        received_count,
        events.len(),
        "Did not receive all published events"
    );

    // Cleanup
    nats.cleanup().await?;

    Ok(())
}

#[tokio::test]
async fn test_jetstream_persistence() -> DomainResult<()> {
    // Arrange
    let nats = TestNatsServer::start().await?;
    let subject = "test.events.graph.persisted";

    // Publish event
    let event_data = r#"{"event_type":"TestEvent","data":"persistent"}"#;
    let ack = nats
        .jetstream()
        .publish(subject, event_data.into())
        .await
        .map_err(|e| cim_domain::DomainError::Infrastructure(format!("Failed to publish: {}", e)))?
        .await
        .map_err(|e| {
            cim_domain::DomainError::Infrastructure(format!("Publish not acknowledged: {}", e))
        })?;

    // Act - Create consumer after publishing
    let consumer = nats
        .jetstream()
        .create_consumer(
            "TEST-EVENTS",
            jetstream::consumer::pull::Config {
                durable_name: Some("persistence-test".to_string()),
                filter_subject: subject.to_string(),
                deliver_policy: jetstream::consumer::DeliverPolicy::All,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| {
            cim_domain::DomainError::Infrastructure(format!("Failed to create consumer: {}", e))
        })?;

    // Assert - Should receive previously published event
    let mut messages = consumer
        .fetch()
        .max_messages(1)
        .expires(std::time::Duration::from_secs(1))
        .messages()
        .await
        .map_err(|e| {
            cim_domain::DomainError::Infrastructure(format!("Failed to fetch messages: {}", e))
        })?;

    let msg = messages.next().await;
    assert!(msg.is_some(), "Expected to receive persisted event");

    if let Some(msg) = msg {
        let msg = msg.map_err(|e| {
            cim_domain::DomainError::Infrastructure(format!("Message error: {}", e))
        })?;
        let payload = std::str::from_utf8(&msg.payload)
            .map_err(|e| cim_domain::DomainError::Serialization(format!("Invalid UTF-8: {}", e)))?;
        assert!(payload.contains("persistent"), "Expected persisted data");
    }

    // Cleanup
    nats.cleanup().await?;

    Ok(())
}

#[tokio::test]
async fn test_concurrent_event_publishing() -> DomainResult<()> {
    use tokio::task::JoinSet;

    // Arrange
    let nats = TestNatsServer::start().await?;
    let client = nats.client().clone();

    // Act - Publish events concurrently
    let mut tasks = JoinSet::new();

    for i in 0..10 {
        let client = client.clone();
        tasks.spawn(async move {
            let subject = format!("test.events.graph.concurrent.{}", i);
            let payload = format!(r#"{{"event_id":{}}}"#, i);
            client.publish(subject, payload.into()).await
        });
    }

    // Wait for all publishes
    let mut publish_count = 0;
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(_)) => publish_count += 1,
            _ => panic!("Concurrent publish failed"),
        }
    }

    // Assert
    assert_eq!(publish_count, 10, "Not all concurrent publishes succeeded");

    // Cleanup
    nats.cleanup().await?;

    Ok(())
}
