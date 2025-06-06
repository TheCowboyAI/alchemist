# Source Code Alignment Plan - Fresh Start

## Current State

We have the correct directory structure but empty implementations:
- ✅ Domain layer structure (events, commands, aggregates, services)
- ✅ Application layer structure (command handlers, query handlers, projections)
- ✅ Infrastructure layer structure (event store, persistence, repositories)
- ✅ Presentation layer structure (bevy systems, components, plugins)
- ❌ No actual implementation
- ❌ No NATS integration
- ❌ No event sourcing logic

## Implementation Order

### Step 1: Domain Events (Immediate)

Create the core domain events that drive the system:

```rust
// src/domain/events/mod.rs
pub mod graph_events;
pub mod node_events;
pub mod edge_events;

// src/domain/events/graph_events.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphEvent {
    GraphCreated { id: GraphId, name: String },
    GraphDeleted { id: GraphId },
    GraphRenamed { id: GraphId, new_name: String },
}

// src/domain/events/node_events.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeEvent {
    NodeAdded { graph_id: GraphId, node_id: NodeId, content: NodeContent },
    NodeRemoved { graph_id: GraphId, node_id: NodeId },
    NodeUpdated { graph_id: GraphId, node_id: NodeId, content: NodeContent },
    NodeMoved { graph_id: GraphId, node_id: NodeId, position: Position3D },
}

// src/domain/events/edge_events.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeEvent {
    EdgeConnected { graph_id: GraphId, edge_id: EdgeId, source: NodeId, target: NodeId },
    EdgeDisconnected { graph_id: GraphId, edge_id: EdgeId },
    EdgeUpdated { graph_id: GraphId, edge_id: EdgeId, relationship: EdgeRelationship },
}
```

### Step 2: Domain Commands (Immediate)

Define commands that trigger events:

```rust
// src/domain/commands/mod.rs
pub mod graph_commands;
pub mod node_commands;
pub mod edge_commands;

// src/domain/commands/graph_commands.rs
#[derive(Debug, Clone)]
pub enum GraphCommand {
    CreateGraph { name: String },
    DeleteGraph { id: GraphId },
    RenameGraph { id: GraphId, new_name: String },
}
```

### Step 3: Graph Aggregate (Immediate)

Implement the core aggregate:

```rust
// src/domain/aggregates/graph.rs
pub struct Graph {
    id: GraphId,
    metadata: GraphMetadata,
    version: u64,
    graph: StableGraph<NodeId, EdgeId>,
    nodes: HashMap<NodeId, Node>,
    edges: HashMap<EdgeId, Edge>,
}

impl Graph {
    pub fn handle_command(&mut self, command: GraphCommand) -> Result<Vec<DomainEvent>> {
        match command {
            GraphCommand::CreateGraph { name } => {
                // Business logic
                Ok(vec![DomainEvent::GraphCreated { id: self.id, name }])
            }
            // ... other commands
        }
    }
}
```

### Step 4: Basic Infrastructure (Phase 0)

Start with local event store, prepare for NATS:

```rust
// src/infrastructure/event_store/local.rs
pub struct LocalEventStore {
    events: Vec<EventEnvelope>,
}

// src/infrastructure/event_store/nats.rs
pub struct NatsEventStore {
    client: async_nats::Client,
    jetstream: async_nats::jetstream::Context,
}
```

### Step 5: Minimal Bevy Integration

Get something visible:

```rust
// src/presentation/plugins/graph_editor.rs
impl Plugin for GraphEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<LocalEventStore>()
            .add_systems(Startup, setup_camera)
            .add_systems(Update, handle_graph_commands)
            .add_systems(Update, render_graph);
    }
}
```

## File Creation Order

1. **Domain Layer First** (Pure business logic, no dependencies)
   - [ ] Create domain events
   - [ ] Create domain commands
   - [ ] Create value objects (GraphId, NodeId, etc.)
   - [ ] Create Graph aggregate

2. **Infrastructure Stubs** (Minimal implementation)
   - [ ] Create LocalEventStore
   - [ ] Create EventEnvelope type
   - [ ] Prepare NATS client structure

3. **Application Layer** (Connect domain to infrastructure)
   - [ ] Create command handlers
   - [ ] Create query handlers
   - [ ] Create basic projections

4. **Presentation Layer** (Make it visible)
   - [ ] Create Bevy components
   - [ ] Create basic systems
   - [ ] Wire up the plugin

## Success Criteria

- [ ] Application compiles and runs
- [ ] Can create a graph via command
- [ ] Events are stored locally
- [ ] Basic visualization appears
- [ ] Ready for NATS integration

## Next Steps After Alignment

1. Replace LocalEventStore with NatsEventStore
2. Add conceptual space components
3. Implement force-directed layout
4. Add game theory components
5. Enable dog-fooding features

## Timeline

- **Today**: Implement Steps 1-3 (Domain layer)
- **Tomorrow**: Implement Steps 4-5 (Infrastructure + Presentation)
- **Day 3**: Test and refine, prepare for NATS
