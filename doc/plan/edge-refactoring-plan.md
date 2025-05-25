# Edge Refactoring Plan

## Current Problems

1. **Edge Entities**: We're creating ECS entities for edges (`GraphEdge` component), which violates the dual-layer architecture
2. **Timing Issues**: Deferred edge creation causing flickering
3. **Duplicate Systems**: Both old and new edge creation systems running
4. **Inefficient Rendering**: Individual mesh per edge instead of batched rendering

## Correct Architecture (from graph-implementation-status.md)

```
Graph Data Layer (petgraph/daggy)
    ↓
Rendering Layer (Bevy ECS - meshes only, no edge entities)
```

## Implementation Steps

### 1. Remove Edge Entities
- **DELETE**: `GraphEdgeBundle` - edges shouldn't be entities
- **DELETE**: `GraphEdge` component - edges live in GraphData only
- **KEEP**: `EdgeVisual` for rendering properties only

### 2. Edge Storage
```rust
// Edges exist ONLY in GraphData (petgraph)
pub struct GraphData {
    graph: DiGraph<NodeData, EdgeData>,
    // ...
}

// No edge entities, no edge components
```

### 3. Edge Rendering
```rust
// Single system that renders ALL edges from GraphData as meshes
fn render_edges_from_graph(
    graph_data: Res<GraphData>,
    node_query: Query<(Entity, &Transform), With<GraphNode>>,
    mut edge_mesh_tracker: ResMut<EdgeMeshTracker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Clear old edge meshes
    edge_mesh_tracker.despawn_all(&mut commands);

    // Create batched mesh for all edges
    let edge_meshes = create_batched_edge_meshes(&graph_data, &node_query);

    // Spawn mesh entities (NOT edge entities)
    for mesh in edge_meshes {
        let entity = commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(edge_material())),
            // NO GraphEdge component!
        )).id();

        edge_mesh_tracker.track(entity);
    }
}
```

### 4. Simplify Edge Creation
```rust
// Just add to graph data, no entity creation
fn handle_create_edge_event(
    mut events: EventReader<CreateEdgeEvent>,
    mut graph_data: ResMut<GraphData>,
) {
    for event in events.read() {
        graph_data.add_edge(
            event.source_uuid,
            event.target_uuid,
            EdgeData { /* ... */ }
        )?;
    }
    // That's it! Rendering system will pick it up
}
```

### 5. Remove Deferred Edges
- Delete `DeferredEdgeEvent` completely
- Use UUID-based edge creation that waits for nodes in GraphData
- No retry logic needed

## Benefits
1. **No Flickering**: No entity creation/destruction for edges
2. **Better Performance**: Batched rendering, fewer entities
3. **Simpler Logic**: No timing issues, no deferred events
4. **Follows Architecture**: Clean separation of data and rendering

## Migration Order
1. Create `EdgeMeshTracker` resource
2. Implement `render_edges_from_graph` system
3. Remove edge entity creation from all systems
4. Delete `GraphEdgeBundle` and `GraphEdge` component
5. Delete deferred edge system
6. Clean up unused imports and systems
