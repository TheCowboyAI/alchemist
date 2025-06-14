# Phase 1 Foundation User Stories

## Overview

These user stories define the expected behavior and business value of our Phase 1 foundation modules: `cim-component`, `cim-core-domain`, and `cim-infrastructure`. Each story includes acceptance criteria and test scenarios.

## cim-component User Stories

### Story F1: Extensible Domain Objects
**As a** domain modeler
**I want** to attach typed components to any domain object
**So that** I can extend entities without modifying their core structure

**Acceptance Criteria:**
- Components can be attached to any entity that implements the component container trait
- Components are type-safe and cannot be confused at compile time
- Components can be cloned when duplicating entities
- Component type names are available for debugging and serialization

**Test Scenarios:**
```rust
// Given a graph node entity
let mut node = NodeEntity::new();

// When I attach a Position component
node.attach_component(Position3D { x: 1.0, y: 2.0, z: 3.0 });

// Then I can retrieve it with the correct type
let pos = node.get_component::<Position3D>().unwrap();
assert_eq!(pos.x, 1.0);

// And I cannot retrieve it as a different type
assert!(node.get_component::<Velocity>().is_none());
```

### Story F2: Component Discovery
**As a** system developer
**I want** to discover what components are attached to an entity
**So that** I can build generic systems that work with any component

**Acceptance Criteria:**
- Can list all component type names on an entity
- Can check if a specific component type exists
- Can iterate over all components (as trait objects)
- Performance is O(1) for component lookups

**Test Scenarios:**
```rust
// Given an entity with multiple components
let entity = create_test_entity()
    .with_component(Position3D::default())
    .with_component(Label("Test Node".to_string()))
    .with_component(Color::RED);

// When I query component types
let types = entity.component_types();

// Then I see all attached components
assert!(types.contains("Position3D"));
assert!(types.contains("Label"));
assert!(types.contains("Color"));
```

## cim-core-domain User Stories

### Story F3: Unique Entity Identity
**As a** system architect
**I want** every entity to have a globally unique, immutable identifier
**So that** I can track entities across distributed systems and time

**Acceptance Criteria:**
- Entity IDs are UUIDs that are globally unique
- IDs are immutable - once created, they cannot be changed
- IDs are type-safe - NodeId cannot be used where EdgeId is expected
- IDs can be serialized/deserialized for persistence

**Test Scenarios:**
```rust
// Given two entities
let node1 = Entity::<NodeMarker>::new();
let node2 = Entity::<NodeMarker>::new();

// Then their IDs are unique
assert_ne!(node1.id, node2.id);

// And IDs cannot be modified
// node1.id = node2.id; // This should not compile

// And type safety is enforced
let edge = Entity::<EdgeMarker>::new();
// let mixed: NodeId = edge.id; // This should not compile
```

### Story F4: Aggregate Root Control
**As a** domain expert
**I want** all changes to go through aggregate roots
**So that** business invariants are always maintained

**Acceptance Criteria:**
- Aggregates implement the AggregateRoot trait
- All modifications generate domain events
- Version numbers increment with each change
- Child entities can only be modified through the root

**Test Scenarios:**
```rust
// Given a graph aggregate
let mut graph = GraphAggregate::new();

// When I add a node through the aggregate
let events = graph.handle_command(AddNode {
    position: Position3D::ZERO
})?;

// Then the version increments
assert_eq!(graph.version(), 1);

// And appropriate events are generated
assert_eq!(events.len(), 1);
assert!(matches!(events[0], DomainEvent::NodeAdded { .. }));
```

### Story F5: Domain Error Handling
**As a** developer
**I want** clear, typed errors for domain violations
**So that** I can handle different error cases appropriately

**Acceptance Criteria:**
- Each error type has a specific variant in DomainError
- Errors include context about what went wrong
- Errors can be converted to user-friendly messages
- Errors are serializable for API responses

**Test Scenarios:**
```rust
// Given a graph at capacity
let mut graph = create_full_graph(); // 10,000 nodes

// When I try to add another node
let result = graph.handle_command(AddNode { .. });

// Then I get a specific error
assert!(matches!(
    result,
    Err(DomainError::NodeLimitExceeded { limit: 10000, .. })
));
```

## cim-infrastructure User Stories

### Story F6: NATS Message Publishing
**As a** system operator
**I want** reliable message publishing to NATS
**So that** domain events reach all interested systems

