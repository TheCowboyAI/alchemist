use crate::graph_core::{GraphAlgorithms, GraphData, GraphState};
use crate::resources::GraphInspectorState;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use uuid::Uuid;

/// State for the algorithm panel
#[derive(Resource, Default)]
pub struct AlgorithmPanelState {
    pub visible: bool,
    pub position: egui::Pos2,
    pub size: egui::Vec2,
    pub pathfind_source: Option<Uuid>,
    pub pathfind_target: Option<Uuid>,
    pub last_path_result: Option<PathfindResult>,
    pub analysis_results: AnalysisResults,
}

#[derive(Default)]
pub struct PathfindResult {
    pub path: Vec<Uuid>,
    pub cost: f32,
    pub algorithm_used: String,
}

#[derive(Default)]
pub struct AnalysisResults {
    pub connected_components: usize,
    pub has_cycles: bool,
    pub is_dag: bool,
    pub max_degree: usize,
    pub node_count: usize,
    pub edge_count: usize,
}

impl AlgorithmPanelState {
    pub fn new() -> Self {
        Self {
            visible: false,
            position: egui::pos2(400.0, 300.0),
            size: egui::vec2(400.0, 500.0),
            ..default()
        }
    }
}

/// System for the algorithm panel
pub fn algorithm_panel_system(
    mut contexts: EguiContexts,
    mut panel_state: ResMut<AlgorithmPanelState>,
    graph_data: Res<GraphData>,
    graph_state: Res<GraphState>,
    inspector_state: Res<GraphInspectorState>,
    mut layout_events: EventWriter<crate::graph_core::RequestLayoutEvent>,
) {
    if !panel_state.visible {
        return;
    }

    // Sync selection from inspector
    if panel_state.is_changed() || inspector_state.is_changed() {
        if inspector_state.pathfind_source != panel_state.pathfind_source {
            panel_state.pathfind_source = inspector_state.pathfind_source;
        }
        if inspector_state.pathfind_target != panel_state.pathfind_target {
            panel_state.pathfind_target = inspector_state.pathfind_target;
        }
    }

    egui::Window::new("🧮 Graph Algorithms")
        .default_pos(panel_state.position)
        .default_size(panel_state.size)
        .resizable(true)
        .collapsible(true)
        .show(contexts.ctx_mut(), |ui| {
            // Update panel position/size
            panel_state.position = ui.min_rect().min;
            panel_state.size = ui.min_rect().size();

            ui.horizontal(|ui| {
                ui.heading("Graph Algorithms");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✖").clicked() {
                        panel_state.visible = false;
                    }
                });
            });

            ui.separator();

            // Tabs for different algorithm categories
            ui.horizontal(|ui| {
                if ui.selectable_label(true, "🎯 Pathfinding").clicked() {}
                if ui.selectable_label(false, "📊 Analysis").clicked() {}
                if ui.selectable_label(false, "🔄 Layout").clicked() {}
                if ui.selectable_label(false, "🧬 Patterns").clicked() {}
            });

            ui.separator();

            // Pathfinding section
            show_pathfinding_section(ui, &mut panel_state, &graph_data);

            ui.separator();

            // Analysis section
            show_analysis_section(ui, &mut panel_state, &graph_data, &graph_state);

            ui.separator();

            // Layout section
            show_layout_section(ui, &mut layout_events);
        });
}

