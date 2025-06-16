# ContextGraph Consolidation

## Date: 2025-01-11

## Summary
Consolidated the two implementations of ContextGraph (v1 and v2) into a single PetGraph-based implementation, and cleaned up the nested cim-viz-bevy directory.

## Changes Made

### 1. Removed Nested cim-viz-bevy Directory
- Removed the empty `cim-contextgraph/cim-viz-bevy/` directory
- The actual `cim-viz-bevy` module already exists at the workspace root level

### 2. Consolidated ContextGraph Implementations
- **Removed**: `context_graph.rs` (v1) - HashMap-based custom implementation
- **Kept**: `context_graph_v2.rs` → renamed to `context_graph.rs`
- **Rationale**: The v2 implementation wraps PetGraph, providing:
  - Better performance
  - Access to all PetGraph algorithms
  - More efficient memory layout
  - Proven graph data structure

### 3. API Changes
The new implementation has a different API since it wraps PetGraph:
- Direct access to nodes/edges collections removed
- Use `graph.node_count()` instead of `graph.nodes().len()`
- Use `graph.edge_count()` instead of `graph.edges().len()`
- Component queries remain the same
- Graph algorithms (topological sort, cycle detection, etc.) are exposed

### 4. Fixed Compilation Issues
- Added missing `GraphInvariant` trait definition
- Added `Debug` and `Clone` implementations for `ContextGraph`
- Fixed trait bounds for generic parameters
- Added missing error variants
- Updated examples to use new API

### 5. Updated Examples
- `graph_composition.rs` - Removed references to moved types (CidDag, EventDag, ObjectDag)
- `simple_benchmark.rs` - Updated to use new API
- `benchmark_comparison.rs` - Added `rand` as dev dependency

## Current Status
- ✅ Module compiles successfully
- ✅ Examples compile and demonstrate usage
- ❌ Tests need updating to match new API (79 test failures)

## Benefits of Consolidation
1. **Single Source of Truth**: No confusion about which implementation to use
2. **Better Performance**: PetGraph is highly optimized
3. **More Features**: Access to all PetGraph algorithms
4. **Cleaner Code**: Less custom implementation to maintain
5. **Industry Standard**: PetGraph is the de facto Rust graph library

## Next Steps
1. Update all tests to use the new PetGraph-based API
2. Consider exposing more PetGraph functionality if needed
3. Document the API changes for users
4. Add migration guide from v1 to v2 API

## Design Decision
Following the principle stated by the user: "we should ONLY be wrapping petgraph and daggy", this consolidation aligns perfectly with that directive. The custom HashMap-based implementation has been removed in favor of wrapping the proven PetGraph library.
