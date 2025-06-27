# Graph Abstraction Layer Design

## Overview

The `cim-domain-graph` module needs to become an abstraction layer that unifies all graph implementations in CIM:
- `cim-contextgraph` - Base graph abstraction
- `cim-conceptgraph` - Concept composition graphs
- `cim-workflow-graph` - Workflow-specific graphs
- `cim-ipld-graph` - Content-addressed graphs

## Current State

Currently, `cim-domain-graph` is a standalone domain module implementing DDD patterns for graph management. The other graph modules (`contextgraph`, `conceptgraph`, `workflow-graph`, `ipld-graph`) are independent implementations that all depend on `cim-contextgraph` as their base.

## Target Architecture

```rust
// cim-domain-graph will provide:

/// Trait that all graph implementations must satisfy
pub trait GraphImplementation {
    type Node;
    type Edge;
    type GraphId;
    
    fn new(id: Self::GraphId) -> Self;
    fn add_node(&mut self, node: Self::Node) -> NodeId;
    fn add_edge(&mut self, from: NodeId, to: NodeId, edge: Self::Edge) -> Result<EdgeId>;
    fn get_node(&self, id: NodeId) -> Option<&Self::Node>;
    fn get_edge(&self, id: EdgeId) -> Option<&Self::Edge>;
    // ... other common operations
}

/// Unified graph type that can work with any implementation
pub enum GraphType {
    Context(ContextGraph),
    Concept(ConceptGraph),
    Workflow(WorkflowGraph),
    Ipld(IpldGraph),
}

/// Domain aggregate that manages any graph type
pub struct GraphAggregate {
    id: GraphId,
    graph_type: GraphType,
    metadata: GraphMetadata,
    version: u64,
}
```

## Implementation Steps

### Phase 1: Define Common Interface
1. Create `GraphImplementation` trait in `cim-domain-graph`
2. Define common operations all graphs must support
3. Create `GraphType` enum for runtime polymorphism

### Phase 2: Add Dependencies
1. Add `cim-contextgraph`, `cim-conceptgraph`, `cim-workflow-graph` as dependencies
2. Create adapter implementations for each graph type

### Phase 3: Refactor Domain Logic
1. Update `GraphAggregate` to work with `GraphType`
2. Update commands to specify which graph type to create
3. Update events to include graph type information

### Phase 4: Integration
1. Update handlers to route to appropriate graph implementation
2. Update projections to handle different graph types
3. Add graph type conversion operations

## Benefits

1. **Unified Interface**: Single domain API for all graph operations
2. **Type Safety**: Each graph type maintains its specific invariants
3. **Flexibility**: Easy to add new graph types
4. **Domain Consistency**: All graphs follow same DDD patterns

## Example Usage

```rust
// Create a workflow graph through the domain
let cmd = GraphCommand::Create {
    graph_id: GraphId::new(),
    graph_type: GraphTypeSpec::Workflow {
        workflow_type: WorkflowType::Approval,
    },
    name: "Purchase Approval".to_string(),
};

// Create a concept graph
let cmd = GraphCommand::Create {
    graph_id: GraphId::new(),
    graph_type: GraphTypeSpec::Concept {
        concept_space: "ProductCategories".to_string(),
    },
    name: "Product Taxonomy".to_string(),
};

// Operations work uniformly
let add_node = NodeCommand::Add {
    graph_id,
    node_type: NodeType::WorkflowStep { ... },
    position: Position3D::new(0.0, 0.0, 0.0),
};
``` 