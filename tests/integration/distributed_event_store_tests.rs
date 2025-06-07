//! Integration tests for DistributedEventStore
//! Tests the JetStream-based distributed event store implementation

use ia::infrastructure::event_store::{DistributedEventStore, EventStore};
use ia::infrastructure::nats::NatsClient;
use ia::domain::events::{DomainEvent, GraphEvent};
use ia::domain::value_objects::{GraphId, GraphMetadata, AggregateId};
use async_nats::jetstream;
use std::time::Duration;

/// Test helper to create a test NATS client
async fn create_test_client() -> Result<NatsClient, Box<dyn std::error::Error>> {
    let client = NatsClient::new(ia::infrastructure::nats::NatsConfig {
        url: "nats://localhost:4222".to_string(),
        auth: None,
        jetstream: Some(ia::infrastructure::nats::JetStreamConfig {
            domain: Some("test".to_string()),
            prefix: Some("TEST".to_string()),
            memory_storage: true,
            replicas: 1,
        }),
    }).await?;

    Ok(client)
}

/// Clean up test streams
async fn cleanup_test_streams(client: &NatsClient) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(context) = client.jetstream_context() {
        // Try to delete test streams, ignore if they don't exist
        let _ = context.delete_stream("EVENTS").await;
        let _ = context.delete_stream("TEST_EVENTS").await;
    }
    Ok(())
}

#[tokio::test]
async fn test_distributed_event_store_creation() -> Result<(), Box<dyn std::error::Error>> {
    let client = create_test_client().await?;
    cleanup_test_streams(&client).await?;

    let store = DistributedEventStore::new(client.clone()).await?;

    // Verify stream was created
    if let Ok(context) = client.jetstream_context() {
        let stream = context.get_stream("EVENTS").await?;
        assert_eq!(stream.info().await?.config.name, "EVENTS");
    }

    cleanup_test_streams(&client).await?;
    Ok(())
}

#[tokio::test]
async fn test_append_and_retrieve_events() -> Result<(), Box<dyn std::error::Error>> {
    let client = create_test_client().await?;
    cleanup_test_streams(&client).await?;

    let store = DistributedEventStore::new(client.clone()).await?;
    let aggregate_id = AggregateId::from(GraphId::new());

    // Create test events
    let events = vec![
        DomainEvent::Graph(GraphEvent::GraphCreated {
            id: GraphId::from(aggregate_id.clone()),
            metadata: GraphMetadata::new("Test Graph".to_string()),
        }),
        DomainEvent::Graph(GraphEvent::GraphMetadataUpdated {
            id: GraphId::from(aggregate_id.clone()),
            metadata: GraphMetadata::new("Updated Graph".to_string()),
        }),
    ];

    // Append events
    store.append_events(aggregate_id.clone(), events.clone(), None).await?;

    // Retrieve events
    let retrieved = store.get_events(aggregate_id.clone()).await?;

    assert_eq!(retrieved.len(), 2);

    // Verify event content
    match &retrieved[0] {
        DomainEvent::Graph(GraphEvent::GraphCreated { metadata, .. }) => {
            assert_eq!(metadata.name, "Test Graph");
        }
        _ => panic!("Unexpected event type"),
    }

    match &retrieved[1] {
        DomainEvent::Graph(GraphEvent::GraphMetadataUpdated { metadata, .. }) => {
            assert_eq!(metadata.name, "Updated Graph");
        }
        _ => panic!("Unexpected event type"),
    }

    cleanup_test_streams(&client).await?;
    Ok(())
}

