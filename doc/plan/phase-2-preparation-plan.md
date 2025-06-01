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
**Estimated Time**: 1 hour

Create `doc/user-guide/keyboard-controls.md`:
- Edge type switching (1-4)
- Render mode switching (M/P/W/B)
- Camera controls (←/→)
- Selection (Left Click)

### 3. Dead Code Annotations
**Priority**: Low
**Estimated Time**: 30 minutes

Add appropriate annotations to domain entities:
```rust
#[allow(dead_code)] // Used for event sourcing and future features
pub struct Graph {
    pub identity: GraphIdentity,
    pub metadata: GraphMetadata,
    pub journey: GraphJourney,
}
```

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
