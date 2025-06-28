# CIM Design Documentation

This directory contains **current, active design documents** that guide the ongoing development of the Composable Information Machine (CIM).

## Design Principles

All designs in this directory follow these core principles:
1. **Event-Driven Architecture** - All state changes through events
2. **Domain-Driven Design** - Business concepts drive technical decisions
3. **Content-Addressed Storage** - Immutable data with CID chains
4. **Composability** - Small, focused modules that combine into larger systems

## Current Design Documents

### Core Architecture
- `event-sourced-graph-architecture.md` - Foundation of our event-driven graph system
- `event-system-architecture.md` - Event flow and processing design
- `event-streams-first-class.md` - Events as primary data model
- `value-object-immutability.md` - Immutable value objects in event sourcing

### Graph Systems
- `graph-models-and-morphisms.md` - Mathematical foundation for graph transformations
- `graph-composition-architecture.md` - How graphs compose and interact
- `graph-composition-refinement.md` - Refined composition patterns
- `graph-abstraction-hierarchy.md` - Abstraction layers for graph operations
- `graph-import-architecture.md` - Importing external data as graphs
- `unified-graph-vision.md` - Unified approach to all graph types

### Domain Implementations
- `conceptgraph-design.md` - Concept-based knowledge graphs
- `conceptgraph-aggregate-pattern.md` - DDD patterns for concept graphs
- `contextgraph-architecture.md` - Context-aware graph structures
- `workflow-category-theory.md` - Category theory foundation for workflows
- `workflow-seven-sketches-compatibility.md` - Mathematical compatibility

### Conceptual Spaces
- `conceptual-spaces-business-domains.md` - Business domain modeling with conceptual spaces
- `subgraph-spatial-mapping.md` - Spatial representation of subgraphs
- `subgraph-as-graph-principle.md` - Subgraphs as first-class graphs

### Security & Infrastructure
- `cim-security-core-abstraction.md` - Security abstraction layer
- `security-context-nats-aligned.md` - Security aligned with NATS messaging
- `core-abstractions-alignment.md` - Alignment of core abstractions
- `core-abstractions-implementation.md` - Implementation guide for abstractions

### Presentation Layer
- `hud.md` - Heads-up display design
- `hud-implementation-example.md` - HUD implementation patterns
- `presentation-vs-domain-events.md` - Separation of presentation and domain events

### Technical Standards
- `nats-subject-naming-standard.md` - NATS subject naming conventions
- `ddd-to-contentgraph-mapping.md` - Mapping DDD concepts to content graphs
- `why-contextgraph-not-petgraph.md` - Design rationale for custom graph implementation

### Supporting Documents
- `mapping.md` - Concept mapping strategies
- `recursive-graph-architecture.md` - Self-referential graph patterns
- `subgraph-demo-description.md` - Demonstration of subgraph concepts

## Reference Materials

The `reference/` subdirectory contains:
- `import-patterns.md` - Patterns for importing various data formats
- `bevy-text.md` - Text rendering in Bevy

## Archived Designs

Completed or outdated designs have been moved to `/doc/archive/2025-06-28-design-cleanup/`. These documents provide historical context but do not reflect current implementation approaches.

## Using These Documents

1. **For New Features**: Start with the relevant domain design documents
2. **For Architecture Decisions**: Refer to core architecture documents
3. **For Implementation**: Follow the patterns established in these designs
4. **For Standards**: Check technical standards before implementing

Last Updated: June 28, 2025 (v0.4.1)
