# Core Components Completion Plan

## Overview

This plan addresses all missing functionality identified in the gap analysis. We will implement features incrementally, with comprehensive tests for each component before moving to the next.

## Implementation Schedule

### Week 1: Core Domain Completion

#### Day 1-2: Workflow Aggregate Implementation

**File: `/cim-domain/src/workflow/aggregate.rs`**

```rust
// Complete WorkflowAggregate implementation with:
pub struct WorkflowAggregate {
    pub id: WorkflowId,
    pub name: String,
    pub description: String,
    pub steps: HashMap<StepId, WorkflowStep>,
    pub transitions: Vec<Transition>,
    pub current_state: WorkflowState,
    pub context: WorkflowContext,
    pub version: u64,
}

impl WorkflowAggregate {
    // Command handlers
    pub fn handle_create_workflow(cmd: CreateWorkflow) -> Result<Vec<WorkflowEvent>>
    pub fn handle_add_step(cmd: AddStep) -> Result<Vec<WorkflowEvent>>
    pub fn handle_add_transition(cmd: AddTransition) -> Result<Vec<WorkflowEvent>>
    pub fn handle_execute_step(cmd: ExecuteStep) -> Result<Vec<WorkflowEvent>>
    pub fn handle_complete_workflow(cmd: CompleteWorkflow) -> Result<Vec<WorkflowEvent>>

    // Event handlers
    pub fn apply_event(&mut self, event: &WorkflowEvent) -> Result<()>

    // Business rules
    fn validate_transition(&self, from: StepId, to: StepId) -> Result<()>
    fn validate_step_execution(&self, step_id: StepId) -> Result<()>
    fn check_completion_criteria(&self) -> Result<()>
}
```

**File: `/cim-domain/src/workflow/events.rs`**

```rust
// Define all workflow events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEvent {
    WorkflowCreated {
        workflow_id: WorkflowId,
        name: String,
        description: String,
        created_at: DateTime<Utc>,
    },
    StepAdded {
        workflow_id: WorkflowId,
        step: WorkflowStep,
    },
    TransitionAdded {
        workflow_id: WorkflowId,
        transition: Transition,
    },
    StepExecuted {
        workflow_id: WorkflowId,
        step_id: StepId,
        result: StepResult,
        executed_at: DateTime<Utc>,
    },
    WorkflowCompleted {
        workflow_id: WorkflowId,
        completed_at: DateTime<Utc>,
        final_context: WorkflowContext,
    },
}
```

**Tests Required**:
- Unit tests for each command handler
- Event application tests
- Business rule validation tests
- State machine transition tests

#### Day 3: Command Handler Fixes

**Update all command handlers in `/cim-domain/src/command_handlers.rs`**:

1. Add aggregate loading:
```rust
// Before
pub async fn handle_update_node(cmd: UpdateNode) -> Result<Vec<DomainEvent>> {
    // Direct event generation
}

// After
pub async fn handle_update_node(
    cmd: UpdateNode,
    event_store: &dyn EventStore,
) -> Result<Vec<DomainEvent>> {
    // Load aggregate
    let mut aggregate = event_store.load_aggregate::<GraphAggregate>(cmd.graph_id).await?;

    // Execute command
    let events = aggregate.handle_command(cmd.into())?;

    // Save events
    event_store.save_events(&aggregate.id, &events, aggregate.version).await?;

    Ok(events)
}
```

2. Add idempotency checks
3. Add transaction boundaries
4. Add proper error handling

#### Day 4-5: Basic Projections

**File: `/cim-domain/src/projections/graph_summary.rs`**

```rust
pub struct GraphSummaryProjection {
    pub graph_id: GraphId,
    pub node_count: usize,
    pub edge_count: usize,
    pub last_modified: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Projection for GraphSummaryProjection {
    async fn handle_event(&mut self, event: DomainEvent) -> Result<()>
    async fn get_checkpoint(&self) -> Option<EventSequence>
    async fn save_checkpoint(&mut self, sequence: EventSequence) -> Result<()>
}
```

