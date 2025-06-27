# ECS Refactoring Plan for Remaining Domains

## Status Overview

### ✅ Completed Domains (3/14)
1. **cim-domain-identity** - 100% Complete
2. **cim-domain-policy** - 100% Complete
3. **cim-domain-bevy** - Already ECS (presentation layer)

### 🔄 In Progress (1/14)
4. **cim-domain-person** - Has ECS structure but needs verification

### ❌ Not Started (10/14)
5. **cim-domain-graph** - Has abstraction layer, needs ECS refactoring
6. **cim-domain-organization** - Not started
7. **cim-domain-location** - Not started
8. **cim-domain-dialog** - Not started
9. **cim-domain-document** - Not started
10. **cim-domain-agent** - Not started
11. **cim-domain-conceptualspaces** - Not started
12. **cim-domain-workflow** - Not started
13. **cim-domain-nix** - Not started
14. **cim-domain-git** - Not started

## Priority Order

Based on domain dependencies and CIM architecture:

### Phase 1: Core Entity Domains (Foundation)
1. **cim-domain-graph** - Central to CIM vision
2. **cim-domain-organization** - Pairs with identity/person
3. **cim-domain-location** - Supporting domain for geo concepts

### Phase 2: Workflow & Reasoning Domains
4. **cim-domain-workflow** - Core to graph-based workflows
5. **cim-domain-conceptualspaces** - AI reasoning capabilities
6. **cim-domain-agent** - AI agent foundation

### Phase 3: Content & Integration Domains
7. **cim-domain-document** - Content management
8. **cim-domain-dialog** - Conversation management
9. **cim-domain-git** - Version control integration
10. **cim-domain-nix** - Build system integration

## ECS Refactoring Pattern

Each domain should follow the Identity domain pattern:

### 1. Component Structure
```rust
// Core entity component
#[derive(Component)]
pub struct GraphEntity {
    pub graph_id: GraphId,
    pub graph_type: GraphType,
}

// Data components
#[derive(Component)]
pub struct GraphMetadata {
    pub name: String,
    pub description: String,
}

// Relationship components
#[derive(Component)]
pub struct GraphNode {
    pub node_id: NodeId,
    pub graph_id: GraphId,
}
```

### 2. System Implementation
```rust
// Lifecycle systems
pub fn create_graph_system(
    mut commands: Commands,
    mut events: EventReader<GraphCreated>,
) {
    for event in events.read() {
        commands.spawn((
            GraphEntity {
                graph_id: event.graph_id,
                graph_type: event.graph_type,
            },
            GraphMetadata {
                name: event.name.clone(),
                description: event.description.clone(),
            },
        ));
    }
}
```

### 3. Event-Driven Architecture
- Commands trigger systems
- Systems validate through aggregates
- Systems emit domain events
- Other systems react to events

### 4. Query Operations
```rust
pub fn find_graph_by_id(world: &mut World, graph_id: GraphId) -> Option<GraphView> {
    let mut query = world.query::<(&GraphEntity, &GraphMetadata)>();
    query.iter(world)
        .find(|(entity, _)| entity.graph_id == graph_id)
        .map(|(entity, metadata)| GraphView {
            graph_id: entity.graph_id,
            name: metadata.name.clone(),
            // ...
        })
}
```

## Next Steps

1. **Start with cim-domain-graph**
   - Already has abstraction layer
   - Central to CIM architecture
   - Will benefit from ECS performance

2. **Create ECS components for graphs**
   - GraphEntity, NodeEntity, EdgeEntity
   - Visual components (Position, Color, etc.)
   - Workflow components (Status, Transitions)

3. **Implement systems**
   - Graph lifecycle (create, update, delete)
   - Node/Edge management
   - Layout algorithms
   - Query operations

4. **Integration tests**
   - Test graph operations
   - Test cross-domain events
   - Performance benchmarks

## Success Criteria

Each domain is complete when:
1. All components defined with proper derives
2. All systems implemented and tested
3. Event flow working end-to-end
4. Integration tests passing
5. Documentation updated
6. No compilation warnings
7. Performance benchmarks show improvement 