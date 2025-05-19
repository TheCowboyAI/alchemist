use eframe::egui;
use rand::Rng;
use uuid::Uuid;

mod graph;
mod workflow_editor;
mod ecs;
mod events;
mod graph_patterns;

use workflow_editor::WorkflowEditor;
use ecs::GraphSystem;
use events::{EventStream, GraphEvent, Command, CreateNodeCommand, CreateEdgeCommand, Model};
use graph::AlchemistGraph;
use graph_patterns::{GraphPattern, generate_pattern};

// Graph settings structure to replace unsafe static variables
#[derive(Clone, Debug)]
struct GraphSettings {
    node_size: f32,
    node_color: egui::Color32,
    edge_thickness: f32,
    edge_color: egui::Color32,
    show_arrows: bool,
    show_node_labels: bool,
    show_node_properties: bool,
}

impl Default for GraphSettings {
    fn default() -> Self {
        Self {
            node_size: 20.0,
            node_color: egui::Color32::from_rgb(50, 150, 220),
            edge_thickness: 2.0,
            edge_color: egui::Color32::GRAY,
            show_arrows: true,
            show_node_labels: true,
            show_node_properties: true,
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("Alchemist - Information Graph Workflows"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Alchemist",
        options,
        Box::new(|cc| {
            // Set larger text styles
            let mut style = (*cc.egui_ctx.style()).clone();
            style.text_styles = [
                (egui::TextStyle::Heading, egui::FontId::proportional(32.0)),
                (egui::TextStyle::Body, egui::FontId::proportional(22.0)),
                (egui::TextStyle::Monospace, egui::FontId::monospace(20.0)),
                (egui::TextStyle::Button, egui::FontId::proportional(22.0)),
                (egui::TextStyle::Small, egui::FontId::proportional(18.0)),
            ].into();
            cc.egui_ctx.set_style(style);
            
            Ok(Box::new(AlchemistApp::new(cc)))
        }),
    )
}

struct AlchemistApp {
    name: String,
    age: u32,
    color: egui::Color32,
    show_extra_panel: bool,
    random_value: f64,
    current_view: ViewType,
    workflow_editor: WorkflowEditor,
    graph_system: GraphSystem,
    event_stream: EventStream,
    information_graph: AlchemistGraph,
    layout_type: LayoutType,
    label_filter: String,
    show_all_nodes: bool,
    show_all_edges: bool,
    applying_force_layout: bool,
    layout_iterations: usize,
    node_forces: std::collections::HashMap<Uuid, egui::Vec2>,
    graph_settings: GraphSettings,
    selected_pattern: PatternType,
    tree_branch_factor: usize,
    tree_depth: usize,
    star_points: usize,
    cycle_nodes: usize,
    complete_nodes: usize,
    grid_width: usize,
    grid_height: usize,
    random_nodes: usize,
    random_edge_probability: f32,
}

#[derive(PartialEq, Clone, Copy)]
enum ViewType {
    Main,
    Workflow,
    Settings,
    Events,
}

#[derive(PartialEq, Clone, Copy)]
enum LayoutType {
    Circular,
    ForceDirected,
    Hierarchical,
}

#[derive(PartialEq, Clone, Copy)]
enum PatternType {
    Tree,
    Star,
    Cycle,
    Complete,
    Grid,
    Random,
}

impl AlchemistApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize the graph system
        let mut graph_system = GraphSystem::new();
        let event_stream = EventStream::new();
        let information_graph = AlchemistGraph::new();
        
        // Create a workflow editor
        let workflow_editor = WorkflowEditor::new();
        
        // Import the workflow graph into the ECS system
        graph_system.import_graph(&workflow_editor.workflow.graph);
        
        Self {
            name: "World".to_owned(),
            age: 25,
            color: egui::Color32::from_rgb(100, 150, 250),
            show_extra_panel: false,
            random_value: 0.5,
            current_view: ViewType::Main,
            workflow_editor,
            graph_system,
            event_stream,
            information_graph,
            layout_type: LayoutType::Circular,
            label_filter: String::new(),
            show_all_nodes: true,
            show_all_edges: true,
            applying_force_layout: false,
            layout_iterations: 0,
            node_forces: std::collections::HashMap::new(),
            graph_settings: GraphSettings::default(),
            selected_pattern: PatternType::Tree,
            tree_branch_factor: 2,
            tree_depth: 3,
            star_points: 8,
            cycle_nodes: 6,
            complete_nodes: 5,
            grid_width: 4,
            grid_height: 4,
            random_nodes: 10,
            random_edge_probability: 0.3,
        }
    }
    
