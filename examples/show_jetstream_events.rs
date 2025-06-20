//! Example showing actual events persisted to NATS JetStream
//!
//! This demonstrates:
//! 1. Events with proper subjects (e.g., events.graph.node.added)
//! 2. Correlation IDs linking related events
//! 3. Causation IDs showing event chains
//! 4. CID chains for cryptographic integrity

use async_nats::jetstream;
use chrono::Utc;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DomainEvent {
    // Event identity
    event_id: Uuid,
    aggregate_id: Uuid,
    sequence: u64,

    // Event metadata
    event_type: String,
    timestamp: String,

    // Correlation and causation
    correlation_id: Uuid,
    causation_id: Option<Uuid>,

    // CID chain
    event_cid: Option<String>,
    previous_cid: Option<String>,

    // Payload
    payload: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CIM JetStream Event Viewer ===\n");

    // Connect to NATS
    let client = async_nats::connect("nats://localhost:4222").await?;
    println!("✅ Connected to NATS");

    // Get JetStream context
    let jetstream = jetstream::new(client);

    // List all streams
    println!("\n📊 Available Event Streams:");
    let mut streams = jetstream.streams();

    while let Some(stream) = streams.next().await {
        match stream {
            Ok(stream_info) => {
                println!("\n  Stream: {}", stream_info.config.name);
                println!("  Subjects: {:?}", stream_info.config.subjects);
                println!("  Messages: {}", stream_info.state.messages);
                println!("  Bytes: {}", stream_info.state.bytes);

                // Get the event-store stream
                if stream_info.config.name == "event-store"
                    || stream_info.config.name.contains("EVENT")
                {
                    println!(
                        "\n🔍 Examining events in stream '{}':",
                        stream_info.config.name
                    );

                    // Get the stream handle
                    let stream = jetstream.get_stream(&stream_info.config.name).await?;

                    // Create a consumer to read all messages
                    let consumer = stream
                        .create_consumer(jetstream::consumer::pull::Config {
                            durable_name: Some("event-viewer".to_string()),
                            deliver_policy: jetstream::consumer::DeliverPolicy::All,
                            ..Default::default()
                        })
                        .await?;

                    // Fetch messages
                    let mut messages = consumer.fetch().max_messages(100).messages().await?;

                    let mut event_count = 0;
                    let mut correlation_groups = std::collections::HashMap::new();

                    println!("\n📨 Events (newest first):");
                    println!("{}", "=".repeat(80));

                    while let Some(msg) = messages.next().await {
                        match msg {
                            Ok(msg) => {
                                event_count += 1;

                                // Parse the event
                                if let Ok(event) =
                                    serde_json::from_slice::<DomainEvent>(&msg.payload)
                                {
                                    println!("\nEvent #{}", event_count);
                                    println!("  Subject: {}", msg.subject);
                                    println!("  Event ID: {}", event.event_id);
                                    println!("  Type: {}", event.event_type);
                                    println!("  Aggregate: {}", event.aggregate_id);
                                    println!("  Sequence: {}", event.sequence);
                                    println!("  Timestamp: {}", event.timestamp);
                                    println!("  Correlation ID: {}", event.correlation_id);

                                    if let Some(causation_id) = &event.causation_id {
                                        println!(
                                            "  Causation ID: {} (caused by another event)",
                                            causation_id
                                        );
                                    } else {
                                        println!("  Causation ID: None (root event)");
                                    }

                                    if let Some(cid) = &event.event_cid {
                                        println!("  Event CID: {}", cid);
                                    }

                                    if let Some(prev_cid) = &event.previous_cid {
                                        println!("  Previous CID: {} (chain link)", prev_cid);
                                    }

                                    println!(
                                        "  Payload: {}",
                                        serde_json::to_string_pretty(&event.payload)?
                                    );

                                    // Track correlation groups
                                    correlation_groups
                                        .entry(event.correlation_id)
                                        .or_insert_with(Vec::new)
                                        .push(event.event_type.clone());
                                } else {
                                    // Try to show raw message if not a DomainEvent
                                    println!("\nMessage #{} (Raw)", event_count);
                                    println!("  Subject: {}", msg.subject);
                                    println!("  Headers: {:?}", msg.headers);

                                    // Try to parse as JSON
                                    if let Ok(json) =
                                        serde_json::from_slice::<serde_json::Value>(&msg.payload)
                                    {
                                        println!(
                                            "  Payload: {}",
                                            serde_json::to_string_pretty(&json)?
                                        );
                                    } else {
                                        println!(
                                            "  Payload: {:?}",
                                            String::from_utf8_lossy(&msg.payload)
                                        );
                                    }
                                }

                                println!("{}", "-".repeat(80));
                            }
                            Err(e) => {
                                eprintln!("Error reading message: {}", e);
                                break;
                            }
                        }
                    }

                    // Show correlation analysis
                    if !correlation_groups.is_empty() {
                        println!("\n🔗 Event Correlation Groups:");
                        for (correlation_id, events) in correlation_groups.iter() {
                            println!("\n  Correlation ID: {}", correlation_id);
                            println!("  Related Events: {}", events.len());
                            for (i, event_type) in events.iter().enumerate() {
                                println!("    {}. {}", i + 1, event_type);
                            }
                        }
                    }

                    println!("\n📊 Summary:");
                    println!("  Total Events: {}", event_count);
                    println!("  Correlation Groups: {}", correlation_groups.len());

                    // Note: Consumer will be cleaned up automatically
                }
            }
            Err(e) => {
                eprintln!("Error listing stream: {}", e);
            }
        }
    }

    // Also check for specific CIM event subjects
    println!("\n🎯 Checking CIM Event Subjects:");
    let cim_subjects = vec![
        "events.graph.>",
        "events.node.>",
        "events.edge.>",
        "events.person.>",
        "events.agent.>",
        "events.workflow.>",
        "events.conceptual.>",
        "events.identity.>",
    ];

    for subject in cim_subjects {
        println!("\n  Checking subject pattern: {}", subject);

        // Try to get stream info for this subject
        match jetstream.stream_by_subject(subject).await {
            Ok(stream_name) => {
                let mut stream = jetstream.get_stream(&stream_name).await?;
                let info = stream.info().await?;
                println!("    ✅ Found in stream: {}", info.config.name);
                println!("    Messages matching: {}", info.state.messages);
            }
            Err(_) => {
                println!("    ❌ No stream found for this subject");
            }
        }
    }

    println!("\n✨ Event inspection complete!");

    Ok(())
}
