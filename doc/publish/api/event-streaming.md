# Event Streaming

## NATS JetStream Integration

CIM leverages NATS JetStream for persistent event streaming, enabling reliable message delivery, event replay capabilities, and distributed system coordination. All event streaming operations provide durability, ordering guarantees, and at-least-once delivery semantics.

## Stream Configuration

### Creating Event Streams

#### Primary Event Stream
```rust
// Create the main CIM event stream
use async_nats::jetstream::{self, stream::Config as StreamConfig};

async fn create_cim_event_stream(js: &jetstream::Context) -> Result<(), Box<dyn std::error::Error>> {
    let stream_config = StreamConfig {
        name: "CIM_EVENTS".to_string(),
        description: Some("Primary event stream for CIM operations".to_string()),
        subjects: vec![
            "event.graph.>".to_string(),
            "event.conceptual.>".to_string(),
            "event.workflow.>".to_string(),
            "event.agent.>".to_string(),
            "event.system.>".to_string(),
        ],
        retention: jetstream::stream::RetentionPolicy::Limits,
        storage: jetstream::stream::StorageType::File,
        max_consumers: Some(100),
        max_msgs: Some(1_000_000),
        max_bytes: Some(10 * 1024 * 1024 * 1024), // 10GB
        max_age: std::time::Duration::from_secs(365 * 24 * 3600), // 1 year
        max_msg_size: Some(1024 * 1024), // 1MB per message
        duplicate_window: std::time::Duration::from_secs(120),
        ..Default::default()
    };

    let stream = js.create_stream(stream_config).await?;
    println!("Created stream: {}", stream.info().await?.config.name);
    Ok(())
}
```

#### Domain-Specific Streams
```rust
// Create specialized streams for different domains
async fn create_domain_streams(js: &jetstream::Context) -> Result<(), Box<dyn std::error::Error>> {
    // High-frequency graph operations stream
    let graph_stream = StreamConfig {
        name: "CIM_GRAPH_EVENTS".to_string(),
        subjects: vec!["event.graph.>".to_string()],
        retention: jetstream::stream::RetentionPolicy::WorkQueue,
        storage: jetstream::stream::StorageType::Memory, // Fast access
        max_age: std::time::Duration::from_secs(24 * 3600), // 1 day
        max_msgs: Some(10_000_000),
        ..Default::default()
    };

    // Long-term conceptual space evolution
    let conceptual_stream = StreamConfig {
        name: "CIM_CONCEPTUAL_EVENTS".to_string(),
        subjects: vec!["event.conceptual.>".to_string()],
        retention: jetstream::stream::RetentionPolicy::Limits,
        storage: jetstream::stream::StorageType::File, // Persistent
        max_age: std::time::Duration::from_secs(365 * 24 * 3600 * 5), // 5 years
        ..Default::default()
    };

    // Critical system events
    let system_stream = StreamConfig {
        name: "CIM_SYSTEM_EVENTS".to_string(),
        subjects: vec![
            "event.system.>".to_string(),
            "event.agent.registered".to_string(),
            "event.agent.deregistered".to_string(),
        ],
        retention: jetstream::stream::RetentionPolicy::Limits,
        storage: jetstream::stream::StorageType::File,
        max_age: std::time::Duration::from_secs(365 * 24 * 3600 * 10), // 10 years
        replicas: 3, // High availability for critical events
        ..Default::default()
    };

    js.create_stream(graph_stream).await?;
    js.create_stream(conceptual_stream).await?;
    js.create_stream(system_stream).await?;
    
    Ok(())
}
```

## Consumer Patterns

### Pull Consumer Configuration

