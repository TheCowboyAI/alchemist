# Graph Aggregate Implementation Roadmap

## Overview

Based on our Event Storming session, we've identified 6 bounded contexts. This roadmap shows how to implement them following DDD principles with consistent naming for knowledge graph extraction.

## Implementation Priority

### Phase 1: Core Domain (Weeks 1-2)
**Graph Management Context** - The heart of our system

### Phase 2: Essential Supporting (Weeks 3-4)
**Visualization Context** - User interaction layer
**Import/Export Context** - Data persistence

### Phase 3: Advanced Features (Weeks 5-6)
**Analysis Context** - Graph algorithms
**Animation Context** - Replay capabilities

### Phase 4: Collaboration (Week 7)
**Collaboration Context** - Multi-user support

## Detailed Implementation Plan

### Week 1: Graph Management Context Foundation

#### Day 1-2: Core Domain Model
```rust
// src/contexts/graph_management/domain/mod.rs
pub mod aggregates {
    pub struct Graph {
        pub identity: GraphIdentity,
        pub metadata: GraphMetadata,
        pub journey: GraphJourney,
    }
}

pub mod entities {
    pub struct Node {
        pub identity: NodeIdentity,
        pub graph: GraphIdentity,
        pub content: NodeContent,
        pub position: SpatialPosition,
    }

    pub struct Edge {
        pub identity: EdgeIdentity,
        pub graph: GraphIdentity,
        pub relationship: EdgeRelationship,
    }
}

pub mod value_objects {
    pub struct GraphIdentity(Uuid);
    pub struct NodeIdentity(Uuid);
    pub struct EdgeIdentity(Uuid);
    pub struct NodeContent { /* ... */ }
    pub struct EdgeRelationship { /* ... */ }
    pub struct SpatialPosition { /* ... */ }
}
```

#### Day 3-4: Domain Events
```rust
// src/contexts/graph_management/events/mod.rs
pub trait GraphManagementEvent: Event {
    fn graph_identity(&self) -> GraphIdentity;
    fn correlation(&self) -> EventCorrelation;
}

pub struct GraphCreatedEvent { /* ... */ }
pub struct NodeAddedEvent { /* ... */ }
pub struct EdgeConnectedEvent { /* ... */ }
// ... other events
```

#### Day 5: Repository & Domain Services
```rust
// src/contexts/graph_management/infrastructure/repository.rs
pub struct GraphRepository {
    // Daggy storage implementation
}

// src/contexts/graph_management/domain/services/mod.rs
pub struct GraphValidationDomainService { /* ... */ }
```

### Week 2: Event Store & Integration

#### Day 1-2: Event Store Implementation
```rust
// src/infrastructure/event_store/mod.rs
pub struct EventStore {
    contexts: HashMap<BoundedContext, ContextEventStore>,
}

pub struct ContextEventStore {
    events: Vec<Box<dyn Any>>,
    projections: HashMap<String, Projection>,
}
```

#### Day 3-4: Command Handlers
```rust
// src/contexts/graph_management/application/commands/mod.rs
pub struct CreateGraphCommand {
    pub metadata: GraphMetadata,
}

pub struct CreateGraphCommandHandler {
    repository: GraphRepository,
    event_store: EventStore,
}

impl CommandHandler for CreateGraphCommandHandler {
    type Command = CreateGraphCommand;
    type Event = GraphCreatedEvent;

    fn handle(&self, command: Self::Command) -> Result<Self::Event> {
        // Implementation
    }
}
```

#### Day 5: Integration Tests
```rust
#[test]
fn test_graph_creation_flow() {
    // Given a CreateGraphCommand
    // When handled
    // Then GraphCreatedEvent is stored
    // And Graph aggregate is persisted
}
```

### Week 3: Visualization Context

#### Day 1-2: View Aggregates
```rust
// src/contexts/visualization/domain/mod.rs
pub struct GraphView {
    pub identity: ViewIdentity,
    pub graph: GraphIdentity,
    pub perspective: GraphPerspective,
    pub camera: CameraConfiguration,
    pub selection: SelectionState,
}
```

#### Day 3-4: Bevy Integration
```rust
// src/contexts/visualization/infrastructure/bevy_systems.rs
pub fn sync_graph_to_view(
    graph_events: EventReader<GraphManagementEvent>,
    mut commands: Commands,
) {
    // Convert domain events to ECS entities
}
```

#### Day 5: Layout Domain Service
```rust
// src/contexts/visualization/domain/services/layout.rs
pub struct GraphLayoutDomainService {
    strategies: HashMap<String, Box<dyn LayoutStrategy>>,
}
```

### Week 4: Import/Export Context

