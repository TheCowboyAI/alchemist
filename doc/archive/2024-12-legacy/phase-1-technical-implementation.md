# Phase 1: Technical Implementation Guide

## Overview

This guide provides detailed technical steps for implementing Phase 1 of the EventStore-CQRS-Graph engine, focusing on NATS JetStream setup and the async/sync bridge.

## Prerequisites

### Development Environment

1. **NixOS Setup**
   ```nix
   # Add to flake.nix
   {
     inputs = {
       nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
       rust-overlay.url = "github:oxalica/rust-overlay";
     };

     outputs = { self, nixpkgs, rust-overlay }:
       let
         system = "x86_64-linux";
         pkgs = import nixpkgs {
           inherit system;
           overlays = [ rust-overlay.overlays.default ];
         };
       in {
         devShells.${system}.default = pkgs.mkShell {
           buildInputs = with pkgs; [
             # Rust toolchain
             rust-bin.stable.latest.default
             rust-analyzer

             # NATS tools
             nats-server
             natscli

             # Development tools
             pkg-config
             openssl
             protobuf

             # Bevy dependencies
             alsa-lib
             udev
             vulkan-loader
             wayland
             libxkbcommon
           ];

           NATS_URL = "nats://localhost:4222";
           RUST_LOG = "info,alchemist=debug";
           RUST_BACKTRACE = "1";
         };
       };
   }
   ```

2. **NATS Server Configuration**
   ```yaml
   # nats-server.conf
   port: 4222

   jetstream {
     store_dir: "./data/jetstream"
     max_memory_store: 1GB
     max_file_store: 10GB
   }

   # Enable monitoring
   http_port: 8222

   # Logging
   debug: false
   trace: false
   logtime: true

   # Limits
   max_payload: 1MB
   max_pending: 10MB
   ```

## 1.1 NATS JetStream Event Store

### Step 1: Core Event Store Structure

Create `src/infrastructure/event_store/mod.rs`:

```rust
use async_nats::jetstream::{self, stream::Config as StreamConfig};
use cid::Cid;
use dashmap::DashMap;
use ipld::codec::Codec;
use ipld_dagcbor::DagCborCodec;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod types;
pub mod service;
pub mod transaction;

use types::*;

/// Event Store backed by NATS JetStream with CID chains
pub struct EventStore {
    /// NATS JetStream context
    jetstream: jetstream::Context,

    /// Stream configuration
    stream_name: String,

    /// Track latest CID per aggregate for chain integrity
    latest_cids: Arc<DashMap<AggregateId, Cid>>,

    /// Event sequence tracking
    sequence_tracker: Arc<RwLock<SequenceTracker>>,
}

impl EventStore {
    /// Initialize Event Store with NATS connection
    pub async fn new(nats_client: async_nats::Client) -> Result<Self, EventStoreError> {
        let jetstream = jetstream::new(nats_client);

        // Configure event stream
        let stream_config = StreamConfig {
            name: "event-store".to_string(),
            subjects: vec!["event.store.>".to_string()],
            retention: jetstream::stream::RetentionPolicy::Limits,
            storage: jetstream::stream::StorageType::File,
            max_age: std::time::Duration::from_days(365),
            duplicate_window: std::time::Duration::from_secs(120),
            ..Default::default()
        };

        // Create or update stream
        jetstream.create_stream(stream_config).await
            .map_err(|e| EventStoreError::StreamCreation(e.to_string()))?;

        Ok(EventStore {
            jetstream,
            stream_name: "event-store".to_string(),
            latest_cids: Arc::new(DashMap::new()),
            sequence_tracker: Arc::new(RwLock::new(SequenceTracker::new())),
        })
    }
}
```

### Step 2: Event Types and CID Calculation

Create `src/infrastructure/event_store/types.rs`:

