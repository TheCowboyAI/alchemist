# Alchemist Graph Viewer

## Overview

The Alchemist Graph Editor now has a fully functional 3D/2D graph visualization system with:

- **Dual viewing modes**: Switch between 3D orbit camera and 2D top-down view
- **Real-time rendering**: See graph nodes and edges rendered in the Bevy world
- **Interactive controls**: Mouse and keyboard controls for navigation
- **Domain-driven nodes**: Different node types with distinct colors
- **Graph patterns**: Generate common graph structures with a single click
- **File operations**: Load and save graph data as JSON files
- **Fixed 2D zoom**: The 2D view now starts at a proper zoom level for better visibility

## How to Run

```bash
# Using nix
nix run

# Using cargo
cargo run
```

## Controls

### 3D View Mode (Default)
- **Right Mouse**: Orbit camera around focus point
- **Middle Mouse + Shift**: Pan camera
- **Scroll Wheel**: Zoom in/out
- **Tab or V**: Switch to 2D view

### 2D View Mode
- **Middle Mouse**: Pan camera
- **Scroll Wheel**: Zoom in/out
- **Tab or V**: Switch to 3D view

## Features

### Visual Elements
- **Nodes**: Rendered as 3D spheres (in 3D mode) or 2D circles (in 2D mode)
- **Node Colors**: Based on domain type
  - Blue: Process nodes
  - Yellow: Decision nodes
  - Green: Event nodes
  - Purple: Storage nodes
  - Cyan: Interface nodes
- **Reference Grid**: A ground plane in 3D mode for spatial reference

### Graph Patterns
The UI now includes buttons to generate common graph structures:
- **⭐ Star**: Central hub with radiating connections
- **🌳 Tree**: Hierarchical branching structure
- **🔄 Cycle**: Nodes connected in a ring
- **🔗 Complete**: Every node connected to every other
- **📊 DAG**: Directed Acyclic Graph with levels
- **🤖 Moore**: Moore state machine pattern
- **🔷 Grid**: 2D lattice structure
- **🎭 Bipartite**: Two sets of nodes with cross-connections

### File Operations
- **Load JSON Files**: Browse and load graph data from `assets/models/` directory
- **Save**: Save current graph to file
- **Save As**: Save with a new timestamped filename
- **Sample File**: Includes `sample_graph.json` demonstrating the format

### UI Panel
- Shows current view mode
- Displays control instructions
- Graph statistics (node/edge count)
- Graph pattern generation buttons
- File operations section
- "Add Random Node" button to create individual nodes

## Architecture

The system follows the plan from `doc/plan/`:
- **Camera Module** (`src/camera/`): Handles dual-mode camera system with improved zoom calculations
- **Graph Core Module** (`src/graph_core/`): ECS-based graph components and systems
- **Graph Patterns** (`src/graph_patterns.rs`): Pattern generation algorithms
- **Event-Driven**: All graph modifications go through events
- **Performance Optimized**: Includes frustum culling and LOD systems

## What You're Seeing

When you run the application, you'll see:
1. A window with the 3D scene in the center
2. A left panel with controls and information
3. Initial test nodes arranged in a cross pattern
4. Smooth camera transitions when switching modes (2D view now properly zoomed)
5. Real-time rendering of graph elements
6. Pattern generation capabilities
7. File loading/saving options

The viewport properly accounts for the UI panel, and the camera smoothly transitions between 3D orbit and 2D top-down views. The 2D view now starts at a reasonable zoom level (0.1) for better visibility.

## JSON File Format

The graph data uses a simple JSON format:
```json
{
  "nodes": [
    {
      "id": "n1",
      "position": { "x": 0, "y": 0 },
      "caption": "Node Name",
      "labels": ["type"],
      "properties": {},
      "style": {}
    }
  ],
  "relationships": [
    {
      "id": "r1",
      "fromId": "n1",
      "toId": "n2",
      "type": "connection_type",
      "properties": {},
      "style": {}
    }
  ]
}
```

## Next Steps

From here, you can:
- Add edge rendering for patterns and loaded files
- Implement node selection and manipulation
- Add more sophisticated graph algorithms
- Integrate with the workflow and domain systems
- Enhance the JSON loader to properly create edges 