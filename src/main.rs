use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin};

// Import your existing modules
mod ecs;
mod events;
mod graph;
mod graph_patterns;
mod models;

// Import the new unified graph editor system
mod graph_types;
mod unified_graph_editor;
mod theming;

// Keep some useful modules
mod graph_layout;
mod dashboard_ui;

// Import the new unified system
use unified_graph_editor::{
    UnifiedGraphEditorPlugin, 
    AddPatternToBaseGraphEvent,
    AddNodeToBaseGraphEvent,
    SwitchEditorModeEvent,
    ResetBaseGraphEvent,
    ViewMode,
    EditorState,
    EditorMode,
    BaseGraphResource,
};
use graph_patterns::GraphPattern;
use graph_layout::GraphLayoutPlugin;
use dashboard_ui::DashboardUiPlugin;
use ecs::EcsEditorPlugin;
use theming::{ThemingPlugin, AlchemistTheme, theme_selector_ui};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Alchemist Graph Editor".to_string(),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        })
        .set(ImagePlugin::default_nearest())
        )
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: false })
        // Add the new unified graph editor system
        .add_plugins(UnifiedGraphEditorPlugin)
        .add_plugins(ThemingPlugin)
        .add_plugins(GraphLayoutPlugin)
        .add_plugins(DashboardUiPlugin)
        .add_plugins(EcsEditorPlugin)
        .add_systems(Update, (
            unified_ui_system,
        ))
        .run();
}

