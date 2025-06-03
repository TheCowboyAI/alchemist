# Implementation Plan for Missing Features

## Overview

This plan provides step-by-step instructions for implementing all missing features identified in the functionality audit. Each feature includes clear acceptance criteria, implementation steps, and test verification.

## Implementation Priority Order

Features are ordered by dependency and complexity, starting with foundational features.

---

## Phase 1: Core Infrastructure (Foundation)

### 1.1 Event-Driven Architecture with Audit Trail

**Why First**: Other features depend on proper event handling

**Acceptance Criteria**:
- Every state change generates an event
- Events are stored in an append-only log
- State can be rebuilt from event history
- Events are serializable and timestamped

**Implementation Steps**:

1. Create event store module:
```rust
// src/contexts/event_store/mod.rs
pub struct EventStore {
    events: Vec<DomainEvent>,
    snapshots: HashMap<u64, GraphState>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DomainEvent {
    id: Uuid,
    timestamp: SystemTime,
    event_type: EventType,
    payload: serde_json::Value,
    metadata: EventMetadata,
}
```

2. Add event sourcing to graph operations:
```rust
// src/contexts/graph_management/events.rs
#[derive(Event, Serialize, Deserialize)]
pub enum GraphEvent {
    NodeAdded { id: NodeIdentity, data: NodeData },
    NodeRemoved { id: NodeIdentity },
    EdgeAdded { id: EdgeIdentity, data: EdgeData },
    EdgeRemoved { id: EdgeIdentity },
    GraphLoaded { id: GraphIdentity },
    GraphSaved { id: GraphIdentity, path: PathBuf },
}
```

3. Implement event replay system:
```rust
// src/contexts/event_store/replay.rs
pub fn replay_events(events: &[DomainEvent]) -> Result<GraphState, Error> {
    let mut state = GraphState::default();
    for event in events {
        state.apply(event)?;
    }
    Ok(state)
}
```

**Tests to Update**:
- `test_event_audit_trail()` should pass
- `test_event_sourcing()` should pass
- `test_event_replay()` should pass

---

### 1.2 File I/O with Dialog Support

**Why Early**: Many features need proper file handling

**Acceptance Criteria**:
- File dialog for import (not hardcoded path)
- Support multiple formats (JSON, GraphML, GEXF)
- Create new empty graph
- Round-trip data preservation

**Implementation Steps**:

1. Create file format abstraction:
```rust
// src/contexts/import_export/formats.rs
pub trait GraphFormat {
    fn export(&self, graph: &GraphData) -> Result<String, Error>;
    fn import(&self, content: &str) -> Result<GraphData, Error>;
}

pub struct JsonFormat;
pub struct GraphMLFormat;
pub struct GexfFormat;
```

2. Update importer to use file dialog:
```rust
// src/contexts/graph_management/importer.rs
pub fn import_with_dialog() -> Result<GraphData, Error> {
    let file = rfd::FileDialog::new()
        .add_filter("Graph files", &["json", "graphml", "gexf"])
        .pick_file()
        .ok_or(Error::Cancelled)?;

    let content = fs::read_to_string(file)?;
    let format = detect_format(&file)?;
    format.import(&content)
}
```

3. Add "New Graph" functionality:
```rust
// src/contexts/graph_management/services.rs
pub fn create_new_graph(name: String) -> GraphData {
    GraphData {
        identity: GraphIdentity::new(),
        metadata: GraphMetadata {
            name,
            created: SystemTime::now(),
            // ... other fields
        },
        nodes: vec![],
        edges: vec![],
    }
}
```

**Tests to Update**:
- `test_import_from_user_selected_file()` should pass
- `test_create_new_graph()` should pass
- `test_json_round_trip_preserves_all_data()` should pass

---

## Phase 2: Interactive Editing

### 2.1 Node and Edge Manipulation

**Acceptance Criteria**:
- Right-click context menu
- Add/delete nodes interactively
- Drag to create edges
- Delete selected items
- Edit properties dialog

**Implementation Steps**:

