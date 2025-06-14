# Refactor cim-contextgraph to Standalone Module

## Overview
Refactor `cim-contextgraph` to be a pure, standalone graph theory implementation without domain-specific dependencies. This module should provide fundamental graph abstractions that other modules can compose with their domain logic.

## Current Issues
1. **workflow_graph.rs** - Contains domain-specific workflow logic that depends on `cim_domain`
2. **cid_dag.rs** - Implements content-addressed DAG functionality that should be in a separate module
3. **lib.rs** - Exports domain-specific types that don't belong in a pure graph library

## Refactoring Plan

### Phase 1: Remove Domain-Specific Files
1. **Move workflow_graph.rs**
   - Target: Create new module `cim-domain-workflow-graph` or move to `cim-domain-workflow`
   - This file imports from `cim_domain::workflow` and contains workflow-specific logic
   - It's a composition of context graph with workflow domain concepts

2. **Move cid_dag.rs**
   - Target: Create new module `cim-ipld-graph` or move to `cim-ipld`
   - This file implements EventDag and ObjectDag using CIDs and daggy
   - It's a composition of DAG structures with content-addressing

### Phase 2: Clean Up Dependencies
1. **Remove from Cargo.toml**:
   - `cid` - Only needed for cid_dag.rs
   - `daggy` - Only needed for cid_dag.rs
   - Keep: `petgraph`, `nalgebra`, `serde`, `uuid`, `thiserror`

2. **Update lib.rs**:
   - Remove exports of `workflow_graph` module
   - Remove exports of `cid_dag` module and related types
   - Remove `CidDag`, `EventDag`, `ObjectDag`, `CidNode`, `CidEdge`, `EventNode`, `ObjectNode` exports

### Phase 3: Ensure Pure Graph Abstractions
1. **Core Types to Keep**:
   - `ContextGraph<N, E>` - Generic graph structure
   - `NodeId`, `EdgeId`, `ContextGraphId`, `ConceptGraphId` - Identity types
   - `Component`, `ComponentStorage` - Component system
   - `NodeEntry`, `EdgeEntry` - Graph entries
   - `Label`, `Metadata`, `GraphReference`, `Subgraph` - Generic components
   - `GraphError`, `GraphResult` - Error handling
   - `GraphInvariant`, `Acyclic`, `Connected` - Graph properties
   - Graph operations: `compose`, `union`, `intersection`, `product`

2. **Verify No Domain Dependencies**:
   - No imports from `cim_domain` or other domain modules
   - All types should be generic and reusable
   - Components should be trait-based, not concrete domain types

### Phase 4: Create New Modules for Moved Code
1. **cim-workflow-graph** (new module):
   ```toml
   [dependencies]
   cim-contextgraph = { path = "../cim-contextgraph" }
   cim-domain = { path = "../cim-domain" }
   ```
   - Move `workflow_graph.rs` here
   - This becomes a composition module

2. **cim-ipld-graph** (new module or add to cim-ipld):
   ```toml
   [dependencies]
   cim-contextgraph = { path = "../cim-contextgraph" }
   cim-ipld = { path = "../cim-ipld" }
   daggy = "0.8"
   cid = "0.11"
   ```
   - Move `cid_dag.rs` here
   - This becomes a composition module

### Phase 5: Update Documentation
1. Update module documentation to reflect pure graph focus
2. Add examples showing how to compose with domain types
3. Document the component system for extensibility

## Implementation Steps

1. **Create new module directories**:
   ```bash
   mkdir -p cim-workflow-graph/src
   mkdir -p cim-ipld-graph/src
   ```

2. **Move files**:
   ```bash
   mv cim-contextgraph/src/workflow_graph.rs cim-workflow-graph/src/lib.rs
   mv cim-contextgraph/src/cid_dag.rs cim-ipld-graph/src/lib.rs
   ```

3. **Update cim-contextgraph/src/lib.rs**:
   - Remove module declarations for moved files
   - Remove re-exports of moved types

4. **Update cim-contextgraph/Cargo.toml**:
   - Remove `cid` and `daggy` dependencies

5. **Create Cargo.toml for new modules** with appropriate dependencies

6. **Update root Cargo.toml** to include new workspace members

7. **Fix any compilation errors** in dependent modules

8. **Run tests** to ensure nothing is broken

## Success Criteria
- `cim-contextgraph` has no dependencies on domain modules
- `cim-contextgraph` provides only generic graph abstractions
- Moved code works in new modules with proper dependencies
- All tests pass
- Documentation is updated

## Benefits
1. **Modularity**: Clear separation of concerns
2. **Reusability**: Pure graph module can be used by any domain
3. **Maintainability**: Easier to understand and modify
4. **Testability**: Can test graph logic independently of domains
5. **Composability**: Other modules can compose graph with their domains
