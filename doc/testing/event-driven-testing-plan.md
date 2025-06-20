# Event-Driven Testing Plan for CIM

## Overview

This plan establishes a systematic approach to testing and validating the event-driven architecture of CIM, starting from the most fundamental components and building up to full system integration.

## Testing Principles

1. **Event Stream Validation**: Every test must validate that the correct events are published in the correct order
2. **User Story Driven**: Each test starts with a clear user story
3. **Bottom-Up Approach**: Start with infrastructure, then domains, then integration
4. **Event Sequence Verification**: All tests must verify the complete event sequence

## Testing Layers

### Layer 1: Infrastructure (Foundation)

#### 1.1 NATS JetStream Connection
**User Story**: As a system, I need to connect to NATS JetStream and create event streams

**Test Requirements**:
- Verify NATS connection establishment
- Verify stream creation with correct configuration
- Verify event publishing with acknowledgment
- Verify event consumption with proper ordering

**Event Sequence**:
1. `ConnectionEstablished`
2. `StreamCreated { name, subjects }`
3. `EventPublished { subject, sequence }`
4. `EventConsumed { subject, sequence }`

#### 1.2 Event Store
**User Story**: As a domain, I need to persist events with CID chains for integrity

**Test Requirements**:
- Verify event persistence with CID calculation
- Verify CID chain integrity
- Verify event replay from store
- Verify snapshot creation and restoration

**Event Sequence**:
1. `EventStoreInitialized`
2. `EventPersisted { event_id, cid, previous_cid }`
3. `CIDChainValidated { start_cid, end_cid, length }`
4. `EventsReplayed { count, aggregate_id }`

### Layer 2: Domain Fundamentals

#### 2.1 Basic Aggregate
**User Story**: As a developer, I need aggregates that process commands and emit events

**Test Requirements**:
- Create a minimal test aggregate (Counter)
- Verify command handling
- Verify event emission
- Verify state reconstruction from events

**Event Sequence**:
1. `AggregateCreated { aggregate_id }`
2. `CommandReceived { command_type, aggregate_id }`
3. `EventEmitted { event_type, aggregate_id, sequence }`
4. `StateUpdated { aggregate_id, new_state }`

#### 2.2 Command Handler
**User Story**: As a system, I need to route commands to appropriate aggregates

**Test Requirements**:
- Verify command routing
- Verify aggregate loading
- Verify event persistence after command
- Verify error handling

**Event Sequence**:
1. `CommandHandlerRegistered { command_type }`
2. `CommandRouted { command_id, aggregate_id }`
3. `AggregateLoaded { aggregate_id, version }`
4. `CommandProcessed { command_id, events_count }`

### Layer 3: Domain Implementation

#### 3.1 Graph Domain (Simplest)
**User Story**: As a user, I can create a graph with nodes and edges

**Test Requirements**:
- Create graph aggregate
- Add nodes to graph
- Connect nodes with edges
- Verify complete event stream

**Event Sequence**:
1. `GraphCreated { graph_id }`
2. `NodeAdded { graph_id, node_id, position }`
3. `NodeAdded { graph_id, node_id, position }`
4. `NodesConnected { graph_id, edge_id, source, target }`

#### 3.2 Identity Domain
**User Story**: As a user, I can register people and organizations

**Test Requirements**:
- Register person with identity
- Create organization
- Add person to organization
- Verify relationships

**Event Sequence**:
1. `PersonRegistered { person_id, identity }`
2. `OrganizationCreated { org_id, name }`
3. `OrganizationMemberAdded { org_id, person_id, role }`

### Layer 4: Cross-Domain Integration

#### 4.1 Bevy ECS Bridge
**User Story**: As a UI, I need to reflect domain events in the visual representation

**Test Requirements**:
- Domain event triggers ECS update
- ECS component changes trigger domain commands
- Bidirectional event flow
- Event ordering preservation

**Event Sequence**:
1. `DomainEventReceived { event_type, aggregate_id }`
2. `BridgeEventQueued { event_id, direction }`
3. `ECSEntitySpawned { entity_id, components }`
4. `VisualStateUpdated { entity_id }`

#### 4.2 UI Event Flow
**User Story**: As a user, I can interact with the UI and see changes reflected

**Test Requirements**:
- UI interaction generates commands
- Commands produce domain events
- Domain events update UI
- Complete round-trip validation

**Event Sequence**:
1. `UIInteraction { interaction_type, target }`
2. `CommandGenerated { command_type, payload }`
3. `DomainEventEmitted { event_type, aggregate_id }`
4. `UIUpdated { component_id, new_state }`

