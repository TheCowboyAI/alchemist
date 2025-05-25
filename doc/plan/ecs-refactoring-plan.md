# ECS Refactoring Plan for Alchemist Graph Editor

## Executive Summary

The Alchemist Graph Editor has grown into a monolithic structure causing timing issues, race conditions, and maintenance difficulties. This plan outlines a comprehensive refactoring to a proper Entity-Component-System (ECS) architecture with clear module boundaries and system execution order.

## 1. Problem Analysis

### Current Issues

1. **Critical Timing Bugs**
   - UI systems running before `EguiPreUpdateSet::InitContexts`
   - Panic: `EguiContexts::ctx_mut` called for uninitialized context
   - Screen flashing due to improper system ordering

2. **Architectural Problems**
   - Monolithic files with mixed responsibilities
   - Poor separation of concerns
   - Components, systems, and resources mixed together
   - No clear system execution order
   - Tight coupling between modules

3. **Maintenance Challenges**
   - Difficult to add new features
   - Hard to debug timing issues
   - Complex interdependencies
   - Large files (600+ lines)

## 2. Proposed Architecture

### Core Principles

1. **Single Responsibility**: Each system does one thing well
2. **Data Locality**: Systems only access components they need
3. **Clear Dependencies**: Explicit system ordering via SystemSets
4. **Modular Plugins**: Each plugin handles one domain
5. **Event-Driven**: Use events for cross-system communication
6. **Resource Minimization**: Prefer components over global resources

### Module Structure

```
src/
├── components/           # Pure data components (no logic)
│   ├── mod.rs
│   ├── graph.rs         # GraphNode, GraphEdge, NodeIndex
│   ├── visual.rs        # NodeVisual, EdgeVisual, MaterialHandle
│   ├── selection.rs     # Selected, Hovered, Focused
│   ├── camera.rs        # GraphViewCamera, ViewMode, CameraTransition
│   ├── ui.rs            # PanelAnchor, UIInteractable
│   └── metadata.rs      # Labels, Properties, DomainType
│
├── resources/           # Shared state (minimize these)
│   ├── mod.rs
│   ├── graph_state.rs   # GraphState, GraphMetadata
│   ├── panel_state.rs   # PanelManager, WorkspaceMode
│   ├── file_state.rs    # FileOperationState, CurrentFile
│   ├── app_config.rs    # AppConfig, DpiScaling
│   └── graph_data.rs    # GraphData (petgraph integration)
│
├── events/              # Event definitions
│   ├── mod.rs
│   ├── graph_events.rs  # CreateNode, DeleteNode, CreateEdge, etc.
│   ├── ui_events.rs     # TogglePanel, ChangeWorkspace, ShowTooltip
│   ├── io_events.rs     # LoadFile, SaveFile, ExportGraph
│   └── camera_events.rs # SwitchViewMode, FocusNode, ResetCamera
│
├── systems/             # Pure systems (logic only)
│   ├── mod.rs
│   ├── graph/
│   │   ├── mod.rs
│   │   ├── creation.rs      # handle_create_node, handle_create_edge
│   │   ├── deletion.rs      # handle_delete_node, handle_delete_edge
│   │   ├── selection.rs     # handle_selection, update_hover_state
│   │   ├── movement.rs      # handle_node_movement, apply_constraints
│   │   ├── validation.rs    # validate_graph_structure, check_cycles
│   │   └── algorithms.rs    # pathfinding, layout_algorithms
│   │
│   ├── rendering/
│   │   ├── mod.rs
│   │   ├── node_rendering.rs    # spawn_node_mesh, update_node_visual
│   │   ├── edge_rendering.rs    # spawn_edge_mesh, update_edge_path
│   │   ├── material_updates.rs  # update_selection_materials
│   │   └── lod_system.rs        # level_of_detail_culling
│   │
│   ├── camera/
│   │   ├── mod.rs
│   │   ├── orbit_controls.rs    # handle_orbit_input, update_orbit
│   │   ├── pan_zoom.rs          # handle_pan_input, handle_zoom
│   │   ├── view_switching.rs    # switch_view_mode, transition_camera
│   │   └── focus_system.rs      # focus_on_selection, fit_to_bounds
│   │
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── panel_systems.rs     # update_panel_visibility
│   │   ├── control_panel.rs     # render_control_panel
│   │   ├── inspector_panel.rs   # render_inspector_panel
│   │   ├── menu_bar.rs          # render_menu_bar
│   │   └── interaction.rs       # handle_ui_interaction
│   │
│   └── io/
│       ├── mod.rs
│       ├── file_loading.rs      # load_json_graph, parse_graph_data
│       ├── file_saving.rs       # save_graph_json, serialize_graph
│       └── auto_save.rs         # periodic_auto_save
│
├── bundles/             # Component bundles for common patterns
│   ├── mod.rs
│   ├── node_bundle.rs   # StandardNode, DecisionNode, EventNode
│   └── edge_bundle.rs   # DirectedEdge, BidirectionalEdge
│
├── plugins/             # Bevy plugins with proper system ordering
│   ├── mod.rs
│   ├── core_plugin.rs   # Core resources and basic setup
│   ├── graph_plugin.rs  # Graph manipulation systems
│   ├── render_plugin.rs # Rendering and visual systems
│   ├── ui_plugin.rs     # UI systems with egui ordering
│   ├── camera_plugin.rs # Camera control systems
│   └── io_plugin.rs     # File I/O systems
│
└── system_sets.rs       # System execution order definitions
```

