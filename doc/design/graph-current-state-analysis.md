# Graph Current State Analysis

## Overview

This document analyzes the current implementation against our DDD-compliant design to identify gaps and required changes.

## Current Implementation

### What We Have

1. **Basic Graph Components**
   ```rust
   pub struct Graph;  // Marker component
   pub struct GraphId(Uuid);
   pub struct GraphMetadata { name, description, tags }
   pub struct GraphNode { graph_id, position, properties }
   pub struct GraphEdge { graph_id, source, target, properties }
   ```

2. **Basic Events** (with "Event" suffix - violates new rules)
   ```rust
   pub struct GraphCreatedEvent { graph_id, metadata }
   pub struct NodeAddedEvent { graph_id, node_id, position }
   pub struct EdgeCreatedEvent { graph_id, edge_id, source, target }
   ```

3. **Working Features**
   - 3D visualization with Bevy
   - Basic node spawning (3 nodes: Rust, Bevy, ECS)
   - Camera controls
   - Blue sphere rendering

### What's Missing

## Gap Analysis by Context

### 1. Graph Management Context

| Component | Current State | Target State | Gap |
|-----------|--------------|--------------|-----|
| Graph Aggregate | ❌ Marker only | Graph with identity, metadata, journey | Need full aggregate |
| Storage | ❌ ECS components | Graphs (plural storage) | Need Daggy integration |
| Events | ⚠️ Has "Event" suffix | GraphCreated, NodeAdded (no suffix) | Rename all events |
| Services | ❌ Systems only | CreateGraph, AddNodeToGraph | Need service components |

### 2. Visualization Context

| Component | Current State | Target State | Gap |
|-----------|--------------|--------------|-----|
| 3D Rendering | ✅ Working | Keep and enhance | Minor improvements |
| 2D Support | ❌ None | 2D/3D switching | Need 2D camera |
| Edge Rendering | ❌ Not visible | Visible edges | Need edge meshes |
| Layout | ❌ Manual only | ApplyGraphLayout service | Need algorithms |
| Selection | ⚠️ Components exist | TrackNodeSelection service | Need service wrapper |

### 3. Analysis Context

| Component | Current State | Target State | Gap |
|-----------|--------------|--------------|-----|
| Algorithms | ❌ None | AnalyzeGraph, FindGraphPaths | Full implementation |
| Metrics | ❌ None | CalculateGraphMetrics | Full implementation |

### 4. Import/Export Context

| Component | Current State | Target State | Gap |
|-----------|--------------|--------------|-----|
| Serialization | ❌ None | ImportGraphFormats, ExportGraphFormats | Full implementation |
| Formats | ❌ None | JSON, Cypher, Mermaid | Need all formats |

### 5. Event System

| Component | Current State | Target State | Gap |
|-----------|--------------|--------------|-----|
| Event Store | ❌ None | Full event store | Need implementation |
| Event Replay | ❌ None | Replay capability | Need implementation |
| Topics | ❌ None | graphs.created, node.added | Need routing |

## Migration Requirements

### Immediate Changes (Breaking)

1. **Remove "Event" Suffix**
   ```rust
   // Before
   GraphCreatedEvent → GraphCreated
   NodeAddedEvent → NodeAdded
   EdgeCreatedEvent → EdgeConnected
   ```

2. **Rename Components**
   ```rust
   // Storage
   GraphRepository → Graphs

   // Services
   LayoutEngine → ApplyGraphLayout
   GraphAnalyzer → AnalyzeGraph
   ```

### Non-Breaking Additions

1. **Add Daggy Storage**
   - Implement alongside current ECS
   - Gradually migrate data

2. **Add Service Components**
   - Wrap current systems
   - Maintain compatibility

3. **Add Event Store**
   - Record all new events
   - Build history going forward

## Implementation Priority

### Critical Path (Week 1)
1. Fix event naming (breaking change)
2. Implement Graphs storage with Daggy
3. Create service wrappers for existing functionality

### Essential Features (Week 2-3)
1. Event store implementation
2. Edge visualization
3. Basic import/export (JSON)

### Enhanced Features (Week 4+)
1. 2D view support
2. Graph algorithms
3. Animation system

## Code Migration Example

### Current Code
```rust
fn handle_graph_events(
    mut events: EventReader<GraphCreatedEvent>,
    mut commands: Commands,
) {
    for event in events.read() {
        // Handle event
    }
}
```

### Target Code
```rust
// Service component
pub struct CreateGraph {
    graphs: Graphs,
}

impl CreateGraph {
    pub fn execute(&self, metadata: GraphMetadata) -> GraphCreated {
        // Create graph
        let graph = Graph {
            identity: GraphIdentity::new(),
            metadata,
            journey: GraphJourney::new(),
        };

        // Store and return event
        self.graphs.add(graph);
        GraphCreated {
            graph: graph.identity,
            metadata,
            timestamp: SystemTime::now(),
        }
    }
}
```

## Risks and Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking event names | HIGH | Do it once, early |
| Complex migration | MEDIUM | Parallel implementation |
| Learning curve | MEDIUM | Start simple, iterate |

## Conclusion

The current implementation provides a good foundation but requires significant refactoring to comply with DDD principles. The most critical changes are:

1. Remove "Event" suffix from all events
2. Implement proper storage with Daggy
3. Create service components following verb phrase pattern
4. Build event store for persistence

These changes will enable reliable knowledge graph extraction and maintain consistency with our domain model.
