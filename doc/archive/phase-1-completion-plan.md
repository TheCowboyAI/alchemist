# Phase 1 Completion Plan

## Overview

Before proceeding to Phase 2 (Selection System), we need to complete several TODOs and implement functionality that was marked as "future work" in Phase 1. This plan outlines the necessary work to ensure Phase 1 is fully functional.

## Identified TODOs and Incomplete Features

### 1. Graph Domain Validation Rules
**Location**: `src/contexts/graph_management/services.rs:173,187`
**Priority**: High
**Description**: The ValidateGraph service has placeholder TODOs for domain rules implementation.

### 2. Node Selection Raycasting
**Location**: `src/contexts/visualization/services.rs:668`
**Priority**: Critical (blocks Phase 2)
**Description**: Raycasting implementation for mouse selection is required for Phase 2.

### 3. Render Mode Implementations
**Priority**: Medium
**Description**: Several render modes are partially implemented:
- Point Cloud rendering (requires dedicated plugin)
- Billboard rendering (requires custom shader)
- Wireframe rendering (needs proper wireframe shader)

## Implementation Tasks

### Task 1: Complete Graph Validation Rules
**Estimated Time**: 4 hours

#### 1.1 Define Domain Constraints
```rust
// src/contexts/graph_management/services.rs

impl ValidateGraph {
    /// Validates that an operation is allowed
    pub fn can_add_node(&self, graph_id: GraphIdentity) -> Result<(), GraphConstraintViolation> {
        // Implement:
        // - Check if graph exists
        // - Check if graph is not deleted
        // - Check node count limits (e.g., max 10,000 nodes)
        // - Check if graph is not locked
    }

    pub fn can_connect_nodes(
        &self,
        _graph: GraphIdentity,
        _source: NodeIdentity,
        _target: NodeIdentity,
    ) -> Result<(), GraphConstraintViolation> {
        // Implement:
        // - Check if nodes exist
        // - Check if nodes are in same graph
        // - Prevent self-loops (optional)
        // - Check edge count limits
        // - Prevent duplicate edges (optional)
    }
}
```

#### 1.2 Implement GraphConstraintViolation Variants
```rust
pub enum GraphConstraintViolation {
    GraphNotFound,
    GraphDeleted,
    NodeLimitExceeded { limit: usize, current: usize },
    GraphLocked,
    NodeNotFound(NodeIdentity),
    NodesInDifferentGraphs,
    SelfLoopNotAllowed,
    EdgeLimitExceeded { limit: usize, current: usize },
    DuplicateEdgeNotAllowed,
}
```

### Task 2: Implement Raycasting for Selection
**Estimated Time**: 6 hours

#### 2.1 Create Raycasting Service
```rust
// src/contexts/visualization/services.rs

pub struct PerformRaycast;

impl PerformRaycast {
    /// Converts screen coordinates to ray
    pub fn screen_to_ray(
        window: &Window,
        camera: &Camera,
        camera_transform: &GlobalTransform,
        screen_pos: Vec2,
    ) -> Option<Ray3d> {
        // Implementation
    }

    /// Checks ray-sphere intersection
    pub fn ray_intersects_sphere(
        ray: &Ray3d,
        sphere_center: Vec3,
        sphere_radius: f32,
    ) -> Option<f32> {
        // Implementation
    }
}
```

#### 2.2 Update HandleUserInput::process_selection
```rust
impl HandleUserInput {
    pub fn process_selection(
        windows: Query<&Window>,
        camera: Query<(&Camera, &GlobalTransform)>,
        nodes: Query<(Entity, &Transform, &NodeIdentity), With<Node>>,
        mouse_button: Res<ButtonInput<MouseButton>>,
        mut events: EventWriter<NodeSelected>,
    ) {
        // Implement using raycasting
    }
}
```

#### 2.3 Add Selection Events
```rust
#[derive(Event, Debug, Clone)]
pub struct NodeSelected {
    pub entity: Entity,
    pub node: NodeIdentity,
}

#[derive(Event, Debug, Clone)]
pub struct NodeDeselected {
    pub entity: Entity,
    pub node: NodeIdentity,
}
```

