use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::math::Rect;
use bevy::prelude::*;
use bevy::render::camera::{OrthographicProjection, Projection, ScalingMode, Viewport};
use std::f32::consts::PI;

use super::components::*;

/// Update camera transform and projection based on current view mode
pub fn update_camera_system(
    mut cameras: Query<(&mut Transform, &mut Projection, &GraphViewCamera)>,
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

fn update_3d_camera(transform: &mut Transform, projection: &mut Projection, state: ThreeDState) {
    // Calculate camera position from orbit parameters
    let x = state.focus_point.x + state.orbit_radius * state.elevation.cos() * state.azimuth.cos();
    let y = state.focus_point.y + state.orbit_radius * state.elevation.sin();
    let z = state.focus_point.z + state.orbit_radius * state.elevation.cos() * state.azimuth.sin();

    transform.translation = Vec3::new(x, y, z);
    transform.look_at(state.focus_point, Vec3::Y);

    // Ensure perspective projection for 3D
    if let Projection::Orthographic(_) = projection {
        *projection = Projection::Perspective(PerspectiveProjection {
            fov: PI / 4.0,
            ..default()
        });
    }
}

fn update_2d_camera(transform: &mut Transform, projection: &mut Projection, state: TwoDState) {
    // Position camera for top-down view
    transform.translation = Vec3::new(state.center.x, state.fixed_height, state.center.y);
    transform.rotation = Quat::from_rotation_x(-PI / 2.0); // Look down

    // Use orthographic projection for 2D
    *projection = Projection::Orthographic(OrthographicProjection {
        scale: state.zoom_level,
        near: -1000.0,
        far: 1000.0,
        viewport_origin: Vec2::new(0.5, 0.5),
        scaling_mode: ScalingMode::WindowSize,
        area: Rect {
            min: Vec2::new(-1.0, -1.0),
            max: Vec2::new(1.0, 1.0),
        },
    });
}

/// Handle smooth transitions between camera modes
pub fn camera_transition_system(
    mut cameras: Query<(&mut GraphViewCamera, &mut Transform, &mut Projection)>,
    time: Res<Time>,
) {
    for (mut view_camera, mut transform, mut projection) in &mut cameras {
        if view_camera.transition.active {
            // Update transition progress
            view_camera.transition.progress += time.delta_secs() / view_camera.transition.duration;

            if view_camera.transition.progress >= 1.0 {
                // Complete transition
                view_camera.view_mode = view_camera.transition.to_mode;
                view_camera.transition.active = false;
            } else {
                // Interpolate between modes
                interpolate_camera_modes(&mut transform, &mut projection, &view_camera.transition);
            }
        }
    }
}

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
            if t > 0.5 {
                *projection = Projection::Orthographic(OrthographicProjection {
                    scale: to_2d.zoom_level,
                    near: -1000.0,
                    far: 1000.0,
                    viewport_origin: Vec2::new(0.5, 0.5),
                    scaling_mode: ScalingMode::WindowSize,
                    area: Rect {
                        min: Vec2::new(-1.0, -1.0),
                        max: Vec2::new(1.0, 1.0),
                    },
                });
            }
        }
        (ViewMode::TwoD(from_2d), ViewMode::ThreeD(to_3d)) => {
            // Calculate intermediate position
            let from_pos = Vec3::new(from_2d.center.x, from_2d.fixed_height, from_2d.center.y);
            let to_pos = calculate_3d_position(to_3d);

            transform.translation = from_pos.lerp(to_pos, t);

            // Smoothly transition from looking down
            let from_rot = Quat::from_rotation_x(-PI / 2.0);
            let to_rot = calculate_3d_rotation(to_3d);

            transform.rotation = from_rot.slerp(to_rot, t);

            // Transition projection
            if t > 0.5 {
                *projection = Projection::Perspective(PerspectiveProjection {
                    fov: PI / 4.0,
                    ..default()
                });
            }
        }
        _ => {} // Same mode transitions (shouldn't happen)
    }
}

fn smooth_step(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn calculate_3d_position(state: ThreeDState) -> Vec3 {
    let x = state.focus_point.x + state.orbit_radius * state.elevation.cos() * state.azimuth.cos();
    let y = state.focus_point.y + state.orbit_radius * state.elevation.sin();
    let z = state.focus_point.z + state.orbit_radius * state.elevation.cos() * state.azimuth.sin();
    Vec3::new(x, y, z)
}

fn calculate_3d_rotation(state: ThreeDState) -> Quat {
    let position = calculate_3d_position(state);
    Transform::from_translation(position)
        .looking_at(state.focus_point, Vec3::Y)
        .rotation
}

/// Handle 3D orbit camera input
pub fn orbit_camera_input_system(
    mut cameras: Query<&mut GraphViewCamera>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for mut camera in &mut cameras {
        if let ViewMode::ThreeD(mut state) = camera.view_mode {
            // Pan with middle mouse button
            if mouse_input.pressed(MouseButton::Middle)
                && keyboard_input.pressed(KeyCode::ShiftLeft)
            {
                for motion in mouse_motion.read() {
                    let right = Vec3::new(state.azimuth.cos(), 0.0, -state.azimuth.sin());
                    let forward = Vec3::new(state.azimuth.sin(), 0.0, state.azimuth.cos());

                    state.focus_point += right * motion.delta.x * 0.01 * state.orbit_radius;
                    state.focus_point += forward * motion.delta.y * 0.01 * state.orbit_radius;
                }
            }
            // Orbit with right mouse button or middle mouse without shift
            else if mouse_input.pressed(MouseButton::Right)
                || (mouse_input.pressed(MouseButton::Middle)
                    && !keyboard_input.pressed(KeyCode::ShiftLeft))
            {
                for motion in mouse_motion.read() {
                    state.azimuth -= motion.delta.x * 0.01;
                    state.elevation = (state.elevation - motion.delta.y * 0.01)
                        .clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);
                }
            }

            // Zoom with mouse wheel
            for wheel in mouse_wheel.read() {
                state.orbit_radius = (state.orbit_radius - wheel.y * 2.0).clamp(5.0, 100.0);
            }

            camera.view_mode = ViewMode::ThreeD(state);
        }
    }
}

