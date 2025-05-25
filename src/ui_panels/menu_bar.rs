use super::{ControlPanelState, InspectorPanelState};
use super::panel_manager::{PanelManager, WorkspaceMode};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

/// System for the top menu bar with panel controls
pub fn menu_bar_system(
    mut contexts: EguiContexts,
    mut control_panel_state: ResMut<ControlPanelState>,
    mut inspector_panel_state: ResMut<InspectorPanelState>,
    mut panel_manager: ResMut<PanelManager>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Handle keyboard shortcuts for workspaces
    if keyboard_input.just_pressed(KeyCode::F1) {
        panel_manager.toggle_panel("control");
    }
    if keyboard_input.just_pressed(KeyCode::F2) {
        panel_manager.toggle_panel("inspector");
    }
    if keyboard_input.just_pressed(KeyCode::F3) {
        panel_manager.set_workspace(WorkspaceMode::Minimal);
    }
    if keyboard_input.just_pressed(KeyCode::F4) {
        panel_manager.set_workspace(WorkspaceMode::Standard);
    }
    if keyboard_input.just_pressed(KeyCode::F5) {
        panel_manager.set_workspace(WorkspaceMode::Advanced);
    }

    // Sync panel states with panel manager
    control_panel_state.visible = panel_manager.panels.control_panel.visible;
    inspector_panel_state.visible = panel_manager.panels.inspector_panel.visible;

    // Top menu bar
    egui::TopBottomPanel::top("menu_bar").show(contexts.ctx_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            // File menu
            ui.menu_button("File", |ui| {
                if ui.button("New Graph").clicked() {
                    info!("New graph requested");
                    ui.close_menu();
                }
                if ui.button("Open...").clicked() {
                    info!("Open graph requested");
                    ui.close_menu();
                }
                if ui.button("Save").clicked() {
                    info!("Save graph requested");
                    ui.close_menu();
                }
                if ui.button("Save As...").clicked() {
                    info!("Save as requested");
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    info!("Exit requested");
                    ui.close_menu();
                }
            });

            // View menu with workspace management
            ui.menu_button("View", |ui| {
                ui.label("🏢 Workspaces:");
                ui.separator();

                if ui.selectable_label(
                    panel_manager.current_workspace == WorkspaceMode::Minimal,
                    "🎯 Minimal (F3)"
                ).clicked() {
                    panel_manager.set_workspace(WorkspaceMode::Minimal);
                    ui.close_menu();
                }

                if ui.selectable_label(
                    panel_manager.current_workspace == WorkspaceMode::Standard,
                    "⚖️ Standard (F4)"
                ).clicked() {
                    panel_manager.set_workspace(WorkspaceMode::Standard);
                    ui.close_menu();
                }

                if ui.selectable_label(
                    panel_manager.current_workspace == WorkspaceMode::Advanced,
                    "🔧 Advanced (F5)"
                ).clicked() {
                    panel_manager.set_workspace(WorkspaceMode::Advanced);
                    ui.close_menu();
                }

                ui.separator();
                ui.label("🎨 Specialized:");

                if ui.selectable_label(
                    panel_manager.current_workspace == WorkspaceMode::DDD,
                    "🏗️ DDD Focus"
                ).clicked() {
                    panel_manager.set_workspace(WorkspaceMode::DDD);
                    ui.close_menu();
                }

                if ui.selectable_label(
                    panel_manager.current_workspace == WorkspaceMode::ECS,
                    "⚙️ ECS Focus"
                ).clicked() {
                    panel_manager.set_workspace(WorkspaceMode::ECS);
                    ui.close_menu();
                }

                if ui.selectable_label(
                    panel_manager.current_workspace == WorkspaceMode::Algorithms,
                    "🧮 Algorithms"
                ).clicked() {
                    panel_manager.set_workspace(WorkspaceMode::Algorithms);
                    ui.close_menu();
                }

                ui.separator();
                ui.label("🔧 Individual Panels:");

                if ui.checkbox(&mut panel_manager.panels.control_panel.visible, "Control Panel (F1)").clicked() {
                    panel_manager.current_workspace = WorkspaceMode::Custom;
                }
                if ui.checkbox(&mut panel_manager.panels.inspector_panel.visible, "Inspector Panel (F2)").clicked() {
                    panel_manager.current_workspace = WorkspaceMode::Custom;
                }
                if ui.checkbox(&mut panel_manager.panels.properties_panel.visible, "Properties Panel").clicked() {
                    panel_manager.current_workspace = WorkspaceMode::Custom;
                }
                if ui.checkbox(&mut panel_manager.panels.algorithms_panel.visible, "Algorithms Panel").clicked() {
                    panel_manager.current_workspace = WorkspaceMode::Custom;
                }
                if ui.checkbox(&mut panel_manager.panels.console_panel.visible, "Console Panel").clicked() {
                    panel_manager.current_workspace = WorkspaceMode::Custom;
                }
                if ui.checkbox(&mut panel_manager.panels.minimap_panel.visible, "Minimap Panel").clicked() {
                    panel_manager.current_workspace = WorkspaceMode::Custom;
                }

                ui.separator();

                if ui.button("🔄 Reset Layout").clicked() {
                    panel_manager.set_workspace(WorkspaceMode::Standard);
                    info!("Layout reset to Standard");
                    ui.close_menu();
                }

                if ui.button("⚙️ Panel Configuration").clicked() {
                    panel_manager.show_panel_config = !panel_manager.show_panel_config;
                    ui.close_menu();
                }
            });

            // Tools menu
            ui.menu_button("Tools", |ui| {
                if ui.button("Graph Algorithms").clicked() {
                    panel_manager.set_workspace(WorkspaceMode::Algorithms);
                    info!("Switched to algorithms workspace");
                    ui.close_menu();
                }
                if ui.button("DDD Editor").clicked() {
                    panel_manager.set_workspace(WorkspaceMode::DDD);
                    info!("Switched to DDD workspace");
                    ui.close_menu();
                }
                if ui.button("ECS Editor").clicked() {
                    panel_manager.set_workspace(WorkspaceMode::ECS);
                    info!("Switched to ECS workspace");
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("🔍 Search Nodes").clicked() {
                    panel_manager.toggle_panel("search");
                    ui.close_menu();
                }
                if ui.button("📚 History").clicked() {
                    panel_manager.toggle_panel("history");
                    ui.close_menu();
                }
                if ui.button("🔖 Bookmarks").clicked() {
                    panel_manager.toggle_panel("bookmarks");
                    ui.close_menu();
                }
            });

            // Help menu
            ui.menu_button("Help", |ui| {
                if ui.button("Keyboard Shortcuts").clicked() {
                    info!("Show keyboard shortcuts");
                    ui.close_menu();
                }
                if ui.button("Workspace Guide").clicked() {
                    info!("Show workspace guide");
                    ui.close_menu();
                }
                if ui.button("About").clicked() {
                    info!("Show about dialog");
                    ui.close_menu();
                }
            });

            // Right-aligned status with workspace indicator
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let workspace_icon = match panel_manager.current_workspace {
                    WorkspaceMode::Minimal => "🎯",
                    WorkspaceMode::Standard => "⚖️",
                    WorkspaceMode::Advanced => "🔧",
                    WorkspaceMode::DDD => "🏗️",
                    WorkspaceMode::ECS => "⚙️",
                    WorkspaceMode::Algorithms => "🧮",
                    WorkspaceMode::Custom => "🎨",
                };

                ui.label(format!("{} {:?} | {} panels",
                    workspace_icon,
                    panel_manager.current_workspace,
                    panel_manager.visible_panel_count()
                ));
            });
        });
    });
}

