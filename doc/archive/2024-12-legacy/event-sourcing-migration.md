# Event Sourcing Migration Progress

## Overview

Tracking the migration from legacy architecture to Event Sourcing with petgraph and CQRS.

## Migration Status

### ✅ Completed
- [x] Archive legacy implementation
- [x] Create new architecture design
- [x] Create implementation plan
- [x] Set up new project structure
- [x] Basic skeleton compiles

### 🚧 In Progress
- [ ] Phase 1: Core Event Infrastructure (Week 1)

### 📅 Upcoming
- [ ] Phase 2: Graph Aggregate with Petgraph
- [ ] Phase 3: Read Model and Queries
- [ ] Phase 4: Bevy Integration
- [ ] Phase 5: Feature Migration
- [ ] Phase 6: Performance and Polish

## Phase 1 Progress: Core Event Infrastructure

### Domain Events (0%)
- [ ] Create event type definitions
- [ ] Implement EventEnvelope
- [ ] Add event metadata
- [ ] Define all domain events

### Event Store (0%)
- [ ] In-memory event log
- [ ] Sequence numbering
- [ ] Event persistence trait
- [ ] JSON file persistence

### Command Model (0%)
- [ ] Command type definitions
- [ ] Command validation
- [ ] Command handler
- [ ] Error handling

### Testing (0%)
- [ ] Unit tests for events
- [ ] Event store tests
- [ ] Command handler tests
- [ ] Integration tests

## Architecture Decisions

### Decision: Local Event Store First
**Date**: December 2024
**Status**: Approved
**Rationale**: Start with a simple local event store before adding distributed capabilities. This allows us to focus on the core event sourcing patterns without the complexity of NATS.

### Decision: Petgraph for Graph Storage
**Date**: December 2024
**Status**: Approved
**Rationale**: Petgraph provides efficient graph algorithms and stable indices, which are crucial for our performance targets of 100K+ nodes.

### Decision: Separate Domain and ECS Events
**Date**: December 2024
**Status**: Approved
**Rationale**: Not all UI interactions need to be persisted. Mouse movements, hover states, etc. should remain as ephemeral ECS events.

## Next Steps

1. **Implement Domain Events**
   - Start with core event types
   - Add serialization support
   - Create event builders

2. **Build Event Store**
   - Simple in-memory implementation
   - Add persistence layer
   - Create event indices

3. **Create Command Model**
   - Define command types
   - Implement validation
   - Build command handlers

## Notes

- Following TDD approach: write tests first
- Maintaining DDD naming conventions throughout
- Focusing on simplicity in Phase 1
- Performance optimization deferred to Phase 6

## Blockers

None currently.

## Resources

- [Architecture Design](../design/event-sourced-graph-architecture.md)
- [Implementation Plan](../plan/event-sourcing-implementation-plan.md)
- [Legacy Features](../archive/2024-12-legacy/FEATURE-SET.md)
