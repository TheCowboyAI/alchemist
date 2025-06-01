use crate::contexts::visualization::point_cloud::PointCloudPlugin;
use crate::contexts::visualization::services::*;
use bevy::prelude::*;

/// Plugin for the Visualization bounded context
pub struct VisualizationPlugin;

impl Plugin for VisualizationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add the point cloud plugin
            .add_plugins(PointCloudPlugin)
            // Events
            .add_event::<EdgeTypeChanged>()
            .add_event::<RenderModeChanged>()
            .add_event::<VisualizationUpdateRequested>()
            .add_event::<ConvertToPointCloud>()
            .add_event::<NodeSelected>()
            .add_event::<NodeDeselected>()
            // Startup systems
            .add_systems(
                Startup,
                (
                    ControlCamera::setup_camera,
                    Self::setup_visualization_settings,
                ),
            )
            // Visualization systems
            .add_systems(
                Update,
                (
                    RenderGraphElements::visualize_new_nodes,
                    RenderGraphElements::visualize_new_edges,
                    HandleUserInput::process_selection,
                    HandleUserInput::change_edge_type,
                    HandleUserInput::change_render_mode,
                    ControlCamera::orbit_camera,
                    ControlCamera::update_billboards,
                    // State update systems
                    UpdateVisualizationState::handle_edge_type_changed,
                    UpdateVisualizationState::handle_render_mode_changed,
                    UpdateVisualizationState::update_existing_edges.after(UpdateVisualizationState::handle_edge_type_changed),
                    // Selection visualization systems
                    SelectionVisualization::handle_node_selection,
                    SelectionVisualization::handle_node_deselection,
                    SelectionVisualization::handle_deselect_all,
                    // Animation systems - hierarchical order matters
                    AnimateGraphElements::animate_graphs,
                    AnimateGraphElements::animate_subgraphs,
                    AnimateGraphElements::animate_nodes,
                    AnimateGraphElements::animate_edges,
                    AnimateGraphElements::handle_graph_animation_events,
                ),
            );
    }
}

impl VisualizationPlugin {
    /// Creates the settings entity with default visualization settings
    fn setup_visualization_settings(mut commands: Commands) {
        commands.spawn(CurrentVisualizationSettings::default());
        info!("Visualization settings entity created");
    }
}
