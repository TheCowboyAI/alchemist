use super::components::*;
use super::systems::*;
use bevy::prelude::*;

/// Plugin for the dual-mode camera system
pub struct CameraViewportPlugin;

impl Plugin for CameraViewportPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<ViewportConfig>()
            .init_resource::<GraphBounds>()
            // Update systems - ordered for proper execution
            .add_systems(
                Update,
                (
                    // Input handling first
                    orbit_camera_input_system,
                    pan_camera_input_system,
                    switch_view_mode,
                    // Then update camera state
                    camera_transition_system,
                    update_camera_system,
                    // Update viewport
                    update_viewport_system,
                    // Update graph bounds
                    update_graph_bounds_system,
                )
                    .chain(),
            )
            // Post-update systems for optimization
            .add_systems(PostUpdate, (update_frustum_culling, update_lod_system));
    }
}