1. Add context menu system:
```rust
// src/contexts/ui/context_menu.rs
#[derive(Component)]
pub struct ContextMenu {
    position: Vec2,
    items: Vec<MenuItem>,
}

pub fn show_context_menu(
    windows: Query<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
) {
    if mouse.just_pressed(MouseButton::Right) {
        // Show menu at cursor position
    }
}
```

2. Implement node creation:
```rust
// src/contexts/graph_management/interaction.rs
pub fn handle_add_node(
    cursor_pos: Vec3,
    mut commands: Commands,
    mut graph_events: EventWriter<GraphEvent>,
) {
    let node = create_node_at_position(cursor_pos);
    commands.spawn(NodeBundle::from(node.clone()));
    graph_events.send(GraphEvent::NodeAdded {
        id: node.identity,
        data: node.into()
    });
}
```

3. Add edge creation by dragging:
```rust
// src/contexts/graph_management/edge_creation.rs
#[derive(Resource)]
pub struct EdgeCreationState {
    start_node: Option<Entity>,
    preview_line: Option<Entity>,
}

pub fn handle_edge_drag(
    mouse: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<EdgeCreationState>,
    nodes: Query<(Entity, &Transform), With<Node>>,
) {
    // Implement drag detection and edge creation
}
```

**Tests to Update**:
- `test_add_node_interactively()` should pass
- `test_delete_nodes()` should pass
- `test_create_edge_by_dragging()` should pass

---

## Phase 3: Visualization Modes

### 3.1 2D/3D Mode Switching

**Acceptance Criteria**:
- Press key to toggle between 2D and 3D
- 2D mode shows orthographic top-down view
- 3D mode shows perspective view
- Smooth transition animation

**Implementation Steps**:

1. Create view mode enum:
```rust
// src/contexts/visualization/view_mode.rs
#[derive(Resource, Default)]
pub enum ViewMode {
    #[default]
    ThreeD,
    TwoD,
}

pub fn toggle_view_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut view_mode: ResMut<ViewMode>,
    mut cameras: Query<(&mut Projection, &mut Transform)>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
        *view_mode = match *view_mode {
            ViewMode::ThreeD => ViewMode::TwoD,
            ViewMode::TwoD => ViewMode::ThreeD,
        };
        update_camera_for_mode(&view_mode, &mut cameras);
    }
}
```

2. Implement 2D layout algorithm:
```rust
// src/contexts/layout/layout_2d.rs
pub fn apply_2d_layout(
    mut nodes: Query<&mut Transform, With<Node>>,
) {
    // Force all nodes to Y=0 for 2D view
    // Apply 2D force-directed layout
}
```

**Tests to Update**:
- `test_2d_mode_exists()` should pass
- `test_3d_to_2d_switching()` should pass

---

## Phase 4: Performance Optimization

### 4.1 Large Graph Handling

**Acceptance Criteria**:
- Render 250k+ nodes at 60 FPS
- Implement LOD system
- Add frustum culling
- Use GPU instancing

**Implementation Steps**:

1. Implement LOD system:
```rust
// src/contexts/visualization/lod.rs
#[derive(Component)]
pub struct LevelOfDetail {
    distance_thresholds: Vec<f32>,
    mesh_handles: Vec<Handle<Mesh>>,
}

pub fn update_lod(
    camera: Query<&Transform, With<Camera>>,
    mut nodes: Query<(&Transform, &mut Handle<Mesh>, &LevelOfDetail)>,
) {
    // Switch mesh based on distance from camera
}
```

2. Add GPU instancing:
```rust
// src/contexts/visualization/instancing.rs
pub fn setup_instanced_rendering(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Use Bevy's instancing API for nodes
}
```

3. Implement spatial indexing:
```rust
// src/contexts/graph_management/spatial_index.rs
pub struct SpatialIndex {
    octree: Octree<Entity>,
}

impl SpatialIndex {
    pub fn query_visible(&self, frustum: &Frustum) -> Vec<Entity> {
        // Return only entities within frustum
    }
}
```

