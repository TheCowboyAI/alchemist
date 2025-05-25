use crate::camera::{GraphViewCamera, ViewMode};
use crate::graph_core::{CreateEdgeEvent, CreateNodeEvent, GraphState};
use crate::graph_patterns::GraphPattern;
use crate::json_loader::{FileOperationState, LoadJsonFileEvent, SaveJsonFileEvent};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

/// State for the control panel
#[derive(Resource, Default)]
pub struct ControlPanelState {
    pub visible: bool,
    pub width: f32,
    pub active_tab: ControlPanelTab,
}

#[derive(Default, PartialEq)]
pub enum ControlPanelTab {
    #[default]
    Graph,
    DDD,
    ECS,
    Algorithms,
}

impl ControlPanelState {
    pub fn new() -> Self {
        Self {
            visible: true,
            width: 300.0,
            active_tab: ControlPanelTab::Graph,
        }
    }
}

/// System for the left control panel
pub fn control_panel_system(
    mut contexts: EguiContexts,
    mut panel_state: ResMut<ControlPanelState>,
    graph_state: Res<GraphState>,
    camera_query: Query<&GraphViewCamera>,
    mut create_node_events: EventWriter<CreateNodeEvent>,
    mut create_edge_events: EventWriter<CreateEdgeEvent>,
    mut load_json_events: EventWriter<LoadJsonFileEvent>,
    mut save_json_events: EventWriter<SaveJsonFileEvent>,
    file_state: Res<FileOperationState>,
    node_query: Query<(Entity, &crate::graph_core::GraphNode)>,
) {
    // Only log when visibility actually changes
    if panel_state.is_changed() {
        debug!("Control panel visibility changed to: {}", panel_state.visible);
    }

    // Only update if panel is visible
    if !panel_state.visible {
        // Show a small floating button when panel is hidden
        egui::Window::new("Show Control Panel")
            .title_bar(false)
            .resizable(false)
            .default_pos([10.0, 50.0])
            .default_size([120.0, 30.0])
            .show(contexts.ctx_mut(), |ui| {
                if ui.button("📊 Show Controls (F1)").clicked() {
                    panel_state.visible = true;
                }
            });
        return;
    }

    // Only show panel when visible
    egui::SidePanel::left("control_panel")
        .default_width(panel_state.width)
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            // Header with close button
            ui.horizontal(|ui| {
                ui.heading("Alchemist Control Panel");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✖").clicked() {
                        panel_state.visible = false;
                    }
                });
            });

            ui.separator();

            // Tab selection
            ui.horizontal(|ui| {
                ui.selectable_value(&mut panel_state.active_tab, ControlPanelTab::Graph, "📊 Graph");
                ui.selectable_value(&mut panel_state.active_tab, ControlPanelTab::DDD, "🏗️ DDD");
                ui.selectable_value(&mut panel_state.active_tab, ControlPanelTab::ECS, "⚙️ ECS");
                ui.selectable_value(&mut panel_state.active_tab, ControlPanelTab::Algorithms, "🧮 Algorithms");
            });

            ui.separator();

            // Content based on active tab
            match panel_state.active_tab {
                ControlPanelTab::Graph => {
                    show_graph_tab(
                        ui,
                        &graph_state,
                        &camera_query,
                        &mut create_node_events,
                        &mut create_edge_events,
                        &mut load_json_events,
                        &mut save_json_events,
                        &file_state,
                        &node_query,
                    );
                }
                ControlPanelTab::DDD => {
                    show_ddd_tab(ui);
                }
                ControlPanelTab::ECS => {
                    show_ecs_tab(ui);
                }
                ControlPanelTab::Algorithms => {
                    show_algorithms_tab(ui, &graph_state);
                }
            }

            // Only update panel width if it actually changed significantly
            let new_width = ui.available_width();
            if (new_width - panel_state.width).abs() > 5.0 {
                panel_state.width = new_width;
            }
        });
}

