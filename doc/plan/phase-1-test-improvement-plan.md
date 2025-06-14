# Phase 1 Test Improvement Plan

## Objective

Transform our Phase 1 foundation tests from technical unit tests to user story-aligned acceptance tests that validate business value.

## User Stories for Foundation Modules

### cim-component User Stories

**US-F1: Component Attachment**
- As a developer
- I want to attach typed components to domain objects
- So that I can extend entities without modifying core structures

**US-F2: Component Type Safety**
- As a developer
- I want compile-time type safety for components
- So that I prevent runtime errors from component misuse

### cim-core-domain User Stories

**US-F3: Entity Identity Management**
- As a domain expert
- I want entities with guaranteed unique identities
- So that I can track objects across their lifecycle

**US-F4: Aggregate Root Enforcement**
- As a developer
- I want aggregate roots to control all modifications
- So that domain invariants are maintained

**US-F5: Event Sourcing Foundation**
- As a system architect
- I want entities that support event sourcing
- So that I have complete audit trails

### cim-infrastructure User Stories

**US-F6: Reliable Message Delivery**
- As a system operator
- I want guaranteed message delivery to NATS
- So that no business events are lost

**US-F7: Async/Sync Bridge**
- As a developer
- I want seamless async/sync communication
- So that I can use NATS with Bevy ECS

**US-F8: Resilient Infrastructure**
- As a system operator
- I want automatic reconnection and error recovery
- So that the system remains available

## Implementation Tasks

### Task 1: Add AsyncSyncBridge (Critical Missing Component)

```rust
// cim-infrastructure/src/bridge.rs

/// User Story: US-F7 - Async/Sync Bridge
///
/// As a developer
/// I want seamless async/sync communication
/// So that I can use NATS with Bevy ECS
///
/// ```mermaid
/// graph LR
///     BevySync[Bevy System] -->|Command| Channel1[Crossbeam Channel]
///     Channel1 --> AsyncTask[Tokio Task]
///     AsyncTask -->|Publish| NATS[NATS Server]
///     NATS -->|Event| AsyncTask2[Tokio Task]
///     AsyncTask2 --> Channel2[Tokio Channel]
///     Channel2 -->|Event| BevySync2[Bevy System]
/// ```
pub struct AsyncSyncBridge {
    command_tx: crossbeam::channel::Sender<Command>,
    command_rx: crossbeam::channel::Receiver<Command>,
    event_tx: tokio::sync::mpsc::Sender<Event>,
    event_rx: tokio::sync::mpsc::Receiver<Event>,
}

#[cfg(test)]
mod tests {
    /// Acceptance Criteria:
    /// - Commands flow from sync to async without blocking
    /// - Events flow from async to sync with batching
    /// - No data loss during high throughput
    /// - Graceful handling of channel overflow
    #[test]
    fn test_bridge_bidirectional_flow() {
        // Test implementation
    }
}
```

### Task 2: Refactor Component Tests

```rust
// cim-component/src/lib.rs

#[cfg(test)]
mod tests {
    /// User Story: US-F1 - Component Attachment
    ///
    /// As a developer
    /// I want to attach typed components to domain objects
    /// So that I can extend entities without modifying core structures
    ///
    /// ```mermaid
    /// graph TD
    ///     Entity[Domain Entity]
    ///     Comp1[Position Component]
    ///     Comp2[Metadata Component]
    ///     Entity -->|attach| Comp1
    ///     Entity -->|attach| Comp2
    /// ```
    ///
    /// Acceptance Criteria:
    /// - Components can be attached to any entity
    /// - Components are type-safe at compile time
    /// - Components can be cloned for entity duplication
    /// - Component type names are accessible for debugging
    #[test]
    fn test_component_attachment_to_entity() {
        // Given a domain entity
        let entity = TestEntity::new();

        // When I attach a component
        let position = Position3D { x: 1.0, y: 2.0, z: 3.0 };
        entity.attach_component(position);

        // Then the component is retrievable with correct type
        let retrieved = entity.get_component::<Position3D>().unwrap();
        assert_eq!(retrieved.x, 1.0);
    }
}
```

### Task 3: Add Domain Behavior Tests

```rust
// cim-core-domain/src/aggregate.rs (new file)

