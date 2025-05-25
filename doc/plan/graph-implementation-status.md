# Graph Implementation Status

## 2024 ECS Edge Refactor: Nodes as Entities, Edges as Components

### New ECS Pattern

- **Nodes as Entities:** Each graph node is an ECS entity with a `GraphNode` component.
- **Edges as Components:** Each edge is represented as an `OutgoingEdge` component attached to the *source* node entity. This component contains:
    - `id`: UUID of the edge (matches GraphData and events)
    - `target`: ECS entity of the target node
    - `edge_type`, `labels`, `properties`: all edge metadata
- **Edge Creation:**
    - Add the edge to `GraphData` (data layer).
    - Attach an `OutgoingEdge` component to the source node entity in ECS.
- **Edge Deletion:**
    - Remove the edge from `GraphData`.
    - Emit a `DeleteEdgeEvent { source, edge_id }`.
    - System removes the `OutgoingEdge` component with the matching `edge_id` from the source node entity.
- **Edge Rendering:**
    - Rendering system iterates all nodes and their `OutgoingEdge` components.
    - For each outgoing edge, renders a mesh between the source and target node entities.
    - Uses `EdgeMeshTracker` to manage and clean up mesh entities.

### Benefits
- **No edge ECS entities:** Only nodes are entities; edges are lightweight components.
- **Efficient traversal:** Query all outgoing edges for a node in O(1) ECS time.
- **No data duplication:** All properties live in `GraphData`; ECS is a projection for rendering and interaction.
- **Easy selection/highlighting:** Both nodes and edges can be selected, highlighted, or animated via ECS.

### Migration/Implementation Steps

1. Add `OutgoingEdge` component to ECS.
2. Update edge creation system to attach `OutgoingEdge` to the source node entity.
3. Update edge deletion system to remove the correct `OutgoingEdge` from the source node entity.
4. Refactor edge rendering system to use `OutgoingEdge` components.
5. Remove all legacy edge entity logic and ensure no ECS edge entities remain.
6. Test and validate: Ensure correct rendering, selection, and deletion of edges.

### Status Table Update

| Area                | Old Approach                | New Approach (2024)         | Status      |
|---------------------|----------------------------|-----------------------------|-------------|
| Node ECS Mapping    | Entity per node            | Entity per node             | ✅ Complete |
| Edge ECS Mapping    | Entity per edge            | OutgoingEdge component      | ✅ Complete |
| Edge Properties     | ECS + GraphData (dup)      | GraphData only, ECS ref     | ✅ Complete |
| Edge Rendering      | Per-edge entity/mesh       | Mesh per OutgoingEdge       | ✅ Complete |
| Edge Deletion       | Despawn entity             | Remove OutgoingEdge comp    | ✅ Complete |
| Batched Rendering   | Not implemented            | (Planned/Optional)          | 🚧 Planned  |

### Example Code

```rust
#[derive(Component, Debug, Clone)]
pub struct OutgoingEdge {
    pub id: Uuid,         // Edge UUID
    pub target: Entity,   // Target node entity
    pub edge_type: DomainEdgeType,
    pub labels: Vec<String>,
    pub properties: HashMap<String, String>,
}

#[derive(Event)]
pub struct DeleteEdgeEvent {
    pub source: Entity,
    pub edge_id: Uuid,
}

pub fn handle_delete_edge_events(
    mut commands: Commands,
    events: EventReader<DeleteEdgeEvent>,
    mut node_query: Query<&mut OutgoingEdge, With<GraphNode>>,
) {
    for event in &events {
        if let Ok(outgoing_edge) = node_query.get_mut(event.source) {
            if outgoing_edge.id == event.edge_id {
                commands.entity(event.source).remove::<OutgoingEdge>();
            }
        }
    }
}
```

---

## Previous Status and Architecture (for reference)

## Overview

Following the frustration with our ad-hoc graph implementation (December 2024), we've begun transitioning to a proper dual-layer architecture using established graph libraries.

## Current Status

### ✅ Completed
1. **Dependencies Added**
   - `petgraph = "0.6"` - General graph algorithms
   - `daggy = "0.8"` - DAG-specific operations for Merkle DAGs

