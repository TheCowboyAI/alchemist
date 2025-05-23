use super::components::*;
use super::events::*;
use super::rendering::*;
use super::systems::*;
use bevy::prelude::*;

/// Plugin for the graph editor functionality
pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<GraphState>()
            .init_resource::<GraphMetadata>()
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
            .add_event::<ValidateGraphEvent>()
            .add_event::<SaveGraphEvent>()
            .add_event::<LoadGraphEvent>()
            // .add_event::<CreatePatternEvent>() // TODO: Implement graph_patterns module
            .add_event::<CreateSubgraphEvent>()
            .add_event::<UndoEvent>()
            .add_event::<RedoEvent>()
            .add_event::<GraphModificationEvent>()
            // Systems - ordered for proper execution
            .add_systems(
                Update,
                (
                    // Event handlers first
                    handle_create_node_events,
                    handle_create_edge_events,
                    handle_move_node_events,
                    handle_selection_events,
                    handle_hover_events,
                    // handle_pattern_creation, // TODO: Implement graph_patterns module
                    handle_validation_events,
                    // Then update systems
                    update_edge_positions,
                    update_node_visuals,
                )
                    .chain(),
            )
            // Rendering systems - run after update
            .add_systems(
                PostUpdate,
                (
                    render_reference_grid,
                    render_graph_nodes,
                    render_graph_edges,
                )
                    .chain(),
            );
    }
}
