# Petgraph and Event Sourcing Integration

## Overview

This document describes how petgraph integrates with our event sourcing system using NATS JetStream and EventStream transactions. The architecture ensures that:

1. **Petgraph** provides the current graph state and algorithms
2. **NATS JetStream** stores the complete event history
3. **EventStream Transactions** batch related events for atomic processing
4. **Event Replay** reconstructs graph state from any point in time

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    NATS JetStream                            │
│  ┌──────────────────┐        ┌──────────────────┐          │
│  │ Event Store      │───────►│ EventStream      │          │
│  │ (Persistent)     │        │ Transactions     │          │
│  └──────────────────┘        └──────────────────┘          │
└─────────────────────────────────────────────────────────────┘
                                        │
                              ┌─────────┴─────────┐
                              │ Event Processing  │
                              │    Pipeline       │
                              └─────────┬─────────┘
                                        │
┌─────────────────────────────┴─────────────────────────────┐
│                  Graph State Management                      │
│  ┌──────────────┐    ┌──────────────┐   ┌──────────────┐ │
│  │   Petgraph   │    │ Event Apply  │   │ Snapshot     │ │
│  │   (Current)  │◄───│   Logic      │   │ Management   │ │
│  └──────────────┘    └──────────────┘   └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Event-Sourced Graph Model

```rust
use petgraph::graph::{Graph, NodeIndex, EdgeIndex};
use async_nats::jetstream;
use cid::Cid;

/// Event-sourced graph that maintains current state via petgraph
#[derive(Debug, Clone)]
pub struct EventSourcedGraph {
    /// The current graph state
    pub graph: Graph<NodeData, EdgeData>,

    /// Graph metadata
    pub id: GraphId,
    pub version: u64,

    /// Last processed event sequence from NATS
    pub last_sequence: u64,

    /// Snapshot information
    pub last_snapshot: Option<SnapshotInfo>,
}

impl EventSourcedGraph {
    /// Apply an EventStream transaction to update graph state
    pub fn apply_transaction(&mut self, transaction: &EventStreamTransaction) -> Result<(), GraphError> {
        // Validate transaction sequence
        if transaction.sequence_range.start != self.last_sequence + 1 {
            return Err(GraphError::SequenceGap {
                expected: self.last_sequence + 1,
                actual: transaction.sequence_range.start,
            });
        }

        // Apply each event in the transaction
        for event in &transaction.events {
            self.apply_event(event)?;
        }

        // Update tracking
        self.last_sequence = transaction.sequence_range.end;
        self.version += 1;

        Ok(())
    }

    /// Apply a single event to the graph
    fn apply_event(&mut self, event: &DomainEvent) -> Result<(), GraphError> {
        match event.event_type.as_str() {
            "NodeAdded" => {
                let payload: NodeAddedPayload = serde_json::from_value(&event.payload)?;
                let node_data = NodeData {
                    id: payload.node_id,
                    node_type: payload.node_type,
                    properties: payload.properties,
                };
                self.graph.add_node(node_data);
            }

            "EdgeAdded" => {
                let payload: EdgeAddedPayload = serde_json::from_value(&event.payload)?;
                let source = self.find_node_index(&payload.source_id)?;
                let target = self.find_node_index(&payload.target_id)?;
                let edge_data = EdgeData {
                    id: payload.edge_id,
                    edge_type: payload.edge_type,
                    properties: payload.properties,
                };
                self.graph.add_edge(source, target, edge_data);
            }

            "NodeUpdated" => {
                let payload: NodeUpdatedPayload = serde_json::from_value(&event.payload)?;
                let node_idx = self.find_node_index(&payload.node_id)?;
                if let Some(node) = self.graph.node_weight_mut(node_idx) {
                    node.properties.extend(payload.updates);
                }
            }

            "NodeRemoved" => {
                let payload: NodeRemovedPayload = serde_json::from_value(&event.payload)?;
                let node_idx = self.find_node_index(&payload.node_id)?;
                self.graph.remove_node(node_idx);
            }

            _ => {} // Unknown events are ignored
        }

        Ok(())
    }
}
```

### 2. Graph Event Store Service

