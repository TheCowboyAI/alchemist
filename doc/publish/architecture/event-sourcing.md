# Event Sourcing Patterns

## Overview

Event sourcing is the foundation of our architecture, where all state changes are captured as a sequence of immutable events. This document details our implementation patterns, conventions, and best practices.

## Core Concepts

### Events as First-Class Citizens

In our system, events are not side effects - they are the primary source of truth:

```rust
// Events follow DDD naming: past tense, no "Event" suffix
pub enum GraphEvent {
    Created { id: GraphId, name: String },
    NodeAdded { graph_id: GraphId, node: Node },
    EdgeConnected { graph_id: GraphId, edge: Edge },
    ConceptualPointMapped { node_id: NodeId, point: ConceptualPoint },
}
```

### Event Properties

Every event in our system has these characteristics:

1. **Immutable**: Once created, events cannot be modified
2. **Ordered**: Events have a strict temporal ordering
3. **Replayable**: System state can be rebuilt from events
4. **Auditable**: Complete history of all changes

## CQRS Implementation

### Command Side

Commands express intent to change the system:

```rust
pub enum GraphCommand {
    CreateGraph { name: String },
    AddNode { graph_id: GraphId, node_data: NodeData },
    ConnectNodes { graph_id: GraphId, source: NodeId, target: NodeId },
}

// Command handler in the domain layer
impl GraphAggregate {
    pub fn handle_command(&self, command: GraphCommand) -> Result<Vec<GraphEvent>, DomainError> {
        match command {
            GraphCommand::CreateGraph { name } => {
                // Validate business rules
                if name.is_empty() {
                    return Err(DomainError::InvalidGraphName);
                }

                // Generate events
                Ok(vec![GraphEvent::Created {
                    id: GraphId::new(),
                    name,
                }])
            }
            // ... other command handlers
        }
    }
}
```

### Query Side

Queries read from optimized projections:

```rust
// Read model optimized for graph visualization
pub struct GraphProjection {
    pub id: GraphId,
    pub name: String,
    pub nodes: HashMap<NodeId, NodeView>,
    pub edges: Vec<EdgeView>,
    pub conceptual_mappings: HashMap<NodeId, ConceptualPoint>,
}

// Query handler
pub struct GraphQueryHandler {
    projection_store: Arc<ProjectionStore>,
}

impl GraphQueryHandler {
    pub async fn get_graph(&self, id: GraphId) -> Result<GraphProjection, QueryError> {
        self.projection_store.get_graph(id).await
    }

    pub async fn find_similar_graphs(&self, reference: GraphId, threshold: f32) -> Result<Vec<GraphProjection>, QueryError> {
        // Use conceptual space for similarity search
        self.projection_store.find_similar(reference, threshold).await
    }
}
```

## State Machine-Driven Aggregates

### Mealy State Machines for Transactional Control

**Fundamental Principle**: All transactional behavior in the system is controlled by aggregates implementing Mealy State Machines. If a domain concept requires transactions, it MUST be modeled as an aggregate with a state machine.

```rust
// Mealy State Machine: Output depends on current state AND input
pub trait StateMachineAggregate {
    type State;
    type Command;
    type Event;

    fn transition(
        current_state: &Self::State,
        command: Self::Command
    ) -> Result<(Self::State, Vec<Self::Event>), DomainError>;
}

// Example: Order Aggregate with State Machine
#[derive(Clone, Debug)]
pub enum OrderState {
    Draft,
    Submitted { at: SystemTime },
    Validated { payment_id: PaymentId },
    Fulfilled { tracking: TrackingNumber },
    Cancelled { reason: String },
}

pub struct OrderAggregate {
    id: OrderId,
    state: OrderState,
    version: u64,
}

impl StateMachineAggregate for OrderAggregate {
    type State = OrderState;
    type Command = OrderCommand;
    type Event = OrderEvent;

    fn transition(
        current_state: &Self::State,
        command: Self::Command
    ) -> Result<(Self::State, Vec<Self::Event>), DomainError> {
        match (current_state, command) {
            // Draft -> Submitted
            (OrderState::Draft, OrderCommand::Submit { items }) => {
                let new_state = OrderState::Submitted { at: SystemTime::now() };
                let events = vec![OrderEvent::Submitted { items, at: SystemTime::now() }];
                Ok((new_state, events))
            },

            // Submitted -> Validated
            (OrderState::Submitted { .. }, OrderCommand::ValidatePayment { payment_id }) => {
                let new_state = OrderState::Validated { payment_id };
                let events = vec![OrderEvent::PaymentValidated { payment_id }];
                Ok((new_state, events))
            },

            // Invalid transitions return errors
            (OrderState::Fulfilled { .. }, OrderCommand::Submit { .. }) => {
                Err(DomainError::InvalidStateTransition)
            },

            // ... other transitions
        }
    }
}
```

### Transaction Boundaries

**Key Rules**:
1. **One Aggregate = One Transaction**: Each state transition is atomic
2. **State + Input = Output + Next State**: Pure Mealy machine semantics
3. **Events as Output**: State transitions produce events as side effects

