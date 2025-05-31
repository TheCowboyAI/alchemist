# Graph Aggregate Design (DDD Compliant)

## Overview

This document presents the Graph aggregate design following strict DDD naming conventions. All names are derived from the business domain vocabulary, avoiding technical suffixes unless they are part of the domain language.

## Domain Language Glossary

- **Graph**: A collection of interconnected information elements
- **Node**: An information element within a graph
- **Edge**: A relationship between nodes
- **Journey**: The sequence of events that created a graph
- **Snapshot**: A point-in-time state of a graph
- **Perspective**: A view mode (2D or 3D) for visualizing graphs

## Core Domain Model

### 1. Graph Aggregate (Domain Layer)

```rust
/// Graph aggregate root - represents a knowledge graph
#[derive(Component)]
pub struct Graph {
    pub identity: GraphIdentity,
    pub metadata: GraphMetadata,
    pub journey: GraphJourney,
}

/// Value object for graph identity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GraphIdentity(pub Uuid);

/// Value object for graph metadata
#[derive(Debug, Clone)]
pub struct GraphMetadata {
    pub name: String,
    pub description: String,
    pub domain: String, // "knowledge", "workflow", etc.
    pub created: SystemTime,
    pub modified: SystemTime,
    pub tags: Vec<String>,
}

/// Value object tracking the graph's event journey
#[derive(Debug, Clone)]
pub struct GraphJourney {
    pub version: u64,
    pub event_count: u64,
    pub last_event: Option<EventIdentity>,
}
```

### 2. Node and Edge Entities

```rust
/// Node entity within a graph
#[derive(Component)]
pub struct Node {
    pub identity: NodeIdentity,
    pub graph: GraphIdentity,
    pub content: NodeContent,
    pub position: SpatialPosition,
}

/// Value object for node identity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeIdentity(pub Uuid);

/// Value object for node content
#[derive(Debug, Clone)]
pub struct NodeContent {
    pub label: String,
    pub category: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub merkle_hash: Option<ContentHash>, // For Merkle DAG
}

/// Value object for spatial positioning
#[derive(Debug, Clone, Copy)]
pub struct SpatialPosition {
    pub coordinates_3d: Vec3,
    pub coordinates_2d: Vec2,
}

/// Edge entity connecting nodes
#[derive(Component)]
pub struct Edge {
    pub identity: EdgeIdentity,
    pub graph: GraphIdentity,
    pub relationship: EdgeRelationship,
}

/// Value object for edge identity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeIdentity(pub Uuid);

/// Value object for edge relationship
#[derive(Debug, Clone)]
pub struct EdgeRelationship {
    pub source: NodeIdentity,
    pub target: NodeIdentity,
    pub category: String, // "implements", "uses", "contains"
    pub strength: f32,
    pub properties: HashMap<String, serde_json::Value>,
}
```

### 3. Domain Events (Following DDD Naming)

```rust
/// Base trait for all graph domain events
pub trait GraphDomainEvent: Event + Clone + Debug {
    fn graph(&self) -> GraphIdentity;
    fn identity(&self) -> EventIdentity;
    fn occurred(&self) -> SystemTime;
    fn correlation(&self) -> EventCorrelation;
}

/// Value object for event identity
#[derive(Debug, Clone, Copy)]
pub struct EventIdentity(pub Uuid);

/// Value object for event correlation
#[derive(Debug, Clone)]
pub struct EventCorrelation {
    pub sequence: u64,
    pub causation: Option<EventIdentity>,
    pub correlation: Uuid,
    pub actor: Option<ActorIdentity>,
}

// Domain Events (proper DDD naming)

#[derive(Debug, Clone, Event)]
pub struct GraphCreatedEvent {
    pub graph: GraphIdentity,
    pub metadata: GraphMetadata,
    pub correlation: EventCorrelation,
}

#[derive(Debug, Clone, Event)]
pub struct NodeAddedEvent {
    pub graph: GraphIdentity,
    pub node: NodeIdentity,
    pub content: NodeContent,
    pub position: SpatialPosition,
    pub correlation: EventCorrelation,
}

#[derive(Debug, Clone, Event)]
pub struct EdgeConnectedEvent {
    pub graph: GraphIdentity,
    pub edge: EdgeIdentity,
    pub relationship: EdgeRelationship,
    pub correlation: EventCorrelation,
}

#[derive(Debug, Clone, Event)]
pub struct NodeRepositionedEvent {
    pub graph: GraphIdentity,
    pub node: NodeIdentity,
    pub previous_position: SpatialPosition,
    pub new_position: SpatialPosition,
    pub correlation: EventCorrelation,
}

#[derive(Debug, Clone, Event)]
pub struct GraphSnapshotTakenEvent {
    pub graph: GraphIdentity,
    pub snapshot: GraphSnapshot,
    pub correlation: EventCorrelation,
}
```

### 4. Repository (Following DDD Pattern)