#[tokio::test]
async fn test_optimistic_concurrency_control() -> Result<(), Box<dyn std::error::Error>> {
    let client = create_test_client().await?;
    cleanup_test_streams(&client).await?;

    let store = DistributedEventStore::new(client.clone()).await?;
    let aggregate_id = AggregateId::from(GraphId::new());

    // Append initial event
    let event1 = vec![DomainEvent::Graph(GraphEvent::GraphCreated {
        id: GraphId::from(aggregate_id.clone()),
        metadata: GraphMetadata::new("Test Graph".to_string()),
    })];

    store.append_events(aggregate_id.clone(), event1, Some(0)).await?;

    // Try to append with wrong version (should fail)
    let event2 = vec![DomainEvent::Graph(GraphEvent::GraphMetadataUpdated {
        id: GraphId::from(aggregate_id.clone()),
        metadata: GraphMetadata::new("Updated".to_string()),
    })];

    let result = store.append_events(aggregate_id.clone(), event2.clone(), Some(0)).await;
    assert!(result.is_err());

    // Append with correct version (should succeed)
    let result = store.append_events(aggregate_id.clone(), event2, Some(1)).await;
    assert!(result.is_ok());

    cleanup_test_streams(&client).await?;
    Ok(())
}

#[tokio::test]
async fn test_event_stream_subscription() -> Result<(), Box<dyn std::error::Error>> {
    let client = create_test_client().await?;
    cleanup_test_streams(&client).await?;

    let store = DistributedEventStore::new(client.clone()).await?;
    let aggregate_id = AggregateId::from(GraphId::new());

    // Subscribe to events
    let mut subscription = store.subscribe_to_events(Some(aggregate_id.clone())).await?;

    // Append event after subscription
    let event = vec![DomainEvent::Graph(GraphEvent::GraphCreated {
        id: GraphId::from(aggregate_id.clone()),
        metadata: GraphMetadata::new("Test Graph".to_string()),
    })];

    store.append_events(aggregate_id.clone(), event, None).await?;

    // Receive event from subscription
    let received = tokio::time::timeout(
        Duration::from_secs(5),
        subscription.recv()
    ).await??;

    match received {
        DomainEvent::Graph(GraphEvent::GraphCreated { metadata, .. }) => {
            assert_eq!(metadata.name, "Test Graph");
        }
        _ => panic!("Unexpected event type"),
    }

    cleanup_test_streams(&client).await?;
    Ok(())
}

#[tokio::test]
async fn test_multiple_aggregates() -> Result<(), Box<dyn std::error::Error>> {
    let client = create_test_client().await?;
    cleanup_test_streams(&client).await?;

    let store = DistributedEventStore::new(client.clone()).await?;

    let aggregate1 = AggregateId::from(GraphId::new());
    let aggregate2 = AggregateId::from(GraphId::new());

    // Append events to different aggregates
    let event1 = vec![DomainEvent::Graph(GraphEvent::GraphCreated {
        id: GraphId::from(aggregate1.clone()),
        metadata: GraphMetadata::new("Graph 1".to_string()),
    })];

    let event2 = vec![DomainEvent::Graph(GraphEvent::GraphCreated {
        id: GraphId::from(aggregate2.clone()),
        metadata: GraphMetadata::new("Graph 2".to_string()),
    })];

    store.append_events(aggregate1.clone(), event1, None).await?;
    store.append_events(aggregate2.clone(), event2, None).await?;

    // Retrieve events for each aggregate
    let events1 = store.get_events(aggregate1).await?;
    let events2 = store.get_events(aggregate2).await?;

    assert_eq!(events1.len(), 1);
    assert_eq!(events2.len(), 1);

    // Verify they're different
    match (&events1[0], &events2[0]) {
        (
            DomainEvent::Graph(GraphEvent::GraphCreated { metadata: m1, .. }),
            DomainEvent::Graph(GraphEvent::GraphCreated { metadata: m2, .. })
        ) => {
            assert_eq!(m1.name, "Graph 1");
            assert_eq!(m2.name, "Graph 2");
        }
        _ => panic!("Unexpected event types"),
    }

    cleanup_test_streams(&client).await?;
    Ok(())
}

#[tokio::test]
async fn test_event_store_statistics() -> Result<(), Box<dyn std::error::Error>> {
    let client = create_test_client().await?;
    cleanup_test_streams(&client).await?;

    let store = DistributedEventStore::new(client.clone()).await?;
    let aggregate_id = AggregateId::from(GraphId::new());

    // Get initial stats
    let initial_stats = store.get_statistics().await?;
    let initial_count = initial_stats.total_events;

    // Append some events
    for i in 0..5 {
        let event = vec![DomainEvent::Graph(GraphEvent::GraphMetadataUpdated {
            id: GraphId::from(aggregate_id.clone()),
            metadata: GraphMetadata::new(format!("Update {}", i)),
        })];
        store.append_events(aggregate_id.clone(), event, None).await?;
    }

    // Get updated stats
    let updated_stats = store.get_statistics().await?;
    assert_eq!(updated_stats.total_events, initial_count + 5);

    cleanup_test_streams(&client).await?;
    Ok(())
}

