# Camera and Viewport Technical Specification

## Overview

This document specifies the implementation details for the dual-mode camera system that enables seamless switching between 3D orbital and 2D top-down views within a single Bevy viewport. The camera system is designed to work with the dual-layer graph architecture, operating on the visualization layer while being independent of the graph data layer.

## Integration with Dual-Layer Architecture

The camera system operates exclusively on the **Visualization Layer** (Bevy ECS), rendering entities that are linked to graph nodes/edges via the GraphData resource. This separation ensures:

- Camera controls work independently of graph topology
- Rendering performance is not affected by graph complexity
- View transformations don't require graph data updates
- Multiple camera views could render the same graph data

## Camera Architecture

### Component Definitions

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct GraphViewCamera {
    pub view_mode: ViewMode,
    pub transition: CameraTransition,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ViewMode {
    ThreeD(ThreeDState),
    TwoD(TwoDState),
}

#[derive(Clone, Copy, PartialEq)]
pub struct ThreeDState {
    pub focus_point: Vec3,
    pub orbit_radius: f32,
    pub azimuth: f32,      // Horizontal rotation (0-2π)
    pub elevation: f32,     // Vertical rotation (-π/2 to π/2)
}

#[derive(Clone, Copy, PartialEq)]
pub struct TwoDState {
    pub center: Vec2,
    pub zoom_level: f32,    // Orthographic scale
    pub fixed_height: f32,  // Y position for top-down view
}

#[derive(Component)]
pub struct CameraTransition {
    pub active: bool,
    pub from_mode: ViewMode,
    pub to_mode: ViewMode,
    pub progress: f32,
    pub duration: f32,
}
```

### Viewport Configuration

```rust
#[derive(Resource)]
pub struct ViewportConfig {
    pub main_viewport: ViewportRect,
    pub tools_panel_width: f32,
    pub aspect_ratio: f32,
}

#[derive(Clone, Copy)]
pub struct ViewportRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
```

## Camera Systems

### 1. Camera Update System

```rust
fn update_camera_system(
    mut cameras: Query<(&mut Transform, &mut Projection, &GraphViewCamera)>,
    time: Res<Time>,
) {
    for (mut transform, mut projection, view_camera) in &mut cameras {
        match view_camera.view_mode {
            ViewMode::ThreeD(state) => {
                update_3d_camera(&mut transform, &mut projection, state);
            }
            ViewMode::TwoD(state) => {
                update_2d_camera(&mut transform, &mut projection, state);
            }
        }
    }
}
```

### 2. Camera Transition System

```rust
fn camera_transition_system(
    mut cameras: Query<(&mut GraphViewCamera, &mut Transform, &mut Projection)>,
    time: Res<Time>,
) {
    for (mut view_camera, mut transform, mut projection) in &mut cameras {
        if view_camera.transition.active {
            // Update transition progress
            view_camera.transition.progress += time.delta_seconds() / view_camera.transition.duration;

            if view_camera.transition.progress >= 1.0 {
                // Complete transition
                view_camera.view_mode = view_camera.transition.to_mode;
                view_camera.transition.active = false;
            } else {
                // Interpolate between modes
                interpolate_camera_modes(
                    &mut transform,
                    &mut projection,
                    &view_camera.transition,
                );
            }
        }
    }
}
```

### 3. Input Handling Systems

```rust
// 3D Orbit Camera Controls
fn orbit_camera_input_system(
    mut cameras: Query<&mut GraphViewCamera>,
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    for mut camera in &mut cameras {
        if let ViewMode::ThreeD(mut state) = camera.view_mode {
            // Pan with middle mouse
            if mouse_input.pressed(MouseButton::Middle) {
                for motion in mouse_motion.read() {
                    // Update focus point based on motion
                }
            }

            // Orbit with right mouse
            if mouse_input.pressed(MouseButton::Right) {
                for motion in mouse_motion.read() {
                    state.azimuth += motion.delta.x * 0.01;
                    state.elevation = (state.elevation + motion.delta.y * 0.01)
                        .clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);
                }
            }

            camera.view_mode = ViewMode::ThreeD(state);
        }
    }
}

