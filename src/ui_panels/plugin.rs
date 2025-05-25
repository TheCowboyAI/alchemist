use super::{control_panel::*, inspector_panel::*};
use super::menu_bar::{menu_bar_system, show_keyboard_shortcuts_help, panel_configuration_system};
use super::panel_manager::PanelManager;
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
            // UI systems - run after egui setup but before rendering
            .add_systems(
                Update,
                (
                    menu_bar_system,
                    panel_configuration_system,
                    control_panel_system,
                    inspector_panel_system,
                    show_keyboard_shortcuts_help,
                )
                .after(bevy_egui::EguiPreUpdateSet::InitContexts)
                .before(bevy_egui::EguiPreUpdateSet::ProcessInput),
            );
    }
}
