# Graph Aggregate v2 Design

## Overview

This document outlines the enhanced Graph aggregate design that supports:
- Multiple serialization formats (JSON, Cypher, Mermaid)
- 2D/3D visualization modes
- Event-driven manipulation with full correlation
- Event replay and animation
- Separation of persisted and working models

## Core Architecture

### 1. Graph Storage Layer (Daggy Integration)

```rust
/// Central repository for all graphs in the system
#[derive(Resource)]
pub struct GraphRepository {
    /// Persistent graph storage using Daggy
    graphs: HashMap<GraphId, daggy::Dag<NodeData, EdgeData>>,
    /// Event store for all graph operations
    event_store: EventStore,
}

/// Node data stored in Daggy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    pub id: NodeId,
    pub cid: Option<Cid>, // For Merkle DAG support
    pub properties: HashMap<String, serde_json::Value>,
    pub position_3d: Vec3,
    pub position_2d: Vec2,
    pub node_type: String,
    pub created_at: SystemTime,
    pub modified_at: SystemTime,
}

/// Edge data stored in Daggy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeData {
    pub id: EdgeId,
    pub weight: f32,
    pub edge_type: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub created_at: SystemTime,
}
```

### 2. ECS Components (Visual Layer)

```rust
/// Component marking a graph entity (aggregate root)
#[derive(Component)]
pub struct GraphAggregate {
    pub id: GraphId,
    pub metadata: GraphMetadata,
    /// Reference to the Daggy graph
    pub dag_version: u64,
}

/// Component linking an entity to a Daggy node
#[derive(Component)]
pub struct GraphNodeRef {
    pub graph_id: GraphId,
    pub node_index: daggy::NodeIndex,
    pub node_id: NodeId,
    /// Version for detecting updates
    pub version: u64,
}

/// Component linking an entity to a Daggy edge
#[derive(Component)]
pub struct GraphEdgeRef {
    pub graph_id: GraphId,
    pub edge_index: daggy::EdgeIndex,
    pub edge_id: EdgeId,
    pub source_entity: Entity,
    pub target_entity: Entity,
    pub version: u64,
}

/// View state for 2D/3D switching
#[derive(Component)]
pub struct GraphViewState {
    pub mode: ViewMode,
    pub camera_2d: Option<Entity>,
    pub camera_3d: Option<Entity>,
    pub active_camera: Entity,
}

#[derive(Debug, Clone, Copy)]
pub enum ViewMode {
    TwoD,
    ThreeD,
}
```

### 3. Enhanced Event System

```rust
/// Base trait for all graph events
pub trait GraphEvent: Event + Clone + Debug {
    fn graph_id(&self) -> GraphId;
    fn aggregate_version(&self) -> u64;
    fn event_id(&self) -> Uuid;
    fn timestamp(&self) -> SystemTime;
}

/// Event envelope with correlation metadata
#[derive(Debug, Clone, Event)]
pub struct EventEnvelope<E: GraphEvent> {
    /// The actual domain event
    pub event: E,
    /// Unique event ID
    pub event_id: Uuid,
    /// ID linking related events
    pub correlation_id: Uuid,
    /// ID of the event that caused this one
    pub causation_id: Option<Uuid>,
    /// Sequence number for ordering
    pub sequence: u64,
    /// Entity that triggered the event
    pub triggered_by: Option<Entity>,
    /// Metadata for replay/animation
    pub metadata: EventMetadata,
}

#[derive(Debug, Clone)]
pub struct EventMetadata {
    pub animation_duration: Duration,
    pub animation_type: AnimationType,
    pub user_action: Option<String>,
}

/// Event store for persistence and replay
#[derive(Default)]
pub struct EventStore {
    /// All events indexed by graph ID
    events: HashMap<GraphId, Vec<EventEnvelope<Box<dyn GraphEvent>>>>,
    /// Global sequence counter
    sequence: AtomicU64,
}
```

### 4. Import/Export System