```rust
use cid::Cid;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Aggregate identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AggregateId(pub String);

/// Event identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub uuid::Uuid);

/// Domain event with CID chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub event_id: EventId,
    pub aggregate_id: AggregateId,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub sequence: u64,
    pub timestamp: SystemTime,
    pub event_cid: Option<Cid>,
    pub previous_cid: Option<Cid>,
    pub metadata: EventMetadata,
}

/// Event metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventMetadata {
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub user_id: Option<String>,
    pub tags: Vec<String>,
}

/// Structure for CID calculation
#[derive(Serialize, Deserialize)]
pub struct EventForCid {
    pub payload: Vec<u8>,
    pub previous_cid: Option<Cid>,
    pub aggregate_id: AggregateId,
    pub event_type: String,
    pub timestamp: SystemTime,
}

/// Calculate CID for an event using IPLD dag-cbor
pub fn calculate_event_cid(
    payload: &[u8],
    previous_cid: Option<Cid>,
    aggregate_id: &AggregateId,
    event_type: &str,
    timestamp: SystemTime,
) -> Result<Cid, EventStoreError> {
    use ipld::codec::Codec;
    use ipld_dagcbor::DagCborCodec;

    let event_for_cid = EventForCid {
        payload: payload.to_vec(),
        previous_cid,
        aggregate_id: aggregate_id.clone(),
        event_type: event_type.to_string(),
        timestamp,
    };

    // Encode using dag-cbor
    let encoded = DagCborCodec.encode(&event_for_cid)
        .map_err(|e| EventStoreError::CidCalculation(e.to_string()))?;

    // Create CID with dag-cbor codec (0x71)
    let cid = Cid::new_v1(0x71, multihash::Code::Sha2_256.digest(&encoded));

    Ok(cid)
}
```

### Step 3: Event Store Service Implementation

Create `src/infrastructure/event_store/service.rs`:

```rust
use super::*;
use async_nats::jetstream;

impl EventStore {
    /// Append event to NATS JetStream with CID chain
    pub async fn append_event(
        &self,
        aggregate_id: AggregateId,
        event_type: String,
        payload: serde_json::Value,
    ) -> Result<DomainEvent, EventStoreError> {
        // Get previous CID for this aggregate
        let previous_cid = self.latest_cids.get(&aggregate_id)
            .map(|entry| entry.value().clone());

        // Serialize payload for CID calculation
        let payload_bytes = serde_json::to_vec(&payload)
            .map_err(|e| EventStoreError::Serialization(e.to_string()))?;

        // Calculate event CID
        let event_cid = calculate_event_cid(
            &payload_bytes,
            previous_cid.clone(),
            &aggregate_id,
            &event_type,
            SystemTime::now(),
        )?;

        // Create domain event
        let mut event = DomainEvent {
            event_id: EventId(uuid::Uuid::new_v4()),
            aggregate_id: aggregate_id.clone(),
            event_type: event_type.clone(),
            payload,
            sequence: 0, // Will be set by NATS
            timestamp: SystemTime::now(),
            event_cid: Some(event_cid.clone()),
            previous_cid,
            metadata: EventMetadata::default(),
        };

        // Serialize event
        let event_bytes = serde_json::to_vec(&event)
            .map_err(|e| EventStoreError::Serialization(e.to_string()))?;

        // Create headers
        let mut headers = async_nats::HeaderMap::new();
        headers.insert("Cid", event_cid.to_string());
        headers.insert("Aggregate-Id", aggregate_id.0.clone());
        headers.insert("Event-Type", event_type.clone());

        if let Some(prev_cid) = &previous_cid {
            headers.insert("Previous-Cid", prev_cid.to_string());
        }

        // Publish to NATS
        let subject = format!("event.store.{}.{}", aggregate_id.0, event_type);
        let ack = self.jetstream
            .publish_with_headers(subject, headers, event_bytes.into())
            .await
            .map_err(|e| EventStoreError::PublishFailed(e.to_string()))?
            .await
            .map_err(|e| EventStoreError::AckFailed(e.to_string()))?;

        // Update event with sequence
        event.sequence = ack.sequence;

        // Update latest CID for aggregate
        self.latest_cids.insert(aggregate_id, event_cid);

        // Track sequence
        self.sequence_tracker.write().await
            .update(event.aggregate_id.clone(), ack.sequence);

        Ok(event)
    }

    /// Verify CID chain integrity
    pub async fn verify_chain(
        &self,
        aggregate_id: &AggregateId,
    ) -> Result<ChainVerification, EventStoreError> {
        // Implementation for chain verification
        todo!()
    }
}
```

### Step 4: EventStream Transaction Support

Create `src/infrastructure/event_store/transaction.rs`:

```rust
use super::*;

/// EventStream transaction for atomic event processing
#[derive(Debug, Clone)]
pub struct EventStreamTransaction {
    pub transaction_id: TransactionId,
    pub sequence_range: SequenceRange,
    pub aggregate_id: AggregateId,
    pub events: Vec<DomainEvent>,
    pub metadata: TransactionMetadata,
}

#[derive(Debug, Clone)]
pub struct TransactionId(pub uuid::Uuid);

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

#[derive(Debug, Clone)]
pub enum ReplayPolicy {
    FromBeginning,
    AfterSequence(u64),
    Latest,
    ByTime { start_time: SystemTime },
}

/// Service for fetching event transactions
pub struct EventStreamService {
    event_store: Arc<EventStore>,
    transaction_cache: Arc<DashMap<TransactionId, EventStreamTransaction>>,
}

impl EventStreamService {
    pub fn new(event_store: Arc<EventStore>) -> Self {
        Self {
            event_store,
            transaction_cache: Arc::new(DashMap::new()),
        }
    }

    /// Fetch a transaction of events
    pub async fn fetch_transaction(
        &self,
        aggregate_id: AggregateId,
        options: TransactionOptions,
    ) -> Result<EventStreamTransaction, EventStoreError> {
        // Create consumer configuration
        let consumer_config = jetstream::consumer::pull::Config {
            durable_name: Some(format!("txn-{}", uuid::Uuid::new_v4())),
            deliver_policy: match &options.replay_policy {
                ReplayPolicy::FromBeginning => jetstream::consumer::DeliverPolicy::All,
                ReplayPolicy::AfterSequence(seq) => {
                    jetstream::consumer::DeliverPolicy::ByStartSequence {
                        start_sequence: *seq + 1,
                    }
                },
                ReplayPolicy::Latest => jetstream::consumer::DeliverPolicy::Last,
                ReplayPolicy::ByTime { start_time } => {
                    jetstream::consumer::DeliverPolicy::ByStartTime {
                        start_time: (*start_time).into(),
                    }
                },
            },
            filter_subject: options.filter_subject.clone()
                .unwrap_or_else(|| format!("event.store.{}.>", aggregate_id.0)),
            ..Default::default()
        };

        // Create consumer
        let consumer = self.event_store.jetstream
            .create_consumer("event-store", consumer_config)
            .await
            .map_err(|e| EventStoreError::ConsumerCreation(e.to_string()))?;

        // Fetch events
        let mut events = Vec::new();
        let mut sequence_start = None;
        let mut sequence_end = None;

        let messages = consumer
            .fetch()
            .max_messages(options.max_events.unwrap_or(1000))
            .expires(options.timeout.unwrap_or(Duration::from_secs(5)))
            .messages()
            .await
            .map_err(|e| EventStoreError::FetchFailed(e.to_string()))?;

        // Process messages
        pin_mut!(messages);
        while let Some(msg) = messages.next().await {
            let msg = msg.map_err(|e| EventStoreError::MessageError(e.to_string()))?;

            // Get sequence
            let seq = msg.info()
                .map_err(|e| EventStoreError::MessageInfo(e.to_string()))?
                .stream_sequence;

            if sequence_start.is_none() {
                sequence_start = Some(seq);
            }
            sequence_end = Some(seq);

            // Parse event
            let event: DomainEvent = serde_json::from_slice(&msg.payload)
                .map_err(|e| EventStoreError::Deserialization(e.to_string()))?;

            // Verify CID if present
            if let Some(expected_cid) = msg.headers.get("Cid") {
                if let Some(event_cid) = &event.event_cid {
                    if event_cid.to_string() != expected_cid.as_str() {
                        return Err(EventStoreError::CidMismatch {
                            expected: expected_cid.as_str().to_string(),
                            actual: event_cid.to_string(),
                        });
                    }
                }
            }

            events.push(event);

            // Acknowledge
            msg.ack().await
                .map_err(|e| EventStoreError::AckFailed(e.to_string()))?;
        }

        // Create transaction
        let transaction = EventStreamTransaction {
            transaction_id: TransactionId(uuid::Uuid::new_v4()),
            sequence_range: SequenceRange {
                start: sequence_start.unwrap_or(0),
                end: sequence_end.unwrap_or(0),
                stream_name: "event-store".to_string(),
            },
            aggregate_id,
            events,
            metadata: TransactionMetadata {
                fetched_at: SystemTime::now(),
                consumer_name: consumer.info().await?.config.durable_name.unwrap_or_default(),
                filter_subject: options.filter_subject,
                replay_policy: options.replay_policy,
            },
        };

        // Cache transaction
        self.transaction_cache.insert(
            transaction.transaction_id.clone(),
            transaction.clone(),
        );

        Ok(transaction)
    }
}
```