#### Standard Event Processor
```rust
use async_nats::jetstream::consumer::{pull::Config as ConsumerConfig, DeliverPolicy};

async fn create_event_processor_consumer(
    js: &jetstream::Context,
    consumer_name: &str,
    filter_subjects: Vec<String>,
) -> Result<jetstream::consumer::Consumer<jetstream::consumer::pull::Config>, Box<dyn std::error::Error>> {
    let consumer_config = ConsumerConfig {
        name: Some(consumer_name.to_string()),
        durable_name: Some(consumer_name.to_string()),
        description: Some(format!("Event processor: {}", consumer_name)),
        filter_subjects,
        deliver_policy: DeliverPolicy::All,
        ack_policy: jetstream::consumer::AckPolicy::Explicit,
        ack_wait: std::time::Duration::from_secs(30),
        max_deliver: 3,
        max_ack_pending: 1000,
        replay_policy: jetstream::consumer::ReplayPolicy::Instant,
        ..Default::default()
    };

    let consumer = js
        .create_consumer_on_stream(consumer_config, "CIM_EVENTS")
        .await?;

    Ok(consumer)
}

// Usage example
async fn process_graph_events() -> Result<(), Box<dyn std::error::Error>> {
    let client = async_nats::connect("nats://localhost:4222").await?;
    let jetstream = async_nats::jetstream::new(client);

    let consumer = create_event_processor_consumer(
        &jetstream,
        "graph_event_processor",
        vec!["event.graph.>".to_string()],
    ).await?;

    let mut messages = consumer.messages().await?;

    while let Some(message) = messages.next().await {
        let message = message?;
        
        // Process the event
        match process_graph_event(&message.payload).await {
            Ok(_) => {
                message.ack().await?;
                println!("Processed event: {}", message.subject);
            }
            Err(e) => {
                eprintln!("Failed to process event: {}", e);
                message.nak().await?;
            }
        }
    }

    Ok(())
}
```

#### Batch Event Processing
```rust
async fn batch_event_processor() -> Result<(), Box<dyn std::error::Error>> {
    let client = async_nats::connect("nats://localhost:4222").await?;
    let jetstream = async_nats::jetstream::new(client);

    let consumer = create_event_processor_consumer(
        &jetstream,
        "batch_processor",
        vec!["event.>".to_string()],
    ).await?;

    loop {
        // Fetch batch of messages
        let batch_size = 100;
        let timeout = std::time::Duration::from_secs(1);
        
        let mut batch = consumer
            .batch()
            .max_messages(batch_size)
            .expires(timeout)
            .messages()
            .await?;

        let mut events = Vec::new();
        let mut messages_to_ack = Vec::new();

        // Collect batch
        while let Some(message) = batch.next().await {
            let message = message?;
            
            match serde_json::from_slice::<DomainEvent>(&message.payload) {
                Ok(event) => {
                    events.push(event);
                    messages_to_ack.push(message);
                }
                Err(e) => {
                    eprintln!("Failed to deserialize event: {}", e);
                    message.nak().await?;
                }
            }
        }

        if !events.is_empty() {
            // Process entire batch
            match process_event_batch(events).await {
                Ok(_) => {
                    // Acknowledge all messages in batch
                    for message in messages_to_ack {
                        message.ack().await?;
                    }
                    println!("Processed batch of {} events", messages_to_ack.len());
                }
                Err(e) => {
                    eprintln!("Failed to process batch: {}", e);
                    // NAK all messages for retry
                    for message in messages_to_ack {
                        message.nak().await?;
                    }
                }
            }
        }
    }
}
```

### Push Consumer for Real-Time Updates

#### Real-Time UI Updates
```rust
async fn create_realtime_ui_consumer(
    js: &jetstream::Context,
) -> Result<(), Box<dyn std::error::Error>> {
    let consumer_config = jetstream::consumer::push::Config {
        name: Some("realtime_ui_updates".to_string()),
        durable_name: Some("realtime_ui_updates".to_string()),
        description: Some("Real-time updates for UI components".to_string()),
        filter_subjects: vec![
            "event.graph.node_created".to_string(),
            "event.graph.node_updated".to_string(),
            "event.graph.node_deleted".to_string(),
            "event.graph.edge_created".to_string(),
            "event.graph.edge_deleted".to_string(),
        ],
        deliver_policy: DeliverPolicy::New, // Only new events
        deliver_subject: "ui.updates".to_string(),
        ack_policy: jetstream::consumer::AckPolicy::None, // Fire and forget
        replay_policy: jetstream::consumer::ReplayPolicy::Instant,
        ..Default::default()
    };

    js.create_consumer_on_stream(consumer_config, "CIM_EVENTS").await?;
    
    Ok(())
}

// Subscribe to UI updates
async fn handle_realtime_ui_updates() -> Result<(), Box<dyn std::error::Error>> {
    let client = async_nats::connect("nats://localhost:4222").await?;
    let mut subscriber = client.subscribe("ui.updates").await?;

    while let Some(message) = subscriber.next().await {
        let event: DomainEvent = serde_json::from_slice(&message.payload)?;
        
        // Update UI in real-time
        match event.event_type.as_str() {
            "graph.node_created" => update_ui_with_new_node(event).await?,
            "graph.node_updated" => update_ui_node_properties(event).await?,
            "graph.node_deleted" => remove_node_from_ui(event).await?,
            "graph.edge_created" => add_edge_to_ui(event).await?,
            "graph.edge_deleted" => remove_edge_from_ui(event).await?,
            _ => {}
        }
    }

    Ok(())
}
```

