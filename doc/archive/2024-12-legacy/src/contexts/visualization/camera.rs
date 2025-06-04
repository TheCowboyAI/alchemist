//! Camera controller for graph visualization
//!
//! Provides orbit camera controls for navigating the 3D graph view.

use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};

/// Simple orbit camera controller component
#[derive(Component, Debug)]
pub struct OrbitCamera {
    /// Focus point the camera orbits around
    pub focus: Vec3,
    /// Distance from the focus point
    pub distance: f32,
    /// Rotation around the Y axis (yaw)
    pub yaw: f32,
    /// Rotation around the X axis (pitch)
    pub pitch: f32,
    /// Mouse sensitivity for rotation
    pub sensitivity: f32,
    /// Zoom speed
    pub zoom_speed: f32,
    /// Minimum zoom distance
    pub min_distance: f32,
    /// Maximum zoom distance
    pub max_distance: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            distance: 15.0,
            yaw: 0.0,
            pitch: -0.3,
            sensitivity: 0.005,
            zoom_speed: 0.5,
            min_distance: 2.0,
            max_distance: 100.0,
        }
    }
}

/// Updates the camera transform based on orbit parameters
pub fn update_orbit_camera(
    mut cameras: Query<(&mut Transform, &OrbitCamera)>,
) {
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
    // Only rotate when left mouse button is pressed
    if !mouse_button.pressed(MouseButton::Left) {
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
            orbit.pitch = (orbit.pitch - delta.y * orbit.sensitivity)
                .clamp(-1.5, 1.5); // Limit pitch to avoid flipping
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
    time: Res<Time>,
    mut cameras: Query<(&Transform, &mut OrbitCamera)>,
) {
    let mut pan = Vec3::ZERO;
    let pan_speed = 5.0;

    // WASD for panning
    if keyboard.pressed(KeyCode::KeyW) {
        pan.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        pan.z += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        pan.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        pan.x += 1.0;
    }

    // Q/E for vertical panning
    if keyboard.pressed(KeyCode::KeyQ) {
        pan.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyE) {
        pan.y += 1.0;
    }

    if pan.length_squared() > 0.0 {
        pan = pan.normalize() * pan_speed * time.delta_secs();

        for (transform, mut orbit) in cameras.iter_mut() {
            // Transform pan direction to camera space
            let right = transform.right();
            let forward = transform.forward();

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
            orbit.focus = Vec3::ZERO;
            orbit.distance = 15.0;
            orbit.yaw = 0.0;
            orbit.pitch = -0.3;
        }
        info!("Camera reset to default view");
    }
}