/// Handle 2D pan camera input
pub fn pan_camera_input_system(
    mut cameras: Query<&mut GraphViewCamera>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
) {
    for mut camera in &mut cameras {
        if let ViewMode::TwoD(mut state) = camera.view_mode {
            // Pan with middle mouse button
            if mouse_input.pressed(MouseButton::Middle) {
                for motion in mouse_motion.read() {
                    state.center.x -= motion.delta.x * state.zoom_level;
                    state.center.y += motion.delta.y * state.zoom_level;
                }
            }

            // Zoom with mouse wheel
            for wheel in mouse_wheel.read() {
                state.zoom_level = (state.zoom_level - wheel.y * 0.1).clamp(0.1, 10.0);
            }

            camera.view_mode = ViewMode::TwoD(state);
        }
    }
}

/// Switch between view modes
pub fn switch_view_mode(
    mut cameras: Query<&mut GraphViewCamera>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    graph_bounds: Res<GraphBounds>,
) {
    if keyboard_input.just_pressed(KeyCode::Tab) || keyboard_input.just_pressed(KeyCode::KeyV) {
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

            camera.start_transition(new_mode, 0.5);
            info!("Switching camera mode to: {:?}", new_mode);
        }
    }
}

fn calculate_zoom_from_orbit(orbit_radius: f32) -> f32 {
    // Scale down from orbit radius to orthographic zoom
    // Smaller values = more zoomed in
    orbit_radius / 150.0 // Changed from 15.0 to 150.0
}

fn calculate_orbit_from_zoom(zoom_level: f32) -> f32 {
    // Scale up from orthographic zoom to orbit radius
    zoom_level * 150.0 // Changed from 15.0 to 150.0
}

/// Update viewport based on window size and UI panels
pub fn update_viewport_system(
    mut cameras: Query<&mut Camera>,
    windows: Query<&Window>,
    viewport_config: Res<ViewportConfig>,
) {
    let Ok(window) = windows.single() else { return };

    for mut camera in &mut cameras {
        // Account for tools panel
        camera.viewport = Some(Viewport {
            physical_position: UVec2::new(viewport_config.tools_panel_width as u32, 0),
            physical_size: UVec2::new(
                (window.physical_width() as f32 - viewport_config.tools_panel_width) as u32,
                window.physical_height(),
            ),
            ..default()
        });
    }
}

/// Update graph bounds for camera calculations
pub fn update_graph_bounds_system(
    node_query: Query<&Transform, With<crate::graph_core::GraphNode>>,
    mut graph_bounds: ResMut<GraphBounds>,
) {
    if node_query.is_empty() {
        *graph_bounds = GraphBounds::default();
        return;
    }

    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);

    for transform in &node_query {
        min = min.min(transform.translation);
        max = max.max(transform.translation);
    }

    graph_bounds.min = min;
    graph_bounds.max = max;
    graph_bounds.center = (min + max) * 0.5;
    graph_bounds.max_y = max.y;
}

/// Frustum culling for performance optimization
pub fn update_frustum_culling(
    cameras: Query<(&GraphViewCamera, &Transform, &Projection)>,
    mut nodes: Query<(&mut ViewFrustum, &Transform), With<crate::graph_core::GraphNode>>,
) {
    let Ok((view_camera, camera_transform, _projection)) = cameras.single() else {
        return;
    };

    // Simple distance-based culling for now
    let camera_pos = camera_transform.translation;
    let cull_distance = match view_camera.view_mode {
        ViewMode::ThreeD(state) => state.orbit_radius * 3.0,
        ViewMode::TwoD(state) => state.zoom_level * 1000.0,
    };

    for (mut frustum, node_transform) in &mut nodes {
        let distance = camera_pos.distance(node_transform.translation);
        frustum.in_view = distance < cull_distance;
    }
}

/// Update level of detail based on camera distance
pub fn update_lod_system(
    cameras: Query<&GraphViewCamera>,
    mut nodes: Query<(&mut GraphNodeLod, &Transform, &ViewFrustum)>,
) {
    let Ok(view_camera) = cameras.single() else {
        return;
    };

    for (mut lod, _transform, frustum) in &mut nodes {
        if !frustum.in_view {
            lod.detail_level = DetailLevel::Culled;
            continue;
        }

        // Set LOD based on view mode and distance
        lod.detail_level = match view_camera.view_mode {
            ViewMode::ThreeD(_) => DetailLevel::High, // Always high detail in 3D for now
            ViewMode::TwoD(state) => {
                if state.zoom_level < 0.5 {
                    DetailLevel::Low
                } else if state.zoom_level < 1.0 {
                    DetailLevel::Medium
                } else {
                    DetailLevel::High
                }
            }
        };
    }
}