## 1.2 Async/Sync Bridge Implementation

### Step 1: Bridge Core Structure

Create `src/infrastructure/bridge/mod.rs`:

```rust
use crossbeam::channel;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod command;
pub mod event;
pub mod processor;

use command::*;
use event::*;

/// Bridge between async NATS and sync Bevy
pub struct AsyncSyncBridge {
    /// Commands from Bevy (sync) to NATS (async)
    command_tx: channel::Sender<BridgeCommand>,
    command_rx: Arc<Mutex<channel::Receiver<BridgeCommand>>>,

    /// Events from NATS (async) to Bevy (sync)
    event_tx: tokio::sync::mpsc::UnboundedSender<BridgeEvent>,
    event_rx: channel::Receiver<BridgeEvent>,

    /// Batch configuration
    batch_config: BatchConfig,
}

#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub max_batch_size: usize,
    pub batch_timeout: Duration,
    pub max_pending_events: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            batch_timeout: Duration::from_millis(10),
            max_pending_events: 10_000,
        }
    }
}

impl AsyncSyncBridge {
    pub fn new(batch_config: BatchConfig) -> Self {
        let (command_tx, command_rx) = channel::bounded(1000);
        let (event_tx_async, mut event_rx_async) = tokio::sync::mpsc::unbounded_channel();
        let (event_tx_sync, event_rx_sync) = channel::bounded(batch_config.max_pending_events);

        // Spawn task to bridge async events to sync
        tokio::spawn(async move {
            while let Some(event) = event_rx_async.recv().await {
                if event_tx_sync.send(event).is_err() {
                    log::error!("Failed to send event to sync channel");
                    break;
                }
            }
        });

        Self {
            command_tx,
            command_rx: Arc::new(Mutex::new(command_rx)),
            event_tx: event_tx_async,
            event_rx: event_rx_sync,
            batch_config,
        }
    }

    /// Send command from Bevy (sync)
    pub fn send_command(&self, command: BridgeCommand) -> Result<(), BridgeError> {
        self.command_tx.send(command)
            .map_err(|_| BridgeError::CommandChannelClosed)
    }

    /// Receive events in Bevy (sync)
    pub fn receive_events(&self) -> Vec<BridgeEvent> {
        let mut events = Vec::new();
        let deadline = std::time::Instant::now() + self.batch_config.batch_timeout;

        // Collect events until batch size or timeout
        while events.len() < self.batch_config.max_batch_size {
            let timeout = deadline.saturating_duration_since(std::time::Instant::now());

            match self.event_rx.recv_timeout(timeout) {
                Ok(event) => events.push(event),
                Err(channel::RecvTimeoutError::Timeout) => break,
                Err(channel::RecvTimeoutError::Disconnected) => {
                    log::error!("Event channel disconnected");
                    break;
                }
            }
        }

        events
    }
}
```

### Step 2: Command Processing

Create `src/infrastructure/bridge/command.rs`:

