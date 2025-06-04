# CIM-Integrated Event-Sourced Graph Architecture

## Overview

Information Alchemist is a sophisticated UI leaf node in the Composable Information Machine (CIM) cluster, providing:
- **CIM Integration**: Full NATS-based communication with distributed backend
- **Event Sourcing**: Complete audit trail via distributed event store
- **Conceptual Spaces**: Spatial knowledge representation based on Gärdenfors
- **Game Theory**: Strategic components for multi-agent coordination
- **AI Ready**: Prepared for intelligent agent integration
- **Performance**: Designed for 100K+ nodes with distributed scalability

## Architecture Documents

### 📋 [event-sourced-graph-architecture.md](./event-sourced-graph-architecture.md)
**Complete CIM-Integrated Architecture Design**

Comprehensive design covering:
- CIM leaf node architecture
- NATS messaging integration
- Distributed storage (Event Store + Object Store)
- Conceptual spaces implementation
- Game theory components
- AI agent interface
- Modular plugin system

## Key Concepts

### System Architecture

Information Alchemist operates as part of the CIM cluster:

```
CIM Cluster (Backend Nodes, Event Store, Object Store)
                    ↓
              NATS Messaging
                    ↓
     Information Alchemist (CIM Leaf Node)
```

### Communication via NATS

All backend communication uses NATS subjects:
- **Commands**: `graph.commands.*`, `node.commands.*`
- **Events**: `graph.events.*`, `node.events.*`
- **Queries**: `graph.queries.*`, `node.queries.*`
- **AI Agents**: `agent.commands.*`, `agent.events.*`

### Enhanced Components

1. **Conceptual Positioning**
   - Spatial knowledge representation
   - Semantic similarity calculations
   - Category-based clustering

2. **Game Theory**
   - Strategy components
   - Utility functions
   - Coalition formation

3. **AI Integration**
   - Agent communication interface
   - Analysis requests
   - Suggestion handling

### Architecture Layers

```
Presentation (Bevy ECS Visualization)
    ↓
Application (Commands, Queries, Projections)
    ↓
Domain (Graph Aggregate with Conceptual Spaces)
    ↓
Infrastructure (NATS Client, Distributed Storage)
```

### Performance Targets

- **Nodes**: 100K+ supported
- **Local Query**: < 10ms latency
- **Distributed Query**: < 100ms latency
- **Frame Rate**: 60 FPS maintained
- **Memory**: < 2GB for 100K nodes

## Implementation Status

Migrating to full CIM integration. See [implementation plan](../plan/event-sourcing-implementation-plan.md) for details.

### Phases
- [ ] Phase 0: NATS Integration Foundation
- [ ] Phase 1: Distributed Event Infrastructure
- [ ] Phase 2: Domain Model with CIM Extensions
- [ ] Phase 3: Conceptual Spaces Implementation
- [ ] Phase 4: Game Theory Components
- [ ] Phase 5: AI Agent Interface
- [ ] Phase 6: Full CIM Integration
- [ ] Phase 7: Advanced Features

## Quick Start

The new architecture provides all legacy features plus:
- Distributed graph storage and queries
- Real-time collaboration
- Conceptual space navigation
- Strategic agent interactions
- AI-powered analysis

## Development Guidelines

### CIM Integration Principles
- **Distributed First**: Design for NATS communication
- **Event Driven**: All state changes via events
- **Modular**: Plugin-based architecture
- **Resilient**: Handle network failures gracefully
- **Secure**: JWT authentication, TLS encryption

### Domain Language
Following CIM vocabulary:
- Events: Past-tense facts (`NodeAdded`, not `AddNode`)
- Commands: Imperative verbs (`AddNode`, not `NodeAdded`)
- Subjects: Hierarchical naming (`graph.events.created`)
- Components: Domain-specific (`ConceptualPosition`, `StrategyComponent`)

### Code Organization
```
src/
├── domain/          # Business logic with CIM concepts
├── infrastructure/  # NATS, distributed storage
├── application/     # Command/query handlers
└── presentation/    # Bevy ECS visualization
```

### Testing Strategy
- Domain logic: Pure unit tests
- NATS integration: Integration tests
- Distributed: System tests
- Performance: Continuous benchmarks

## Key Differences from Standalone

| Standalone | CIM-Integrated |
|------------|----------------|
| Local storage | Distributed Event/Object Store |
| Direct mutations | NATS messaging |
| Simple events | Full event sourcing |
| Basic layout | Conceptual space positioning |
| Single user | Multi-user collaboration |
| No AI | AI agent ready |

## Resources

- [Implementation Plan](../plan/event-sourcing-implementation-plan.md)
- [Published Documentation](../publish/)
- [CIM Research](../research/)
- [Vocabulary](../publish/vocabulary.md)
