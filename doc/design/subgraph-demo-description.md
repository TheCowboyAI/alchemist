# Subgraph Spatial Mapping Demo - Visual Description

## What the Demo Displays

When you run `cargo run --example subgraph_demo`, you'll see:

### Visual Elements

1. **Three Subgraphs**: Each subgraph is displayed as a collection of nodes arranged in a circular pattern:
   - **Subgraph 1 (Red)**: Located on the left (-15, 0, 0), contains 5 red spherical nodes
   - **Subgraph 2 (Green)**: Located in the center (0, 0, 0), contains 5 green spherical nodes
   - **Subgraph 3 (Blue)**: Located on the right (15, 0, 0), contains 5 blue spherical nodes

2. **Node Arrangement**: Within each subgraph, nodes are arranged in a perfect circle with a radius of 5 units around their subgraph's origin point.

3. **Subgraph Labels**: Each subgraph has a text label floating above it showing "Subgraph 1", "Subgraph 2", or "Subgraph 3".

4. **Boundary Visualization**: The `visualize_subgraph_boundaries` system draws semi-transparent bounding boxes around each subgraph, showing the spatial extent of each group.

### Interactive Controls

- **Number Keys (1, 2, 3)**: Select which subgraph to control
- **Arrow Keys**: Move the selected subgraph as a unit:
  - Left/Right arrows: Move along the X-axis
  - Up/Down arrows: Move along the Z-axis
- **Space Bar**: Activate animation mode

### Animation Patterns

When holding the Space bar, each subgraph animates with a different pattern:

1. **Subgraph 1 (Red)**: Moves in a circular pattern on the XZ plane
2. **Subgraph 2 (Green)**: Oscillates up and down along the Y-axis
3. **Subgraph 3 (Blue)**: Traces a figure-8 pattern on the XZ plane

### Key Demonstration Features

1. **Hierarchical Movement**: When you move a subgraph using the arrow keys, all nodes within that subgraph move together as a unit, maintaining their relative positions.

2. **Independent Origins**: Each subgraph has its own origin point (an invisible parent entity) that serves as the base for all transformations.

3. **Spatial Mapping**: The `SubgraphSpatialMap` resource tracks the relationship between graph IDs and their spatial entities, allowing for efficient lookup and manipulation.

4. **Parent-Child Transform Hierarchy**: Bevy's built-in transform system handles the hierarchical relationships, so moving a parent (subgraph origin) automatically moves all children (nodes).

### Technical Implementation

The demo showcases:
- How to create invisible parent entities as subgraph origins
- How to establish parent-child relationships in Bevy ECS
- How to move entire groups of entities as units
- How to visualize spatial boundaries of entity groups
- How to implement different animation patterns for different subgraphs

This demonstrates the core concept you requested: subgraphs that can be moved as cohesive units while maintaining the relative positions of their constituent nodes.
