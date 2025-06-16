# Domain Events

## Event-Driven Architecture via NATS

CIM uses domain events as the primary mechanism for communicating state changes across the system. All events are published to NATS subjects following the pattern `event.{domain}.{event_type}`.

## Event Structure

All domain events follow a consistent structure:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub event_id: EventId,
    pub event_type: String,
    pub aggregate_id: AggregateId,
    pub aggregate_type: String,
    pub sequence: u64,
    pub timestamp: SystemTime,
    pub correlation_id: Option<CorrelationId>,
    pub causation_id: Option<CausationId>,
    pub actor: Option<ActorId>,
    pub payload: serde_json::Value,
    pub metadata: EventMetadata,
}
```

## Graph Domain Events

### Node Lifecycle Events

#### `event.graph.node_created`
Published when a new node is added to a graph.

```rust
#[derive(Serialize, Deserialize)]
pub struct NodeCreated {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub node_type: NodeType,
    pub position: Position3D,
    pub conceptual_point: Option<ConceptualPoint>,
    pub components: Vec<ComponentData>,
    pub metadata: HashMap<String, Value>,
}
```

**NATS Example:**
```rust
// Subscribe to all node creation events
let mut subscriber = client.subscribe("event.graph.node_created").await?;

while let Some(message) = subscriber.next().await {
    let event: NodeCreated = serde_json::from_slice(&message.payload)?;
    println!("New node created: {} in graph {}", event.node_id, event.graph_id);
}
```

#### `event.graph.node_updated`
Published when node properties are modified.

```rust
#[derive(Serialize, Deserialize)]
pub struct NodeUpdated {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub changes: NodeChanges,
    pub old_values: HashMap<String, Value>,
    pub new_values: HashMap<String, Value>,
    pub reason: UpdateReason,
}
```

#### `event.graph.node_deleted`
Published when a node is removed from a graph.

```rust
#[derive(Serialize, Deserialize)]
pub struct NodeDeleted {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub node_type: NodeType,
    pub connected_edges: Vec<EdgeId>,
    pub reason: DeletionReason,
}
```

### Edge Lifecycle Events

#### `event.graph.edge_created`
Published when nodes are connected.

```rust
#[derive(Serialize, Deserialize)]
pub struct EdgeCreated {
    pub graph_id: GraphId,
    pub edge_id: EdgeId,
    pub source_node: NodeId,
    pub target_node: NodeId,
    pub edge_type: EdgeType,
    pub weight: Option<f32>,
    pub properties: HashMap<String, Value>,
    pub semantic_distance: Option<f32>,
}
```

#### `event.graph.edge_updated`
Published when edge properties change.

```rust
#[derive(Serialize, Deserialize)]
pub struct EdgeUpdated {
    pub graph_id: GraphId,
    pub edge_id: EdgeId,
    pub changes: EdgeChanges,
    pub old_properties: HashMap<String, Value>,
    pub new_properties: HashMap<String, Value>,
}
```

#### `event.graph.edge_deleted`
Published when an edge is removed.

```rust
#[derive(Serialize, Deserialize)]
pub struct EdgeDeleted {
    pub graph_id: GraphId,
    pub edge_id: EdgeId,
    pub source_node: NodeId,
    pub target_node: NodeId,
    pub reason: DeletionReason,
}
```

### Graph Structure Events

#### `event.graph.graph_created`
Published when a new graph is initialized.

```rust
#[derive(Serialize, Deserialize)]
pub struct GraphCreated {
    pub graph_id: GraphId,
    pub graph_type: GraphType,
    pub name: String,
    pub description: Option<String>,
    pub owner: ActorId,
    pub access_policy: AccessPolicy,
    pub conceptual_space_id: Option<ConceptualSpaceId>,
}
```

#### `event.graph.layout_calculated`
Published when graph layout algorithms complete.

```rust
#[derive(Serialize, Deserialize)]
pub struct LayoutCalculated {
    pub graph_id: GraphId,
    pub layout_algorithm: LayoutAlgorithm,
    pub node_positions: HashMap<NodeId, Position3D>,
    pub layout_metrics: LayoutMetrics,
    pub calculation_time_ms: u64,
}
```

## Conceptual Space Events

### Embedding Events

#### `event.conceptual.embedding_calculated`
Published when new embeddings are computed.

```rust
#[derive(Serialize, Deserialize)]
pub struct EmbeddingCalculated {
    pub entity_id: EntityId,
    pub entity_type: EntityType,
    pub embedding_model: String,
    pub embedding_vector: Vec<f32>,
    pub confidence_score: f32,
    pub computation_time_ms: u64,
}
```

#### `event.conceptual.similarity_computed`
Published when similarity calculations complete.

```rust
#[derive(Serialize, Deserialize)]
pub struct SimilarityComputed {
    pub entity_a: EntityId,
    pub entity_b: EntityId,
    pub similarity_score: f32,
    pub similarity_type: SimilarityType,
    pub threshold_exceeded: bool,
}
```

### Category Events

#### `event.conceptual.category_formed`
Published when new categories are discovered.

```rust
#[derive(Serialize, Deserialize)]
pub struct CategoryFormed {
    pub category_id: CategoryId,
    pub category_name: String,
    pub member_entities: Vec<EntityId>,
    pub prototype_entity: EntityId,
    pub convex_region: ConvexRegion,
    pub confidence: f32,
}
```

#### `event.conceptual.space_recalibrated`
Published when conceptual spaces are updated.

```rust
#[derive(Serialize, Deserialize)]
pub struct SpaceRecalibrated {
    pub space_id: ConceptualSpaceId,
    pub dimension_changes: Vec<DimensionChange>,
    pub affected_entities: Vec<EntityId>,
    pub recalibration_reason: RecalibrationReason,
    pub new_mapping: ConceptualMapping,
}
```

## Workflow Events

### Process Events

#### `event.workflow.process_started`
Published when workflow execution begins.

```rust
#[derive(Serialize, Deserialize)]
pub struct ProcessStarted {
    pub workflow_id: WorkflowId,
    pub process_instance_id: ProcessInstanceId,
    pub start_node: NodeId,
    pub input_data: HashMap<String, Value>,
    pub initiator: ActorId,
    pub context: WorkflowContext,
}
```

#### `event.workflow.step_completed`
Published when workflow steps finish.

```rust
#[derive(Serialize, Deserialize)]
pub struct StepCompleted {
    pub workflow_id: WorkflowId,
    pub process_instance_id: ProcessInstanceId,
    pub step_id: NodeId,
    pub step_type: StepType,
    pub execution_time_ms: u64,
    pub output_data: HashMap<String, Value>,
    pub next_steps: Vec<NodeId>,
}
```

#### `event.workflow.process_completed`
Published when entire workflows finish.

```rust
#[derive(Serialize, Deserialize)]
pub struct ProcessCompleted {
    pub workflow_id: WorkflowId,
    pub process_instance_id: ProcessInstanceId,
    pub completion_status: CompletionStatus,
    pub final_output: HashMap<String, Value>,
    pub total_execution_time_ms: u64,
    pub steps_executed: u32,
}
```

### Decision Events

#### `event.workflow.decision_reached`
Published when decision nodes evaluate.

```rust
#[derive(Serialize, Deserialize)]
pub struct DecisionReached {
    pub workflow_id: WorkflowId,
    pub process_instance_id: ProcessInstanceId,
    pub decision_node: NodeId,
    pub decision_criteria: DecisionCriteria,
    pub chosen_path: NodeId,
    pub evaluation_data: HashMap<String, Value>,
    pub confidence: f32,
}
```

## Agent Events

### Agent Lifecycle

#### `event.agent.registered`
Published when AI agents join the system.

```rust
#[derive(Serialize, Deserialize)]
pub struct AgentRegistered {
    pub agent_id: AgentId,
    pub agent_type: AgentType,
    pub capabilities: Vec<Capability>,
    pub communication_patterns: Vec<CommunicationPattern>,
    pub resource_requirements: ResourceRequirements,
    pub registration_timestamp: SystemTime,
}
```

#### `event.agent.capability_updated`
Published when agent capabilities change.

```rust
#[derive(Serialize, Deserialize)]
pub struct CapabilityUpdated {
    pub agent_id: AgentId,
    pub added_capabilities: Vec<Capability>,
    pub removed_capabilities: Vec<Capability>,
    pub performance_metrics: PerformanceMetrics,
    pub update_reason: UpdateReason,
}
```

### Agent Interaction Events

#### `event.agent.task_assigned`
Published when tasks are delegated to agents.

```rust
#[derive(Serialize, Deserialize)]
pub struct TaskAssigned {
    pub agent_id: AgentId,
    pub task_id: TaskId,
    pub task_type: TaskType,
    pub task_parameters: HashMap<String, Value>,
    pub priority: Priority,
    pub deadline: Option<SystemTime>,
    pub assigner: ActorId,
}
```

#### `event.agent.collaboration_initiated`
Published when agents begin collaborative work.

```rust
#[derive(Serialize, Deserialize)]
pub struct CollaborationInitiated {
    pub collaboration_id: CollaborationId,
    pub participating_agents: Vec<AgentId>,
    pub collaboration_type: CollaborationType,
    pub shared_context: HashMap<String, Value>,
    pub coordination_pattern: CoordinationPattern,
}
```

## System Events

### Infrastructure Events

#### `event.system.node_joined`
Published when system nodes come online.

```rust
#[derive(Serialize, Deserialize)]
pub struct NodeJoined {
    pub node_id: SystemNodeId,
    pub node_type: SystemNodeType,
    pub capabilities: Vec<SystemCapability>,
    pub resource_capacity: ResourceCapacity,
    pub network_address: String,
}
```

#### `event.system.performance_threshold_exceeded`
Published when performance limits are reached.

```rust
#[derive(Serialize, Deserialize)]
pub struct PerformanceThresholdExceeded {
    pub metric_name: String,
    pub current_value: f64,
    pub threshold_value: f64,
    pub severity: SeverityLevel,
    pub affected_components: Vec<ComponentId>,
    pub recommended_actions: Vec<String>,
}
```

## Event Subscription Patterns

### Subscribe to All Graph Events
```rust
let mut subscriber = client.subscribe("event.graph.>").await?;
```

### Subscribe to Specific Event Types
```rust
let mut node_events = client.subscribe("event.graph.node_*").await?;
let mut workflow_events = client.subscribe("event.workflow.>").await?;
```

### Queue Groups for Load Balancing
```rust
let mut subscriber = client.queue_subscribe(
    "event.graph.>",
    "graph_processors".to_string()
).await?;
```

### JetStream Persistent Subscriptions
```rust
let consumer = jetstream.create_consumer_on_stream(
    async_nats::jetstream::consumer::pull::Config {
        name: Some("event_processor".to_string()),
        filter_subjects: vec!["event.>".to_string()],
        ..Default::default()
    },
    "CIM_EVENTS"
).await?;
```

## Event Handling Patterns

### Event Sourcing Replay
```rust
// Replay events from specific timestamp
let mut messages = consumer
    .messages()
    .await?
    .take_while(|msg| {
        msg.info().map(|info| info.published <= target_time)
            .unwrap_or(false)
    });

