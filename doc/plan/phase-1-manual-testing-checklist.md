# Phase 1 Manual Testing Checklist

## Overview

This checklist ensures all Phase 1 functionality works correctly when running the application.

## Prerequisites

- Build and run the application: `nix run`
- Have a visible window with 3D scene
- Can interact with mouse and keyboard

## Testing Checklist

### 1. Basic Rendering ✓

- [ ] Application starts without errors
- [ ] 3D camera view is visible
- [ ] Lighting is appropriate (not too dark/bright)
- [ ] Default grid or axes visible (if implemented)

### 2. Graph Creation

- [ ] Create a graph programmatically (check console for "Graph created" messages)
- [ ] Graph entity appears in scene
- [ ] Graph can be rotated (if animation enabled)

### 3. Node Operations

- [ ] Add nodes to graph (check console for "Visualizing node" messages)
- [ ] Nodes appear as blue spheres (default)
- [ ] Nodes are positioned correctly in 3D space
- [ ] Node labels visible in Billboard mode

### 4. Edge Operations

- [ ] Connect nodes with edges (check console for "Visualizing edge" messages)
- [ ] Edges appear as cylinders (default)
- [ ] Edges connect correct nodes
- [ ] No self-loops visible

### 5. Render Modes (Keyboard: M, P, W, B)

- [ ] **M key**: Mesh mode - Solid spheres for nodes
- [ ] **P key**: Point cloud mode - Points instead of meshes
- [ ] **W key**: Wireframe mode - Wireframe spheres with emissive
- [ ] **B key**: Billboard mode - Text labels facing camera

### 6. Edge Types (Keyboard: 1, 2, 3, 4)

- [ ] **1 key**: Line edges - Thin lines
- [ ] **2 key**: Cylinder edges - 3D cylinders
- [ ] **3 key**: Arc edges - Curved arcs
- [ ] **4 key**: Bezier edges - Smooth curves

### 7. Camera Controls (Arrow keys)

- [ ] **Left Arrow**: Orbit camera left around origin
- [ ] **Right Arrow**: Orbit camera right around origin
- [ ] Camera maintains focus on scene center

### 8. Selection System (Mouse)

- [ ] **Left Click on Node**:
  - Node changes to golden color
  - Emissive glow effect visible
  - Console shows "Node selected" message
- [ ] **Right Click**:
  - All selected nodes deselect
  - Original colors restored
  - Console shows "Deselected all nodes" message
- [ ] **Left Click on Empty Space**: No selection change

### 9. Animation System

#### Graph Animation
- [ ] Graphs rotate continuously (if enabled)
- [ ] Oscillation works (if configured)
- [ ] Scale animation works (if configured)

#### Node Animation
- [ ] Nodes bounce (if NodePulse component added)
- [ ] Nodes pulse/scale (if NodePulse component added)

#### Edge Animation (NEW!)
- [ ] Some edges pulse (scale animation)
- [ ] Some edges wave (vertical motion)
- [ ] Some edges cycle colors (if materials update)
- [ ] ~30% of edges have animation

### 10. Performance

- [ ] Smooth 60 FPS with 10-20 nodes
- [ ] No stuttering during animations
- [ ] Selection response < 100ms
- [ ] Mode switching is instant

## Known Issues & Limitations

1. **Point Cloud Mode**: Requires point cloud plugin for actual rendering
2. **Color Cycling**: Material updates may not be visible without separate system
3. **Keyboard Controls**: May require window focus
4. **Edge Animation**: Material-based effects (emissive, color) simplified

## Success Criteria

Phase 1 is complete when:
- ✅ All render modes work visually
- ✅ All edge types display correctly
- ✅ Keyboard controls function
- ✅ Selection shows visual feedback
- ✅ All animations work (including edges)
- ✅ No runtime panics
- ✅ Performance is acceptable