### Task 3: Complete Render Mode Implementations
**Estimated Time**: 8 hours

#### 3.1 Wireframe Rendering
```rust
// Add proper wireframe material
fn create_wireframe_material(
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.5, 0.9),
        unlit: true,
        cull_mode: None,
        alpha_mode: AlphaMode::Blend,
        // Note: Bevy 0.16 may need custom shader for true wireframe
        ..default()
    })
}
```

#### 3.2 Point Cloud Plugin Stub
```rust
// src/contexts/visualization/point_cloud.rs (new file)

/// Plugin for point cloud rendering
pub struct PointCloudPlugin;

impl Plugin for PointCloudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_point_clouds);
    }
}

fn render_point_clouds(
    point_clouds: Query<(&NodePointCloud, &Transform)>,
    mut gizmos: Gizmos,
) {
    // Simple implementation using gizmos
    for (cloud, transform) in point_clouds.iter() {
        for (i, point) in cloud.points.iter().enumerate() {
            let world_pos = transform.transform_point(*point);
            let color = cloud.colors.get(i).copied().unwrap_or(Color::WHITE);
            let size = cloud.sizes.get(i).copied().unwrap_or(0.02);

            gizmos.sphere(world_pos, size, color);
        }
    }
}
```

#### 3.3 Billboard Rendering Stub
```rust
// Simple billboard implementation using sprites
fn create_billboard_node(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
    label: &str,
) {
    commands.spawn((
        Sprite {
            image: asset_server.load("textures/node_billboard.png"),
            ..default()
        },
        Transform::from_translation(position),
        // Add billboard behavior component
        Billboard,
    ));
}

#[derive(Component)]
pub struct Billboard;

// System to make billboards face camera
pub fn update_billboards(
    camera: Query<&Transform, With<Camera3d>>,
    mut billboards: Query<&mut Transform, (With<Billboard>, Without<Camera3d>)>,
) {
    if let Ok(camera_transform) = camera.get_single() {
        for mut transform in billboards.iter_mut() {
            transform.look_at(camera_transform.translation, Vec3::Y);
        }
    }
}
```

### Task 4: Integration and Testing
**Estimated Time**: 4 hours

#### 4.1 Integration Tasks
1. Add validation calls to graph services
2. Wire up raycasting to existing mouse input
3. Add render mode switching for individual entities
4. Update plugin to include new systems

#### 4.2 Test Scenarios
1. Test graph validation limits
2. Test node selection accuracy
3. Test render mode switching
4. Performance test with 100+ nodes

## Implementation Order

1. **Week 1, Days 1-2**: Graph Validation Rules
   - Define constraints
   - Implement validation logic
   - Add tests

2. **Week 1, Days 3-4**: Raycasting Implementation
   - Create raycasting utilities
   - Update selection system
   - Test selection accuracy

3. **Week 1, Day 5 - Week 2, Day 1**: Render Modes
   - Implement wireframe properly
   - Create point cloud plugin
   - Add billboard support

4. **Week 2, Days 2-3**: Integration and Testing
   - Integrate all systems
   - Performance testing
   - Bug fixes

## Success Criteria

### Must Have (Blocking Phase 2)
- [x] Raycasting selection works accurately
- [x] Graph validation prevents invalid operations
- [x] All render modes at least minimally functional

### Should Have
- [x] Wireframe mode looks correct
- [x] Point clouds render (even if basic)
- [x] Performance maintained at 60 FPS

### Could Have
- [ ] Advanced point cloud shaders
- [ ] Sophisticated billboard rendering
- [ ] Animated render mode transitions

## Risk Mitigation

1. **Raycasting Complexity**: Start with simple sphere intersection, optimize later
2. **Shader Limitations**: Use Bevy's built-in features where possible
3. **Performance**: Profile early and often, especially with point clouds

## Documentation Updates

After implementation:
1. Update vocabulary.md with new terms
2. Document keyboard/mouse controls
3. Add examples for each render mode
4. Update architecture diagrams

## Next Steps

Once this plan is complete:
1. Run full QA validation
2. Update Phase 1 documentation
3. Begin Phase 2: Selection System

---

*Estimated Total Time*: 22 hours (approximately 3-4 days of focused work)
