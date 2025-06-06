# Design Documentation

This directory contains design documents for the Information Alchemist CIM implementation.

## Directory Structure

### 📂 current/
**Active design documents currently being implemented or referenced**

- **[event-sourced-graph-architecture.md](./current/event-sourced-graph-architecture.md)** - Complete CIM-integrated architecture design
- **[presentation-vs-domain-events.md](./current/presentation-vs-domain-events.md)** - Critical distinction between UI and business events
- **[graph-models-and-morphisms.md](./current/graph-models-and-morphisms.md)** - Mathematical foundation for graph operations
- **[value-object-immutability.md](./current/value-object-immutability.md)** - DDD principles for value objects in event sourcing
- **[nats-subject-naming-standard.md](./current/nats-subject-naming-standard.md)** - NATS messaging conventions
- **[hud.md](./current/hud.md)** - Heads-Up Display design for graph insights (upcoming implementation)
- **[hud-implementation-example.md](./current/hud-implementation-example.md)** - Concrete HUD implementation patterns

### 📂 reference/
**Reference materials and future considerations**

- **[bevy-text.md](./reference/bevy-text.md)** - Text rendering considerations for future UI enhancements

### Completed Designs
Implemented designs have been moved to `/doc/completed/`. These include:
- CID/IPLD architecture documents (implemented in cim-ipld library)
- Dog-fooding self-visualization design
- Documentation reorganization plans

## Key Design Principles

### 1. Event-Driven Architecture
- All state changes flow through immutable events
- Clear separation between presentation and domain events
- NATS-based distributed messaging

### 2. Domain-Driven Design
- Aggregates enforce business rules
- Value objects are immutable
- Commands express intent, events record facts

### 3. CIM Integration
- NATS subjects follow hierarchical naming
- Distributed event and object stores
- Conceptual spaces for semantic relationships

### 4. Graph-First Approach
- Everything is a graph (workflows, knowledge, relationships)
- Structure-preserving transformations
- Model recognition and validation

## Current Focus Areas

1. **Domain Model Implementation** - Completing aggregates and command handlers
2. **CQRS Pattern** - Separating reads and writes effectively
3. **HUD System** - Real-time graph insights and metrics (next up)
4. **Integration Testing** - End-to-end event flow validation

## Design Evolution

Our design documents evolve through stages:
1. **Draft** → Initial ideas in main folder
2. **Current** → Move to `current/` when ready for implementation
3. **Reference** → Move to `reference/` if deferred
4. **Completed** → Move to `/doc/completed/` when fully implemented

## Quick Reference

### For Developers
- Start with [event-sourced-graph-architecture.md](./current/event-sourced-graph-architecture.md) for the big picture
- Read [presentation-vs-domain-events.md](./current/presentation-vs-domain-events.md) before implementing any events
- Follow [nats-subject-naming-standard.md](./current/nats-subject-naming-standard.md) for all NATS subjects

### For Understanding Graph Operations
- [graph-models-and-morphisms.md](./current/graph-models-and-morphisms.md) explains our graph theory foundation
- [hud.md](./current/hud.md) shows how we'll visualize graph properties

### For DDD Principles
- [value-object-immutability.md](./current/value-object-immutability.md) is critical for event sourcing

## Contributing

When adding new design documents:
1. Start in the main design folder for drafts
2. Move to `current/` when ready for implementation
3. Move to `reference/` if deferred for future consideration
4. Move to `/doc/completed/` once fully implemented in code
