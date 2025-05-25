use super::components::GraphNode;
use super::graph_data::GraphData;
use super::{GraphAlgorithms, GraphInspectorState, GraphState};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts};
use uuid::Uuid;

/// UI state for the graph inspector
#[derive(Resource, Default)]
pub struct GraphInspectorState {
    /// Currently selected node
    pub selected_node: Option<Uuid>,
    /// Currently selected edge
    pub selected_edge: Option<Uuid>,
    /// Show graph statistics
    pub show_stats: bool,
    /// Show algorithm controls
    pub show_algorithms: bool,
    /// Path finding source
    pub pathfind_source: Option<Uuid>,
    /// Path finding target
    pub pathfind_target: Option<Uuid>,
    /// Search filter
    pub search_filter: String,
}

/// Main UI system for graph inspection
pub fn graph_inspector_ui(
    mut contexts: EguiContexts,
    mut inspector_state: ResMut<GraphInspectorState>,
    graph_data: Res<GraphData>,
    _node_query: Query<(&GraphNode, &Transform)>,
    mut layout_events: EventWriter<super::RequestLayoutEvent>,
) {
    let ctx = contexts.ctx_mut();

    // Left panel - Graph Inspector
    egui::SidePanel::left("graph_inspector")
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.heading("Graph Inspector");

            ui.separator();

            // Search box
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut inspector_state.search_filter);
            });

            ui.separator();

            // Node list
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.collapsing("Nodes", |ui| {
                    for (_node_idx, node_data) in graph_data.nodes() {
                        if !inspector_state.search_filter.is_empty()
                            && !node_data
                                .name
                                .to_lowercase()
                                .contains(&inspector_state.search_filter.to_lowercase())
                        {
                            continue;
                        }

                        let is_selected = inspector_state.selected_node == Some(node_data.id);
                        if ui.selectable_label(is_selected, &node_data.name).clicked() {
                            inspector_state.selected_node = Some(node_data.id);
                            inspector_state.selected_edge = None;
                        }
                    }
                });

                ui.separator();

                // Edge list
                ui.collapsing("Edges", |ui| {
                    for (_edge_idx, edge_data, source_idx, target_idx) in graph_data.edges() {
                        let source_name = graph_data
                            .graph
                            .node_weight(source_idx)
                            .map(|n| n.name.as_str())
                            .unwrap_or("Unknown");
                        let target_name = graph_data
                            .graph
                            .node_weight(target_idx)
                            .map(|n| n.name.as_str())
                            .unwrap_or("Unknown");

                        let edge_label = format!("{source_name} → {target_name}");

                        let is_selected = inspector_state.selected_edge == Some(edge_data.id);
                        if ui.selectable_label(is_selected, edge_label).clicked() {
                            inspector_state.selected_edge = Some(edge_data.id);
                            inspector_state.selected_node = None;
                        }
                    }
                });
            });
        });

    // Right panel - Properties
    egui::SidePanel::right("properties")
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.heading("Properties");

            ui.separator();

            // Node properties
            if let Some(node_id) = inspector_state.selected_node {
                if let Some(node_data) = graph_data.get_node(node_id) {
                    ui.label("Node Properties");
                    ui.separator();

                    ui.label(format!("Name: {}", node_data.name));
                    ui.label(format!("Type: {:?}", node_data.domain_type));
                    ui.label(format!("ID: {}", node_data.id));

                    ui.separator();
                    ui.label("Position:");
                    ui.label(format!("  X: {:.2}", node_data.position.x));
                    ui.label(format!("  Y: {:.2}", node_data.position.y));
                    ui.label(format!("  Z: {:.2}", node_data.position.z));

                    if !node_data.labels.is_empty() {
                        ui.separator();
                        ui.label("Labels:");
                        for label in &node_data.labels {
                            ui.label(format!("  • {label}"));
                        }
                    }

                    if !node_data.properties.is_empty() {
                        ui.separator();
                        ui.label("Properties:");
                        for (key, value) in &node_data.properties {
                            ui.label(format!("  {key}: {value}"));
                        }
                    }

                    ui.separator();

                    // Actions
                    if ui.button("Set as Pathfind Source").clicked() {
                        inspector_state.pathfind_source = Some(node_id);
                    }

                    if ui.button("Set as Pathfind Target").clicked() {
                        inspector_state.pathfind_target = Some(node_id);
                    }
                }
            }

            // Edge properties
            if let Some(edge_id) = inspector_state.selected_edge {
                // Find edge in GraphData
                for (_edge_idx, edge_data, _source_idx, _target_idx) in graph_data.edges() {
                    if edge_data.id == edge_id {
                        ui.label("Edge Properties");
                        ui.separator();

                        ui.label(format!("Type: {:?}", edge_data.edge_type));
                        ui.label(format!("ID: {}", edge_data.id));

                        if !edge_data.labels.is_empty() {
                            ui.separator();
                            ui.label("Labels:");
                            for label in &edge_data.labels {
                                ui.label(format!("  • {label}"));
                            }
                        }

                        if !edge_data.properties.is_empty() {
                            ui.separator();
                            ui.label("Properties:");
                            for (key, value) in &edge_data.properties {
                                ui.label(format!("  {key}: {value}"));
                            }
                        }

                        break;
                    }
                }
            }
        });

    // Top panel - Statistics
    egui::TopBottomPanel::top("statistics").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(format!("Nodes: {}", graph_data.node_count()));
            ui.separator();
            ui.label(format!("Edges: {}", graph_data.edge_count()));
            ui.separator();

            ui.checkbox(&mut inspector_state.show_stats, "Statistics");
            ui.checkbox(&mut inspector_state.show_algorithms, "Algorithms");
        });
    });

    // Statistics window
    if inspector_state.show_stats {
        egui::Window::new("Graph Statistics")
            .default_pos([400.0, 100.0])
            .show(ctx, |ui| {
                show_graph_statistics(ui, &graph_data);
            });
    }

    // Algorithms window
    if inspector_state.show_algorithms {
        egui::Window::new("Graph Algorithms")
            .default_pos([400.0, 300.0])
            .show(ctx, |ui| {
                show_algorithm_controls(ui, &mut inspector_state, &graph_data, &mut layout_events);
            });
    }
}

