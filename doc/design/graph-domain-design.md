# Graph Domain Design

## Overview

This document defines the complete design for the Information Alchemist Graph system following strict DDD principles with pure domain language.

## Domain Model

### Core Aggregates

#### Graph (Aggregate Root)
```rust
pub struct Graph {
    pub identity: GraphIdentity,
    pub metadata: GraphMetadata,
    pub journey: GraphJourney,
}
```

#### GraphView
```rust
pub struct GraphView {
    pub identity: ViewIdentity,
    pub graph: GraphIdentity,
    pub perspective: GraphPerspective,
    pub camera: CameraConfiguration,
    pub selection: SelectionState,
}
```

#### GraphAnalysis
```rust
pub struct GraphAnalysis {
    pub identity: AnalysisIdentity,
    pub graph: GraphIdentity,
    pub snapshot: GraphSnapshot,
    pub results: AnalysisResults,
}
```

### Entities

#### Node
```rust
pub struct Node {
    pub identity: NodeIdentity,
    pub graph: GraphIdentity,
    pub content: NodeContent,
    pub position: SpatialPosition,
}
```

#### Edge
```rust
pub struct Edge {
    pub identity: EdgeIdentity,
    pub graph: GraphIdentity,
    pub relationship: EdgeRelationship,
}
```

### Value Objects

```rust
// Identities
pub struct GraphIdentity(Uuid);
pub struct NodeIdentity(Uuid);
pub struct EdgeIdentity(Uuid);
pub struct ViewIdentity(Uuid);

// Metadata
pub struct GraphMetadata {
    pub name: String,
    pub description: String,
    pub domain: String,
    pub created: SystemTime,
    pub modified: SystemTime,
    pub tags: Vec<String>,
}

// Content
pub struct NodeContent {
    pub label: String,
    pub category: String,
    pub properties: Properties,
}

pub struct EdgeRelationship {
    pub source: NodeIdentity,
    pub target: NodeIdentity,
    pub category: String,
    pub strength: f32,
    pub properties: Properties,
}

// Positioning
pub struct SpatialPosition {
    pub coordinates_3d: Vec3,
    pub coordinates_2d: Vec2,
}

// Journey tracking
pub struct GraphJourney {
    pub version: u64,
    pub event_count: u64,
    pub last_event: Option<EventIdentity>,
}
```

## Domain Events

Following Rule #6: No "Event" suffix - events are past-tense facts.

### Graph Management Events
```rust
pub struct GraphCreated {
    pub graph: GraphIdentity,
    pub metadata: GraphMetadata,
    pub timestamp: SystemTime,
}

pub struct NodeAdded {
    pub graph: GraphIdentity,
    pub node: NodeIdentity,
    pub content: NodeContent,
    pub position: SpatialPosition,
}

pub struct EdgeConnected {
    pub graph: GraphIdentity,
    pub edge: EdgeIdentity,
    pub relationship: EdgeRelationship,
}

pub struct NodeRemoved {
    pub graph: GraphIdentity,
    pub node: NodeIdentity,
}

pub struct GraphDeleted {
    pub graph: GraphIdentity,
    pub reason: DeletionReason,
}
```

### Visualization Events
```rust
pub struct GraphViewChanged {
    pub view: ViewIdentity,
    pub from_mode: ViewMode,
    pub to_mode: ViewMode,
}

pub struct NodeSelected {
    pub view: ViewIdentity,
    pub node: NodeIdentity,
}

pub struct LayoutCalculated {
    pub graph: GraphIdentity,
    pub layout_type: LayoutType,
    pub positions: HashMap<NodeIdentity, SpatialPosition>,
}
```

## Domain Components

Following Rule #3: ServiceContext pattern (verb phrases).

### Graph Management Context

```rust
// Storage (Rule #4: plural domain context)
pub struct Graphs {
    storage: HashMap<GraphIdentity, Graph>,
    index: GraphIndex,
}

// Services (verb phrases revealing intent)
pub struct CreateGraph {
    graphs: Graphs,
}

pub struct AddNodeToGraph {
    graphs: Graphs,
}

pub struct ConnectGraphNodes {
    graphs: Graphs,
}

pub struct ValidateGraph {
    rules: Vec<GraphRule>,
}
```

### Visualization Context

```rust
pub struct ApplyGraphLayout {
    algorithms: HashMap<LayoutType, LayoutAlgorithm>,
}

pub struct TrackNodeSelection {
    selections: HashMap<ViewIdentity, SelectionState>,
}

pub struct RenderGraphView {
    renderers: HashMap<ViewMode, ViewRenderer>,
}
```