/// Show the Graph tab content
fn show_graph_tab(
    ui: &mut egui::Ui,
    graph_state: &GraphState,
    camera_query: &Query<&GraphViewCamera>,
    create_node_events: &mut EventWriter<CreateNodeEvent>,
    create_edge_events: &mut EventWriter<CreateEdgeEvent>,
    load_json_events: &mut EventWriter<LoadJsonFileEvent>,
    save_json_events: &mut EventWriter<SaveJsonFileEvent>,
    file_state: &FileOperationState,
    node_query: &Query<(Entity, &crate::graph_core::GraphNode)>,
) {
    // View mode info
    if let Ok(camera) = camera_query.single() {
        ui.label("View Mode:");
        match camera.view_mode {
            ViewMode::ThreeD(_) => {
                ui.label("🎲 3D View");
                ui.label("Controls:");
                ui.label("• Right Mouse: Orbit");
                ui.label("• Middle Mouse: Pan (with Shift)");
                ui.label("• Scroll: Zoom");
                ui.label("• Tab/V: Switch to 2D");
            }
            ViewMode::TwoD(_) => {
                ui.label("📄 2D View");
                ui.label("Controls:");
                ui.label("• Middle Mouse: Pan");
                ui.label("• Scroll: Zoom");
                ui.label("• Tab/V: Switch to 3D");
            }
        }
    }

    ui.separator();

    // Graph stats
    ui.label(format!("Nodes: {}", graph_state.node_count));
    ui.label(format!("Edges: {}", graph_state.edge_count));

    ui.separator();

    // View type selection
    ui.heading("🎯 View Options");
    ui.horizontal(|ui| {
        if ui.button("📊 Graph View").clicked() {
            info!("Graph view selected");
        }
        if ui.button("🔄 Workflow View").clicked() {
            info!("Workflow view selected");
        }
        if ui.button("🎲 3D View").clicked() {
            info!("3D view selected");
        }
        if ui.button("📋 Events View").clicked() {
            info!("Events view selected");
        }
    });

    ui.separator();

    // Selected/Hovered Node Info
    if let Some(hovered_entity) = graph_state.hovered_entity {
        if let Ok((_, node)) = node_query.get(hovered_entity) {
            ui.heading("Hovered Node:");
            ui.label(format!("Name: {}", node.name));
            ui.label(format!("Type: {:?}", node.domain_type));

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
        }
    }

    // Graph Patterns section
    ui.heading("📐 Graph Patterns");

    ui.horizontal(|ui| {
        if ui.button("⭐ Star").clicked() {
            add_pattern_to_graph(
                GraphPattern::Star { points: 6 },
                create_node_events,
                create_edge_events,
                node_query,
            );
        }
        if ui.button("🌳 Tree").clicked() {
            add_pattern_to_graph(
                GraphPattern::Tree {
                    branch_factor: 3,
                    depth: 3,
                },
                create_node_events,
                create_edge_events,
                node_query,
            );
        }
    });

    ui.horizontal(|ui| {
        if ui.button("🔄 Cycle").clicked() {
            add_pattern_to_graph(
                GraphPattern::Cycle { nodes: 5 },
                create_node_events,
                create_edge_events,
                node_query,
            );
        }
        if ui.button("🔗 Complete").clicked() {
            add_pattern_to_graph(
                GraphPattern::Complete { nodes: 4 },
                create_node_events,
                create_edge_events,
                node_query,
            );
        }
    });

    ui.separator();

    // File operations
    ui.heading("📁 File Operations");

    ui.horizontal(|ui| {
        if ui.button("💾 Save Graph").clicked() {
            save_json_events.write(SaveJsonFileEvent {
                file_path: "saved_graph.json".to_string(),
            });
        }

        if ui.button("📂 Load Graph").clicked() {
            load_json_events.write(LoadJsonFileEvent {
                file_path: "saved_graph.json".to_string(),
            });
        }
    });

    if let Some(current_file) = &file_state.current_file_path {
        ui.label(format!(
            "Current: {}",
            current_file.split('/').last().unwrap_or("unknown")
        ));
    } else {
        ui.label("No file loaded");
    }
}

