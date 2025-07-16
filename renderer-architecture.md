# Alchemist Hybrid Renderer Architecture

## Overview

Alchemist uses a hybrid rendering architecture that can spawn both Bevy (3D) and Iced (2D) windows from the main shell application. This allows for optimal visualization of different data types.

## Architecture

```
┌─────────────────────┐
│  Alchemist Shell    │
│  (Main Process)     │
└──────────┬──────────┘
           │ spawn
           ├─────────────────┐
           │                 │
    ┌──────▼──────┐   ┌──────▼──────┐
    │ Bevy Window │   │ Iced Window │
    │    (3D)     │   │    (2D)     │
    └─────────────┘   └─────────────┘
```

## Components

### 1. Main Shell (`alchemist`)
- Manages renderer processes
- Decides which renderer to use based on data type
- Handles IPC with renderer windows
- Tracks active windows

### 2. Renderer Binary (`alchemist-renderer`)
- Single binary that can launch either Bevy or Iced
- Receives render requests via temporary files
- Runs as separate process for isolation

### 3. Bevy Renderer (3D)
Best for:
- 3D graph visualizations
- Workflow diagrams with spatial layout
- Interactive 3D scenes
- Complex data relationships

### 4. Iced Renderer (2D)
Best for:
- Document viewing (Markdown, HTML)
- Text editing
- Video/audio playback
- Form-based UIs
- Simple 2D visualizations

## Usage

### From CLI

```bash
# Launch a 3D graph
alchemist render graph --title "My Graph"

# Open a document
alchemist render document README.md

# Launch text editor
alchemist render edit myfile.rs

# List active windows
alchemist render list

# Close a window
alchemist render close <id>
```

### From Interactive Shell

```bash
alchemist --interactive

# In the shell:
render demo graph3d     # 3D graph demo
render demo document    # Document viewer demo
render list            # List active windows
```

### Programmatic Usage

```rust
// Spawn a 3D graph
let id = renderer_manager.spawn_graph_3d(
    "Network Visualization",
    nodes,
    edges
).await?;

// Spawn a document viewer
let id = renderer_manager.spawn_document(
    "User Manual",
    markdown_content,
    "markdown"
).await?;

// Update data in real-time
renderer_manager.update_data(&id, new_data).await?;
```

## Data Types and Renderer Selection

| Data Type | Preferred Renderer | Reason |
|-----------|-------------------|---------|
| Graph3D | Bevy | 3D spatial visualization |
| Workflow | Bevy | Complex node relationships |
| Document | Iced | Text rendering, scrolling |
| TextEditor | Iced | Native text input |
| Video | Iced | Media playback controls |
| Audio | Iced | UI controls, playlists |
| Scene3D | Bevy | 3D rendering required |

## Communication

Currently uses file-based IPC:
1. Shell creates temp file with render request
2. Spawns renderer process with file path
3. Renderer reads and deletes temp file
4. Future: Add bidirectional IPC for updates

## Future Enhancements

1. **Real-time Updates**: Implement IPC channels for live data updates
2. **Window Management**: Add docking, splitting, and layout management
3. **Shared State**: Allow windows to communicate with each other
4. **Custom Renderers**: Plugin system for domain-specific visualizations
5. **Export**: Save visualizations as images or videos
6. **Performance**: GPU-accelerated compute for large graphs

## Example: Workflow Visualization

```rust
// In the shell
let workflow_data = load_workflow("order-processing.json")?;

// Spawn 3D visualization
let id = renderer_manager.spawn_workflow(
    "Order Processing Workflow",
    workflow_id,
    workflow_data,
    true  // use 3D
).await?;

// User can interact with the 3D workflow
// - Rotate camera with mouse
// - Click nodes for details
// - Animate execution flow
```

## Building

```bash
# Build everything
cargo build --release

# Run demos
./demo_renderer.sh
```

This architecture provides the flexibility to use the best visualization tool for each type of data while maintaining a clean separation between the control system (shell) and the visualization layer (renderers).