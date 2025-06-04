# Multi-Graph Architecture with External Domains

## Overview

Our system uses multiple specialized graph types, each optimized for its specific purpose:

1. **MerkleDAG Graphs (using Daggy)**:
   - Event Store - Immutable event history (backed by NATS JetStream)
   - Object Store - Content-addressed storage (backed by NATS JetStream)
   - Future: Audit logs, Version control, etc.

2. **General Graphs (using Petgraph)**:
   - Workflow graphs
   - Relationship graphs
   - Authorization graphs
   - Visualization graphs

3. **External Domain References**:
   - People Domain
   - Organization Domain
   - Location Domain
   - (Extensible for future domains)

## Architecture

### 1. MerkleDAG Layer (Daggy + NATS JetStream)

```rust
use daggy::{Dag, NodeIndex as DagNodeIndex, EdgeIndex as DagEdgeIndex};
use cid::Cid;
use async_nats::jetstream;
use ipld::codec::Codec;
use ipld_dagcbor::DagCborCodec;

/// Event Store backed by NATS JetStream
pub struct EventStore {
    /// NATS JetStream context
    jetstream: jetstream::Context,
    /// In-memory MerkleDAG for traversal
    dag: Arc<RwLock<Dag<EventNode, EventEdge>>>,
    /// CID to NATS sequence mapping
    cid_to_seq: Arc<RwLock<HashMap<Cid, u64>>>,
    /// Track latest CID per aggregate for chain calculation
    latest_cids: Arc<RwLock<HashMap<AggregateId, Cid>>>,
}

/// Object Store backed by NATS JetStream
pub struct ObjectStore {
    /// NATS JetStream object store
    object_store: jetstream::object_store::ObjectStore,
    /// In-memory MerkleDAG for object relationships
    dag: Arc<RwLock<Dag<ObjectNode, ObjectEdge>>>,
}

impl EventStore {
    /// Initialize with NATS JetStream
    pub async fn new(nats_client: async_nats::Client) -> Result<Self, EventStoreError> {
        let jetstream = jetstream::new(nats_client);

        // Configure JetStream for event storage
        let stream_config = jetstream::stream::Config {
            name: "event-store".to_string(),
            subjects: vec!["event.store.>".to_string()],
            retention: jetstream::stream::RetentionPolicy::Limits,
            storage: jetstream::stream::StorageType::File,
            max_age: std::time::Duration::from_days(365 * 10), // 10 year retention
            ..Default::default()
        };

        jetstream.create_stream(stream_config).await?;

        let store = EventStore {
            jetstream,
            dag: Arc::new(RwLock::new(Dag::new())),
            cid_to_seq: Arc::new(RwLock::new(HashMap::new())),
            latest_cids: Arc::new(RwLock::new(HashMap::new())),
        };

        // Rebuild DAG from NATS on startup
        store.rebuild_dag_from_nats().await?;

        Ok(store)
    }

    /// Append event to NATS JetStream with IPLD dag-cbor CID
    pub async fn append_event(
        &self,
        aggregate_id: AggregateId,
        event_type: String,
        payload: Vec<u8>,
        parent_cids: Vec<Cid>,
    ) -> Result<EventNode, EventStoreError> {
        // Get previous CID for this aggregate (for chain calculation)
        let previous_cid = self.latest_cids.read().await
            .get(&aggregate_id)
            .cloned();

        // Create structure for CID calculation
        let event_for_cid = EventForCid {
            payload: payload.clone(),
            previous_cid,
            aggregate_id: aggregate_id.clone(),
            event_type: event_type.clone(),
            timestamp: SystemTime::now(),
        };

        // Calculate CID using IPLD dag-cbor
        let encoded = DagCborCodec.encode(&event_for_cid)?;
        let event_cid = Cid::new_v1(0x71, multihash::Code::Sha2_256.digest(&encoded));

        // Store in NATS JetStream
        let subject = format!("event.store.{}.{}", aggregate_id, event_type);
        let mut headers = async_nats::HeaderMap::new();
        headers.insert("Cid", event_cid.to_string());
        headers.insert("Aggregate-Id", aggregate_id.to_string());
        headers.insert("Previous-Cid", previous_cid.as_ref().map(|c| c.to_string()).unwrap_or_default());

        let ack = self.jetstream
            .publish_with_headers(subject, headers, payload.into())
            .await?;

        // Update latest CID for this aggregate
        self.latest_cids.write().await.insert(aggregate_id.clone(), event_cid.clone());

        // Update in-memory DAG
        let event_node = EventNode {
            cid: event_cid,
            event_type,
            aggregate_id,
            timestamp: event_for_cid.timestamp,
            nats_sequence: ack.sequence,
            previous_cid,
        };

        self.update_dag(event_node.clone(), parent_cids).await?;

        Ok(event_node)
    }
}

/// Structure used for CID calculation
#[derive(Serialize, Deserialize)]
struct EventForCid {
    payload: Vec<u8>,
    previous_cid: Option<Cid>,
    aggregate_id: AggregateId,
    event_type: String,
    timestamp: SystemTime,
}

/// Event node in the MerkleDAG
#[derive(Debug, Clone)]
pub struct EventNode {
    pub cid: Cid,
    pub event_type: String,
    pub aggregate_id: AggregateId,
    pub timestamp: SystemTime,
    pub nats_sequence: u64,
    pub previous_cid: Option<Cid>, // For chain integrity
}

impl ObjectStore {
    /// Store object with IPLD dag-cbor CID
    pub async fn store<T: Serialize>(&self, object: &T, previous_cid: Option<Cid>) -> Result<ObjectNode, ObjectStoreError> {
        // Create structure for CID calculation
        let object_for_cid = ObjectForCid {
            data: object,
            previous_cid,
            timestamp: SystemTime::now(),
        };

        // Calculate CID using IPLD dag-cbor
        let encoded = DagCborCodec.encode(&object_for_cid)?;
        let cid = Cid::new_v1(0x71, multihash::Code::Sha2_256.digest(&encoded));

        // Store in NATS Object Store
        self.object_store.put(cid.to_string(), &mut encoded.as_slice()).await?;

        let object_node = ObjectNode {
            cid,
            content_type: std::any::type_name::<T>().to_string(),
            size: encoded.len(),
            metadata: ObjectMetadata::default(),
            nats_name: cid.to_string(),
            previous_cid,
        };

        Ok(object_node)
    }
}

/// Structure used for object CID calculation
#[derive(Serialize, Deserialize)]
struct ObjectForCid<T> {
    data: T,
    previous_cid: Option<Cid>,
    timestamp: SystemTime,
}

/// Object node in the MerkleDAG
#[derive(Debug, Clone)]
pub struct ObjectNode {
    pub cid: Cid,
    pub content_type: ContentType,
    pub size: usize,
    pub metadata: ObjectMetadata,
    pub nats_name: String,
    pub previous_cid: Option<Cid>, // For chain integrity
}
```

