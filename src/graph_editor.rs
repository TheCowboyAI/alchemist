use crate::graph::{AlchemistGraph};
use crate::models::GraphNodeData;
use bevy::prelude::*;
use bevy_egui::egui;
use egui_snarl::ui::{SnarlViewer, SnarlStyle, PinInfo};
use egui_snarl::{Snarl, NodeId, InPin, OutPin};
use uuid::Uuid;
use egui::Ui;
use std::collections::HashMap;
use crate::dashboard_ui::ToggleStandardGraphEditorEvent;

// A simple viewer implementation for Snarl
pub struct GraphSnarlViewer;

// GraphEditor is the main editor component
#[derive(Resource)]
pub struct GraphEditor {
    pub graph: AlchemistGraph,
    pub snarl_graph: Option<Snarl<GraphNodeData>>,
    pub selected_node: Option<Uuid>,
    pub show_context_menu: bool,
    pub context_menu_pos: egui::Pos2,
    pub node_sizes: HashMap<Uuid, egui::Vec2>,
    pub collapsed_nodes: HashMap<Uuid, bool>,
    pub node_contents: HashMap<Uuid, HashMap<String, String>>,
    pub visible: bool,
    pub window_pos: Option<egui::Pos2>,
}

impl Default for GraphEditor {
    fn default() -> Self {
        let graph = AlchemistGraph::new();
        
        Self {
            graph,
            snarl_graph: None,
            selected_node: None,
            show_context_menu: false,
            context_menu_pos: egui::Pos2::ZERO,
            node_sizes: HashMap::new(),
            collapsed_nodes: HashMap::new(),
            node_contents: HashMap::new(),
            visible: false,
            window_pos: None,
        }
    }
}

impl GraphEditor {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn ui(&mut self, ctx: &mut egui::Context) {
        if !self.visible {
            return;
        }
        
        let mut window = egui::Window::new("Graph Editor")
            .default_size([800.0, 600.0]);
            
        if let Some(pos) = self.window_pos {
            window = window.default_pos(pos);
        }
        
        let response = window.show(ctx, |ui| {
            self.show_editor_content(ui);
        });
        
        // Save window position for next frame
        if let Some(inner_response) = response {
            self.window_pos = Some(inner_response.response.rect.min);
        }
    }