    // Helper method to execute a command and update the information graph from events
    fn execute_command(&mut self, command: &dyn Command) {
        // Execute the command to get events
        let events = command.execute();
        
        // Add all events to the event stream and apply to our models
        for event in events {
            if let Some(graph_event) = event.as_any().downcast_ref::<GraphEvent>() {
                // Apply the event to our graph model
                self.information_graph.apply_event(graph_event);
                
                // Apply the event to our ECS system
                self.graph_system.apply_event(graph_event);
                
                // Add to event stream
                self.event_stream.append(graph_event.clone());
            }
        }
    }
    
    // Add a method to apply force-directed layout
    fn apply_force_directed_layout(&mut self, node_positions: &mut std::collections::HashMap<Uuid, egui::Pos2>) {
        // Reset forces
        self.node_forces.clear();
        
        // Initialize forces for all nodes
        for id in self.information_graph.nodes.keys() {
            self.node_forces.insert(*id, egui::Vec2::ZERO);
        }
        
        // Calculate repulsive forces (nodes repel each other)
        let repulsion_strength = 5000.0;
        let attraction_strength = 0.05;
        let min_distance = 20.0;
        
        // Apply repulsive forces between all nodes
        let node_ids: Vec<Uuid> = self.information_graph.nodes.keys().cloned().collect();
        for i in 0..node_ids.len() {
            for j in (i+1)..node_ids.len() {
                let id1 = node_ids[i];
                let id2 = node_ids[j];
                
                if let (Some(pos1), Some(pos2)) = (node_positions.get(&id1), node_positions.get(&id2)) {
                    let delta = *pos2 - *pos1;
                    let distance = delta.length().max(min_distance);
                    let force = delta.normalized() * repulsion_strength / (distance * distance);
                    
                    // Apply repulsive force
                    if let Some(force1) = self.node_forces.get_mut(&id1) {
                        *force1 -= force;
                    }
                    if let Some(force2) = self.node_forces.get_mut(&id2) {
                        *force2 += force;
                    }
                }
            }
        }
        
        // Apply attractive forces for edges (connected nodes attract)
        for (_, edge) in &self.information_graph.edges {
            if let (Some(pos1), Some(pos2)) = (node_positions.get(&edge.source), node_positions.get(&edge.target)) {
                let delta = *pos2 - *pos1;
                let distance = delta.length().max(min_distance);
                let force = delta.normalized() * distance * attraction_strength;
                
                // Apply attractive force
                if let Some(force1) = self.node_forces.get_mut(&edge.source) {
                    *force1 += force;
                }
                if let Some(force2) = self.node_forces.get_mut(&edge.target) {
                    *force2 -= force;
                }
            }
        }
        
        // Apply forces to update positions
        for (id, force) in &self.node_forces {
            if let Some(pos) = node_positions.get_mut(id) {
                // Apply force with damping
                let damping = 0.9;
                let max_movement = 5.0; // Limit maximum movement per iteration
                let movement = force.normalized() * force.length().min(max_movement) * damping;
                pos.x += movement.x;
                pos.y += movement.y;
            }
        }
        
        // Keep track of iterations
        self.layout_iterations += 1;
        if self.layout_iterations >= 100 {
            self.applying_force_layout = false;
            self.layout_iterations = 0;
        }
    }
    
