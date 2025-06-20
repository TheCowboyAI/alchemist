//! Infrastructure Test: NATS JetStream Connection
//! 
//! User Story: As a system, I need to connect to NATS JetStream and create event streams
//! 
//! This test validates:
//! 1. NATS connection establishment
//! 2. Stream creation with correct configuration  
//! 3. Event publishing with acknowledgment
//! 4. Event consumption with proper ordering

use async_nats::jetstream;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

/// Test event for validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestEvent {
    event_id: Uuid,
    event_type: String,
    sequence: u64,
    timestamp: String,
    payload: String,
}

/// Expected event sequence for this test
const EXPECTED_SEQUENCE: &[&str] = &[
    "ConnectionEstablished",
    "StreamCreated",
    "EventPublished",
    "EventConsumed",
];

#[tokio::test]
async fn test_nats_connection_and_event_flow() {
    println!("🧪 Infrastructure Test: NATS JetStream Connection\n");
    
    let mut captured_events = Vec::new();
    
    // Step 1: Connect to NATS
    println!("📡 Connecting to NATS...");
    let client = async_nats::connect("nats://localhost:4222")
        .await
        .expect("Failed to connect to NATS");
    
    captured_events.push("ConnectionEstablished");
    println!("✅ Connection established");
    
    // Step 2: Create JetStream context and stream
    println!("\n📊 Creating JetStream stream...");
    let jetstream = jetstream::new(client);
    
    let stream_config = jetstream::stream::Config {
        name: "TEST-INFRASTRUCTURE".to_string(),
        subjects: vec!["test.infrastructure.>".to_string()],
        retention: jetstream::stream::RetentionPolicy::Limits,
        max_messages: 1000,
        ..Default::default()
    };
    
    let stream = jetstream
        .create_stream(stream_config)
        .await
        .expect("Failed to create stream");
    
    captured_events.push("StreamCreated");
    println!("✅ Stream created: {}", stream.info().await.unwrap().config.name);
    
    // Step 3: Publish test events
    println!("\n📤 Publishing test events...");
    let mut published_events = Vec::new();
    
    for i in 0..3 {
        let event = TestEvent {
            event_id: Uuid::new_v4(),
            event_type: "TestEvent".to_string(),
            sequence: i + 1,
            timestamp: Utc::now().to_rfc3339(),
            payload: format!("Test payload {}", i + 1),
        };
        
        let subject = format!("test.infrastructure.event.{}", i);
        let payload = serde_json::to_vec(&event).unwrap();
        
        let ack = jetstream
            .publish(subject.clone(), payload.into())
            .await
            .expect("Failed to publish")
            .await
            .expect("Failed to get ack");
        
        println!("  ✅ Published event {} (seq: {})", event.event_id, ack.sequence);
        published_events.push(event);
    }
    
    captured_events.push("EventPublished");
    
    // Step 4: Consume events and verify ordering
    println!("\n📥 Consuming events...");
    let consumer = stream
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("test-consumer".to_string()),
            deliver_policy: jetstream::consumer::DeliverPolicy::All,
            ..Default::default()
        })
        .await
        .expect("Failed to create consumer");
    
    let mut messages = consumer
        .fetch()
        .max_messages(3)
        .messages()
        .await
        .expect("Failed to fetch messages");
    
    let mut consumed_events = Vec::new();
    let mut sequence_numbers = Vec::new();
    
    while let Some(msg) = messages.next().await {
        match msg {
            Ok(msg) => {
                let event: TestEvent = serde_json::from_slice(&msg.payload)
                    .expect("Failed to deserialize event");
                
                println!("  ✅ Consumed event {} (seq: {})", 
                    event.event_id, 
                    event.sequence
                );
                
                sequence_numbers.push(event.sequence);
                consumed_events.push(event);
            }
            Err(e) => {
                panic!("Error consuming message: {}", e);
            }
        }
    }
    
    captured_events.push("EventConsumed");
    
    // Validate results
    println!("\n🔍 Validating event sequence...");
    
    // Check sequence matches expected
    assert_eq!(
        captured_events,
        EXPECTED_SEQUENCE,
        "Event sequence mismatch"
    );
    println!("  ✅ Event sequence matches expected");
    
    // Check all published events were consumed
    assert_eq!(
        published_events.len(),
        consumed_events.len(),
        "Not all events were consumed"
    );
    println!("  ✅ All published events were consumed");
    
    // Check ordering is preserved
    assert_eq!(
        sequence_numbers,
        vec![1, 2, 3],
        "Event ordering was not preserved"
    );
    println!("  ✅ Event ordering preserved");
    
    // Check event integrity
    for (published, consumed) in published_events.iter().zip(consumed_events.iter()) {
        assert_eq!(
            published.payload, consumed.payload,
            "Event payload mismatch"
        );
    }
    println!("  ✅ Event integrity verified");
    
    // Cleanup
    consumer.delete().await.ok();
    stream.delete().await.ok();
    
    println!("\n✨ Infrastructure test passed!");
    println!("\n📊 Event Stream Summary:");
    println!("  - Total events in sequence: {}", captured_events.len());
    println!("  - Events published: {}", published_events.len());
    println!("  - Events consumed: {}", consumed_events.len());
    println!("  - Sequence validated: ✅");
}