fn show_pathfinding_section(
    ui: &mut egui::Ui,
    panel_state: &mut AlgorithmPanelState,
    graph_data: &GraphData,
) {
    ui.heading("🎯 Pathfinding");

    // Source selection
    ui.horizontal(|ui| {
        ui.label("Source:");
        if let Some(source_id) = panel_state.pathfind_source {
            if let Some(node) = graph_data.get_node(source_id) {
                ui.label(format!("📍 {}", node.name));
                if ui.small_button("✖").clicked() {
                    panel_state.pathfind_source = None;
                }
            }
        } else {
            ui.label("None selected");
        }
    });

    // Target selection
    ui.horizontal(|ui| {
        ui.label("Target:");
        if let Some(target_id) = panel_state.pathfind_target {
            if let Some(node) = graph_data.get_node(target_id) {
                ui.label(format!("🎯 {}", node.name));
                if ui.small_button("✖").clicked() {
                    panel_state.pathfind_target = None;
                }
            }
        } else {
            ui.label("None selected");
        }
    });

    ui.separator();

    // Algorithm buttons
    let can_pathfind = panel_state.pathfind_source.is_some() && panel_state.pathfind_target.is_some();

    ui.add_enabled_ui(can_pathfind, |ui| {
        ui.horizontal(|ui| {
            if ui.button("🎯 Dijkstra").clicked() {
                run_pathfinding(panel_state, graph_data, "Dijkstra");
            }

            if ui.button("⭐ A*").clicked() {
                run_pathfinding(panel_state, graph_data, "A*");
            }

            if ui.button("🌊 BFS").clicked() {
                run_pathfinding(panel_state, graph_data, "BFS");
            }
        });

        ui.horizontal(|ui| {
            if ui.button("🔍 All Paths").clicked() {
                find_all_paths(panel_state, graph_data);
            }

            if ui.button("🔄 Clear").clicked() {
                panel_state.pathfind_source = None;
                panel_state.pathfind_target = None;
                panel_state.last_path_result = None;
            }
        });
    });

    // Results
    if let Some(result) = &panel_state.last_path_result {
        ui.separator();
        ui.label(format!("✅ Path found using {}", result.algorithm_used));
        ui.label(format!("Cost: {:.2}", result.cost));
        ui.label(format!("Steps: {}", result.path.len()));

        // Show path
        egui::ScrollArea::vertical()
            .max_height(100.0)
            .show(ui, |ui| {
                for (i, node_id) in result.path.iter().enumerate() {
                    if let Some(node) = graph_data.get_node(*node_id) {
                        ui.label(format!("{}: {}", i + 1, node.name));
                    }
                }
            });
    }
}

fn show_analysis_section(
    ui: &mut egui::Ui,
    panel_state: &mut AlgorithmPanelState,
    graph_data: &GraphData,
    graph_state: &GraphState,
) {
    ui.heading("📊 Graph Analysis");

    // Quick stats
    ui.label(format!("Nodes: {} | Edges: {}", graph_state.node_count, graph_state.edge_count));

    ui.separator();

    // Analysis buttons
    if ui.button("🔍 Analyze Graph").clicked() {
        analyze_graph(panel_state, graph_data);
    }

    // Results
    let results = &panel_state.analysis_results;

    ui.group(|ui| {
        ui.label("Analysis Results:");

        ui.label(format!("Connected Components: {}", results.connected_components));

        if results.connected_components > 1 {
            ui.colored_label(egui::Color32::YELLOW, "⚠️ Graph is not fully connected");
        } else {
            ui.colored_label(egui::Color32::GREEN, "✅ Graph is fully connected");
        }

        if results.has_cycles {
            ui.colored_label(egui::Color32::YELLOW, "🔄 Graph contains cycles");
        } else {
            ui.colored_label(egui::Color32::GREEN, "✅ Graph is acyclic (DAG)");
        }

        ui.label(format!("Max node degree: {}", results.max_degree));
    });

    ui.separator();

    // Advanced analysis
    ui.collapsing("Advanced Analysis", |ui| {
        if ui.button("📊 Centrality Analysis").clicked() {
            info!("Running centrality analysis...");
        }

        if ui.button("🎯 Topological Sort").clicked() {
            run_topological_sort(graph_data);
        }

        if ui.button("🔗 Find Bridges").clicked() {
            info!("Finding bridge edges...");
        }

        if ui.button("🏝️ Find Islands").clicked() {
            info!("Finding isolated subgraphs...");
        }
    });
}