    fn apply_hierarchical_layout(&mut self, node_positions: &mut std::collections::HashMap<Uuid, egui::Pos2>, rect: egui::Rect) {
        // Maps to track node levels and positions within levels
        let mut node_levels = std::collections::HashMap::new();
        let mut nodes_per_level = std::collections::HashMap::new();
        
        // First, assign levels to nodes (0 for nodes with no incoming edges, etc.)
        let mut nodes_with_no_incoming = Vec::new();
        let mut incoming_edges_count = std::collections::HashMap::new();
        
        // Initialize incoming edges count
        for id in self.information_graph.nodes.keys() {
            incoming_edges_count.insert(*id, 0);
        }
        
        // Count incoming edges
        for (_, edge) in &self.information_graph.edges {
            if let Some(count) = incoming_edges_count.get_mut(&edge.target) {
                *count += 1;
            }
        }
        
        // Find nodes with no incoming edges (level 0)
        for (id, count) in &incoming_edges_count {
            if *count == 0 {
                nodes_with_no_incoming.push(*id);
                node_levels.insert(*id, 0);
                
                // Initialize level 0 counter
                nodes_per_level.entry(0).or_insert(0);
                *nodes_per_level.get_mut(&0).unwrap() += 1;
            }
        }
        
        // Breadth-first traversal to assign levels
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        
        // Start with nodes having no incoming edges
        for id in nodes_with_no_incoming {
            queue.push_back(id);
            visited.insert(id);
        }
        
        // If no entry points found, start with any node
        if queue.is_empty() && !self.information_graph.nodes.is_empty() {
            let first_node = *self.information_graph.nodes.keys().next().unwrap();
            queue.push_back(first_node);
            visited.insert(first_node);
            node_levels.insert(first_node, 0);
            nodes_per_level.entry(0).or_insert(0);
            *nodes_per_level.get_mut(&0).unwrap() += 1;
        }
        
        // Process queue
        while let Some(node_id) = queue.pop_front() {
            let current_level = *node_levels.get(&node_id).unwrap();
            
            // Find all outgoing edges
            for (_, edge) in &self.information_graph.edges {
                if edge.source == node_id {
                    let target_id = edge.target;
                    
                    // If not visited yet, assign level and add to queue
                    if !visited.contains(&target_id) {
                        let next_level = current_level + 1;
                        node_levels.insert(target_id, next_level);
                        
                        // Update counter for this level
                        nodes_per_level.entry(next_level).or_insert(0);
                        *nodes_per_level.get_mut(&next_level).unwrap() += 1;
                        
                        queue.push_back(target_id);
                        visited.insert(target_id);
                    }
                }
            }
        }
        
        // Assign positions based on levels
        let mut positions_in_level = std::collections::HashMap::new();
        let level_height = rect.height() / (nodes_per_level.len().max(1) as f32 + 1.0);
        
        for (id, level) in &node_levels {
            // Initialize position counter for this level if not exists
            positions_in_level.entry(*level).or_insert(0);
            let position = *positions_in_level.get(level).unwrap();
            
            // Calculate horizontal position
            let level_width = rect.width() / (nodes_per_level.get(level).unwrap_or(&1) + 1) as f32;
            let x = rect.left() + level_width * (position + 1) as f32;
            
            // Calculate vertical position
            let y = rect.top() + level_height * (*level + 1) as f32;
            
            // Update position
            node_positions.insert(*id, egui::Pos2::new(x, y));
            
            // Increment position counter for this level
            *positions_in_level.get_mut(level).unwrap() += 1;
        }
        
        // Assign positions for any nodes not visited (isolated)
        let default_level = nodes_per_level.len();
        for id in self.information_graph.nodes.keys() {
            if !node_levels.contains_key(id) {
                // Place at bottom level
                let x = rect.left() + rect.width() / 2.0;
                let y = rect.top() + level_height * (default_level + 1) as f32;
                node_positions.insert(*id, egui::Pos2::new(x, y));
            }
        }
    }
    
    fn apply_circular_layout(&mut self, node_positions: &mut std::collections::HashMap<Uuid, egui::Pos2>, rect: egui::Rect) {
        let count = self.information_graph.nodes.len();
        if count == 0 {
            return;
        }
        
        let center_x = rect.center().x;
        let center_y = rect.center().y;
        let radius = (rect.width().min(rect.height()) * 0.4).max(100.0);
        
        let angle_step = 2.0 * std::f32::consts::PI / count as f32;
        
        for (i, (id, _)) in self.information_graph.nodes.iter().enumerate() {
            let angle = i as f32 * angle_step;
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            node_positions.insert(*id, egui::Pos2::new(x, y));
        }
    }
}

impl eframe::App for AlchemistApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Alchemist - Information Graph Workflows");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Generate Random").clicked() {
                        self.random_value = rand::rng().random_range(0.0..1.0);
                    }
                    
                    if ui.button("Toggle Extra Panel").clicked() {
                        self.show_extra_panel = !self.show_extra_panel;
                    }
                });
            });
            
            ui.horizontal(|ui| {
                if ui.selectable_label(self.current_view == ViewType::Main, "Main View").clicked() {
                    self.current_view = ViewType::Main;
                }
                if ui.selectable_label(self.current_view == ViewType::Workflow, "Workflow Editor").clicked() {
                    self.current_view = ViewType::Workflow;
                }
                if ui.selectable_label(self.current_view == ViewType::Settings, "Settings").clicked() {
                    self.current_view = ViewType::Settings;
                }
                if ui.selectable_label(self.current_view == ViewType::Events, "Events").clicked() {
                    self.current_view = ViewType::Events;
                }
            });
        });
        
        match self.current_view {
            ViewType::Main => self.show_main_view(ctx),
            ViewType::Workflow => self.show_workflow_view(ctx),
            ViewType::Settings => self.show_settings_view(ctx),
            ViewType::Events => self.show_events_view(ctx),
        }
        
        if self.show_extra_panel {
            egui::Window::new("Extra Information")
                .default_pos(egui::pos2(300.0, 300.0))
                .show(ctx, |ui| {
                    ui.label("This is an additional panel that can be toggled.");
                    ui.label("It demonstrates the window functionality in egui.");
                    ui.add_space(10.0);
                    
                    if ui.button("Close").clicked() {
                        self.show_extra_panel = false;
                    }
                });
        }
    }
}