/// Show graph statistics
fn show_graph_statistics(ui: &mut egui::Ui, graph_data: &GraphData) {
    ui.label(format!("Total Nodes: {}", graph_data.node_count()));
    ui.label(format!("Total Edges: {}", graph_data.edge_count()));

    ui.separator();

    // Check connectivity
    let components = GraphAlgorithms::find_components(graph_data);
    ui.label(format!("Connected Components: {}", components.len()));

    if components.len() > 1 {
        ui.label("⚠️ Graph is not fully connected");
    } else {
        ui.label("✓ Graph is fully connected");
    }

    // Check for cycles
    let has_cycles = GraphAlgorithms::has_cycles(graph_data);
    if has_cycles {
        ui.label("⚠️ Graph contains cycles");
    } else {
        ui.label("✓ Graph is acyclic (DAG)");
    }

    ui.separator();

    // Degree statistics
    let centrality = GraphAlgorithms::degree_centrality(graph_data);
    let mut max_in = 0;
    let mut max_out = 0;
    let mut max_total = 0;

    for (in_deg, out_deg, total) in centrality.values() {
        max_in = max_in.max(*in_deg);
        max_out = max_out.max(*out_deg);
        max_total = max_total.max(*total);
    }

    ui.label(format!("Max In-Degree: {max_in}"));
    ui.label(format!("Max Out-Degree: {max_out}"));
    ui.label(format!("Max Total Degree: {max_total}"));
}