```rust
// Transaction handler ensures atomicity
pub struct TransactionalCommandHandler {
    event_store: Arc<EventStore>,
}

impl TransactionalCommandHandler {
    pub async fn handle<A: StateMachineAggregate>(
        &self,
        aggregate_id: AggregateId,
        command: A::Command,
    ) -> Result<(), TransactionError> {
        // Load current state
        let current_state = self.load_aggregate_state::<A>(aggregate_id).await?;

        // Apply state machine transition
        let (new_state, events) = A::transition(&current_state, command)?;

        // Persist events atomically
        self.event_store.append_events(aggregate_id, events).await?;

        Ok(())
    }
}
```

### Non-Transactional Aggregates

Not all aggregates require transactional guarantees:

```rust
// Read-only aggregate for queries
pub struct GraphViewAggregate {
    nodes: Vec<NodeView>,
    edges: Vec<EdgeView>,
}

// Event-sourced but not transactional (eventual consistency is fine)
pub struct MetricsAggregate {
    event_count: u64,
    last_updated: SystemTime,
}

// Only aggregates with state machines enforce transactions
pub struct WorkflowAggregate {
    state_machine: WorkflowStateMachine, // This MUST be transactional
}
```

### State Machine Visualization

Since we're building a graph editor, we can visualize aggregate state machines:

```rust
// Component for state machine visualization
#[derive(Component)]
pub struct StateMachineVisual {
    states: Vec<StateNode>,
    transitions: Vec<TransitionEdge>,
    current_state: StateId,
}

// System to render state machines as graphs
fn render_state_machine_system(
    query: Query<&StateMachineVisual>,
    mut gizmos: Gizmos,
) {
    for machine in query.iter() {
        // Render states as nodes
        for state in &machine.states {
            gizmos.circle_2d(state.position, 20.0, state.color);
        }

        // Render transitions as edges
        for transition in &machine.transitions {
            gizmos.arrow_2d(
                transition.from_position,
                transition.to_position,
                Color::WHITE,
            );
        }

        // Highlight current state
        if let Some(current) = machine.states.iter().find(|s| s.id == machine.current_state) {
            gizmos.circle_2d(current.position, 25.0, Color::YELLOW);
        }
    }
}
```

### Benefits of State Machine Aggregates

1. **Explicit State Transitions**: No hidden state changes
2. **Compile-Time Safety**: Invalid transitions caught by type system
3. **Visual Debugging**: State machines can be rendered and inspected
4. **Formal Verification**: State machines can be formally analyzed
5. **Event Generation**: Events naturally flow from state transitions

## Event Store Design

### NATS JetStream Integration

Events are persisted using NATS JetStream:

```rust
pub struct EventStore {
    client: async_nats::Client,
    stream: Stream,
}

impl EventStore {
    pub async fn append_events(&self, aggregate_id: AggregateId, events: Vec<DomainEvent>) -> Result<(), EventStoreError> {
        for event in events {
            let subject = format!("{}.events.{}", aggregate_id.context(), event.event_type());

            let message = async_nats::Message {
                subject,
                payload: serde_json::to_vec(&event)?,
                headers: self.create_headers(&event),
            };

            self.stream.publish(message).await?;
        }
        Ok(())
    }

    pub async fn get_events(&self, aggregate_id: AggregateId, from_version: u64) -> Result<Vec<DomainEvent>, EventStoreError> {
        let subject = format!("{}.events.>", aggregate_id.context());

        let messages = self.stream
            .get_messages(subject, from_version)
            .await?;

        messages.into_iter()
            .map(|msg| serde_json::from_slice(&msg.payload))
            .collect()
    }
}
```

### Event Metadata

Each event carries metadata for traceability:

```rust
pub struct EventMetadata {
    pub event_id: EventId,
    pub aggregate_id: AggregateId,
    pub aggregate_version: u64,
    pub timestamp: SystemTime,
    pub correlation_id: CorrelationId,
    pub causation_id: CausationId,
    pub actor: Actor,
}
```

## Projection Patterns

### Projection Types

1. **Live Projections**: Updated in real-time as events arrive
2. **Snapshot Projections**: Periodic materialization for performance
3. **Query-Specific Projections**: Optimized for specific use cases

### Projection Handler Example

```rust
pub struct GraphVisualizationProjection {
    store: Arc<VisualizationStore>,
}

#[async_trait]
impl ProjectionHandler for GraphVisualizationProjection {
    async fn handle_event(&self, event: DomainEvent) -> Result<(), ProjectionError> {
        match event.payload {
            EventPayload::GraphCreated { id, name } => {
                self.store.create_graph_view(id, name).await?;
            }
            EventPayload::NodeAdded { graph_id, node } => {
                // Calculate visual position based on conceptual space
                let position = self.calculate_visual_position(&node).await?;
                self.store.add_node_view(graph_id, node, position).await?;
            }
            EventPayload::EdgeConnected { graph_id, edge } => {
                self.store.add_edge_view(graph_id, edge).await?;
            }
            _ => {} // Ignore events this projection doesn't care about
        }
        Ok(())
    }
}
```

