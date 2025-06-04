# NATS EventStream Transactions and Bevy Integration

## Overview

This document describes how we fetch portions of the event store from NATS as transactional event streams and connect them to Bevy's event system for real-time graph updates. Event streams can transform existing graphs or add new nodes and edges based on their content and context.

Our system fetches portions of the event store as transactional units, allowing for coherent batches of events to be processed together. Events are stored in NATS JetStream with CIDs calculated using IPLD dag-cbor format, incorporating the previous CID to create a verifiable chain.

## CID Calculation

### Event CID Chain

Each event's CID is calculated using IPLD dag-cbor encoding of:
- The event payload
- The previous CID in the aggregate's chain
- The aggregate ID
- The event type
- The timestamp

```rust
use ipld::codec::Codec;
use ipld_dagcbor::DagCborCodec;
use cid::Cid;

/// Structure used for CID calculation
#[derive(Serialize, Deserialize)]
struct EventForCid {
    /// The actual event payload
    payload: Vec<u8>,
    /// Previous CID in the chain (if any)
    previous_cid: Option<Cid>,
    /// Aggregate this event belongs to
    aggregate_id: AggregateId,
    /// Type of event
    event_type: String,
    /// When the event occurred
    timestamp: SystemTime,
}

/// Calculate CID for an event
fn calculate_event_cid(
    payload: &[u8],
    previous_cid: Option<Cid>,
    aggregate_id: &AggregateId,
    event_type: &str,
) -> Result<Cid, Error> {
    let event_for_cid = EventForCid {
        payload: payload.to_vec(),
        previous_cid,
        aggregate_id: aggregate_id.clone(),
        event_type: event_type.to_string(),
        timestamp: SystemTime::now(),
    };

    // Encode using dag-cbor
    let encoded = DagCborCodec.encode(&event_for_cid)?;

    // Create CID with dag-cbor codec (0x71)
    let cid = Cid::new_v1(0x71, multihash::Code::Sha2_256.digest(&encoded));

    Ok(cid)
}
```

This ensures:
- **Content Addressing**: CID uniquely identifies the event content
- **Chain Integrity**: Previous CID inclusion creates an immutable chain
- **IPLD Compatibility**: dag-cbor format works with IPLD ecosystem
- **Verifiability**: Anyone can recalculate and verify the CID

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    NATS JetStream                            │
│  ┌──────────────────┐        ┌──────────────────┐          │
│  │ Event Store      │───────►│ Consumer Groups  │          │
│  │ (Persistent)     │        │ (Transactional)  │          │
│  └──────────────────┘        └──────────────────┘          │
└─────────────────────────────────────────────────────────────┘
                                        │
                              ┌─────────┴─────────┐
                              │ EventStream       │
                              │ Transaction       │
                              └─────────┬─────────┘
                                        │
┌─────────────────────────────┴─────────────────────────────┐
│                 EventStream Bridge                          │
│  ┌──────────────┐    ┌──────────────┐   ┌──────────────┐ │
│  │ Stream Cache │    │ Event Parser │   │ Graph Mapper │ │
│  └──────────────┘    └──────────────┘   └──────────────┘ │
└─────────────────────────────┬─────────────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    │   Bevy Events     │
                    │  (ECS Updates)    │
                    └───────────────────┘
```

## Core Components

### 1. EventStream Transaction Model

```rust
use async_nats::jetstream;
use bevy::prelude::*;
use cid::Cid;

/// Represents a transactional slice of the event store
#[derive(Debug, Clone)]
pub struct EventStreamTransaction {
    /// Unique transaction ID
    pub transaction_id: TransactionId,
    /// Range of events in this transaction
    pub sequence_range: SequenceRange,
    /// Aggregate this transaction affects
    pub aggregate_id: AggregateId,
    /// Events in order
    pub events: Vec<DomainEvent>,
    /// Transaction metadata
    pub metadata: TransactionMetadata,
}