### 2. Workflow Layer (Petgraph)

```rust
use petgraph::graph::{Graph, NodeIndex, EdgeIndex};

/// General purpose graphs for workflows and relationships
pub struct WorkflowGraph {
    /// The petgraph for current state and algorithms
    graph: Graph<WorkflowNode, WorkflowEdge>,
    /// Links to event history in NATS
    event_links: HashMap<NodeIndex, Cid>,
    /// External domain references
    external_refs: ExternalDomainRefs,
}

#[derive(Debug, Clone)]
pub struct WorkflowNode {
    pub id: NodeId,
    pub node_type: WorkflowNodeType,
    /// References to external domains
    pub external_entities: Vec<ExternalEntityRef>,
}

#[derive(Debug, Clone)]
pub enum WorkflowNodeType {
    Task {
        assigned_to: ExternalEntityRef, // Person from People domain
        department: ExternalEntityRef,   // OrganizationalUnit
    },
    Decision {
        approver: ExternalEntityRef,    // Person with authority
        location: ExternalEntityRef,    // Where decision is made
    },
    Document {
        author: ExternalEntityRef,       // Person who created
        organization: ExternalEntityRef, // Organization context
    },
}
```

### 3. External Domain Integration

```rust
/// External domain references - these exist outside our system
#[derive(Debug, Clone)]
pub struct ExternalDomainRefs {
    people: PeopleDomainAdapter,
    organization: OrganizationDomainAdapter,
    location: LocationDomainAdapter,
    // Extensible for future domains
    custom_domains: HashMap<String, Box<dyn DomainAdapter>>,
}

/// Reference to an entity in an external domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalEntityRef {
    pub domain: DomainType,
    pub entity_type: String,
    pub entity_id: String,
    /// Optional CID if the entity is cached in our NATS Object Store
    pub cached_cid: Option<Cid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainType {
    People,
    Organization,
    Location,
    Custom(String),
}

/// Adapter trait for external domains
pub trait DomainAdapter: Send + Sync {
    /// Resolve an external reference
    fn resolve(&self, entity_ref: &ExternalEntityRef) -> Result<DomainEntity, DomainError>;

    /// Query entities in the domain
    fn query(&self, criteria: QueryCriteria) -> Result<Vec<DomainEntity>, DomainError>;

    /// Subscribe to changes in the domain
    fn subscribe(&self, entity_ref: &ExternalEntityRef) -> Result<DomainEventStream, DomainError>;
}
```