/// Show algorithm controls
fn show_algorithm_controls(
    ui: &mut egui::Ui,
    inspector_state: &mut GraphInspectorState,
    graph_data: &GraphData,
    layout_events: &mut EventWriter<super::RequestLayoutEvent>,
) {
    ui.heading("Pathfinding");

    // Display selected nodes
    if let Some(source_id) = inspector_state.pathfind_source {
        if let Some(node) = graph_data.get_node(source_id) {
            ui.label(format!("Source: {}", node.name));
        }
    } else {
        ui.label("Source: None selected");
    }

    if let Some(target_id) = inspector_state.pathfind_target {
        if let Some(node) = graph_data.get_node(target_id) {
            ui.label(format!("Target: {}", node.name));
        }
    } else {
        ui.label("Target: None selected");
    }

    ui.separator();

    // Find path button
    if ui.button("Find Shortest Path").clicked() {
        if let (Some(source), Some(target)) = (
            inspector_state.pathfind_source,
            inspector_state.pathfind_target,
        ) {
            match GraphAlgorithms::shortest_path(graph_data, source, target) {
                Some((path, cost)) => {
                    info!("Found path with cost {}: {:?}", cost, path);
                    // TODO: Highlight path in visualization
                }
                None => {
                    warn!("No path found between selected nodes");
                }
            }
        }
    }

    if ui.button("Clear Selection").clicked() {
        inspector_state.pathfind_source = None;
        inspector_state.pathfind_target = None;
    }

    ui.separator();
    ui.heading("Analysis");

    if ui.button("Find All Paths").clicked() {
        if let (Some(source), Some(target)) = (
            inspector_state.pathfind_source,
            inspector_state.pathfind_target,
        ) {
            let paths = GraphAlgorithms::find_all_paths(graph_data, source, target, 10);
            info!("Found {} paths between nodes", paths.len());
        }
    }

    if ui.button("Topological Sort").clicked() {
        match GraphAlgorithms::topological_sort(graph_data) {
            Ok(order) => {
                info!("Topological order found with {} nodes", order.len());
                // TODO: Visualize the ordering
            }
            Err(e) => {
                warn!("Cannot perform topological sort: {}", e);
            }
        }
    }

    ui.separator();
    ui.heading("Layout");

    if ui.button("Force-Directed Layout").clicked() {
        layout_events.write(super::RequestLayoutEvent {
            layout_type: super::LayoutType::ForceDirected,
        });
        info!("Force-directed layout requested");
    }

    if ui.button("Hierarchical Layout").clicked() {
        layout_events.write(super::RequestLayoutEvent {
            layout_type: super::LayoutType::Hierarchical,
        });
        info!("Hierarchical layout requested");
    }
}

/// Handle mouse clicks for node selection
pub fn handle_node_selection(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    node_query: Query<(Entity, &GraphNode, &Transform)>,
    mut inspector_state: ResMut<GraphInspectorState>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Convert screen coordinates to world ray
    let ray = camera.viewport_to_world(camera_transform, cursor_position);

    if let Ok(ray) = ray {
        // Simple distance-based picking for now
        let mut closest_node = None;
        let mut closest_distance = f32::MAX;

        for (_entity, node, transform) in &node_query {
            let node_pos = transform.translation;
            let distance = ray.origin.distance(node_pos);

            if distance < closest_distance && distance < 2.0 {
                // Within 2 units
                closest_distance = distance;
                closest_node = Some(node.id);
            }
        }

        if let Some(node_id) = closest_node {
            inspector_state.selected_node = Some(node_id);
            inspector_state.selected_edge = None;
            // info!("Selected node: {:?}", node_id);
        }
    }
}

/// Component to highlight selected entities
#[derive(Component)]
pub struct SelectionHighlight;

/// System to visually highlight selected nodes/edges
pub fn update_selection_highlights(
    mut commands: Commands,
    inspector_state: Res<GraphInspectorState>,
    node_query: Query<(Entity, &GraphNode), Without<SelectionHighlight>>,
    highlighted_query: Query<Entity, With<SelectionHighlight>>,
    _materials: ResMut<Assets<StandardMaterial>>,
) {
    // Remove old highlights
    for entity in &highlighted_query {
        commands.entity(entity).remove::<SelectionHighlight>();
    }

    // Add new highlight
    if let Some(selected_id) = inspector_state.selected_node {
        for (entity, node) in &node_query {
            if node.id == selected_id {
                commands.entity(entity).insert(SelectionHighlight);
                // Could also modify material here for visual feedback
            }
        }
    }
}
