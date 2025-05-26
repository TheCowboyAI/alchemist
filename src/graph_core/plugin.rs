use super::algorithms::{GraphAlgorithms, demonstrate_algorithms};
use super::change_detection::{
    GraphChangeTracker, detect_component_changes, process_graph_changes, update_change_flags,
};
use super::events::*;
use super::graph_data::GraphData;
use super::merkle_dag::MerkleDag;
use super::rendering::*;
use super::systems::*;
use super::ui::{
    graph_inspector_ui, handle_node_selection, update_selection_highlights,
};
use crate::resources::{GraphState, GraphMetadata, GraphInspectorState, EdgeMeshTracker, LastViewMode};
use crate::system_sets::{GraphSystemSet, GraphChangeFlags, reset_change_flags, view_mode_changed, nodes_changed, edges_changed};
use bevy::diagnostic::FrameCount;
use bevy::prelude::*;

/// Plugin for the graph editor functionality
pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<GraphState>()
            .init_resource::<GraphMetadata>()
            .init_resource::<GraphData>()
            .init_resource::<MerkleDag>()
            .init_resource::<GraphChangeTracker>()
            .init_resource::<GraphInspectorState>()
            .init_resource::<EdgeMeshTracker>()
            .init_resource::<LastViewMode>()
            .init_resource::<GraphChangeFlags>()
            .insert_resource(GraphAlgorithms)
            // Events
            .add_event::<CreateNodeEvent>()
            .add_event::<MoveNodeEvent>()
            .add_event::<CreateEdgeEvent>()
            .add_event::<DeleteNodeEvent>()
            .add_event::<DeleteEdgeEvent>()
            .add_event::<SelectEvent>()
            .add_event::<DeselectAllEvent>()
            .add_event::<HoverEvent>()
            .add_event::<LayoutUpdateEvent>()
            .add_event::<RequestLayoutEvent>()
            .add_event::<ValidateGraphEvent>()
            .add_event::<SaveGraphEvent>()
            .add_event::<LoadGraphEvent>()
            .add_event::<DeferredEdgeEvent>()
            // .add_event::<CreatePatternEvent>() // TODO: Implement graph_patterns module
            .add_event::<CreateSubgraphEvent>()
            .add_event::<UndoEvent>()
            .add_event::<RedoEvent>()
            .add_event::<GraphModificationEvent>()

            // Event Processing Systems - Phase 2
            .add_systems(
                Update,
                (
                    // Process node events first
                    handle_create_node_with_graph,
                    handle_move_node_events,
                    handle_validation_events,
                )
                    .chain()
                    .in_set(GraphSystemSet::EventProcessing),
            )
            .add_systems(
                Update,
                (
                    // Process edge events after nodes
                    handle_create_edge_with_graph,
                    handle_deferred_edge_events,
                )
                    .chain()
                    .in_set(GraphSystemSet::EventProcessing)
                    .after(handle_create_node_with_graph),
            )
            .add_systems(
                Update,
                (
                    // Process selection/interaction events
                    handle_selection_events,
                    handle_hover_events,
                )
                    .chain()
                    .in_set(GraphSystemSet::EventProcessing),
            )

            // State Update Systems - Phase 3
            .add_systems(
                Update,
                (
                    // Update visual states
                    update_node_visuals,
                    update_selection_highlights,
                )
                    .chain()
                    .in_set(GraphSystemSet::StateUpdate),
            )

            // Change Detection Systems - Phase 4
            .add_systems(
                Update,
                (
                    update_change_flags,
                    detect_component_changes,
                    process_graph_changes,
                )
                    .chain()
                    .in_set(GraphSystemSet::ChangeDetection),
            )

            // UI Systems - Phase 5
            .add_systems(
                Update,
                (
                    // Mouse selection needs to generate events
                    handle_node_selection.in_set(GraphSystemSet::Input),
                    // Inspector UI runs after all state updates
                    graph_inspector_ui.in_set(GraphSystemSet::UI),
                ),
            )

            // Rendering Systems - Phase 6 (PostUpdate)
            .add_systems(
                PostUpdate,
                (
                    clear_rendering_on_view_change
                        .run_if(view_mode_changed),
                    render_reference_grid
                        .run_if(view_mode_changed),
                    render_graph_nodes
                        .run_if(nodes_changed),
                    render_graph_edges
                        .run_if(edges_changed),
                )
                    .chain()
                    .in_set(GraphSystemSet::RenderPrep),
            )

            // Algorithm demo - run occasionally
            .add_systems(
                Update,
                demonstrate_algorithms
                    .run_if(|frame: Res<FrameCount>| frame.0 % 300 == 0)
                    .after(GraphSystemSet::StateUpdate),
            )

            // Reset change flags at end of frame
            .add_systems(Last, reset_change_flags);
    }
}
