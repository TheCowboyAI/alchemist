# CIM (Composable Information Machine) Documentation

## Overview

The Composable Information Machine (CIM) is a revolutionary distributed system architecture that transforms how we build, visualize, and reason about information systems. CIM combines:

- **Event-Driven Architecture**: All state changes flow through immutable events
- **Graph-Based Workflows**: Visual representation of business processes and knowledge
- **Conceptual Spaces**: Geometric representation of semantic relationships
- **AI-Native Design**: Built for seamless integration with intelligent agents
- **Self-Referential Capability**: Systems that can visualize and reason about themselves

## Documentation Structure

### Core Architecture
- [Architecture Overview](architecture/README.md) - High-level system architecture
- [Domain-Driven Design](architecture/domain-driven-design.md) - DDD principles and implementation
- [Event Sourcing & CQRS](architecture/event-sourcing-cqrs.md) - Event-driven patterns
- [Graph-Based Workflows](architecture/graph-workflows.md) - Visual workflow representation
- [Conceptual Spaces](architecture/conceptual-spaces.md) - Semantic knowledge representation

### Domain Modules
- [Domain Module Overview](domains/README.md) - Bounded contexts and domain separation
- [Core Domains](domains/core-domains.md) - Person, Organization, Agent, etc.
- [Infrastructure Domains](domains/infrastructure-domains.md) - Git, Nix, Document processing
- [Visualization Domains](domains/visualization-domains.md) - Graph, Workflow, Conceptual visualization

### Technical Guides
- [Getting Started](guides/getting-started.md) - Quick start guide
- [Development Setup](guides/development-setup.md) - NixOS development environment
- [Testing Strategy](guides/testing-strategy.md) - TDD and testing practices
- [Integration Patterns](guides/integration-patterns.md) - NATS messaging and event flows

### API Reference
- [Domain Events](api/domain-events.md) - Event catalog and schemas
- [Commands & Queries](api/commands-queries.md) - CQRS interface reference
- [Graph Operations](api/graph-operations.md) - Graph manipulation APIs
- [Conceptual Space APIs](api/conceptual-spaces.md) - Semantic operations

## Key Concepts

### Information as Events
We build a world where information exists as a sequential, append-only series of events:

```
(Command<T> | Query<T>) → [Events<T>] → Models/Projections
```

### Graph-Based Representation
CIM uses graphs as the primary abstraction for:
- Business workflows
- Knowledge structures
- Event flows
- System architecture

### Conceptual Spaces
Every entity exists in both:
- **Visual Space**: 3D position for rendering
- **Conceptual Space**: Semantic position in knowledge dimensions

## Quick Links

- [Architecture Decision Records](architecture/adr/)
- [Domain Glossary](glossary.md)
- [Contributing Guide](../CONTRIBUTING.md)
- [License](../LICENSE.md) 