    fn show_editor_content(&mut self, ui: &mut egui::Ui) {
        // Top toolbar for graph actions
        ui.horizontal(|ui| {
            if ui.button("New Graph").clicked() {
                self.graph = AlchemistGraph::new();
                self.node_sizes.clear();
                self.collapsed_nodes.clear();
                self.node_contents.clear();
                self.sync_to_snarl();
            }
            
            if ui.button("Add Node").clicked() {
                let node_id = self.graph.add_node("New Node", vec!["basic".to_string()]);
                self.selected_node = Some(node_id);
                self.node_sizes.insert(node_id, egui::vec2(200.0, 120.0)); // Default size
                self.collapsed_nodes.insert(node_id, false); // Not collapsed by default
                self.sync_to_snarl();
            }
            
            if ui.button("Example Graph").clicked() {
                self.create_example_graph();
                
                // Setup default sizes and collapsed states for example nodes
                for (id, _) in &self.graph.nodes {
                    self.node_sizes.insert(*id, egui::vec2(200.0, 120.0));
                    self.collapsed_nodes.insert(*id, false);
                }
                
                self.sync_to_snarl();
            }
        });
        
        ui.separator();
        
        // Main editor area using egui_snarl
        if let Some(snarl_graph) = &mut self.snarl_graph {
            // Create a style with collapsible nodes
            let mut style = SnarlStyle::default();
            style.collapsible = Some(true); // Enable collapsible nodes
            
            // Set dark style for nodes with 3D lighting effect
            let node_frame = egui::Frame::new()
                .fill(egui::Color32::from_rgb(40, 40, 45))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 120)))
                .corner_radius(12.0)
                .shadow(egui::epaint::Shadow::default());
            style.node_frame = Some(node_frame);
            
            let mut viewer = GraphSnarlViewer;
            
            // Use the show method with proper parameters
            let _response = snarl_graph.show(
                &mut viewer,
                &style,
                "graph_editor",
                ui
            );
            
            // Handle right-click for context menu
            if ui.input(|i| i.pointer.secondary_clicked()) {
                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    self.show_context_menu = true;
                    self.context_menu_pos = pos;
                }
            }
            
            // Check for node clicks to track selection
            if ui.input(|i| i.pointer.primary_clicked()) {
                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    // If we clicked somewhere in the graph, check if any node is at that position
                    for (node_id, node_data) in snarl_graph.nodes_ids_data() {
                        // Get the node position from snarl
                        if let Some(node_info) = snarl_graph.get_node_info(node_id) {
                            let node_pos = node_info.pos;
                            
                            // Simple box check
                            let default_size = egui::vec2(200.0, 120.0);
                            let node_size = self.node_sizes.entry(node_data.value.uuid).or_insert(default_size).clone();
                            let rect = egui::Rect::from_center_size(node_pos, node_size);
                            
                            if rect.contains(pos) {
                                self.selected_node = Some(node_data.value.uuid);
                                break;
                            }
                        }
                    }
                }
            }
            
            // Show context menu if activated
            if self.show_context_menu {
                let popup_id = ui.make_persistent_id("add_node_popup");
                let area = egui::Area::new(popup_id)
                    .movable(false)
                    .fixed_pos(self.context_menu_pos)
                    .interactable(true);
                
                area.show(ui.ctx(), |ui| {
                    ui.set_max_width(150.0);
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        if ui.button("Add Node").clicked() {
                            let node_id = self.graph.add_node("New Node", vec!["basic".to_string()]);
                            self.selected_node = Some(node_id);
                            self.node_sizes.insert(node_id, egui::vec2(200.0, 120.0));
                            self.collapsed_nodes.insert(node_id, false);
                            self.sync_to_snarl();
                            self.show_context_menu = false;
                        }
                        
                        ui.separator();
                        
                        if ui.button("Add Data Node").clicked() {
                            let node_id = self.graph.add_node("Data", vec!["data".to_string()]);
                            self.selected_node = Some(node_id);
                            self.node_sizes.insert(node_id, egui::vec2(200.0, 80.0));
                            self.collapsed_nodes.insert(node_id, false);
                            self.sync_to_snarl();
                            self.show_context_menu = false;
                        }
                        
                        if ui.button("Add Process Node").clicked() {
                            let node_id = self.graph.add_node("Process", vec!["process".to_string()]);
                            self.selected_node = Some(node_id);
                            self.node_sizes.insert(node_id, egui::vec2(200.0, 120.0));
                            self.collapsed_nodes.insert(node_id, false);
                            self.sync_to_snarl();
                            self.show_context_menu = false;
                        }
                        
                        if ui.button("Add Connection Node").clicked() {
                            let node_id = self.graph.add_node("Connection", vec!["connection".to_string()]);
                            self.selected_node = Some(node_id);
                            self.node_sizes.insert(node_id, egui::vec2(200.0, 120.0));
                            self.collapsed_nodes.insert(node_id, false);
                            self.sync_to_snarl();
                            self.show_context_menu = false;
                        }
                    });
                });
            }
            
            // Show properties panel if a node is selected
            self.show_node_properties(ui);
        }
    }
    
    pub fn show_node_properties(&mut self, ui: &mut Ui) {
        if let Some(selected_node) = self.selected_node {
            // First, create a clone of the node name to avoid borrowing self.graph multiple times
            let mut node_name = String::new();
            let mut node_labels = Vec::new();
            
            // Get node data in a scope to avoid borrowing self.graph
            if let Some(node) = self.graph.nodes.get(&selected_node) {
                node_name = node.name.clone();
                node_labels = node.labels.clone();
            }
            
            ui.separator();
            
            ui.vertical(|ui| {
                ui.heading("Node Properties");
                
                // Node name
                if ui.text_edit_singleline(&mut node_name).changed() {
                    if let Some(node) = self.graph.nodes.get_mut(&selected_node) {
                        node.name = node_name.clone();
                    }
                    self.sync_to_snarl();
                }
                
                // Node type
                let mut node_type = if node_labels.contains(&"data".to_string()) {
                    "Data"
                } else if node_labels.contains(&"process".to_string()) {
                    "Process"
                } else if node_labels.contains(&"connection".to_string()) {
                    "Connection"
                } else {
                    "Basic"
                }.to_string();
                
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::new("node_type", "")
                        .selected_text(&node_type)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut node_type, "Basic".to_string(), "Basic");
                            ui.selectable_value(&mut node_type, "Data".to_string(), "Data");
                            ui.selectable_value(&mut node_type, "Process".to_string(), "Process");
                            ui.selectable_value(&mut node_type, "Connection".to_string(), "Connection");
                        });
                });
                
                // Node parameters based on type
                if node_labels.contains(&"data".to_string()) {
                    ui.label("Data properties:");
                    let node_contents = self.node_contents.entry(selected_node).or_insert_with(|| {
                        let mut map = HashMap::new();
                        map.insert("value".to_string(), "".to_string());
                        map
                    });
                    
                    let mut value = node_contents.get("value").unwrap_or(&"".to_string()).clone();
                    if ui.text_edit_multiline(&mut value).changed() {
                        node_contents.insert("value".to_string(), value);
                    }
                } else if node_labels.contains(&"process".to_string()) {
                    ui.label("Process properties:");
                    let node_contents = self.node_contents.entry(selected_node).or_insert_with(|| {
                        let mut map = HashMap::new();
                        map.insert("description".to_string(), "".to_string());
                        map
                    });
                    
                    let mut description = node_contents.get("description").unwrap_or(&"".to_string()).clone();
                    if ui.text_edit_multiline(&mut description).changed() {
                        node_contents.insert("description".to_string(), description);
                    }
                }
            });
        }
    }
    
    fn create_example_graph(&mut self) {
        self.graph = AlchemistGraph::new();
        
        // Add some example nodes
        let node1 = self.graph.add_node("Data Source", vec!["data".to_string()]);
        let node2 = self.graph.add_node("Process A", vec!["process".to_string()]);
        let node3 = self.graph.add_node("Process B", vec!["process".to_string()]);
        let node4 = self.graph.add_node("Output", vec!["data".to_string()]);
        
        // Connect the nodes
        self.graph.add_edge(node1, node2, vec!["feeds".to_string()]);
        self.graph.add_edge(node2, node3, vec!["next".to_string()]);
        self.graph.add_edge(node3, node4, vec!["produces".to_string()]);
        
        // Add example data to nodes
        let mut node1_data = HashMap::new();
        node1_data.insert("value".to_string(), "Sample data input".to_string());
        self.node_contents.insert(node1, node1_data);
        
        let mut node2_data = HashMap::new();
        node2_data.insert("description".to_string(), "Transforms input data".to_string());
        self.node_contents.insert(node2, node2_data);
        
        let mut node3_data = HashMap::new();
        node3_data.insert("description".to_string(), "Further processing".to_string());
        self.node_contents.insert(node3, node3_data);
        
        let mut node4_data = HashMap::new();
        node4_data.insert("value".to_string(), "Results".to_string());
        self.node_contents.insert(node4, node4_data);
    }
    
    pub fn sync_to_snarl(&mut self) {
        // Create a new Snarl graph from our AlchemistGraph
        let mut snarl = Snarl::new();
        
        // Map to store the UUID to NodeId mapping
        let mut node_id_map = HashMap::new();
        
        // Add all nodes to the snarl graph
        for (id, node) in &self.graph.nodes {
            let node_data = GraphNodeData {
                uuid: *id,
                name: node.name.clone(),
                properties: node.properties.clone(),
                labels: node.labels.clone(),
                radius: 20.0,
            };
            
            // Default position (can be improved)
            let pos = egui::Pos2::new(
                (id.as_u128() % 1000) as f32 * 0.5, 
                (id.as_u128() % 500) as f32 * 0.6
            );
            
            // Use insert_node method
            let node_id = snarl.insert_node(pos, node_data);
            node_id_map.insert(*id, node_id);
        }
        
        // Add all edges to the snarl graph
        for (_, edge) in &self.graph.edges {
            // Convert node IDs to snarl node IDs
            if let (Some(&source_node_id), Some(&target_node_id)) = (
                node_id_map.get(&edge.source),
                node_id_map.get(&edge.target)
            ) {
                // Create pin IDs
                let out_pin_id = egui_snarl::OutPinId { node: source_node_id, output: 0 };
                let in_pin_id = egui_snarl::InPinId { node: target_node_id, input: 0 };
                
                // Connect using the proper method
                snarl.connect(out_pin_id, in_pin_id);
            }
        }
        
        self.snarl_graph = Some(snarl);
    }
}