```rust
/// Trait for graph serialization
pub trait GraphSerializer {
    fn export(&self, graph: &daggy::Dag<NodeData, EdgeData>) -> Result<String, SerializationError>;
    fn import(&self, data: &str) -> Result<daggy::Dag<NodeData, EdgeData>, SerializationError>;
    fn validate(&self, data: &str) -> Result<(), ValidationError>;
}

/// JSON serializer implementation
pub struct JsonGraphSerializer;

/// Cypher query serializer
pub struct CypherGraphSerializer {
    /// Configuration for Cypher generation
    pub config: CypherConfig,
}

/// Mermaid diagram serializer
pub struct MermaidGraphSerializer {
    /// Diagram type and styling options
    pub style: MermaidStyle,
}
```

### 5. Animation and Replay System

```rust
/// Component for animating graph changes
#[derive(Component)]
pub struct GraphAnimator {
    /// Current animation state
    pub state: AnimationState,
    /// Timeline of events to replay
    pub timeline: Vec<EventEnvelope<Box<dyn GraphEvent>>>,
    /// Current position in timeline
    pub position: usize,
    /// Playback speed multiplier
    pub speed: f32,
    /// Interpolation data
    pub interpolations: HashMap<Entity, InterpolationData>,
}

#[derive(Debug, Clone)]
pub enum AnimationState {
    Idle,
    Playing { start_time: Instant },
    Paused { elapsed: Duration },
    Rewinding,
}

/// Resource for controlling replay
#[derive(Resource)]
pub struct ReplayController {
    pub target_graph: Option<GraphId>,
    pub mode: ReplayMode,
    pub filters: ReplayFilters,
}
```

## System Architecture

### Core Systems

1. **GraphSyncSystem**: Syncs Daggy changes to ECS entities
2. **GraphEventSystem**: Processes events and updates Daggy
3. **GraphRenderSystem**: Handles 2D/3D rendering
4. **GraphAnimationSystem**: Manages replay and transitions
5. **GraphSerializationSystem**: Import/export operations

### Event Flow

```
User Action → Command → Event Creation → Event Store → Daggy Update → ECS Sync → Render
                          ↓
                    Event Correlation
                          ↓
                    Animation Queue
```

## Usage Examples

### Creating a Graph with Events

```rust
// Create graph
let graph_id = GraphId::new();
let create_event = EventEnvelope::new(
    GraphCreatedEvent {
        graph_id,
        metadata: GraphMetadata {
            name: "Knowledge Graph".to_string(),
            domain_type: "knowledge".to_string(),
            ..default()
        },
    },
    correlation_id,
);

// Add node with animation
let node_event = EventEnvelope::new(
    NodeAddedEvent {
        graph_id,
        node_id: NodeId::new(),
        data: NodeData {
            position_3d: Vec3::new(0.0, 0.0, 0.0),
            position_2d: Vec2::new(100.0, 100.0),
            node_type: "Concept".to_string(),
            ..default()
        },
    },
    correlation_id,
)
.with_animation(AnimationType::FadeIn, Duration::from_millis(500));
```

### Replaying Graph Construction

```rust
fn setup_replay(
    mut replay: ResMut<ReplayController>,
    graphs: Query<&GraphAggregate>,
) {
    if let Some(graph) = graphs.iter().next() {
        replay.target_graph = Some(graph.id);
        replay.mode = ReplayMode::Sequential;
        replay.filters = ReplayFilters {
            event_types: vec!["NodeAdded", "EdgeCreated"],
            time_range: None,
        };
    }
}
```

### Switching View Modes

```rust
fn toggle_view_mode(
    mut graphs: Query<&mut GraphViewState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        for mut view_state in graphs.iter_mut() {
            view_state.mode = match view_state.mode {
                ViewMode::TwoD => ViewMode::ThreeD,
                ViewMode::ThreeD => ViewMode::TwoD,
            };
        }
    }
}
```

## Migration Plan

1. **Phase 1**: Implement GraphRepository and Daggy integration
2. **Phase 2**: Enhance event system with correlation
3. **Phase 3**: Add serialization support
4. **Phase 4**: Implement animation and replay
5. **Phase 5**: Add graph theory operations

## Benefits

- **Separation of Concerns**: Domain model (Daggy) separate from visualization (ECS)
- **Event Sourcing**: Complete history with replay capability
- **Multiple Views**: Seamless 2D/3D switching
- **Extensibility**: Easy to add new formats and animations
- **Performance**: Daggy for algorithms, ECS for rendering
