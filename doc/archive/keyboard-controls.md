# Graph Editor Keyboard and Mouse Controls

## Overview

This document describes all available keyboard and mouse controls for the Information Alchemist graph editor.

## Mouse Controls

### Selection
- **Left Click**: Select a node
- **Right Click**: Deselect all nodes
- **Drag** (coming in Phase 2): Box selection of multiple nodes

### Camera
- **Mouse Drag**: Rotate camera around the graph
- **Mouse Wheel**: Zoom in/out

## Keyboard Controls

### Edge Type Selection
Change the visual style of newly created edges:

- **1**: Line edges (simple straight lines)
- **2**: Cylinder edges (3D cylinders) - Default
- **3**: Arc edges (curved arcs)
- **4**: Bezier edges (smooth bezier curves)

### Render Mode Selection
Change how nodes are rendered:

- **M**: Mesh mode (3D spheres) - Default
- **P**: Point Cloud mode (particle effects) - Requires plugin
- **W**: Wireframe mode (mesh outlines)
- **B**: Billboard mode (bright colored spheres, graph rotation paused)

### Camera Controls
- **Arrow Left**: Orbit camera left around the graph
- **Arrow Right**: Orbit camera right around the graph

## Visual Feedback

### Node Selection
- Selected nodes display a golden highlight with emissive glow
- Material properties: Golden color, metallic finish, slight emission

### Billboard Mode
- Nodes appear as brightly colored spheres (smaller than normal)
- Colors are assigned based on node label for easy identification:
  - Red, Green, Blue, Yellow, or Magenta
- All graph rotation animations are paused in this mode
- Useful for focusing on graph structure without distracting animations

### Edge Animations
Currently, edges have various animated effects applied randomly:

- **30%** - Pulse animation (thickness and brightness pulsing)
- **20%** - Wave animation (undulating motion)
- **20%** - Color cycling (smooth color transitions)
- **15%** - Flow animation (directional flow indicators)
- **15%** - No animation (static edges)

## Tips

- Press **B** to use billboard mode when you need to focus on the graph structure
- Press **M** to return to normal mesh rendering with animations
- Right-click anywhere to deselect all nodes
- Edge animations add visual interest but can be performance-intensive with many edges

## Coming in Phase 2

- Multi-selection with Ctrl+Click and Shift+Click
- Box selection by dragging
- Edge selection and highlighting
- Improved camera controls with Bevy Panorbit Camera
- Proper 3D text labels for billboard mode
- Performance optimizations for large graphs
- 2D visualization mode in a separate module

## Troubleshooting

If controls are not responding:
1. Ensure the game window has focus (click on it)
2. Check that no UI panels are blocking input
3. Verify that the graph has been properly initialized

---

*Last updated*: Phase 1 Completion
*Version*: 0.1.0