### Layer 5: Full System Integration

#### 5.1 Agent Interaction Flow
**User Story**: As a user, I can ask the AI agent questions and receive responses

**Test Requirements**:
- Question event generation
- NATS publication with correlation ID
- Agent processing
- Response event with causation ID
- UI update with response

**Event Sequence**:
1. `AgentQuestionEvent { question, correlation_id }`
2. `EventPublishedToNATS { subject: "cim.ui.agent.question", correlation_id }`
3. `AgentProcessingStarted { correlation_id }`
4. `AgentResponseEvent { response, correlation_id, causation_id }`
5. `EventPublishedToNATS { subject: "cim.ui.agent.response", causation_id }`
6. `UIUpdatedWithResponse { message_id }`

## Test Implementation Strategy

### Phase 1: Infrastructure Tests (Week 1)
1. Create `tests/infrastructure/` directory
2. Implement NATS connection tests
3. Implement event store tests
4. Validate CID chain functionality

### Phase 2: Domain Foundation Tests (Week 2)
1. Create `tests/domain_foundation/` directory
2. Implement minimal aggregate tests
3. Implement command handler tests
4. Validate event sourcing patterns

### Phase 3: Individual Domain Tests (Week 3-4)
1. Test each domain in isolation
2. Start with Graph (simplest)
3. Progress through Identity, Person, Agent, etc.
4. Validate domain event sequences

### Phase 4: Integration Tests (Week 5)
1. Create `tests/integration/` directory
2. Test Bevy ECS bridge
3. Test cross-domain interactions
4. Validate end-to-end flows

### Phase 5: System Tests (Week 6)
1. Full UI interaction tests
2. Agent conversation tests
3. Workflow execution tests
4. Performance validation

## Event Stream Validation Framework

```rust
/// Framework for validating event sequences in tests
pub struct EventStreamValidator {
    expected_events: Vec<ExpectedEvent>,
    captured_events: Vec<CapturedEvent>,
}

impl EventStreamValidator {
    pub fn expect_sequence(mut self, events: Vec<&str>) -> Self {
        // Define expected event sequence
        self
    }
    
    pub fn capture_from_nats(&mut self, subject: &str) -> Result<()> {
        // Capture actual events from NATS
        Ok(())
    }
    
    pub fn validate(&self) -> Result<ValidationReport> {
        // Compare expected vs actual
        // Check ordering
        // Check payloads
        // Generate report
        Ok(ValidationReport::default())
    }
}
```

## Success Criteria

Each test must:
1. Have a clear user story
2. Define expected event sequence
3. Execute the functionality
4. Capture actual event stream
5. Validate sequence matches expected
6. Verify event payloads are correct
7. Confirm correlation/causation IDs
8. Pass without any CRUD operations

## Common Test Patterns

### Pattern 1: Command-Event-Projection
```rust
#[tokio::test]
async fn test_command_produces_correct_events() {
    // Arrange
    let validator = EventStreamValidator::new()
        .expect_sequence(vec![
            "CommandReceived",
            "AggregateLoaded", 
            "EventEmitted",
            "EventPersisted",
            "ProjectionUpdated"
        ]);
    
    // Act
    send_command(CreateNode { ... }).await?;
    
    // Assert
    let report = validator
        .capture_from_nats("events.>")
        .validate()?;
    
    assert!(report.is_valid());
    assert_eq!(report.event_count(), 5);
}
```

### Pattern 2: Cross-Domain Interaction
```rust
#[tokio::test]
async fn test_cross_domain_event_flow() {
    // Arrange
    let validator = EventStreamValidator::new()
        .expect_correlation_chain(vec![
            ("PersonRegistered", None),
            ("OrganizationCreated", None),
            ("MembershipCreated", Some("PersonRegistered")),
        ]);
    
    // Act
    let person_id = register_person(...).await?;
    let org_id = create_organization(...).await?;
    add_member(org_id, person_id).await?;
    
    // Assert
    validator.validate_correlations()?;
}
```

## Debugging Failed Tests

When a test fails:
1. Examine the captured event stream
2. Compare with expected sequence
3. Check event payloads for correctness
4. Verify correlation/causation chains
5. Look for missing events
6. Check for unexpected events
7. Validate timing and ordering

## Metrics and Reporting

Track for each test:
- Event count
- Event sequence accuracy
- Correlation chain integrity
- Performance metrics
- Error rates
- Test execution time

Generate reports showing:
- Test coverage by domain
- Event flow visualization
- Performance trends
- Error patterns 