#[derive(Debug, Clone)]
pub struct SequenceRange {
    pub start: u64,
    pub end: u64,
    pub stream_name: String,
}

#[derive(Debug, Clone)]
pub struct TransactionMetadata {
    pub fetched_at: SystemTime,
    pub consumer_name: String,
    pub filter_subject: Option<String>,
    pub replay_policy: ReplayPolicy,
}

/// Service for fetching event stream transactions
pub struct EventStreamService {
    jetstream: jetstream::Context,
    transaction_cache: Arc<RwLock<HashMap<TransactionId, EventStreamTransaction>>>,
    /// Track latest CIDs for chain verification
    latest_cids: Arc<RwLock<HashMap<AggregateId, Cid>>>,
}

impl EventStreamService {
    /// Fetch a transaction of events for an aggregate
    pub async fn fetch_transaction(
        &self,
        aggregate_id: AggregateId,
        options: TransactionOptions,
    ) -> Result<EventStreamTransaction, EventStreamError> {
        let consumer = self.create_transactional_consumer(&aggregate_id, &options).await?;

        let mut events = Vec::new();
        let mut sequence_start = None;
        let mut sequence_end = None;

        // Fetch batch of messages
        let messages = consumer
            .fetch()
            .max_messages(options.max_events.unwrap_or(1000))
            .expires(options.timeout.unwrap_or(Duration::from_secs(5)))
            .messages()
            .await?;

        for msg in messages {
            let seq = msg.info()?.stream_sequence;
            if sequence_start.is_none() {
                sequence_start = Some(seq);
            }
            sequence_end = Some(seq);

            // Parse event from message with CID verification
            let event = self.parse_and_verify_event(msg).await?;
            events.push(event);

            msg.ack().await?;
        }

        let transaction = EventStreamTransaction {
            transaction_id: TransactionId::new(),
            sequence_range: SequenceRange {
                start: sequence_start.unwrap_or(0),
                end: sequence_end.unwrap_or(0),
                stream_name: "event-store".to_string(),
            },
            aggregate_id,
            events,
            metadata: TransactionMetadata {
                fetched_at: SystemTime::now(),
                consumer_name: consumer.name,
                filter_subject: options.filter_subject,
                replay_policy: options.replay_policy,
            },
        };

        // Cache transaction
        self.transaction_cache.write().await
            .insert(transaction.transaction_id.clone(), transaction.clone());

        Ok(transaction)
    }

    /// Parse event and verify CID chain
    async fn parse_and_verify_event(&self, msg: Message) -> Result<DomainEvent, EventStreamError> {
        let headers = &msg.headers;
        let cid = Cid::from_str(headers.get("Cid").unwrap())?;
        let previous_cid = headers.get("Previous-Cid")
            .and_then(|s| Cid::from_str(s).ok());

        // Decode event
        let event: DomainEvent = DagCborCodec.decode(&msg.payload)?;

        // Verify CID chain
        if let Some(expected_prev) = self.latest_cids.read().await.get(&event.aggregate_id) {
            if previous_cid != Some(expected_prev.clone()) {
                return Err(EventStreamError::ChainBroken {
                    expected: expected_prev.clone(),
                    actual: previous_cid,
                });
            }
        }

        // Update latest CID
        self.latest_cids.write().await.insert(event.aggregate_id.clone(), cid.clone());

        Ok(event)
    }

