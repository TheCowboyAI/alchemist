# System Conflict Analysis for Alchemist Graph Editor

## Overview

This document analyzes potential system conflicts and timing issues that could cause flashing, race conditions, and other odd behavior in the Alchemist graph editor.

## Identified Issues

### 1. Multiple UI Systems Running Without Coordination

**Problem**: Multiple UI systems are running in different plugins without proper ordering:
- `graph_inspector_ui` in GraphPlugin
- `control_panel_system` in UiPanelsPlugin
- `inspector_panel_system` in UiPanelsPlugin
- `menu_bar_system` in UiPanelsPlugin

**Symptoms**:
- UI panels might fight for focus
- Conflicting state updates
- Potential egui context conflicts

**Solution**: Create a unified UI system ordering with SystemSets.

### 2. Rendering Systems Running Multiple Times

**Problem**: The rendering systems check for view mode changes but might still run unnecessarily:
- `clear_rendering_on_view_change` runs every frame
- `render_reference_grid` runs every frame
- `render_graph_nodes` and `render_graph_edges` run in PostUpdate

**Symptoms**:
- Screen flashing when view mode changes
- Performance issues
- Duplicate rendering attempts

**Solution**: Add proper change detection and run conditions.

### 3. Event Processing Order Issues

**Problem**: Events are processed in multiple places without clear ordering:
- Node creation events processed in Update
- Edge creation events processed after node creation
- Deferred edge events processed separately
- UI events might trigger before graph state is ready

**Symptoms**:
- Edges created before nodes exist
- Missing entity references
- Timing-dependent bugs

**Solution**: Establish clear event processing phases.

### 4. Camera System Conflicts

**Problem**: Camera systems run without coordination with rendering:
- Camera input systems run before transition system
- Viewport updates might happen after rendering
- View mode changes trigger multiple system reactions

**Symptoms**:
- Jerky camera movement
- Delayed viewport updates
- Rendering artifacts during transitions

**Solution**: Properly order camera systems with rendering.

### 5. Selection System Conflicts

**Problem**: Multiple systems handle selection without coordination:
- `handle_node_selection` in graph_core
- `handle_selection_events` in graph_core
- `update_selection_highlights` runs after selection
- Inspector UI reads selection state

**Symptoms**:
- Selection state inconsistencies
- Visual feedback lag
- Multiple selection handlers fighting

**Solution**: Centralize selection handling.

## Proposed System Execution Order

### Phase 1: Input & Events (PreUpdate)
```rust
// All input gathering and event generation
InputSystemSet {
    - keyboard_input_system
    - mouse_input_system
    - camera_input_systems (orbit, pan, zoom)
    - file_operation_events
}
```

### Phase 2: Event Processing (Update - Early)
```rust
// Process events in dependency order
EventProcessingSet {
    - handle_file_events (load/save)
    - handle_create_node_events
    - handle_create_edge_events
    - handle_deferred_edge_events
    - handle_delete_events
    - handle_selection_events
    - handle_movement_events
}
```

### Phase 3: State Updates (Update - Mid)
```rust
// Update component states based on events
StateUpdateSet {
    - update_graph_data
    - update_node_visuals
    - update_selection_highlights
    - camera_transition_system
    - update_camera_system
}
```

### Phase 4: Change Detection (Update - Late)
```rust
// Detect what changed for rendering
ChangeDetectionSet {
    - detect_component_changes
    - process_graph_changes
    - detect_view_mode_changes
}
```

### Phase 5: UI Systems (Update - After Change Detection)
```rust
// All UI runs after state is stable
UiSystemSet {
    - menu_bar_system
    - control_panel_system
    - inspector_panel_system
    - graph_inspector_ui
}
```

### Phase 6: Rendering Preparation (PostUpdate)
```rust
// Prepare rendering based on final state
RenderPrepSet {
    - clear_rendering_on_view_change (run_if view_changed)
    - render_reference_grid (run_if view_changed)
    - render_graph_nodes (run_if nodes_changed)
    - render_graph_edges (run_if edges_changed)
}
```

## Implementation Plan

### Step 1: Create SystemSets
```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GraphSystemSet {
    Input,
    EventProcessing,
    StateUpdate,
    ChangeDetection,
    UI,
    RenderPrep,
}
```

### Step 2: Add Run Conditions
```rust
// Resource to track what changed
#[derive(Resource, Default)]
pub struct GraphChangeFlags {
    pub nodes_changed: bool,
    pub edges_changed: bool,
    pub view_mode_changed: bool,
    pub selection_changed: bool,
}

// Run conditions
fn nodes_changed(flags: Res<GraphChangeFlags>) -> bool {
    flags.nodes_changed
}

fn view_mode_changed(flags: Res<GraphChangeFlags>) -> bool {
    flags.view_mode_changed
}
```

### Step 3: Consolidate Duplicate Systems
- Merge multiple selection systems into one
- Combine UI panel systems where possible
- Unify change detection systems

### Step 4: Add Proper State Management
- Use States for major mode changes (2D/3D)
- Use Resources for frame-to-frame change tracking
- Clear change flags at end of frame

## Critical Fixes Needed

### 1. Fix Rendering Flash on View Change
```rust
// Only clear rendering when actually changing
pub fn clear_rendering_on_view_change(
    camera_query: Query<&GraphViewCamera, Changed<GraphViewCamera>>,
    // ... only run when camera actually changed
) {
    // Clear logic
}
```

### 2. Fix Edge Rendering Updates
```rust
// Track edge changes properly
pub fn render_graph_edges(
    edge_changes: Query<Entity, Or<(Added<OutgoingEdge>, Changed<OutgoingEdge>)>>,
    // ... only update changed edges
) {
    // Incremental updates only
}
```

### 3. Fix UI Context Conflicts
```rust
// Ensure UI systems run in order
.add_systems(
    Update,
    (menu_bar_system, panels_system, inspector_system)
        .chain()
        .in_set(GraphSystemSet::UI)
)
```

### 4. Fix Event Timing
```rust
// Ensure events are processed in order
.add_systems(
    Update,
    (
        process_node_events,
        process_edge_events.after(process_node_events),
        process_deferred_edges.after(process_edge_events),
    )
        .in_set(GraphSystemSet::EventProcessing)
)
```

## Testing Strategy

1. **Frame Timing Tests**: Log system execution order
2. **State Consistency Tests**: Verify state between systems
3. **Visual Regression Tests**: Check for flashing/artifacts
4. **Performance Tests**: Measure unnecessary system runs

## Conclusion

The main issues stem from:
1. Lack of explicit system ordering
2. Systems running every frame without change detection
3. Multiple systems modifying the same state
4. No clear phases of execution

Implementing the proposed SystemSets and run conditions will resolve these conflicts and create a stable, predictable execution order.