**File: `/cim-domain/src/projections/node_list.rs`**
**File: `/cim-domain/src/projections/workflow_status.rs`**

### Week 2: Infrastructure and Integration

#### Day 6-7: NATS Plugin Implementation

**File: `/src/presentation/plugins/nats_plugin.rs`**

Complete implementation with:
- Subscription management
- Message routing based on subject patterns
- Error handling and retry logic
- Connection state management

#### Day 8-9: Graph Composition Operations

**File: `/cim-contextgraph/src/composition.rs`**

Implement:
- Graph union with node/edge merging
- Graph intersection with common elements
- Graph product for categorical operations
- Composition validation

#### Day 10: Integration Test Suite

**Create comprehensive tests in `/tests/integration/`**:
- `workflow_integration_tests.rs`
- `graph_operations_tests.rs`
- `projection_sync_tests.rs`
- `event_replay_tests.rs`
- `failure_recovery_tests.rs`

### Week 3: Identity Context and Polish

#### Day 11-12: Identity Context Implementation

**Implement in `/cim-identity-context/`**:
- Person aggregate with commands and events
- Organization aggregate with hierarchy
- Command and query handlers
- Repository implementations

#### Day 13: Graph Invariants

**File: `/cim-contextgraph/src/invariants.rs`**

Implement:
- Cycle detection using DFS
- Connectivity checking
- Custom invariant framework

#### Day 14-15: Testing and Documentation

- Achieve 80% test coverage
- Update all documentation
- Performance benchmarks
- Final validation

## Testing Strategy

### Unit Tests (Per Component)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_creation() {
        // Given
        let cmd = CreateWorkflow { ... };

        // When
        let events = WorkflowAggregate::handle_create_workflow(cmd).unwrap();

        // Then
        assert_eq!(events.len(), 1);
        match &events[0] {
            WorkflowEvent::WorkflowCreated { .. } => {},
            _ => panic!("Wrong event type"),
        }
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_complete_workflow_execution() {
    // Setup
    let event_store = create_test_event_store().await;
    let projections = create_test_projections().await;

    // Execute workflow
    let workflow_id = create_workflow(&event_store).await;
    add_steps(&event_store, workflow_id).await;
    execute_workflow(&event_store, workflow_id).await;

    // Verify projections updated
    let summary = projections.get_workflow_summary(workflow_id).await;
    assert_eq!(summary.status, WorkflowStatus::Completed);
}
```

## Success Criteria

### Component Completion Checklist

- [ ] WorkflowAggregate fully implemented with all handlers
- [ ] All command handlers load aggregates properly
- [ ] 3+ projections implemented and tested
- [ ] NATS plugin routing messages correctly
- [ ] Graph composition operations working
- [ ] Identity context basic functionality complete
- [ ] Graph invariants checking implemented
- [ ] Integration test suite passing
- [ ] 80%+ test coverage achieved
- [ ] No TODO/unimplemented in core modules
- [ ] All documentation updated

### Performance Targets

- Command processing: < 10ms average
- Event replay: > 10,000 events/second
- Projection update latency: < 100ms
- Memory usage: < 1GB for 100K entities

## Risk Mitigation

### Workflow Aggregate Complexity
- Start with simple state machine
- Add complex transitions incrementally
- Extensive testing for edge cases

### Integration Test Flakiness
- Use deterministic test data
- Proper test isolation
- Retry mechanisms for timing issues

### Performance Issues
- Profile early and often
- Optimize hot paths
- Consider caching strategies

## Daily Progress Tracking

Each day should:
1. Start with failing tests (TDD)
2. Implement functionality
3. Make tests pass
4. Refactor for clarity
5. Update documentation
6. Commit with descriptive message

## Deliverables

By end of Week 3:
1. All core components fully implemented
2. Comprehensive test suite with >80% coverage
3. Updated documentation
4. Performance benchmarks
5. Clean codebase with no TODOs
6. System ready for Phase 3

This plan ensures systematic completion of all missing functionality with proper testing and documentation at each step.
