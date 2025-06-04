# Graph Domain Design

## Overview

This document defines the complete domain model for the Information Alchemist Graph system, following strict DDD principles with pure business language.

## Core Domain Model

### Aggregates

#### Graph (Aggregate Root)
The central concept representing a collection of nodes and edges with identity and metadata.

```rust
pub struct Graph {
    pub identity: GraphIdentity,
    pub metadata: GraphMetadata,
    pub journey: GraphJourney,
}
```

#### GraphView
A perspective on a graph with camera position and selection state.

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
Results from analyzing a graph's structure and properties.

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
A vertex in the graph with content and position.

```rust
pub struct Node {
    pub identity: NodeIdentity,
    pub graph: GraphIdentity,
    pub content: NodeContent,
    pub position: SpatialPosition,
}
```

#### Edge
A connection between two nodes with relationship properties.

```rust
pub struct Edge {
    pub identity: EdgeIdentity,
    pub graph: GraphIdentity,
    pub relationship: EdgeRelationship,
}
```

### Value Objects

#### Identities
Unique identifiers for domain objects.

```rust
pub struct GraphIdentity(Uuid);
pub struct NodeIdentity(Uuid);
pub struct EdgeIdentity(Uuid);
pub struct ViewIdentity(Uuid);
pub struct AnalysisIdentity(Uuid);
pub struct EventIdentity(Uuid);
```

#### Graph Metadata
Descriptive information about a graph.

```rust
pub struct GraphMetadata {
    pub name: String,
    pub description: String,
    pub domain: String,
    pub created: SystemTime,
    pub modified: SystemTime,
    pub tags: Vec<String>,
}
```

#### Node Content
The data contained within a node.

```rust
pub struct NodeContent {
    pub label: String,
    pub category: String,
    pub properties: Properties,
}
```

#### Edge Relationship
The connection details between nodes.

```rust
pub struct EdgeRelationship {
    pub source: NodeIdentity,
    pub target: NodeIdentity,
    pub category: String,
    pub strength: f32,
    pub properties: Properties,
}
```

#### Spatial Position
Location in 2D and 3D space.

```rust
pub struct SpatialPosition {
    pub coordinates_3d: Vec3,
    pub coordinates_2d: Vec2,
}
```

#### Graph Journey
The history and version tracking of a graph.

```rust
pub struct GraphJourney {
    pub version: u64,
    pub event_count: u64,
    pub last_event: Option<EventIdentity>,
}
```

## Domain Events

All events are past-tense facts without technical suffixes.

### Graph Lifecycle Events

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

pub struct EdgeDisconnected {
    pub graph: GraphIdentity,
    pub edge: EdgeIdentity,
}

pub struct NodeMoved {
    pub graph: GraphIdentity,
    pub node: NodeIdentity,
    pub from: SpatialPosition,
    pub to: SpatialPosition,
}

pub struct PropertyUpdated {
    pub graph: GraphIdentity,
    pub target: PropertyTarget,
    pub property: String,
    pub old_value: Option<PropertyValue>,
    pub new_value: PropertyValue,
}