**Tests to Update**:
- `test_handle_250k_elements()` should pass
- `test_maintain_60_fps_with_large_graphs()` should pass

---

## Phase 5: Advanced Features

### 5.1 Subgraph Composition

**Acceptance Criteria**:
- Load multiple graphs simultaneously
- Maintain separate graph identities
- Compose graphs while preserving structure
- Visual distinction between subgraphs

**Implementation Steps**:

1. Create subgraph management:
```rust
// src/contexts/graph_management/subgraph.rs
#[derive(Resource)]
pub struct SubgraphManager {
    graphs: HashMap<GraphIdentity, GraphData>,
    active_graph: Option<GraphIdentity>,
}

pub fn load_additional_graph(
    mut manager: ResMut<SubgraphManager>,
    path: PathBuf,
) -> Result<GraphIdentity, Error> {
    let graph = load_graph_from_file(path)?;
    let id = graph.identity;
    manager.graphs.insert(id, graph);
    Ok(id)
}
```

**Tests to Update**:
- `test_load_multiple_graphs()` should pass
- `test_maintain_subgraph_structure()` should pass

### 5.2 WASM Plugin System

**Acceptance Criteria**:
- Load WASM plugins at runtime
- Plugins can add custom algorithms
- Plugins can add visualizations
- Safe sandboxed execution

**Implementation Steps**:

1. Create plugin interface:
```rust
// src/contexts/plugins/mod.rs
pub trait GraphPlugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn execute(&self, graph: &mut GraphData) -> Result<(), Error>;
}

pub struct WasmPluginLoader {
    runtime: wasmtime::Engine,
}
```

**Tests to Update**:
- `test_wasm_plugin_loading()` should pass

### 5.3 Real-time Collaboration

**Acceptance Criteria**:
- Multiple users can connect
- Changes sync in real-time
- Conflict resolution
- User awareness (cursors, selections)

**Implementation Steps**:

1. Add networking layer:
```rust
// src/contexts/collaboration/network.rs
pub struct CollaborationServer {
    sessions: HashMap<SessionId, UserSession>,
    graph_state: Arc<RwLock<GraphData>>,
}

pub struct CollaborationClient {
    connection: WebSocket,
    local_changes: Vec<GraphEvent>,
}
```

**Tests to Update**:
- `test_multi_user_connection()` should pass
- `test_real_time_sync()` should pass

### 5.4 AI Integration

**Acceptance Criteria**:
- AI agents for pattern recognition
- Layout optimization suggestions
- Anomaly detection
- Natural language graph queries

**Implementation Steps**:

1. Create AI agent interface:
```rust
// src/contexts/ai/mod.rs
pub trait AIAgent {
    fn analyze_graph(&self, graph: &GraphData) -> Analysis;
    fn suggest_optimizations(&self, graph: &GraphData) -> Vec<Suggestion>;
}
```

**Tests to Update**:
- `test_ai_agent_exists()` should pass
- `test_pattern_recognition()` should pass

---

## Implementation Schedule

### Week 1-2: Phase 1 (Core Infrastructure)
- Event sourcing system
- File I/O improvements

### Week 3-4: Phase 2 (Interactive Editing)
- Context menus
- Node/edge manipulation

### Week 5-6: Phase 3 (Visualization)
- 2D/3D modes
- View transitions

### Week 7-8: Phase 4 (Performance)
- LOD and culling
- GPU optimization

### Week 9-12: Phase 5 (Advanced Features)
- Subgraphs
- Plugins
- Collaboration
- AI

## Success Metrics

1. All 60+ failing tests now pass
2. README claims match actual functionality
3. Performance benchmarks meet targets
4. User workflows are smooth and intuitive

## Notes for AI Implementation

1. **Start Small**: Implement one feature at a time
2. **Test First**: Update tests before implementation
3. **Incremental**: Make small, testable changes
4. **Document**: Update docs as you implement
5. **Ask Questions**: If unclear, ask for clarification
6. **Use Examples**: Reference `/samples` for Bevy patterns
