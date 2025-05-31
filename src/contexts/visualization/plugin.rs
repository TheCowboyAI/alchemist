use crate::contexts::visualization::services::*;
use bevy::prelude::*;

/// Plugin for the Visualization bounded context
pub struct VisualizationPlugin;

impl Plugin for VisualizationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Setup systems
            .add_systems(Startup, ControlCamera::setup_camera)
            // Update systems - using proper verb phrases!
            .add_systems(
                Update,
                (
                    ControlCamera::orbit_camera,
                    HandleUserInput::process_selection,
                    RenderGraphElements::visualize_new_nodes,
                ),
            );
    }
}