pub struct LabelApplied {
    pub graph: GraphIdentity,
    pub target: LabelTarget,
    pub label: String,
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

pub struct SelectionCleared {
    pub view: ViewIdentity,
}

pub struct LayoutCalculated {
    pub graph: GraphIdentity,
    pub layout_type: LayoutType,
    pub positions: HashMap<NodeIdentity, SpatialPosition>,
}
```

### Analysis Events

```rust
pub struct PathFound {
    pub analysis: AnalysisIdentity,
    pub source: NodeIdentity,
    pub target: NodeIdentity,
    pub path: Vec<NodeIdentity>,
}

pub struct MetricsCalculated {
    pub analysis: AnalysisIdentity,
    pub graph: GraphIdentity,
    pub metrics: GraphMetrics,
}

pub struct PatternDetected {
    pub analysis: AnalysisIdentity,
    pub pattern_type: PatternType,
    pub nodes: Vec<NodeIdentity>,
}
```

## Domain Services

All services use verb phrases that reveal their intent.

### Graph Management Services

```rust
/// Creates new graphs with metadata
pub struct CreateGraph;

/// Adds nodes to existing graphs
pub struct AddNodeToGraph;

/// Connects nodes with edges
pub struct ConnectGraphNodes;

/// Validates graph structure and constraints
pub struct ValidateGraph;

/// Removes nodes from graphs
pub struct RemoveNodeFromGraph;

/// Disconnects edges between nodes
pub struct DisconnectGraphNodes;

/// Updates properties on graph elements
pub struct UpdateGraphProperties;
```

### Visualization Services

```rust
/// Calculates optimal node positions
pub struct ApplyGraphLayout;

/// Tracks user selections in views
pub struct TrackNodeSelection;

/// Renders graphs to the screen
pub struct RenderGraphView;

/// Switches between 2D and 3D views
pub struct ToggleViewPerspective;

/// Controls camera movement
pub struct ControlGraphCamera;

/// Animates graph changes
pub struct AnimateGraphElements;
```

### Analysis Services

```rust
/// Analyzes graph structure and properties
pub struct AnalyzeGraph;

/// Finds paths between nodes
pub struct FindGraphPaths;

/// Calculates graph metrics
pub struct CalculateGraphMetrics;

/// Detects patterns in graph structure
pub struct DetectGraphPatterns;

/// Identifies graph communities
pub struct FindGraphCommunities;
```

### Import/Export Services

```rust
/// Imports graphs from various formats
pub struct ImportGraphData;

/// Exports graphs to various formats
pub struct ExportGraphData;

/// Validates imported data
pub struct ValidateGraphData;

/// Transforms between formats
pub struct TransformGraphFormat;
```

## Storage Components

Storage uses plural domain terms.

```rust
/// Storage for graph aggregates
pub struct Graphs;

/// Storage for graph events
pub struct GraphEvents;

/// Index for fast node lookups
pub struct Nodes;

/// Index for edge traversal
pub struct Edges;

/// Storage for analysis results
pub struct Analyses;

/// Storage for view states
pub struct Views;
```

## Event Topics

Following domain-driven topic naming.

### Collection Topics (plural)
- `graphs.created`
- `nodes.added`
- `edges.connected`
- `nodes.removed`
- `edges.disconnected`

### Entity Topics (singular)
- `graph.deleted`
- `node.selected`
- `node.moved`
- `view.changed`
- `layout.calculated`

## Bounded Contexts

### 1. Graph Management (Core Domain)
**Purpose**: Manage graph structure and lifecycle
**Language**: Graph, Node, Edge, Connection, Relationship
**Services**: CreateGraph, AddNodeToGraph, ConnectGraphNodes, ValidateGraph
**Storage**: Graphs, GraphEvents, Nodes, Edges

### 2. Visualization (Supporting Domain)
**Purpose**: Display and interact with graphs
**Language**: View, Perspective, Camera, Selection, Layout
**Services**: RenderGraphView, ApplyGraphLayout, TrackNodeSelection
**Storage**: Views

### 3. Analysis (Supporting Domain)
**Purpose**: Analyze graph properties and patterns
**Language**: Path, Metric, Pattern, Community, Centrality
**Services**: AnalyzeGraph, FindGraphPaths, CalculateGraphMetrics
**Storage**: Analyses

### 4. Import/Export (Supporting Domain)
**Purpose**: Transform graphs between formats
**Language**: Format, Schema, Transformation, Validation
**Services**: ImportGraphData, ExportGraphData, ValidateGraphData

### 5. Animation (Supporting Domain)
**Purpose**: Animate graph changes over time
**Language**: Motion, Transition, Timeline, Replay
**Services**: AnimateGraphElements, ReplayGraphChanges

### 6. Collaboration (Generic Subdomain)
**Purpose**: Enable multi-user graph editing
**Language**: Session, Participant, Conflict, Synchronization
**Services**: ShareGraphSession, ResolveEditConflicts

## Context Integration

### Event Flow Between Contexts

```
Graph Management → (GraphCreated) → Visualization
                                 → Analysis
                                 → Animation

Visualization → (NodeSelected) → Graph Management
                             → Analysis

Analysis → (PatternDetected) → Visualization
                           → Graph Management
```

### Integration Patterns

1. **Event-Driven**: Contexts communicate through domain events
2. **Eventual Consistency**: Each context maintains its own view
3. **No Shared State**: Contexts are completely autonomous
4. **Clear Boundaries**: No direct dependencies between contexts

## Implementation Guidelines

### Domain Model Implementation
- Use strong typing for all identities
- Implement value objects as immutable
- Validate invariants in constructors
- Keep aggregates small and focused

### Service Implementation
- Services should be stateless when possible
- Use dependency injection for storage
- Return events or results, not void
- Handle errors with domain-specific types

### Event Implementation
- Events must be immutable
- Include all necessary data for replay
- Use primitive types or value objects
- Design for schema evolution

### Storage Implementation
- Abstract storage behind domain interfaces
- Use appropriate data structures (Daggy for graphs)
- Implement proper indexing for performance
- Support event sourcing patterns

## Key Principles

1. **Pure Domain Language**: No technical terms or patterns
2. **Verb Phrases for Actions**: Services that do things
3. **Plural for Collections**: Storage of multiple items
4. **Past-Tense for Facts**: Events that happened
5. **Clear Intent**: Names reveal business purpose

This design enables reliable domain model extraction and maintains consistency with business language throughout the system.
