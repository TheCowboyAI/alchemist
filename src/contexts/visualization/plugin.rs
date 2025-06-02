use crate::contexts::visualization::layout::LayoutPlugin;
use crate::contexts::visualization::point_cloud::PointCloudPlugin;
use crate::contexts::visualization::services::*;
use crate::contexts::graph_management::plugin::GraphManagementSet;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;

/// Plugin for the Visualization bounded context
pub struct VisualizationPlugin;

impl Plugin for VisualizationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add the panorbit camera plugin
            .add_plugins(PanOrbitCameraPlugin)
            // Add the point cloud plugin
            .add_plugins(PointCloudPlugin)
            // Add the layout plugin
            .add_plugins(LayoutPlugin)
            // Events
            .add_event::<EdgeTypeChanged>()
            .add_event::<RenderModeChanged>()
            .add_event::<VisualizationUpdateRequested>()
            .add_event::<ConvertToPointCloud>()
            // Startup systems
            .add_systems(
                Startup,
                (
                    ControlCamera::setup_camera,
                    Self::setup_visualization_settings,
                ),
            )
            // Basic visualization systems - run after graph management completes
            .add_systems(
                Update,
                (
                    RenderGraphElements::visualize_new_nodes,
                    RenderGraphElements::visualize_new_edges,
                    RenderGraphElements::handle_visualization_update_requests,
                    RenderGraphElements::handle_convert_to_point_cloud,
                    RenderGraphElements::render_edge_flow_particles,
                )
                .after(GraphManagementSet::Hierarchy),
            )
            // User input systems
            .add_systems(
                Update,
                (
                    HandleUserInput::change_edge_type,
                    HandleUserInput::change_render_mode,
                    HandleUserInput::trigger_layout,
                    HandleUserInput::trigger_visualization_update,
                    HandleUserInput::trigger_point_cloud_conversion,
                    ControlCamera::update_billboards,
                ),
            )
            // State update systems
            .add_systems(
                Update,
                (
                    UpdateVisualizationState::handle_edge_type_changed,
                    UpdateVisualizationState::handle_render_mode_changed,
                    UpdateVisualizationState::update_existing_edges
                        .after(UpdateVisualizationState::handle_edge_type_changed),
                    UpdateVisualizationState::update_existing_nodes
                        .after(UpdateVisualizationState::handle_render_mode_changed),
                    UpdateVisualizationState::manage_graph_animations_on_mode_change
                        .after(UpdateVisualizationState::handle_render_mode_changed),
                ),
            )
            // Animation systems
            .add_systems(
                Update,
                (
                    AnimateGraphElements::animate_graphs,
                    AnimateGraphElements::animate_subgraphs,
                    AnimateGraphElements::animate_nodes,
                    AnimateGraphElements::animate_edges,
                    AnimateGraphElements::animate_edge_materials,
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
