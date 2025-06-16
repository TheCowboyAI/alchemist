# Integration Tests and Read Model Projections Implementation Plan

## Overview

This plan addresses two Priority 1 items from our QA remediation:
1. Create integration test suite with NATS end-to-end tests
2. Implement read model projections for CQRS queries

## Projection Types Clarification

### Internal Read Model Projections (This Document)
- **Purpose**: Optimize query performance within the CIM system
- **Location**: In-memory or local storage (NATS KV)
- **Examples**: GraphSummaryProjection, NodeListProjection, EdgeConnectionProjection
- **Usage**: Support fast queries for UI, search, and graph operations

### External System Projections (See: multi-system-projections-plan.md)
- **Purpose**: Integrate with external domain modules
- **Location**: External systems via NATS messaging
- **Examples**: GraphPersistence Module, WorkflowOrchestration Module
- **Usage**: Bidirectional data flow with external capabilities

## Current State

### What We Have
- Basic NATS integration tests (connection, publishing, consuming)
- Event flow tests (command to event store)
- Placeholder projection tests (not implemented)
- Basic GraphProjection structure (no event handlers)
- Graph Aggregate with full command handling

### What We Need
- Complete end-to-end integration tests covering all layers
- Event handlers for updating projections
- Query handlers for reading from projections
- Projection persistence and replay capability

## Implementation Tasks

### Phase 1: Read Model Projections (2 days)

#### Task 1.1: Implement GraphSummaryProjection
- Create projection that maintains graph metadata and statistics
- Handle GraphCreated, GraphUpdated, GraphDeleted events
- Track node count, edge count, last modified time
- Store in-memory with periodic snapshots

#### Task 1.2: Implement NodeListProjection
- Create projection for fast node lookups
- Index nodes by type, content, and position
- Handle NodeAdded, NodeUpdated, NodeRemoved events
- Support search and filtering queries

#### Task 1.3: Implement EdgeConnectionProjection
- Create projection for graph connectivity queries
- Maintain adjacency lists for fast traversal
- Handle EdgeConnected, EdgeDisconnected events
- Support path finding and neighbor queries

#### Task 1.4: Create Projection Event Handlers
- Implement EventHandler trait for each projection
- Subscribe to appropriate event streams
- Handle checkpointing for replay
- Add error handling and retry logic

### Phase 2: Integration Test Suite (2 days)

#### Task 2.1: End-to-End Command Flow Tests
- Test complete flow: Bevy Command → NATS → Event Store → Projection → Query
- Cover all command types (Graph, Node, Edge)
- Verify event ordering and CID chains
- Test concurrent command processing

#### Task 2.2: Projection Update Tests
- Test that projections update correctly from events
- Verify eventual consistency
- Test projection replay from checkpoint
- Test multiple projections from same event stream

#### Task 2.3: Query Handler Tests
- Test GraphSummaryQuery returns correct data
- Test NodeSearchQuery with various filters
- Test PathFindingQuery for graph traversal
- Test performance with large datasets

#### Task 2.4: Error Recovery Tests
- Test NATS disconnection and reconnection
- Test projection recovery from crash
- Test handling of out-of-order events
- Test compensation for failed commands

### Phase 3: Integration with Bevy (1 day)

#### Task 3.1: Connect Projections to Bevy Resources
- Update GraphProjection resource from event handlers
- Ensure thread-safe updates
- Implement change detection for UI updates

#### Task 3.2: Create Query Systems
- Implement Bevy systems that read from projections
- Add caching for frequently accessed data
- Ensure read-only access patterns

## Success Criteria

### For Projections
- [ ] All domain events update appropriate projections
- [ ] Projections can be replayed from any checkpoint
- [ ] Query performance < 10ms for common queries
- [ ] Memory usage scales linearly with data size

### For Integration Tests
- [ ] 100% coverage of command → event → projection flow
- [ ] All error scenarios tested
- [ ] Tests run in < 30 seconds
- [ ] No flaky tests

## Technical Considerations

### Projection Design
```rust
#[async_trait]
pub trait Projection: Send + Sync {
    type Event;

    async fn handle_event(&mut self, event: Self::Event) -> Result<()>;
    async fn get_checkpoint(&self) -> Option<EventSequence>;
    async fn save_checkpoint(&mut self, sequence: EventSequence) -> Result<()>;
}
```

### Integration Test Pattern
```rust
#[tokio::test]
async fn test_end_to_end_flow() {
    // 1. Setup infrastructure
    let (nats, event_store, projections) = setup_test_env().await;

    // 2. Submit command
    let command = CreateGraphCommand { ... };
    let handler = GraphCommandHandler::new(event_store.clone());

    // 3. Process command
    let events = handler.handle(command).await?;

    // 4. Wait for projection update
    wait_for_projection_update(&projections, Duration::from_millis(100)).await;

    // 5. Query projection
    let query = GraphSummaryQuery { graph_id };
    let result = projections.graph_summary.query(query).await?;

    // 6. Assert results
    assert_eq!(result.name, "expected-name");
}
```

## Timeline

- Day 1-2: Implement projections and event handlers
- Day 3-4: Create comprehensive integration tests
- Day 5: Integrate with Bevy and final testing

Total: 5 days

## Dependencies

- Graph Aggregate implementation (✅ Complete)
- NATS infrastructure (✅ Complete)
- Event Store (✅ Complete)
- Event Bridge (✅ Complete)

## Next Steps

1. Start with GraphSummaryProjection implementation
2. Create projection event handler infrastructure
3. Write first end-to-end integration test
4. Iterate on remaining projections and tests