```rust
/// Service for managing graph events in NATS JetStream
pub struct GraphEventStore {
    jetstream: jetstream::Context,
    event_stream_service: Arc<EventStreamService>,
}

impl GraphEventStore {
    /// Append a new event to the store
    pub async fn append_event(
        &self,
        graph_id: GraphId,
        event_type: String,
        payload: serde_json::Value,
    ) -> Result<DomainEvent, EventStoreError> {
        let event = DomainEvent {
            event_id: EventId::new(),
            aggregate_id: graph_id.into(),
            event_type: event_type.clone(),
            payload,
            sequence: 0, // Set by NATS
            timestamp: SystemTime::now(),
            metadata: EventMetadata::default(),
        };

        // Publish to NATS
        let subject = format!("event.store.{}.{}", graph_id, event_type);
        let ack = self.jetstream
            .publish(subject, serde_json::to_vec(&event)?)
            .await?
            .await?;

        Ok(DomainEvent {
            sequence: ack.sequence,
            ..event
        })
    }

    /// Load graph by replaying events from beginning or snapshot
    pub async fn load_graph(
        &self,
        graph_id: GraphId,
        from_snapshot: Option<SnapshotInfo>,
    ) -> Result<EventSourcedGraph, EventStoreError> {
        let mut graph = if let Some(snapshot) = from_snapshot {
            // Load from snapshot
            self.load_snapshot(snapshot).await?
        } else {
            // Start with empty graph
            EventSourcedGraph {
                graph: Graph::new(),
                id: graph_id.clone(),
                version: 0,
                last_sequence: 0,
                last_snapshot: None,
            }
        };

        // Fetch events after snapshot
        let transaction = self.event_stream_service
            .fetch_transaction(
                graph_id.into(),
                TransactionOptions {
                    replay_policy: ReplayPolicy::AfterSequence(graph.last_sequence),
                    ..Default::default()
                },
            )
            .await?;

        // Apply transaction
        graph.apply_transaction(&transaction)?;

        Ok(graph)
    }

    /// Create a snapshot of current graph state
    pub async fn create_snapshot(
        &self,
        graph: &EventSourcedGraph,
    ) -> Result<SnapshotInfo, EventStoreError> {
        let snapshot_data = SnapshotData {
            graph_id: graph.id.clone(),
            version: graph.version,
            last_sequence: graph.last_sequence,
            graph_data: bincode::serialize(&graph.graph)?,
            created_at: SystemTime::now(),
        };

        // Store snapshot in NATS Object Store
        let cid = self.store_snapshot(snapshot_data).await?;

        Ok(SnapshotInfo {
            cid,
            graph_id: graph.id.clone(),
            version: graph.version,
            last_sequence: graph.last_sequence,
            created_at: SystemTime::now(),
        })
    }
}
```

### 3. Real-time Event Processing

```rust
/// System that processes real-time events and updates graphs
pub struct GraphEventProcessor {
    graphs: Arc<RwLock<HashMap<GraphId, EventSourcedGraph>>>,
    event_store: Arc<GraphEventStore>,
    subscription_manager: Arc<NatsSubscriptionManager>,
}

impl GraphEventProcessor {
    /// Subscribe to graph events and process them
    pub async fn start_processing(&self) -> Result<(), ProcessorError> {
        // Subscribe to all graph events
        let subscription_id = self.subscription_manager
            .subscribe(
                "event.store.*.>".to_string(),
                None,
                SubscriptionHandler::Custom(Arc::new(move |event| {
                    // Convert to graph mutation
                    Some(convert_to_graph_mutation(&event))
                })),
            )
            .await?;

        // Process incoming events
        self.process_event_stream(subscription_id).await
    }

    /// Process events from subscription
    async fn process_event_stream(&self, subscription_id: SubscriptionId) -> Result<(), ProcessorError> {
        let mut event_buffer = HashMap::new();

        loop {
            // Poll for new events
            if let Some(events) = self.subscription_manager.poll_events(subscription_id).await? {
                // Group events by graph ID for transactional processing
                for event in events {
                    let graph_id = GraphId::from(event.aggregate_id.clone());
                    event_buffer.entry(graph_id).or_insert_with(Vec::new).push(event);
                }

                // Process each graph's events as a transaction
                for (graph_id, events) in event_buffer.drain() {
                    let transaction = EventStreamTransaction {
                        transaction_id: TransactionId::new(),
                        sequence_range: SequenceRange {
                            start: events.first().unwrap().sequence,
                            end: events.last().unwrap().sequence,
                            stream_name: "event-store".to_string(),
                        },
                        aggregate_id: graph_id.clone().into(),
                        events,
                        metadata: TransactionMetadata {
                            fetched_at: SystemTime::now(),
                            consumer_name: "graph-processor".to_string(),
                            filter_subject: None,
                            replay_policy: ReplayPolicy::Latest,
                        },
                    };

                    self.apply_transaction_to_graph(graph_id, transaction).await?;
                }
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Apply transaction to a specific graph
    async fn apply_transaction_to_graph(
        &self,
        graph_id: GraphId,
        transaction: EventStreamTransaction,
    ) -> Result<(), ProcessorError> {
        let mut graphs = self.graphs.write().await;

        // Load graph if not in memory
        if !graphs.contains_key(&graph_id) {
            let graph = self.event_store.load_graph(graph_id.clone(), None).await?;
            graphs.insert(graph_id.clone(), graph);
        }

        // Apply transaction
        if let Some(graph) = graphs.get_mut(&graph_id) {
            graph.apply_transaction(&transaction)?;

            // Check if snapshot is needed
            if graph.version % 100 == 0 {
                self.event_store.create_snapshot(graph).await?;
            }
        }

        Ok(())
    }
}
```

