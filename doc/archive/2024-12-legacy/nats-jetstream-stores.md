# NATS JetStream for Event Store and Object Store

## Overview

Our system uses NATS JetStream as the underlying implementation for both Event Store and Object Store. The MerkleDAG structures are maintained through CID references stored in JetStream, with Daggy providing the graph representation for traversal and algorithms. CIDs are created using IPLD dag-cbor format and calculated from the event/object payload plus the previous CID.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    NATS JetStream                            │
│  ┌──────────────────┐        ┌──────────────────┐          │
│  │   Event Stream   │        │  Object Stream   │          │
│  │  (event.store)   │        │ (object.store)   │          │
│  └──────────────────┘        └──────────────────┘          │
└─────────────────────────────────────────────────────────────┘
                    │                      │
          ┌─────────┴──────────┐ ┌────────┴─────────┐
          │  Event Store API   │ │ Object Store API │
          │  (MerkleDAG ops)   │ │ (CAS operations) │
          └─────────┬──────────┘ └────────┬─────────┘
                    │                      │
                    └──────────┬───────────┘
                               │
                    ┌──────────┴───────────┐
                    │   Daggy MerkleDAG    │
                    │  (Graph Structure)    │
                    └──────────────────────┘
```

## Event Store Implementation

### 1. NATS JetStream Configuration

```rust
use async_nats::jetstream::{self, stream::Config as StreamConfig};
use cid::Cid;
use daggy::Dag;
use ipld::codec::Codec;
use ipld_dagcbor::DagCborCodec;

/// Event Store backed by NATS JetStream
pub struct EventStore {
    /// NATS JetStream context
    jetstream: jetstream::Context,
    /// Stream name for events
    stream_name: String,
    /// In-memory MerkleDAG for traversal (rebuilt from NATS)
    dag: Arc<RwLock<Dag<EventNode, EventEdge>>>,
    /// CID to NATS sequence mapping
    cid_to_seq: Arc<RwLock<HashMap<Cid, u64>>>,
    /// Track the latest CID for each aggregate
    latest_cids: Arc<RwLock<HashMap<AggregateId, Cid>>>,
}

