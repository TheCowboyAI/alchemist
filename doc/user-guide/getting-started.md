# Getting Started with CIM (Composable Information Machine)

## Overview

The Composable Information Machine (CIM) is an event-driven, graph-based workflow system built with Domain-Driven Design principles. It combines:

- **Event Sourcing**: All state changes are recorded as immutable events
- **Graph Visualization**: Visual representation of workflows and knowledge
- **AI Integration**: Multiple AI providers for intelligent analysis
- **Semantic Search**: Vector-based similarity search across domains

## Prerequisites

### System Requirements

- **Operating System**: Linux (NixOS recommended) or macOS
- **Rust**: Nightly 2024+ (managed by Nix)
- **NATS Server**: 2.10+ with JetStream enabled
- **Memory**: 8GB RAM minimum
- **Storage**: 10GB free space

### Development Environment

1. **Install Nix** (if not on NixOS):
   ```bash
   sh <(curl -L https://nixos.org/nix/install) --daemon
   ```

2. **Clone the Repository**:
   ```bash
   git clone https://github.com/thecowboyai/alchemist.git
   cd alchemist
   ```

3. **Enter Development Shell**:
   ```bash
   nix develop
   # Or with direnv:
   direnv allow
   ```

## Quick Start

### 1. Start NATS Server

```bash
# In development shell
nats-server -js
```

### 2. Run Basic Examples

#### Graph Creation Example
```bash
cargo run --example graph_basic -p cim-domain-graph
```

This demonstrates:
- Creating a graph aggregate
- Adding nodes and edges
- Querying graph structure
- Event sourcing in action

#### AI Agent Example
```bash
# Set up AI provider (optional, uses mock by default)
export OPENAI_API_KEY="your-key-here"

cargo run --example basic_agent -p cim-domain-agent
```

#### Workflow Example
```bash
cargo run --example workflow_basic -p cim-domain-workflow
```

## Core Concepts

### 1. Domain-Driven Design

CIM is organized into bounded contexts (domains):

- **Graph Domain**: Core graph operations and visualization
- **Agent Domain**: AI capabilities and analysis
- **Workflow Domain**: Business process execution
- **Identity Domain**: Person and organization management
- **ConceptualSpaces Domain**: Semantic reasoning

### 2. Event-Driven Architecture

All state changes flow through events:

```rust
// Commands express intent
let command = GraphCommand::AddNode {
    node_type: NodeType::Task,
    position: Position3D::new(0.0, 0.0, 0.0),
    metadata: HashMap::new(),
};

// Commands generate events
let events = graph.handle_command(command)?;

// Events are the source of truth
for event in events {
    event_store.append(event).await?;
}
```

### 3. CQRS Pattern

- **Commands**: Modify state through aggregates
- **Queries**: Read from optimized projections
- **Events**: Bridge between write and read models

## Building Your First Application

### Step 1: Define Your Domain

Create a new domain module:

```rust
// my_domain/src/lib.rs
use cim_domain::{DomainEvent, Aggregate, Command};

#[derive(Debug, Clone)]
pub struct MyAggregate {
    id: MyAggregateId,
    // Your domain state
}

impl Aggregate for MyAggregate {
    type Command = MyCommand;
    type Event = MyEvent;
    
    fn handle_command(&mut self, cmd: Self::Command) -> Result<Vec<Self::Event>> {
        // Business logic here
    }
}
```

### Step 2: Create Event Store Integration

```rust
use cim_infrastructure::EventStore;

let event_store = EventStore::new("nats://localhost:4222").await?;

// Store events
event_store.append_events(aggregate_id, events).await?;

// Load aggregate
let events = event_store.get_events(aggregate_id).await?;
let aggregate = MyAggregate::from_events(events)?;
```

### Step 3: Add Bevy Visualization (Optional)

```rust
use bevy::prelude::*;
use cim_domain_bevy::GraphPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GraphPlugin)
        .run();
}
```

## Common Use Cases

### 1. Workflow Automation