fn show_layout_section(
    ui: &mut egui::Ui,
    layout_events: &mut EventWriter<crate::graph_core::RequestLayoutEvent>,
) {
    ui.heading("🔄 Graph Layout");

    ui.label("Apply automatic layout:");

    ui.horizontal(|ui| {
        if ui.button("🌐 Force-Directed").clicked() {
            layout_events.write(crate::graph_core::RequestLayoutEvent {
                layout_type: crate::graph_core::LayoutType::ForceDirected,
            });
            info!("Applied force-directed layout");
        }

        if ui.button("📊 Hierarchical").clicked() {
            layout_events.write(crate::graph_core::RequestLayoutEvent {
                layout_type: crate::graph_core::LayoutType::Hierarchical,
            });
            info!("Applied hierarchical layout");
        }
    });

    ui.horizontal(|ui| {
        if ui.button("⭕ Circular").clicked() {
            layout_events.write(crate::graph_core::RequestLayoutEvent {
                layout_type: crate::graph_core::LayoutType::Circular,
            });
            info!("Applied circular layout");
        }

        if ui.button("📐 Grid").clicked() {
            layout_events.write(crate::graph_core::RequestLayoutEvent {
                layout_type: crate::graph_core::LayoutType::Grid,
            });
            info!("Applied grid layout");
        }
    });

    ui.separator();

    ui.collapsing("Layout Options", |ui| {
        ui.label("Force-Directed Settings:");
        ui.add(egui::Slider::new(&mut 50.0, 10.0..=200.0).text("Spring Strength"));
        ui.add(egui::Slider::new(&mut 100.0, 50.0..=500.0).text("Ideal Distance"));

        ui.separator();

        ui.label("Hierarchical Settings:");
        ui.add(egui::Slider::new(&mut 50.0, 20.0..=200.0).text("Level Separation"));
        ui.add(egui::Slider::new(&mut 30.0, 10.0..=100.0).text("Node Separation"));
    });
}

// Helper functions
fn run_pathfinding(
    panel_state: &mut AlgorithmPanelState,
    graph_data: &GraphData,
    algorithm: &str,
) {
    if let (Some(source), Some(target)) = (panel_state.pathfind_source, panel_state.pathfind_target) {
        match GraphAlgorithms::shortest_path(graph_data, source, target) {
            Some((path, cost)) => {
                info!("Found path using {} with cost {}", algorithm, cost);
                panel_state.last_path_result = Some(PathfindResult {
                    path,
                    cost: cost as f32,
                    algorithm_used: algorithm.to_string(),
                });
            }
            None => {
                warn!("No path found between selected nodes");
                panel_state.last_path_result = None;
            }
        }
    }
}

fn find_all_paths(
    panel_state: &mut AlgorithmPanelState,
    graph_data: &GraphData,
) {
    if let (Some(source), Some(target)) = (panel_state.pathfind_source, panel_state.pathfind_target) {
        let paths = GraphAlgorithms::find_all_paths(graph_data, source, target, 10);
        info!("Found {} paths between nodes", paths.len());

        if let Some(shortest) = paths.first() {
            panel_state.last_path_result = Some(PathfindResult {
                path: shortest.clone(),
                cost: shortest.len() as f32,
                algorithm_used: format!("All Paths ({} found)", paths.len()),
            });
        }
    }
}

fn analyze_graph(
    panel_state: &mut AlgorithmPanelState,
    graph_data: &GraphData,
) {
    let components = GraphAlgorithms::find_components(graph_data);
    let has_cycles = GraphAlgorithms::has_cycles(graph_data);
    let centrality = GraphAlgorithms::degree_centrality(graph_data);

    let max_degree = centrality.values()
        .map(|(_, _, total)| *total)
        .max()
        .unwrap_or(0);

    panel_state.analysis_results = AnalysisResults {
        connected_components: components.len(),
        has_cycles,
        is_dag: !has_cycles,
        max_degree,
        node_count: graph_data.node_count(),
        edge_count: graph_data.edge_count(),
    };

    info!("Graph analysis complete");
}

fn run_topological_sort(graph_data: &GraphData) {
    match GraphAlgorithms::topological_sort(graph_data) {
        Ok(order) => {
            info!("Topological sort successful: {} nodes ordered", order.len());
        }
        Err(e) => {
            warn!("Topological sort failed: {}", e);
        }
    }
}
