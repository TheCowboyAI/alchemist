# Information Alchemist Architecture Overview

## System Architecture

Information Alchemist implements a clean, layered architecture based on Domain-Driven Design principles and event sourcing patterns.

### Architecture Layers

```
┌─────────────────────────────────────────────────────────────┐
│                    Presentation Layer                        │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ Bevy Systems │  │  Components  │  │   UI Events     │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬────────┘  │
└─────────┼──────────────────┼──────────────────┼────────────┘
          │                  │                  │
┌─────────▼──────────────────▼──────────────────▼────────────┐
│                    Application Layer                         │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │   Command   │  │    Query     │  │   Event         │  │
│  │  Handlers   │  │  Handlers    │  │  Projections    │  │
│  └─────────────┘  └──────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────┘
          │                  │                  │
┌─────────▼──────────────────▼──────────────────▼────────────┐
│                     Domain Layer                            │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ Aggregates  │  │   Commands   │  │    Events       │  │
│  │   (Graph)   │  │              │  │                 │  │
│  └─────────────┘  └──────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────┘
          │                  │                  │
┌─────────▼──────────────────▼──────────────────▼────────────┐
│                  Infrastructure Layer                       │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ Event Store │  │ Repositories │  │  NATS Client    │  │
│  │   (JSON)    │  │              │  │                 │  │
│  └─────────────┘  └──────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Component Overview

### Presentation Layer (Bevy ECS)

The presentation layer uses Bevy's Entity Component System for visualization:

- **Entities**: Visual representations of domain objects
- **Components**: Visual properties (Transform, Mesh, Material)
- **Systems**: Update logic responding to domain events
- **Resources**: Shared state (cameras, input, UI state)

Key components:
- `GraphNode`: Links visual entities to domain nodes
- `GraphEdge`: Links visual entities to domain edges
- `Selected`, `Hovered`: Interaction state components

### Application Layer (CQRS)

The application layer implements Command Query Responsibility Segregation:

#### Command Side
- **Command Handlers**: Process commands and generate events
- **Validation**: Ensure commands meet business rules
- **Event Generation**: Create domain events from valid commands

#### Query Side
- **Query Handlers**: Serve read requests
- **Read Models**: Optimized data structures for queries
- **Projections**: Build read models from event streams

### Domain Layer (Event Sourcing)

The domain layer contains the core business logic:

#### Aggregates
- **Graph**: The aggregate root managing nodes and edges
- **Node**: Entities within the graph
- **Edge**: Connections between nodes

#### Value Objects
- **GraphId**, **NodeId**, **EdgeId**: Unique identifiers
- **Position3D**: Spatial coordinates
- **NodeContent**: Node payload
- **EdgeRelationship**: Edge semantics

#### Domain Events
- Graph lifecycle: `GraphCreated`, `GraphDeleted`
- Node operations: `NodeAdded`, `NodeRemoved`, `NodeMoved`
- Edge operations: `EdgeConnected`, `EdgeDisconnected`

### Infrastructure Layer

The infrastructure layer provides technical capabilities:

#### Event Store
- Append-only log of all domain events
- JSON file persistence (upgradeable to database)
- Event streaming and replay capabilities

#### Repositories
- Load aggregates from event streams
- Save new events to the store
- Cache loaded aggregates for performance

#### NATS Integration
- Publish events to NATS subjects
- Subscribe to backend events
- RPC for queries

## Event Flow

### Command Processing Flow

```
User Action → UI Event → Command → Command Handler → Domain Event → Event Store
                                          ↓
                                    Validation
                                          ↓
                                    Aggregate
```

### Event Processing Flow

```
Event Store → Event Stream → Projections → Read Model
                    ↓
              Event Bridge
                    ↓
              Bevy Systems → Visual Update
```

### Query Processing Flow

```
UI Request → Query → Query Handler → Read Model → Response
```

## Storage Architecture

### Event Store Structure

```
events/
├── graph-{id}/
│   ├── events.json      # Event log
│   ├── snapshot.json    # Latest snapshot
│   └── metadata.json    # Graph metadata
└── index.json           # Global event index
```

### Read Model Structure

The read model uses Petgraph for efficient graph operations:
- `StableGraph<NodeData, EdgeData>`: Core graph structure
- `HashMap<NodeId, NodeIndex>`: Fast node lookup
- `HashMap<EdgeId, EdgeIndex>`: Fast edge lookup

## Performance Considerations

### Optimization Strategies

1. **Event Batching**: Group related events for efficiency
2. **Snapshot Strategy**: Periodic snapshots to speed loading
3. **Lazy Loading**: Load graph details on demand
4. **Caching**: Cache frequently accessed read models

### Scalability Features

1. **Horizontal Scaling**: Multiple read model instances
2. **Event Partitioning**: Partition by graph ID
3. **Async Processing**: Non-blocking event handling
4. **Efficient Queries**: Indexed lookups in read models

## Security Architecture

### Authentication
- JWT tokens for user authentication
- Integration with NATS security

### Authorization
- Command-level permissions
- Graph-level access control
- Event filtering based on permissions

### Audit Trail
- All commands logged with user attribution
- Event history provides complete audit trail
- Tamper-proof event log

## Integration Points

### NATS Subjects

Command subjects:
- `graph.commands.create`
- `graph.commands.delete`
- `node.commands.add`
- `node.commands.remove`
- `edge.commands.connect`
- `edge.commands.disconnect`

Event subjects:
- `graph.events.created`
- `graph.events.deleted`
- `node.events.added`
- `node.events.removed`
- `edge.events.connected`
- `edge.events.disconnected`

Query subjects:
- `graph.queries.get`
- `graph.queries.list`
- `node.queries.find`

### External Systems

The architecture supports integration with:
- AI agents via NATS messaging
- External data sources via import commands
- Analytics systems via event streams
- Monitoring systems via metrics

## Deployment Architecture

### Local Development
- Single process with all layers
- In-memory event store option
- Local NATS server

### Production Deployment
- Separate UI and backend processes
- Distributed event store
- NATS cluster for reliability
- Load-balanced read models

## Future Architecture Evolution

The architecture is designed to evolve:

1. **Distributed Event Store**: Move from local to distributed storage
2. **Microservices**: Split into specialized services
3. **Event Streaming**: Kafka or similar for high volume
4. **Cloud Native**: Kubernetes deployment
5. **Global Distribution**: Multi-region support

This architecture provides a solid foundation that can scale from a single-user application to a globally distributed system while maintaining consistency and performance.
