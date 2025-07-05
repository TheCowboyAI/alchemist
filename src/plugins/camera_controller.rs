//! Camera controller plugin for graph visualization
//!
//! Provides smooth camera controls for navigating the 3D graph space

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

/// Plugin for camera controls
pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraState>()
            .add_systems(Startup, setup_camera_controller)
            .add_systems(
                Update,
                (
                    handle_camera_pan,
                    handle_camera_orbit,
                    handle_camera_zoom,
                    smooth_camera_movement,
                ),
            );
    }
}

/// State for camera controls
#[derive(Resource)]
pub struct CameraState {
    /// Target position the camera is looking at
    pub focus_point: Vec3,
    /// Distance from the focus point
    pub distance: f32,
    /// Rotation around the Y axis (horizontal)
    pub yaw: f32,
    /// Rotation around the X axis (vertical)
    pub pitch: f32,
    /// Whether the camera is being controlled
    pub is_orbiting: bool,
    pub is_panning: bool,
    /// Smoothing factors
    pub orbit_sensitivity: f32,
    pub pan_sensitivity: f32,
    pub zoom_sensitivity: f32,
    /// Limits
    pub min_distance: f32,
    pub max_distance: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            focus_point: Vec3::ZERO,
            distance: 10.0,
            yaw: -std::f32::consts::FRAC_PI_4,
            pitch: -std::f32::consts::FRAC_PI_6,
            is_orbiting: false,
            is_panning: false,
            orbit_sensitivity: 0.5,
            pan_sensitivity: 0.01,
            zoom_sensitivity: 0.1,
            min_distance: 2.0,
            max_distance: 50.0,
            min_pitch: -std::f32::consts::FRAC_PI_2 + 0.1,
            max_pitch: std::f32::consts::FRAC_PI_2 - 0.1,
        }
    }
}

/// Marker component for the main camera
#[derive(Component)]
pub struct GraphCamera;

/// Setup camera controller
fn setup_camera_controller(mut commands: Commands, cameras: Query<Entity, With<Camera3d>>) {
    // Add marker to existing camera
    if let Ok(camera_entity) = cameras.get_single() {
        commands.entity(camera_entity).insert(GraphCamera);
    }
}

/// Handle camera panning with middle mouse button
fn handle_camera_pan(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut camera_state: ResMut<CameraState>,
    cameras: Query<&Transform, With<GraphCamera>>,
) {
    // Check if middle mouse button is pressed
    camera_state.is_panning = mouse_button.pressed(MouseButton::Middle);

    if camera_state.is_panning {
        let Ok(camera_transform) = cameras.get_single() else {
            return;
        };

        let mut delta = Vec2::ZERO;
        for event in mouse_motion.read() {
            delta += event.delta;
        }

        if delta.length_squared() > 0.0 {
            // Calculate movement in camera space
            let right = camera_transform.right();
            let up = camera_transform.up();

            let movement = -right * delta.x * camera_state.pan_sensitivity * camera_state.distance
                + up * delta.y * camera_state.pan_sensitivity * camera_state.distance;

            camera_state.focus_point += movement;
        }
    }
}

/// Handle camera orbit with right mouse button
fn handle_camera_orbit(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut camera_state: ResMut<CameraState>,
) {
    // Check if right mouse button is pressed
    camera_state.is_orbiting = mouse_button.pressed(MouseButton::Right);

    if camera_state.is_orbiting {
        let mut delta = Vec2::ZERO;
        for event in mouse_motion.read() {
            delta += event.delta;
        }

        if delta.length_squared() > 0.0 {
            // Update yaw and pitch
            camera_state.yaw -= delta.x * camera_state.orbit_sensitivity * 0.01;
            camera_state.pitch -= delta.y * camera_state.orbit_sensitivity * 0.01;

            // Clamp pitch to prevent camera flipping
            camera_state.pitch = camera_state
                .pitch
                .clamp(camera_state.min_pitch, camera_state.max_pitch);

            // Wrap yaw
            camera_state.yaw %= (2.0 * std::f32::consts::PI);
        }
    }
}

/// Handle camera zoom with mouse wheel
fn handle_camera_zoom(
    mut mouse_wheel: EventReader<MouseWheel>,
    mut camera_state: ResMut<CameraState>,
) {
    for event in mouse_wheel.read() {
        let zoom_delta = event.y * camera_state.zoom_sensitivity;
        
        // Exponential zoom for smoother feel
        camera_state.distance *= 1.0 - zoom_delta;
        
        // Clamp distance
        camera_state.distance = camera_state
            .distance
            .clamp(camera_state.min_distance, camera_state.max_distance);
    }
}

/// Smoothly update camera position based on state
fn smooth_camera_movement(
    mut cameras: Query<&mut Transform, With<GraphCamera>>,
    camera_state: Res<CameraState>,
    time: Res<Time>,
) {
    let Ok(mut camera_transform) = cameras.get_single_mut() else {
        return;
    };

    // Calculate target position
    let offset = Vec3::new(
        camera_state.yaw.cos() * camera_state.pitch.cos() * camera_state.distance,
        camera_state.pitch.sin() * camera_state.distance,
        camera_state.yaw.sin() * camera_state.pitch.cos() * camera_state.distance,
    );

    let target_position = camera_state.focus_point + offset;

    // Smooth interpolation
    let smoothing = 10.0 * time.delta_secs();
    let smoothing = smoothing.clamp(0.0, 1.0);

    camera_transform.translation = camera_transform
        .translation
        .lerp(target_position, smoothing);

    // Look at focus point
    camera_transform.look_at(camera_state.focus_point, Vec3::Y);
}

/// Focus camera on a specific position
pub fn focus_camera_on(camera_state: &mut CameraState, position: Vec3, distance: Option<f32>) {
    camera_state.focus_point = position;
    if let Some(dist) = distance {
        camera_state.distance = dist.clamp(camera_state.min_distance, camera_state.max_distance);
    }
}

/// Reset camera to default view
pub fn reset_camera(camera_state: &mut CameraState) {
    *camera_state = CameraState::default();
} 