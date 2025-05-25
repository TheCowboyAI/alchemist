# Edge Refactoring Plan (OBSOLETE)

## ⚠️ This Plan is Obsolete

This edge refactoring plan has been superseded by the **ECS Refactoring Phase 5** implementation, which introduced a better approach for handling edges.

## Current Implementation (2024)

The edge architecture has been completely redesigned as part of the ECS refactoring:

### New Approach: Edges as Components
- **Nodes as Entities**: Each graph node is an ECS entity
- **Edges as Components**: Edges are `OutgoingEdge` components attached to source nodes
- **No Edge Entities**: Edges are not separate entities, avoiding the dual-layer violation
- **Efficient Rendering**: Edge meshes are rendered by iterating nodes with `OutgoingEdge` components

### Implementation Details
See `doc/plan/graph-implementation-status.md` for the complete edge implementation approach, including:
- `OutgoingEdge` component structure
- Edge creation/deletion systems
- Rendering approach using `EdgeMeshTracker`

### Benefits Achieved
1. ✅ **No Flickering**: Stable edge rendering without entity creation/destruction
2. ✅ **Better Performance**: Efficient component queries instead of entity lookups
3. ✅ **Simpler Logic**: No deferred edge events needed
4. ✅ **Clean Architecture**: Proper separation between data (GraphData) and rendering (ECS)

## References
- [ECS Refactoring Plan](./ecs-refactoring-plan.md) - See Phase 5 for system decomposition
- [Graph Implementation Status](./graph-implementation-status.md) - See "2024 ECS Edge Refactor" section
- [Graph Systems Implementation](../../src/systems/graph/) - Actual implementation code
