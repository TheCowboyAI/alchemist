# Phase 2 Preparation Plan

## Overview

Based on the Phase 1 Quality Assurance Report, this plan addresses the identified issues and prepares the codebase for Phase 2: Selection System implementation.

## Immediate Actions Required

### 1. Add Selection Visual Feedback
**Priority**: High
**Estimated Time**: 2 hours

**Implementation**:
```rust
// Add to visualization services
#[derive(Component)]
pub struct Selected;

// System to highlight selected nodes
pub fn highlight_selected_nodes(
    mut materials: ResMut<Assets<StandardMaterial>>,
    selected: Query<&MeshMaterial3d<StandardMaterial>, With<Selected>>,
    not_selected: Query<&MeshMaterial3d<StandardMaterial>, Without<Selected>>,
) {
    // Apply highlight material to selected nodes
    // Dim or desaturate non-selected nodes
}
```

### 2. Document Keyboard Controls
**Priority**: Medium
**Estimated Time**: 30 minutes

Create user documentation for:
- Number keys 1-4: Change edge types
- M, P, W, B keys: Change render modes
- Arrow keys: Orbit camera
- Mouse left click: Select node

### 3. Implement Edge Animation ��
**Priority**: High
**Estimated Time**: 4-6 hours

**Missing Feature Discovered**: Edge animation is completely missing!

**Implementation Plan**:
```rust
// Add edge animation components
#[derive(Component)]
pub struct EdgePulse {
    pub pulse_speed: f32,
    pub pulse_intensity: f32,
    pub color_variation: f32,
}

#[derive(Component)]
pub struct EdgeFlow {
    pub flow_speed: f32,
    pub particle_density: f32,
    pub particle_size: f32,
}

// Add animation system to AnimateGraphElements
impl AnimateGraphElements {
    pub fn animate_edges(
        time: Res<Time>,
        mut edges: Query<(&mut Transform, &EdgePulse), With<Edge>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        // Implement edge pulsing, flowing, color cycling
    }
}
```

### 4. Fix Selection Visual Feedback Loop
**Priority**: Low
**Estimated Time**: 1 hour

The current selection only emits events but doesn't update visual state. Add a system to consume `NodeSelected` events and update materials.

## Phase 2 Core Implementation

### 1. Multi-Selection Support
- Ctrl+Click for adding to selection
- Shift+Click for range selection
- Drag box selection

### 2. Selection State Management
- Maintain selection set
- Provide selection queries
- Handle selection clearing

### 3. Selection Visualization
- Outline/glow effects
- Color highlighting
- Selection box rendering

### 4. Edge Selection
- Implement edge raycasting
- Edge hover effects
- Edge selection feedback

## Technical Debt from Phase 1

1. **Ray3d Type Issues**: Consider creating wrapper types for cleaner raycasting API
2. **Event System Migration**: Complete migration to `write()` pattern
3. **Result Handling**: Add proper error handling for systems

## Success Criteria

- [ ] All nodes and edges can be selected
- [ ] Visual feedback clearly shows selection state
- [ ] Multi-selection works intuitively
- [ ] Edge animation enhances visualization
- [ ] Keyboard/mouse controls are documented
- [ ] All tests pass including new selection tests

## Testing Requirements

1. **Unit Tests**:
   - Selection raycasting accuracy
   - Multi-selection state management
   - Edge animation components

2. **Integration Tests**:
   - Full selection workflow
   - Visual feedback updates
   - Animation performance

## Timeline

- Week 1: Edge animation + selection visual feedback
- Week 2: Multi-selection implementation
- Week 3: Edge selection + testing
- Week 4: Documentation + polish

## Phase 2 Readiness Checklist

### Selection System Foundation
- [x] Raycasting infrastructure
- [x] Selection events (NodeSelected/NodeDeselected)
- [ ] Visual feedback system
- [ ] Selection state management
- [ ] Multi-selection support

### Performance Monitoring
- [ ] Add metrics collection for raycasting
- [ ] Graph size benchmarks
- [ ] Frame time monitoring
- [ ] Memory usage tracking

### Documentation Updates
- [ ] User guide for controls
- [ ] Developer guide for selection system
- [ ] API documentation for new services
- [ ] Architecture diagrams update

## Technical Debt Items

### 1. Selection State Management
Create a proper selection state resource:
```rust
#[derive(Resource, Default)]
pub struct SelectionState {
    pub selected_nodes: HashSet<NodeIdentity>,
    pub selected_edges: HashSet<EdgeIdentity>,
    pub selection_mode: SelectionMode,
}

#[derive(Default, PartialEq)]
pub enum SelectionMode {
    #[default]
    Single,
    Multiple,
    Box,
}
```

### 2. Performance Monitoring
Implement fitness function monitoring:
```rust
#[derive(Resource)]
pub struct PerformanceMetrics {
    pub frame_time: MovingAverage,
    pub raycast_time: MovingAverage,
    pub node_count: usize,
    pub edge_count: usize,
}
```

## Phase 2 Architecture Preparation

### Selection Context Structure
```
src/contexts/selection/
├── mod.rs
├── plugin.rs
├── domain.rs      // SelectionSet, SelectionBounds
├── events.rs      // SelectionChanged, SelectionCleared
├── services.rs    // ManageSelection, HighlightSelection
└── systems.rs     // Input handling, visual feedback
```

### Integration Points
1. **Graph Management**: Query node/edge existence
2. **Visualization**: Apply selection highlighting
3. **User Input**: Handle modifier keys (Shift/Ctrl)

## Implementation Schedule

### Week 1: Foundation
- Day 1-2: Selection visual feedback
- Day 3: Documentation updates
- Day 4-5: Performance monitoring setup

### Week 2: Phase 2 Start
- Begin selection system implementation
- Multi-selection support
- Box selection preparation

## Success Metrics

1. **Visual Feedback**: Selected nodes clearly distinguishable
2. **Performance**: Selection operations < 1ms for 1000 nodes
3. **Documentation**: 100% coverage of user-facing features
4. **Code Quality**: Zero linter warnings (excluding allowed dead code)

## Risk Mitigation

1. **Performance Degradation**: Profile before and after changes
2. **Breaking Changes**: Maintain backward compatibility
3. **Complexity Growth**: Keep selection logic separate from rendering

## Next Steps

1. Implement selection visual feedback
2. Create user documentation
3. Set up performance monitoring
4. Begin Phase 2 implementation

---

*Prepared by*: Quality Assurance Assistant
*Target*: Phase 2 Selection System
*Timeline*: 1 week preparation + Phase 2 start
