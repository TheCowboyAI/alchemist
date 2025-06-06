# Phase 1: Core Event Infrastructure

## Overview

Implementing the foundation of Event Sourcing for Information Alchemist. This phase focuses on domain events, event store, and command model without any external dependencies.

## Phase Status: 🚧 In Progress (0% Complete)

**Start Date**: December 2024
**Target Completion**: Week 1
**Actual Progress**: Just started

## Task Breakdown

### 1.1 Domain Events (0%)

#### Event Type Definitions
- [ ] Create base event types
  - [ ] `GraphCreated`
  - [ ] `GraphRenamed`
  - [ ] `GraphDeleted`
  - [ ] `NodeAdded`
  - [ ] `NodeUpdated`
  - [ ] `NodeRemoved`
  - [ ] `EdgeConnected`
  - [ ] `EdgeUpdated`
  - [ ] `EdgeDisconnected`
  - [ ] `NodeBatchAdded`
  - [ ] `EdgeBatchConnected`
  - [ ] `LayoutApplied`

#### Event Infrastructure
- [ ] Create `EventId` value object
- [ ] Create `EventEnvelope` structure
- [ ] Add event metadata types
- [ ] Implement serialization (serde)
- [ ] Add event builders/constructors

#### Testing
- [ ] Unit tests for each event type
- [ ] Serialization/deserialization tests
- [ ] Event metadata tests

**Files to create**:
- `src/domain/events/mod.rs`
- `src/domain/events/graph_events.rs`
- `src/domain/events/node_events.rs`
- `src/domain/events/edge_events.rs`
- `src/domain/events/metadata.rs`

### 1.2 Event Store (0%)

#### Core Implementation
- [ ] Create `EventStore` struct
- [ ] Implement in-memory event log
- [ ] Add sequence numbering
- [ ] Create event indices
  - [ ] By aggregate ID
  - [ ] By sequence number
  - [ ] By timestamp

#### Persistence Layer
- [ ] Define `EventPersistence` trait
- [ ] Implement `JsonFilePersistence`
- [ ] Add file I/O operations
- [ ] Handle persistence errors

#### Event Store Operations
- [ ] `append()` - Add new events
- [ ] `get_events()` - Query by sequence
- [ ] `get_events_for_aggregate()` - Query by aggregate
- [ ] `get_latest_sequence()` - Get current position

#### Testing
- [ ] Unit tests for event store
- [ ] Persistence tests
- [ ] Concurrent access tests
- [ ] Index performance tests

**Files to create**:
- `src/infrastructure/event_store/mod.rs`
- `src/infrastructure/event_store/store.rs`
- `src/infrastructure/event_store/indices.rs`
- `src/infrastructure/persistence/mod.rs`
- `src/infrastructure/persistence/json.rs`

### 1.3 Command Model (0%)

#### Command Types
- [ ] Create base command enum
- [ ] Define all command variants
  - [ ] `CreateGraph`
  - [ ] `RenameGraph`
  - [ ] `DeleteGraph`
  - [ ] `AddNode`
  - [ ] `UpdateNode`
  - [ ] `RemoveNode`
  - [ ] `ConnectNodes`
  - [ ] `DisconnectNodes`
  - [ ] `ImportGraph`
  - [ ] `ApplyLayout`

#### Command Handler
- [ ] Create `GraphCommandHandler` struct
- [ ] Implement command validation
- [ ] Add event generation logic
- [ ] Handle command errors

#### Command Processing
- [ ] Command-to-event mapping
- [ ] Business rule validation
- [ ] Error handling strategy
- [ ] Command result types

#### Testing
- [ ] Unit tests for commands
- [ ] Validation tests
- [ ] Command handler tests
- [ ] Integration tests

**Files to create**:
- `src/domain/commands/mod.rs`
- `src/domain/commands/graph_commands.rs`
- `src/domain/commands/validation.rs`
- `src/application/command_handlers/mod.rs`
- `src/application/command_handlers/graph_handler.rs`

### 1.4 Value Objects (0%)

#### Core Types
- [ ] `GraphId`
- [ ] `NodeId`
- [ ] `EdgeId`
- [ ] `ComponentId`
- [ ] `Position3D`
- [ ] `GraphMetadata`
- [ ] `NodeContent`
- [ ] `EdgeRelationship`

#### Testing
- [ ] Value object tests
- [ ] Equality tests
- [ ] Serialization tests

**Files to create**:
- `src/domain/value_objects/mod.rs`
- `src/domain/value_objects/identities.rs`
- `src/domain/value_objects/spatial.rs`
- `src/domain/value_objects/metadata.rs`

## Test Coverage Goals

- Domain Events: 100%
- Event Store: 90%+
- Command Model: 90%+
- Value Objects: 100%

## Performance Benchmarks

### Target Metrics
- Event append: < 1ms
- Event query by ID: < 100μs
- Aggregate event loading: < 10ms for 1000 events
- JSON persistence: < 5ms per event

### Current Metrics
- Not yet measured

## Dependencies

### Required Crates
- [x] `serde` - Serialization
- [x] `serde_json` - JSON support
- [x] `uuid` - ID generation
- [ ] `chrono` or `time` - Timestamps
- [ ] `thiserror` - Error handling
- [ ] `dashmap` - Concurrent indices
- [ ] `parking_lot` - Better RwLock

### To Add to Cargo.toml
```toml
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2.0"
dashmap = "6.1"
parking_lot = "0.12"
```

## Architecture Decisions

### Decision: Use StableGraph from petgraph
**Rationale**: Provides stable node/edge indices even after removals, which is crucial for event sourcing where we need consistent references.

### Decision: JSON for initial persistence
**Rationale**: Human-readable, easy to debug, and sufficient for local development. Can migrate to binary format later if needed.

### Decision: Sync API initially
**Rationale**: Simpler to implement and test. Async can be added in the infrastructure layer without affecting domain logic.

## Next Steps

1. Start with value objects (foundation)
2. Implement domain events
3. Build event store
4. Create command model
5. Write comprehensive tests

## Blockers

None currently.

## Notes

- Following TDD: Write tests first
- Keep domain layer pure (no I/O, no Bevy dependencies)
- Use newtype pattern for all IDs
- Ensure all events are immutable
- Document public APIs thoroughly

## Daily Progress Log

### Day 1 (Starting)
- [ ] Set up value objects module
- [ ] Create basic ID types
- [ ] Write first tests

---

**Last Updated**: Just now
**Next Review**: End of Day 1
