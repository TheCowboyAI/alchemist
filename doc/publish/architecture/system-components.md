# System Components

## Overview

This document provides a detailed reference for all system components in the CIM-integrated Information Alchemist architecture. Components are organized by architectural layer and bounded context.

## Presentation Layer (Bevy ECS)

### Graph Visualization Components

#### GraphNode Component
```rust
#[derive(Component)]
pub struct GraphNode {
    pub node_id: NodeId,
    pub label: String,
    pub node_type: NodeType,
    pub conceptual_point: Option<ConceptualPoint>,
}

#[derive(Component)]
pub struct NodePosition {
    pub current: Vec3,
    pub target: Vec3,
    pub velocity: Vec3,
}

#[derive(Component)]
pub struct NodeVisuals {
    pub color: Color,
    pub size: f32,
    pub shape: NodeShape,
    pub glow_intensity: f32,
}
```

#### GraphEdge Component
```rust
#[derive(Component)]
pub struct GraphEdge {
    pub edge_id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub edge_type: EdgeType,
    pub weight: f32,
}

#[derive(Component)]
pub struct EdgeVisuals {
    pub color: Color,
    pub thickness: f32,
    pub style: EdgeStyle,
    pub animation: Option<EdgeAnimation>,
}
```

### Camera and Navigation

#### CameraController
```rust
#[derive(Component)]
pub struct CameraController {
    pub focus_point: Vec3,
    pub distance: f32,
    pub rotation: Quat,
    pub movement_speed: f32,
    pub rotation_speed: f32,
}

#[derive(Resource)]
pub struct CameraState {
    pub mode: CameraMode,
    pub target_node: Option<NodeId>,
    pub transition: Option<CameraTransition>,
}
```

### Interaction Systems

#### SelectionSystem
```rust
pub struct SelectionSystem;

impl SelectionSystem {
    pub fn handle_mouse_click(
        &self,
        mouse_pos: Vec2,
        camera: &Camera,
        nodes: &Query<(&Transform, &GraphNode)>,
    ) -> Option<NodeId> {
        // Ray casting for node selection
    }
}

#[derive(Component)]
pub struct Selected;

#[derive(Resource)]
pub struct SelectionState {
    pub selected_nodes: HashSet<NodeId>,
    pub selected_edges: HashSet<EdgeId>,
    pub multi_select: bool,
}
```

## Domain Layer

### Aggregates (State Machine-Driven)

**Critical Principle**: If a domain concept requires transactional guarantees, it MUST be implemented as an aggregate with a Mealy State Machine. Not all aggregates are transactional, but all transactions require aggregates.

#### Graph Aggregate (Transactional)

```rust
// State machine for graph lifecycle
#[derive(Clone, Debug)]
pub enum GraphState {
    Empty,
    Active { node_count: usize },
    Executing { workflow_id: WorkflowId },
    Archived { at: SystemTime },
}

pub struct GraphAggregate {
    id: GraphId,
    state: GraphState,
    nodes: HashMap<NodeId, Node>,
    edges: Vec<Edge>,
    version: u64,
}

impl StateMachineAggregate for GraphAggregate {
    type State = GraphState;
    type Command = GraphCommand;
    type Event = GraphEvent;

    fn transition(
        current_state: &Self::State,
        command: Self::Command
    ) -> Result<(Self::State, Vec<Self::Event>), DomainError> {
        match (current_state, command) {
            // Empty -> Active
            (GraphState::Empty, GraphCommand::AddNode { node }) => {
                let new_state = GraphState::Active { node_count: 1 };
                let events = vec![GraphEvent::NodeAdded { node }];
                Ok((new_state, events))
            },

            // Active -> Executing
            (GraphState::Active { .. }, GraphCommand::ExecuteWorkflow { workflow_id }) => {
                let new_state = GraphState::Executing { workflow_id };
                let events = vec![GraphEvent::WorkflowStarted { workflow_id }];
                Ok((new_state, events))
            },

            // Invalid transitions
            (GraphState::Archived { .. }, _) => {
                Err(DomainError::AggregateArchived)
            },

            // ... other transitions
        }
    }
}
```

#### Workflow Aggregate (Transactional)

