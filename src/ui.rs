use crate::app_state::{AlchemistAppState, ViewType};
use crate::viewport::{ViewportState, create_new_viewport, update_immediate_viewports};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

pub struct UiPlugin;
pub struct tmp;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_system
            .after(bevy_egui::EguiPreUpdateSet::InitContexts)
            .before(bevy_egui::EguiPreUpdateSet::ProcessInput));
    }
}

// Main UI system that handles egui integration
pub fn ui_system(
    mut contexts: EguiContexts,
    mut app_state: ResMut<AlchemistAppState>,
    mut viewport_state: ResMut<ViewportState>,
) {
    let ctx = contexts.ctx_mut();

    // Create the main menu bar
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Graph").clicked() {
                    app_state.create_new_graph();
                }
                if ui.button("New Viewport").clicked() {
                    create_new_viewport(&mut viewport_state);
                }
            });

            ui.menu_button("View", |ui| {
                if ui
                    .selectable_label(app_state.current_view == ViewType::Graph, "Graph View")
                    .clicked()
                {
                    app_state.current_view = ViewType::Graph;
                }
                if ui
                    .selectable_label(
                        app_state.current_view == ViewType::Workflow,
                        "Workflow View",
                    )
                    .clicked()
                {
                    app_state.current_view = ViewType::Workflow;
                }
                if ui
                    .selectable_label(app_state.current_view == ViewType::ThreeD, "3D View")
                    .clicked()
                {
                    app_state.current_view = ViewType::ThreeD;
                }
                if ui
                    .selectable_label(app_state.current_view == ViewType::Events, "Events View")
                    .clicked()
                {
                    app_state.current_view = ViewType::Events;
                }
            });
        });
    });

    // Update all immediate viewports
    update_immediate_viewports(ctx, &mut viewport_state);

    // Main central area
    egui::CentralPanel::default().show(ctx, |ui| match app_state.current_view {
        ViewType::Graph => {
            ui.heading("Graph View");
            ui.label("This is the graph visualization panel.");
        }
        ViewType::Workflow => {
            ui.heading("Workflow View");
            ui.label("This is the workflow editor panel.");
        }
        ViewType::ThreeD => {
            ui.heading("3D View");
            ui.label("This is the main 3D scene panel.");
        }
        ViewType::Events => {
            ui.heading("Events View");
            ui.label("This is the events log panel.");
        }
    });
}

// Show the about window using an immediate viewport
pub fn show_about_window(ctx: &egui::Context) {
    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("about_window"),
        egui::ViewportBuilder::default()
            .with_title("About Alchemist")
            .with_inner_size([300.0, 200.0]),
        |ctx, class| {
            if class != egui::ViewportClass::Immediate {
                return;
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Alchemist");
                ui.label("Information Graph Workflows");
                ui.label("Version 0.1.0");
                ui.separator();
                ui.label("Created by CowboyAI");

                ui.add_space(10.0);

                if ui.button("Close").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            // Close the viewport if requested
            if ctx.input(|i| i.viewport().close_requested()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        },
    );
}

// Display the graph view
pub fn show_graph_view(ctx: &egui::Context, app_state: &mut ResMut<AlchemistAppState>) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Graph View");

        if ui.button("Switch to Workflow View").clicked() {
            app_state.current_view = ViewType::Workflow;
        }

        ui.separator();

        // Graph view content
        ui.label("This is the main graph view. The 3D visualization is now in separate viewports.");
    });
}

// Display the workflow view
pub fn show_workflow_view(ctx: &egui::Context, app_state: &mut ResMut<AlchemistAppState>) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Workflow View");

        if ui.button("Switch to Graph View").clicked() {
            app_state.current_view = ViewType::Graph;
        }

        ui.separator();

        // Your workflow view content here
        ui.label("This is the workflow editor view.");
    });
}

// Display the 3D view
pub fn show_3d_view(ctx: &egui::Context, app_state: &mut ResMut<AlchemistAppState>) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("3D View");

        if ui.button("Return to Graph View").clicked() {
            app_state.current_view = ViewType::Graph;
        }

        ui.separator();

        // 3D view content
        ui.label("This is the 3D visualization view.");
    });
}

// Display the events view
pub fn show_events_view(ctx: &egui::Context, app_state: &mut ResMut<AlchemistAppState>) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Events");

        if ui.button("Return to Graph View").clicked() {
            app_state.current_view = ViewType::Graph;
        }

        ui.separator();

        // Your events view content here
        ui.label("This is the events view.");
    });
}
