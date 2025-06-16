# CIM (Composable Information Machine) Documentation

## Overview

The Composable Information Machine (CIM) is a revolutionary distributed system architecture that transforms how we build, visualize, and reason about information systems. CIM combines:

- **Event-Driven Architecture**: All state changes flow through immutable events
- **Graph-Based Workflows**: Visual representation of business processes and knowledge
- **Conceptual Spaces**: Geometric representation of semantic relationships
- **AI-Native Design**: Built for seamless integration with intelligent agents
- **Self-Referential Capability**: Systems that can visualize and reason about themselves

## Documentation Structure

### Business Documentation
- [Business Overview](business/README.md) - Business value proposition and use cases
- [Introduction](business/01-introduction.md) - What is CIM and why it matters
- [Core Concepts](business/02-core-concepts.md) - Key business concepts and terminology
- [Use Cases](business/03-use-cases.md) - Real-world applications and scenarios
- [Getting Started](business/04-getting-started.md) - Business user quick start

### Technical Documentation  
- [Technical Overview](technical/README.md) - Technical architecture and implementation
- [Architecture Overview](technical/01-architecture-overview.md) - System architecture and design principles
- [Core Components](technical/02-core-components.md) - Key technical components and their roles
- [Event System](technical/03-event-system.md) - Event sourcing and CQRS implementation
- [Integration Guide](technical/04-integration-guide.md) - How to integrate with CIM
- [Performance Guide](technical/05-performance-guide.md) - Performance optimization and monitoring
- [Plugin Development](technical/06-plugin-development.md) - Developing plugins and extensions

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

## Quick Navigation

### 🚀 **New to CIM?** Start Here:
1. [Business Introduction](business/01-introduction.md) - Understand the value proposition
2. [Getting Started Guide](guides/getting-started.md) - Set up your development environment
3. [Core Concepts](business/02-core-concepts.md) - Learn the fundamentals

### 💼 **Business Users:**
- [Business Use Cases](business/03-use-cases.md) - See real-world applications
- [Business Getting Started](business/04-getting-started.md) - 30-day implementation roadmap
- [Domain Glossary](glossary.md) - Business terminology

### 👨‍💻 **Developers:**
- [Technical Architecture](technical/01-architecture-overview.md) - System design overview
- [Core Components](technical/02-core-components.md) - Technical implementation details
- [Integration Guide](technical/04-integration-guide.md) - Connect with CIM backend

### 🔧 **DevOps & Administrators:**
- [Performance Guide](technical/05-performance-guide.md) - Optimization and monitoring
- [Architecture Overview](architecture/README.md) - Infrastructure and deployment

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