    /// Fetch events within a time window
    pub async fn fetch_time_window(
        &self,
        start_time: SystemTime,
        end_time: SystemTime,
        filter: Option<EventFilter>,
    ) -> Result<EventStreamTransaction, EventStreamError> {
        let options = TransactionOptions {
            replay_policy: ReplayPolicy::ByTime { start_time },
            max_events: None,
            timeout: Some(Duration::from_secs(30)),
            filter_subject: filter.map(|f| f.to_subject()),
        };

        // Create ephemeral consumer for time-based query
        let consumer = self.jetstream
            .create_consumer(
                "event-store",
                jetstream::consumer::Config {
                    deliver_policy: jetstream::consumer::DeliverPolicy::ByStartTime {
                        start_time: start_time.into(),
                    },
                    ..Default::default()
                },
            )
            .await?;

        // Fetch events until end_time
        let mut events = Vec::new();
        loop {
            let messages = consumer.fetch().messages().await?;

            let mut done = false;
            for msg in messages {
                let event = self.parse_and_verify_event(msg).await?;

                if event.timestamp > end_time {
                    done = true;
                    break;
                }

                events.push(event);
                msg.ack().await?;
            }

            if done || events.is_empty() {
                break;
            }
        }

        Ok(EventStreamTransaction {
            transaction_id: TransactionId::new(),
            sequence_range: SequenceRange {
                start: events.first().map(|e| e.sequence).unwrap_or(0),
                end: events.last().map(|e| e.sequence).unwrap_or(0),
                stream_name: "event-store".to_string(),
            },
            aggregate_id: AggregateId::default(), // Multiple aggregates possible
            events,
            metadata: TransactionMetadata {
                fetched_at: SystemTime::now(),
                consumer_name: consumer.name,
                filter_subject: filter.map(|f| f.to_subject()),
                replay_policy: ReplayPolicy::ByTime { start_time },
            },
        })
    }
}
```

### 2. Bevy Event Bridge

```rust
/// Bevy event that represents a graph mutation from NATS
#[derive(Event, Debug, Clone)]
pub struct GraphMutationEvent {
    pub source: EventSource,
    pub mutation: GraphMutation,
    pub transaction_id: Option<TransactionId>,
}

#[derive(Debug, Clone)]
pub enum EventSource {
    Nats { sequence: u64, subject: String },
    Local,
    Replay { original_sequence: u64 },
}

#[derive(Debug, Clone)]
pub enum GraphMutation {
    AddNode {
        node_id: NodeId,
        node_type: NodeType,
        properties: HashMap<String, Value>,
    },
    AddEdge {
        edge_id: EdgeId,
        source: NodeId,
        target: NodeId,
        edge_type: EdgeType,
        properties: HashMap<String, Value>,
    },
    UpdateNode {
        node_id: NodeId,
        updates: HashMap<String, Value>,
    },
    RemoveNode {
        node_id: NodeId,
    },
    RemoveEdge {
        edge_id: EdgeId,
    },
    Transform {
        target: TransformTarget,
        operation: TransformOperation,
    },
}

/// System that bridges NATS events to Bevy events
pub fn nats_event_bridge_system(
    event_stream_service: Res<EventStreamService>,
    mut graph_events: EventWriter<GraphMutationEvent>,
    mut last_sequence: Local<HashMap<String, u64>>,
) {
    // Poll for new events from active subscriptions
    if let Ok(new_events) = event_stream_service.poll_new_events() {
        for (subject, events) in new_events {
            let last_seq = last_sequence.get(&subject).copied().unwrap_or(0);

            for event in events.iter().filter(|e| e.sequence > last_seq) {
                // Convert domain event to graph mutation
                if let Some(mutation) = convert_to_graph_mutation(&event) {
                    graph_events.send(GraphMutationEvent {
                        source: EventSource::Nats {
                            sequence: event.sequence,
                            subject: subject.clone(),
                        },
                        mutation,
                        transaction_id: event.transaction_id.clone(),
                    });
                }

                last_sequence.insert(subject.clone(), event.sequence);
            }
        }
    }
}