impl EventStore {
    /// Initialize Event Store with NATS JetStream
    pub async fn new(nats_client: async_nats::Client) -> Result<Self, EventStoreError> {
        let jetstream = jetstream::new(nats_client);

        // Create or get event stream
        let stream_config = StreamConfig {
            name: "event-store".to_string(),
            subjects: vec!["event.store.>".to_string()],
            retention: jetstream::stream::RetentionPolicy::Limits,
            storage: jetstream::stream::StorageType::File,
            max_age: std::time::Duration::from_days(365 * 10), // 10 years
            duplicate_window: std::time::Duration::from_secs(120),
            ..Default::default()
        };

        jetstream.create_stream(stream_config).await?;

        Ok(EventStore {
            jetstream,
            stream_name: "event-store".to_string(),
            dag: Arc::new(RwLock::new(Dag::new())),
            cid_to_seq: Arc::new(RwLock::new(HashMap::new())),
            latest_cids: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Append event to NATS JetStream with IPLD dag-cbor CID
    pub async fn append_event(
        &self,
        aggregate_id: AggregateId,
        event_type: String,
        payload: Vec<u8>,
        parent_cids: Vec<Cid>,
    ) -> Result<DomainEvent, EventStoreError> {
        // Get the previous CID for this aggregate
        let previous_cid = self.latest_cids.read().await
            .get(&aggregate_id)
            .cloned();

        // Create the event structure for CID calculation
        let event_for_cid = EventForCid {
            payload: payload.clone(),
            previous_cid,
            aggregate_id: aggregate_id.clone(),
            event_type: event_type.clone(),
            timestamp: SystemTime::now(),
        };

        // Calculate CID using IPLD dag-cbor
        let event_cid = self.calculate_event_cid(&event_for_cid)?;

        // Create the full event
        let event = DomainEvent {
            aggregate_id,
            event_type: event_type.clone(),
            payload,
            parent_cids: parent_cids.clone(),
            timestamp: event_for_cid.timestamp,
            event_cid: Some(event_cid.clone()),
            previous_cid,
        };

        // Serialize event for storage
        let event_bytes = DagCborCodec.encode(&event)?;

        // Store in NATS with CID as header
        let subject = format!("event.store.{}.{}", aggregate_id, event_type);
        let mut headers = async_nats::HeaderMap::new();
        headers.insert("Cid", event_cid.to_string());
        headers.insert("Aggregate-Id", aggregate_id.to_string());
        headers.insert("Event-Type", event_type);

        // Add parent CIDs as headers
        for (i, parent) in parent_cids.iter().enumerate() {
            headers.insert(format!("Parent-{}", i), parent.to_string());
        }

        // Add previous CID if exists
        if let Some(prev_cid) = &previous_cid {
            headers.insert("Previous-Cid", prev_cid.to_string());
        }

        let ack = self.jetstream
            .publish_with_headers(subject, headers, event_bytes.into())
            .await?
            .await?;

        // Update latest CID for this aggregate
        self.latest_cids.write().await.insert(aggregate_id.clone(), event_cid.clone());

        // Update in-memory DAG
        self.update_dag(event_cid, event.clone(), ack.sequence).await?;

        Ok(event)
    }

    /// Calculate CID using IPLD dag-cbor format
    fn calculate_event_cid(&self, event_for_cid: &EventForCid) -> Result<Cid, EventStoreError> {
        // Encode using dag-cbor
        let encoded = DagCborCodec.encode(event_for_cid)?;

        // Create CID with dag-cbor codec (0x71)
        let cid = Cid::new_v1(0x71, multihash::Code::Sha2_256.digest(&encoded));

        Ok(cid)
    }

    /// Rebuild DAG from NATS on startup
    pub async fn rebuild_dag(&self) -> Result<(), EventStoreError> {
        let mut consumer = self.jetstream
            .create_consumer(
                self.stream_name.clone(),
                jetstream::consumer::pull::Config {
                    durable_name: Some("dag-rebuilder".to_string()),
                    ..Default::default()
                },
            )
            .await?;

        let mut dag = self.dag.write().await;
        let mut cid_to_seq = self.cid_to_seq.write().await;
        let mut cid_to_node = HashMap::new();
        let mut latest_cids = self.latest_cids.write().await;

        // Process all messages to rebuild DAG
        loop {
            let messages = consumer.fetch()
                .max_messages(1000)
                .messages()
                .await?;

            if messages.is_empty() {
                break;
            }

            for msg in messages {
                let cid = Cid::from_str(&msg.headers.get("Cid").unwrap())?;
                let event: DomainEvent = DagCborCodec.decode(&msg.payload)?;

                // Add to DAG
                let node_idx = dag.add_node(EventNode {
                    cid: cid.clone(),
                    event: event.clone(),
                });

                cid_to_seq.insert(cid.clone(), msg.info().unwrap().stream_sequence);
                cid_to_node.insert(cid.clone(), node_idx);

                // Update latest CID for aggregate
                latest_cids.insert(event.aggregate_id.clone(), cid.clone());

                // Add edge from previous CID if exists
                if let Some(prev_cid) = &event.previous_cid {
                    if let Some(&prev_idx) = cid_to_node.get(prev_cid) {
                        dag.add_edge(prev_idx, node_idx, EventEdge::Previous)?;
                    }
                }

                // Add edges for parent relationships
                for parent_cid in &event.parent_cids {
                    if let Some(&parent_idx) = cid_to_node.get(parent_cid) {
                        dag.add_edge(parent_idx, node_idx, EventEdge::Parent)?;
                    }
                }

                msg.ack().await?;
            }
        }

        Ok(())
    }
}

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

/// Event node in the MerkleDAG
#[derive(Debug, Clone)]
pub struct EventNode {
    pub cid: Cid,
    pub event: DomainEvent,
}

/// Domain event with CID references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub aggregate_id: AggregateId,
    pub event_type: String,
    pub payload: Vec<u8>,
    pub parent_cids: Vec<Cid>,
    pub timestamp: SystemTime,
    pub event_cid: Option<Cid>,
    pub previous_cid: Option<Cid>,
}

/// Edge types in the event DAG
#[derive(Debug, Clone)]
pub enum EventEdge {
    /// Link to previous event in same aggregate
    Previous,
    /// Link to parent event (causal dependency)
    Parent,
}
```

### 2. Object Store Implementation

```rust
/// Object Store backed by NATS JetStream
pub struct ObjectStore {
    /// NATS JetStream context
    jetstream: jetstream::Context,
    /// Object store bucket
    object_store: jetstream::object_store::ObjectStore,
    /// In-memory MerkleDAG for object relationships
    dag: Arc<RwLock<Dag<ObjectNode, ObjectEdge>>>,
}

impl ObjectStore {
    /// Initialize Object Store with NATS JetStream
    pub async fn new(nats_client: async_nats::Client) -> Result<Self, ObjectStoreError> {
        let jetstream = jetstream::new(nats_client);

        // Create object store bucket
        let object_store = jetstream
            .create_object_store(jetstream::object_store::Config {
                bucket: "object-store".to_string(),
                description: Some("Content-addressed object storage".to_string()),
                storage: jetstream::stream::StorageType::File,
                ..Default::default()
            })
            .await?;

        Ok(ObjectStore {
            jetstream,
            object_store,
            dag: Arc::new(RwLock::new(Dag::new())),
        })
    }

    /// Store object in NATS Object Store with IPLD dag-cbor CID
    pub async fn store<T: Serialize>(&self, object: &T, previous_cid: Option<Cid>) -> Result<Cid, ObjectStoreError> {
        // Create structure for CID calculation
        let object_for_cid = ObjectForCid {
            data: object,
            previous_cid,
            timestamp: SystemTime::now(),
        };

        // Encode using dag-cbor
        let bytes = DagCborCodec.encode(&object_for_cid)?;

        // Calculate CID
        let cid = Cid::new_v1(0x71, multihash::Code::Sha2_256.digest(&bytes));

        // Store in NATS Object Store with CID as key
        self.object_store
            .put(
                cid.to_string(),
                &mut bytes.as_slice(),
            )
            .await?;

        // Update DAG
        let mut dag = self.dag.write().await;
        let node_idx = dag.add_node(ObjectNode {
            cid: cid.clone(),
            size: bytes.len(),
            content_type: std::any::type_name::<T>().to_string(),
            stored_at: SystemTime::now(),
            previous_cid,
        });

        // Link to previous if exists
        if let Some(prev_cid) = previous_cid {
            if let Some(prev_idx) = self.find_node_by_cid(&dag, &prev_cid)? {
                dag.add_edge(prev_idx, node_idx, ObjectEdge {
                    link_type: LinkType::Previous,
                    created_at: SystemTime::now(),
                })?;
            }
        }

        Ok(cid)
    }

    /// Retrieve object from NATS Object Store
    pub async fn get<T: DeserializeOwned>(&self, cid: &Cid) -> Result<T, ObjectStoreError> {
        let object = self.object_store.get(cid.to_string()).await?;
        let object_for_cid: ObjectForCid<T> = DagCborCodec.decode(&object.data)?;
        Ok(object_for_cid.data)
    }

    /// Link objects in MerkleDAG
    pub async fn link_objects(
        &self,
        parent_cid: &Cid,
        child_cid: &Cid,
        link_type: LinkType,
    ) -> Result<(), ObjectStoreError> {
        let mut dag = self.dag.write().await;

        // Find nodes
        let parent_idx = self.find_node_by_cid(&dag, parent_cid)?;
        let child_idx = self.find_node_by_cid(&dag, child_cid)?;

        // Add edge
        dag.add_edge(parent_idx, child_idx, ObjectEdge {
            link_type,
            created_at: SystemTime::now(),
        })?;

        Ok(())
    }
}

/// Structure used for object CID calculation
#[derive(Serialize, Deserialize)]
struct ObjectForCid<T> {
    /// The actual object data
    data: T,
    /// Previous CID in the chain (if any)
    previous_cid: Option<Cid>,
    /// When the object was stored
    timestamp: SystemTime,
}

/// Object node in the MerkleDAG
#[derive(Debug, Clone)]
pub struct ObjectNode {
    pub cid: Cid,
    pub size: usize,
    pub content_type: String,
    pub stored_at: SystemTime,
    pub previous_cid: Option<Cid>,
}
```

### 3. Integration with Graph Rendering

```rust
/// Service that bridges NATS stores with Bevy rendering
pub struct NatsGraphBridge {
    event_store: Arc<EventStore>,
    object_store: Arc<ObjectStore>,
    /// Petgraph for algorithms and visualization
    workflow_graphs: Arc<RwLock<HashMap<WorkflowId, WorkflowGraph>>>,
}

impl NatsGraphBridge {
    /// Subscribe to event updates from NATS
    pub async fn subscribe_to_events(&self) -> Result<(), BridgeError> {
        let mut subscriber = self.event_store.jetstream
            .subscribe("event.store.>")
            .await?;

        while let Some(msg) = subscriber.next().await {
            let cid = Cid::from_str(&msg.headers.get("Cid").unwrap())?;
            let event: DomainEvent = DagCborCodec.decode(&msg.payload)?;

            // Update visualization
            self.update_visualization(EventUpdate {
                cid,
                event,
                sequence: msg.info()?.stream_sequence,
            }).await?;
        }

        Ok(())
    }

    /// Convert NATS event stream to Bevy components
    pub fn create_bevy_components(
        &self,
        commands: &mut Commands,
        event: &DomainEvent,
    ) -> Entity {
        commands.spawn((
            DddElement {
                element_type: DddElementType::Event {
                    cid: event.event_cid.clone().unwrap(),
                    event_type: event.event_type.clone(),
                },
                graph_index: GraphIndex::Daggy(/* node index */),
                metadata: DddMetadata {
                    created_at: event.timestamp,
                    aggregate_id: event.aggregate_id.clone(),
                    previous_cid: event.previous_cid.clone(),
                },
            },
            GraphNodeVisual {
                base_color: Color::BLUE,
                highlight_color: Color::CYAN,
                size: 0.5,
                shape: NodeShape::Diamond,
            },
        )).id()
    }
}
```

### 4. Testing with Mock Stores

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Mock event store for testing without NATS
    pub struct MockEventStore {
        events: Vec<DomainEvent>,
        dag: Dag<EventNode, EventEdge>,
        latest_cids: HashMap<AggregateId, Cid>,
    }

    impl MockEventStore {
        pub fn new() -> Self {
            Self {
                events: Vec::new(),
                dag: Dag::new(),
                latest_cids: HashMap::new(),
            }
        }

        pub fn append_event(&mut self, mut event: DomainEvent) -> Result<Cid, String> {
            // Get previous CID
            let previous_cid = self.latest_cids.get(&event.aggregate_id).cloned();
            event.previous_cid = previous_cid;

            // Create structure for CID calculation
            let event_for_cid = EventForCid {
                payload: event.payload.clone(),
                previous_cid,
                aggregate_id: event.aggregate_id.clone(),
                event_type: event.event_type.clone(),
                timestamp: event.timestamp,
            };

            // Calculate CID using dag-cbor
            let encoded = DagCborCodec.encode(&event_for_cid).unwrap();
            let cid = Cid::new_v1(0x71, multihash::Code::Sha2_256.digest(&encoded));

            event.event_cid = Some(cid.clone());

            // Update latest CID
            self.latest_cids.insert(event.aggregate_id.clone(), cid.clone());

            self.events.push(event);
            // Update DAG...
            Ok(cid)
        }
    }

    #[test]
    fn test_event_cid_chain() {
        let mut store = MockEventStore::new();

        // First event has no previous CID
        let event1 = DomainEvent {
            aggregate_id: AggregateId::from("test-agg"),
            event_type: "Created".to_string(),
            payload: b"first event".to_vec(),
            parent_cids: vec![],
            timestamp: SystemTime::now(),
            event_cid: None,
            previous_cid: None,
        };

        let cid1 = store.append_event(event1).unwrap();

        // Second event includes previous CID in calculation
        let event2 = DomainEvent {
            aggregate_id: AggregateId::from("test-agg"),
            event_type: "Updated".to_string(),
            payload: b"second event".to_vec(),
            parent_cids: vec![],
            timestamp: SystemTime::now(),
            event_cid: None,
            previous_cid: None,
        };

        let cid2 = store.append_event(event2).unwrap();

        // CIDs should be different even with same payload
        assert_ne!(cid1, cid2);

        // Verify chain
        assert_eq!(store.events[1].previous_cid, Some(cid1));
    }
}
```

## Configuration

### NATS Server Configuration

```yaml
# nats-server.conf
jetstream {
  store_dir: "/data/nats/jetstream"
  max_memory_store: 1GB
  max_file_store: 100GB
}

# Event store specific limits
limits {
  max_payload: 1MB
  max_pending: 10GB
}

# Clustering for high availability
cluster {
  name: "event-store-cluster"
  routes: [
    "nats://node1:6222"
    "nats://node2:6222"
    "nats://node3:6222"
  ]
}
```

### Client Configuration

```rust
/// NATS client configuration
pub struct NatsConfig {
    pub servers: Vec<String>,
    pub auth: NatsAuth,
    pub jetstream: JetStreamConfig,
}

pub struct JetStreamConfig {
    pub event_stream: StreamConfig,
    pub object_bucket: ObjectStoreConfig,
    pub max_reconnects: u32,
    pub reconnect_delay: Duration,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            servers: vec!["nats://localhost:4222".to_string()],
            auth: NatsAuth::None,
            jetstream: JetStreamConfig {
                event_stream: StreamConfig {
                    name: "event-store".to_string(),
                    subjects: vec!["event.store.>".to_string()],
                    retention: RetentionPolicy::Limits,
                    ..Default::default()
                },
                object_bucket: ObjectStoreConfig {
                    bucket: "object-store".to_string(),
                    ..Default::default()
                },
                max_reconnects: 10,
                reconnect_delay: Duration::from_secs(5),
            },
        }
    }
}
```

## Benefits of NATS JetStream with IPLD

1. **Content-Addressed Storage**: CIDs provide unique identifiers based on content
2. **Verifiable History**: Previous CID in calculation ensures chain integrity
3. **IPLD Compatibility**: dag-cbor format works with IPLD ecosystem
4. **Built-in Persistence**: No need for separate database
5. **Streaming Support**: Real-time event updates
6. **At-least-once Delivery**: Guaranteed message delivery
7. **Horizontal Scaling**: Cluster support for high availability

## Usage Example

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to NATS
    let client = async_nats::connect("nats://localhost:4222").await?;

    // Initialize stores
    let event_store = EventStore::new(client.clone()).await?;
    let object_store = ObjectStore::new(client.clone()).await?;

    // Create domain event (CID calculated with previous CID)
    let event = event_store.append_event(
        AggregateId::new(),
        "OrderCreated".to_string(),
        serde_json::to_vec(&OrderCreatedPayload {
            order_id: OrderId::new(),
            customer: "john@example.com".to_string(),
            items: vec![/* ... */],
        })?,
        vec![], // No parent events
    ).await?;

    println!("Event stored with CID: {}", event.event_cid.unwrap());

    // Subscribe to events for real-time updates
    let mut subscriber = client.subscribe("event.store.>").await?;

    while let Some(msg) = subscriber.next().await {
        println!("New event: {:?}", msg);
        // Update Bevy visualization...
    }

    Ok(())
}
```

This architecture leverages NATS JetStream's built-in features for both event and object storage while maintaining the MerkleDAG structure through IPLD dag-cbor CIDs that include the previous CID in their calculation, ensuring a verifiable chain of events.
