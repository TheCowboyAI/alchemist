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
mod json_loader;
mod theming;
mod unified_graph_editor;

// Keep some useful modules
mod graph_layout;

// Import the new camera system
mod camera;

// Import the new graph core module
mod graph_core;

// Import the new unified system
use camera::{CameraViewportPlugin, GraphViewCamera, ViewMode as CameraViewMode};
use ecs::{EcsEditor, EcsEditorPlugin};
use graph_core::{CreateNodeEvent, DomainNodeType, GraphPlugin as GraphCorePlugin};
use graph_layout::GraphLayoutPlugin;
use graph_patterns::GraphPattern;
use json_loader::{
    FileOperationState, JsonFileLoadedEvent, JsonFileSavedEvent, LoadJsonFileEvent,
    SaveJsonFileEvent, handle_json_file_loading, handle_json_file_saving,
};
use theming::{AlchemistTheme, ThemingPlugin, theme_selector_ui};
use unified_graph_editor::{
    AddNodeToBaseGraphEvent, AddPatternToBaseGraphEvent, BaseGraphResource, EditorMode,
    EditorState, ResetBaseGraphEvent, SwitchEditorModeEvent, UnifiedGraphEditorPlugin, ViewMode,
};

/// Different editor contexts/focus areas
#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum EditorContext {
    #[default]
    GraphEditor,
    WorkflowEditor,
    DddEditor,
    EcsEditor,
}

/// Resource to track the current active context
#[derive(Resource, Default)]
pub struct AppState {
    pub current_context: EditorContext,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Alchemist Graph Editor".to_string(),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: false,
        })
        // Add the new unified graph editor system
        .add_plugins(UnifiedGraphEditorPlugin)
        .add_plugins(ThemingPlugin)
        .add_plugins(GraphLayoutPlugin)
        .add_plugins(EcsEditorPlugin)
        .init_resource::<AppState>()
        .init_resource::<FileOperationState>()
        .add_event::<LoadJsonFileEvent>()
        .add_event::<SaveJsonFileEvent>()
        .add_event::<JsonFileLoadedEvent>()
        .add_event::<JsonFileSavedEvent>()
        .add_systems(Startup, setup_file_scanner)
        .add_systems(
            Update,
            (
                unified_ui_system,
                handle_json_file_loading,
                handle_json_file_saving,
            ),
        )
        .run();
}

/// Setup system to scan for available files
fn setup_file_scanner(mut file_state: ResMut<FileOperationState>) {
    file_state.scan_models_directory();
}

/// Unified UI system for the new graph editor with contextual focus areas
fn unified_ui_system(
    mut contexts: EguiContexts,
    mut app_state: ResMut<AppState>,
    mut add_pattern_events: EventWriter<AddPatternToBaseGraphEvent>,
    mut add_node_events: EventWriter<AddNodeToBaseGraphEvent>,
    mut switch_mode_events: EventWriter<SwitchEditorModeEvent>,
    mut reset_base_graph_events: EventWriter<ResetBaseGraphEvent>,
    mut load_json_events: EventWriter<LoadJsonFileEvent>,
    mut save_json_events: EventWriter<SaveJsonFileEvent>,
    base_graph: Res<BaseGraphResource>,
    editor_state: Res<EditorState>,
    editor_mode: Res<EditorMode>,
    mut theme: ResMut<AlchemistTheme>,
    mut ecs_editor: ResMut<EcsEditor>,
    mut file_state: ResMut<FileOperationState>,
) {
    // Top menu bar for context switching
    egui::TopBottomPanel::top("top_menu")
        .exact_height(60.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                ui.heading("🧪 Alchemist Editor Dashboard");

                ui.add_space(40.0);

                // Context switching buttons
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(
                            app_state.current_context == EditorContext::GraphEditor,
                            "📊 Graph Editor",
                        )
                        .clicked()
                    {
                        app_state.current_context = EditorContext::GraphEditor;
                    }

                    if ui
                        .selectable_label(
                            app_state.current_context == EditorContext::WorkflowEditor,
                            "🔄 Workflow Editor",
                        )
                        .clicked()
                    {
                        app_state.current_context = EditorContext::WorkflowEditor;
                    }

                    if ui
                        .selectable_label(
                            app_state.current_context == EditorContext::DddEditor,
                            "🏗️ DDD Editor",
                        )
                        .clicked()
                    {
                        app_state.current_context = EditorContext::DddEditor;
                    }

                    if ui
                        .selectable_label(
                            app_state.current_context == EditorContext::EcsEditor,
                            "⚙️ ECS Editor",
                        )
                        .clicked()
                    {
                        app_state.current_context = EditorContext::EcsEditor;
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Active: {:?}", app_state.current_context));
                });
            });
        });

    // Main content area - changes based on current context
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| match app_state.current_context {
        EditorContext::GraphEditor => {
            show_graph_editor_content(
                ui,
                &mut add_pattern_events,
                &mut add_node_events,
                &mut switch_mode_events,
                &mut reset_base_graph_events,
                &mut load_json_events,
                &mut save_json_events,
                &base_graph,
                &editor_state,
                &editor_mode,
                &mut theme,
                &mut file_state,
            );
        }
        EditorContext::WorkflowEditor => {
            show_workflow_editor_content(ui);
        }
        EditorContext::DddEditor => {
            show_ddd_editor_content(ui);
        }
        EditorContext::EcsEditor => {
            show_ecs_editor_content(ui, &mut ecs_editor);
        }
    });

    // Status bar with fixed height
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(25.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Context: {:?}", app_state.current_context));
                ui.separator();
                ui.label(format!("Mode: {:?}", editor_mode.mode));
                ui.separator();
                ui.label(format!(
                    "Base Graph: {} nodes, {} edges",
                    base_graph.graph.nodes.len(),
                    base_graph.graph.edges.len()
                ));
                ui.separator();
                ui.label(format!("Subgraphs: {}", base_graph.subgraphs.len()));
                if let Some(_) = editor_state.selected_subgraph {
                    ui.separator();
                    ui.label("Subgraph Selected");
                }
            });
        });
}

