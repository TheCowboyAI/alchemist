use bevy::prelude::*;
use std::f32::consts::PI;

/// Main camera component for graph viewing with dual modes
#[derive(Component)]
pub struct GraphViewCamera {
    pub view_mode: ViewMode,
    pub transition: CameraTransition,
}

impl Default for GraphViewCamera {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::ThreeD(ThreeDState::default()),
            transition: CameraTransition::default(),
        }
    }
}

/// Camera view modes
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ViewMode {
    ThreeD(ThreeDState),
    TwoD(TwoDState),
}

/// 3D orbit camera state
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ThreeDState {
    pub focus_point: Vec3,
    pub orbit_radius: f32,
    pub azimuth: f32,   // Horizontal rotation (0-2π)
    pub elevation: f32, // Vertical rotation (-π/2 to π/2)
}

impl Default for ThreeDState {
    fn default() -> Self {
        Self {
            focus_point: Vec3::ZERO,
            orbit_radius: 15.0,
            azimuth: PI / 4.0,
            elevation: PI / 6.0,
        }
    }
}

/// 2D top-down camera state
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TwoDState {
    pub center: Vec2,
    pub zoom_level: f32,   // Orthographic scale
    pub fixed_height: f32, // Y position for top-down view
}

impl Default for TwoDState {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            zoom_level: 1.0,
            fixed_height: 100.0,
        }
    }
}

/// Camera transition state for smooth mode switching
#[derive(Component, Clone)]
pub struct CameraTransition {
    pub active: bool,
    pub from_mode: ViewMode,
    pub to_mode: ViewMode,
    pub progress: f32,
    pub duration: f32,
}

impl Default for CameraTransition {
    fn default() -> Self {
        Self {
            active: false,
            from_mode: ViewMode::ThreeD(ThreeDState::default()),
            to_mode: ViewMode::ThreeD(ThreeDState::default()),
            progress: 0.0,
            duration: 0.5,
        }
    }
}

/// Resource for viewport configuration
#[derive(Resource)]
pub struct ViewportConfig {
    pub main_viewport: ViewportRect,
    pub tools_panel_width: f32,
    pub aspect_ratio: f32,
}

impl Default for ViewportConfig {
    fn default() -> Self {
        Self {
            main_viewport: ViewportRect::default(),
            tools_panel_width: 300.0,
            aspect_ratio: 16.0 / 9.0,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct ViewportRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Bounds of the graph for camera calculations
#[derive(Resource, Default)]
pub struct GraphBounds {
    pub min: Vec3,
    pub max: Vec3,
    pub center: Vec3,
    pub max_y: f32,
}

/// Component to mark nodes for frustum culling
#[derive(Component)]
pub struct ViewFrustum {
    pub in_view: bool,
}

/// Level of detail component for performance optimization
#[derive(Component)]
pub struct GraphNodeLod {
    pub detail_level: DetailLevel,
}

#[derive(Clone, Copy, PartialEq)]
pub enum DetailLevel {
    High,
    Medium,
    Low,
    Culled,
}

impl GraphViewCamera {
    /// Convert screen coordinates to world position based on current view mode
    pub fn screen_to_world(
        &self,
        screen_pos: Vec2,
        _camera_transform: &Transform,
        window_size: Vec2,
    ) -> Option<Vec3> {
        match self.view_mode {
            ViewMode::TwoD(state) => {
                // Simple orthographic projection for 2D
                let normalized_pos = Vec2::new(
                    (screen_pos.x / window_size.x - 0.5) * 2.0,
                    -(screen_pos.y / window_size.y - 0.5) * 2.0,
                );

                Some(Vec3::new(
                    normalized_pos.x * window_size.x * state.zoom_level + state.center.x,
                    0.0,
                    normalized_pos.y * window_size.y * state.zoom_level + state.center.y,
                ))
            }
            ViewMode::ThreeD(_) => {
                // Ray casting for 3D picking will be implemented in systems
                None
            }
        }
    }

    /// Start a transition to a new view mode
    pub fn start_transition(&mut self, to_mode: ViewMode, duration: f32) {
        self.transition = CameraTransition {
            active: true,
            from_mode: self.view_mode,
            to_mode,
            progress: 0.0,
            duration,
        };
    }
}