## Event Ordering and Consistency

### Ordering Guarantees

1. **Per-Aggregate Ordering**: Events for a single aggregate are strictly ordered
2. **Global Ordering**: Cross-aggregate events use vector clocks
3. **Causal Ordering**: Causation IDs maintain cause-effect relationships

### Consistency Patterns

```rust
// Eventual consistency with saga pattern
pub struct WorkflowSaga {
    event_store: Arc<EventStore>,
}

impl WorkflowSaga {
    pub async fn handle_workflow_started(&self, event: WorkflowStarted) -> Result<(), SagaError> {
        // Coordinate multiple aggregates
        let graph_events = self.prepare_graph(event.workflow_id).await?;
        let concept_events = self.map_concepts(event.workflow_id).await?;

        // Transactional outbox pattern
        self.event_store.append_with_outbox(vec![
            graph_events,
            concept_events,
        ]).await?;

        Ok(())
    }
}
```

## Event Versioning

### Schema Evolution

Events must evolve without breaking existing consumers:

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum GraphNodeAdded {
    #[serde(rename = "1")]
    V1 {
        graph_id: GraphId,
        node_id: NodeId,
        label: String,
    },
    #[serde(rename = "2")]
    V2 {
        graph_id: GraphId,
        node_id: NodeId,
        label: String,
        metadata: HashMap<String, Value>, // Added in v2
    },
}

// Upcasting for compatibility
impl GraphNodeAdded {
    pub fn to_latest(self) -> GraphNodeAddedV2 {
        match self {
            GraphNodeAdded::V1 { graph_id, node_id, label } => {
                GraphNodeAddedV2 {
                    graph_id,
                    node_id,
                    label,
                    metadata: HashMap::new(), // Default for v1
                }
            }
            GraphNodeAdded::V2 { .. } => self,
        }
    }
}
```

## Testing Event-Sourced Systems

### Given-When-Then Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        // Given: No existing graph
        let aggregate = GraphAggregate::new();

        // When: Create graph command is handled
        let command = GraphCommand::CreateGraph {
            name: "Test Graph".to_string(),
        };
        let events = aggregate.handle_command(command).unwrap();

        // Then: GraphCreated event is generated
        assert_eq!(events.len(), 1);
        match &events[0] {
            GraphEvent::Created { name, .. } => {
                assert_eq!(name, "Test Graph");
            }
            _ => panic!("Expected GraphCreated event"),
        }
    }
}
```

### Event Store Testing

```rust
#[tokio::test]
async fn test_event_replay() {
    let event_store = EventStore::in_memory();
    let aggregate_id = AggregateId::new();

    // Append events
    let events = vec![
        GraphEvent::Created { id: GraphId::new(), name: "Test".into() },
        GraphEvent::NodeAdded { graph_id: GraphId::new(), node: test_node() },
    ];

    event_store.append_events(aggregate_id, events.clone()).await.unwrap();

    // Replay events
    let replayed = event_store.get_events(aggregate_id, 0).await.unwrap();
    assert_eq!(replayed.len(), 2);
}
```

## Performance Considerations

### Event Store Optimization

1. **Partitioning**: Events partitioned by aggregate type
2. **Indexing**: Secondary indexes for common queries
3. **Compression**: Event payloads compressed for storage
4. **Retention**: Old events archived to object storage

### Projection Optimization

```rust
// Batch processing for efficiency
pub struct BatchProjectionHandler {
    batch_size: usize,
    buffer: Vec<DomainEvent>,
}

impl BatchProjectionHandler {
    pub async fn handle_event(&mut self, event: DomainEvent) -> Result<(), ProjectionError> {
        self.buffer.push(event);

        if self.buffer.len() >= self.batch_size {
            self.flush().await?;
        }

        Ok(())
    }

    async fn flush(&mut self) -> Result<(), ProjectionError> {
        // Process batch of events efficiently
        let events = std::mem::take(&mut self.buffer);
        self.process_batch(events).await
    }
}
```

## Best Practices

### Do's
- ✅ Name events in past tense (Created, Added, Connected)
- ✅ Keep events small and focused
- ✅ Include all necessary data in events
- ✅ Version events from the start
- ✅ Use correlation IDs for tracing

### Don'ts
- ❌ Don't mutate events after creation
- ❌ Don't include behavior in events
- ❌ Don't create "generic" events
- ❌ Don't expose internal state
- ❌ Don't couple events to UI concerns

## Integration with CIM

Event sourcing enables CIM's key features:

1. **Time Travel**: Replay events to any point
2. **Audit Trail**: Complete history of changes
3. **Distribution**: Events naturally distribute across nodes
4. **AI Analysis**: Events provide training data
5. **Self-Reflection**: System can analyze its own event stream

This foundation ensures our CIM implementation is robust, scalable, and ready for advanced features like conceptual reasoning and AI integration.