impl AlchemistApp {
    fn show_main_view(&mut self, ctx: &egui::Context) {
        // Define IDs for persistent storage
        let node_positions_id = egui::Id::new("node_positions");
        let selected_node_id = egui::Id::new("selected_node");
        
        // Get persistent node positions and selection with proper API
        let mut node_positions: std::collections::HashMap<Uuid, egui::Pos2> = 
            ctx.memory_mut(|mem| mem.data.get_persisted(node_positions_id))
            .unwrap_or_default();
            
        let mut selected_node: Option<Uuid> = 
            ctx.memory_mut(|mem| mem.data.get_persisted(selected_node_id))
            .unwrap_or_default();
        
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Graph Settings");
            ui.add(egui::Separator::default());
            ui.add_space(10.0);
            
            // Node Appearance Settings
            ui.collapsing("Node Appearance", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Node Size:");
                    let mut size = self.graph_settings.node_size;
                    if ui.add(egui::Slider::new(&mut size, 10.0..=50.0)).changed() {
                        self.graph_settings.node_size = size;
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Node Color:");
                    let mut color = self.graph_settings.node_color;
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        self.graph_settings.node_color = color;
                    }
                });
                
                ui.checkbox(&mut self.graph_settings.show_node_labels, "Show Node Labels");
                ui.checkbox(&mut self.graph_settings.show_node_properties, "Show Node Properties");
            });
            
            // Edge Appearance Settings
            ui.collapsing("Edge Appearance", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Edge Thickness:");
                    let mut thickness = self.graph_settings.edge_thickness;
                    if ui.add(egui::Slider::new(&mut thickness, 1.0..=10.0)).changed() {
                        self.graph_settings.edge_thickness = thickness;
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Edge Color:");
                    let mut color = self.graph_settings.edge_color;
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        self.graph_settings.edge_color = color;
                    }
                });
                
                let mut show_arrows = self.graph_settings.show_arrows;
                if ui.checkbox(&mut show_arrows, "Show Arrows").changed() {
                    self.graph_settings.show_arrows = show_arrows;
                }
            });
            
            // Layout Settings
            ui.collapsing("Layout", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Layout Type:");
                    ui.selectable_value(&mut self.layout_type, LayoutType::Circular, "Circular");
                    ui.selectable_value(&mut self.layout_type, LayoutType::ForceDirected, "Force Directed");
                    ui.selectable_value(&mut self.layout_type, LayoutType::Hierarchical, "Hierarchical");
                });
                
                if ui.button("Apply Layout").clicked() {
                    match self.layout_type {
                        LayoutType::Circular => {
                            // Clear positions to trigger recalculation with circular layout
                            ctx.memory_mut(|mem| mem.data.remove::<std::collections::HashMap<Uuid, egui::Pos2>>(node_positions_id));
                        },
                        LayoutType::ForceDirected => {
                            // Start force-directed layout
                            self.applying_force_layout = true;
                            self.layout_iterations = 0;
                        },
                        LayoutType::Hierarchical => {
                            // Clear positions to trigger recalculation with hierarchical layout
                            ctx.memory_mut(|mem| mem.data.remove::<std::collections::HashMap<Uuid, egui::Pos2>>(node_positions_id));
                            // The hierarchical layout will be applied in the drawing code
                            self.layout_type = LayoutType::Hierarchical;
                        }
                    }
                }
                
                if ui.button("Reset Layout").clicked() {
                    // Clear node positions to trigger recalculation
                    ctx.memory_mut(|mem| mem.data.remove::<std::collections::HashMap<Uuid, egui::Pos2>>(node_positions_id));
                }
            });
            
            // Filter Settings
            ui.collapsing("Filters", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Filter by Label:");
                    ui.text_edit_singleline(&mut self.label_filter);
                });
                
                ui.checkbox(&mut self.show_all_nodes, "Show All Nodes");
                ui.checkbox(&mut self.show_all_edges, "Show All Edges");
                
                if ui.button("Apply Filters").clicked() {
                    // Logic to apply filters
                }
            });
            
            // Show entity counts from the ECS system
            ui.add_space(10.0);
            ui.separator();
            ui.heading("System Status");
            ui.label(format!("ECS Entities: {}", self.graph_system.registry.entity_count()));
            ui.label(format!("Event Count: {}", self.event_stream.len()));
            
            // Add a new section for graph patterns
            ui.collapsing("Graph Patterns", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Pattern Type:");
                    ui.selectable_value(&mut self.selected_pattern, PatternType::Tree, "Tree");
                    ui.selectable_value(&mut self.selected_pattern, PatternType::Star, "Star");
                    ui.selectable_value(&mut self.selected_pattern, PatternType::Cycle, "Cycle");
                });
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.selected_pattern, PatternType::Complete, "Complete");
                    ui.selectable_value(&mut self.selected_pattern, PatternType::Grid, "Grid");
                    ui.selectable_value(&mut self.selected_pattern, PatternType::Random, "Random");
                });
                
                // Show pattern-specific parameters
                match self.selected_pattern {
                    PatternType::Tree => {
                        ui.horizontal(|ui| {
                            ui.label("Branch Factor:");
                            ui.add(egui::Slider::new(&mut self.tree_branch_factor, 1..=5));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Depth:");
                            ui.add(egui::Slider::new(&mut self.tree_depth, 1..=5));
                        });
                    },
                    PatternType::Star => {
                        ui.horizontal(|ui| {
                            ui.label("Points:");
                            ui.add(egui::Slider::new(&mut self.star_points, 3..=20));
                        });
                    },
                    PatternType::Cycle => {
                        ui.horizontal(|ui| {
                            ui.label("Nodes:");
                            ui.add(egui::Slider::new(&mut self.cycle_nodes, 3..=20));
                        });
                    },
                    PatternType::Complete => {
                        ui.horizontal(|ui| {
                            ui.label("Nodes:");
                            ui.add(egui::Slider::new(&mut self.complete_nodes, 3..=10));
                            ui.label("(caution: generates many edges)");
                        });
                    },
                    PatternType::Grid => {
                        ui.horizontal(|ui| {
                            ui.label("Width:");
                            ui.add(egui::Slider::new(&mut self.grid_width, 2..=10));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Height:");
                            ui.add(egui::Slider::new(&mut self.grid_height, 2..=10));
                        });
                    },
                    PatternType::Random => {
                        ui.horizontal(|ui| {
                            ui.label("Nodes:");
                            ui.add(egui::Slider::new(&mut self.random_nodes, 3..=20));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Edge Probability:");
                            ui.add(egui::Slider::new(&mut self.random_edge_probability, 0.1..=0.9));
                        });
                    },
                }
                
                if ui.button("Generate Pattern").clicked() {
                    // Create the selected pattern
                    let pattern = match self.selected_pattern {
                        PatternType::Tree => GraphPattern::Tree { 
                            branch_factor: self.tree_branch_factor, 
                            depth: self.tree_depth 
                        },
                        PatternType::Star => GraphPattern::Star { 
                            points: self.star_points 
                        },
                        PatternType::Cycle => GraphPattern::Cycle { 
                            nodes: self.cycle_nodes 
                        },
                        PatternType::Complete => GraphPattern::Complete { 
                            nodes: self.complete_nodes 
                        },
                        PatternType::Grid => GraphPattern::Grid { 
                            width: self.grid_width, 
                            height: self.grid_height 
                        },
                        PatternType::Random => GraphPattern::Random { 
                            nodes: self.random_nodes, 
                            edge_probability: self.random_edge_probability 
                        },
                    };
                    
                    // Generate the pattern and replace the current graph
                    self.information_graph = generate_pattern(pattern);
                    
                    // Clear the node positions to trigger layout recalculation
                    ctx.memory_mut(|mem| mem.data.remove::<std::collections::HashMap<Uuid, egui::Pos2>>(egui::Id::new("node_positions")));
                    
                    // Apply the selected layout
                    match self.layout_type {
                        LayoutType::ForceDirected => {
                            // Start force-directed layout
                            self.applying_force_layout = true;
                            self.layout_iterations = 0;
                        },
                        _ => {} // Other layouts will be applied automatically
                    }
                }
            });
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Information Graph Visualization");
            
            // Graph control toolbar
            ui.horizontal(|ui| {
                if ui.button("Generate Test Nodes").clicked() {
                    // Example of using the Command pattern to create nodes
                    let cmd = CreateNodeCommand {
                        name: format!("Test Node {}", rand::rng().random_range(1..100)),
                        labels: vec!["test".to_string(), "node".to_string()],
                    };
                    
                    // Execute the command using our helper method
                    self.execute_command(&cmd);
                    
                    // Create a random connection between nodes if we have at least 2 nodes
                    let nodes: Vec<_> = self.information_graph.nodes.keys().cloned().collect();
                    if nodes.len() >= 2 {
                        let mut rng = rand::rng();
                        let source_idx = rng.random_range(0..nodes.len());
                        let mut target_idx = rng.random_range(0..nodes.len());
                        
                        // Make sure source and target are different
                        while target_idx == source_idx {
                            target_idx = rng.random_range(0..nodes.len());
                        }
                        
                        let edge_cmd = CreateEdgeCommand {
                            source: nodes[source_idx],
                            target: nodes[target_idx],
                            labels: vec!["connection".to_string()],
                        };
                        
                        self.execute_command(&edge_cmd);
                    }
                }
                
                if ui.button("Update Graph System").clicked() {
                    // Run the position update system
                    self.graph_system.update_positions();
                }
                
                if ui.button("Synchronize Systems").clicked() {
                    // Export the ECS data to a graph, which will replace our information graph
                    self.information_graph = self.graph_system.export_to_graph();
                }
                
                if ui.button("Reset Layout").clicked() {
                    // Clear node positions to trigger recalculation
                    ctx.memory_mut(|mem| mem.data.remove::<std::collections::HashMap<Uuid, egui::Pos2>>(node_positions_id));
                }
                
                ui.separator();
                
                if ui.button("Clear Graph").clicked() {
                    // Clear the entire graph
                    self.information_graph = AlchemistGraph::new();
                }
                
                if ui.button("Export Graph").clicked() {
                    // Placeholder for export functionality
                    // Would save the graph to a file in a real implementation
                }
                
                if ui.button("Import Graph").clicked() {
                    // Placeholder for import functionality
                    // Would load a graph from a file in a real implementation
                }
            });
            
            ui.add_space(5.0);
            
            // Display information about our graph
            ui.horizontal(|ui| {
                ui.strong(format!("Nodes: {}", self.information_graph.nodes.len()));
                ui.strong(format!("Edges: {}", self.information_graph.edges.len()));
                
                if let Some(node_id) = selected_node {
                    ui.separator();
                    ui.strong(format!("Selected: {}", 
                        self.information_graph.nodes.get(&node_id)
                            .map(|n| n.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string())
                    ));
                }
            });
            
            ui.separator();
            
            // Draw the graph visualization
            let rect = ui.available_rect_before_wrap();
            let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
            let painter = ui.painter();
            
            // Calculate positions for nodes based on selected layout
            if !self.information_graph.nodes.is_empty() {
                // Remove any positions for nodes that no longer exist
                node_positions.retain(|id, _| self.information_graph.nodes.contains_key(id));
                
                // Apply force-directed layout if active
                if self.applying_force_layout {
                    self.apply_force_directed_layout(&mut node_positions);
                    // Request a repaint to animate the layout
                    ctx.request_repaint();
                }
                
                // Add positions for new nodes based on selected layout
                if node_positions.len() < self.information_graph.nodes.len() {
                    match self.layout_type {
                        LayoutType::Circular => {
                            self.apply_circular_layout(&mut node_positions, rect);
                        },
                        LayoutType::ForceDirected => {
                            // Initialize with circular layout then let force layout take over
                            let mut temp_positions = std::collections::HashMap::new();
                            self.apply_circular_layout(&mut temp_positions, rect);
                            
                            // Copy positions for new nodes
                            for (id, pos) in temp_positions {
                                if !node_positions.contains_key(&id) {
                                    node_positions.insert(id, pos);
                                }
                            }
                            
                            // Start force-directed layout if not already running
                            if !self.applying_force_layout {
                                self.applying_force_layout = true;
                                self.layout_iterations = 0;
                            }
                        },
                        LayoutType::Hierarchical => {
                            self.apply_hierarchical_layout(&mut node_positions, rect);
                        }
                    }
                }
                
                // Draw curved edges with different colors based on edge type
                for (_, edge) in &self.information_graph.edges {
                    if let (Some(src_pos), Some(dst_pos)) = (
                        node_positions.get(&edge.source), 
                        node_positions.get(&edge.target)
                    ) {
                        // Determine edge color based on labels
                        let edge_type_color = if edge.labels.contains(&"connection".to_string()) {
                            self.graph_settings.edge_color
                        } else if edge.labels.contains(&"dependency".to_string()) {
                            egui::Color32::from_rgb(200, 100, 100)
                        } else if edge.labels.contains(&"flow".to_string()) {
                            egui::Color32::from_rgb(100, 200, 100)
                        } else {
                            self.graph_settings.edge_color
                        };
                        
                        // Calculate control point for curved edge
                        let mid_point = egui::Pos2::lerp(src_pos, *dst_pos, 0.5);
                        let normal = egui::Vec2::new(-(dst_pos.y - src_pos.y), dst_pos.x - src_pos.x).normalized();
                        
                        // Make self-loops more visible
                        let curve_strength = if edge.source == edge.target {
                            50.0
                        } else {
                            20.0
                        };
                        
                        let control_point = mid_point + normal * curve_strength;
                        
                        // Draw a quadratic bezier curve for the edge
                        let steps = 20; // Number of line segments to approximate the curve
                        let mut prev_point = *src_pos;
                        
                        for i in 1..=steps {
                            let t = i as f32 / steps as f32;
                            
                            // Quadratic Bezier curve calculation using proper references
                            let p0 = prev_point.lerp(control_point, t);
                            let p1 = control_point.lerp(*dst_pos, t);
                            let curve_point = p0.lerp(p1, t);
                            
                            // Draw line segment of the curve using tuples converted to array
                            painter.line_segment(
                                (prev_point, curve_point).into(),
                                egui::Stroke::new(self.graph_settings.edge_thickness, edge_type_color)
                            );
                            
                            prev_point = curve_point;
                        }
                        
                        // Draw arrowhead if enabled
                        if self.graph_settings.show_arrows {
                            // Calculate direction vector for arrow at the end of the curve
                            let last_segment = *dst_pos - prev_point;
                            let dir = last_segment.normalized();
                            let arrow_size = 10.0;
                            let arrow_pos = *dst_pos - dir * self.graph_settings.node_size; // offset by node radius
                            
                            // Draw arrowhead lines
                            let perpendicular = egui::Vec2::new(-dir.y, dir.x);
                            let arrow_left = arrow_pos - perpendicular * arrow_size - dir * arrow_size;
                            let arrow_right = arrow_pos + perpendicular * arrow_size - dir * arrow_size;
                            
                            painter.line_segment(
                                (arrow_pos, arrow_left).into(),
                                egui::Stroke::new(self.graph_settings.edge_thickness, edge_type_color)
                            );
                            painter.line_segment(
                                (arrow_pos, arrow_right).into(),
                                egui::Stroke::new(self.graph_settings.edge_thickness, edge_type_color)
                            );
                        }
                    }
                }
                
                // Check for node dragging and clicks
                let mut hovered_node: Option<Uuid> = None;
                let mut clicked_node: Option<Uuid> = None;
                
                // Draw the nodes and check for interactions
                for (id, node) in &self.information_graph.nodes {
                    let pos = *node_positions.get(id).unwrap();
                    
                    // Determine node color based on selection
                    let current_node_color = if Some(*id) == selected_node {
                        egui::Color32::from_rgb(220, 100, 100) // Highlighted color
                    } else {
                        self.graph_settings.node_color // Default color from settings
                    };
                    
                    // Draw node circle
                    painter.circle_filled(pos, self.graph_settings.node_size, current_node_color);
                    
                    // Draw node text (name) if enabled
                    if self.graph_settings.show_node_labels {
                        painter.text(
                            pos,
                            egui::Align2::CENTER_CENTER,
                            &node.name,
                            egui::FontId::proportional(24.0),
                            egui::Color32::WHITE
                        );
                    }
                    
                    // Draw node properties if enabled
                    if self.graph_settings.show_node_properties && node.properties.len() > 0 {
                        let props_text = node.properties.iter()
                            .take(3) // Show at most 3 properties
                            .map(|(k, v)| format!("{}: {}", k, v))
                            .collect::<Vec<_>>()
                            .join("\n");
                            
                        let props_pos = pos + egui::vec2(0.0, self.graph_settings.node_size + 5.0);
                        painter.text(
                            props_pos,
                            egui::Align2::CENTER_TOP,
                            props_text,
                            egui::FontId::proportional(16.0),
                            egui::Color32::LIGHT_GRAY
                        );
                    }
                    
                    // Check if mouse is over this node
                    let node_rect = egui::Rect::from_center_size(
                        pos, 
                        egui::Vec2::new(self.graph_settings.node_size * 2.0, self.graph_settings.node_size * 2.0)
                    );
                    
                    // Check for hover and clicks
                    if node_rect.contains(response.hover_pos().unwrap_or_default()) {
                        hovered_node = Some(*id);
                        
                        // Show tooltip on hover
                        ui.ctx().debug_painter().text(
                            response.hover_pos().unwrap_or_default() + egui::vec2(15.0, 15.0),
                            egui::Align2::LEFT_TOP,
                            format!("{}\nID: {}\nProperties: {}\nLabels: {}", 
                                node.name, 
                                id, 
                                node.properties.len(), 
                                node.labels.join(", ")
                            ),
                            egui::FontId::proportional(18.0),
                            egui::Color32::WHITE,
                        );
                        
                        if response.clicked() {
                            clicked_node = Some(*id);
                        }
                    }
                    
                    // Handle dragging
                    if response.dragged() && response.hover_pos().is_some() {
                        if let Some(hover_pos) = response.hover_pos() {
                            if node_rect.contains(hover_pos) || Some(*id) == selected_node {
                                let delta = response.drag_delta();
                                let mut curr_pos = pos;
                                curr_pos.x += delta.x;
                                curr_pos.y += delta.y;
                                node_positions.insert(*id, curr_pos);
                            }
                        }
                    }
                }
                
                // Update selected node if a node was clicked
                if let Some(node_id) = clicked_node {
                    if selected_node == Some(node_id) {
                        selected_node = None; // Deselect if clicked again
                    } else {
                        selected_node = Some(node_id); // Select new node
                    }
                } else if response.clicked() && hovered_node.is_none() {
                    // Clicked empty space, deselect
                    selected_node = None;
                }
            } else {
                // Show a message if there are no nodes
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "No nodes in graph - Click 'Generate Test Nodes' to create some",
                    egui::FontId::proportional(32.0),
                    egui::Color32::WHITE
                );
            }
            
            // Show detailed information about selected node
            if let Some(node_id) = selected_node {
                if let Some(node) = self.information_graph.nodes.get(&node_id) {
                    ui.separator();
                    ui.heading(format!("Selected Node: {}", node.name));
                    ui.label(format!("ID: {}", node_id));
                    
                    ui.label("Properties:");
                    for (key, value) in &node.properties {
                        ui.horizontal(|ui| {
                            ui.label(format!("• {}: {}", key, value));
                        });
                    }
                    
                    ui.label(format!("Labels: {}", node.labels.join(", ")));
                    ui.label(format!("Radius: {:.2}", node.radius));
                    
                    // Show connected edges
                    ui.collapsing("Connected Edges", |ui| {
                        let mut has_edges = false;
                        
                        ui.label("Outgoing:");
                        for (_, edge) in &self.information_graph.edges {
                            if edge.source == node_id {
                                has_edges = true;
                                if let Some(target) = self.information_graph.nodes.get(&edge.target) {
                                    ui.label(format!("→ {} ({})", target.name, edge.target));
                                }
                            }
                        }
                        
                        ui.label("Incoming:");
                        for (_, edge) in &self.information_graph.edges {
                            if edge.target == node_id {
                                has_edges = true;
                                if let Some(source) = self.information_graph.nodes.get(&edge.source) {
                                    ui.label(format!("← {} ({})", source.name, edge.source));
                                }
                            }
                        }
                        
                        if !has_edges {
                            ui.label("No connected edges");
                        }
                    });
                }
            }
        });
        
        // Store node positions and selection for the next frame
        ctx.memory_mut(|mem| mem.data.insert_persisted(node_positions_id, node_positions));
        ctx.memory_mut(|mem| mem.data.insert_persisted(selected_node_id, selected_node));
    }
    
    fn show_workflow_view(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.workflow_editor.ui(ui);
        });
    }
    
    fn show_settings_view(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Alchemist Settings");
            ui.label("Configure the application settings here.");
            
            ui.add_space(20.0);
            
            ui.collapsing("About Graphs in Alchemist", |ui| {
                ui.label("Alchemist represents the universe as a hypergraph.");
                ui.label("A hypergraph is a special graph that allows an edge to connect more than two nodes.");
                ui.add_space(10.0);
                ui.label("Graph Features:");
                ui.label("• Nodes represent things (nouns)");
                ui.label("• Edges represent relationships (actions/behaviors)");
                ui.label("• Edges have direction");
                ui.label("• Nodes and edges have properties and labels");
                ui.label("• Nodes have a radius");
                ui.label("• Edges have a weight");
            });
            
            ui.collapsing("About Workflows", |ui| {
                ui.label("Workflows are a special type of graph representing sequential processes.");
                ui.label("Use the Workflow Editor to create and edit workflow graphs.");
            });
            
            ui.collapsing("About ECS", |ui| {
                ui.label("Entity Component System Architecture:");
                ui.label("• Entities are identifiable objects with unique identifiers");
                ui.label("• Components are values (collections of data structures)");
                ui.label("• Systems provide behavior and functionality");
                ui.add_space(10.0);
                ui.label("The system maintains a separation of data and behavior.");
            });
            
            ui.collapsing("About Events", |ui| {
                ui.label("The event system follows the Command-Event pattern:");
                ui.label("• Commands produce Events");
                ui.label("• Events are applied to models");
                ui.label("• The event stream is append-only and sequential");
                ui.label("• Models are reconstructed by replaying events");
            });
        });
    }
    
    fn show_events_view(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Event Stream");
            ui.label(format!("Total events: {}", self.event_stream.len()));
            
            ui.add_space(10.0);
            
            if ui.button("Replay All Events").clicked() {
                // Clear and reconstruct the models from events
                self.information_graph = AlchemistGraph::new();
                
                // Re-create the graph system
                self.graph_system = GraphSystem::new();
                
                // Replay all events in order
                for event in self.event_stream.get_events() {
                    if let Some(graph_event) = event.as_any().downcast_ref::<GraphEvent>() {
                        self.information_graph.apply_event(graph_event);
                        self.graph_system.apply_event(graph_event);
                    }
                }
            }
            
            // Display events in a scrollable list
            egui::ScrollArea::vertical().show(ui, |ui| {
                for event in self.event_stream.get_events() {
                    ui.group(|ui| {
                        ui.label(format!("Type: {}", event.event_type()));
                        if let Some(entity_id) = event.entity_id() {
                            ui.label(format!("Entity: {}", entity_id));
                        }
                        ui.label(format!("Timestamp: {}", event.timestamp()));
                        
                        // Try to downcast to GraphEvent to get payload
                        if let Some(graph_event) = event.as_any().downcast_ref::<GraphEvent>() {
                            ui.label("Payload:");
                            for (key, value) in &graph_event.payload {
                                ui.label(format!("  {}: {}", key, value));
                            }
                        }
                    });
                    ui.add_space(5.0);
                }
            });
        });
    }
}