/// Convert domain events to graph mutations
fn convert_to_graph_mutation(event: &DomainEvent) -> Option<GraphMutation> {
    match event.event_type.as_str() {
        "NodeCreated" => {
            let payload: NodeCreatedPayload = serde_json::from_value(&event.payload).ok()?;
            Some(GraphMutation::AddNode {
                node_id: payload.node_id,
                node_type: payload.node_type,
                properties: payload.properties,
            })
        }
        "EdgeConnected" => {
            let payload: EdgeConnectedPayload = serde_json::from_value(&event.payload).ok()?;
            Some(GraphMutation::AddEdge {
                edge_id: payload.edge_id,
                source: payload.source,
                target: payload.target,
                edge_type: payload.edge_type,
                properties: payload.properties,
            })
        }
        "NodeTransformed" => {
            let payload: NodeTransformedPayload = serde_json::from_value(&event.payload).ok()?;
            Some(GraphMutation::Transform {
                target: TransformTarget::Node(payload.node_id),
                operation: payload.operation,
            })
        }
        _ => None,
    }
}
```

### 3. Graph Update Systems

```rust
/// System that applies graph mutations to the ECS world
pub fn apply_graph_mutations_system(
    mut commands: Commands,
    mut graph_events: EventReader<GraphMutationEvent>,
    mut graph_query: Query<(Entity, &mut GraphNode, &mut Transform)>,
    edge_query: Query<(Entity, &GraphEdge)>,
    mut graph_model: ResMut<GraphModel>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in graph_events.read() {
        match &event.mutation {
            GraphMutation::AddNode { node_id, node_type, properties } => {
                // Update graph model
                let node_index = graph_model.add_node(node_id.clone(), node_type.clone());

                // Create visual entity
                let entity = spawn_node_visual(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    node_id,
                    node_type,
                    properties,
                );

                // Link to graph model
                commands.entity(entity).insert(GraphNode {
                    node_id: node_id.clone(),
                    graph_index: node_index,
                });
            }

            GraphMutation::AddEdge { edge_id, source, target, edge_type, properties } => {
                // Find source and target entities
                let source_entity = find_node_entity(&graph_query, source);
                let target_entity = find_node_entity(&graph_query, target);

                if let (Some(source_e), Some(target_e)) = (source_entity, target_entity) {
                    // Update graph model
                    let edge_index = graph_model.add_edge(source, target, edge_type.clone());

                    // Create visual edge
                    let edge_entity = spawn_edge_visual(
                        &mut commands,
                        source_e,
                        target_e,
                        edge_type,
                        properties,
                    );

                    commands.entity(edge_entity).insert(GraphEdge {
                        edge_id: edge_id.clone(),
                        graph_index: edge_index,
                    });
                }
            }

            GraphMutation::Transform { target, operation } => {
                apply_transform_operation(
                    &mut graph_query,
                    &graph_model,
                    target,
                    operation,
                );
            }

            // Handle other mutations...
        }
    }
}

/// Apply transform operations to graph elements
fn apply_transform_operation(
    graph_query: &mut Query<(Entity, &mut GraphNode, &mut Transform)>,
    graph_model: &GraphModel,
    target: &TransformTarget,
    operation: &TransformOperation,
) {
    match operation {
        TransformOperation::Translate { delta } => {
            match target {
                TransformTarget::Node(node_id) => {
                    if let Some((_, _, mut transform)) = find_node_mut(graph_query, node_id) {
                        transform.translation += *delta;
                    }
                }
                TransformTarget::Subgraph(nodes) => {
                    for node_id in nodes {
                        if let Some((_, _, mut transform)) = find_node_mut(graph_query, node_id) {
                            transform.translation += *delta;
                        }
                    }
                }
            }
        }

        TransformOperation::Scale { factor } => {
            match target {
                TransformTarget::Node(node_id) => {
                    if let Some((_, _, mut transform)) = find_node_mut(graph_query, node_id) {
                        transform.scale *= *factor;
                    }
                }
                _ => {}
            }
        }

        TransformOperation::Rotate { rotation } => {
            match target {
                TransformTarget::Node(node_id) => {
                    if let Some((_, _, mut transform)) = find_node_mut(graph_query, node_id) {
                        transform.rotation = transform.rotation * *rotation;
                    }
                }
                _ => {}
            }
        }

        TransformOperation::Reparent { new_parent } => {
            // Update graph hierarchy
            // ...
        }
    }
}
```

### 4. Transaction Replay System

```rust
/// Component for tracking transaction replay state
#[derive(Component)]
pub struct TransactionReplay {
    pub transaction_id: TransactionId,
    pub current_event: usize,
    pub total_events: usize,
    pub replay_speed: f32,
    pub timer: Timer,
}

