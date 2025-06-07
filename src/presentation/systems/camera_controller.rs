//! Camera controller system for graph visualization
//!
//! Provides orbit camera controls for navigating the 3D graph view.

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use tracing::info;
use crate::presentation::components::OrbitCamera;

/// Updates the camera transform based on orbit parameters
pub fn update_orbit_camera(mut cameras: Query<(&mut Transform, &OrbitCamera)>) {
    for (mut transform, orbit) in cameras.iter_mut() {
        // Calculate the camera position based on spherical coordinates
        let x = orbit.distance * orbit.yaw.cos() * orbit.pitch.cos();
        let y = orbit.distance * orbit.pitch.sin();
        let z = orbit.distance * orbit.yaw.sin() * orbit.pitch.cos();

        let position = orbit.focus + Vec3::new(x, y, z);

        // Update transform
        transform.translation = position;
        transform.look_at(orbit.focus, Vec3::Y);
    }
}

/// Handles mouse input for camera rotation
pub fn orbit_camera_mouse_rotation(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut cameras: Query<&mut OrbitCamera>,
) {
    // Only rotate when right mouse button is pressed (left is for selection)
    if !mouse_button.pressed(MouseButton::Right) {
        return;
    }

    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }

    if delta.length_squared() > 0.0 {
        for mut orbit in cameras.iter_mut() {
            // Update yaw and pitch based on mouse movement
            orbit.yaw -= delta.x * orbit.sensitivity;
            orbit.pitch = (orbit.pitch - delta.y * orbit.sensitivity).clamp(-1.5, 1.5); // Limit pitch to avoid flipping
        }
    }
}

/// Handles mouse wheel input for camera zoom
pub fn orbit_camera_zoom(
    mut mouse_wheel: EventReader<MouseWheel>,
    mut cameras: Query<&mut OrbitCamera>,
) {
    let mut scroll = 0.0;
    for event in mouse_wheel.read() {
        scroll += event.y;
    }

    if scroll.abs() > 0.0 {
        for mut orbit in cameras.iter_mut() {
            // Update distance based on scroll
            orbit.distance = (orbit.distance - scroll * orbit.zoom_speed)
                .clamp(orbit.min_distance, orbit.max_distance);
        }
    }
}

/// Handles keyboard input for camera panning
pub fn orbit_camera_pan(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    time: Res<Time>,
    mut cameras: Query<(&Transform, &mut OrbitCamera)>,
) {
    let mut pan = Vec3::ZERO;
    let pan_speed = 10.0;

    // WASD for keyboard panning
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        pan.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        pan.z += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        pan.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        pan.x += 1.0;
    }

    // Q/E for vertical panning
    if keyboard.pressed(KeyCode::KeyQ) {
        pan.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyE) {
        pan.y += 1.0;
    }

    // Middle mouse button for mouse panning
    if mouse_button.pressed(MouseButton::Middle) {
        let mut delta = Vec2::ZERO;
        for event in mouse_motion.read() {
            delta += event.delta;
        }

        if delta.length_squared() > 0.0 {
            for (transform, _) in cameras.iter() {
                let right = transform.right();
                let up = transform.up();
                pan += right * -delta.x * 0.1 + up * delta.y * 0.1;
            }
        }
    }

    if pan.length_squared() > 0.0 {
        if keyboard.get_pressed().len() > 0 {
            pan = pan.normalize() * pan_speed * time.delta_secs();
        }

        for (transform, mut orbit) in cameras.iter_mut() {
            // Transform pan direction to camera space
            let right = transform.right();
            let forward = -transform.forward(); // Negative because camera looks at focus

            orbit.focus += right * pan.x + Vec3::Y * pan.y + forward * pan.z;
        }
    }
}

/// Resets camera to default view
pub fn reset_camera_view(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cameras: Query<&mut OrbitCamera>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        for mut orbit in cameras.iter_mut() {
            orbit.focus = Vec3::new(100.0, 0.0, 0.0);
            orbit.distance = 150.0;
            orbit.yaw = 0.0;
            orbit.pitch = -0.5;
        }
        info!("Camera reset to default view");
    }
}

/// Focus camera on selected nodes
pub fn focus_camera_on_selection(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cameras: Query<&mut OrbitCamera>,
    nodes: Query<&Transform, With<crate::presentation::components::GraphNode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyF) {
        // Calculate bounding box of all nodes
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        let mut count = 0;

        for transform in nodes.iter() {
            min = min.min(transform.translation);
            max = max.max(transform.translation);
            count += 1;
        }

        if count > 0 {
            // Calculate center and size
            let center = (min + max) * 0.5;
            let size = (max - min).length();

            for mut orbit in cameras.iter_mut() {
                orbit.focus = center;
                // Set distance based on bounding box size
                orbit.distance = (size * 1.5).max(orbit.min_distance).min(orbit.max_distance);
            }

            info!("Camera focused on {} nodes", count);
        }
    }
}

/// Display camera control help
pub fn display_camera_help() {
    eprintln!("Camera Controls:");
    eprintln!("  Left Mouse + Drag - Orbit camera");
    eprintln!("  Right Mouse + Drag - Pan camera");
    eprintln!("  Mouse Wheel - Zoom in/out");
    eprintln!("  R - Reset camera view");
    eprintln!("  F - Focus on selection");

    info!("Camera Controls:");
    info!("  Left Mouse + Drag - Orbit camera");
    info!("  Right Mouse + Drag - Pan camera");
    info!("  Mouse Wheel - Zoom in/out");
    info!("  R - Reset camera view");
    info!("  F - Focus on selection");
}