// 2D Pan Controls
fn pan_camera_input_system(
    mut cameras: Query<&mut GraphViewCamera>,
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
) {
    for mut camera in &mut cameras {
        if let ViewMode::TwoD(mut state) = camera.view_mode {
            if mouse_input.pressed(MouseButton::Middle) {
                for motion in mouse_motion.read() {
                    state.center.x -= motion.delta.x * state.zoom_level;
                    state.center.y += motion.delta.y * state.zoom_level;
                }
            }

            camera.view_mode = ViewMode::TwoD(state);
        }
    }
}
```

## View Mode Switching

### Mode Transition Logic

```rust
fn switch_view_mode(
    mut cameras: Query<&mut GraphViewCamera>,
    keyboard_input: Res<Input<KeyCode>>,
    graph_bounds: Res<GraphBounds>,
) {
    if keyboard_input.just_pressed(KeyCode::V) {
        for mut camera in &mut cameras {
            let new_mode = match camera.view_mode {
                ViewMode::ThreeD(state) => {
                    // Calculate 2D center from 3D focus
                    ViewMode::TwoD(TwoDState {
                        center: Vec2::new(state.focus_point.x, state.focus_point.z),
                        zoom_level: calculate_zoom_from_orbit(state.orbit_radius),
                        fixed_height: graph_bounds.max_y + 100.0,
                    })
                }
                ViewMode::TwoD(state) => {
                    // Calculate 3D position from 2D view
                    ViewMode::ThreeD(ThreeDState {
                        focus_point: Vec3::new(state.center.x, 0.0, state.center.y),
                        orbit_radius: calculate_orbit_from_zoom(state.zoom_level),
                        azimuth: PI / 4.0,
                        elevation: PI / 6.0,
                    })
                }
            };

            // Start transition
            camera.transition = CameraTransition {
                active: true,
                from_mode: camera.view_mode,
                to_mode: new_mode,
                progress: 0.0,
                duration: 0.5,
            };
        }
    }
}
```

### Smooth Interpolation

```rust
fn interpolate_camera_modes(
    transform: &mut Transform,
    projection: &mut Projection,
    transition: &CameraTransition,
) {
    let t = smooth_step(transition.progress);

    match (transition.from_mode, transition.to_mode) {
        (ViewMode::ThreeD(from_3d), ViewMode::TwoD(to_2d)) => {
            // Calculate intermediate position
            let from_pos = calculate_3d_position(from_3d);
            let to_pos = Vec3::new(to_2d.center.x, to_2d.fixed_height, to_2d.center.y);

            transform.translation = from_pos.lerp(to_pos, t);

            // Smoothly transition to looking down
            let from_rot = calculate_3d_rotation(from_3d);
            let to_rot = Quat::from_rotation_x(-PI / 2.0);

            transform.rotation = from_rot.slerp(to_rot, t);

            // Transition projection
            transition_to_orthographic(projection, t, to_2d.zoom_level);
        }
        (ViewMode::TwoD(from_2d), ViewMode::ThreeD(to_3d)) => {
            // Reverse transition logic
            // ...
        }
        _ => {} // Same mode transitions (shouldn't happen)
    }
}

fn smooth_step(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}
```

## Viewport and UI Integration

### Viewport Management

```rust
fn update_viewport_system(
    mut cameras: Query<&mut Camera>,
    windows: Query<&Window>,
    viewport_config: Res<ViewportConfig>,
) {
    let window = windows.single();

    for mut camera in &mut cameras {
        // Account for tools panel
        camera.viewport = Some(Viewport {
            physical_position: UVec2::new(
                viewport_config.tools_panel_width as u32,
                0,
            ),
            physical_size: UVec2::new(
                (window.width() - viewport_config.tools_panel_width) as u32,
                window.height() as u32,
            ),
            ..default()
        });
    }
}
```

### Coordinate Transformation

```rust
impl GraphViewCamera {
    pub fn screen_to_world(&self, screen_pos: Vec2, camera_transform: &Transform) -> Option<Vec3> {
        match self.view_mode {
            ViewMode::TwoD(state) => {
                // Simple orthographic projection
                Some(Vec3::new(
                    screen_pos.x * state.zoom_level + state.center.x,
                    0.0,
                    screen_pos.y * state.zoom_level + state.center.y,
                ))
            }
            ViewMode::ThreeD(_) => {
                // Ray casting for 3D picking
                // Implementation depends on specific requirements
                None
            }
        }
    }
}
```

## Performance Considerations

### Culling Strategy

```rust
#[derive(Component)]
pub struct ViewFrustum {
    pub bounds: FrustumBounds,
    pub mode: ViewMode,
}

fn update_frustum_culling(
    cameras: Query<(&GraphViewCamera, &Transform, &Projection)>,
    mut nodes: Query<(&mut Visibility, &Transform), With<GraphNode>>,
) {
    // Calculate visible bounds based on current view
    // Cull nodes outside frustum
}
```

### Level of Detail

```rust
fn update_lod_system(
    cameras: Query<&GraphViewCamera>,
    mut nodes: Query<(&mut GraphNodeLod, &Transform)>,
) {
    // Adjust detail level based on zoom/distance
}
```

## Plugin Structure

```rust
pub struct CameraViewportPlugin;

impl Plugin for CameraViewportPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewportConfig>()
            .add_systems(Update, (
                update_camera_system,
                camera_transition_system,
                orbit_camera_input_system,
                pan_camera_input_system,
                switch_view_mode,
                update_viewport_system,
            ).chain())
            .add_systems(PostUpdate, (
                update_frustum_culling,
                update_lod_system,
            ));
    }
}
```

This specification provides a complete technical foundation for implementing the dual-mode camera system with smooth transitions and proper viewport management.
