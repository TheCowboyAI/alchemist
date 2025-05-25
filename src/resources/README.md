# Resources Module

This module contains all shared state resources for the Alchemist Graph Editor. Resources should be used sparingly for truly global state that needs to be accessed across multiple systems.

## Resource Guidelines

### When to Use Resources

Resources are appropriate for:
- Application-wide configuration (e.g., DpiScaling, ViewportConfig)
- Global UI state (e.g., PanelManager, DashboardState)
- Shared data structures (e.g., GraphData, GraphState)
- File operation state (e.g., FileOperationState, AutoSaveConfig)

### When NOT to Use Resources

Consider using components instead for:
- Entity-specific data
- Data that only a few systems need
- Data that has a clear owner entity

## Available Resources

### Graph State Resources

- **GraphState**: Tracks overall graph statistics and selection state
- **GraphMetadata**: Stores graph metadata (name, description, version, domain)
- **GraphBounds**: Tracks graph bounds for camera calculations
- **GraphInspectorState**: UI state for the graph inspector panel
- **GraphData**: The core petgraph data structure (in graph_data.rs)

### UI State Resources

- **PanelManager**: Manages visibility and sizing of UI panels
- **ControlPanelState**: State for the control panel
- **InspectorPanelState**: State for the inspector panel
- **DashboardState**: Tracks which editor is active
- **UiInteractionState**: Tracks mouse-over-UI state

### Application Configuration

- **DpiScaling**: Manages DPI scaling for the application
- **ViewportConfig**: Configuration for viewport rendering
- **NodeCounter**: Simple counter for node creation
- **EdgeMeshTracker**: Tracks edge mesh entities for rendering
- **LastViewMode**: Tracks the last camera view mode

### File Operations

- **FileOperationState**: Tracks current file and unsaved changes
- **AutoSaveConfig**: Configuration for auto-save functionality

## Resource Access Patterns

### Reading Resources

```rust
fn my_system(
    graph_state: Res<GraphState>,
    dpi_scaling: Res<DpiScaling>,
) {
    // Read-only access to resources
    println!("Node count: {}", graph_state.node_count);
    println!("Scale factor: {}", dpi_scaling.scale_factor);
}
```

### Mutating Resources

```rust
fn my_system(
    mut graph_state: ResMut<GraphState>,
    mut panel_manager: ResMut<PanelManager>,
) {
    // Mutable access to resources
    graph_state.node_count += 1;
    panel_manager.left_panel_visible = true;
}
```

### Optional Resources

```rust
fn my_system(
    graph_metadata: Option<Res<GraphMetadata>>,
) {
    if let Some(metadata) = graph_metadata {
        println!("Graph name: {}", metadata.name);
    }
}
```

## Migration Notes

During Phase 3 of the ECS refactoring, the following resources were consolidated:

1. **Removed Duplicates**:
   - `GraphState` and `GraphMetadata` were defined in both `graph_core/components.rs` and `resources/graph_state.rs`
   - `NodeCounter` was defined in both `main.rs` and `resources/app_config.rs`
   - `DpiScaling` was defined in both `main.rs` and `resources/app_config.rs`
   - `EdgeMeshTracker` and `LastViewMode` were defined in both `graph_core/rendering.rs` and `resources/app_config.rs`
   - `GraphInspectorState` was defined in both `graph_core/ui.rs` and `resources/graph_state.rs`

2. **Import Updates**:
   All modules now import these resources from the `resources` module:
   ```rust
   use crate::resources::{GraphState, GraphMetadata, NodeCounter, DpiScaling, EdgeMeshTracker, LastViewMode, GraphInspectorState};
   ```

3. **Future Considerations**:
   - Consider merging `GraphInspectorState` with `InspectorPanelState`
   - `UiInteractionState` might be better represented as events
   - Some panel states could be consolidated into a single UI state resource
