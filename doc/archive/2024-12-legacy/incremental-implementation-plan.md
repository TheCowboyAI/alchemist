# Incremental Implementation Plan

## Overview

This plan reflects our current DDD-compliant implementation and provides a step-by-step approach to enhance the graph system one component at a time.

## Current State

### ✅ Implemented (100% DDD-Compliant)

```
src/contexts/
├── graph_management/     # Core domain
│   ├── domain.rs        # Graph, Node, Edge entities
│   ├── events.rs        # GraphCreated, NodeAdded, etc.
│   ├── services.rs      # CreateGraph, AddNodeToGraph, etc.
│   ├── repositories.rs  # Graphs, GraphEvents, Nodes, Edges
│   └── plugin.rs
├── visualization/       # Supporting domain
│   ├── services.rs      # RenderGraphElements, AnimateGraphElements
│   └── plugin.rs
└── selection/           # Selection domain ✅ PHASE 2 COMPLETE
    ├── domain.rs        # SelectionState, SelectionMode, etc.
    ├── events.rs        # NodeSelected, EdgeDeselected, etc.
    ├── services.rs      # ManageSelection, ProcessSelectionInput, etc.
    ├── plugin.rs        # Selection plugin integration
    ├── tests.rs         # Comprehensive test coverage
    └── test_utils.rs    # Test isolation utilities
```

### Working Features
- ✅ Graph creation with metadata
- ✅ Node creation and positioning
- ✅ Edge creation and rendering
- ✅ 3D node visualization (blue spheres)
- ✅ Edge visualization (lines between nodes)
- ✅ Camera controls (Panorbit integration)
- ✅ Graph rotation animation
- ✅ Event-driven architecture
- ✅ **PHASE 2 COMPLETE**: Full selection system
  - ✅ Mouse selection with raycasting
  - ✅ Keyboard shortcuts (Ctrl+A, Ctrl+I, Tab)
  - ✅ Box selection (Shift+drag)
  - ✅ Multi-selection modes
  - ✅ Visual feedback and highlighting
  - ✅ Animation-aware selection
  - ✅ Connected nodes selection

## Implementation Phases

Each phase focuses on implementing one component or service at a time, ensuring we can test and validate each addition.

## Phase 1: Edge Visualization ✅ COMPLETE

### Component 1.1: Edge Rendering Service ✅

**File**: `src/contexts/visualization/services.rs`

**Add Service**: `RenderGraphEdges` ✅

```rust
pub struct RenderGraphEdges;

impl RenderGraphEdges {
    /// Creates visual representation for edges
    pub fn render_edge(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        source_pos: Vec3,
        target_pos: Vec3,
        edge_entity: Entity,
    ) {
        // Implementation
    }

    /// System that listens for EdgeConnected events
    pub fn visualize_new_edges(
        mut commands: Commands,
        mut events: EventReader<EdgeConnected>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        nodes: Query<(&NodeIdentity, &Transform)>,
        edges: Query<(Entity, &EdgeIdentity, &EdgeRelationship)>,
    ) {
        // Implementation
    }
}
```

**Success Criteria** ✅:
- ✅ Edges render as lines between nodes
- ✅ Edge color differs from nodes
- ✅ System responds to EdgeConnected events

### Component 1.2: Edge Components ✅

**File**: `src/contexts/graph_management/domain.rs`

**Add Components** ✅:
```rust
/// Visual representation of an edge
#[derive(Component)]
pub struct EdgeVisual {
    pub line_thickness: f32,
    pub arrow_size: f32,
}

/// Bundle for edge visualization
#[derive(Bundle)]
pub struct EdgeVisualBundle {
    pub visual: EdgeVisual,
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<StandardMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}
```

**Success Criteria** ✅:
- ✅ Edge entities can have visual components
- ✅ Bundles properly integrate with Bevy

## Phase 2: Selection System ✅ COMPLETE

### Component 2.1: Selection Components ✅

**File**: `src/contexts/selection/domain.rs`

**Add Components** ✅:
```rust
/// Marks an entity as selectable
#[derive(Component)]
pub struct Selectable;

/// Tracks selection state
#[derive(Component)]
pub struct Selected;

/// Highlights selected entities
#[derive(Component)]
pub struct SelectionHighlight {
    pub original_color: Color,
    pub highlight_color: Color,
}
```

### Component 2.2: Selection Service ✅

**File**: `src/contexts/selection/services.rs`

```rust
impl ProcessSelectionInput {
    /// Enhanced selection with raycasting
    pub fn handle_mouse_selection(
        windows: Query<&Window>,
        camera: Query<(&Camera, &GlobalTransform)>,
        selectables: Query<(Entity, &Transform, &Selectable), Without<Selected>>,
        selected: Query<Entity, With<Selected>>,
        mut commands: Commands,
        mouse_button: Res<ButtonInput<MouseButton>>,
    ) {
        // Implementation
    }

    /// Highlights selected entities
    pub fn update_selection_visuals(
        mut materials: ResMut<Assets<StandardMaterial>>,
        selected_query: Query<&MeshMaterial3d<StandardMaterial>, Added<Selected>>,
        deselected_query: Query<&MeshMaterial3d<StandardMaterial>, Without<Selected>>,
    ) {
        // Implementation
    }
}
```

**Success Criteria** ✅:
- ✅ Click on node to select
- ✅ Selected nodes change color
- ✅ Click empty space to deselect
- ✅ Multi-selection with Ctrl
- ✅ Box selection with Shift
- ✅ Keyboard shortcuts work
- ✅ Animation-aware selection