/// Show the DDD tab content
fn show_ddd_tab(ui: &mut egui::Ui) {
    ui.heading("🏗️ Domain-Driven Design");

    ui.label("DDD Concepts:");

    ui.collapsing("Bounded Contexts", |ui| {
        ui.label("• Core Domain");
        ui.label("• Supporting Subdomain");
        ui.label("• Generic Subdomain");

        if ui.button("+ Add Bounded Context").clicked() {
            info!("Add bounded context clicked");
        }
    });

    ui.collapsing("Aggregates", |ui| {
        ui.label("• User Aggregate");
        ui.label("• Order Aggregate");
        ui.label("• Product Aggregate");

        if ui.button("+ Add Aggregate").clicked() {
            info!("Add aggregate clicked");
        }
    });

    ui.collapsing("Entities & Value Objects", |ui| {
        ui.label("Entities:");
        ui.label("  • Customer");
        ui.label("  • Order");
        ui.label("  • Product");

        ui.separator();

        ui.label("Value Objects:");
        ui.label("  • Address");
        ui.label("  • Money");
        ui.label("  • Email");

        ui.horizontal(|ui| {
            if ui.button("+ Add Entity").clicked() {
                info!("Add entity clicked");
            }
            if ui.button("+ Add Value Object").clicked() {
                info!("Add value object clicked");
            }
        });
    });

    ui.separator();

    ui.heading("DDD Patterns");
    ui.horizontal(|ui| {
        if ui.button("🏛️ Repository").clicked() {
            info!("Repository pattern clicked");
        }
        if ui.button("🏭 Factory").clicked() {
            info!("Factory pattern clicked");
        }
        if ui.button("📋 Service").clicked() {
            info!("Service pattern clicked");
        }
    });
}

/// Show the ECS tab content
fn show_ecs_tab(ui: &mut egui::Ui) {
    ui.heading("⚙️ Entity Component System");

    ui.label("ECS Architecture:");

    ui.collapsing("Entities", |ui| {
        ui.label("• Player Entity");
        ui.label("• Enemy Entity");
        ui.label("• Projectile Entity");

        if ui.button("+ Create Entity").clicked() {
            info!("Create entity clicked");
        }
    });

    ui.collapsing("Components", |ui| {
        ui.label("• Transform Component");
        ui.label("• Velocity Component");
        ui.label("• Health Component");
        ui.label("• Render Component");

        if ui.button("+ Add Component").clicked() {
            info!("Add component clicked");
        }
    });

    ui.collapsing("Systems", |ui| {
        ui.label("• Movement System");
        ui.label("• Collision System");
        ui.label("• Render System");
        ui.label("• AI System");

        if ui.button("+ Add System").clicked() {
            info!("Add system clicked");
        }
    });

    ui.separator();

    ui.heading("ECS Operations");
    ui.horizontal(|ui| {
        if ui.button("🔄 Update Systems").clicked() {
            info!("Update systems clicked");
        }
        if ui.button("🔍 Query Entities").clicked() {
            info!("Query entities clicked");
        }
    });

    ui.separator();

    ui.label("System Performance:");
    ui.label("• Movement: 60 FPS");
    ui.label("• Collision: 60 FPS");
    ui.label("• Render: 60 FPS");
}

