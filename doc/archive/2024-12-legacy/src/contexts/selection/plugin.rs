use super::domain::*;
use super::events::*;
use super::services::*;
use bevy::prelude::*;

/// Plugin for the Selection bounded context
pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<SelectionState>()

            // Events
            .add_event::<SelectionChanged>()
            .add_event::<SelectionCleared>()
            .add_event::<SelectionModeChanged>()
            .add_event::<NodeSelected>()
            .add_event::<NodeDeselected>()
            .add_event::<EdgeSelected>()
            .add_event::<EdgeDeselected>()
            .add_event::<BoxSelectionStarted>()
            .add_event::<BoxSelectionUpdated>()
            .add_event::<BoxSelectionCompleted>()
            .add_event::<BoxSelectionCancelled>()
            .add_event::<EntityHovered>()
            .add_event::<EntityUnhovered>()
            .add_event::<AllSelected>()
            .add_event::<SelectionInverted>()
            .add_event::<ConnectedNodesSelected>()

            // Selection management systems
            .add_systems(
                Update,
                (
                    ManageSelection::handle_node_selected,
                    ManageSelection::handle_node_deselected,
                    ManageSelection::handle_edge_selected,
                    ManageSelection::handle_selection_cleared,
                    ManageSelection::handle_selection_mode_changed,
                )
                .chain() // Ensure proper ordering
            )

            // Highlighting systems
            .add_systems(
                Update,
                (
                    HighlightSelection::apply_selection_highlight,
                    HighlightSelection::remove_selection_highlight,
                    HighlightSelection::apply_hover_highlight,
                    HighlightSelection::remove_hover_highlight,
                    HighlightSelection::handle_entity_hovered,
                    HighlightSelection::handle_entity_unhovered,
                )
            )

            // Input processing systems
            // These need to run after animation systems to get correct transformed positions
            .add_systems(
                Update,
                (
                    ProcessSelectionInput::handle_mouse_selection,
                    ProcessSelectionInput::handle_keyboard_selection,
                )
                // Run after all animation systems from visualization context
                .after(crate::contexts::visualization::services::AnimateGraphElements::animate_graphs)
                .after(crate::contexts::visualization::services::AnimateGraphElements::animate_nodes)
                .after(crate::contexts::visualization::services::AnimateGraphElements::animate_edges)
            )

            // Box selection systems
            .add_systems(
                Update,
                (
                    PerformBoxSelection::handle_box_selection_started,
                    PerformBoxSelection::handle_box_selection_updated,
                    PerformBoxSelection::handle_box_selection_completed,
                )
                .chain()
                // Also run after animations for accurate positions
                .after(crate::contexts::visualization::services::AnimateGraphElements::animate_nodes)
            )

            // Advanced selection systems
            .add_systems(
                Update,
                (
                    AdvancedSelection::handle_all_selected,
                    AdvancedSelection::handle_selection_inverted,
                    AdvancedSelection::handle_connected_nodes_selected,
                )
            );

        info!("Selection plugin initialized");
    }
}