### Analysis Context

```rust
pub struct AnalyzeGraph {
    algorithms: AlgorithmLibrary,
}

pub struct FindGraphPaths {
    pathfinding: PathfindingAlgorithms,
}

pub struct CalculateGraphMetrics {
    calculators: MetricCalculators,
}
```

### Import/Export Context

```rust
pub struct ImportGraphFormats {
    importers: HashMap<FormatType, GraphImporter>,
}

pub struct ExportGraphFormats {
    exporters: HashMap<FormatType, GraphExporter>,
}

pub struct ValidateImportFormat {
    validators: HashMap<FormatType, FormatValidator>,
}
```

## Event Topics

Following Rule #7: Event topic naming.

### Collection Events (plural)
- `graphs.created`
- `nodes.added`
- `edges.connected`

### Single Entity Events (singular)
- `graph.deleted`
- `node.selected`
- `view.changed`

## Bounded Contexts

### 1. Graph Management (Core Domain)
- **Purpose**: Manage graph lifecycle and structure
- **Language**: Graph, Node, Edge, Connection
- **Components**: Graphs, CreateGraph, ValidateGraph

### 2. Visualization (Supporting Domain)
- **Purpose**: Display and interact with graphs
- **Language**: View, Layout, Selection, Camera
- **Components**: ApplyGraphLayout, RenderGraphView

### 3. Analysis (Supporting Domain)
- **Purpose**: Analyze graph properties and patterns
- **Language**: Path, Metric, Algorithm, Pattern
- **Components**: AnalyzeGraph, FindGraphPaths

### 4. Import/Export (Supporting Domain)
- **Purpose**: Serialize graphs to various formats
- **Language**: Format, Schema, Validation
- **Components**: ImportGraphFormats, ExportGraphFormats

### 5. Collaboration (Generic Subdomain)
- **Purpose**: Enable multi-user graph editing
- **Language**: Session, Collaborator, Conflict
- **Components**: CoordinateGraphSharing, ResolveConflicts

### 6. Animation (Supporting Domain)
- **Purpose**: Animate graph changes over time
- **Language**: Timeline, Replay, Transition
- **Components**: ReplayGraphChanges, AnimateTransitions

## Implementation Architecture

### Storage Layer (Daggy Integration)

```rust
pub struct GraphStorage {
    graphs: HashMap<GraphIdentity, daggy::Dag<NodeData, EdgeData>>,
    node_indices: HashMap<(GraphIdentity, NodeIdentity), daggy::NodeIndex>,
    edge_indices: HashMap<(GraphIdentity, EdgeIdentity), daggy::EdgeIndex>,
}
```

### Event Store

```rust
pub struct EventStore {
    events: HashMap<GraphIdentity, Vec<DomainEvent>>,
    projections: HashMap<ProjectionType, Projection>,
}
```

### ECS Integration (Bevy)

```rust
// Reference components for visualization
#[derive(Component)]
struct NodeReference {
    graph: GraphIdentity,
    node: NodeIdentity,
    dag_index: daggy::NodeIndex,
}

#[derive(Component)]
struct EdgeReference {
    graph: GraphIdentity,
    edge: EdgeIdentity,
    dag_index: daggy::EdgeIndex,
}
```

## Example Usage

```rust
// Creating a graph
let graphs = Graphs::new();
let create_graph = CreateGraph::new(graphs);
let graph_created = create_graph.execute(GraphMetadata {
    name: "Knowledge Graph".into(),
    domain: "research".into(),
    ..default()
});

// Adding nodes
let add_node = AddNodeToGraph::new(graphs);
let node_added = add_node.execute(
    graph_created.graph,
    NodeContent {
        label: "Rust".into(),
        category: "Technology".into(),
        properties: Properties::new(),
    },
    SpatialPosition::at(0.0, 0.0, 0.0),
);

// Applying layout
let layout_engine = ApplyGraphLayout::new();
let layout_calculated = layout_engine.execute(
    graph_created.graph,
    LayoutType::ForceDirected,
);
```

## Key Principles

1. **Pure Domain Language**: No technical suffixes or patterns
2. **Verb Phrases for Services**: Components that do things use ServiceContext pattern
3. **Plural for Storage**: Storage components use plural domain terms
4. **Event as Facts**: Events are named as past-tense facts without "Event" suffix
5. **Clear Intent**: All names reveal their purpose in domain terms

This design ensures consistency with DDD principles and enables reliable knowledge graph extraction from the codebase.
