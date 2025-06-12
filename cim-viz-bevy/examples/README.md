# CIM-Viz-Bevy Examples

This directory contains demonstration examples showing how to use the cim-viz-bevy library for graph visualization in Bevy applications.

## Running the Examples

Make sure you're in the workspace root directory, then run:

```bash
# Basic graph visualization demo
cargo run --example visual_demo --package cim-viz-bevy

# Simple minimal demo
cargo run --example simple_demo --package cim-viz-bevy

# Workflow visualization demo (if completed)
cargo run --example workflow_demo --package cim-viz-bevy

# Usage example (shows integration pattern)
cargo run --example usage_example --package cim-viz-bevy
```

## Examples Overview

### 1. Visual Demo (`visual_demo.rs`) ✅ FULLY WORKING

An interactive 3D graph visualization demonstrating:
- **Node Creation**: Press SPACE to add random nodes
- **Node Selection**: Click nodes to select them (visual feedback)
- **Node Deletion**: Press D to delete selected nodes (with edge cleanup)
- **Camera Reset**: Press R to reset camera to default position
- **Visual Feedback**: Hover and selection effects
- **Real-time Stats**: Node and edge count display
- **Smooth Animations**: Node transitions and hover effects

### 2. Simple Demo (`simple_demo.rs`) ✅ WORKING

A minimal example showing:
- Basic graph setup with 3 nodes in a triangle
- Automatic edge rendering with gizmos
- Clean separation of concerns
- No user interaction (static visualization)

### 3. Workflow Demo (`workflow_demo.rs`) 🚧 IN PROGRESS

Would demonstrate:
- Business workflow visualization
- Different node types for workflow steps
- Animated state transitions
- Process flow visualization

### 4. Usage Example (`usage_example.rs`)

Shows the basic integration pattern for using the library in your own applications.

## Key Features Demonstrated

1. **Categorical Functor Pattern**: The examples show how domain graph structures are mapped to visual representations while preserving relationships.

2. **Event-Driven Architecture**: All interactions go through events, maintaining clean separation between domain logic and visualization.

3. **Component-Based Design**: Visual properties are attached as ECS components, allowing flexible customization.

4. **Real-time Updates**: Changes to the domain graph are immediately reflected in the visualization.

## Controls Summary

| Key | Action |
|-----|--------|
| **SPACE** | Add a new random node |
| **Left Click** | Select a node |
| **D** | Delete selected node |
| **R** | Reset camera position |
| **Mouse** | Rotate camera (drag) |
| **Scroll** | Zoom in/out |

## Technical Notes

- The demos use Bevy's default 3D renderer with PBR materials
- Edge rendering uses Gizmos for simplicity (could be replaced with mesh-based edges)
- The bridge pattern ensures domain events and visual updates stay synchronized
- All demos support headless testing mode when `BEVY_HEADLESS=1` is set

## Troubleshooting

If you see Vulkan validation errors, these are typically harmless warnings from the graphics driver and can be ignored. The demos should still run correctly.

For better performance, you can enable dynamic linking:
```bash
cargo run --example visual_demo --package cim-viz-bevy --features bevy/dynamic_linking
```

## Key Concepts Demonstrated

### Event-Driven Architecture
All demos use events to communicate between the domain and visualization layers:
- `CreateNodeVisual`, `RemoveNodeVisual` - Domain to visual updates
- `NodeClicked`, `NodeDragging` - User interactions
- `VisualizationCommand` - Visual to domain commands

### Categorical Functor
The demos show the functor in action:
- Domain nodes/edges map to visual entities
- Visual interactions map to domain commands
- Structure is preserved across the mapping

### Resource Management
Following the event-driven pattern:
- Resources are read-only (configuration, metrics)
- State changes happen through events
- No mutable global state

## Extending the Examples

To create your own visualization:

1. **Add the Plugin**:
   ```rust
   app.add_plugins(CimVizPlugin::default())
   ```

2. **Handle Creation Events**:
   ```rust
   fn handle_node_creation(
       mut commands: Commands,
       mut events: EventReader<CreateNodeVisual>,
       // ... resources for meshes, materials
   ) {
       for event in events.read() {
           commands.spawn(
               NodeVisualBundle::new(event.node_id, event.graph_id, event.position)
           );
       }
   }
   ```

3. **Send Domain Events**:
   ```rust
   let event = DomainEvent::NodeAdded {
       graph_id,
       node_id: NodeId::new(),
       position: Some(Vec3::new(x, y, z)),
       metadata: serde_json::json!({"name": "My Node"}),
   };
   bridge.domain_sender().send(event);
   ```

4. **Handle Interactions**:
   ```rust
   if mouse_clicked {
       events.send(NodeClicked {
           entity,
           node_id,
           graph_id,
           world_position,
       });
   }
   ```

## Performance Considerations

- The demos use simple rendering for clarity
- For large graphs, consider:
  - Level-of-detail (LOD) systems
  - Frustum culling
  - Instanced rendering for nodes/edges
  - Spatial indexing for interaction tests

## Next Steps

- Implement layout algorithms (force-directed, hierarchical)
- Add more interaction modes (box selection, edge creation)
- Create domain-specific visualizations
- Add graph analysis visualizations (shortest path, clusters)
