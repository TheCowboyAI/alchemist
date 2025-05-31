# Graph Aggregate Gap Analysis

## Current Implementation vs Requirements

| Feature | Current State | Required State | Priority |
|---------|--------------|----------------|----------|
| **Core Architecture** | | | |
| Graph Storage | ❌ Simple marker component | ✅ Daggy-based repository | HIGH |
| Node/Edge Storage | ⚠️ Direct in components | ✅ Daggy with ECS refs | HIGH |
| Event Correlation | ❌ Basic events only | ✅ Full correlation system | HIGH |
| Aggregate Versioning | ❌ Not implemented | ✅ Version tracking | MEDIUM |
| | | | |
| **Persistence** | | | |
| JSON Import/Export | ❌ Not implemented | ✅ Full serialization | HIGH |
| Cypher Support | ❌ Not implemented | ✅ Query generation | MEDIUM |
| Mermaid Support | ❌ Not implemented | ✅ Diagram export | LOW |
| Event Store | ❌ Not implemented | ✅ Persistent store | HIGH |
| | | | |
| **Visualization** | | | |
| 3D Rendering | ✅ Basic spheres | ✅ Keep and enhance | DONE |
| 2D Rendering | ❌ Not implemented | ✅ 2D projection | HIGH |
| View Switching | ❌ Not implemented | ✅ Runtime toggle | MEDIUM |
| Edge Rendering | ❌ Not visible | ✅ Lines/curves | HIGH |
| | | | |
| **Interaction** | | | |
| Event-Driven Updates | ⚠️ Basic implementation | ✅ Full CQRS pattern | HIGH |
| Selection System | ❌ Components only | ✅ Working selection | MEDIUM |
| Drag & Drop | ❌ Components only | ✅ Working drag | MEDIUM |
| Property Editing | ❌ Not implemented | ✅ UI panel | LOW |
| | | | |
| **Animation** | | | |
| Event Replay | ❌ Not implemented | ✅ Timeline control | MEDIUM |
| Graph Construction | ❌ Not implemented | ✅ Animated build | MEDIUM |
| Graph Theory Ops | ❌ Not implemented | ✅ Algorithm viz | LOW |
| Transitions | ❌ Not implemented | ✅ Smooth morphing | LOW |

## Critical Missing Pieces

### 1. Daggy Integration (HIGHEST PRIORITY)
Our current implementation stores everything directly in ECS components. We need:
- Separate `GraphRepository` resource containing Daggy graphs
- Components that reference Daggy indices, not store data
- Sync system to update ECS from Daggy changes

### 2. Event System Enhancement (HIGH PRIORITY)
Current events are too simple. We need:
- Event correlation (correlation_id, causation_id)
- Event versioning and sequencing
- Event store for persistence
- Event replay capability

### 3. Serialization (HIGH PRIORITY)
No current support. We need:
- Trait-based serialization system
- JSON format (complete graph state)
- Cypher format (Neo4j compatible)
- Mermaid format (documentation)

### 4. 2D/3D View System (MEDIUM PRIORITY)
Currently 3D only. We need:
- Dual camera system
- View state management
- Coordinate transformation
- Layout algorithms for 2D

## Implementation Order

### Sprint 3-4 (Current)
1. **Daggy Integration**
   - Create `GraphRepository` resource
   - Refactor components to use references
   - Implement sync system

2. **Enhanced Events**
   - Add correlation to events
   - Create event store
   - Basic replay system

### Sprint 5-6
3. **Basic Serialization**
   - JSON import/export
   - Graph validation

4. **2D View**
   - Add 2D camera
   - View switching
   - Basic 2D layout

### Sprint 7-8
5. **Advanced Features**
   - Cypher support
   - Animation system
   - Graph algorithms

## Code Changes Required

### 1. Refactor Current Components
```rust
// FROM: Storing data directly
pub struct GraphNode {
    pub graph_id: GraphId,
    pub position: Vec3,
    pub properties: HashMap<String, serde_json::Value>,
}

// TO: Referencing Daggy
pub struct GraphNodeRef {
    pub graph_id: GraphId,
    pub node_index: daggy::NodeIndex,
    pub version: u64,
}
```

### 2. Add Repository Pattern
```rust
// New resource for graph storage
#[derive(Resource)]
pub struct GraphRepository {
    graphs: HashMap<GraphId, daggy::Dag<NodeData, EdgeData>>,
    event_store: EventStore,
}
```

### 3. Enhance Event System
```rust
// Wrap all events with correlation
pub struct EventEnvelope<E: GraphEvent> {
    pub event: E,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub sequence: u64,
}
```

## Risks and Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking changes to current code | HIGH | Implement alongside, migrate gradually |
| Daggy learning curve | MEDIUM | Study examples, prototype first |
| Performance with large graphs | MEDIUM | Implement LOD, streaming |
| Event store size | LOW | Implement snapshots, pruning |

## Conclusion

Our current implementation provides a good foundation but needs significant enhancement to meet all requirements. The most critical gap is the lack of proper graph storage (Daggy) and event correlation. These should be addressed immediately in Sprint 3-4.