while let Some(message) = messages.next().await {
    let event: DomainEvent = serde_json::from_slice(&message.payload)?;
    apply_event_to_projection(event).await?;
    message.ack().await?;
}
```

### Event Filtering and Transformation
```rust
let mut subscriber = client.subscribe("event.graph.>").await?;

while let Some(message) = subscriber.next().await {
    let event: DomainEvent = serde_json::from_slice(&message.payload)?;
    
    // Filter events by criteria
    if should_process_event(&event) {
        // Transform for specific use case
        let transformed = transform_event_for_ui(event);
        
        // Forward to appropriate handler
        ui_event_channel.send(transformed).await?;
    }
}
```

### Error Handling and Dead Letter Queue
```rust
async fn process_event(event: DomainEvent) -> Result<(), EventProcessingError> {
    match event.event_type.as_str() {
        "graph.node_created" => handle_node_created(event).await,
        "graph.edge_created" => handle_edge_created(event).await,
        _ => Ok(()), // Ignore unknown events
    }
}

// Dead letter queue for failed events
if let Err(error) = process_event(event.clone()).await {
    let dlq_message = FailedEventMessage {
        original_event: event,
        error_message: error.to_string(),
        retry_count: 0,
        failure_timestamp: SystemTime::now(),
    };
    
    client.publish(
        "event.dlq.processing_failed",
        serde_json::to_vec(&dlq_message)?.into()
    ).await?;
}
```

---

**All events in CIM flow through NATS, enabling real-time, distributed, and fault-tolerant event processing across the entire system.** 