```rust
// Define workflow
let workflow = WorkflowBuilder::new("order-processing")
    .add_step("validate-order", StepType::Validation)
    .add_step("check-inventory", StepType::Decision)
    .add_step("process-payment", StepType::Action)
    .add_step("ship-order", StepType::Action)
    .build();

// Execute workflow
let context = WorkflowContext::new();
let result = workflow.execute(context).await?;
```

### 2. Knowledge Graph Building

```rust
// Create knowledge graph
let mut graph = GraphAggregate::new(GraphId::new());

// Add concepts
let ml_node = graph.add_node(NodeType::Concept, "Machine Learning")?;
let ai_node = graph.add_node(NodeType::Concept, "Artificial Intelligence")?;

// Connect with relationships
graph.connect_nodes(ml_node, ai_node, EdgeType::PartOf)?;
```

### 3. AI-Powered Analysis

```rust
// Analyze graph with AI
let agent = Agent::new(AgentType::Analyst);
let analysis = agent.analyze_graph(&graph).await?;

println!("Insights: {:?}", analysis.insights);
println!("Suggestions: {:?}", analysis.suggestions);
```

### 4. Semantic Search

```rust
// Index content
let engine = SemanticSearchEngine::new();
engine.index(EmbeddingRequest {
    text: "Graph-based workflow optimization".to_string(),
    source_id: "doc1".to_string(),
    source_type: "document".to_string(),
    metadata: HashMap::new(),
    model: None,
}).await?;

// Search
let results = engine.search(
    SearchQuery::new("workflow improvement")
        .with_limit(5)
        .with_min_similarity(0.7)
).await?;
```

## Configuration

### NATS Configuration

Create `nats.conf`:
```
jetstream {
    store_dir = "./data/jetstream"
    max_memory_store = 1GB
    max_file_store = 10GB
}

# Event store stream
stream {
    name = "EVENTS"
    subjects = ["events.>"]
    retention = limits
    max_age = 365d
}
```

### Environment Variables

```bash
# NATS
export NATS_URL="nats://localhost:4222"

# AI Providers (optional)
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export OLLAMA_HOST="http://localhost:11434"

# Development
export RUST_LOG="info,cim=debug"
export BEVY_HEADLESS=1  # For tests
```

## Testing

### Unit Tests
```bash
cargo test
```

### Integration Tests
```bash
# Start NATS first
cargo test --test '*' -- --test-threads=1
```

### Domain-Specific Tests
```bash
# Test specific domain
cargo test -p cim-domain-graph

# Test with output
cargo test -- --nocapture
```

## Troubleshooting

### Common Issues

1. **NATS Connection Failed**
   - Ensure NATS server is running: `nats-server -js`
   - Check URL: `nats://localhost:4222`

2. **Compilation Errors**
   - Update dependencies: `cargo update`
   - Clean build: `cargo clean && cargo build`

3. **Event Store Issues**
   - Check JetStream is enabled
   - Verify stream configuration
   - Clear data if corrupted: `rm -rf ./data/jetstream`

4. **AI Provider Errors**
   - Verify API keys are set
   - Check network connectivity
   - Fall back to mock provider

### Debug Mode

Enable detailed logging:
```bash
export RUST_LOG="debug,cim=trace"
export RUST_BACKTRACE=1
```

## Next Steps

1. **Explore Examples**: Check `examples/` in each domain crate
2. **Read Architecture Guide**: See `/doc/design/architecture.md`
3. **Join Community**: Contribute to the project
4. **Build Something**: Create your own domain or workflow

## Resources

- **API Documentation**: `/doc/api/`
- **Design Documents**: `/doc/design/`
- **User Stories**: `/doc/user-stories/`
- **Progress Tracking**: `/doc/progress/progress.json`

## Getting Help

- **GitHub Issues**: Report bugs and request features
- **Documentation**: Check `/doc` for detailed guides
- **Examples**: Working code in `examples/` directories

---

*Last updated: January 2025* 