# Architecture Documentation

This directory contains detailed technical documentation about the Information Alchemist architecture.

## 📚 Core Documents

### [CIM Overview](cim-overview.md)
**The Big Picture: Information Alchemist as a CIM Leaf Node**

Learn how Information Alchemist fits into the Composable Information Machine ecosystem:
- Distributed architecture with NATS messaging
- Event-driven communication patterns
- Integration with CIM backend services
- Conceptual spaces for semantic understanding

### [Event Sourcing](event-sourcing.md)
**Event-Driven Architecture in Practice**

Deep dive into our event sourcing implementation:
- Domain events as the source of truth
- CID chains for cryptographic integrity
- CQRS pattern for read/write separation
- Event replay and time travel debugging

### [System Components](system-components.md)
**Detailed Component Reference**

Comprehensive guide to all system components:
- Domain aggregates and entities
- Command and event handlers
- Projections and read models
- Infrastructure services

## 🏗️ Architecture Principles

### 1. **Event-First Design**
Every state change flows through events:
```
User Action → Presentation Event → Domain Command → Domain Event → Projection Update
```

### 2. **Clean Architecture Layers**
```
Presentation (Bevy ECS)
    ↓
Application (CQRS)
    ↓
Domain (Business Logic)
    ↓
Infrastructure (NATS, Storage)
```

### 3. **Domain-Driven Design**
- Aggregates enforce business rules
- Value objects are immutable
- Events record business facts
- Commands express user intent

### 4. **Distributed by Design**
- NATS for messaging backbone
- Event store for distributed state
- Object store for large content
- CID chains for integrity

## 🔄 Event Flow Architecture

### Presentation Events (Stay in Bevy)
```rust
// UI interactions that don't affect domain state
pub enum PresentationEvent {
    DragStarted { node_id: NodeId, position: Vec3 },
    AnimationFrame { progress: f32 },
    CameraRotated { rotation: Quat },
}
```

### Domain Events (Distributed via NATS)
```rust
// Business-meaningful state changes
pub enum DomainEvent {
    NodeAdded { graph_id: GraphId, node_id: NodeId, content: NodeContent },
    EdgeConnected { source: NodeId, target: NodeId, relationship: EdgeRelationship },
    GraphPublished { graph_id: GraphId, version: Version },
}
```

## 🎯 Key Design Decisions

### Why Event Sourcing?
- **Complete audit trail** - Every change is recorded
- **Time travel** - Replay to any point in history
- **Distributed consensus** - Events as shared truth
- **AI-ready** - Events provide training data

### Why NATS?
- **High performance** - Millions of messages/second
- **Built-in persistence** - JetStream for event store
- **Clustering** - Automatic failover
- **Security** - JWT auth and TLS

### Why Bevy ECS?
- **Performance** - Cache-friendly data layout
- **Flexibility** - Compose behaviors from components
- **Parallelism** - Automatic system scheduling
- **Hot reload** - Rapid development

### Why CID Chains?
- **Integrity** - Cryptographic proof of history
- **Deduplication** - Same content = same CID
- **Distribution** - Content-addressed storage
- **Interoperability** - IPLD standards

## 📊 Current Implementation Status

### ✅ Implemented
- Event sourcing with CID chains
- NATS integration with JetStream
- Graph aggregate with full business logic
- CQRS command/query separation
- Integration test suite
- Projection system for read models
- Presentation/domain event separation

### 🚧 In Progress
- Additional domain aggregates (Workflow, ConceptualSpace)
- Query handler optimization
- Snapshot management
- Performance benchmarking

### 📅 Planned
- Conceptual space implementation
- AI agent integration
- Multi-user collaboration
- Plugin architecture

## 🔗 Related Documentation

### Implementation Details
- `/doc/design/current/` - Active design documents
- `/doc/plan/current/` - Implementation plans
- `/doc/completed/` - Completed designs and plans

### Key Design Documents
- `event-sourced-graph-architecture.md` - Complete system design
- `presentation-vs-domain-events.md` - Event separation patterns
- `graph-models-and-morphisms.md` - Graph theory foundation
- `value-object-immutability.md` - DDD principles

## 🚀 Getting Started

For developers new to the architecture:

1. Start with [CIM Overview](cim-overview.md) for the big picture
2. Read [Event Sourcing](event-sourcing.md) to understand data flow
3. Explore [System Components](system-components.md) for implementation details
4. Check `/doc/design/current/` for detailed design rationale

## 💡 Architecture Highlights

### Isomorphic DDD-ECS Mapping
We maintain a clean mapping between DDD concepts and ECS implementation:
- DDD Entities → ECS Entities
- Value Objects → Components
- Domain Services → Systems
- Events → Events (shared abstraction)

### Event Aggregation Pattern
Multiple presentation events aggregate into single domain commands:
```rust
DragStarted + DragUpdated + DragEnded → MoveNode command
```

### Bidirectional Event Flow
External systems can both:
- Subscribe to our domain events
- Inject events that we process

This enables seamless integration with the broader CIM ecosystem.

---

*Architecture documentation is continuously updated as the system evolves.*