/// User Story: US-F4 - Aggregate Root Enforcement
///
/// ```mermaid
/// graph TD
///     Client[Client Code]
///     Root[Aggregate Root]
///     Entity1[Child Entity 1]
///     Entity2[Child Entity 2]
///
///     Client -->|Commands| Root
///     Root -->|Controls| Entity1
///     Root -->|Controls| Entity2
///     Root -->|Emits| Events[Domain Events]
/// ```
pub trait CommandHandler {
    type Command;
    type Event;
    type Error;

    fn handle_command(&mut self, cmd: Self::Command) -> Result<Vec<Self::Event>, Self::Error>;
}

#[cfg(test)]
mod tests {
    /// Acceptance Criteria:
    /// - All modifications go through aggregate root
    /// - Commands generate domain events
    /// - Invalid commands are rejected with errors
    /// - Aggregate version increments on changes
    #[test]
    fn test_aggregate_command_handling() {
        // Test implementation
    }
}
```

### Task 4: Add Event Store Abstraction

```rust
// cim-infrastructure/src/event_store.rs (new file)

/// User Story: US-F5 - Event Sourcing Foundation
///
/// ```mermaid
/// sequenceDiagram
///     participant Cmd as Command
///     participant Agg as Aggregate
///     participant Store as EventStore
///     participant NATS as NATS JetStream
///
///     Cmd->>Agg: handle_command()
///     Agg->>Agg: validate & apply
///     Agg->>Store: append_events()
///     Store->>NATS: publish to stream
///     Store-->>Agg: success
/// ```
#[async_trait]
pub trait EventStore {
    async fn append_events(&self, events: Vec<DomainEvent>) -> Result<(), EventStoreError>;
    async fn load_events(&self, aggregate_id: &str) -> Result<Vec<DomainEvent>, EventStoreError>;
}
```

### Task 5: Integration Test Suite

```rust
// tests/integration/phase_1_integration.rs

/// User Story: US-F6 - Reliable Message Delivery
///
/// ```mermaid
/// graph TD
///     subgraph "Test Scenario"
///         Cmd[Command] --> Bridge[AsyncSyncBridge]
///         Bridge --> NATS[NATS Server]
///         NATS --> Store[Event Store]
///         Store --> Proj[Projection]
///     end
/// ```
#[tokio::test]
async fn test_end_to_end_command_processing() {
    // Given a running NATS server
    let nats = start_test_nats_server().await;

    // And our infrastructure components
    let bridge = AsyncSyncBridge::new();
    let event_store = NatsEventStore::new(nats.client());

    // When a command is sent
    let command = CreateEntity { id: "test-123" };
    bridge.send_command(command).await.unwrap();

    // Then the event appears in the store
    let events = event_store.load_events("test-123").await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].entity_id, "test-123");
}
```

## Test Documentation Standards

Every test must include:

1. **User Story Header**
   ```rust
   /// User Story: US-XX - Story Name
   ```

2. **Given/When/Then Format**
   ```rust
   /// As a [role]
   /// I want [feature]
   /// So that [benefit]
   ```

3. **Mermaid Diagram**
   ```rust
   /// ```mermaid
   /// [diagram showing test scenario]
   /// ```
   ```

4. **Acceptance Criteria**
   ```rust
   /// Acceptance Criteria:
   /// - Specific measurable criteria
   /// - That validate the story
   ```

## Success Metrics

- [ ] 100% of tests have user story references
- [ ] 100% of tests have mermaid diagrams
- [ ] AsyncSyncBridge implemented and tested
- [ ] Event store abstraction implemented
- [ ] Integration tests pass with real NATS
- [ ] All acceptance criteria documented

## Timeline

1. **Day 1**: Implement AsyncSyncBridge with tests
2. **Day 2**: Refactor existing tests with user stories
3. **Day 3**: Add domain behavior tests
4. **Day 4**: Add event store abstraction
5. **Day 5**: Create integration test suite

## Next Steps After Completion

Once Phase 1 tests are properly aligned:
1. Proceed to Phase 2: Create bounded contexts
2. Each context will have its own user stories
3. Tests will validate cross-context communication
4. Full end-to-end scenarios will be tested