/// System to show panel configuration window
pub fn panel_configuration_system(
    mut contexts: EguiContexts,
    mut panel_manager: ResMut<PanelManager>,
) {
    if !panel_manager.show_panel_config {
        return;
    }

    egui::Window::new("🔧 Panel Configuration")
        .default_width(500.0)
        .default_height(400.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Workspace Management");

            ui.horizontal(|ui| {
                ui.label("Current workspace:");
                ui.label(format!("{:?}", panel_manager.current_workspace));
            });

            ui.separator();

            ui.heading("Quick Workspace Switch");
            ui.horizontal(|ui| {
                if ui.button("🎯 Minimal").clicked() {
                    panel_manager.set_workspace(WorkspaceMode::Minimal);
                }
                if ui.button("⚖️ Standard").clicked() {
                    panel_manager.set_workspace(WorkspaceMode::Standard);
                }
                if ui.button("🔧 Advanced").clicked() {
                    panel_manager.set_workspace(WorkspaceMode::Advanced);
                }
            });

            ui.horizontal(|ui| {
                if ui.button("🏗️ DDD").clicked() {
                    panel_manager.set_workspace(WorkspaceMode::DDD);
                }
                if ui.button("⚙️ ECS").clicked() {
                    panel_manager.set_workspace(WorkspaceMode::ECS);
                }
                if ui.button("🧮 Algorithms").clicked() {
                    panel_manager.set_workspace(WorkspaceMode::Algorithms);
                }
            });

            ui.separator();

            ui.heading("Individual Panel Controls");

            ui.columns(2, |columns| {
                columns[0].label("Core Panels:");
                columns[0].checkbox(&mut panel_manager.panels.control_panel.visible, "Control Panel");
                columns[0].checkbox(&mut panel_manager.panels.inspector_panel.visible, "Inspector Panel");
                columns[0].checkbox(&mut panel_manager.panels.properties_panel.visible, "Properties Panel");

                columns[1].label("Tool Panels:");
                columns[1].checkbox(&mut panel_manager.panels.algorithms_panel.visible, "Algorithms Panel");
                columns[1].checkbox(&mut panel_manager.panels.console_panel.visible, "Console Panel");
                columns[1].checkbox(&mut panel_manager.panels.minimap_panel.visible, "Minimap Panel");
                columns[1].checkbox(&mut panel_manager.panels.search_panel.visible, "Search Panel");
                columns[1].checkbox(&mut panel_manager.panels.history_panel.visible, "History Panel");
                columns[1].checkbox(&mut panel_manager.panels.bookmarks_panel.visible, "Bookmarks Panel");
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Close").clicked() {
                    panel_manager.show_panel_config = false;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Visible panels: {}", panel_manager.visible_panel_count()));
                });
            });
        });
}