```rust
// Complex state machine for workflow execution
#[derive(Clone, Debug)]
pub enum WorkflowState {
    Designed,
    Ready { validated_at: SystemTime },
    Running {
        started_at: SystemTime,
        current_step: StepId,
    },
    Paused {
        paused_at: SystemTime,
        resume_point: StepId,
    },
    Completed {
        completed_at: SystemTime,
        result: WorkflowResult,
    },
    Failed {
        failed_at: SystemTime,
        error: String,
        recovery_point: Option<StepId>,
    },
}

impl WorkflowAggregate {
    // State machine ensures workflow integrity
    pub fn can_transition(&self, to: &WorkflowState) -> bool {
        match (&self.state, to) {
            (WorkflowState::Designed, WorkflowState::Ready { .. }) => true,
            (WorkflowState::Ready { .. }, WorkflowState::Running { .. }) => true,
            (WorkflowState::Running { .. }, WorkflowState::Paused { .. }) => true,
            (WorkflowState::Running { .. }, WorkflowState::Completed { .. }) => true,
            (WorkflowState::Running { .. }, WorkflowState::Failed { .. }) => true,
            (WorkflowState::Paused { .. }, WorkflowState::Running { .. }) => true,
            (WorkflowState::Failed { recovery_point: Some(_), .. }, WorkflowState::Running { .. }) => true,
            _ => false,
        }
    }
}
```

#### Non-Transactional Aggregates

```rust
// View aggregate - no transactions needed
pub struct GraphViewAggregate {
    id: GraphId,
    layout: GraphLayout,
    visible_nodes: HashSet<NodeId>,
    zoom_level: f32,
}

// Metrics aggregate - eventual consistency is fine
pub struct GraphMetricsAggregate {
    graph_id: GraphId,
    node_count: usize,
    edge_count: usize,
    last_updated: SystemTime,
}

// These aggregates handle events but don't enforce transactions
impl GraphViewAggregate {
    pub fn handle_event(&mut self, event: GraphEvent) {
        match event {
            GraphEvent::NodeAdded { node } => {
                self.visible_nodes.insert(node.id);
            }
            GraphEvent::ZoomChanged { level } => {
                self.zoom_level = level;
            }
            _ => {}
        }
    }
}
```

### State Machine Components

```rust
// Component for visualizing state machines in the graph editor
#[derive(Component)]
pub struct StateMachineComponent {
    aggregate_type: String,
    current_state: String,
    available_transitions: Vec<String>,
    state_history: Vec<StateTransition>,
}

#[derive(Component)]
pub struct StateTransition {
    from: String,
    to: String,
    command: String,
    timestamp: SystemTime,
}

// System to visualize aggregate state machines
fn visualize_state_machines(
    query: Query<(&StateMachineComponent, &Transform)>,
    mut gizmos: Gizmos,
) {
    for (state_machine, transform) in query.iter() {
        // Draw current state
        gizmos.circle(
            transform.translation,
            Vec3::Y,
            30.0,
            Color::GREEN,
        );

        // Draw available transitions
        for (i, transition) in state_machine.available_transitions.iter().enumerate() {
            let angle = (i as f32) * std::f32::consts::TAU / state_machine.available_transitions.len() as f32;
            let end = transform.translation + Vec3::new(angle.cos() * 100.0, 0.0, angle.sin() * 100.0);
            gizmos.arrow(transform.translation, end, Color::YELLOW);
        }
    }
}
```

### Conceptual Space Domain