### 4. People Domain Integration

```rust
/// People domain types (external)
pub mod people {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Person {
        pub id: PersonId,
        pub name: Name,
        pub email: Email,
        pub roles: Vec<Role>,
        pub organization_refs: Vec<ExternalEntityRef>, // Links to Organization domain
    }

    pub struct PeopleDomainAdapter {
        /// Connection to external People service
        client: PeopleServiceClient,
        /// NATS Object Store for caching
        object_store: Arc<ObjectStore>,
    }

    impl DomainAdapter for PeopleDomainAdapter {
        fn resolve(&self, entity_ref: &ExternalEntityRef) -> Result<DomainEntity, DomainError> {
            // First check cache via CID in NATS Object Store
            if let Some(cid) = &entity_ref.cached_cid {
                if let Ok(cached) = self.object_store.get(cid).await {
                    return Ok(cached);
                }
            }

            // Fetch from external service
            let person = self.client.get_person(&entity_ref.entity_id)?;

            // Cache in NATS Object Store
            let cid = self.object_store.store(&person).await?;

            Ok(DomainEntity::Person(person))
        }
    }
}
```

### 5. Organization Domain Integration

```rust
/// Organization domain types (external)
pub mod organization {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OrganizationalUnit {
        pub id: OrgUnitId,
        pub name: String,
        pub type_: OrganizationType,
        pub parent: Option<OrgUnitId>,
        pub location_ref: Option<ExternalEntityRef>, // Links to Location domain
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum OrganizationType {
        Company,
        Division,
        Department,
        Team,
        Custom(String),
    }

    pub struct OrganizationDomainAdapter {
        client: OrgServiceClient,
        object_store: Arc<ObjectStore>, // NATS Object Store for caching
    }
}
```

### 6. Location Domain Integration

```rust
/// Location domain types (external)
pub mod location {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Address {
        pub id: AddressId,
        pub street: String,
        pub city: String,
        pub state: String,
        pub country: String,
        pub postal_code: String,
        pub geo_location: Option<GeoLocation>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GeoLocation {
        pub latitude: f64,
        pub longitude: f64,
        pub altitude: Option<f64>,
        pub accuracy: Option<f64>,
    }

    pub struct LocationDomainAdapter {
        client: LocationServiceClient,
        object_store: Arc<ObjectStore>, // NATS Object Store for caching
    }
}
```

### 7. Unified Graph Service

