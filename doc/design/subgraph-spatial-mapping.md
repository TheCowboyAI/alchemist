# Subgraph Spatial Mapping System

## Overview

The subgraph spatial mapping system provides functionality for managing subgraphs as spatial units with their own coordinate systems and base origins. This allows entire subgraphs to be moved as cohesive units while maintaining the relative positions of nodes within them.

## Key Components

### 1. SubgraphOrigin Component
```rust
#[derive(Component, Debug, Clone)]
pub struct SubgraphOrigin {
    pub graph_id: GraphId,
    pub base_position: Vec3,
}
```
- Represents the base origin point for a subgraph
- Acts as the parent transform for all nodes in the subgraph
- Hidden from visual rendering but participates in transform hierarchy

### 2. SubgraphMember Component
```rust
#[derive(Component, Debug, Clone)]
pub struct SubgraphMember {
    pub graph_id: GraphId,
}
```
- Marks entities as belonging to a specific subgraph
- Enables filtering and operations on subgraph members

### 3. SubgraphSpatialMap Resource
```rust
#[derive(Resource, Default)]
pub struct SubgraphSpatialMap {
    pub origins: HashMap<GraphId, Entity>,
    pub positions: HashMap<GraphId, Vec3>,
}
```
- Global resource tracking all subgraph origins
- Maps graph IDs to their origin entities and positions
- Enables efficient lookup and management of subgraphs

## Core Systems

### Creating Subgraphs
```rust
pub fn create_subgraph_origin(
    mut commands: Commands,
    mut spatial_map: ResMut<SubgraphSpatialMap>,
) -> Entity
```
- Creates a new subgraph with an origin entity
- Registers the subgraph in the spatial map
- Returns the origin entity for parent-child relationships

### Adding Nodes to Subgraphs
```rust
pub fn add_node_to_subgraph(
    mut commands: Commands,
    spatial_map: Res<SubgraphSpatialMap>,
    graph_id: GraphId,
    node_id: NodeId,
    relative_position: Vec3,
    label: String,
) -> Option<Entity>
```
- Creates nodes as children of the subgraph origin
- Positions are relative to the subgraph origin
- Automatically inherits transforms from parent

### Moving Subgraphs
```rust
pub fn move_subgraph(
    mut spatial_map: ResMut<SubgraphSpatialMap>,
    mut query: Query<&mut Transform, With<SubgraphOrigin>>,
    graph_id: GraphId,
    new_position: Vec3,
)
```
- Updates the origin position of a subgraph
- All child nodes automatically move with the origin
- Leverages Bevy's transform hierarchy

### Layout Functions
```rust
pub fn circular_layout(radius: f32, count: usize) -> impl Fn(usize) -> Vec3
```
- Example layout function for arranging nodes in a circle
- Returns a closure that maps node indices to positions
- Can be used with `layout_subgraph_nodes` system

### Visualization
```rust
pub fn visualize_subgraph_boundaries(
    mut gizmos: Gizmos,
    spatial_map: Res<SubgraphSpatialMap>,
    nodes: Query<(&GlobalTransform, &SubgraphMember), With<GraphNode>>,
)
```
- Draws bounding boxes around subgraphs
- Helps visualize subgraph boundaries and spatial relationships
- Uses Bevy's gizmo system for debug visualization

## Usage Example

```rust
// Create a subgraph
let origin_entity = create_subgraph_origin(&mut commands, &mut spatial_map);
let graph_id = /* get graph_id from spatial_map */;

// Add nodes to the subgraph
for i in 0..5 {
    add_node_to_subgraph(
        &mut commands,
        &spatial_map,
        graph_id,
        NodeId::new(),
        Vec3::new(i as f32 * 2.0, 0.0, 0.0),
        format!("Node {}", i),
    );
}

// Move the entire subgraph
move_subgraph(&mut spatial_map, &mut transforms, graph_id, Vec3::new(10.0, 5.0, 0.0));
```

## Benefits

1. **Hierarchical Organization**: Subgraphs maintain clear parent-child relationships
2. **Efficient Movement**: Moving a subgraph origin automatically moves all children
3. **Relative Positioning**: Nodes maintain positions relative to their subgraph origin
4. **Visual Clarity**: Boundary visualization helps understand graph structure
5. **Modular Design**: Each subgraph can be managed independently

## Integration with CIM

This subgraph spatial mapping system aligns with CIM's graph-based workflow representation by:

- Enabling visual grouping of related workflow components
- Supporting hierarchical workflow structures
- Facilitating spatial reasoning about workflow relationships
- Providing foundation for conceptual space mapping

## Testing

The system includes comprehensive integration tests covering:
- Subgraph creation and registration
- Node addition to subgraphs
- Subgraph movement with child nodes
- Multiple subgraph management
- Layout function application

## Future Enhancements

1. **Nested Subgraphs**: Support for subgraphs within subgraphs
2. **Dynamic Layouts**: More sophisticated layout algorithms
3. **Collision Detection**: Prevent subgraph overlap
4. **Serialization**: Save/load subgraph configurations
5. **Animation**: Smooth transitions when moving subgraphs
