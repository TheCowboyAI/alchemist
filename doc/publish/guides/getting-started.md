# Getting Started with CIM

## Prerequisites

### System Requirements

- **Operating System**: NixOS or Linux with Nix package manager
- **Memory**: 16GB RAM minimum (32GB recommended)
- **Storage**: 20GB free space
- **GPU**: Vulkan-capable GPU for 3D visualization

### Required Software

- Nix package manager (2.18+)
- Git
- Rust (managed by Nix)
- NATS server (managed by Nix)

## Installation

### 1. Clone the Repository

```bash
git clone --recursive https://github.com/TheCowboyAI/alchemist.git
cd alchemist
```

### 2. Initialize Submodules

```bash
git submodule update --init --recursive
```

### 3. Enter Development Shell

```bash
direnv allow  # If using direnv
# OR
nix develop
```

This will set up:
- Rust toolchain
- NATS server
- Development tools
- Environment variables

### 4. Build the Project

```bash
nix build
```

## Quick Start Example

### 1. Start NATS Server

In a separate terminal:

```bash
nats-server -js
```

### 2. Run the Example

```bash
nix run .#example-graph-visualization
```

This launches a 3D graph visualization showing:
- Interactive nodes and edges
- Real-time updates via NATS
- Conceptual space positioning

## Creating Your First Domain

### 1. Generate Domain Structure

```bash
./scripts/create-domain.sh my-domain
```

This creates:
```
cim-domain-my-domain/
├── src/
│   ├── aggregate/
│   ├── commands/
│   ├── events/
│   ├── handlers/
│   ├── projections/
│   ├── queries/
│   └── value_objects/
├── Cargo.toml
└── README.md
```

### 2. Define Your Aggregate

```rust
// src/aggregate/mod.rs
use cim_domain::prelude::*;

#[derive(Debug, Clone)]
pub struct MyAggregate {
    id: EntityId<MyAggregateMarker>,
    state: MyState,
    version: u64,
}

impl AggregateRoot for MyAggregate {
    type Command = MyCommand;
    type Event = MyEvent;
    type Error = MyError;

    fn handle_command(&mut self, cmd: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match cmd {
            MyCommand::Create { name } => {
                let event = MyEvent::Created {
                    id: self.id,
                    name,
                    timestamp: SystemTime::now(),
                };
                Ok(vec![event])
            }
        }
    }

    fn apply_event(&mut self, event: &Self::Event) -> Result<(), Self::Error> {
        match event {
            MyEvent::Created { name, .. } => {
                self.state.name = name.clone();
                self.version += 1;
            }
        }
        Ok(())
    }
}
```

### 3. Create Commands and Events

```rust
// src/commands/mod.rs
#[derive(Debug, Clone)]
pub enum MyCommand {
    Create { name: String },
    Update { id: EntityId<MyAggregateMarker>, name: String },
}

// src/events/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MyEvent {
    Created {
        id: EntityId<MyAggregateMarker>,
        name: String,
        timestamp: SystemTime,
    },
    Updated {
        id: EntityId<MyAggregateMarker>,
        name: String,
        timestamp: SystemTime,
    },
}
```

### 4. Implement Command Handler

```rust
// src/handlers/command_handlers.rs
pub struct MyCommandHandler {
    repository: Arc<dyn Repository<MyAggregate>>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl CommandHandler<CreateMyAggregate> for MyCommandHandler {
    type Error = MyError;

    async fn handle(&self, cmd: CreateMyAggregate) -> Result<CommandAcknowledgment, Self::Error> {
        let mut aggregate = MyAggregate::new(cmd.id);
        let events = aggregate.handle_command(MyCommand::Create { name: cmd.name })?;
        
        self.repository.save(&aggregate).await?;
        
        for event in events {
            self.event_publisher.publish(event).await?;
        }
        
        Ok(CommandAcknowledgment::accepted(cmd.id))
    }
}
```

## Working with Graphs

### Creating a Graph

```rust
use cim_domain_graph::prelude::*;

// Create a new graph
let graph_id = GraphId::new();
let mut graph = GraphAggregate::new(graph_id, "My Knowledge Graph");

// Add nodes
let node1 = graph.add_node(
    NodeType::Entity,
    Position3D::new(0.0, 0.0, 0.0),
    json!({ "label": "Concept A" }),
)?;

let node2 = graph.add_node(
    NodeType::Entity,
    Position3D::new(1.0, 0.0, 0.0),
    json!({ "label": "Concept B" }),
)?;

// Connect nodes
graph.add_edge(
    node1,
    node2,
    EdgeRelationship::Association,
    json!({ "strength": 0.8 }),
)?;
```

### Visualizing in Bevy

```rust
use cim_domain_bevy::prelude::*;

fn setup_visualization(
    mut commands: Commands,
    graph_query: Res<GraphQuery>,
) {
    // Load graph from domain
    let graph = graph_query.get_graph(graph_id).unwrap();
    
    // Spawn visual entities
    for (node_id, node) in graph.nodes() {
        commands.spawn((
            NodeBundle {
                node: Node { id: node_id },
                position: node.position,
                mesh: meshes.add(Sphere::new(0.1)),
                material: materials.add(Color::BLUE),
                ..default()
            },
        ));
    }
}
```

## Testing Your Domain

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_creation() {
        let mut aggregate = MyAggregate::new(EntityId::new());
        let events = aggregate.handle_command(
            MyCommand::Create { name: "Test".to_string() }
        ).unwrap();
        
        assert_eq!(events.len(), 1);
        assert_eq!(aggregate.version, 0); // Not yet applied
        
        aggregate.apply_event(&events[0]).unwrap();
        assert_eq!(aggregate.version, 1);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_command_handler() {
    let handler = MyCommandHandler::new(
        InMemoryRepository::new(),
        MockEventPublisher::new(),
    );
    
    let cmd = CreateMyAggregate {
        id: EntityId::new(),
        name: "Test".to_string(),
    };
    
    let ack = handler.handle(cmd).await.unwrap();
    assert_eq!(ack.status, CommandStatus::Accepted);
}
```

## Next Steps

### 1. Explore Examples

- `examples/graph-visualization` - Interactive 3D graphs
- `examples/workflow-engine` - Process automation
- `examples/conceptual-spaces` - Semantic positioning

### 2. Read Documentation

- [Architecture Overview](../architecture/README.md)
- [Domain-Driven Design](../architecture/domain-driven-design.md)
- [Event Sourcing & CQRS](../architecture/event-sourcing-cqrs.md)

### 3. Join the Community

- GitHub Discussions
- Discord Server
- Weekly Office Hours

## Common Issues

### NATS Connection Failed

```bash
# Ensure NATS is running
nats-server -js

# Check connection
nats account info
```

### Build Errors

```bash
# Clean build
rm -rf target
nix build --rebuild

# Check flake
nix flake check
```

### GPU/Vulkan Issues

```bash
# Set headless mode for testing
export BEVY_HEADLESS=1

# Check Vulkan
vulkaninfo
```

## Getting Help

- Check the [FAQ](../faq.md)
- Search [GitHub Issues](https://github.com/TheCowboyAI/alchemist/issues)
- Ask in [Discussions](https://github.com/TheCowboyAI/alchemist/discussions)
- Read the [Glossary](../glossary.md) for terminology 