```rust
/// Service that coordinates between all graph types
pub struct UnifiedGraphService {
    /// MerkleDAG stores (backed by NATS JetStream)
    event_store: EventStore,
    object_store: ObjectStore,

    /// Workflow graphs (in-memory with petgraph)
    workflows: HashMap<WorkflowId, WorkflowGraph>,

    /// External domain adapters
    domains: ExternalDomainRefs,

    /// NATS client for real-time updates
    nats_client: async_nats::Client,
}

impl UnifiedGraphService {
    /// Initialize with NATS connection
    pub async fn new(nats_url: &str) -> Result<Self, ServiceError> {
        let nats_client = async_nats::connect(nats_url).await?;

        Ok(UnifiedGraphService {
            event_store: EventStore::new(nats_client.clone()).await?,
            object_store: ObjectStore::new(nats_client.clone()).await?,
            workflows: HashMap::new(),
            domains: ExternalDomainRefs::new(),
            nats_client,
        })
    }

    /// Subscribe to real-time event updates
    pub async fn subscribe_to_events(&self) -> Result<(), ServiceError> {
        let mut subscriber = self.nats_client
            .subscribe("event.store.>")
            .await?;

        while let Some(msg) = subscriber.next().await {
            // Update visualizations in real-time
            self.handle_event_update(msg).await?;
        }

        Ok(())
    }

    /// Create a document workflow with external references
    pub async fn create_document_workflow(
        &mut self,
        author_id: &str,
        org_unit_id: &str,
    ) -> Result<WorkflowId, ServiceError> {
        // Resolve external entities
        let author = self.domains.people.resolve(&ExternalEntityRef {
            domain: DomainType::People,
            entity_type: "Person".to_string(),
            entity_id: author_id.to_string(),
            cached_cid: None,
        })?;

        let org_unit = self.domains.organization.resolve(&ExternalEntityRef {
            domain: DomainType::Organization,
            entity_type: "OrganizationalUnit".to_string(),
            entity_id: org_unit_id.to_string(),
            cached_cid: None,
        })?;

        // Create workflow graph
        let mut workflow = WorkflowGraph::new();

        // Add nodes with external references
        let draft_node = workflow.add_node(WorkflowNode {
            id: NodeId::new(),
            node_type: WorkflowNodeType::Document {
                author: author.to_ref(),
                organization: org_unit.to_ref(),
            },
            external_entities: vec![author.to_ref(), org_unit.to_ref()],
        });

        // Record event in NATS JetStream
        let event = self.event_store.append_event(
            workflow.id,
            "WorkflowCreated".to_string(),
            serde_json::to_vec(&json!({
                "author": author_id,
                "organization": org_unit_id,
                "workflow_type": "document",
            }))?,
            vec![], // No parent events
        ).await?;

        // Link workflow to event history
        workflow.event_links.insert(draft_node, event.cid);

        Ok(workflow.id)
    }
}
```

### 8. Event Sourcing with External Domains

```rust
/// Events that reference external domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    // Document events with external references
    DocumentCreated {
        document_id: DocumentId,
        author: ExternalEntityRef,      // Person
        organization: ExternalEntityRef, // OrganizationalUnit
        location: ExternalEntityRef,     // Address/GeoLocation
    },

    // Approval events
    ApprovalRequested {
        document_id: DocumentId,
        approver: ExternalEntityRef,    // Person
        due_date: DateTime<Utc>,
    },

    // Assignment events
    TaskAssigned {
        task_id: TaskId,
        assignee: ExternalEntityRef,    // Person
        department: ExternalEntityRef,  // OrganizationalUnit
    },
}

impl EventStore {
    /// Append event with external domain validation
    pub async fn append_with_validation(
        &mut self,
        event: DomainEvent,
        domains: &ExternalDomainRefs,
    ) -> Result<EventNode, EventError> {
        // Validate all external references exist
        match &event {
            DomainEvent::DocumentCreated { author, organization, location, .. } => {
                domains.people.resolve(author)?;
                domains.organization.resolve(organization)?;
                domains.location.resolve(location)?;
            }
            // ... validate other event types
        }

        // Store event in NATS JetStream
        let event_data = serde_json::to_vec(&event)?;
        let aggregate_id = event.aggregate_id();
        let event_type = event.type_name();

        self.append_event(aggregate_id, event_type, event_data, vec![]).await
    }
}
```