### 4. Time Travel and Replay

```rust
/// Service for replaying graph history
pub struct GraphReplayService {
    event_stream_service: Arc<EventStreamService>,
}

impl GraphReplayService {
    /// Replay graph state at a specific point in time
    pub async fn replay_at_time(
        &self,
        graph_id: GraphId,
        target_time: SystemTime,
    ) -> Result<EventSourcedGraph, ReplayError> {
        // Fetch all events up to target time
        let transaction = self.event_stream_service
            .fetch_time_window(
                SystemTime::UNIX_EPOCH,
                target_time,
                Some(EventFilter::ByAggregate(graph_id.clone().into())),
            )
            .await?;

        // Build graph from events
        let mut graph = EventSourcedGraph {
            graph: Graph::new(),
            id: graph_id,
            version: 0,
            last_sequence: 0,
            last_snapshot: None,
        };

        graph.apply_transaction(&transaction)?;

        Ok(graph)
    }

    /// Create an animated replay of graph evolution
    pub async fn create_replay_animation(
        &self,
        graph_id: GraphId,
        start_time: SystemTime,
        end_time: SystemTime,
        step_duration: Duration,
    ) -> Result<ReplayAnimation, ReplayError> {
        // Fetch events for time range
        let transaction = self.event_stream_service
            .fetch_time_window(start_time, end_time, Some(EventFilter::ByAggregate(graph_id.into())))
            .await?;

        // Create animation frames
        let mut frames = Vec::new();
        let mut current_graph = EventSourcedGraph::new(graph_id);
        let mut frame_events = Vec::new();

        for event in transaction.events {
            frame_events.push(event.clone());

            // Check if we should create a frame
            if frame_events.len() >= 10 || // Every 10 events
               event.timestamp.duration_since(start_time).unwrap() > step_duration * frames.len() as u32 {
                // Apply events to graph
                let frame_transaction = EventStreamTransaction {
                    transaction_id: TransactionId::new(),
                    sequence_range: SequenceRange {
                        start: frame_events.first().unwrap().sequence,
                        end: frame_events.last().unwrap().sequence,
                        stream_name: "event-store".to_string(),
                    },
                    aggregate_id: graph_id.into(),
                    events: frame_events.clone(),
                    metadata: TransactionMetadata::default(),
                };

                current_graph.apply_transaction(&frame_transaction)?;

                // Create frame
                frames.push(AnimationFrame {
                    timestamp: event.timestamp,
                    graph_state: current_graph.graph.clone(),
                    events_in_frame: frame_events.len(),
                });

                frame_events.clear();
            }
        }

        Ok(ReplayAnimation {
            graph_id,
            frames,
            total_duration: end_time.duration_since(start_time).unwrap(),
        })
    }
}
```

### 5. Integration with Bevy

```rust
/// Bevy plugin for event-sourced graphs
pub struct EventSourcedGraphPlugin;

impl Plugin for EventSourcedGraphPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(GraphEventStore::new())
            .insert_resource(GraphEventProcessor::new())
            .insert_resource(GraphReplayService::new())

            // Events
            .add_event::<GraphMutationEvent>()
            .add_event::<ReplayFrameEvent>()

            // Systems
            .add_systems(Update, (
                process_graph_mutations_system,
                update_graph_visuals_system,
                handle_replay_animations_system,
            ).chain());
    }
}

/// System that processes graph mutations from event stream
fn process_graph_mutations_system(
    processor: Res<GraphEventProcessor>,
    mut graph_events: EventWriter<GraphMutationEvent>,
) {
    // Poll for processed transactions
    if let Ok(mutations) = processor.poll_mutations() {
        for mutation in mutations {
            graph_events.send(mutation);
        }
    }
}
```

## Benefits

1. **Complete History**: All graph changes stored in NATS JetStream
2. **Transactional Updates**: Related events processed atomically
3. **Time Travel**: Replay graph state at any point in time
4. **Performance**: Petgraph provides fast algorithms on current state
5. **Scalability**: NATS handles distributed event streaming
6. **Resilience**: Snapshots enable fast recovery

This architecture combines the performance of petgraph with the reliability and auditability of event sourcing through NATS JetStream.