#[tokio::test]
async fn test_event_persistence_and_replay() {
    println!("🧪 Infrastructure Test: Event Persistence and Replay\n");
    
    // Connect and create stream
    let client = async_nats::connect("nats://localhost:4222")
        .await
        .expect("Failed to connect");
    
    let jetstream = jetstream::new(client);
    
    let stream = jetstream
        .get_or_create_stream(jetstream::stream::Config {
            name: "TEST-PERSISTENCE".to_string(),
            subjects: vec!["test.persist.>".to_string()],
            retention: jetstream::stream::RetentionPolicy::Limits,
            ..Default::default()
        })
        .await
        .expect("Failed to create stream");
    
    // Publish events
    println!("📤 Publishing events for persistence test...");
    let mut event_ids = Vec::new();
    
    for i in 0..5 {
        let event = TestEvent {
            event_id: Uuid::new_v4(),
            event_type: "PersistenceTest".to_string(),
            sequence: i + 1,
            timestamp: Utc::now().to_rfc3339(),
            payload: format!("Persistence test {}", i + 1),
        };
        
        event_ids.push(event.event_id);
        
        jetstream
            .publish(
                "test.persist.event",
                serde_json::to_vec(&event).unwrap().into(),
            )
            .await
            .expect("Failed to publish")
            .await
            .expect("Failed to get ack");
    }
    
    println!("✅ Published {} events", event_ids.len());
    
    // Create consumer from beginning
    println!("\n🔄 Replaying events from beginning...");
    let consumer = stream
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("replay-consumer".to_string()),
            deliver_policy: jetstream::consumer::DeliverPolicy::All,
            ..Default::default()
        })
        .await
        .expect("Failed to create consumer");
    
    // Replay all events
    let mut messages = consumer
        .fetch()
        .max_messages(10)
        .messages()
        .await
        .expect("Failed to fetch");
    
    let mut replayed_count = 0;
    let mut replayed_ids = Vec::new();
    
    while let Some(msg) = messages.next().await {
        if let Ok(msg) = msg {
            let event: TestEvent = serde_json::from_slice(&msg.payload)
                .expect("Failed to deserialize");
            
            replayed_ids.push(event.event_id);
            replayed_count += 1;
        }
    }
    
    // Validate replay
    assert_eq!(
        replayed_count,
        event_ids.len(),
        "Not all events were replayed"
    );
    
    assert_eq!(
        replayed_ids,
        event_ids,
        "Replayed events don't match original"
    );
    
    println!("✅ Successfully replayed {} events", replayed_count);
    println!("✅ Event persistence and replay validated");
    
    // Cleanup
    consumer.delete().await.ok();
    stream.delete().await.ok();
}

#[tokio::test] 
async fn test_concurrent_event_publishing() {
    println!("🧪 Infrastructure Test: Concurrent Event Publishing\n");
    
    let client = async_nats::connect("nats://localhost:4222")
        .await
        .expect("Failed to connect");
    
    let jetstream = jetstream::new(client);
    
    // Create stream
    let _stream = jetstream
        .get_or_create_stream(jetstream::stream::Config {
            name: "TEST-CONCURRENT".to_string(),
            subjects: vec!["test.concurrent.>".to_string()],
            ..Default::default()
        })
        .await
        .expect("Failed to create stream");
    
    println!("🚀 Publishing events concurrently...");
    
    // Publish events from multiple tasks
    let mut handles = Vec::new();
    
    for task_id in 0..3 {
        let js = jetstream.clone();
        
        let handle = tokio::spawn(async move {
            let mut published = Vec::new();
            
            for i in 0..5 {
                let event = TestEvent {
                    event_id: Uuid::new_v4(),
                    event_type: format!("Task{}", task_id),
                    sequence: i + 1,
                    timestamp: Utc::now().to_rfc3339(),
                    payload: format!("Task {} Event {}", task_id, i + 1),
                };
                
                let ack = js
                    .publish(
                        format!("test.concurrent.task{}", task_id),
                        serde_json::to_vec(&event).unwrap().into(),
                    )
                    .await
                    .expect("Failed to publish")
                    .await
                    .expect("Failed to get ack");
                
                published.push((event.event_id, ack.sequence));
            }
            
            published
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks
    let mut total_published = 0;
    for handle in handles {
        let published = handle.await.expect("Task failed");
        total_published += published.len();
    }
    
    println!("✅ Published {} events concurrently", total_published);
    
    // Verify all events were persisted
    let info = jetstream
        .stream_by_subject("test.concurrent.>")
        .await
        .expect("Failed to get stream")
        .info()
        .await
        .expect("Failed to get info");
    
    assert_eq!(
        info.state.messages,
        total_published as u64,
        "Not all concurrent events were persisted"
    );
    
    println!("✅ All concurrent events persisted successfully");
    println!("✅ Concurrent publishing validated");
} 