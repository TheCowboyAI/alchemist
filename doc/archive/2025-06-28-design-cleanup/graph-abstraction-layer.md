# Graph Abstraction Layer Design

## Overview

The `cim-domain-graph` module provides an abstraction layer that unifies all graph implementations in CIM:
- `cim-contextgraph` - Base graph abstraction
- `cim-conceptgraph` - Concept composition graphs
- `cim-workflow-graph` - Workflow-specific graphs
- `cim-ipld-graph` - Content-addressed graphs

## Current State

Previously, `cim-domain-graph` was a standalone domain module implementing DDD patterns for graph management. The other graph modules (`contextgraph`, `conceptgraph`, `workflow-graph`, `ipld-graph`) were independent implementations that all depend on `cim-contextgraph` as their base.

## Implemented Architecture

### Core Abstraction

```rust
/// Core trait that all graph implementations must satisfy
pub trait GraphImplementation: Send + Sync {
    type NodeData: Clone + Send + Sync;
    type EdgeData: Clone + Send + Sync;
    
    fn add_node(&mut self, node_id: NodeId, data: Self::NodeData) -> GraphResult<()>;
    fn add_edge(&mut self, edge_id: EdgeId, from: NodeId, to: NodeId, data: Self::EdgeData) -> GraphResult<()>;
    fn remove_node(&mut self, node_id: NodeId) -> GraphResult<Self::NodeData>;
    fn remove_edge(&mut self, edge_id: EdgeId) -> GraphResult<Self::EdgeData>;
    // ... other operations
}
```

### Graph Type Enum

```rust
/// Runtime polymorphic graph type using concrete types
#[derive(Debug)]
pub enum GraphType {
    Context(ContextGraphAdapter),
    // TODO: Add these when implementations are ready
    // Concept(ConceptGraphAdapter),
    // Workflow(WorkflowGraphAdapter),
    // Ipld(IpldGraphAdapter),
}
```

### Abstract Graph Aggregate

```rust
/// Abstract graph aggregate that can work with any graph implementation
#[derive(Debug)]
pub struct AbstractGraph {
    graph: GraphType,
    version: u64,
}
```

## Implementation Status

### Phase 1: Define Common Interface ✅
- Created `GraphImplementation` trait
- Defined common operations all graphs must support
- Created `GraphType` enum for runtime polymorphism

### Phase 2: Add Dependencies ✅
- Added `cim-contextgraph`, `cim-conceptgraph`, `cim-workflow-graph` as dependencies
- Created `ContextGraphAdapter` with full implementation
- Created placeholder adapters for other graph types

### Phase 3: Refactor Domain Logic ✅
- Created `AbstractGraph` aggregate that works with `GraphType`
- Updated to use concrete types instead of trait objects (for dyn compatibility)
- Implemented all basic graph operations

### Phase 4: Integration 🚧
- Need to update handlers to use `AbstractGraph`
- Need to update projections to handle different graph types
- Need to update tests and examples for new API

## Key Design Decisions

1. **Concrete Types over Trait Objects**: Due to Rust's dyn compatibility requirements, we use concrete enum types rather than trait objects. This provides better performance and type safety.

2. **Adapter Pattern**: Each graph implementation has an adapter that implements the `GraphImplementation` trait, providing a uniform interface while preserving specific functionality.

3. **ID Mapping**: The adapters maintain bidirectional mappings between domain IDs and implementation-specific IDs, ensuring consistent identity management.

## Benefits Achieved

1. **Unified Interface**: Single domain API for all graph operations
2. **Type Safety**: Each graph type maintains its specific invariants
3. **Flexibility**: Easy to add new graph types by implementing the trait
4. **Domain Consistency**: All graphs follow same DDD patterns

## Next Steps

1. Implement remaining adapters (Concept, Workflow, IPLD)
2. Update command handlers to use `AbstractGraph`
3. Create migration guide for existing code
4. Add comprehensive tests for the abstraction layer

## Example Usage

```rust
// Create a context graph through the abstraction
let graph = AbstractGraph::new(
    GraphId::new(),
    "My Graph".to_string(),
    GraphTypeSpec::Context { invariants: vec![] },
)?;

// Operations work uniformly
graph.add_node(node_id, NodeType::Task, position, metadata)?;
graph.add_edge(edge_id, from, to, EdgeType::Sequence, metadata)?;
``` 