/// System for replaying event transactions
pub fn replay_transaction_system(
    mut commands: Commands,
    time: Res<Time>,
    mut replay_query: Query<(Entity, &mut TransactionReplay)>,
    transaction_cache: Res<Arc<RwLock<HashMap<TransactionId, EventStreamTransaction>>>>,
    mut graph_events: EventWriter<GraphMutationEvent>,
) {
    for (entity, mut replay) in replay_query.iter_mut() {
        replay.timer.tick(time.delta());

        if replay.timer.finished() && replay.current_event < replay.total_events {
            // Get transaction from cache
            if let Ok(cache) = transaction_cache.read() {
                if let Some(transaction) = cache.get(&replay.transaction_id) {
                    if let Some(event) = transaction.events.get(replay.current_event) {
                        // Convert and send as graph mutation
                        if let Some(mutation) = convert_to_graph_mutation(event) {
                            graph_events.send(GraphMutationEvent {
                                source: EventSource::Replay {
                                    original_sequence: event.sequence,
                                },
                                mutation,
                                transaction_id: Some(transaction.transaction_id.clone()),
                            });
                        }

                        replay.current_event += 1;
                        replay.timer.reset();
                    }
                }
            }

            // Remove component when done
            if replay.current_event >= replay.total_events {
                commands.entity(entity).remove::<TransactionReplay>();
            }
        }
    }
}

/// Start replaying a transaction
pub fn start_transaction_replay(
    commands: &mut Commands,
    transaction_id: TransactionId,
    replay_speed: f32,
) -> Entity {
    commands.spawn(TransactionReplay {
        transaction_id,
        current_event: 0,
        total_events: 0, // Will be set from transaction
        replay_speed,
        timer: Timer::from_seconds(1.0 / replay_speed, TimerMode::Repeating),
    }).id()
}
```

### 5. Real-time Subscription System

```rust
/// Resource for managing NATS subscriptions
#[derive(Resource)]
pub struct NatsSubscriptionManager {
    subscriptions: HashMap<SubscriptionId, ActiveSubscription>,
    event_buffer: Arc<RwLock<VecDeque<BufferedEvent>>>,
}

pub struct ActiveSubscription {
    pub id: SubscriptionId,
    pub subject: String,
    pub subscriber: async_nats::Subscriber,
    pub filter: Option<EventFilter>,
    pub handler: SubscriptionHandler,
}

#[derive(Clone)]
pub enum SubscriptionHandler {
    /// Direct graph updates
    GraphUpdate { target_graph: GraphId },
    /// Buffer for batch processing
    Buffered { max_size: usize, max_age: Duration },
    /// Custom handler
    Custom(Arc<dyn Fn(DomainEvent) -> Option<GraphMutation> + Send + Sync>),
}

impl NatsSubscriptionManager {
    /// Subscribe to a NATS subject for real-time updates
    pub async fn subscribe(
        &mut self,
        subject: String,
        filter: Option<EventFilter>,
        handler: SubscriptionHandler,
    ) -> Result<SubscriptionId, SubscriptionError> {
        let subscriber = self.nats_client.subscribe(subject.clone()).await?;
        let id = SubscriptionId::new();

        let subscription = ActiveSubscription {
            id: id.clone(),
            subject,
            subscriber,
            filter,
            handler,
        };

        self.subscriptions.insert(id.clone(), subscription);

        // Spawn async task to handle messages
        self.spawn_subscription_handler(id.clone());

        Ok(id)
    }

