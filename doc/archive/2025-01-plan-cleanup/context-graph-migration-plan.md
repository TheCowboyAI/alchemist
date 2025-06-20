# ContextGraph Migration Plan

## Overview

This plan outlines the migration from ContentGraph/GraphComposition to ContextGraph architecture where every graph has a designated ContextRoot entity that serves as the semantic anchor.

## Phase 1: Core Domain Model Updates

### 1.1 Rename GraphComposition to ContextGraph
- [ ] Rename `GraphComposition` struct to `ContextGraph`
- [ ] Update `composition_root` field to `context_root`
- [ ] Update `CompositionType` to `ContextType`
- [ ] Update all related types (CompositionNode → ContextNode, etc.)

### 1.2 Update Domain Events
- [ ] Rename composition-related events to context-related events
- [ ] Add context_root validation to all graph creation events
- [ ] Update event handlers to respect context boundaries

### 1.3 Update Commands
- [ ] Update graph creation commands to require context_root
- [ ] Add validation for context root operations
- [ ] Update command handlers

## Phase 2: Demo Updates

### 2.1 Infrastructure Demos (Working)
1. **demo_nats_connection.rs** - No changes needed (infrastructure only)
2. **demo_event_persistence.rs** - No changes needed (infrastructure only)

### 2.2 Domain Model Demos (Need Updates)
1. **demo_graph_create.rs**
   - Update to create ContextGraph with explicit context_root
   - Show BoundedContext, Aggregate, Module, and Service context types
   - Demonstrate context boundary enforcement

2. **demo_node_operations.rs**
   - Update to show operations flowing through context_root
   - Demonstrate invariant enforcement via root
   - Show context-aware node operations

3. **demo_conceptual_space_create.rs**
   - Update to show conceptual space as a ContextGraph
   - Root entity defines the semantic anchor
   - Show how concepts relate to the root

4. **demo_cim_rules_violations.rs**
   - Update examples to use ContextGraph
   - Show proper context boundaries
   - Demonstrate anti-patterns with contexts

### 2.3 Broken Demos (Need Fixing + Updates)
1. **demo_event_bridge.rs**
   - Fix compilation errors
   - Update to use ContextGraph events
   - Show context-aware event routing

2. **demo_async_sync_bridge.rs**
   - Fix compilation errors
   - Update to bridge context events
   - Show context preservation across async boundary

### 2.4 Visual Demos
1. **workflow_designer_demo.rs**
   - Update to design workflows as ContextGraphs
   - Show workflow root as context anchor

2. **conceptual_graph_visual_demo.rs**
   - Update to visualize ContextGraphs
   - Highlight context roots visually
   - Show context boundaries

3. **import_graph.rs**
   - Update to import as ContextGraphs
   - Identify and set context roots during import

## Phase 3: Implementation Details

### 3.1 ContextGraph Structure
```rust
pub struct ContextGraph {
    pub id: GraphId,
    pub context_root: NodeId,  // The semantic anchor
    pub context_type: ContextType,
    pub nodes: HashMap<NodeId, ContextNode>,
    pub edges: HashMap<EdgeId, ContextEdge>,
    pub metadata: ContextMetadata,
}

pub enum ContextType {
    /// Bounded Context (top-level)
    BoundedContext { domain: String },

    /// Aggregate Context (consistency boundary)
    Aggregate { aggregate_type: String },

    /// Module Context (functional grouping)
    Module { module_name: String },

    /// Service Context (operational boundary)
    Service { service_type: String },
}
```

### 3.2 Context Root Validation
- Every ContextGraph MUST have a valid context_root
- The root node must exist in the nodes collection
- Operations should flow through or relate to the root
- Root defines the semantic meaning of the context

### 3.3 Demo Patterns

#### Pattern 1: Creating a Bounded Context
```rust
let user_context = ContextGraph::new_bounded_context(
    "UserManagement",
    "User Aggregate Root"
);
```

#### Pattern 2: Adding Entities to Context
```rust
// All entities relate to the context root
context.add_entity_related_to_root(
    "User Profile",
    RelationshipType::Contains
);
```

#### Pattern 3: Context Composition
```rust
// Contexts can contain other contexts
let auth_module = ContextGraph::new_module(
    "Authentication",
    "Auth Service Root"
);
user_context.add_nested_context(auth_module);
```

## Phase 4: Testing Strategy

1. **Unit Tests**
   - Test context root validation
   - Test context boundary enforcement
   - Test recursive context nesting

2. **Integration Tests**
   - Test context event flow
   - Test cross-context communication
   - Test context persistence

3. **Demo Validation**
   - Each demo must compile and run
   - Each demo must demonstrate context principles
   - Each demo must have clear output showing context behavior

## Phase 5: Documentation Updates

1. Update all documentation to use ContextGraph terminology
2. Create examples showing context patterns
3. Document migration guide for existing code
4. Update architecture diagrams

## Success Criteria

- [ ] All demos compile and run successfully
- [ ] All demos demonstrate ContextGraph principles
- [ ] Context roots are clearly identified in all graphs
- [ ] Context boundaries are enforced
- [ ] Documentation reflects new architecture
- [ ] Tests pass with >80% coverage

## Timeline

- Phase 1: 2 days (Core domain model)
- Phase 2: 3 days (Demo updates)
- Phase 3: 2 days (Implementation refinement)
- Phase 4: 2 days (Testing)
- Phase 5: 1 day (Documentation)

Total: ~10 days