**Acceptance Criteria:**
- Messages are published with appropriate subjects
- Serialization errors are caught and reported
- Connection failures trigger reconnection attempts
- Published messages can be confirmed (when using JetStream)

**Test Scenarios:**
```rust
// Given a NATS client
let client = NatsClient::new(config).await?;

// When I publish a domain event
let event = NodeAdded { node_id: NodeId::new(), .. };
client.publish("events.graph.node.added", &event).await?;

// Then subscribers receive the event
let mut sub = client.subscribe("events.graph.>").await?;
let msg = sub.next().await.unwrap();
let received: NodeAdded = serde_json::from_slice(&msg.payload)?;
assert_eq!(received.node_id, event.node_id);
```

### Story F7: Event Bridge Operation
**As a** Bevy developer
**I want** seamless async/sync event bridging
**So that** I can use NATS in my ECS systems

**Acceptance Criteria:**
- Commands flow from Bevy (sync) to NATS (async) without blocking
- Events flow from NATS (async) to Bevy (sync) with batching
- Bridge handles backpressure appropriately
- No events are lost during high throughput

**Test Scenarios:**
```rust
// Given an event bridge (already implemented with crossbeam)
let bridge = EventBridge::new();

// When I send a command from Bevy
bridge.send_command(BridgeCommand::PublishEvent(event))?;

// Then it reaches NATS without blocking the game loop
// (verified by performance metrics)

// And when NATS publishes events
// Then Bevy receives them in batches
let events = bridge.receive_events(); // Non-blocking
assert!(!events.is_empty());
```

### Story F8: Message Handler Processing
**As a** service developer
**I want** a framework for processing NATS messages
**So that** I can focus on business logic, not infrastructure

**Acceptance Criteria:**
- Handlers are registered for specific subjects
- Messages are automatically deserialized to the correct type
- Errors in handlers don't crash the processor
- Multiple handlers can process the same message type

**Test Scenarios:**
```rust
// Given a message handler
struct NodeEventHandler;

impl MessageHandler for NodeEventHandler {
    type Message = NodeEvent;

    async fn handle(&self, event: NodeEvent) -> Result<()> {
        // Process the event
        Ok(())
    }

    fn subject(&self) -> &str {
        "events.graph.node.>"
    }
}

// When I run the processor
let processor = MessageProcessor::new(client);
processor.run_handler(Arc::new(NodeEventHandler)).await?;

// Then it processes matching messages
// And continues running even if some messages fail
```

### Story F9: Resilient Connections
**As a** system operator
**I want** automatic reconnection to NATS
**So that** temporary network issues don't require manual intervention

**Acceptance Criteria:**
- Client reconnects automatically on connection loss
- Reconnection uses exponential backoff
- Subscriptions are restored after reconnection
- Connection status is observable

**Test Scenarios:**
```rust
// Given a NATS client
let client = NatsClient::new(config).await?;

// When the connection is lost
// (simulated by stopping NATS server)

// Then the client attempts reconnection
// And eventually reconnects when server returns
// And subscriptions continue working
```

## Cross-Module Integration Stories

### Story F10: Component-Based Entities
**As a** domain modeler
**I want** to use components with domain entities
**So that** I can build flexible, extensible domain models

**Acceptance Criteria:**
- Domain entities can have components attached
- Components are preserved through event sourcing
- Components can be queried efficiently
- Type safety is maintained throughout

**Test Scenarios:**
```rust
// Given a domain entity with components
let mut node = Entity::<NodeMarker>::new();
node.attach_component(Position3D { x: 1.0, y: 2.0, z: 3.0 });
node.attach_component(Label("Origin".to_string()));

// When I persist and reload through events
let events = create_events_from_entity(&node);
let restored = restore_entity_from_events(events)?;

// Then components are preserved
assert_eq!(
    restored.get_component::<Position3D>(),
    Some(&Position3D { x: 1.0, y: 2.0, z: 3.0 })
);
```

## Test Coverage Requirements

Each user story must have:
1. **Unit tests** verifying the acceptance criteria
2. **Integration tests** showing cross-module interaction
3. **Documentation tests** demonstrating usage
4. **Property tests** for invariants (where applicable)

## Success Metrics

- 100% of acceptance criteria have corresponding tests
- All tests include user story references
- All tests include mermaid diagrams
- Test names clearly indicate what user story they validate
- Tests use Given/When/Then structure
