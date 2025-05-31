use crate::contexts::visualization::services::*;
use bevy::prelude::*;

/// Plugin for the Visualization bounded context
pub struct VisualizationPlugin;

impl Plugin for VisualizationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Camera setup
            .add_systems(Startup, ControlCamera::setup_camera)

            // Visualization systems
            .add_systems(Update, (
                RenderGraphElements::visualize_new_nodes,
                HandleUserInput::process_selection,
                ControlCamera::orbit_camera,

                // Animation systems - hierarchical order matters
                AnimateGraphElements::animate_graphs,
                AnimateGraphElements::animate_subgraphs,
                AnimateGraphElements::animate_nodes,
                AnimateGraphElements::handle_graph_animation_events,
            ));
    }
}