## Phase 3: Storage Layer 🚧 CURRENT PRIORITY

### Component 3.1: Daggy Integration

**File**: `src/contexts/graph_management/storage.rs` (new file)

**Add Storage**:
```rust
use daggy::{Dag, NodeIndex, EdgeIndex};

/// Primary graph storage using Daggy
pub struct GraphStorage {
    graphs: HashMap<GraphIdentity, Dag<NodeData, EdgeData>>,
    node_indices: HashMap<(GraphIdentity, NodeIdentity), NodeIndex>,
    edge_indices: HashMap<(GraphIdentity, EdgeIdentity), EdgeIndex>,
}

/// Node data stored in Daggy
#[derive(Clone, Debug)]
pub struct NodeData {
    pub identity: NodeIdentity,
    pub content: NodeContent,
    pub position: SpatialPosition,
}

/// Edge data stored in Daggy
#[derive(Clone, Debug)]
pub struct EdgeData {
    pub identity: EdgeIdentity,
    pub relationship: EdgeRelationship,
}
```

### Component 3.2: Storage Sync Service

**Add Service**: `SyncGraphWithStorage`

```rust
pub struct SyncGraphWithStorage;

impl SyncGraphWithStorage {
    /// Syncs ECS entities with Daggy storage
    pub fn sync_to_storage(
        storage: ResMut<GraphStorage>,
        events: EventReader<GraphCreated>,
        // Other event readers
    ) {
        // Implementation
    }

    /// Loads graphs from storage to ECS
    pub fn load_from_storage(
        storage: Res<GraphStorage>,
        graph_id: GraphIdentity,
        commands: &mut Commands,
    ) {
        // Implementation
    }
}
```

**Success Criteria**:
- Graphs persist in Daggy
- Can reload graphs from storage
- Sync maintains consistency

## Phase 4: Layout Algorithms

### Component 4.1: Layout Services

**File**: `src/contexts/visualization/layout.rs` (new file)

**Add Services**:
```rust
/// Calculates force-directed layout
pub struct CalculateForceDirectedLayout {
    pub repulsion_strength: f32,
    pub attraction_strength: f32,
    pub damping: f32,
}

impl CalculateForceDirectedLayout {
    pub fn execute(
        &self,
        nodes: Query<(Entity, &Transform, &NodeIdentity)>,
        edges: Query<&EdgeRelationship>,
    ) -> HashMap<NodeIdentity, Vec3> {
        // Implementation using physics
    }
}

/// Applies calculated layout
pub struct ApplyGraphLayout;

impl ApplyGraphLayout {
    pub fn execute(
        positions: HashMap<NodeIdentity, Vec3>,
        mut nodes: Query<(&NodeIdentity, &mut Transform)>,
    ) {
        // Animate to new positions
    }
}
```

**Success Criteria**:
- Nodes arrange automatically
- Smooth animation to positions
- Can trigger layout manually

## Phase 5: Import/Export

### Component 5.1: JSON Serialization

**File**: `src/contexts/graph_management/formats/json.rs` (new file)

**Add Services**:
```rust
/// Serializes graph to JSON
pub struct SerializeGraphToJson;

impl SerializeGraphToJson {
    pub fn execute(
        &self,
        graph_id: GraphIdentity,
        storage: &GraphStorage,
    ) -> Result<String, SerializationError> {
        // Implementation
    }
}

/// Deserializes graph from JSON
pub struct DeserializeGraphFromJson;

impl DeserializeGraphFromJson {
    pub fn execute(
        &self,
        json: &str,
        commands: &mut Commands,
        events: &mut EventWriter<GraphCreated>,
    ) -> Result<GraphIdentity, DeserializationError> {
        // Implementation
    }
}
```

**Success Criteria**:
- Export graph to JSON file
- Import JSON creates graph
- Round-trip preserves data

## Implementation Guidelines

### 1. One Component at a Time
- Implement complete component before moving on
- Test thoroughly with existing features
- Commit after each working component

### 2. Event-Driven Updates
- All state changes through events
- Services respond to events
- No direct component modification

### 3. Testing Each Component
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_component_functionality() {
        // Test the specific component
    }
}
```

### 4. Documentation
- Document each new service
- Update vocabulary.md with new terms
- Add examples for each feature

## Success Metrics

### Phase 1 (Edge Visualization)
- [ ] Edges render between connected nodes
- [ ] Edge events trigger visualization
- [ ] Performance remains at 60 FPS

### Phase 2 (Selection)
- [ ] Can select nodes with mouse
- [ ] Visual feedback for selection
- [ ] Multi-select with Shift key

### Phase 3 (Storage)
- [ ] Graphs persist to Daggy
- [ ] Can reload graphs
- [ ] Event replay works

### Phase 4 (Layout)
- [ ] Force-directed layout works
- [ ] Smooth animations
- [ ] Manual positioning preserved

### Phase 5 (Import/Export)
- [ ] JSON import/export works
- [ ] Data integrity maintained
- [ ] Error handling graceful

## Next Steps

1. **Immediate**: Implement Phase 1.1 (Edge Rendering Service)
2. **This Week**: Complete Phase 1 (Edge Visualization)
3. **Next Week**: Begin Phase 2 (Selection System)

This incremental approach ensures steady progress while maintaining system stability and DDD compliance.