/// System to show keyboard shortcuts help
pub fn show_keyboard_shortcuts_help(
    mut contexts: EguiContexts,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Show help when F12 is held
    if keyboard_input.pressed(KeyCode::F12) {
        egui::Window::new("⌨️ Keyboard Shortcuts")
            .default_width(450.0)
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("Workspace Controls");
                ui.label("F3 - Minimal Workspace (clean view)");
                ui.label("F4 - Standard Workspace (balanced)");
                ui.label("F5 - Advanced Workspace (all panels)");

                ui.separator();

                ui.heading("Panel Controls");
                ui.label("F1 - Toggle Control Panel");
                ui.label("F2 - Toggle Inspector Panel");
                ui.label("F12 - Show this help (hold)");

                ui.separator();

                ui.heading("Camera Controls");
                ui.label("Right Mouse - Orbit camera (3D)");
                ui.label("Middle Mouse - Pan camera");
                ui.label("Scroll - Zoom in/out");
                ui.label("Tab/V - Switch between 2D/3D view");

                ui.separator();

                ui.heading("Graph Interaction");
                ui.label("Left Click - Select node/edge");
                ui.label("Ctrl+Click - Multi-select");
                ui.label("Shift+Click - Add to selection");
                ui.label("Delete - Remove selected items");

                ui.separator();

                ui.heading("Workspace Tips");
                ui.label("• Start with Minimal for focus");
                ui.label("• Use Standard for general work");
                ui.label("• Switch to specialized workspaces for specific tasks");
                ui.label("• Customize panels individually for your workflow");
            });
    }
}