## Event Replay and Time Travel

### Historical Event Replay

#### Replay from Timestamp
```rust
async fn replay_events_from_timestamp(
    js: &jetstream::Context,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert to NATS timestamp format
    let start_nanos = start_time.timestamp_nanos_opt().unwrap() as u64;
    let end_nanos = end_time.map(|t| t.timestamp_nanos_opt().unwrap() as u64);

    let consumer_config = ConsumerConfig {
        name: Some("replay_consumer".to_string()),
        deliver_policy: DeliverPolicy::ByStartTime {
            start_time: std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_nanos(start_nanos),
        },
        replay_policy: jetstream::consumer::ReplayPolicy::Original, // Preserve timing
        filter_subjects: vec!["event.>".to_string()],
        ..Default::default()
    };

    let consumer = js
        .create_consumer_on_stream(consumer_config, "CIM_EVENTS")
        .await?;

    let mut messages = consumer.messages().await?;

    while let Some(message) = messages.next().await {
        let message = message?;
        let event: DomainEvent = serde_json::from_slice(&message.payload)?;

        // Check if we've reached the end time
        if let Some(end_nanos) = end_nanos {
            let event_time = event.timestamp.duration_since(std::time::UNIX_EPOCH)?.as_nanos() as u64;
            if event_time > end_nanos {
                break;
            }
        }

        // Replay the event
        replay_event_to_projection(event).await?;
        message.ack().await?;

        println!("Replayed event: {} at {:?}", event.event_type, event.timestamp);
    }

    Ok(())
}
```

#### Replay by Sequence Number
```rust
async fn replay_events_by_sequence(
    js: &jetstream::Context,
    start_sequence: u64,
    end_sequence: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let consumer_config = ConsumerConfig {
        name: Some("sequence_replay".to_string()),
        deliver_policy: DeliverPolicy::ByStartSequence { start_sequence },
        replay_policy: jetstream::consumer::ReplayPolicy::Instant, // As fast as possible
        filter_subjects: vec!["event.>".to_string()],
        ..Default::default()
    };

    let consumer = js
        .create_consumer_on_stream(consumer_config, "CIM_EVENTS")
        .await?;

    let mut messages = consumer.messages().await?;
    let mut processed_count = 0;

    while let Some(message) = messages.next().await {
        let message = message?;
        
        // Get message metadata
        let info = message.info().ok_or("No message info available")?;
        
        // Check if we've reached the end sequence
        if let Some(end_seq) = end_sequence {
            if info.stream_sequence > end_seq {
                break;
            }
        }

        let event: DomainEvent = serde_json::from_slice(&message.payload)?;
        
        // Apply event to rebuild state
        apply_event_to_aggregate(event).await?;
        message.ack().await?;

        processed_count += 1;
        if processed_count % 1000 == 0 {
            println!("Replayed {} events", processed_count);
        }
    }

    println!("Replay completed: {} events processed", processed_count);
    Ok(())
}
```

### Snapshot Creation and Restoration

#### Create System Snapshot
```rust
async fn create_system_snapshot(
    js: &jetstream::Context,
    snapshot_name: &str,
) -> Result<SnapshotMetadata, Box<dyn std::error::Error>> {
    // Get current stream information
    let stream_info = js.get_stream("CIM_EVENTS").await?.info().await?;
    let current_sequence = stream_info.state.last_sequence;
    let current_time = std::time::SystemTime::now();

    // Create snapshot metadata
    let snapshot = SnapshotMetadata {
        name: snapshot_name.to_string(),
        created_at: current_time,
        stream_sequence: current_sequence,
        stream_name: "CIM_EVENTS".to_string(),
        event_count: stream_info.state.messages,
        description: format!("System snapshot at sequence {}", current_sequence),
    };

    // Store snapshot metadata
    let snapshot_subject = format!("snapshot.metadata.{}", snapshot_name);
    let client = js.context().client();
    client.publish(
        snapshot_subject,
        serde_json::to_vec(&snapshot)?.into()
    ).await?;

    // Optionally create actual data snapshot
    create_projection_snapshots(&snapshot).await?;

    println!("Created snapshot '{}' at sequence {}", snapshot_name, current_sequence);
    Ok(snapshot)
}

async fn create_projection_snapshots(
    snapshot: &SnapshotMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    // Snapshot graph projections
    let graph_projection = get_current_graph_projection().await?;
    store_projection_snapshot("graph", &snapshot.name, &graph_projection).await?;

    // Snapshot conceptual space projections
    let conceptual_projection = get_current_conceptual_projection().await?;
    store_projection_snapshot("conceptual", &snapshot.name, &conceptual_projection).await?;

    // Snapshot workflow projections
    let workflow_projection = get_current_workflow_projection().await?;
    store_projection_snapshot("workflow", &snapshot.name, &workflow_projection).await?;

    Ok(())
}
```

