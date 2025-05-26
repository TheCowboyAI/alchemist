use super::components::*;
use super::systems::*;
use crate::system_sets::CameraSystemSet;
use bevy::prelude::*;

/// Plugin for the dual-mode camera system
pub struct CameraViewportPlugin;

impl Plugin for CameraViewportPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<ViewportConfig>()
            .init_resource::<GraphBounds>()

            // Camera Input Systems - Phase 1
            .add_systems(
                Update,
                (
                    orbit_camera_input_system,
                    pan_camera_input_system,
                    switch_view_mode,
                )
                    .in_set(CameraSystemSet::Input),
            )

            // Camera Update Systems - Phase 2
            .add_systems(
                Update,
                (
                    camera_transition_system,
                    update_camera_system,
                )
                    .chain()
                    .in_set(CameraSystemSet::Update),
            )

            // Viewport Systems - Phase 3
            .add_systems(
                Update,
                (
                    update_viewport_system,
                    update_graph_bounds_system,
                )
                    .in_set(CameraSystemSet::Viewport),
            )

            // Post-update systems for optimization
            .add_systems(PostUpdate, (update_frustum_culling, update_lod_system));
    }
}