#### Day 1-2: Serialization Aggregates
```rust
// src/contexts/import_export/domain/mod.rs
pub struct ImportSession {
    pub identity: SessionIdentity,
    pub format: ImportFormat,
    pub validation_rules: ValidationRules,
}

pub struct ExportSession {
    pub identity: SessionIdentity,
    pub format: ExportFormat,
    pub options: ExportOptions,
}
```

#### Day 3-5: Format Implementations
```rust
// src/contexts/import_export/infrastructure/formats/mod.rs
pub mod json;
pub mod cypher;
pub mod mermaid;

pub trait GraphFormat {
    fn import(&self, content: &str) -> Result<GraphImportedEvent>;
    fn export(&self, graph: &Graph) -> Result<String>;
}
```

### Week 5: Analysis Context

#### Day 1-3: Algorithm Integration
```rust
// src/contexts/analysis/domain/services/algorithms.rs
pub struct GraphAnalysisDomainService {
    pub fn shortest_path(&self, graph: &Graph, from: NodeIdentity, to: NodeIdentity) -> Path;
    pub fn detect_cycles(&self, graph: &Graph) -> Vec<Cycle>;
    pub fn calculate_centrality(&self, graph: &Graph) -> CentralityMetrics;
}
```

#### Day 4-5: Analysis Events
```rust
// src/contexts/analysis/events/mod.rs
pub struct ShortestPathCalculatedEvent {
    pub graph: GraphIdentity,
    pub path: Path,
    pub algorithm: AlgorithmIdentity,
}
```

### Week 6: Animation Context

#### Day 1-3: Timeline Implementation
```rust
// src/contexts/animation/domain/mod.rs
pub struct AnimationSession {
    pub identity: SessionIdentity,
    pub graph: GraphIdentity,
    pub timeline: EventTimeline,
    pub playback: PlaybackState,
}
```

#### Day 4-5: Replay System
```rust
// src/contexts/animation/infrastructure/replay.rs
pub struct EventReplaySystem {
    pub fn replay_from(&self, start: EventIdentity, speed: f32);
    pub fn pause(&self);
    pub fn step(&self);
}
```

### Week 7: Collaboration Context

#### Day 1-3: Session Management
```rust
// src/contexts/collaboration/domain/mod.rs
pub struct GraphCollaboration {
    pub identity: SessionIdentity,
    pub graph: GraphIdentity,
    pub collaborators: Vec<Collaborator>,
    pub change_log: ChangeLog,
}
```

#### Day 4-5: Conflict Resolution
```rust
// src/contexts/collaboration/domain/services/conflict.rs
pub struct ConflictResolutionDomainService {
    pub fn detect_conflicts(&self, changes: &[Change]) -> Vec<Conflict>;
    pub fn resolve(&self, conflict: &Conflict, strategy: ResolutionStrategy) -> Resolution;
}
```

## Context Integration Points

### Published Language (Graph Management → Others)
```rust
// src/contexts/graph_management/public/mod.rs
pub mod language {
    pub use super::domain::{GraphIdentity, NodeIdentity, EdgeIdentity};
    pub use super::events::{GraphCreatedEvent, NodeAddedEvent};
}
```

### Anti-Corruption Layer (Import/Export → External)
```rust
// src/contexts/import_export/acl/mod.rs
pub struct ExternalFormatAdapter {
    pub fn adapt_cypher(&self, query: &str) -> Result<ImportSession>;
    pub fn adapt_to_cypher(&self, graph: &Graph) -> Result<String>;
}
```

## Testing Strategy

### Unit Tests (Per Context)
- Domain model invariants
- Value object behavior
- Event correlation

### Integration Tests (Cross-Context)
- Event flow scenarios
- Context communication
- Data consistency

### Acceptance Tests (End-to-End)
- User story validation
- Performance benchmarks
- Knowledge graph extraction

## Success Metrics

1. **Naming Consistency**: 100% DDD compliance
2. **Event Coverage**: All user actions produce events
3. **Context Isolation**: No direct dependencies between contexts
4. **Knowledge Graph**: Automated extraction works correctly

## Deliverables per Phase

### Phase 1 Deliverables
- [ ] Graph Management context fully implemented
- [ ] Event store operational
- [ ] Basic graph CRUD working

### Phase 2 Deliverables
- [ ] 2D/3D visualization working
- [ ] JSON import/export functional
- [ ] Basic UI connected

### Phase 3 Deliverables
- [ ] Graph algorithms available
- [ ] Event replay working
- [ ] Animation system functional

### Phase 4 Deliverables
- [ ] Multi-user collaboration
- [ ] Conflict resolution
- [ ] Real-time sync

This roadmap ensures we build the system incrementally while maintaining DDD principles and consistent naming throughout!