#### Restore from Snapshot
```rust
async fn restore_from_snapshot(
    js: &jetstream::Context,
    snapshot_name: &str,
    replay_to_current: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load snapshot metadata
    let client = js.context().client();
    let snapshot_subject = format!("snapshot.metadata.{}", snapshot_name);
    
    let response = client.request(
        &format!("query.snapshot.get.{}", snapshot_name),
        "".into()
    ).await?;
    
    let snapshot: SnapshotMetadata = serde_json::from_slice(&response.payload)?;

    println!("Restoring from snapshot '{}' (sequence: {})", 
        snapshot.name, snapshot.stream_sequence);

    // Restore projection snapshots
    restore_projection_snapshots(&snapshot).await?;

    // Optionally replay events since snapshot
    if replay_to_current {
        let stream_info = js.get_stream(&snapshot.stream_name).await?.info().await?;
        let current_sequence = stream_info.state.last_sequence;

        if current_sequence > snapshot.stream_sequence {
            println!("Replaying {} events since snapshot", 
                current_sequence - snapshot.stream_sequence);
            
            replay_events_by_sequence(
                js,
                snapshot.stream_sequence + 1,
                Some(current_sequence),
            ).await?;
        }
    }

    println!("Snapshot restoration completed");
    Ok(())
}
```

## Stream Monitoring and Management

### Stream Health Monitoring

#### Monitor Stream Metrics
```rust
async fn monitor_stream_health(
    js: &jetstream::Context,
    stream_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let stream = js.get_stream(stream_name).await?;
        let info = stream.info().await?;

        let metrics = StreamMetrics {
            name: info.config.name.clone(),
            messages: info.state.messages,
            bytes: info.state.bytes,
            consumers: info.state.consumer_count,
            first_sequence: info.state.first_sequence,
            last_sequence: info.state.last_sequence,
            last_timestamp: info.state.last_time,
        };

        // Check for concerning metrics
        if metrics.messages > 500_000 {
            log::warn!("Stream {} has high message count: {}", 
                metrics.name, metrics.messages);
        }

        if metrics.bytes > 5 * 1024 * 1024 * 1024 { // 5GB
            log::warn!("Stream {} is using significant storage: {} GB", 
                metrics.name, metrics.bytes / (1024 * 1024 * 1024));
        }

        // Publish metrics for monitoring systems
        let client = js.context().client();
        client.publish(
            &format!("metrics.stream.{}", stream_name),
            serde_json::to_vec(&metrics)?.into()
        ).await?;

        // Wait before next check
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
}
```

#### Consumer Lag Monitoring
```rust
async fn monitor_consumer_lag(
    js: &jetstream::Context,
    stream_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let stream = js.get_stream(stream_name).await?;
    let consumer_names = stream.consumer_names().await?;

    for consumer_name in consumer_names {
        let consumer = stream.get_consumer(&consumer_name).await?;
        let info = consumer.info().await?;

        let lag = info.num_pending + info.num_redelivered;
        
        if lag > 10_000 {
            log::error!("Consumer {} has high lag: {} messages", 
                consumer_name, lag);
            
            // Publish alert
            let alert = ConsumerLagAlert {
                consumer_name: consumer_name.clone(),
                stream_name: stream_name.to_string(),
                lag,
                num_pending: info.num_pending,
                num_redelivered: info.num_redelivered,
                timestamp: std::time::SystemTime::now(),
            };

            let client = js.context().client();
            client.publish(
                "alert.consumer.high_lag",
                serde_json::to_vec(&alert)?.into()
            ).await?;
        }

        println!("Consumer {}: lag={}, pending={}, redelivered={}", 
            consumer_name, lag, info.num_pending, info.num_redelivered);
    }

    Ok(())
}
```