#### Conceptual Mapping
```rust
pub struct ConceptualMapping {
    embedding_model: Arc<EmbeddingModel>,
    space_dimensions: usize,
}

#[derive(Clone, Debug)]
pub struct ConceptualPoint {
    pub coordinates: Vec<f32>,
    pub confidence: f32,
}

impl ConceptualMapping {
    pub async fn map_to_conceptual(&self, text: &str) -> Result<ConceptualPoint, ConceptualError> {
        let embedding = self.embedding_model.embed(text).await?;
        Ok(ConceptualPoint {
            coordinates: embedding.to_vec(),
            confidence: 1.0,
        })
    }
}

pub struct ConceptualSimilarity;

impl ConceptualSimilarity {
    pub fn calculate(&self, point_a: &ConceptualPoint, point_b: &ConceptualPoint) -> f32 {
        // Cosine similarity calculation
        let dot_product: f32 = point_a.coordinates.iter()
            .zip(&point_b.coordinates)
            .map(|(a, b)| a * b)
            .sum();

        let magnitude_a: f32 = point_a.coordinates.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = point_b.coordinates.iter().map(|x| x * x).sum::<f32>().sqrt();

        dot_product / (magnitude_a * magnitude_b)
    }
}

pub struct ConceptualTopology {
    points: Vec<(NodeId, ConceptualPoint)>,
    distance_metric: DistanceMetric,
}

impl ConceptualTopology {
    pub fn find_neighbors(&self, reference: &ConceptualPoint, radius: f32) -> Vec<(NodeId, f32)> {
        self.points.iter()
            .map(|(id, point)| {
                let distance = self.distance_metric.calculate(reference, point);
                (*id, distance)
            })
            .filter(|(_, distance)| *distance <= radius)
            .collect()
    }

    pub fn cluster_regions(&self) -> Vec<ConceptualRegion> {
        // Clustering algorithm to identify dense regions in conceptual space
        // Returns convex regions as per Gärdenfors' theory
        todo!()
    }
}
```

## Infrastructure Layer

### NATS Integration

#### Event Publisher
```rust
pub struct NatsEventPublisher {
    client: async_nats::Client,
    jetstream: async_nats::jetstream::Context,
}

impl NatsEventPublisher {
    pub async fn publish_event(&self, event: DomainEvent) -> Result<(), PublishError> {
        let subject = format!(
            "{}.events.{}.{}",
            event.aggregate_type(),
            event.aggregate_id(),
            event.event_type()
        );

        let payload = serde_json::to_vec(&event)?;

        self.jetstream
            .publish(subject, payload.into())
            .await?
            .await?;

        Ok(())
    }
}
```

#### Event Subscriber
```rust
pub struct NatsEventSubscriber {
    client: async_nats::Client,
    handlers: Arc<RwLock<HashMap<String, Box<dyn EventHandler>>>>,
}

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle_event(&self, event: DomainEvent) -> Result<(), HandleError>;
}

impl NatsEventSubscriber {
    pub async fn subscribe(&self, pattern: &str) -> Result<(), SubscribeError> {
        let mut subscriber = self.client.subscribe(pattern).await?;

        while let Some(message) = subscriber.next().await {
            let event: DomainEvent = serde_json::from_slice(&message.payload)?;

            if let Some(handler) = self.handlers.read().await.get(&event.event_type()) {
                handler.handle_event(event).await?;
            }
        }

        Ok(())
    }
}
```

### Event Store

#### JetStream Event Store
```rust
pub struct JetStreamEventStore {
    context: async_nats::jetstream::Context,
    stream_name: String,
}

impl JetStreamEventStore {
    pub async fn create_stream(&self) -> Result<Stream, StoreError> {
        self.context
            .create_stream(async_nats::jetstream::stream::Config {
                name: self.stream_name.clone(),
                subjects: vec!["*.events.>".to_string()],
                retention: RetentionPolicy::Limits,
                storage: StorageType::File,
                num_replicas: 3,
                ..Default::default()
            })
            .await
            .map_err(Into::into)
    }

    pub async fn load_events(
        &self,
        aggregate_id: &AggregateId,
        from_version: u64,
    ) -> Result<Vec<StoredEvent>, StoreError> {
        let subject = format!("{}.events.{}.>", aggregate_id.context(), aggregate_id.id());

        let messages = self.context
            .get_stream(&self.stream_name)
            .await?
            .get_raw_message_stream(
                StreamSequence::from(from_version),
                Some(100), // batch size
            )
            .await?;

        // Convert messages to events
        Ok(messages.map(|msg| self.message_to_event(msg)).collect().await)
    }
}
```

### Projection Store

