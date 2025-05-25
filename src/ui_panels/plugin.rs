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
            // UI systems need to run every frame for responsiveness, but with optimizations
            .add_systems(
                Update,
                (
                    // Menu bar first to handle keyboard shortcuts
                    menu_bar_system,
                    // Panel configuration system
                    panel_configuration_system,
                    // Main panel systems - run only when needed
                    control_panel_system.run_if(resource_exists::<ControlPanelState>),
                    inspector_panel_system.run_if(resource_exists::<InspectorPanelState>),
                    // Help system - only when F12 is pressed
                    show_keyboard_shortcuts_help,
                )
                .chain() // Ensure proper ordering
                .after(bevy_egui::EguiPreUpdateSet::InitContexts)
                .before(bevy_egui::EguiPreUpdateSet::ProcessInput),
            );
    }
}
