use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

// Dashboard state to track which editors are active
#[derive(Resource, Default)]
pub struct DashboardState {
    pub standard_graph_editor_active: bool,
    pub workflow_editor_active: bool,
    pub ddd_editor_active: bool,
    pub ecs_editor_active: bool,
}

// Events for toggling editor visibility
#[derive(Event)]
pub struct ToggleStandardGraphEditorEvent(pub bool);

#[derive(Event)]
pub struct ToggleWorkflowEditorEvent(pub bool);

#[derive(Event)]
pub struct ToggleDddEditorEvent(pub bool);

#[derive(Event)]
pub struct ToggleEcsEditorEvent(pub bool);

// Dashboard UI Plugin
#[derive(Default)]
pub struct DashboardUiPlugin;

impl Plugin for DashboardUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DashboardState>()
            .add_event::<ToggleStandardGraphEditorEvent>()
            .add_event::<ToggleWorkflowEditorEvent>()
            .add_event::<ToggleDddEditorEvent>()
            .add_event::<ToggleEcsEditorEvent>()
            .add_systems(Update, dashboard_ui_system);
    }
}

// Main dashboard UI system
fn dashboard_ui_system(
    mut contexts: EguiContexts,
    mut dashboard_state: ResMut<DashboardState>,
    mut std_graph_events: EventWriter<ToggleStandardGraphEditorEvent>,
    mut workflow_events: EventWriter<ToggleWorkflowEditorEvent>,
    mut ddd_events: EventWriter<ToggleDddEditorEvent>,
    mut ecs_events: EventWriter<ToggleEcsEditorEvent>,
) {
    // Create a top panel for the dashboard
    egui::TopBottomPanel::top("dashboard_panel").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.heading("Alchemist Editor Dashboard");

            // Add some spacing
            ui.add_space(20.0);

            // Top bar menu
            ui.horizontal(|ui| {
                // Toggle active editor buttons
                if ui.button("Graph Editor").clicked() {
                    dashboard_state.standard_graph_editor_active =
                        !dashboard_state.standard_graph_editor_active;
                    std_graph_events.write(ToggleStandardGraphEditorEvent(
                        dashboard_state.standard_graph_editor_active,
                    ));
                }

                if ui.button("Workflow Editor").clicked() {
                    dashboard_state.workflow_editor_active =
                        !dashboard_state.workflow_editor_active;
                    workflow_events.write(ToggleWorkflowEditorEvent(
                        dashboard_state.workflow_editor_active,
                    ));
                }

                if ui.button("DDD Editor").clicked() {
                    dashboard_state.ddd_editor_active = !dashboard_state.ddd_editor_active;
                    ddd_events.write(ToggleDddEditorEvent(dashboard_state.ddd_editor_active));
                }

                if ui.button("ECS Editor").clicked() {
                    dashboard_state.ecs_editor_active = !dashboard_state.ecs_editor_active;
                    ecs_events.write(ToggleEcsEditorEvent(dashboard_state.ecs_editor_active));
                }

                ui.separator();

                // Create new editors from templates
                ui.menu_button("New", |ui| {
                    if ui.button("Graph").clicked() {
                        dashboard_state.standard_graph_editor_active = true;
                        std_graph_events.write(ToggleStandardGraphEditorEvent(true));
                        ui.close_menu();
                    }

                    if ui.button("Workflow").clicked() {
                        dashboard_state.workflow_editor_active = true;
                        workflow_events.write(ToggleWorkflowEditorEvent(true));
                        ui.close_menu();
                    }

                    if ui.button("DDD").clicked() {
                        dashboard_state.ddd_editor_active = true;
                        ddd_events.write(ToggleDddEditorEvent(true));
                        ui.close_menu();
                    }

                    if ui.button("ECS").clicked() {
                        dashboard_state.ecs_editor_active = true;
                        ecs_events.write(ToggleEcsEditorEvent(true));
                        ui.close_menu();
                    }
                });
            });
        });
    });
}
