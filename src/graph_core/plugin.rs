use super::algorithms::{GraphAlgorithms, demonstrate_algorithms};
use super::change_detection::{
    GraphChangeTracker, detect_component_changes, process_graph_changes,
};
use super::components::*;
use super::events::*;
use super::graph_data::GraphData;
use super::merkle_dag::MerkleDag;
use super::rendering::*;
use super::systems::*;
use super::ui::{
    GraphInspectorState, graph_inspector_ui, handle_node_selection, update_selection_highlights,
};
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
            // Systems - ordered for proper execution
            .add_systems(
                Update,
                (
                    // Event handlers first - use the new graph-based handlers
                    handle_create_node_with_graph,
                    handle_create_edge_with_graph,
                    handle_move_node_events,
                    handle_selection_events,
                    handle_hover_events,
                    // handle_pattern_creation, // TODO: Implement graph_patterns module
                    handle_validation_events,
                    // Process deferred edges after nodes are created
                    process_deferred_edges,
                    // Synchronize edge entities with graph data
                    synchronize_edge_entities,
                    // Then update systems
                    update_edge_positions,
                    update_node_visuals,
                    // Change detection
                    detect_component_changes,
                    process_graph_changes,
                    // UI interaction
                    handle_node_selection,
                    update_selection_highlights,
                )
                    .chain(),
            )
            // UI systems
            .add_systems(
                Update,
                (
                    graph_inspector_ui,
                    // Run algorithm demo periodically (optional)
                    demonstrate_algorithms.run_if(|frame: Res<FrameCount>| frame.0 % 300 == 0),
                ),
            )
            // Rendering systems - run after update
            .add_systems(
                PostUpdate,
                (
                    clear_rendering_on_view_change,
                    render_reference_grid,
                    render_graph_nodes,
                    render_graph_edges,
                    render_graph_edges_from_data,
                )
                    .chain(),
            );
    }
}