```rust
/// Repository for graph aggregates
#[derive(Resource)]
pub struct GraphRepository {
    storage: GraphStorage,
    snapshots: SnapshotStorage,
}

/// Internal storage using Daggy (implementation detail)
struct GraphStorage {
    graphs: HashMap<GraphIdentity, daggy::Dag<NodeContent, EdgeRelationship>>,
    node_index: HashMap<(GraphIdentity, NodeIdentity), daggy::NodeIndex>,
    edge_index: HashMap<(GraphIdentity, EdgeIdentity), daggy::EdgeIndex>,
}

impl GraphRepository {
    pub fn store(&mut self, graph: Graph) -> Result<(), StorageError> {
        // Implementation
    }

    pub fn find(&self, identity: GraphIdentity) -> Option<&Graph> {
        // Implementation
    }

    pub fn find_all(&self) -> Vec<&Graph> {
        // Implementation
    }
}
```

### 5. Domain Services

```rust
/// Domain service for graph layout algorithms
pub struct GraphLayoutDomainService {
    strategies: HashMap<String, Box<dyn LayoutStrategy>>,
}

impl GraphLayoutDomainService {
    pub fn apply_layout(
        &self,
        graph: &Graph,
        strategy_name: &str,
    ) -> Result<LayoutResult, LayoutError> {
        // Implementation
    }
}

/// Domain service for graph analysis
pub struct GraphAnalysisDomainService;

impl GraphAnalysisDomainService {
    pub fn find_shortest_path(
        &self,
        graph: &Graph,
        source: NodeIdentity,
        target: NodeIdentity,
    ) -> Option<Path> {
        // Implementation
    }

    pub fn detect_cycles(&self, graph: &Graph) -> Vec<Cycle> {
        // Implementation
    }
}

/// Domain service for graph serialization
pub struct GraphSerializationDomainService {
    formats: HashMap<String, Box<dyn GraphFormat>>,
}

pub trait GraphFormat {
    fn export(&self, graph: &Graph) -> Result<String, FormatError>;
    fn import(&self, content: &str) -> Result<Graph, FormatError>;
}
```

### 6. Value Objects for Visualization

```rust
/// Value object for graph perspective (2D/3D view)
#[derive(Debug, Clone, Copy, Component)]
pub struct GraphPerspective {
    pub mode: PerspectiveMode,
    pub camera_configuration: CameraConfiguration,
}

#[derive(Debug, Clone, Copy)]
pub enum PerspectiveMode {
    TwoDimensional,
    ThreeDimensional,
}

/// Value object for camera configuration
#[derive(Debug, Clone, Copy)]
pub struct CameraConfiguration {
    pub position: Vec3,
    pub target: Vec3,
    pub field_of_view: f32,
}

/// Value object for visual style
#[derive(Debug, Clone, Component)]
pub struct VisualStyle {
    pub theme: String,
    pub node_appearance: NodeAppearance,
    pub edge_appearance: EdgeAppearance,
}
```

### 7. Event Store (Infrastructure Concern)

```rust
/// Event store for graph domain events
#[derive(Resource)]
pub struct GraphEventStore {
    events: EventStorage,
    projections: ProjectionStorage,
}

impl GraphEventStore {
    pub fn append(&mut self, event: impl GraphDomainEvent) -> EventIdentity {
        // Implementation
    }

    pub fn events_for(&self, graph: GraphIdentity) -> EventStream {
        // Implementation
    }

    pub fn replay_from(&self, snapshot: GraphSnapshot) -> Result<Graph, ReplayError> {
        // Implementation
    }
}
```

## Naming Principles Applied

1. **No Technical Suffixes**:
   - ❌ `GraphNodeRef`, `GraphEdgeRef`
   - ✅ `Node`, `Edge`

2. **Domain Language**:
   - ❌ `dag_version`, `node_index`
   - ✅ `journey`, `identity`

3. **Event Naming**:
   - ❌ `NodeAdded`, `EdgeCreated`
   - ✅ `NodeAddedEvent`, `EdgeConnectedEvent`

4. **Value Objects**:
   - ✅ `GraphIdentity`, `NodeContent`, `SpatialPosition`
   - Clear, descriptive nouns

5. **Repository Pattern**:
   - ✅ `GraphRepository` (not GraphStore or GraphManager)

6. **Domain Services**:
   - ✅ `GraphLayoutDomainService`, `GraphAnalysisDomainService`
   - Clear suffix when in domain layer

## Benefits for Knowledge Graph Extraction

With consistent DDD naming:

1. **Entity Recognition**: Clear distinction between aggregates, entities, and value objects
2. **Relationship Mapping**: Event names reveal domain relationships
3. **Domain Boundaries**: Services and repositories clearly delineate contexts
4. **Semantic Clarity**: All names carry business meaning, not technical details

## Example Knowledge Graph Extraction

```
Graph (Aggregate)
  ├── has → GraphIdentity (ValueObject)
  ├── has → GraphMetadata (ValueObject)
  ├── contains → Node (Entity)
  │     ├── has → NodeIdentity (ValueObject)
  │     ├── has → NodeContent (ValueObject)
  │     └── has → SpatialPosition (ValueObject)
  ├── contains → Edge (Entity)
  │     ├── has → EdgeIdentity (ValueObject)
  │     └── has → EdgeRelationship (ValueObject)
  └── tracked_by → GraphJourney (ValueObject)

GraphRepository (Repository)
  ├── stores → Graph
  └── uses → GraphStorage (internal)

NodeAddedEvent (DomainEvent)
  ├── affects → Graph
  ├── creates → Node
  └── has → EventCorrelation
```

This structure makes it trivial to extract a knowledge graph from the codebase!