// Add this system to handle toggling the Graph Editor
pub fn handle_graph_editor_visibility(
    mut events: EventReader<ToggleStandardGraphEditorEvent>,
    mut graph_editor: ResMut<GraphEditor>,
) {
    for event in events.read() {
        graph_editor.visible = event.0;
        
        // If we're making it visible, initialize it if necessary
        if graph_editor.visible && graph_editor.snarl_graph.is_none() {
            graph_editor.sync_to_snarl();
        }
    }
}

// GraphEditorPlugin to manage systems
#[derive(Default)]
pub struct GraphEditorPlugin;

impl Plugin for GraphEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphEditor>()
           .add_systems(Update, handle_graph_editor_visibility);
    }
}

// Implementation for SnarlViewer trait
impl SnarlViewer<GraphNodeData> for GraphSnarlViewer {
    fn title(&mut self, node: &GraphNodeData) -> String {
        node.name.clone()
    }
    
    fn has_body(&mut self, _node: &GraphNodeData) -> bool {
        true // All nodes have a body
    }
    
    fn inputs(&mut self, node: &GraphNodeData) -> usize {
        if node.labels.contains(&"data".to_string()) {
            if node.name.contains("Source") || node.name.contains("Input") {
                0 // Input data nodes don't have inputs
            } else {
                1 // Output data nodes have one input
            }
        } else if node.labels.contains(&"process".to_string()) {
            1 // Process nodes have one input
        } else if node.labels.contains(&"connection".to_string()) {
            2 // Connection nodes have two inputs
        } else {
            1 // Default is one input
        }
    }
    