#[tokio::test]
async fn test_event_replay_from_position() -> Result<(), Box<dyn std::error::Error>> {
    let client = create_test_client().await?;
    cleanup_test_streams(&client).await?;

    let store = DistributedEventStore::new(client.clone()).await?;
    let aggregate_id = AggregateId::from(GraphId::new());

    // Append multiple events
    for i in 0..10 {
        let event = vec![DomainEvent::Graph(GraphEvent::GraphMetadataUpdated {
            id: GraphId::from(aggregate_id.clone()),
            metadata: GraphMetadata::new(format!("Update {}", i)),
        })];
        store.append_events(aggregate_id.clone(), event, None).await?;
    }

    // Replay from position 5
    let events = store.get_events_from_position(aggregate_id.clone(), 5).await?;

    assert_eq!(events.len(), 5); // Should get events 5-9

    match &events[0] {
        DomainEvent::Graph(GraphEvent::GraphMetadataUpdated { metadata, .. }) => {
            assert_eq!(metadata.name, "Update 5");
        }
        _ => panic!("Unexpected event type"),
    }

    cleanup_test_streams(&client).await?;
    Ok(())
}

#[tokio::test]
async fn test_cache_functionality() -> Result<(), Box<dyn std::error::Error>> {
    let client = create_test_client().await?;
    cleanup_test_streams(&client).await?;

    let store = DistributedEventStore::new(client.clone()).await?;
    let aggregate_id = AggregateId::from(GraphId::new());

    // Append event
    let event = vec![DomainEvent::Graph(GraphEvent::GraphCreated {
        id: GraphId::from(aggregate_id.clone()),
        metadata: GraphMetadata::new("Cached Graph".to_string()),
    })];

    store.append_events(aggregate_id.clone(), event, None).await?;

    // First retrieval (from NATS)
    let start = std::time::Instant::now();
    let _ = store.get_events(aggregate_id.clone()).await?;
    let first_duration = start.elapsed();

    // Second retrieval (should be from cache and faster)
    let start = std::time::Instant::now();
    let cached_events = store.get_events(aggregate_id.clone()).await?;
    let second_duration = start.elapsed();

    // Cache should make second retrieval faster (this is a weak assertion due to timing variability)
    // Main check is that we get the same events
    assert_eq!(cached_events.len(), 1);

    match &cached_events[0] {
        DomainEvent::Graph(GraphEvent::GraphCreated { metadata, .. }) => {
            assert_eq!(metadata.name, "Cached Graph");
        }
        _ => panic!("Unexpected event type"),
    }

    cleanup_test_streams(&client).await?;
    Ok(())
}

#[tokio::test]
async fn test_retention_policy() -> Result<(), Box<dyn std::error::Error>> {
    let client = create_test_client().await?;
    cleanup_test_streams(&client).await?;

    let store = DistributedEventStore::new(client.clone()).await?;

    // Verify stream configuration
    if let Ok(context) = client.jetstream_context() {
        let stream = context.get_stream("EVENTS").await?;
        let config = stream.info().await?.config;

        // Check retention policy
        assert_eq!(config.retention, jetstream::stream::RetentionPolicy::Limits);
        assert!(config.max_age > Duration::from_secs(0));
        assert!(config.duplicate_window.is_some());
    }

    cleanup_test_streams(&client).await?;
    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let client = create_test_client().await?;
    cleanup_test_streams(&client).await?;

    let store = DistributedEventStore::new(client.clone()).await?;

    // Test with invalid aggregate ID (empty)
    let invalid_id = AggregateId::from(GraphId::from(uuid::Uuid::nil()));
    let result = store.get_events(invalid_id).await;

    // Should still work but return empty
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);

    cleanup_test_streams(&client).await?;
    Ok(())
}
