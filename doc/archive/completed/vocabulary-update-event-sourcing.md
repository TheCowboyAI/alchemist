# Vocabulary Update for Event Sourcing Architecture

## Summary

Updated the CIM vocabulary documentation to reflect the new event-sourcing architecture and removed deprecated graph management references from the legacy system.

## Changes Made

### Added Event Sourcing Terms Section
- Domain Event
- Event Store
- Event Envelope
- Aggregate
- Aggregate Root
- Command Handler
- Projection
- Event Sourcing
- CQRS

### Updated Technical Terms
- Added Petgraph (graph data structure library)
- Added Bevy ECS (visualization framework)

### Updated Implementation Components
- Added Read Model
- Added Repository pattern

### Transformed Graph Domain
- Changed from direct mutation model to event-sourced model
- Updated all identifiers from *Identity to *Id (following Rust conventions)
- Changed from mutable entities to event-sourced aggregates
- Removed deprecated visualization components (GraphMotion, SubgraphOrbit, NodePulse)
- Removed deprecated services that don't align with event sourcing

### Added New Sections
- **Domain Events**: GraphCreated, NodeAdded, EdgeConnected, etc.
- **Commands**: CreateGraph, AddNode, ConnectNodes
- **Infrastructure Components**: EventStore, GraphRepository, GraphReadModel, GraphCommandHandler
- **Bevy Integration Components**: GraphNode, GraphEdge, DomainEventOccurred, EventBridge

### Updated Code References
- Changed from `src/contexts/*/` to new structure:
  - `src/domain/*` for domain objects
  - `src/infrastructure/*` for infrastructure
  - `src/application/*` for application services
  - `src/presentation/*` for UI/visualization

## Rationale

The vocabulary now accurately reflects:
1. Event sourcing as the primary architectural pattern
2. Clear separation between domain events and ECS events
3. CQRS pattern with separate command and query models
4. Petgraph as the graph storage engine
5. Bevy ECS for visualization only (not domain logic)

## Impact

This vocabulary update ensures that all documentation and communication about the system uses consistent terminology aligned with the event-sourcing architecture. It removes confusion from legacy terms and provides clear definitions for the new architectural components.

## Date Completed

December 4, 2024