    fn outputs(&mut self, node: &GraphNodeData) -> usize {
        if node.labels.contains(&"data".to_string()) {
            if node.name.contains("Output") || node.name.contains("Result") {
                0 // Output data nodes don't have outputs
            } else {
                1 // Input data nodes have one output
            }
        } else if node.labels.contains(&"process".to_string()) {
            1 // Process nodes have one output
        } else if node.labels.contains(&"connection".to_string()) {
            1 // Connection nodes have one output
        } else {
            1 // Default is one output
        }
    }
    
    fn show_body(
        &mut self, 
        node_id: NodeId, 
        _inpins: &[InPin], 
        _outpins: &[OutPin], 
        ui: &mut egui::Ui, 
        _scale: f32, 
        snarl: &mut Snarl<GraphNodeData>
    ) {
        if let Some(node_data) = snarl.get_node(node_id) {
            ui.vertical(|ui| {
                ui.add_space(8.0);
                
                if node_data.labels.contains(&"data".to_string()) {
                    ui.label("Data Node");
                    if node_data.name.contains("Source") || node_data.name.contains("Input") {
                        ui.label("Provides data to the graph");
                    } else {
                        ui.label("Stores processed data");
                    }
                } else if node_data.labels.contains(&"process".to_string()) {
                    ui.label("Process Node");
                    ui.label("Transforms input → output");
                } else if node_data.labels.contains(&"connection".to_string()) {
                    ui.label("Connection Node");
                    ui.label("Connects multiple sources");
                } else {
                    ui.label("Basic Node");
                }
                
                ui.add_space(8.0);
            });
        }
    }
    
    fn show_input(
        &mut self, 
        pin: &InPin, 
        _ui: &mut egui::Ui, 
        _scale: f32, 
        snarl: &mut Snarl<GraphNodeData>
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        let _node_data = snarl.get_node(pin.id.node).unwrap();
        
        let pin_color = {
            match pin.id.input {
                0 => egui::Color32::from_rgb(100, 150, 255),
                1 => egui::Color32::from_rgb(200, 120, 255),
                _ => egui::Color32::from_rgb(150, 150, 150),
            }
        };
        
        PinInfo::circle().with_wire_color(pin_color)
    }
    
    fn show_output(
        &mut self, 
        pin: &OutPin, 
        _ui: &mut egui::Ui, 
        _scale: f32, 
        snarl: &mut Snarl<GraphNodeData>
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        let _node_data = snarl.get_node(pin.id.node).unwrap();
        
        let pin_color = egui::Color32::from_rgb(100, 230, 150);
        
        PinInfo::circle().with_wire_color(pin_color)
    }
} 