    /// Process incoming messages from subscriptions
    async fn process_subscription_messages(&self, subscription_id: SubscriptionId) {
        if let Some(subscription) = self.subscriptions.get(&subscription_id) {
            while let Some(msg) = subscription.subscriber.next().await {
                // Parse event from NATS message
                if let Ok(event) = parse_nats_event(&msg) {
                    // Apply filter if present
                    if let Some(filter) = &subscription.filter {
                        if !filter.matches(&event) {
                            continue;
                        }
                    }

                    // Handle based on subscription type
                    match &subscription.handler {
                        SubscriptionHandler::GraphUpdate { target_graph } => {
                            if let Some(mutation) = convert_to_graph_mutation(&event) {
                                self.send_graph_update(*target_graph, mutation).await;
                            }
                        }
                        SubscriptionHandler::Buffered { max_size, max_age } => {
                            self.buffer_event(BufferedEvent {
                                event,
                                received_at: SystemTime::now(),
                                subscription_id: subscription_id.clone(),
                            }).await;
                        }
                        SubscriptionHandler::Custom(handler) => {
                            if let Some(mutation) = handler(event) {
                                self.send_custom_mutation(mutation).await;
                            }
                        }
                    }
                }
            }
        }
    }
}
```

## Usage Examples

### 1. Fetching Historical Events

```rust
pub async fn load_graph_history(
    event_service: Res<EventStreamService>,
    mut commands: Commands,
) {
    // Fetch last 24 hours of events for a specific aggregate
    let transaction = event_service.fetch_time_window(
        SystemTime::now() - Duration::from_hours(24),
        SystemTime::now(),
        Some(EventFilter::ByAggregate(aggregate_id)),
    ).await.unwrap();

    // Start replay visualization
    start_transaction_replay(&mut commands, transaction.transaction_id, 2.0);
}
```

### 2. Real-time Graph Updates

```rust
pub async fn setup_realtime_updates(
    mut subscription_manager: ResMut<NatsSubscriptionManager>,
    graph_id: GraphId,
) {
    // Subscribe to all events for this graph
    subscription_manager.subscribe(
        format!("event.store.{}.>", graph_id),
        None,
        SubscriptionHandler::GraphUpdate { target_graph: graph_id },
    ).await.unwrap();

    // Subscribe to specific event types with filtering
    subscription_manager.subscribe(
        "event.store.*.NodeTransformed".to_string(),
        Some(EventFilter::ByEventType("NodeTransformed".to_string())),
        SubscriptionHandler::Custom(Arc::new(|event| {
            // Custom transformation logic
            Some(create_transform_mutation(event))
        })),
    ).await.unwrap();
}
```

### 3. Batch Processing

```rust
pub async fn process_event_batch(
    event_service: Res<EventStreamService>,
    mut graph_model: ResMut<GraphModel>,
) {
    // Fetch a batch of events
    let transaction = event_service.fetch_transaction(
        aggregate_id,
        TransactionOptions {
            max_events: Some(100),
            replay_policy: ReplayPolicy::LastSequence(last_processed_seq),
            ..Default::default()
        },
    ).await.unwrap();

    // Apply all mutations in a single frame
    for event in &transaction.events {
        if let Some(mutation) = convert_to_graph_mutation(event) {
            apply_mutation_to_model(&mut graph_model, mutation);
        }
    }
}
```

## Benefits

1. **Transactional Consistency**: Events are fetched and applied as atomic transactions
2. **Flexible Replay**: Can replay any time window or sequence range
3. **Real-time Updates**: Direct NATS to Bevy event pipeline
4. **Batching Support**: Efficient processing of large event sets
5. **Context-Aware**: Mutations based on event content and graph context
6. **Performance**: Caching and buffering for optimal throughput

This architecture provides a robust bridge between NATS event streams and Bevy's ECS, enabling both historical replay and real-time graph updates.
