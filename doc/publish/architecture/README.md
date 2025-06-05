# Architecture Documentation

Welcome to the Information Alchemist architecture documentation. This comprehensive guide covers the system's design as a CIM (Composable Information Machine) leaf node.

## Overview

This directory contains the comprehensive architectural documentation for the CIM-integrated Information Alchemist system. The documentation has been updated to reflect the current event-sourced, distributed architecture following Domain-Driven Design principles.

## Document Structure

### 1. [CIM Integration Overview](./cim-overview.md)
The foundational document explaining:
- What CIM is and why we chose it
- High-level architecture and design decisions
- Implementation phases and progress
- Benefits for developers, users, and organizations
- Dog-fooding approach for self-visualization

### 2. [Event Sourcing Patterns](./event-sourcing.md)
Detailed patterns and implementation guide for:
- Core event sourcing concepts
- CQRS implementation with commands and queries
- Event store design using NATS JetStream
- Projection patterns for read models
- Event versioning and schema evolution
- Testing strategies for event-sourced systems

### 3. [System Components](./system-components.md)
Complete reference for all system components:
- Presentation Layer (Bevy ECS) components
- Domain Layer aggregates and services
- Infrastructure Layer (NATS, Event Store, Projections)
- Bridge components for async/sync communication
- Supporting components (configuration, error handling)

## Quick Start Guide

### For Developers
1. Start with the [CIM Integration Overview](./cim-overview.md) to understand the overall architecture
2. Review [Event Sourcing Patterns](./event-sourcing.md) for implementation patterns
3. Reference [System Components](./system-components.md) for specific component details

### For Architects
1. Focus on the design decisions in [CIM Integration Overview](./cim-overview.md)
2. Understand the event flow and CQRS patterns in [Event Sourcing Patterns](./event-sourcing.md)
3. Review component interactions in [System Components](./system-components.md)

### For Users
1. Read the executive summary in [CIM Integration Overview](./cim-overview.md)
2. Understand the benefits section for your role
3. See the dog-fooding example for practical application

## Key Architectural Principles

### 1. Event-Driven Design
- All state changes are captured as immutable events
- Events are the single source of truth
- System state can be reconstructed from events

### 2. Separation of Concerns
- **Presentation Layer**: Bevy ECS for real-time visualization
- **Domain Layer**: Pure business logic with event sourcing
- **Infrastructure Layer**: NATS messaging and persistence

### 3. Conceptual Integration
- Every entity has both visual and semantic representation
- Conceptual spaces enable AI reasoning
- Similarity and categorization built into the core

### 4. Distributed Architecture
- NATS provides distributed messaging backbone
- Event streams enable natural distribution
- CID chains ensure cryptographic integrity

## Architecture Diagrams

### System Overview
```
┌─────────────────────────────────────────────────────────────┐
│                    Presentation Layer                        │
│                      (Bevy ECS)                             │
└─────────────────────────┬───────────────────────────────────┘
                          │ Async/Sync Bridge
┌─────────────────────────┴───────────────────────────────────┐
│                    Domain Layer                              │
│               (Event Sourcing + CQRS)                        │
└─────────────────────────┬───────────────────────────────────┘
                          │ Events
┌─────────────────────────┴───────────────────────────────────┐
│                 Infrastructure Layer                         │
│                    (NATS + Storage)                         │
└─────────────────────────────────────────────────────────────┘
```

### Event Flow
```
Command → Aggregate → Event → Event Store
                        ↓
                   Projection → Read Model
                        ↓
                   UI Update ← Event Bridge
```

## Related Documentation

- **Design Documents**: `/doc/design/`
- **Implementation Plans**: `/doc/plan/`
- **Progress Tracking**: `/doc/progress/progress.json`
- **User Guides**: `/doc/publish/guides/` (coming soon)
- **API Reference**: `/doc/publish/reference/` (coming soon)

## Version History

- **v2.0** (2025-06-05): Complete rewrite for CIM integration
- **v1.0** (Legacy): Original architecture (archived)

## Contributing

When updating architecture documentation:
1. Follow DDD naming conventions (no technical suffixes)
2. Ensure consistency with implementation
3. Update progress.json to reflect changes
4. Test all code examples
5. Maintain cross-references

## Questions?

For questions about the architecture:
- Check the specific component documentation
- Review the implementation in `/src/`
- Consult the project rules in `.cursor/rules`

## Architectural Philosophy

### Component-Centric Architecture

```rust
// Values are components
#[derive(Component)]
struct ConceptualPoint(Vec<f32>);

#[derive(Component)]
struct GraphNode { id: NodeId }

#[derive(Component)]
struct WorkflowState { status: Status }

// Systems process components and emit events
fn process_conceptual_mapping(
    query: Query<(&GraphNode, &ConceptualPoint)>,
    mut events: EventWriter<ConceptualMappingComplete>
) {
    for (node, point) in query.iter() {
        // Process and emit events
        events.send(ConceptualMappingComplete {
            node_id: node.id,
            similarity_score: calculate_similarity(point)
        });
    }
}
```

### Implementation Focus

We focus on:
- Event streams as the source of truth
- Components as the unit of data
- Systems as pure event transformers
- Emergent behavior from event cascades