/// Show content for the Graph Editor context
fn show_graph_editor_content(
    ui: &mut egui::Ui,
    add_pattern_events: &mut EventWriter<AddPatternToBaseGraphEvent>,
    add_node_events: &mut EventWriter<AddNodeToBaseGraphEvent>,
    switch_mode_events: &mut EventWriter<SwitchEditorModeEvent>,
    reset_base_graph_events: &mut EventWriter<ResetBaseGraphEvent>,
    load_json_events: &mut EventWriter<LoadJsonFileEvent>,
    save_json_events: &mut EventWriter<SaveJsonFileEvent>,
    base_graph: &Res<BaseGraphResource>,
    editor_state: &Res<EditorState>,
    editor_mode: &Res<EditorMode>,
    theme: &mut ResMut<AlchemistTheme>,
    file_state: &mut ResMut<FileOperationState>,
) {
    ui.horizontal(|ui| {
        // Left control panel
        ui.vertical(|ui| {
            ui.set_min_width(300.0);
            ui.set_max_width(300.0);
            ui.set_min_height(600.0);

            egui::ScrollArea::vertical()
                .min_scrolled_height(400.0)
                .show(ui, |ui| {
                    ui.heading("Graph Editor Controls");

                    ui.separator();

                    // View mode controls
                    ui.label("View Mode:");
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(
                                matches!(editor_mode.mode, ViewMode::Mode3D),
                                "🎲 3D View",
                            )
                            .clicked()
                        {
                            switch_mode_events.write(SwitchEditorModeEvent {
                                mode: ViewMode::Mode3D,
                            });
                        }

                        if ui
                            .selectable_label(
                                matches!(editor_mode.mode, ViewMode::Mode2D),
                                "📄 2D View",
                            )
                            .clicked()
                        {
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
                                pattern: GraphPattern::Tree {
                                    branch_factor: 3,
                                    depth: 3,
                                },
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
                                pattern: GraphPattern::DirectedAcyclicGraph {
                                    levels: 3,
                                    nodes_per_level: 2,
                                },
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
                                pattern: GraphPattern::Grid {
                                    width: 3,
                                    height: 3,
                                },
                                name: format!("Grid-{}", base_graph.next_subgraph_id),
                            });
                        }

                        if ui.button("🎭 Bipartite").clicked() {
                            add_pattern_events.write(AddPatternToBaseGraphEvent {
                                pattern: GraphPattern::Bipartite {
                                    left_nodes: 3,
                                    right_nodes: 3,
                                    edge_density: 0.7,
                                },
                                name: format!("Bipartite-{}", base_graph.next_subgraph_id),
                            });
                        }
                    });

                    ui.separator();

                    // Theme settings
                    theme_selector_ui(ui, theme);

                    ui.separator();

                    // File operations section
                    ui.group(|ui| {
                        ui.label("📁 File Operations:");

                        // Current file display
                        if let Some(current_file) = &file_state.current_file_path {
                            ui.label(format!(
                                "Current: {}",
                                current_file.split('/').last().unwrap_or("unknown")
                            ));
                        } else {
                            ui.label("No file loaded");
                        }

                        ui.separator();

                        // Available files dropdown
                        ui.label("Available JSON files:");
                        if file_state.available_files.is_empty() {
                            ui.label("No JSON files found in assets/models/");
                            if ui.button("🔄 Refresh").clicked() {
                                file_state.scan_models_directory();
                            }
                        } else {
                            for file_path in &file_state.available_files.clone() {
                                let file_name = file_path.split('/').last().unwrap_or("unknown");
                                if ui.button(format!("📂 Load {}", file_name)).clicked() {
                                    load_json_events.write(LoadJsonFileEvent {
                                        file_path: file_path.clone(),
                                    });
                                }
                            }
                        }

                        ui.separator();

                        // Save options
                        ui.horizontal(|ui| {
                            if ui.button("💾 Save Current").clicked() {
                                if let Some(current_file) = &file_state.current_file_path {
                                    save_json_events.write(SaveJsonFileEvent {
                                        file_path: current_file.clone(),
                                    });
                                }
                            }

                            if ui.button("💾 Save As...").clicked() {
                                let timestamp = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs();
                                let new_file =
                                    format!("assets/models/graph_export_{}.json", timestamp);
                                save_json_events.write(SaveJsonFileEvent {
                                    file_path: new_file,
                                });
                            }
                        });

                        // Show last operation message
                        if !file_state.last_operation_message.is_empty() {
                            ui.separator();
                            ui.label(&file_state.last_operation_message);
                        }
                    });

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

                                    let text = format!(
                                        "{} {} ({} nodes)",
                                        color_circle,
                                        subgraph.name,
                                        subgraph.nodes.len()
                                    );

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

        ui.separator();

        // Right side - visualization area (this is where the 3D/2D scene renders)
        ui.vertical(|ui| {
            ui.heading("Graph Visualization");
            ui.label("The 3D/2D graph visualization renders in the background");
            ui.label(format!("Current view mode: {:?}", editor_mode.mode));

            // Add some visual feedback about what's in the scene
            if base_graph.graph.nodes.is_empty() {
                ui.colored_label(
                    ui.visuals().warn_fg_color,
                    "⚠ No graph data loaded. Add some patterns to see visualization!",
                );
            } else {
                ui.colored_label(
                    ui.visuals().selection.stroke.color,
                    format!(
                        "✓ Displaying {} nodes and {} edges",
                        base_graph.graph.nodes.len(),
                        base_graph.graph.edges.len()
                    ),
                );
            }
        });
    });
}