#### Read Model Storage
```rust
pub struct ProjectionStore {
    graph_projections: Arc<RwLock<HashMap<GraphId, GraphProjection>>>,
    workflow_projections: Arc<RwLock<HashMap<WorkflowId, WorkflowProjection>>>,
    conceptual_index: Arc<ConceptualIndex>,
}

pub struct ConceptualIndex {
    points: Vec<(NodeId, ConceptualPoint)>,
    kdtree: Option<KdTree<f32, NodeId>>,
}

impl ProjectionStore {
    pub async fn update_graph_projection(
        &self,
        id: GraphId,
        updater: impl FnOnce(&mut GraphProjection),
    ) -> Result<(), ProjectionError> {
        let mut projections = self.graph_projections.write().await;

        if let Some(projection) = projections.get_mut(&id) {
            updater(projection);

            // Update conceptual index if needed
            if let Some(ref conceptual_mappings) = projection.conceptual_mappings {
                self.conceptual_index.update_mappings(conceptual_mappings).await?;
            }
        }

        Ok(())
    }

    pub async fn find_similar_nodes(
        &self,
        reference: NodeId,
        max_distance: f32,
        limit: usize,
    ) -> Result<Vec<(NodeId, f32)>, ProjectionError> {
        self.conceptual_index.find_nearest(reference, max_distance, limit).await
    }
}
```

## Bridge Components

### Async-Sync Bridge

#### Command Bridge
```rust
pub struct CommandBridge {
    sender: crossbeam_channel::Sender<BridgedCommand>,
    receiver: tokio::sync::mpsc::Receiver<CommandResult>,
}

pub enum BridgedCommand {
    Graph(GraphCommand),
    Workflow(WorkflowCommand),
    Query(QueryCommand),
}

impl CommandBridge {
    pub fn send_command(&self, cmd: BridgedCommand) -> Result<CommandId, BridgeError> {
        let command_id = CommandId::new();
        self.sender.send((command_id, cmd))?;
        Ok(command_id)
    }

    pub async fn await_result(&mut self, command_id: CommandId) -> Result<CommandResult, BridgeError> {
        while let Some(result) = self.receiver.recv().await {
            if result.command_id == command_id {
                return Ok(result);
            }
        }
        Err(BridgeError::ResultTimeout)
    }
}
```

#### Event Bridge
```rust
pub struct EventBridge {
    event_sender: tokio::sync::mpsc::Sender<DomainEvent>,
    bevy_events: Arc<RwLock<Vec<BevyEvent>>>,
}

#[derive(Event)]
pub struct BevyEvent {
    pub event_type: String,
    pub payload: serde_json::Value,
    pub timestamp: Instant,
}

impl EventBridge {
    pub async fn forward_to_bevy(&self, event: DomainEvent) -> Result<(), BridgeError> {
        let bevy_event = BevyEvent {
            event_type: event.event_type(),
            payload: serde_json::to_value(&event)?,
            timestamp: Instant::now(),
        };

        self.bevy_events.write().await.push(bevy_event);
        Ok(())
    }
}
```

## Supporting Components

### Configuration

```rust
#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub nats: NatsConfig,
    pub bevy: BevyConfig,
    pub domain: DomainConfig,
}

#[derive(Deserialize, Clone)]
pub struct NatsConfig {
    pub url: String,
    pub credentials_path: Option<PathBuf>,
    pub jetstream: JetStreamConfig,
}

#[derive(Deserialize, Clone)]
pub struct DomainConfig {
    pub event_store: EventStoreConfig,
    pub conceptual_space: ConceptualSpaceConfig,
    pub workflow: WorkflowConfig,
}
```

### Error Handling

```rust
#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Invalid graph name: {0}")]
    InvalidGraphName(String),

    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),

    #[error("Workflow execution failed: {0}")]
    WorkflowExecutionFailed(String),

    #[error("Conceptual mapping failed: {0}")]
    ConceptualMappingFailed(#[from] ConceptualError),
}

#[derive(Error, Debug)]
pub enum InfrastructureError {
    #[error("NATS connection failed: {0}")]
    NatsConnectionFailed(#[from] async_nats::Error),

    #[error("Event store error: {0}")]
    EventStoreError(String),

    #[error("Projection update failed: {0}")]
    ProjectionUpdateFailed(String),
}
```

## Component Interactions

### Initialization Flow
```
1. Load Configuration
2. Initialize NATS Client
3. Create JetStream Streams
4. Initialize Event Store
5. Start Event Subscribers
6. Initialize Projection Store
7. Create Command/Event Bridges
8. Start Bevy App with Resources
```

### Runtime Flow
```
User Input → Bevy System → Command Bridge → Domain Handler
    ↓                                            ↓
UI Update ← Event Bridge ← Projection ← Domain Event → NATS
```

This component architecture ensures clear separation of concerns, testability, and scalability while maintaining the flexibility needed for a CIM leaf node implementation.
