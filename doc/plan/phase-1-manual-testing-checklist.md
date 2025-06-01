# Phase 1 Manual Testing Checklist - EXACT INSTRUCTIONS

## Prerequisites

1. **Build the application**: `nix build`
2. **Run the application**: `nix run`
3. **Verify window appears** with 3D scene visible
4. **Click on the window** to ensure it has keyboard focus

## Testing Checklist

### 1. Basic Rendering

- [ ] **START**: Run `nix run` from terminal
- [ ] **VERIFY**: Application window opens without errors in console
- [ ] **LOOK FOR**:
  - 3D camera view with dark background
  - Three nodes labeled "Rust", "Bevy", "ECS" (auto-created on startup)
  - Two edges connecting them
- [ ] **NOTE**: If no nodes visible, check console for errors

### 2. Graph Creation (Automatic on Startup)

- [ ] **OBSERVE**: Console shows "Example graph created with DDD-compliant code!"
- [ ] **VERIFY**: Three nodes appear in 3D space:
  - "Rust" node at left (-2, 0, 0)
  - "Bevy" node at right (2, 0, 0)
  - "ECS" node at top (0, 2, 0)
- [ ] **VERIFY**: Two edges connect:
  - Rust → Bevy (labeled "powers")
  - Bevy → ECS (labeled "implements")
- [ ] **⚠️ NOTE**: No UI for creating new graphs yet - only example graph

### 3. Node Operations

- [ ] **OBSERVE**: Blue spheres represent nodes
- [ ] **VERIFY**: Each node has a text label
- [ ] **⚠️ NOTE**: No UI for adding new nodes yet - only pre-created nodes

### 4. Edge Operations

- [ ] **OBSERVE**: Gray cylinders connect nodes
- [ ] **VERIFY**: Edges properly connect source to target nodes
- [ ] **⚠️ NOTE**: No UI for creating new edges yet - only pre-created edges

### 5. Render Modes - EXACT KEY PRESSES

**IMPORTANT**: Window must have focus. Click on the window first!

- [ ] **PRESS 'M'**: Mesh mode (default)
  - Nodes appear as solid blue spheres
  - This is the default mode

- [ ] **PRESS 'P'**: Point Cloud mode
  - ⚠️ **WARNING**: May not visually change - point cloud rendering not fully implemented
  - Console may show mode change but visuals stay the same

- [ ] **PRESS 'W'**: Wireframe mode
  - Nodes should appear as wireframe spheres
  - May show emissive glow effect

- [ ] **PRESS 'B'**: Billboard mode
  - Node labels should always face camera
  - Text orientation updates as camera moves

### 6. Edge Types - EXACT KEY PRESSES

**IMPORTANT**: Window must have focus. Click on the window first!

- [ ] **PRESS '1'**: Line edges
  - Edges appear as thin lines
  - ⚠️ May not be implemented - check if edges disappear

- [ ] **PRESS '2'**: Cylinder edges (default)
  - Edges appear as 3D cylinders
  - This is the default mode

- [ ] **PRESS '3'**: Arc edges
  - Edges should curve upward
  - ⚠️ May show as cylinders if not implemented

- [ ] **PRESS '4'**: Bezier edges
  - Edges should show smooth curves
  - ⚠️ May show as cylinders if not implemented

### 7. Camera Controls - EXACT KEY PRESSES

**IMPORTANT**: Window must have focus. Click on the window first!

- [ ] **HOLD LEFT ARROW**: Camera orbits left around origin
  - Camera should continuously rotate counterclockwise
  - Release to stop

- [ ] **HOLD RIGHT ARROW**: Camera orbits right around origin
  - Camera should continuously rotate clockwise
  - Release to stop

- [ ] **⚠️ NOTE**: Only left/right arrows work. Up/down not implemented
- [ ] **⚠️ NOTE**: No zoom, pan, or other camera controls yet

### 8. Selection System - EXACT MOUSE ACTIONS

- [ ] **LEFT CLICK on a node**:
  1. Move mouse cursor over any blue sphere
  2. Click left mouse button once
  3. **VERIFY**:
     - Node changes to golden/yellow color
     - Node has emissive glow effect
     - Console shows "Node [ID] selected"

- [ ] **LEFT CLICK on empty space**:
  1. Click anywhere without a node
  2. **VERIFY**: Nothing happens (selection unchanged)

- [ ] **RIGHT CLICK anywhere**:
  1. Click right mouse button anywhere in window
  2. **VERIFY**:
     - ALL selected nodes return to original blue color
     - Console shows "Deselected all nodes"
     - Works even if clicking on a node

### 9. Animation System

#### Graph Animation
- [ ] **⚠️ NOT AUTOMATIC**: Graphs don't auto-rotate
- [ ] **⚠️ NO ANIMATION**: Must add GraphMotion component manually

#### Node Animation
- [ ] **⚠️ NOT VISIBLE**: Nodes don't pulse by default
- [ ] **⚠️ NO ANIMATION**: Must add NodePulse component manually

#### Edge Animation
- [ ] **OBSERVE**: ~30% of edges should animate
- [ ] **LOOK FOR**:
  - Edges pulsing (scale up/down)
  - Edges waving (vertical motion)
  - ⚠️ Color cycling may not be visible
- [ ] **⚠️ NOTE**: Animation assignment is random, may vary between runs

### 10. Performance Checks

- [ ] **VERIFY**: Smooth camera rotation (no stuttering)
- [ ] **VERIFY**: Instant response to key presses (<100ms)
- [ ] **VERIFY**: Selection response is immediate
- [ ] **CHECK**: Console for any error messages
- [ ] **⚠️ NOTE**: No FPS counter visible

## Troubleshooting

### Nothing happens when I press keys
1. **CLICK on the window** to give it focus
2. Check console for errors
3. Verify window is not minimized

### Can't see any nodes
1. Check console for "Example graph created" message
2. Try pressing arrow keys to rotate camera
3. Nodes might be behind camera

### Selection not working
1. Make sure to click directly on a node (blue sphere)
2. Cursor must be over the node mesh, not just near it
3. Check console for selection messages

### Modes don't change visually
- Some modes (Point Cloud, Arc, Bezier) may not be fully implemented
- Check console messages to confirm mode changes are registered
- File a bug report if mode should work but doesn't

## Expected vs Actual Behavior

| Feature | Expected | Actual Status |
|---------|----------|---------------|
| Node Creation UI | ❌ Not implemented | Example only |
| Edge Creation UI | ❌ Not implemented | Example only |
| Point Cloud Rendering | ⚠️ May not work | Mode changes, no visual |
| Arc/Bezier Edges | ⚠️ May not work | Shows as cylinders |
| Automatic Graph Rotation | ❌ Not implemented | Manual only |
| Node Pulse Animation | ❌ Not by default | Requires component |
| Up/Down Camera | ❌ Not implemented | Left/Right only |
| Zoom Controls | ❌ Not implemented | Fixed distance |

## Success Criteria

Phase 1 is complete when:
- ✅ Example graph loads and displays
- ✅ All implemented keyboard controls respond
- ✅ Mouse selection works on nodes
- ✅ Right-click deselects all
- ✅ Camera orbits with arrow keys
- ✅ No crashes or panics during testing
- ✅ Console shows expected event messages
