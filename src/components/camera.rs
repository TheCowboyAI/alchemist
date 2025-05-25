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
            zoom_level: 0.1,
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
