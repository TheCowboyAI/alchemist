# CIM Architecture Documentation

This section contains the core architectural documentation for the Composable Information Machine (CIM) project.

## Architecture Documents

### Core Architecture
- **[CIM Overview](./cim-overview.md)** - Introduction to Composable Information Machines
- **[Event Sourcing Patterns](./event-sourcing.md)** - Event-driven architecture implementation
- **[Domain Model](./domain-model.md)** - Domain-Driven Design patterns and practices

### Data Architecture
- **[IPLD Overview](./ipld-overview.md)** - Content-addressed data structures
- **[IPLD for Knowledge Workers](./ipld-for-knowledge-workers.md)** - Non-technical guide
- **[IPLD Executive Summary](./ipld-executive-summary.md)** - Business perspective

### System Architecture
- **[NATS Integration](./nats-integration.md)** - Messaging infrastructure
- **[Bevy ECS Architecture](./bevy-ecs-architecture.md)** - UI and visualization layer
- **[Async/Sync Bridge](./async-sync-bridge.md)** - Bridging async NATS with sync Bevy

## Key Architectural Principles

### 1. Event-Driven Architecture
All state changes flow through immutable events:
- Commands express intent
- Aggregates enforce invariants
- Events record what happened
- Projections optimize for queries

### 2. Domain-Driven Design
Clear bounded contexts with ubiquitous language:
- Each domain is a separate module
- No direct dependencies between domains
- Communication only through events
- Consistent terminology within contexts

### 3. CQRS Pattern
Separated write and read models:
- Commands go through aggregates
- Events update projections
- Queries read from projections
- EventStore is never exposed directly

### 4. Graph-Based Modeling
Everything is a composable graph:
- Entities are nodes
- Relationships are edges
- Domains are subgraphs
- Systems compose through graph operations

## Architecture Layers

```
┌─────────────────────────────────────────┐
│         Presentation Layer              │
│         (Bevy ECS + egui)              │
├─────────────────────────────────────────┤
│         Application Layer               │
│    (Command/Query Handlers)            │
├─────────────────────────────────────────┤
│          Domain Layer                   │
│    (Aggregates, Events, VOs)          │
├─────────────────────────────────────────┤
│       Infrastructure Layer              │
│    (NATS, EventStore, IPLD)           │
└─────────────────────────────────────────┘
```

## Current Implementation Status

### ✅ Completed Components
- Event Store with CID chains
- NATS JetStream integration
- Async/Sync bridge
- 13 domain modules
- Bevy UI framework
- IPLD content addressing

### 🚧 In Progress
- Event-driven testing framework (v0.3.0)
- Cross-domain integration tests
- Performance optimizations

### 📋 Planned
- Distributed deployment
- Multi-tenant support
- Advanced AI agent integration

## Quick Navigation

- **New to CIM?** Start with [CIM Overview](./cim-overview.md)
- **Understanding Events?** See [Event Sourcing Patterns](./event-sourcing.md)
- **Building Domains?** Check [Domain Model](./domain-model.md)
- **Working with Data?** Read [IPLD Overview](./ipld-overview.md) 