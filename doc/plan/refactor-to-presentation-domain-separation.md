# Refactor Plan: Presentation vs Domain Event Separation

## Overview

This plan outlines the refactoring needed to properly separate presentation events from domain events, implement graph model recognition, and support structure-preserving morphisms.

## Phase 1: Event Classification

### 1.1 Create Event Categories

**Files to create:**
- `src/presentation/events/mod.rs` - Presentation event types
- `src/presentation/events/animation.rs` - Animation events
- `src/presentation/events/interaction.rs` - User interaction events
- `src/presentation/events/layout.rs` - Layout calculation events

**Key changes:**
- Move UI-specific events out of domain layer
- Create clear event type hierarchies
- Implement event aggregation patterns

### 1.2 Refactor Domain Events

**Files to modify:**
- `src/domain/events/mod.rs` - Remove presentation concerns
- `src/domain/events/graph.rs` - Focus on business state changes
- `src/domain/events/node.rs` - Remove position updates (aggregate them)
- `src/domain/events/edge.rs` - Keep only structural changes

**Key changes:**
- Remove `NodeMoved` as individual event
- Add `NodesPositioned` for batch position updates
- Add `GraphModelRecognized` event
- Add `GraphMorphismApplied` event

## Phase 2: Graph Model Recognition

### 2.1 Implement Graph Models

**Files to create:**
- `src/domain/models/mod.rs` - Graph model types
- `src/domain/models/complete.rs` - Complete graph (Kn) implementation
- `src/domain/models/cycle.rs` - Cycle graph (Cn) implementation
- `src/domain/models/state_machine.rs` - Mealy/Moore machines
- `src/domain/models/domain_specific.rs` - Address, Workflow, etc.

**Key types:**
```rust
pub enum GraphModel {
    Complete(usize),
    Cycle(usize),
    MealyMachine(MealySpec),
    MooreMachine(MooreSpec),
    AddressGraph(AddressTemplate),
    WorkflowGraph(WorkflowTemplate),
    Custom(String, CustomSpec),
}
```

### 2.2 Add Recognition Logic

**Files to modify:**
- `src/domain/aggregates/graph.rs` - Add `recognize_model()` method
- `src/domain/services/graph_analyzer.rs` - Create pattern matching service

**Key methods:**
```rust
impl Graph {
    pub fn recognize_model(&self) -> Option<GraphModel>;
    pub fn validate_model(&self, model: &GraphModel) -> Result<()>;
    pub fn from_model(model: GraphModel) -> Self;
}
```

## Phase 3: Structure-Preserving Morphisms

### 3.1 Define Morphisms

**Files to create:**
- `src/domain/morphisms/mod.rs` - Morphism types and traits
- `src/domain/morphisms/complete.rs` - Complete graph morphisms
- `src/domain/morphisms/cycle.rs` - Cycle graph morphisms
- `src/domain/morphisms/state_machine.rs` - State machine transformations

**Key types:**
```rust
pub enum GraphMorphism {
    ToComplete,
    ToCycle,
    SubdivideEdges(usize),
    ContractEdges(Vec<EdgeId>),
    MealyToMoore,
    MooreToMealy,
    // ... etc
}
```

### 3.2 Implement Morphism Application

**Files to modify:**
- `src/domain/aggregates/graph.rs` - Add `apply_morphism()` method

**Key methods:**
```rust
impl Graph {
    pub fn apply_morphism(&mut self, morphism: GraphMorphism) -> Result<Vec<DomainEvent>>;
    pub fn can_apply_morphism(&self, morphism: &GraphMorphism) -> bool;
}
```

## Phase 4: Presentation Layer Refactoring

### 4.1 Event Aggregation

**Files to create:**
- `src/presentation/aggregators/mod.rs` - Event aggregation logic
- `src/presentation/aggregators/drag.rs` - Drag operation aggregator
- `src/presentation/aggregators/layout.rs` - Layout change aggregator

**Key patterns:**
```rust
pub struct DragAggregator {
    movements: HashMap<NodeId, Vec<Position3D>>,
    start_time: Instant,
}

impl DragAggregator {
    pub fn add_movement(&mut self, node_id: NodeId, position: Position3D);
    pub fn complete(&self) -> Option<DomainCommand>;
}
```

### 4.2 Bevy System Updates

**Files to modify:**
- `src/presentation/bevy_systems/interaction.rs` - Use aggregators
- `src/presentation/bevy_systems/animation.rs` - Keep local
- `src/presentation/bevy_systems/layout.rs` - Aggregate before sending

**Key changes:**
- Replace direct domain event sending with aggregation
- Add "Save" or "Commit" user actions
- Implement preview states

## Phase 5: Testing Updates

### 5.1 Domain Tests

**Files to create/modify:**
- `tests/domain/graph_models.rs` - Model recognition tests
- `tests/domain/morphisms.rs` - Morphism application tests
- `tests/domain/event_aggregation.rs` - Event aggregation tests

### 5.2 Integration Tests

**Files to create:**
- `tests/integration/presentation_domain_boundary.rs` - Test event flow
- `tests/integration/model_persistence.rs` - Test model save/load

## Phase 6: Documentation Updates

### 6.1 Update Examples

**Files to modify:**
- `examples/create_k7.rs` - Show model-based creation
- `examples/apply_morphism.rs` - Demonstrate transformations
- `examples/drag_and_save.rs` - Show aggregation pattern

### 6.2 Update README

**Files to modify:**
- `README.md` - Add section on event separation
- `doc/user-guide/graph-models.md` - User documentation

## Implementation Order

1. **Week 1**: Phase 1 - Event Classification
   - Create presentation event types
   - Refactor domain events
   - Update event handlers

2. **Week 2**: Phase 2 - Graph Model Recognition
   - Implement model types
   - Add recognition logic
   - Create model templates

3. **Week 3**: Phase 3 - Structure-Preserving Morphisms
   - Define morphism types
   - Implement transformations
   - Add validation

4. **Week 4**: Phase 4 - Presentation Layer
   - Implement aggregators
   - Update Bevy systems
   - Add save/commit UI

5. **Week 5**: Phase 5 & 6 - Testing and Documentation
   - Write comprehensive tests
   - Update documentation
   - Create examples

## Success Criteria

1. **Clear Separation**: No presentation events in domain layer
2. **Model Recognition**: Can identify K7, C5, state machines, etc.
3. **Morphism Support**: Can transform between compatible models
4. **Performance**: Animation at 60fps without domain events
5. **User Experience**: Clear save/commit workflow

## Risks and Mitigations

1. **Risk**: Breaking existing functionality
   - **Mitigation**: Comprehensive test coverage before refactoring

2. **Risk**: Performance regression
   - **Mitigation**: Benchmark before/after, optimize aggregators

3. **Risk**: User confusion with new save workflow
   - **Mitigation**: Clear UI indicators, good documentation

## Notes

- This refactoring aligns with DDD principles
- Maintains Event Sourcing integrity
- Improves performance by reducing event volume
- Enables powerful graph transformations
- Sets foundation for AI agent integration
