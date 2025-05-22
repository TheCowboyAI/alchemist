use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

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
            
            // Create toggle buttons for each editor
            if ui.selectable_label(dashboard_state.standard_graph_editor_active, "Graph Editor").clicked() {
                dashboard_state.standard_graph_editor_active = !dashboard_state.standard_graph_editor_active;
                std_graph_events.send(ToggleStandardGraphEditorEvent(dashboard_state.standard_graph_editor_active));
            }
            
            if ui.selectable_label(dashboard_state.workflow_editor_active, "Workflow Editor").clicked() {
                dashboard_state.workflow_editor_active = !dashboard_state.workflow_editor_active;
                workflow_events.send(ToggleWorkflowEditorEvent(dashboard_state.workflow_editor_active));
            }
            
            if ui.selectable_label(dashboard_state.ddd_editor_active, "DDD Editor").clicked() {
                dashboard_state.ddd_editor_active = !dashboard_state.ddd_editor_active;
                ddd_events.send(ToggleDddEditorEvent(dashboard_state.ddd_editor_active));
            }
            
            if ui.selectable_label(dashboard_state.ecs_editor_active, "ECS Editor").clicked() {
                dashboard_state.ecs_editor_active = !dashboard_state.ecs_editor_active;
                ecs_events.send(ToggleEcsEditorEvent(dashboard_state.ecs_editor_active));
            }
            
            // Add a new editor button
            ui.add_space(20.0);
            if ui.button("➕ New Editor").clicked() {
                // Open a menu to select which type to create
                ui.menu_button("New Editor", |ui| {
                    if ui.button("Standard Graph").clicked() {
                        dashboard_state.standard_graph_editor_active = true;
                        std_graph_events.send(ToggleStandardGraphEditorEvent(true));
                        ui.close_menu();
                    }
                    if ui.button("Workflow").clicked() {
                        dashboard_state.workflow_editor_active = true;
                        workflow_events.send(ToggleWorkflowEditorEvent(true));
                        ui.close_menu();
                    }
                    if ui.button("DDD").clicked() {
                        dashboard_state.ddd_editor_active = true;
                        ddd_events.send(ToggleDddEditorEvent(true));
                        ui.close_menu();
                    }
                    if ui.button("ECS").clicked() {
                        dashboard_state.ecs_editor_active = true;
                        ecs_events.send(ToggleEcsEditorEvent(true));
                        ui.close_menu();
                    }
                });
            }
        });
    });
} 