/// Show the Algorithms tab content
fn show_algorithms_tab(ui: &mut egui::Ui, graph_state: &GraphState) {
    ui.heading("🧮 Graph Algorithms");

    ui.label("Available Algorithms:");

    ui.collapsing("Pathfinding", |ui| {
        if ui.button("🎯 Dijkstra's Algorithm").clicked() {
            info!("Dijkstra's algorithm clicked");
        }
        if ui.button("⭐ A* Algorithm").clicked() {
            info!("A* algorithm clicked");
        }
        if ui.button("🌊 Breadth-First Search").clicked() {
            info!("BFS clicked");
        }
        if ui.button("🏔️ Depth-First Search").clicked() {
            info!("DFS clicked");
        }
    });

    ui.collapsing("Graph Analysis", |ui| {
        if ui.button("🔗 Connected Components").clicked() {
            info!("Connected components clicked");
        }
        if ui.button("🔄 Cycle Detection").clicked() {
            info!("Cycle detection clicked");
        }
        if ui.button("📊 Centrality Measures").clicked() {
            info!("Centrality measures clicked");
        }
        if ui.button("🎯 Topological Sort").clicked() {
            info!("Topological sort clicked");
        }
    });

    ui.collapsing("Network Flow", |ui| {
        if ui.button("💧 Max Flow").clicked() {
            info!("Max flow clicked");
        }
        if ui.button("💰 Min Cost Flow").clicked() {
            info!("Min cost flow clicked");
        }
    });

    ui.separator();

    ui.heading("Algorithm Results");
    ui.label(format!("Graph has {} nodes and {} edges", graph_state.node_count, graph_state.edge_count));

    if graph_state.node_count > 0 {
        ui.label("✅ Graph is ready for analysis");
    } else {
        ui.label("⚠️ Add nodes to run algorithms");
    }
}