## 3. System Execution Order

### SystemSet Definitions

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GraphSystemSet {
    // Input collection (raw input events)
    Input,
    // Event processing (handle events, validate)
    EventProcessing,
    // Graph structure updates
    GraphUpdate,
    // Visual component updates
    VisualUpdate,
    // Rendering preparation
    RenderPrep,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UISystemSet {
    // Update UI state based on app state
    StateSync,
    // Process UI events
    EventHandling,
    // Render UI panels (after egui init)
    PanelRender,
    // Render overlays and tooltips
    OverlayRender,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CameraSystemSet {
    // Process camera input
    Input,
    // Update camera state
    StateUpdate,
    // Apply camera transform
    TransformUpdate,
}
```

### Execution Flow

```
PreUpdate:
  └─ Input Collection
      └─ Camera Input
      └─ UI Input Detection

Update:
  └─ GraphSystemSet::EventProcessing
      └─ Validate Events
      └─ Queue Graph Changes
  └─ GraphSystemSet::GraphUpdate
      └─ Apply Structure Changes
      └─ Update Graph Data
  └─ GraphSystemSet::VisualUpdate
      └─ Update Visual Components
      └─ Calculate Positions
  └─ CameraSystemSet (all)
      └─ Process Input
      └─ Update State
      └─ Apply Transform
  └─ After(EguiSet::InitContexts)
      └─ UISystemSet::StateSync
      └─ UISystemSet::EventHandling
      └─ UISystemSet::PanelRender
      └─ UISystemSet::OverlayRender

PostUpdate:
  └─ GraphSystemSet::RenderPrep
      └─ Prepare Render Data
      └─ Update Materials
```

## 4. Implementation Phases

### Phase 1: Emergency Fix (1-2 days) ✅ COMPLETED
**Goal**: Fix the critical panic and flashing issues

1. ✅ Fix UI system ordering in existing code:
   ```rust
   .add_systems(
       Update,
       (ui_systems)
           .after(bevy_egui::EguiSet::InitContexts)
           .before(bevy_egui::EguiSet::BeginFrame)
   )
   ```
2. ✅ Add temporary state checks to prevent panics
3. ✅ Document current system dependencies

### Phase 2: Component Extraction (3-4 days)
**Goal**: Separate data from logic

1. Create `components/` module structure
2. Extract all component definitions from mixed files
3. Create component documentation
4. Define component invariants

### Phase 3: Resource Consolidation (2-3 days)
**Goal**: Minimize global state

1. Create `resources/` module structure
2. Audit current resource usage
3. Convert unnecessary resources to components
4. Document resource access patterns

### Phase 4: Event System (2-3 days) ✅ COMPLETED
**Goal**: Establish clear communication patterns

1. ✅ Create `events/` module structure
2. ✅ Define all event types:
   - **Graph Events**: CreateNodeEvent, DeleteNodeEvent, UpdateNodeEvent, BatchMoveNodesEvent, CreateEdgeEvent, DeleteEdgeEvent, UpdateEdgeEvent, DeferredEdgeEvent, ValidateEdgeConnectionEvent, ValidateNodePropertiesEvent, ValidateGraphEvent, AnalyzeGraphEvent, GraphMetricsEvent, RequestLayoutEvent, CreatePatternEvent, BatchOperationEvent, GraphClipboardEvent, GraphModificationEvent
   - **UI Events**: ShowNotificationEvent, ShowModalEvent, UpdateStatusBarEvent, UpdateToolbarEvent, UpdateSidebarEvent, RegisterShortcutEvent, ChangeLayoutEvent, ShowHelpEvent
   - **I/O Events**: LoadJsonFileEvent, SaveJsonFileEvent, CreateBackupEvent, FileWatchEvent, BatchFileOperationEvent, ValidateFileEvent, AddRecentFileEvent, CreateProjectEvent, LoadProjectEvent, SaveProjectEvent, LoadTemplateEvent, RecoverFileEvent, FileOperationCompleteEvent
   - **Camera Events**: OrbitCameraEvent, PanCameraEvent, AnimateCameraEvent, SetCameraPresetEvent, ConstrainCameraEvent, FollowNodeEvent, TakeScreenshotEvent, UpdateViewportEvent, SaveCameraStateEvent, LoadCameraStateEvent, CameraAnimationCompleteEvent, FocusSelectionEvent, FocusNodeEvent, FitToViewEvent
3. ✅ Document event flow with comprehensive guides:
   - Created `doc/event-flow-guide.md` with mermaid diagrams
   - Created `doc/event-migration-examples.md` with 7 detailed examples
4. ✅ Replace direct mutations with events (compatibility layer in place)

### Phase 5: System Decomposition (5-7 days) ✅ COMPLETED
**Goal**: Create focused, testable systems

1. ✅ Create `systems/` module structure
2. ✅ Break down monolithic systems into focused modules:
   - **Graph Systems** (`src/systems/graph/`):
     - ✅ `creation.rs`: Node/edge spawning, deferred edges, pattern generation
     - ✅ `deletion.rs`: Safe deletion with edge cleanup, batch operations, cut/clipboard
     - ✅ `selection.rs`: Mouse/keyboard selection, hover states, box selection
     - ✅ `movement.rs`: Dragging, alignment, constraints, arrow key movement
     - ✅ `validation.rs`: Property/connection validation, cycle detection, structure analysis
     - ✅ `algorithms.rs`: Pathfinding, layouts (force-directed/hierarchical/circular/grid), metrics
   - **Rendering Systems** (`src/systems/rendering/`):
     - ✅ Module structure created
     - ✅ `node_rendering.rs`: Example implementation with mesh generation
   - **Camera Systems** (`src/systems/camera/`):
     - ✅ Module structure created
     - ✅ `focus_system.rs`: Complete implementation with animations
   - **UI Systems** (`src/systems/ui/`):
     - ✅ Module structure created
   - **I/O Systems** (`src/systems/io/`):
     - ✅ Module structure created
     - ✅ `file_loading.rs`: Complete JSON loading implementation
3. ✅ Implement single-responsibility systems
4. ✅ Add system documentation

### Phase 6: Bundle Implementation (1-2 days)
**Goal**: Simplify entity creation

1. Create common node bundles
2. Create edge bundles
3. Standardize entity spawning
4. Document bundle usage

### Phase 7: Plugin Architecture (3-4 days)
**Goal**: Organize systems with proper ordering

1. Create plugin structure
2. Implement SystemSets
3. Define execution order
4. Add run conditions

### Phase 8: Testing & Optimization (3-4 days)
**Goal**: Ensure stability and performance

1. Add integration tests
2. Profile system performance
3. Optimize hot paths
4. Document performance considerations

## 5. Migration Strategy

### Incremental Approach

1. **Parallel Structure**: Build new structure alongside old
2. **Gradual Migration**: Move systems one at a time
3. **Feature Flags**: Use feature flags for new systems
4. **Compatibility Layer**: Maintain compatibility during transition

### Risk Mitigation

1. **Version Control**: Create feature branch
2. **Regular Testing**: Test after each phase
3. **Rollback Plan**: Keep ability to revert
4. **Documentation**: Document all changes

## 6. Success Metrics

### Technical Metrics
- Zero panics related to system ordering
- No screen flashing
- System execution time < 16ms (60 FPS)
- Memory usage stable
- Clean clippy/lint output

### Code Quality Metrics
- No file > 300 lines
- Single responsibility per system
- Clear module boundaries
- Comprehensive documentation
- Test coverage > 80%

### Developer Experience
- Easy to add new features
- Clear debugging path
- Intuitive module structure
- Fast compile times
- Good error messages

## 7. Long-term Benefits

### Maintainability
- Clear separation of concerns
- Easy to understand codebase
- Modular architecture
- Testable components

### Performance
- Optimized system execution
- Efficient change detection
- Parallel system execution
- Minimal resource contention

### Extensibility
- Plugin-based architecture
- Event-driven communication
- Clear extension points
- Stable API boundaries

## 8. Timeline Summary

- **Week 1**: ✅ Emergency fixes + Component extraction
- **Week 2**: Resources + ✅ Events + ✅ Begin systems
- **Week 3**: ✅ Complete systems + Bundles + Plugins
- **Week 4**: Testing + Optimization + Documentation

Total estimated time: 4 weeks for complete refactoring

## 9. Implementation Status

### ✅ Completed Phases
- **Phase 1**: Emergency fixes implemented
- **Phase 4**: Event system fully implemented with:
  - Comprehensive event definitions for all domains
  - Event flow documentation with diagrams
  - Migration examples for common patterns
  - Compatibility layer for gradual migration
- **Phase 5**: System decomposition completed with:
  - Full graph systems suite (creation, deletion, selection, movement, validation, algorithms)
  - Rendering system structure with example implementations
  - Camera system with complete focus/animation implementation
  - UI and I/O system structures
  - Comprehensive documentation for each system

### 🚧 In Progress
- **Phase 2**: Component extraction
- **Phase 3**: Resource consolidation

### ⏳ Upcoming
- **Phase 6**: Bundle implementation
- **Phase 7**: Plugin architecture
- **Phase 8**: Testing & optimization

## 10. Next Steps

1. ✅ Get team buy-in on architecture
2. ✅ Create feature branch
3. ✅ Implement Phase 1 emergency fixes
4. ✅ Complete Phase 4 event system
5. ✅ Complete Phase 5 system decomposition
6. Continue with Phase 2 component extraction
7. Regular progress reviews

This refactoring will transform Alchemist from a monolithic application into a well-architected ECS-based system that is maintainable, performant, and extensible.