2. **Core Infrastructure**
   - `GraphData` resource using petgraph's DiGraph
   - `MerkleDag` resource using Daggy for CIM requirements
   - Bidirectional mapping between graph indices and ECS entities

3. **Documentation**
   - Comprehensive architecture plan in `doc/plan/graph-architecture.md`
   - Implementation patterns in cursor rules
   - Business requirements documented in `cim-graphs.md`
   - ✅ **COMPLETED**: JSON schema documentation in `doc/merkle-dag-schema.md`

4. **Migration from Current System**
   - ✅ Replaced `handle_create_node_events` with `handle_create_node_with_graph`
   - ✅ Updated edge creation to use graph indices via `handle_create_edge_with_graph`
   - ✅ Fixed edge rendering using simplified rotation with `Quat::from_rotation_arc`
   - ✅ Added `process_deferred_edges` system for handling edges after nodes are created
   - ✅ Updated plugin to use new graph-based handlers

5. **Daggy API Usage** (Fixed December 2024)
   - ✅ Fixed walker API usage with proper `walker.next(&dag)` pattern
   - ✅ Implemented BFS and DFS traversal using manual stack/queue approach
   - ✅ Fixed node/edge iteration using `raw_nodes()` and `raw_edges()`
   - ✅ Implemented ancestors/descendants methods
   - ✅ **COMPLETED**: Replaced GraphML with arrows.app compatible JSON serialization
   - ✅ **COMPLETED**: Bidirectional conversion between native and arrows.app formats

