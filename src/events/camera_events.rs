//! Camera-related events for view control and navigation
//!
//! These events handle:
//! - Camera movement and rotation
//! - View mode switching (2D/3D)
//! - Focus and framing operations
//! - Camera state persistence
//! - Animation and transitions

use bevy::prelude::*;
use uuid::Uuid;

/// Event for switching camera view mode
///
/// ## Producers
/// - View mode toggle button
/// - Keyboard shortcut (e.g., Tab)
/// - Context-based switching
///
/// ## Consumers
/// - `camera_mode_system` - Switches between 2D/3D cameras
/// - `ui_update_system` - Updates mode indicators
#[derive(Event)]
pub struct SwitchViewModeEvent {
    pub to_2d: bool,
}

/// Event for focusing camera on a node
#[derive(Event)]
pub struct FocusNodeEvent {
    pub node_id: Uuid,
    pub instant: bool,
}

/// Event for focusing camera on selection
#[derive(Event)]
pub struct FocusSelectionEvent {
    pub instant: bool,
}

/// Event for resetting camera to default position
#[derive(Event)]
pub struct ResetCameraEvent;

/// Event for fitting camera to show all nodes
#[derive(Event)]
pub struct FitToViewEvent {
    pub padding: f32,
}

/// Event for camera animation completion
#[derive(Event)]
pub struct CameraAnimationCompleteEvent;

/// Event for saving camera position
#[derive(Event)]
pub struct SaveCameraPositionEvent {
    pub slot: u8,
}

/// Event for loading camera position
#[derive(Event)]
pub struct LoadCameraPositionEvent {
    pub slot: u8,
}

/// Event for camera bounds update
#[derive(Event)]
pub struct UpdateCameraBoundsEvent;

/// Event for camera zoom
#[derive(Event)]
pub struct CameraZoomEvent {
    pub delta: f32,
    pub towards_cursor: bool,
}

/// Event for camera orbit control
///
/// ## Producers
/// - Mouse drag with middle button
/// - Touch gestures
///
/// ## Consumers
/// - `orbit_camera_system` - Updates camera rotation
#[derive(Event)]
pub struct OrbitCameraEvent {
    pub delta_x: f32,
    pub delta_y: f32,
}

/// Event for camera pan control
///
/// ## Producers
/// - Mouse drag with shift
/// - Touch pan gestures
/// - Arrow keys
///
/// ## Consumers
/// - `pan_camera_system` - Updates camera position
#[derive(Event)]
pub struct PanCameraEvent {
    pub delta: Vec2,
    pub screen_space: bool, // If true, delta is in screen coordinates
}

/// Event for camera animation
///
/// ## Producers
/// - Focus operations
/// - View transitions
/// - Preset views
///
/// ## Consumers
/// - `camera_animation_system` - Animates camera smoothly
#[derive(Event)]
pub struct AnimateCameraEvent {
    pub target_position: Vec3,
    pub target_rotation: Option<Quat>,
    pub target_scale: Option<f32>, // For orthographic zoom
    pub duration: f32,
    pub easing: EasingFunction,
}

#[derive(Clone, Copy)]
pub enum EasingFunction {
    Linear,
    EaseInOut,
    EaseIn,
    EaseOut,
    Spring,
}

/// Event for preset camera views
///
/// ## Producers
/// - View menu selections
/// - Numpad shortcuts
/// - View cube widget
///
/// ## Consumers
/// - `preset_view_system` - Sets camera to preset angles
#[derive(Event)]
pub struct SetPresetViewEvent {
    pub preset: CameraPreset,
    pub animate: bool,
}

#[derive(Clone, Copy)]
pub enum CameraPreset {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
    Isometric,
    Custom(usize), // User-defined preset index
}

/// Event for camera constraints update
///
/// ## Producers
/// - Settings changes
/// - Context-based constraints
///
/// ## Consumers
/// - `camera_constraint_system` - Updates movement limits
#[derive(Event)]
pub struct UpdateCameraConstraintsEvent {
    pub constraints: CameraConstraints,
}

#[derive(Clone)]
pub struct CameraConstraints {
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub pan_bounds: Option<(Vec3, Vec3)>, // Min and max positions
    pub orbit_limits: Option<OrbitLimits>,
}

#[derive(Clone)]
pub struct OrbitLimits {
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub min_yaw: Option<f32>,
    pub max_yaw: Option<f32>,
}

/// Event for camera follow mode
///
/// ## Producers
/// - Follow mode toggle
/// - Entity selection with follow
///
/// ## Consumers
/// - `camera_follow_system` - Makes camera follow entity
#[derive(Event)]
pub struct SetCameraFollowEvent {
    pub target: Option<Entity>,
    pub offset: Vec3,
    pub smooth_factor: f32,
}

/// Event for screenshot capture
///
/// ## Producers
/// - Screenshot button/shortcut
/// - Export operations
///
/// ## Consumers
/// - `screenshot_system` - Captures current view
#[derive(Event)]
pub struct CaptureScreenshotEvent {
    pub include_ui: bool,
    pub resolution: Option<(u32, u32)>,
    pub file_path: Option<String>,
}

/// Event for viewport updates
///
/// ## Producers
/// - Window resize
/// - Split view changes
///
/// ## Consumers
/// - `viewport_system` - Updates camera viewports
#[derive(Event)]
pub struct UpdateViewportEvent {
    pub viewport_id: u32,
    pub rect: Option<bevy::render::camera::Viewport>,
}

/// Event for camera state export
///
/// ## Producers
/// - Save view action
/// - State persistence
///
/// ## Consumers
/// - `camera_persistence_system` - Saves camera state
#[derive(Event)]
pub struct ExportCameraStateEvent {
    pub include_constraints: bool,
}

/// Event for camera state import
///
/// ## Producers
/// - Load view action
/// - State restoration
///
/// ## Consumers
/// - `camera_persistence_system` - Loads camera state
#[derive(Event)]
pub struct ImportCameraStateEvent {
    pub state: CameraState,
}

#[derive(Clone)]
pub struct CameraState {
    pub position: Vec3,
    pub rotation: Quat,
    pub projection: CameraProjection,
}

#[derive(Clone)]
pub enum CameraProjection {
    Perspective { fov: f32 },
    Orthographic { scale: f32 },
}
