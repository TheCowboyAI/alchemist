use super::{control_panel::*, inspector_panel::*};
use super::menu_bar::{menu_bar_system, show_keyboard_shortcuts_help, panel_configuration_system};
use super::panel_manager::PanelManager;
use crate::system_sets::GraphSystemSet;
use bevy::prelude::*;

/// Plugin for managing UI panels separately from the 3D viewport
pub struct UiPanelsPlugin;

impl Plugin for UiPanelsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources for panel state
            .init_resource::<ControlPanelState>()
            .init_resource::<InspectorPanelState>()
            .init_resource::<PanelManager>()
            // UI systems - all run in the UI phase after state is stable
            .add_systems(
                Update,
                (
                    // Menu bar first to handle keyboard shortcuts
                    menu_bar_system,
                    // Panel configuration system
                    panel_configuration_system,
                    // Main panel systems
                    control_panel_system,
                    inspector_panel_system,
                    // Help system
                    show_keyboard_shortcuts_help,
                )
                    .chain() // Ensure proper ordering within UI phase
                    .in_set(GraphSystemSet::UI),
            );
    }
}