6. **Graph Algorithms Module** (`algorithms.rs`)
   - ✅ Dijkstra shortest path with path reconstruction
   - ✅ Connectivity checking (has_path_connecting)
   - ✅ Strongly connected components (Tarjan's algorithm)
   - ✅ Cycle detection for directed graphs
   - ✅ BFS/DFS traversal with depth limiting
   - ✅ Topological sorting for DAGs
   - ✅ Degree centrality calculation
   - ✅ Find all paths between nodes (with limit)

7. **Change Detection System** (`change_detection.rs`)
   - ✅ GraphChangeTracker resource for tracking modifications
   - ✅ Per-frame change tracking (added/modified/removed)
   - ✅ Component change detection using Bevy's Ref<T>
   - ✅ LOD system based on camera distance
   - ✅ Batched mesh update preparation

8. **UI for Graph Inspection** (`ui.rs`)
   - ✅ Graph inspector with search and filtering
   - ✅ Node/edge property viewer
   - ✅ Graph statistics window (connectivity, cycles, degrees)
   - ✅ Algorithm controls (pathfinding, analysis)
   - ✅ Node selection via mouse click
   - ✅ Visual selection highlighting

9. **Serialization Formats** (Completed December 2024)
   - ✅ Native JSON schema for efficient serialization
   - ✅ Arrows.app compatible JSON format for visualization
   - ✅ Import/export from arrows.app with full schema support
   - ✅ Metadata preservation with proper namespacing
   - ✅ Color format conversion (RGBA ↔ Hex)
   - ✅ Support for arrows.app style objects and properties
   - ✅ Round-trip conversion testing
   - ❌ ~~GraphML export~~ (Removed in favor of JSON)

10. **Architectural Clarifications** (December 2024)
    - ✅ Separated visualization graph format (arrows.app) from MerkleDag format
    - ✅ MerkleDag provides arrows.app import/export as an interface
    - ✅ Clear distinction between directed graphs for rendering vs cryptographic DAGs
    - ✅ Support for loading existing arrows.app graph files

### 🚧 In Progress
1. **Performance Optimizations**
   - ✅ Implement force-directed layout algorithm
   - ✅ Integrate layout controls in UI
   - ✅ Fix edge rendering from base graph
   - Implement actual batched mesh generation
   - Spatial indexing for large graphs
   - Implement geometric and hierarchical layouts

2. **Advanced UI Features**
   - Path visualization when algorithm finds route
   - Interactive graph editing (add/remove nodes via UI)
   - ✅ Graph layout algorithm integration (Force-Directed)
   - Direct arrows.app file loading for visualization

### ❌ Not Started
1. **Advanced Features**
   - NATS event integration
   - Full Merkle proof validation
   - Real-time collaboration
   - Graph diffing and merging
   - Separate GraphData loader for arrows.app files (non-Merkle)

## Key Issues Resolved

1. **No Graph Algorithms** → ✅ Full petgraph algorithm suite available
2. **Entity Coupling** → ✅ Separated graph data from ECS entities
3. **Constant Re-rendering** → ✅ Change detection implemented
4. **No DAG Semantics** → ✅ Daggy provides cycle detection and DAG operations
5. **Edge Rendering** → ✅ Fixed rotation calculation using proper quaternion math
6. **No Graph Inspection** → ✅ Full UI with search, stats, and algorithms
7. **Limited Export Options** → ✅ Arrows.app compatible JSON format
8. **Format Confusion** → ✅ Clear separation between visualization and Merkle formats

## Next Steps

### Immediate (This Week)
1. ✅ Implement force-directed layout algorithm
2. ✅ Fix edge rendering from base graph
3. Implement hierarchical layout algorithm
4. Add path visualization for algorithm results
5. Optimize batched rendering for 10k+ nodes
6. Add graph editing capabilities to UI
7. ✅ Test arrows.app round-trip conversion
8. Implement edge bundling for complex graphs

### Short Term (Next Sprint)
1. Integrate spatial indexing (R-tree or similar)
2. Add graph validation using Daggy
3. Implement graph diffing
4. Performance benchmarking at scale
5. Create arrows.app templates for common patterns
6. **NEW**: Add GraphData loader for arrows.app files (bypass MerkleDag for pure visualization)

### Long Term (Q1 2025)
1. Full Merkle DAG implementation with proofs
2. 250k+ element support with LOD
3. Real-time collaboration via NATS
4. Advanced visualization modes (3D layouts, VR support)

## Code Examples

### Using Graph Algorithms
```rust
// Find shortest path
let path = GraphAlgorithms::shortest_path(&graph_data, start_id, end_id);

// Check connectivity
let connected = GraphAlgorithms::are_connected(&graph_data, node1_id, node2_id);

// Get topological ordering
let order = GraphAlgorithms::topological_sort(&graph_data)?;

// Find all paths (limited)
let paths = GraphAlgorithms::find_all_paths(&graph_data, start, end, 10);
```

### Using Arrows.app Export/Import
```rust
// Export MerkleDag to arrows.app format
let arrows_json = merkle_dag.to_arrows_json()?;
std::fs::write("graph.json", arrows_json)?;

// Import arrows.app file into MerkleDag
let json = std::fs::read_to_string("graph.json")?;
let dag = MerkleDag::from_arrows_json(&json)?;

// Convert between formats
let native = dag.to_json()?;  // Native MerkleDag format
let arrows = dag.to_arrows_json()?;  // Arrows.app visualization format

// Load existing arrows.app files (from assets/models/)
let capability_map = std::fs::read_to_string("assets/models/Capability Map.json")?;
let dag = MerkleDag::from_arrows_json(&capability_map)?;
```

### UI Interaction
```rust
// Inspector state tracks selection
inspector_state.selected_node = Some(node_id);

// Set pathfinding endpoints
inspector_state.pathfind_source = Some(start_id);
inspector_state.pathfind_target = Some(end_id);

// UI automatically updates based on state
```

## Performance Targets

| Metric | Current | Target | Status |
|--------|---------|---------|---------|
| Max Nodes | ~1,000 | 250,000 | ✅ Architecture ready, needs optimization |
| Max Edges | ~2,000 | 500,000 | ✅ Architecture ready, needs optimization |
| FPS @ 10k nodes | 30 | 60 | 🚧 Change detection helps, needs batching |
| Graph Algorithms | O(V+E) | O(V+E) | ✅ Petgraph provides optimal algorithms |
| UI Responsiveness | Good | Excellent | ✅ egui integration working well |
| JSON Export/Import | Fast | Fast | ✅ Both formats perform well |
| Arrows.app Support | Full | Full | ✅ Complete import/export with tests |

## Resources

- [Petgraph Docs](https://docs.rs/petgraph)
- [Daggy Docs](https://docs.rs/daggy)
- [Bevy ECS Best Practices](../bevy-graphs.md)
- [CIM Requirements](../cim-graphs.md)
- [egui Docs](https://docs.rs/egui)
- [Arrows.app](https://arrows.app) - Graph visualization tool
- [MerkleDag Schema Docs](../merkle-dag-schema.md)