### Stream Maintenance

#### Purge Old Events
```rust
async fn purge_old_events(
    js: &jetstream::Context,
    stream_name: &str,
    keep_duration: std::time::Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let stream = js.get_stream(stream_name).await?;
    
    // Calculate cutoff time
    let cutoff_time = std::time::SystemTime::now() - keep_duration;
    
    // Purge events older than cutoff
    let purge_response = stream.purge()
        .filter(jetstream::stream::PurgeFilter::Subject("event.>".to_string()))
        .request()
        .await?;

    println!("Purged {} old events from stream {}", 
        purge_response.purged, stream_name);

    // Optionally compact stream
    // Note: NATS 2.9+ feature
    // stream.compact().await?;

    Ok(())
}
```

#### Stream Replication Management
```rust
async fn manage_stream_replication(
    js: &jetstream::Context,
    stream_name: &str,
    target_replicas: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = js.get_stream(stream_name).await?;
    let current_config = stream.info().await?.config;
    
    if current_config.replicas != target_replicas {
        let mut updated_config = current_config;
        updated_config.replicas = target_replicas;
        
        stream.update_config(updated_config).await?;
        
        println!("Updated stream {} replica count to {}", 
            stream_name, target_replicas);
    }

    Ok(())
}
```

## Performance Optimization

### Message Batching

#### Batch Event Publishing
```rust
async fn publish_events_in_batch(
    js: &jetstream::Context,
    events: Vec<DomainEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = js.context().client();
    
    // Group events by subject for efficiency
    let mut subject_groups: std::collections::HashMap<String, Vec<DomainEvent>> = 
        std::collections::HashMap::new();
    
    for event in events {
        let subject = format!("event.{}.{}", 
            event.aggregate_type.to_lowercase(), 
            event.event_type);
        
        subject_groups.entry(subject).or_default().push(event);
    }

    // Publish each group as a batch
    for (subject, group_events) in subject_groups {
        for chunk in group_events.chunks(100) { // Batch size of 100
            let futures: Vec<_> = chunk.iter().map(|event| {
                client.publish(
                    subject.clone(),
                    serde_json::to_vec(event).unwrap().into()
                )
            }).collect();

            // Wait for all publishes in chunk to complete
            futures::future::try_join_all(futures).await?;
        }
    }

    Ok(())
}
```

### Consumer Scaling

#### Auto-Scaling Consumers
```rust
async fn auto_scale_consumers(
    js: &jetstream::Context,
    base_consumer_name: &str,
    target_throughput: u64, // messages per second
) -> Result<(), Box<dyn std::error::Error>> {
    let stream = js.get_stream("CIM_EVENTS").await?;
    
    // Monitor current throughput
    let info = stream.info().await?;
    let current_rate = calculate_message_rate(&info).await?;
    
    // Determine required number of consumers
    let messages_per_consumer = 1000; // messages per second per consumer
    let required_consumers = (current_rate / messages_per_consumer).max(1) as usize;
    
    // Get existing consumers
    let existing_consumers = stream.consumer_names().await?;
    let current_consumers = existing_consumers.iter()
        .filter(|name| name.starts_with(base_consumer_name))
        .count();
    
    if required_consumers > current_consumers {
        // Scale up
        for i in current_consumers..required_consumers {
            let consumer_name = format!("{}-{}", base_consumer_name, i);
            
            let consumer_config = ConsumerConfig {
                name: Some(consumer_name.clone()),
                durable_name: Some(consumer_name),
                filter_subjects: vec!["event.>".to_string()],
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                max_ack_pending: 1000,
                ..Default::default()
            };
            
            js.create_consumer_on_stream(consumer_config, "CIM_EVENTS").await?;
            println!("Created additional consumer: {}-{}", base_consumer_name, i);
        }
    } else if required_consumers < current_consumers {
        // Scale down (delete excess consumers)
        for i in required_consumers..current_consumers {
            let consumer_name = format!("{}-{}", base_consumer_name, i);
            
            if let Ok(consumer) = stream.get_consumer(&consumer_name).await {
                consumer.delete().await?;
                println!("Deleted excess consumer: {}", consumer_name);
            }
        }
    }

    Ok(())
}
```

---

**NATS JetStream provides CIM with reliable, scalable, and persistent event streaming capabilities that enable sophisticated event sourcing, real-time collaboration, and distributed system coordination across all CIM operations.** 