/// Helper function to add patterns to the graph
fn add_pattern_to_graph(
    pattern: GraphPattern,
    create_node_events: &mut EventWriter<CreateNodeEvent>,
    _create_edge_events: &mut EventWriter<CreateEdgeEvent>,
    _existing_nodes: &Query<(Entity, &crate::graph_core::GraphNode)>,
) {
    use crate::graph_core::DomainNodeType;
    use bevy::math::Vec3;
    use uuid::Uuid;

    match pattern {
        GraphPattern::Star { points } => {
            // Create center node
            create_node_events.write(CreateNodeEvent {
                id: Uuid::new_v4(),
                name: "Star Center".to_string(),
                domain_type: DomainNodeType::Process,
                labels: vec!["center".to_string()],
                properties: std::collections::HashMap::new(),
                position: Vec3::ZERO,
                subgraph_id: None,
                color: None,
            });

            // Create star points
            for i in 0..points {
                let angle = (i as f32) * 2.0 * std::f32::consts::PI / (points as f32);
                let x = angle.cos() * 5.0;
                let z = angle.sin() * 5.0;

                create_node_events.write(CreateNodeEvent {
                    id: Uuid::new_v4(),
                    name: format!("Star Point {}", i + 1),
                    domain_type: DomainNodeType::Process,
                    labels: vec!["point".to_string()],
                    properties: std::collections::HashMap::new(),
                    position: Vec3::new(x, 0.0, z),
                    subgraph_id: None,
                    color: None,
                });
            }
        }
        GraphPattern::Tree { branch_factor, depth } => {
            // Create root node
            create_node_events.write(CreateNodeEvent {
                id: Uuid::new_v4(),
                name: "Tree Root".to_string(),
                domain_type: DomainNodeType::Process,
                labels: vec!["root".to_string()],
                properties: std::collections::HashMap::new(),
                position: Vec3::ZERO,
                subgraph_id: None,
                color: None,
            });

            // Create tree levels
            for level in 1..=depth {
                let nodes_at_level = branch_factor.pow(level as u32);
                for i in 0..nodes_at_level {
                    let x = (i as f32 - nodes_at_level as f32 / 2.0) * 3.0;
                    let z = level as f32 * 4.0;

                    create_node_events.write(CreateNodeEvent {
                        id: Uuid::new_v4(),
                        name: format!("Tree Node L{}N{}", level, i + 1),
                        domain_type: DomainNodeType::Process,
                        labels: vec!["branch".to_string()],
                        properties: std::collections::HashMap::new(),
                        position: Vec3::new(x, 0.0, z),
                        subgraph_id: None,
                        color: None,
                    });
                }
            }
        }
        GraphPattern::Cycle { nodes } => {
            for i in 0..nodes {
                let angle = (i as f32) * 2.0 * std::f32::consts::PI / (nodes as f32);
                let x = angle.cos() * 4.0;
                let z = angle.sin() * 4.0;

                create_node_events.write(CreateNodeEvent {
                    id: Uuid::new_v4(),
                    name: format!("Cycle Node {}", i + 1),
                    domain_type: DomainNodeType::Process,
                    labels: vec!["cycle".to_string()],
                    properties: std::collections::HashMap::new(),
                    position: Vec3::new(x, 0.0, z),
                    subgraph_id: None,
                    color: None,
                });
            }
        }
        GraphPattern::Complete { nodes } => {
            for i in 0..nodes {
                let angle = (i as f32) * 2.0 * std::f32::consts::PI / (nodes as f32);
                let x = angle.cos() * 3.0;
                let z = angle.sin() * 3.0;

                create_node_events.write(CreateNodeEvent {
                    id: Uuid::new_v4(),
                    name: format!("Complete Node {}", i + 1),
                    domain_type: DomainNodeType::Process,
                    labels: vec!["complete".to_string()],
                    properties: std::collections::HashMap::new(),
                    position: Vec3::new(x, 0.0, z),
                    subgraph_id: None,
                    color: None,
                });
            }
        }
        GraphPattern::DirectedAcyclicGraph { levels, nodes_per_level } => {
            for level in 0..levels {
                for node in 0..nodes_per_level {
                    let x = (node as f32 - nodes_per_level as f32 / 2.0) * 3.0;
                    let z = level as f32 * 4.0;

                    create_node_events.write(CreateNodeEvent {
                        id: Uuid::new_v4(),
                        name: format!("DAG L{}N{}", level, node + 1),
                        domain_type: DomainNodeType::Process,
                        labels: vec!["dag".to_string()],
                        properties: std::collections::HashMap::new(),
                        position: Vec3::new(x, 0.0, z),
                        subgraph_id: None,
                        color: None,
                    });
                }
            }
        }
        GraphPattern::Grid { width, height } => {
            for x in 0..width {
                for y in 0..height {
                    create_node_events.write(CreateNodeEvent {
                        id: Uuid::new_v4(),
                        name: format!("Grid ({}, {})", x, y),
                        domain_type: DomainNodeType::Process,
                        labels: vec!["grid".to_string()],
                        properties: std::collections::HashMap::new(),
                        position: Vec3::new(x as f32 * 3.0, 0.0, y as f32 * 3.0),
                        subgraph_id: None,
                        color: None,
                    });
                }
            }
        }
        GraphPattern::Bipartite { left_nodes, right_nodes, edge_density: _ } => {
            // Left side nodes
            for i in 0..left_nodes {
                create_node_events.write(CreateNodeEvent {
                    id: Uuid::new_v4(),
                    name: format!("Left {}", i + 1),
                    domain_type: DomainNodeType::Process,
                    labels: vec!["left".to_string()],
                    properties: std::collections::HashMap::new(),
                    position: Vec3::new(-3.0, 0.0, i as f32 * 2.0),
                    subgraph_id: None,
                    color: None,
                });
            }

            // Right side nodes
            for i in 0..right_nodes {
                create_node_events.write(CreateNodeEvent {
                    id: Uuid::new_v4(),
                    name: format!("Right {}", i + 1),
                    domain_type: DomainNodeType::Process,
                    labels: vec!["right".to_string()],
                    properties: std::collections::HashMap::new(),
                    position: Vec3::new(3.0, 0.0, i as f32 * 2.0),
                    subgraph_id: None,
                    color: None,
                });
            }
        }
        _ => {
            // Handle other patterns
            create_node_events.write(CreateNodeEvent {
                id: Uuid::new_v4(),
                name: "Pattern Node".to_string(),
                domain_type: DomainNodeType::Process,
                labels: vec!["pattern".to_string()],
                properties: std::collections::HashMap::new(),
                position: Vec3::ZERO,
                subgraph_id: None,
                color: None,
            });
        }
    }
}