```rust
use super::*;
use crate::domain::graph::commands::*;

/// Commands that can be sent through the bridge
#[derive(Debug, Clone)]
pub enum BridgeCommand {
    Graph(GraphCommand),
    Query(QueryCommand),
    Subscription(SubscriptionCommand),
}

/// Graph-related commands
#[derive(Debug, Clone)]
pub enum GraphCommand {
    CreateGraph {
        id: GraphId,
        metadata: GraphMetadata,
    },
    AddNode {
        graph_id: GraphId,
        node: NodeEntity,
    },
    ConnectNodes {
        graph_id: GraphId,
        source: NodeId,
        target: NodeId,
        edge_type: EdgeType,
    },
    RemoveNode {
        graph_id: GraphId,
        node_id: NodeId,
    },
}

/// Async command processor
pub struct CommandProcessor {
    event_store: Arc<EventStore>,
    graph_service: Arc<GraphService>,
}

impl CommandProcessor {
    pub async fn process(&self, command: BridgeCommand) -> Result<Vec<BridgeEvent>, ProcessError> {
        match command {
            BridgeCommand::Graph(graph_cmd) => self.process_graph_command(graph_cmd).await,
            BridgeCommand::Query(query_cmd) => self.process_query_command(query_cmd).await,
            BridgeCommand::Subscription(sub_cmd) => self.process_subscription_command(sub_cmd).await,
        }
    }

    async fn process_graph_command(&self, command: GraphCommand) -> Result<Vec<BridgeEvent>, ProcessError> {
        let events = match command {
            GraphCommand::CreateGraph { id, metadata } => {
                let event = self.graph_service.create_graph(id, metadata).await?;
                vec![BridgeEvent::GraphCreated {
                    graph_id: event.aggregate_id,
                    event_cid: event.event_cid,
                }]
            }
            GraphCommand::AddNode { graph_id, node } => {
                let event = self.graph_service.add_node(graph_id, node).await?;
                vec![BridgeEvent::NodeAdded {
                    graph_id: event.aggregate_id,
                    node_id: node.id,
                    event_cid: event.event_cid,
                }]
            }
            // ... other commands
        };

        Ok(events)
    }
}
```

### Step 3: Event Routing

Create `src/infrastructure/bridge/event.rs`:

```rust
use super::*;

/// Events that flow through the bridge
#[derive(Debug, Clone)]
pub enum BridgeEvent {
    // Graph mutation events
    GraphCreated {
        graph_id: AggregateId,
        event_cid: Option<Cid>,
    },
    NodeAdded {
        graph_id: AggregateId,
        node_id: NodeId,
        event_cid: Option<Cid>,
    },
    EdgeConnected {
        graph_id: AggregateId,
        edge_id: EdgeId,
        source: NodeId,
        target: NodeId,
        event_cid: Option<Cid>,
    },

    // Query results
    QueryResult {
        query_id: QueryId,
        result: QueryResultData,
    },

    // Real-time updates
    RealtimeUpdate {
        subscription_id: SubscriptionId,
        update: UpdateData,
    },
}

/// Event router for processing incoming NATS events
pub struct EventRouter {
    bridge: Arc<AsyncSyncBridge>,
    subscriptions: Arc<DashMap<String, SubscriptionHandler>>,
}

impl EventRouter {
    pub async fn route_event(&self, event: DomainEvent) -> Result<(), RouterError> {
        // Convert domain event to bridge event
        let bridge_event = match event.event_type.as_str() {
            "GraphCreated" => BridgeEvent::GraphCreated {
                graph_id: event.aggregate_id,
                event_cid: event.event_cid,
            },
            "NodeAdded" => {
                let payload: NodeAddedPayload = serde_json::from_value(event.payload)?;
                BridgeEvent::NodeAdded {
                    graph_id: event.aggregate_id,
                    node_id: payload.node_id,
                    event_cid: event.event_cid,
                }
            }
            // ... other event types
            _ => return Ok(()), // Ignore unknown events
        };

        // Send through bridge
        self.bridge.event_tx.send(bridge_event)
            .map_err(|_| RouterError::BridgeClosed)?;

        Ok(())
    }
}
```

### Step 4: Bridge Processor Task

Create `src/infrastructure/bridge/processor.rs`:

```rust
use super::*;

/// Main processor that runs the async side of the bridge
pub struct BridgeProcessor {
    bridge: Arc<AsyncSyncBridge>,
    command_processor: Arc<CommandProcessor>,
    event_router: Arc<EventRouter>,
    event_stream_service: Arc<EventStreamService>,
}

impl BridgeProcessor {
    pub async fn run(self) -> Result<(), ProcessorError> {
        // Spawn command processing task
        let cmd_processor = self.command_processor.clone();
        let cmd_bridge = self.bridge.clone();
        let cmd_task = tokio::spawn(async move {
            Self::process_commands(cmd_bridge, cmd_processor).await
        });

        // Spawn event subscription task
        let event_router = self.event_router.clone();
        let event_service = self.event_stream_service.clone();
        let event_task = tokio::spawn(async move {
            Self::process_subscriptions(event_service, event_router).await
        });

        // Wait for tasks
        tokio::select! {
            result = cmd_task => {
                log::error!("Command processor stopped: {:?}", result);
            }
            result = event_task => {
                log::error!("Event processor stopped: {:?}", result);
            }
        }

        Ok(())
    }

    async fn process_commands(
        bridge: Arc<AsyncSyncBridge>,
        processor: Arc<CommandProcessor>,
    ) -> Result<(), ProcessorError> {
        let receiver = bridge.command_rx.clone();

        loop {
            // Get command from channel
            let command = {
                let rx = receiver.lock().await;
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(cmd) => cmd,
                    Err(channel::RecvTimeoutError::Timeout) => continue,
                    Err(channel::RecvTimeoutError::Disconnected) => {
                        return Err(ProcessorError::ChannelClosed);
                    }
                }
            };

            // Process command
            match processor.process(command).await {
                Ok(events) => {
                    for event in events {
                        if bridge.event_tx.send(event).is_err() {
                            log::error!("Failed to send event to bridge");
                        }
                    }
                }
                Err(e) => {
                    log::error!("Command processing error: {:?}", e);
                }
            }
        }
    }

    async fn process_subscriptions(
        event_service: Arc<EventStreamService>,
        router: Arc<EventRouter>,
    ) -> Result<(), ProcessorError> {
        // Subscribe to all graph events
        let mut subscriber = event_service.event_store.jetstream
            .subscribe("event.store.>")
            .await?;

        while let Some(msg) = subscriber.next().await {
            match msg {
                Ok(msg) => {
                    // Parse event
                    match serde_json::from_slice::<DomainEvent>(&msg.payload) {
                        Ok(event) => {
                            if let Err(e) = router.route_event(event).await {
                                log::error!("Event routing error: {:?}", e);
                            }
                        }
                        Err(e) => {
                            log::error!("Event parsing error: {:?}", e);
                        }
                    }

                    // Acknowledge
                    if let Err(e) = msg.ack().await {
                        log::error!("Failed to ack message: {:?}", e);
                    }
                }
                Err(e) => {
                    log::error!("Subscription error: {:?}", e);
                }
            }
        }

        Ok(())
    }
}
```

## Testing Phase 1

### Integration Test Setup

Create `tests/phase1_integration.rs`:

```rust
use alchemist::infrastructure::{event_store::*, bridge::*};

#[tokio::test]
async fn test_event_store_cid_chain() {
    // Start NATS server
    let _server = nats_server::run_basic_server();

    // Connect to NATS
    let client = async_nats::connect("nats://localhost:4222").await.unwrap();

    // Create event store
    let event_store = EventStore::new(client).await.unwrap();

    // Create aggregate
    let aggregate_id = AggregateId("test-graph".to_string());

    // Append first event (no previous CID)
    let event1 = event_store.append_event(
        aggregate_id.clone(),
        "GraphCreated".to_string(),
        serde_json::json!({
            "name": "Test Graph",
            "metadata": {}
        }),
    ).await.unwrap();

    assert!(event1.previous_cid.is_none());
    assert!(event1.event_cid.is_some());

    // Append second event (should have previous CID)
    let event2 = event_store.append_event(
        aggregate_id.clone(),
        "NodeAdded".to_string(),
        serde_json::json!({
            "node_id": "node-1",
            "node_type": "Entity"
        }),
    ).await.unwrap();

    assert_eq!(event2.previous_cid, event1.event_cid);
}

#[tokio::test]
async fn test_async_sync_bridge() {
    let bridge = AsyncSyncBridge::new(BatchConfig::default());

    // Test command sending (sync side)
    let command = BridgeCommand::Graph(GraphCommand::CreateGraph {
        id: GraphId::new(),
        metadata: GraphMetadata::default(),
    });

    bridge.send_command(command).unwrap();

    // Test event receiving (sync side)
    let events = bridge.receive_events();
    assert!(events.is_empty()); // No events yet
}
```

## Next Steps

1. **Run Tests**: `cargo test --test phase1_integration`
2. **Start NATS**: `nats-server -c nats-server.conf`
3. **Monitor NATS**: Open http://localhost:8222
4. **Verify Streams**: `nats stream ls`
5. **Check Events**: `nats stream view event-store`

This completes the technical implementation for Phase 1. The foundation is now ready for building the write model in Phase 2.
