# Alchemist

The control system for managing the Composable Information Machine (CIM) with visual dashboard and hybrid rendering.

## Overview

Alchemist is built using Domain-Driven Design (DDD), Entity Component System (ECS), and Event Sourcing patterns. It provides:

- **Event-Driven Dashboard**: Real-time monitoring of domain events from JetStream
- **Domain Aggregates**: Each domain (workflow, dialog, policy, etc.) is a bounded context
- **Event Sourcing**: All state changes are captured as events in NATS JetStream
- **ECS Architecture**: Uses Bevy's ECS for efficient state management
- **CQRS Pattern**: Separate read models (projections) from write models (aggregates)
- **Hybrid Rendering**: Launch Bevy (3D) or Iced (2D) windows for different visualizations

### Key Features

- **Real-time Event Streaming**: Dashboard subscribes to JetStream subjects for live updates
- **Domain Event Publishing**: Each domain publishes events to specific subjects
- **Event Replay**: Rebuild state from event history
- **Projection Management**: Maintain read-optimized views of aggregate state
- **Multi-window Architecture**: Spawn specialized renderers for different data types

## Quick Start

```bash
# Launch with visual dashboard (default)
alchemist

# CLI-only mode
alchemist --no-dashboard

# Interactive shell mode
alchemist --interactive
```

## Dashboard

By default, Alchemist launches a visual dashboard built with Iced that provides:
- Real-time domain status monitoring
- Event stream visualization
- Active dialog management
- Policy configuration
- Quick access to launch 3D visualizations

![Dashboard Preview](dashboard-preview.png)

## Architecture

### Core Components

- **Main Shell** (`src/shell.rs`): Command processing and system coordination
- **Dashboard** (`src/dashboard.rs`): Domain monitoring and control interface
- **Renderer System** (`src/renderer.rs`): Manages Bevy/Iced window spawning
- **Domain Modules**: AI, Dialog, Policy, Deployment management

### Hybrid Rendering

Alchemist can spawn specialized windows for different data types:
- **Bevy (3D)**: Graph visualizations, workflows, spatial data
- **Iced (2D)**: Documents, text editing, forms, dashboards

### Usage Examples

```bash
# AI Management
alchemist ai list
alchemist ai add claude-3 --provider anthropic
alchemist ai test claude-3

# Dialog Management
alchemist dialog new --title "System Design"
alchemist dialog list

# Render Windows
alchemist render graph --title "Domain Graph"
alchemist render document README.md
alchemist render demo graph3d

# Policy Management
alchemist policy list
alchemist policy new "api-access" --domain agent

# Domain Visualization
alchemist domain tree
alchemist domain graph --format mermaid
```

## Interactive Shell

The interactive shell provides a command-line interface with tab completion:

```bash
alchemist --interactive

alchemist> dashboard      # Launch dashboard
alchemist> ai list       # List AI models
alchemist> render graph  # Launch 3D graph
alchemist> help         # Show commands
```

## Configuration

Alchemist uses `alchemist.toml` for configuration:

```toml
[general]
default_ai_model = "claude-3"
dialog_history_path = "~/.alchemist/dialogs"
nats_url = "nats://localhost:4222"

[ai_models.claude-3]
provider = "anthropic"
model_name = "claude-3-opus-20240229"
api_key_env = "ANTHROPIC_API_KEY"

[[domains.available]]
name = "workflow"
description = "Business process execution"
module_path = "cim-domain-workflow"
enabled = true
```

## Event Sourcing Architecture

### JetStream Subjects

Alchemist uses a hierarchical subject structure for events:

```
events.domain.workflow.*      # Workflow state changes
events.domain.agent.*         # AI query executions
events.domain.dialog.*        # Conversation events
events.domain.policy.*        # Policy evaluations
events.domain.document.*      # Document lifecycle
events.system.metrics.*       # System metrics
```

### Domain Events

Each domain publishes strongly-typed events:

```rust
// Example workflow event
DomainEvent::WorkflowStateChanged {
    id: Uuid,
    from: String,
    to: String,
    metadata: EventMetadata,
}
```

### ECS Integration

Domains use Bevy's ECS for state management:

```rust
// Workflow aggregate as ECS component
#[derive(Component)]
struct WorkflowAggregate {
    id: Uuid,
    current_state: String,
    // ...
}
```

## Examples

```bash
# Publish domain events
cargo run --example domain_event_publisher

# See ECS + Event Sourcing integration
cargo run --example ecs_event_sourcing_demo

# Show real-time event flow
cargo run --example show_event_flow
```

## Building

```bash
# Build everything
cargo build --release

# Run with NATS connection for real events
NATS_URL=nats://localhost:4222 alchemist

# Run dashboard demo
./demo_renderer.sh
```

## Architecture Documentation

- [RENDERER_ARCHITECTURE.md](RENDERER_ARCHITECTURE.md) - Hybrid rendering system
- [Event Sourcing Guide](doc/design/event-sourcing-guide.md) - Event patterns
- [Domain Model](doc/design/domain-model.md) - DDD boundaries

## License

Apache 2.0