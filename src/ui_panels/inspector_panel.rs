use crate::resources::{GraphInspectorState, GraphState};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

/// State for the inspector panel
#[derive(Resource, Default)]
pub struct InspectorPanelState {
    pub visible: bool,
    pub width: f32,
    pub side: InspectorSide,
}

#[derive(Default)]
pub enum InspectorSide {
    #[default]
    Right,
    Left,
}

impl InspectorPanelState {
    pub fn new() -> Self {
        Self {
            visible: true,
            width: 300.0,
            side: InspectorSide::Right,
        }
    }
}

/// System for the inspector panel
pub fn inspector_panel_system(
    mut contexts: EguiContexts,
    mut panel_state: ResMut<InspectorPanelState>,
    mut inspector_state: ResMut<GraphInspectorState>,
    graph_state: Res<GraphState>,
    node_query: Query<(&crate::graph_core::GraphNode, &Transform)>,
) {
    // Only log when visibility actually changes
    if panel_state.is_changed() {
        debug!("Inspector panel visibility changed to: {}", panel_state.visible);
    }

    // Only update if panel is visible
    if !panel_state.visible {
        // Show a small floating button when panel is hidden
        egui::Window::new("Show Inspector Panel")
            .title_bar(false)
            .resizable(false)
            .default_pos([10.0, 90.0])
            .default_size([120.0, 30.0])
            .show(contexts.ctx_mut(), |ui| {
                if ui.button("🔍 Show Inspector (F2)").clicked() {
                    panel_state.visible = true;
                }
            });
        return;
    }

    // Only show panel when visible
    let panel = match panel_state.side {
        InspectorSide::Right => egui::SidePanel::right("inspector_panel"),
        InspectorSide::Left => egui::SidePanel::left("inspector_panel"),
    };

    panel
        .default_width(panel_state.width)
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            // Header with close button
            ui.horizontal(|ui| {
                ui.heading("Graph Inspector");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✖").clicked() {
                        panel_state.visible = false;
                    }
                });
            });

            ui.separator();

            // Search box
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut inspector_state.search_filter);
            });

            ui.separator();

            // Graph statistics
            ui.collapsing("📊 Graph Statistics", |ui| {
                ui.label(format!("Total Nodes: {}", graph_state.node_count));
                ui.label(format!("Total Edges: {}", graph_state.edge_count));
                ui.label(format!("Selected Nodes: {}", graph_state.selected_nodes.len()));

                if let Some(hovered) = graph_state.hovered_entity {
                    ui.label(format!("Hovered: {:?}", hovered));
                } else {
                    ui.label("Hovered: None");
                }
            });

            ui.separator();

            // Selected node details
            if let Some(selected_node_id) = inspector_state.selected_node {
                ui.collapsing("🔍 Selected Node", |ui| {
                    ui.label(format!("ID: {:?}", selected_node_id));

                    // Find the node entity and display details
                    for (node, transform) in &node_query {
                        if node.id == selected_node_id {
                            ui.label(format!("Name: {}", node.name));
                            ui.label(format!("Type: {:?}", node.domain_type));
                            ui.label(format!("Position: {:.2}, {:.2}, {:.2}",
                                transform.translation.x,
                                transform.translation.y,
                                transform.translation.z
                            ));

                            if !node.labels.is_empty() {
                                ui.label("Labels:");
                                for label in &node.labels {
                                    ui.label(format!("  • {}", label));
                                }
                            }

                            if !node.properties.is_empty() {
                                ui.label("Properties:");
                                for (key, value) in &node.properties {
                                    ui.label(format!("  • {}: {}", key, value));
                                }
                            }

                            ui.separator();

                            // Quick actions
                            ui.horizontal(|ui| {
                                if ui.button("Set as Path Source").clicked() {
                                    inspector_state.pathfind_source = Some(selected_node_id);
                                    info!("Set {} as pathfinding source", node.name);
                                }
                                if ui.button("Set as Path Target").clicked() {
                                    inspector_state.pathfind_target = Some(selected_node_id);
                                    info!("Set {} as pathfinding target", node.name);
                                }
                            });

                            break;
                        }
                    }

                    if ui.button("Deselect").clicked() {
                        inspector_state.selected_node = None;
                    }
                });
                ui.separator();
            }

            // Selected edge details
            if let Some(selected_edge_id) = inspector_state.selected_edge {
                ui.collapsing("🔗 Selected Edge", |ui| {
                    ui.label(format!("ID: {:?}", selected_edge_id));
                    // TODO: Add edge details when edge selection is implemented

                    if ui.button("Deselect").clicked() {
                        inspector_state.selected_edge = None;
                    }
                });
                ui.separator();
            }

            // Node list with filtering
            ui.collapsing("📋 Node List", |ui| {
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for (node, _transform) in &node_query {
                            // Apply search filter
                            if !inspector_state.search_filter.is_empty() {
                                let filter = inspector_state.search_filter.to_lowercase();
                                if !node.name.to_lowercase().contains(&filter)
                                    && !node.labels.iter().any(|l| l.to_lowercase().contains(&filter)) {
                                    continue;
                                }
                            }

                            let is_selected = inspector_state.selected_node == Some(node.id);
                            let button_text = if is_selected {
                                format!("🔸 {}", node.name)
                            } else {
                                format!("🔹 {}", node.name)
                            };

                            if ui.button(button_text).clicked() {
                                if is_selected {
                                    inspector_state.selected_node = None;
                                } else {
                                    inspector_state.selected_node = Some(node.id);
                                }
                            }
                        }
                    });
            });

            // Only update panel width if it actually changed significantly
            let new_width = ui.available_width();
            if (new_width - panel_state.width).abs() > 5.0 {
                panel_state.width = new_width;
            }
        });
}
