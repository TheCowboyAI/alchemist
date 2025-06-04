# Phase 2 Selection System Remediation Plan

## Overview

This plan addresses the critical issues preventing Phase 2 from being production-ready. All tasks must be completed before moving to Phase 3.

## Priority 1: Fix Test Infrastructure (2-3 hours)

### Task 1.1: Isolate Tests from Render Dependencies

**Problem**: Tests fail to link due to Bevy render dependencies.

**Solution**:
```rust
// Create new test utilities module
// src/contexts/selection/test_utils.rs

use bevy::prelude::*;
use bevy::app::PluginGroupBuilder;

pub fn create_test_app() -> App {
    let mut app = App::new();

    // Use minimal plugins instead of DefaultPlugins
    app.add_plugins(MinimalPlugins.set(TaskPoolPlugin {
        task_pool_options: TaskPoolOptions::with_num_threads(1),
    }));

    // Add only ECS-related plugins
    app.add_plugins((
        TransformPlugin,
        HierarchyPlugin,
    ));

    app
}
```

### Task 1.2: Update All Tests

Replace all `App::new()` calls with `create_test_app()` and ensure no render-related components are used.

### Task 1.3: Create Headless Test Runner

```bash
# Add to .cargo/config.toml
[env]
BEVY_HEADLESS = "1"
WGPU_BACKEND = "noop"
```

## Priority 2: DDD Event Naming Migration (1-2 hours)

### Task 2.1: Rename Events to Past Tense

| Current Name | New Name |
|--------------|----------|
| SelectNode | NodeSelected |
| DeselectNode | NodeDeselected |
| SelectEdge | EdgeSelected |
| DeselectEdge | EdgeDeselected |
| SelectAll | AllSelected |
| InvertSelection | SelectionInverted |
| StartBoxSelection | BoxSelectionStarted |
| UpdateBoxSelection | BoxSelectionUpdated |
| CompleteBoxSelection | BoxSelectionCompleted |

### Task 2.2: Update Event Handlers

Update all event readers/writers to use new names:
- ManageSelection service methods
- AdvancedSelection service methods
- ProcessSelectionInput service methods

### Task 2.3: Update Tests

Update all test cases to use new event names.

## Priority 3: Code Cleanup (1-2 hours)

### Task 3.1: Remove Unused Imports

Files to clean:
- `src/contexts/selection/tests.rs` - Remove `EdgeVisual`, `HashMap`
- `src/contexts/selection/mod.rs` - Remove `SelectionPlugin` export
- `src/contexts/graph_management/tests.rs` - Remove unused imports
- `src/contexts/visualization/tests.rs` - Remove unused imports

### Task 3.2: Fix Dead Code Warnings

1. Mark `OriginalMaterial` field as used or remove:
```rust
#[derive(Component)]
pub struct OriginalMaterial(#[allow(dead_code)] pub Handle<StandardMaterial>);
```

2. Remove or implement `Lasso` selection mode:
```rust
pub enum SelectionMode {
    Single,
    Multiple,
    Box,
    // Lasso, // Remove or implement
}
```

### Task 3.3: Fix Mutable Variable Warnings

Review all `let mut` declarations and remove `mut` where not needed.

## Priority 4: Complete Missing Features (2-3 hours)

### Task 4.1: Implement Box Selection Visualization

```rust
// Add to PerformBoxSelection service
pub fn render_selection_box(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    selection_box: Res<SelectionBox>,
) {
    if !selection_box.active {
        return;
    }

    // Create semi-transparent box mesh
    let mesh = create_box_mesh(&selection_box);
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 0.5, 0.8, 0.3),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        SelectionBoxVisual,
    ));
}
```

### Task 4.2: Improve Hover Effects

Enhance hover visual feedback with scaling or glow effects.

## Validation Criteria

### Tests Must Pass
```bash
# All of these must succeed
BEVY_HEADLESS=1 cargo test --package ia contexts::selection
cargo clippy -- -D warnings
cargo fmt --check
```

### DDD Compliance Check
- All events use past tense
- All services use verb phrases
- No technical suffixes unless domain terms

### Code Quality Metrics
- Zero compiler warnings
- Zero clippy warnings
- Test coverage maintained above 80%

## Implementation Order

1. **Day 1**: Fix test infrastructure (Priority 1)
   - Morning: Create test utilities
   - Afternoon: Update tests and verify they run

2. **Day 2**: DDD compliance and cleanup (Priority 2 & 3)
   - Morning: Event renaming migration
   - Afternoon: Code cleanup and warning fixes

3. **Day 3**: Complete features and final validation (Priority 4)
   - Morning: Implement missing features
   - Afternoon: Run full validation suite

## Success Criteria

- [ ] All tests pass in headless mode
- [ ] Zero compiler warnings
- [ ] DDD naming compliance achieved
- [ ] Box selection visualization working
- [ ] Documentation updated
- [ ] Ready for Phase 3

## Risk Mitigation

1. **Test Breakage**: Keep old test setup as fallback
2. **Event Migration**: Use type aliases during transition
3. **Feature Scope**: Defer Lasso selection to Phase 3

## Next Steps After Completion

1. Update `phase-2-selection-system-complete.md` with fixes
2. Create PR with all changes
3. Run full integration test suite
4. Begin Phase 3 planning
