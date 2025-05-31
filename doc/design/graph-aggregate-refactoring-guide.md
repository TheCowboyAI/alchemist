# Graph Aggregate Refactoring Guide

## Overview

This guide shows how to refactor our current implementation to be DDD-compliant, enabling consistent knowledge graph extraction from component names and relationships.

## Key DDD Violations in Current Design

1. **Technical Suffixes**: `GraphNodeRef`, `GraphEdgeRef`
2. **Missing Domain Language**: `node_index`, `dag_version`
3. **Incomplete Event Names**: Should always end with `Event`
4. **No Value Objects**: Everything is a component or struct
5. **Mixed Concerns**: Technical details mixed with domain concepts

## Refactoring Map

### Components → Domain Model

| Current | DDD-Compliant | Type | Rationale |
|---------|---------------|------|-----------|
| `Graph` (marker) | `Graph` (aggregate) | Aggregate | Full aggregate with identity and metadata |
| `GraphId(Uuid)` | `GraphIdentity(Uuid)` | Value Object | Clear that it's an identity, not just ID |
| `GraphMetadata` | `GraphMetadata` | Value Object | Already good, keep as-is |
| `GraphNode` | `Node` | Entity | Remove redundant "Graph" prefix |
| `NodeId(Uuid)` | `NodeIdentity(Uuid)` | Value Object | Identity, not just ID |
| `GraphEdge` | `Edge` | Entity | Remove redundant prefix |
| `EdgeId(Uuid)` | `EdgeIdentity(Uuid)` | Value Object | Identity pattern |
| `GraphNodeRef` | *(Internal only)* | - | Technical detail, hide from domain |
| `ElementState` | `InteractionState` | Value Object | Domain-focused name |

### New Value Objects Needed

```rust
// Current: Properties stored as raw HashMap
pub properties: HashMap<String, serde_json::Value>

// DDD: Domain-specific value objects
pub content: NodeContent {
    label: String,
    category: String,
    properties: Properties,
}

// Current: Position as raw Vec3
pub position: Vec3

// DDD: Spatial value object
pub position: SpatialPosition {
    coordinates_3d: Vec3,
    coordinates_2d: Vec2,
}
```

### Events Refactoring

| Current Event | DDD-Compliant Event | Changes |
|---------------|---------------------|---------|
| `GraphCreatedEvent` | `GraphCreatedEvent` | ✅ Already correct |
| `NodeAddedEvent` | `NodeAddedEvent` | ✅ Already correct |
| `EdgeCreatedEvent` | `EdgeConnectedEvent` | More domain-specific verb |
| `NodeUpdatedEvent` | `NodeModifiedEvent` | Clearer intent |
| `ElementSelectedEvent` | `NodeSelectedEvent` | Specific, not generic |
| `DragStartedEvent` | `NodeDragStartedEvent` | Domain-specific |

### Event Correlation Enhancement

```rust
// Current: Simple events
pub struct NodeAddedEvent {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub position: Vec3,
    pub properties: HashMap<String, serde_json::Value>,
}

// DDD: Rich domain events
pub struct NodeAddedEvent {
    pub graph: GraphIdentity,      // Clear aggregate reference
    pub node: NodeIdentity,        // Clear entity identity
    pub content: NodeContent,      // Domain value object
    pub position: SpatialPosition, // Domain value object
    pub correlation: EventCorrelation, // Event metadata
}
```

### Repository Pattern

```rust
// Current: Missing repository
// (Graph data scattered in components)

// DDD: Proper repository
pub struct GraphRepository {
    pub fn store(&mut self, graph: Graph) -> Result<(), StorageError>
    pub fn find(&self, identity: GraphIdentity) -> Option<&Graph>
    pub fn find_by_domain(&self, domain: &str) -> Vec<&Graph>
    pub fn exists(&self, identity: GraphIdentity) -> bool
}
```

### Domain Services

```rust
// Current: Systems doing everything
fn handle_graph_events(/* many parameters */) { }

// DDD: Domain services with clear responsibilities
pub struct GraphLayoutDomainService {
    pub fn apply_force_directed_layout(&self, graph: &Graph) -> LayoutResult
    pub fn apply_hierarchical_layout(&self, graph: &Graph) -> LayoutResult
}

pub struct GraphAnalysisDomainService {
    pub fn find_shortest_path(&self, graph: &Graph, from: NodeIdentity, to: NodeIdentity) -> Path
    pub fn detect_cycles(&self, graph: &Graph) -> Vec<Cycle>
    pub fn calculate_centrality(&self, graph: &Graph) -> CentralityMetrics
}
```

## Implementation Strategy

### Phase 1: Add Value Objects (Don't Break Existing)
```rust
// Keep old components, add new value objects
pub struct NodeIdentity(pub Uuid);
pub struct GraphIdentity(pub Uuid);
pub struct SpatialPosition { /* ... */ }
pub struct NodeContent { /* ... */ }
```

### Phase 2: Create Parallel Domain Model
```rust
// New domain model alongside old components
mod domain {
    pub struct Graph { /* DDD version */ }
    pub struct Node { /* DDD version */ }
    pub struct Edge { /* DDD version */ }
}
```

### Phase 3: Adapter Layer
```rust
// Adapters to convert between old and new
impl From<GraphNode> for domain::Node { /* ... */ }
impl From<domain::Node> for GraphNode { /* ... */ }
```

### Phase 4: Gradual Migration
1. Update one system at a time to use domain model
2. Keep tests passing throughout
3. Remove old components once fully migrated

## Benefits for Knowledge Graph Extraction

### Before (Technical Focus)
```
GraphNodeRef
  - graph_id: GraphId
  - node_index: NodeIndex
  - version: u64
```

### After (Domain Focus)
```
Node (Entity)
  - identity: NodeIdentity
  - graph: GraphIdentity
  - content: NodeContent
  - position: SpatialPosition
```

The DDD version immediately reveals:
- `Node` is an entity (has identity)
- It belongs to a `Graph` aggregate
- It has `content` (what it contains)
- It has `position` (where it is)

This makes automated knowledge graph extraction much more reliable!

## Validation Checklist

- [ ] All aggregates are singular nouns
- [ ] All entities are singular nouns
- [ ] All value objects are descriptive nouns/phrases
- [ ] All events end with `Event`
- [ ] All repositories end with `Repository`
- [ ] All domain services end with `DomainService`
- [ ] No technical suffixes (Ref, Impl, Helper, Manager)
- [ ] All names from domain vocabulary
- [ ] Clear distinction between domain and infrastructure
