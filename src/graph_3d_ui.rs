use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::graph_3d::{Graph3DState, CreateGraphPatternEvent};
use crate::graph_patterns::{PatternCategory, GraphPattern};

// Plugin for the 3D graph UI
pub struct Graph3DUiPlugin;

impl Plugin for Graph3DUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, graph_3d_ui_system);
    }
}

// System to render the UI for the 3D graph
fn graph_3d_ui_system(
    mut contexts: EguiContexts,
    graph_state: Res<Graph3DState>,
    mut event_writer: EventWriter<CreateGraphPatternEvent>,
) {
    egui::Window::new("3D Graph Editor")
        .default_width(300.0)
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Graph Controls");
            
            if ui.button("Create Decision Workflow").clicked() {
                event_writer.send(CreateGraphPatternEvent {
                    pattern: GraphPattern::DirectedAcyclicGraph { 
                        levels: 3, 
                        nodes_per_level: 2,
                    },
                });
            }
            
            ui.separator();
            ui.heading("Graph Patterns");
            
            // Show patterns by category
            ui.collapsing("Basic Patterns", |ui| {
                show_category_patterns(ui, &graph_state, PatternCategory::Basic, &mut event_writer);
            });
            
            ui.collapsing("Algorithmic Patterns", |ui| {
                show_category_patterns(ui, &graph_state, PatternCategory::Algorithmic, &mut event_writer);
            });
            
            ui.collapsing("Structural Patterns", |ui| {
                show_category_patterns(ui, &graph_state, PatternCategory::Structural, &mut event_writer);
            });
            
            ui.collapsing("Modeling Patterns", |ui| {
                show_category_patterns(ui, &graph_state, PatternCategory::Modeling, &mut event_writer);
            });
            
            ui.separator();
            
            // Node information section
            ui.heading("Selected Node");
            if let Some(selected_id) = graph_state.selected_node {
                if let Some(node) = graph_state.graph.get_node(selected_id) {
                    ui.label(format!("Name: {}", node.name));
                    
                    ui.collapsing("Labels", |ui| {
                        for label in &node.labels {
                            ui.label(format!("• {}", label));
                        }
                    });
                    
                    ui.collapsing("Properties", |ui| {
                        for (key, value) in &node.properties {
                            ui.label(format!("• {}: {}", key, value));
                        }
                    });
                }
            } else {
                ui.label("No node selected");
            }
        });
}

// Helper function to show patterns in a category
fn show_category_patterns(
    ui: &mut egui::Ui,
    graph_state: &Res<Graph3DState>,
    category: PatternCategory,
    event_writer: &mut EventWriter<CreateGraphPatternEvent>,
) {
    let pattern_keys = graph_state.pattern_catalog.get_keys_by_category(category);
    
    for &key in &pattern_keys {
        if let Some(pattern) = graph_state.pattern_catalog.get_pattern(key) {
            let name = pattern.name();
            
            if ui.button(name).clicked() {
                event_writer.send(CreateGraphPatternEvent {
                    pattern: pattern.clone(),
                });
            }
            
            if ui.is_item_hovered() {
                ui.tooltip(|ui| {
                    ui.text(pattern.description());
                });
            }
        }
    }
} 