## Benefits of This Architecture

1. **Separation of Concerns**:
   - Daggy for MerkleDAG graph structure
   - NATS JetStream for persistence and streaming
   - Petgraph for mutable workflow graphs
   - Clear boundaries for external domains

2. **Real-time Updates**:
   - NATS provides instant event notifications
   - Subscribers can react to changes immediately
   - Perfect for updating Bevy visualizations

3. **Scalability**:
   - NATS JetStream handles clustering and replication
   - Object Store provides distributed content storage
   - External domains can scale independently

4. **Integrity**:
   - Event Store maintains complete history in NATS
   - CIDs provide content addressing
   - MerkleDAG ensures tamper-proof history

5. **Performance**:
   - NATS provides high-throughput messaging
   - In-memory Daggy for fast MerkleDAG traversal
   - Petgraph algorithms for workflow optimization
   - Caching of external entities in Object Store

## EventStream Transactions

Our system fetches portions of the event store as transactional units, allowing for:

### Transactional Event Fetching

```rust
/// Fetch a coherent set of events as a transaction
pub async fn fetch_workflow_transaction(
    event_store: &EventStore,
    workflow_id: WorkflowId,
    from_sequence: Option<u64>,
) -> Result<EventStreamTransaction, Error> {
    // Fetch events as a transaction
    let transaction = event_store.fetch_transaction(
        workflow_id.into(),
        TransactionOptions {
            replay_policy: match from_sequence {
                Some(seq) => ReplayPolicy::AfterSequence(seq),
                None => ReplayPolicy::FromBeginning,
            },
            max_events: Some(1000),
            ..Default::default()
        },
    ).await?;

    Ok(transaction)
}
```

### Real-time Graph Updates via Bevy Events

```rust
/// System that processes NATS event transactions and updates graphs
pub fn process_event_transactions_system(
    event_service: Res<EventStreamService>,
    mut graph_events: EventWriter<GraphMutationEvent>,
    mut workflow_graphs: ResMut<HashMap<WorkflowId, WorkflowGraph>>,
) {
    // Poll for new transactions
    if let Ok(transactions) = event_service.poll_transactions() {
        for transaction in transactions {
            // Apply events to appropriate graph
            if let Some(workflow) = workflow_graphs.get_mut(&transaction.workflow_id) {
                for event in &transaction.events {
                    // Convert to graph mutations
                    match event.event_type.as_str() {
                        "NodeAdded" => {
                            let mutation = GraphMutation::AddNode {
                                node_id: event.node_id,
                                node_type: event.node_type,
                                properties: event.properties,
                            };

                            // Send to Bevy event system
                            graph_events.send(GraphMutationEvent {
                                source: EventSource::Nats {
                                    sequence: event.sequence,
                                    subject: event.subject.clone(),
                                },
                                mutation,
                                transaction_id: Some(transaction.transaction_id),
                            });
                        }
                        "EdgeConnected" => {
                            // Similar handling for edges
                        }
                        "SubgraphTransformed" => {
                            // Handle transformations
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
```

### Transaction-based Replay

```rust
/// Replay a historical transaction with visualization
pub async fn replay_workflow_history(
    event_store: &EventStore,
    workflow_id: WorkflowId,
    time_range: (SystemTime, SystemTime),
    commands: &mut Commands,
) -> Result<(), Error> {
    // Fetch historical events as transaction
    let transaction = event_store.fetch_time_window(
        time_range.0,
        time_range.1,
        Some(EventFilter::ByAggregate(workflow_id.into())),
    ).await?;

    // Start animated replay
    commands.spawn(TransactionReplay {
        transaction_id: transaction.transaction_id,
        current_event: 0,
        total_events: transaction.events.len(),
        replay_speed: 2.0, // 2x speed
        timer: Timer::from_seconds(0.5, TimerMode::Repeating),
    });

    Ok(())
}
```

See [NATS EventStream Transactions](./nats-eventstream-transactions.md) for detailed implementation.

This architecture provides a solid foundation for managing multiple graph types while leveraging NATS JetStream for persistence and real-time updates.
