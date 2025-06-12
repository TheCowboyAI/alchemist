# CIM-Viz-Bevy

A categorical functor implementation that provides the isomorphic mapping between Bevy ECS (visualization) and CIM-ContextGraph (domain) categories.

## Overview

This crate implements a **functor** - a structure-preserving mapping between two categories:

1. **CIM-ContextGraph Category**: Domain-driven graph structures with nodes, edges, and operations
2. **Bevy ECS Category**: Visual entities, components, and systems for rendering

The functor ensures that the structure and relationships in the domain are preserved in the visual representation, while maintaining clean separation between the two concerns.

## Categorical Design

### Objects and Morphisms

In category theory terms:

- **Objects in CIM-ContextGraph**: Graphs, Nodes, Edges
- **Objects in Bevy ECS**: Entities with visual components
- **Morphisms in CIM-ContextGraph**: Operations like AddNode, RemoveEdge, UpdatePosition
- **Morphisms in Bevy ECS**: Events like CreateNodeVisual, NodeClicked, NodeDragging

### The Functor

The functor `F: CIM-ContextGraph → Bevy ECS` maps:
- Domain nodes → Visual entities with NodeVisual component
- Domain edges → Visual entities with EdgeVisual component
- Domain operations → Visual events

The inverse functor `G: Bevy ECS → CIM-ContextGraph` maps:
- Visual interactions → Domain commands
- Visual state changes → Domain events

### Isomorphism Property

The functors maintain an isomorphism: `F ∘ G ≈ Id` and `G ∘ F ≈ Id`

This means:
- A domain node mapped to visual and back preserves its identity
- A visual interaction mapped to domain and back preserves its semantics

## Architecture

```
┌─────────────────────┐         ┌─────────────────────┐
│  CIM-ContextGraph   │         │     Bevy ECS        │
│    (Domain)         │         │  (Visualization)    │
├─────────────────────┤         ├─────────────────────┤
│ • Graphs            │   F →   │ • Entities          │
│ • Nodes            │ ←─────→  │ • Components        │
│ • Edges            │   ← G   │ • Systems           │
│ • Operations       │         │ • Events            │
└─────────────────────┘         └─────────────────────┘
         ↑                               ↑
         │                               │
         └──────── Bridge ───────────────┘
              (Async ↔ Sync)
```

## Usage

### 1. Add the Plugin

```rust
use bevy::prelude::*;
use cim_viz_bevy::CimVizPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CimVizPlugin::default())
        .run();
}
```

### 2. Handle Visualization Events

```rust
fn handle_node_creation(
    mut commands: Commands,
    mut events: EventReader<CreateNodeVisual>,
) {
    for event in events.read() {
        // Spawn visual representation
        commands.spawn(
            NodeVisualBundle::new(
                event.node_id,
                event.graph_id,
                event.position,
            )
        );
    }
}
```

### 3. Send Domain Commands

```rust
fn handle_mouse_click(
    bridge: Res<CategoricalBridge>,
    // ... input handling
) {
    let command = VisualizationCommand::CreateNode {
        graph_id,
        position,
        metadata: None,
    };
    bridge.send_command(command);
}
```

## Components

### Visual Components (Objects)

- `GraphVisual`: Visual representation of a graph
- `NodeVisual`: Visual representation of a node
- `EdgeVisual`: Visual representation of an edge
- `Selected`, `Hovered`, `Dragging`: Visual-only states

### Events (Morphisms)

- **Creation**: `CreateNodeVisual`, `CreateEdgeVisual`
- **Deletion**: `RemoveNodeVisual`, `RemoveEdgeVisual`
- **Interaction**: `NodeClicked`, `EdgeClicked`, `NodeDragging`
- **Updates**: `NodePositionChanged`, `NodeMetadataChanged`

## Bridge

The `CategoricalBridge` handles the async/sync boundary:

```rust
// Domain (async) → Bevy (sync)
let sender = bridge.domain_sender();
sender.send(DomainEvent::NodeAdded { ... });

// Bevy (sync) → Domain (async)
bridge.send_command(VisualizationCommand::UpdateNode { ... });
```

## Example

See `examples/usage_example.rs` for a complete example of integrating with a Bevy application.

## Testing

The functor properties are verified through tests:

```bash
cargo test --package cim-viz-bevy
```

Tests verify:
- Identity preservation
- Operation preservation
- Isomorphism properties
- Event flow correctness

## Design Principles

1. **Separation of Concerns**: Domain logic never depends on visualization
2. **Event-Driven**: All communication through events, no direct coupling
3. **Preserving Structure**: The functor maintains relationships and operations
4. **Type Safety**: Rust's type system ensures correct mappings
5. **Performance**: Efficient channel-based communication

## License

Same as the parent project.