/// Show content for the Workflow Editor context
fn show_workflow_editor_content(ui: &mut egui::Ui) {
    ui.heading("🔄 Workflow Editor");
    ui.separator();

    ui.label("Workflow editor content will be implemented here.");
    ui.label("This will allow creating and editing workflow graphs.");

    // Placeholder content
    ui.group(|ui| {
        ui.label("Future features:");
        ui.label("• Step-by-step workflow creation");
        ui.label("• Process flow visualization");
        ui.label("• Conditional branching");
        ui.label("• Workflow execution simulation");
    });
}

/// Show content for the DDD Editor context
fn show_ddd_editor_content(ui: &mut egui::Ui) {
    ui.heading("🏗️ Domain-Driven Design Editor");
    ui.separator();

    ui.label("DDD editor content will be implemented here.");
    ui.label("This will help with domain modeling and bounded contexts.");

    // Placeholder content
    ui.group(|ui| {
        ui.label("Future features:");
        ui.label("• Bounded context mapping");
        ui.label("• Aggregate design");
        ui.label("• Entity relationship modeling");
        ui.label("• Event storming support");
    });
}

/// Show content for the ECS Editor context
fn show_ecs_editor_content(ui: &mut egui::Ui, ecs_editor: &mut ResMut<EcsEditor>) {
    ui.heading("⚙️ Entity Component System Editor");
    ui.separator();

    // Show ECS editor content inline instead of as a separate window
    ui.horizontal(|ui| {
        // Left panel - ECS controls
        ui.vertical(|ui| {
            ui.set_min_width(300.0);
            ui.set_max_width(300.0);

            ui.group(|ui| {
                ui.label("📦 Entities:");
                for entity in &ecs_editor.entities {
                    ui.label(format!("• {}", entity));
                }

                if ui.button("Add Entity").clicked() {
                    let entity_name = format!("Entity_{}", ecs_editor.entities.len() + 1);
                    ecs_editor.add_entity(entity_name);
                }
            });

            ui.separator();

            ui.group(|ui| {
                ui.label("🧩 Components:");
                for component in &ecs_editor.components {
                    ui.label(format!("• {}", component));
                }

                if ui.button("Add Component").clicked() {
                    let component_name = format!("Component_{}", ecs_editor.components.len() + 1);
                    ecs_editor.add_component(component_name);
                }
            });

            ui.separator();

            ui.group(|ui| {
                ui.label("⚙️ Systems:");
                for system in &ecs_editor.systems {
                    ui.label(format!("• {}", system));
                }

                if ui.button("Add System").clicked() {
                    let system_name = format!("System_{}", ecs_editor.systems.len() + 1);
                    ecs_editor.add_system(system_name);
                }
            });
        });

        ui.separator();

        // Right panel - ECS visualization
        ui.vertical(|ui| {
            ui.heading("ECS Graph Visualization");
            ui.label("ECS relationship graph visualization will be shown here.");
            ui.label(format!(
                "Entities: {}, Components: {}, Systems: {}",
                ecs_editor.entities.len(),
                ecs_editor.components.len(),
                ecs_editor.systems.len()
            ));
        });
    });
}