/// Unified UI system for the new graph editor
fn unified_ui_system(
    mut contexts: EguiContexts,
    mut add_pattern_events: EventWriter<AddPatternToBaseGraphEvent>,
    mut add_node_events: EventWriter<AddNodeToBaseGraphEvent>,
    mut switch_mode_events: EventWriter<SwitchEditorModeEvent>,
    mut reset_base_graph_events: EventWriter<ResetBaseGraphEvent>,
    base_graph: Res<BaseGraphResource>,
    editor_state: Res<EditorState>,
    editor_mode: Res<EditorMode>,
    mut theme: ResMut<AlchemistTheme>,
) {
    // Main control panel with fixed width to prevent jumping
    egui::SidePanel::left("control_panel")
        .min_width(300.0)
        .max_width(300.0)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            // Use a scrollable area to prevent height changes from affecting layout
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.heading("Alchemist Graph Editor");
                    
                    ui.separator();
                    
                    // View mode controls
                    ui.label("View Mode:");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(
                            matches!(editor_mode.mode, ViewMode::Mode3D), 
                            "🎲 3D View"
                        ).clicked() {
                            switch_mode_events.write(SwitchEditorModeEvent {
                                mode: ViewMode::Mode3D,
                            });
                        }
                        
                        if ui.selectable_label(
                            matches!(editor_mode.mode, ViewMode::Mode2D), 
                            "📄 2D View"
                        ).clicked() {
                            switch_mode_events.write(SwitchEditorModeEvent {
                                mode: ViewMode::Mode2D,
                            });
                        }
                    });
                    
                    ui.separator();
                    
                    // Pattern generation controls
                    ui.label("📐 Add Graph Patterns to Base Graph:");
                    
                    ui.horizontal(|ui| {
                        if ui.button("⭐ Star Pattern").clicked() {
                            add_pattern_events.write(AddPatternToBaseGraphEvent {
                                pattern: GraphPattern::Star { points: 6 },
                                name: format!("Star-{}", base_graph.next_subgraph_id),
                            });
                        }
                        
                        if ui.button("🌳 Tree Pattern").clicked() {
                            add_pattern_events.write(AddPatternToBaseGraphEvent {
                                pattern: GraphPattern::Tree { branch_factor: 3, depth: 3 },
                                name: format!("Tree-{}", base_graph.next_subgraph_id),
                            });
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        if ui.button("🔄 Cycle Pattern").clicked() {
                            add_pattern_events.write(AddPatternToBaseGraphEvent {
                                pattern: GraphPattern::Cycle { nodes: 5 },
                                name: format!("Cycle-{}", base_graph.next_subgraph_id),
                            });
                        }
                        
                        if ui.button("🔗 Complete Graph").clicked() {
                            add_pattern_events.write(AddPatternToBaseGraphEvent {
                                pattern: GraphPattern::Complete { nodes: 4 },
                                name: format!("Complete-{}", base_graph.next_subgraph_id),
                            });
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        if ui.button("📊 DAG Pattern").clicked() {
                            add_pattern_events.write(AddPatternToBaseGraphEvent {
                                pattern: GraphPattern::DirectedAcyclicGraph { levels: 3, nodes_per_level: 2 },
                                name: format!("DAG-{}", base_graph.next_subgraph_id),
                            });
                        }
                        
                        if ui.button("🤖 Moore Machine").clicked() {
                            add_pattern_events.write(AddPatternToBaseGraphEvent {
                                pattern: GraphPattern::MooreMachine,
                                name: format!("Moore-{}", base_graph.next_subgraph_id),
                            });
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        if ui.button("🔷 Grid Pattern").clicked() {
                            add_pattern_events.write(AddPatternToBaseGraphEvent {
                                pattern: GraphPattern::Grid { width: 3, height: 3 },
                                name: format!("Grid-{}", base_graph.next_subgraph_id),
                            });
                        }
                        
                        if ui.button("🎭 Bipartite").clicked() {
                            add_pattern_events.write(AddPatternToBaseGraphEvent {
                                pattern: GraphPattern::Bipartite { left_nodes: 3, right_nodes: 3, edge_density: 0.7 },
                                name: format!("Bipartite-{}", base_graph.next_subgraph_id),
                            });
                        }
                    });
                    
                    ui.separator();
                    
                    // Theme settings
                    theme_selector_ui(ui, &mut theme);
                    
                    ui.separator();
                    
                    // Base graph status - fixed height section
                    ui.group(|ui| {
                        ui.label("📈 Base Graph Status:");
                        ui.label(format!("  Nodes: {}", base_graph.graph.nodes.len()));
                        ui.label(format!("  Edges: {}", base_graph.graph.edges.len()));
                        ui.label(format!("  Subgraphs: {}", base_graph.subgraphs.len()));
                    });
                    
                    // Reset controls
                    ui.horizontal(|ui| {
                        if ui.button("🗑 Reset Base Graph").clicked() {
                            reset_base_graph_events.write(ResetBaseGraphEvent);
                        }
                    });
                    
                    ui.separator();
                    
                    // Subgraph information - constrained height
                    if !base_graph.subgraphs.is_empty() {
                        ui.label("🎨 Subgraphs in Base Graph:");
                        
                        // Limit the height of subgraph list to prevent jumping
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                for (id, subgraph) in &base_graph.subgraphs {
                                    let selected = editor_state.selected_subgraph == Some(*id);
                                    let color_circle = "🔴"; // Simple colored circle emoji
                                    
                                    let text = format!("{} {} ({} nodes)", 
                                                     color_circle, subgraph.name, subgraph.nodes.len());
                                    
                                    ui.selectable_label(selected, &text);
                                }
                            });
                    }
                    
                    ui.separator();
                    
                    // Manual node addition - fixed height section
                    ui.group(|ui| {
                        if let Some(selected_subgraph) = editor_state.selected_subgraph {
                            ui.label("➕ Add Node to Selected Subgraph:");
                            
                            if ui.button("Add Node").clicked() {
                                add_node_events.write(AddNodeToBaseGraphEvent {
                                    name: format!("Node-{}", base_graph.graph.nodes.len() + 1),
                                    labels: vec!["manual".to_string()],
                                    position: Some(Vec3::new(0.0, 0.0, 0.0)),
                                    subgraph_id: Some(selected_subgraph),
                                });
                            }
                        } else {
                            ui.label("➕ Add Standalone Node:");
                            
                            if ui.button("Add Node").clicked() {
                                add_node_events.write(AddNodeToBaseGraphEvent {
                                    name: format!("Node-{}", base_graph.graph.nodes.len() + 1),
                                    labels: vec!["standalone".to_string()],
                                    position: Some(Vec3::new(0.0, 0.0, 0.0)),
                                    subgraph_id: None,
                                });
                            }
                        }
                    });
                    
                    ui.separator();
                    
                    // Instructions - collapsible to save space
                    ui.collapsing("📋 Instructions", |ui| {
                        ui.label("• Click pattern buttons to ADD to base graph");
                        ui.label("• Each pattern becomes a colored subgraph");
                        ui.label("• Switch between 2D and 3D projections");
                        ui.label("• Reset clears the entire base graph");
                        if matches!(editor_mode.mode, ViewMode::Mode2D) {
                            ui.label("• Use WASD or arrows to navigate in 2D");
                        } else {
                            ui.label("• Mouse to orbit/pan/zoom in 3D");
                        }
                    });
                });
        });
    
    // Status bar with fixed height
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(25.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Mode: {:?}", editor_mode.mode));
                ui.separator();
                ui.label(format!("Base Graph: {} nodes, {} edges", 
                                base_graph.graph.nodes.len(), 
                                base_graph.graph.edges.len()));
                ui.separator();
                ui.label(format!("Subgraphs: {}", base_graph.subgraphs.len()));
                if let Some(_) = editor_state.selected_subgraph {
                    ui.separator();
                    ui.label("Subgraph Selected");
                }
            });
        });
}