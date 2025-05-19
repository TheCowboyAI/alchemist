use crate::graph::{GraphWorkflow};
use crate::models::GraphNodeData;
use egui;
use egui_snarl::ui::{SnarlViewer, SnarlStyle, PinInfo};
use egui_snarl::{Snarl, NodeId, InPin, OutPin};
use uuid::Uuid;
use egui::Ui;
use std::collections::HashMap;
use std::ops::Mul;

// Helper functions for color manipulation
fn lighten_color(color: egui::Color32, factor: f32) -> egui::Color32 {
    let [r, g, b, a] = color.to_array();
    let r = ((r as f32 * (1.0 - factor)) + (255.0 * factor)) as u8;
    let g = ((g as f32 * (1.0 - factor)) + (255.0 * factor)) as u8;
    let b = ((b as f32 * (1.0 - factor)) + (255.0 * factor)) as u8;
    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
}

fn darken_color(color: egui::Color32, factor: f32) -> egui::Color32 {
    let [r, g, b, a] = color.to_array();
    let r = (r as f32 * (1.0 - factor)) as u8;
    let g = (g as f32 * (1.0 - factor)) as u8;
    let b = (b as f32 * (1.0 - factor)) as u8;
    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
}

// A simple viewer implementation for Snarl
pub struct AlchemistSnarlViewer;

// WorkflowEditor is the main editor component
pub struct WorkflowEditor {
    pub workflow: GraphWorkflow,
    pub selected_node: Option<Uuid>,
    pub show_context_menu: bool,
    pub context_menu_pos: egui::Pos2,
    pub node_sizes: HashMap<Uuid, egui::Vec2>,
    pub collapsed_nodes: HashMap<Uuid, bool>,
    pub node_contents: HashMap<Uuid, HashMap<String, String>>,
}

impl WorkflowEditor {
    pub fn new() -> Self {
        let mut workflow = GraphWorkflow::new();
        
        // Initialize with an empty workflow
        workflow.sync_to_snarl();
        
        Self {
            workflow,
            selected_node: None,
            show_context_menu: false,
            context_menu_pos: egui::Pos2::ZERO,
            node_sizes: HashMap::new(),
            collapsed_nodes: HashMap::new(),
            node_contents: HashMap::new(),
        }
    }
    
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Workflow Editor");
            
            // Top toolbar for workflow actions
            ui.horizontal(|ui| {
                if ui.button("New Workflow").clicked() {
                    self.workflow = GraphWorkflow::new();
                    self.node_sizes.clear();
                    self.collapsed_nodes.clear();
                    self.node_contents.clear();
                }
                
                if ui.button("Add Node").clicked() {
                    let node_id = self.workflow.graph.add_node("New Node", vec!["workflow".to_string()]);
                    self.selected_node = Some(node_id);
                    self.node_sizes.insert(node_id, egui::vec2(200.0, 120.0)); // Wider default size
                    self.collapsed_nodes.insert(node_id, false); // Not collapsed by default
                    self.workflow.sync_to_snarl();
                }
                
                if ui.button("Example Workflow").clicked() {
                    self.workflow = GraphWorkflow::new();
                    self.workflow.create_example_workflow();
                    
                    // Setup default sizes and collapsed states for example nodes
                    for (id, _) in &self.workflow.graph.nodes {
                        self.node_sizes.insert(*id, egui::vec2(200.0, 120.0));
                        self.collapsed_nodes.insert(*id, false);
                    }
                    
                    self.workflow.sync_to_snarl();
                }
                
                if ui.button("Decision Workflow").clicked() {
                    self.workflow = GraphWorkflow::new();
                    self.workflow.create_decision_workflow();
                    
                    // Setup appropriate sizes for each node type
                    for (id, node) in &self.workflow.graph.nodes {
                        if node.labels.contains(&"decision".to_string()) {
                            self.node_sizes.insert(*id, egui::vec2(200.0, 140.0)); // Taller for decision node
                        } else if node.labels.contains(&"start".to_string()) || node.labels.contains(&"end".to_string()) {
                            self.node_sizes.insert(*id, egui::vec2(200.0, 100.0));
                        } else {
                            self.node_sizes.insert(*id, egui::vec2(200.0, 120.0));
                        }
                        self.collapsed_nodes.insert(*id, false);
                    }
                    
                    self.workflow.sync_to_snarl();
                }
            });
            
            ui.separator();
            
            // Main editor area using egui_snarl
            if let Some(snarl_graph) = &mut self.workflow.snarl_graph {
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
                
                let mut viewer = AlchemistSnarlViewer;
                
                // Use the show method with proper parameters
                let _response = snarl_graph.show(
                    &mut viewer,
                    &style,
                    "workflow_editor",
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
                    // Update selection based on snarl's internal state
                    if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                        // If we clicked somewhere in the graph, check if any node is at that position
                        for (node_id, node_data) in snarl_graph.nodes_ids_data() {
                            // Get the node position from snarl
                            if let Some(node_info) = snarl_graph.get_node_info(node_id) {
                                let node_pos = node_info.pos;
                                
                                // Simple box check (could be more sophisticated)
                                let default_size = egui::vec2(200.0, 120.0); // Wider default size
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
                                let node_id = self.workflow.graph.add_node("New Node", vec!["workflow".to_string()]);
                                self.selected_node = Some(node_id);
                                self.node_sizes.insert(node_id, egui::vec2(200.0, 120.0)); // Wider default size
                                self.collapsed_nodes.insert(node_id, false); // Not collapsed by default
                                self.workflow.sync_to_snarl();
                                self.show_context_menu = false;
                            }
                            
                            ui.separator();
                            
                            if ui.button("Add Input Node").clicked() {
                                let node_id = self.workflow.graph.add_node("42", vec!["input".to_string()]);
                                self.selected_node = Some(node_id);
                                self.node_sizes.insert(node_id, egui::vec2(200.0, 80.0)); // Wider size
                                self.collapsed_nodes.insert(node_id, false);
                                // Initialize with default value
                                let mut node_content = HashMap::new();
                                node_content.insert("value".to_string(), "42".to_string());
                                self.node_contents.insert(node_id, node_content);
                                self.workflow.sync_to_snarl();
                                self.show_context_menu = false;
                            }
                            
                            if ui.button("Add Math Node").clicked() {
                                let node_id = self.workflow.graph.add_node("a + b", vec!["math".to_string()]);
                                self.selected_node = Some(node_id);
                                self.node_sizes.insert(node_id, egui::vec2(200.0, 120.0)); // Wider size
                                self.collapsed_nodes.insert(node_id, false);
                                // Initialize with default values
                                let mut node_content = HashMap::new();
                                node_content.insert("a".to_string(), "45".to_string());
                                node_content.insert("b".to_string(), "3".to_string());
                                self.node_contents.insert(node_id, node_content);
                                self.workflow.sync_to_snarl();
                                self.show_context_menu = false;
                            }
                            
                            if ui.button("Add Multiply Node").clicked() {
                                let node_id = self.workflow.graph.add_node("foo * bar", vec!["multiply".to_string()]);
                                self.selected_node = Some(node_id);
                                self.node_sizes.insert(node_id, egui::vec2(200.0, 120.0)); // Wider size
                                self.collapsed_nodes.insert(node_id, false);
                                // Initialize with default values
                                let mut node_content = HashMap::new();
                                node_content.insert("foo".to_string(), "630".to_string());
                                node_content.insert("bar".to_string(), "14".to_string());
                                self.node_contents.insert(node_id, node_content);
                                self.workflow.sync_to_snarl();
                                self.show_context_menu = false;
                            }
                            
                            if ui.button("Add Division Node").clicked() {
                                let node_id = self.workflow.graph.add_node("x / y", vec!["division".to_string()]);
                                self.selected_node = Some(node_id);
                                self.node_sizes.insert(node_id, egui::vec2(200.0, 120.0)); // Wider size
                                self.collapsed_nodes.insert(node_id, false);
                                // Initialize with default values
                                let mut node_content = HashMap::new();
                                node_content.insert("x".to_string(), "129".to_string());
                                node_content.insert("y".to_string(), "2.867".to_string());
                                self.node_contents.insert(node_id, node_content);
                                self.workflow.sync_to_snarl();
                                self.show_context_menu = false;
                            }
                            
                            if ui.button("Add Show Image Node").clicked() {
                                let node_id = self.workflow.graph.add_node("Show image", vec!["image".to_string()]);
                                self.selected_node = Some(node_id);
                                self.node_sizes.insert(node_id, egui::vec2(280.0, 80.0)); // Wider for URL
                                self.collapsed_nodes.insert(node_id, false);
                                // Initialize with default URL
                                let mut node_content = HashMap::new();
                                node_content.insert("url".to_string(), "https://www.rust-lang.org/static/images/rust-logo-blk.svg".to_string());
                                self.node_contents.insert(node_id, node_content);
                                self.workflow.sync_to_snarl();
                                self.show_context_menu = false;
                            }
                            
                            if ui.button("Add Sink Node").clicked() {
                                let node_id = self.workflow.graph.add_node("Sink", vec!["sink".to_string()]);
                                self.selected_node = Some(node_id);
                                self.node_sizes.insert(node_id, egui::vec2(200.0, 80.0)); // Wider size
                                self.collapsed_nodes.insert(node_id, false);
                                self.workflow.sync_to_snarl();
                                self.show_context_menu = false;
                            }
                            
                            if ui.button("Add Decision Node").clicked() {
                                let node_id = self.workflow.graph.add_node("Decision", vec!["decision".to_string()]);
                                self.selected_node = Some(node_id);
                                self.node_sizes.insert(node_id, egui::vec2(200.0, 140.0)); // Wider and taller for True/False outputs
                                self.collapsed_nodes.insert(node_id, false);
                                self.workflow.sync_to_snarl();
                                self.show_context_menu = false;
                            }
                            
                            if ui.button("Add Process Node").clicked() {
                                let node_id = self.workflow.graph.add_node("Process Data", vec!["process".to_string()]);
                                self.selected_node = Some(node_id);
                                self.node_sizes.insert(node_id, egui::vec2(200.0, 100.0)); // Wider size
                                self.collapsed_nodes.insert(node_id, false);
                                self.workflow.sync_to_snarl();
                                self.show_context_menu = false;
                            }
                            
                            if ui.button("Add Start Node").clicked() {
                                let node_id = self.workflow.graph.add_node("Start", vec!["start".to_string()]);
                                self.selected_node = Some(node_id);
                                self.node_sizes.insert(node_id, egui::vec2(200.0, 100.0)); // Wider size
                                self.collapsed_nodes.insert(node_id, false);
                                self.workflow.sync_to_snarl();
                                self.show_context_menu = false;
                            }
                            
                            if ui.button("Add End Node").clicked() {
                                let node_id = self.workflow.graph.add_node("End", vec!["end".to_string()]);
                                self.selected_node = Some(node_id);
                                self.node_sizes.insert(node_id, egui::vec2(200.0, 100.0)); // Wider size
                                self.collapsed_nodes.insert(node_id, false);
                                self.workflow.sync_to_snarl();
                                self.show_context_menu = false;
                            }
                        });
                    });
                    
                    // Close the popup when clicking outside
                    if ui.input(|i| i.pointer.primary_clicked() || i.key_pressed(egui::Key::Escape)) {
                        self.show_context_menu = false;
                    }
                }
            } else {
                ui.label("No workflow loaded");
            }
        });
        
        // If a node is selected, show its properties
        self.show_node_properties(ui);
    }
    
    // Get UUID from Snarl NodeId
    fn get_uuid_from_node_id(&self, node_id: NodeId, snarl: &Snarl<GraphNodeData>) -> Option<Uuid> {
        snarl.get_node(node_id).map(|node_data| node_data.uuid)
    }
    
    // Show node properties panel
    pub fn show_node_properties(&mut self, ui: &mut Ui) {
        if let Some(node_id) = self.selected_node {
            let node_name;
            let node_labels: Vec<String>;
            
            // Clone the data we need to avoid mutable borrow issues
            if let Some(node) = self.workflow.graph.get_node(node_id) {
                node_name = node.name.clone();
                node_labels = node.labels.clone();
                
                ui.group(|ui| {
                    ui.heading(&node_name);
                    ui.separator();
                    
                    // Allow resizing the node
                    let size = self.node_sizes.entry(node_id).or_insert(egui::vec2(150.0, 120.0));
                    ui.horizontal(|ui| {
                        ui.label("Size:");
                        ui.add(egui::DragValue::new(&mut size.x).speed(1.0).prefix("W: "));
                        ui.add(egui::DragValue::new(&mut size.y).speed(1.0).prefix("H: "));
                    });
                    
                    // Toggle collapsible state
                    let collapsed = self.collapsed_nodes.entry(node_id).or_insert(false);
                    ui.checkbox(collapsed, "Collapsed");
                    
                    ui.separator();
                    
                    // Edit node name
                    let mut new_name = node_name.clone();
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        if ui.text_edit_singleline(&mut new_name).changed() {
                            if let Some(node) = self.workflow.graph.nodes.get_mut(&node_id) {
                                node.name = new_name;
                                self.workflow.sync_to_snarl();
                            }
                        }
                    });
                    
                    // Display node properties
                    ui.label("Properties:");
                    let node_contents = self.node_contents.entry(node_id).or_insert_with(HashMap::new);
                    
                    // Edit existing properties
                    let mut to_update = Vec::new();
                    for (key, value) in node_contents.iter() {
                        let mut new_value = value.clone();
                        ui.horizontal(|ui| {
                            ui.label(format!("{}:", key));
                            if ui.text_edit_singleline(&mut new_value).changed() {
                                to_update.push((key.clone(), new_value.clone()));
                            }
                        });
                    }
                    
                    // Update any changed values
                    for (key, new_value) in to_update {
                        node_contents.insert(key, new_value);
                    }
                    
                    // Display node labels
                    ui.label("Labels:");
                    for label in &node_labels {
                        ui.label(format!("• {}", label));
                    }
                });
            }
        }
    }
    
    // New helper function to draw a 3D sphere node header
    fn draw_sphere_header(&self, ui: &mut egui::Ui, title: &str, color: egui::Color32) {
        let available_width = ui.available_width();
        let sphere_size = 36.0;
        let height = sphere_size + 8.0;
        
        let (response, painter) = ui.allocate_painter(
            egui::vec2(available_width, height),
            egui::Sense::hover()
        );
        
        let rect = response.rect;
        let center = rect.center_top() + egui::vec2(0.0, sphere_size/2.0 + 4.0);
        
        // Base sphere
        painter.circle_filled(center, sphere_size/2.0, color);
        
        // Light highlight (top-left quarter)
        let highlight_color = lighten_color(color, 0.5);
        let highlight_center = center - egui::vec2(sphere_size/6.0, sphere_size/6.0);
        painter.circle_filled(highlight_center, sphere_size/4.0, highlight_color);
        
        // Shadow (bottom-right quarter)
        let shadow_color = darken_color(color, 0.5);
        let shadow_center = center + egui::vec2(sphere_size/6.0, sphere_size/6.0);
        painter.circle_filled(shadow_center, sphere_size/4.0, shadow_color);
        
        // Calculate the position for orbiting text based on time
        let time = ui.ctx().input(|i| i.time) as f32;
        let angle = (time * 0.5) % (2.0 * std::f32::consts::PI);
        let orbit_radius = sphere_size * 0.7;
        let text_pos = center + egui::vec2(
            orbit_radius * angle.cos(),
            orbit_radius * angle.sin(),
        );
        
        // Draw the title text
        let font_id = egui::FontId::proportional(18.0);
        let text_color = egui::Color32::WHITE;
        painter.text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            title,
            font_id,
            text_color
        );
        
        // Request continuous repaint for animation
        ui.ctx().request_repaint();
    }
}

/// Custom viewer for our snarl graph
impl SnarlViewer<GraphNodeData> for AlchemistSnarlViewer {
    fn title(&mut self, node: &GraphNodeData) -> String {
        node.name.clone()
    }
    
    fn has_body(&mut self, _node: &GraphNodeData) -> bool {
        true
    }
    
    fn inputs(&mut self, node: &GraphNodeData) -> usize {
        // Return the number of inputs based on node type
        if node.labels.contains(&"decision".to_string()) {
            1 // Decision node has one input for condition
        } else if node.labels.contains(&"math".to_string()) {
            2 // Math nodes have two inputs (a, b)
        } else if node.labels.contains(&"multiply".to_string()) {
            2 // Multiply nodes have two inputs (foo, bar)
        } else if node.labels.contains(&"division".to_string()) {
            2 // Division nodes have two inputs (x, y)
        } else if node.labels.contains(&"input".to_string()) {
            0 // Input nodes have no inputs
        } else if node.labels.contains(&"image".to_string()) {
            1 // Image nodes have one input (url)
        } else if node.labels.contains(&"sink".to_string()) {
            1 // Sink nodes have one input
        } else if node.labels.contains(&"start".to_string()) {
            0 // Start node has no inputs
        } else if node.labels.contains(&"end".to_string()) {
            1 // End node has one input
        } else if node.labels.contains(&"process".to_string()) {
            1 // Process node has one input
        } else {
            1 // Default to one input
        }
    }
    
    fn outputs(&mut self, node: &GraphNodeData) -> usize {
        // Return the number of outputs based on node type
        if node.labels.contains(&"decision".to_string()) {
            2 // Decision nodes have two outputs (True/False)
        } else if node.labels.contains(&"input".to_string()) {
            1 // Input nodes have one output
        } else if node.labels.contains(&"math".to_string()) {
            1 // Math nodes have one output
        } else if node.labels.contains(&"multiply".to_string()) {
            1 // Multiply nodes have one output
        } else if node.labels.contains(&"division".to_string()) {
            1 // Division nodes have one output
        } else if node.labels.contains(&"sink".to_string()) {
            0 // Sink nodes have no outputs
        } else if node.labels.contains(&"start".to_string()) {
            1 // Start node has one output
        } else if node.labels.contains(&"end".to_string()) {
            0 // End node has no outputs
        } else if node.labels.contains(&"process".to_string()) {
            1 // Process node has one output
        } else {
            1 // Default to one output
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
        // Get node data
        if let Some(node) = snarl.get_node(node_id) {
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 3.0); // Reduce spacing between items
            
            // Get node type color for the sphere
            let node_color = if node.labels.contains(&"start".to_string()) {
                egui::Color32::from_rgb(0, 128, 255) // Blue for start nodes
            } else if node.labels.contains(&"process".to_string()) {
                egui::Color32::from_rgb(60, 179, 113) // Green for process nodes
            } else if node.labels.contains(&"decision".to_string()) {
                egui::Color32::from_rgb(255, 165, 0) // Orange for decision nodes
            } else if node.labels.contains(&"end".to_string()) {
                egui::Color32::from_rgb(220, 20, 60) // Crimson for end nodes
            } else {
                egui::Color32::from_rgb(100, 149, 237) // Default cornflower blue
            };
            
            // Draw 3D sphere header with node name
            let (rect, response) = ui.allocate_exact_size(
                egui::Vec2::new(ui.available_width(), 40.0),
                egui::Sense::click()
            );
            
            let painter = ui.painter();
            
            // Draw the base sphere
            let sphere_center = rect.center();
            let sphere_radius = 15.0;
            painter.circle_filled(sphere_center, sphere_radius, node_color);
            
            // Draw highlight (upper-left light effect)
            let highlight_pos = sphere_center - egui::vec2(sphere_radius * 0.5, sphere_radius * 0.5);
            let highlight_radius = sphere_radius * 0.4;
            painter.circle_filled(
                highlight_pos, 
                highlight_radius, 
                lighten_color(node_color, 0.5)
            );
            
            // Draw shadow (lower-right shadow effect)
            let shadow_pos = sphere_center + egui::vec2(sphere_radius * 0.4, sphere_radius * 0.4);
            let shadow_radius = sphere_radius * 0.5;
            painter.circle_filled(
                shadow_pos, 
                shadow_radius, 
                darken_color(node_color, 0.5)
            );
            
            // Orbit the label around the sphere based on time
            let time = ui.ctx().input(|i| i.time) as f32;
            let orbit_radius = sphere_radius * 1.5;
            let angle = (time * 0.3) % (2.0 * std::f32::consts::PI);
            
            let text_pos = sphere_center + egui::vec2(
                orbit_radius * angle.cos(),
                orbit_radius * angle.sin()
            );
            
            // Draw the title
            painter.text(
                text_pos,
                egui::Align2::CENTER_CENTER,
                &node.name,
                egui::FontId::proportional(12.0),
                egui::Color32::WHITE
            );
            
            // Request continuous repaint for animation
            ui.ctx().request_repaint();
            
            // Add some space after the sphere header
            ui.add_space(5.0);
            
            // Display the node's properties in a framed section
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgb(30, 30, 35))
                .show(ui, |ui| {
                    match true {
                        _ if node.labels.contains(&"math".to_string()) => {
                            ui.vertical(|ui| {
                                // Show input fields for a and b with red square labels
                                ui.horizontal(|ui| {
                                    ui.add(egui::Label::new(egui::RichText::new("⬛ a").color(egui::Color32::RED)));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add(egui::TextEdit::singleline(&mut "45".to_string())
                                            .desired_width(50.0));
                                    });
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.add(egui::Label::new(egui::RichText::new("⬛ b").color(egui::Color32::RED)));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add(egui::TextEdit::singleline(&mut "3".to_string())
                                            .desired_width(50.0));
                                    });
                                });
                            });
                        },
                        _ if node.labels.contains(&"multiply".to_string()) => {
                            ui.vertical(|ui| {
                                // Show input fields for foo and bar with red square labels
                                ui.horizontal(|ui| {
                                    ui.add(egui::Label::new(egui::RichText::new("⬛ foo").color(egui::Color32::RED)));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add(egui::TextEdit::singleline(&mut "630".to_string())
                                            .desired_width(50.0));
                                    });
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.add(egui::Label::new(egui::RichText::new("⬛ bar").color(egui::Color32::RED)));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add(egui::TextEdit::singleline(&mut "14".to_string())
                                            .desired_width(50.0));
                                    });
                                });
                            });
                        },
                        _ if node.labels.contains(&"division".to_string()) => {
                            ui.vertical(|ui| {
                                // Show input fields for x and y with red square labels
                                ui.horizontal(|ui| {
                                    ui.add(egui::Label::new(egui::RichText::new("⬛ x").color(egui::Color32::RED)));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add(egui::TextEdit::singleline(&mut "129".to_string())
                                            .desired_width(50.0));
                                    });
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.add(egui::Label::new(egui::RichText::new("⬛ y").color(egui::Color32::RED)));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add(egui::TextEdit::singleline(&mut "2.867".to_string())
                                            .desired_width(50.0));
                                    });
                                });
                            });
                        },
                        _ if node.labels.contains(&"input".to_string()) => {
                            ui.vertical(|ui| {
                                // Show single value for input node
                                ui.horizontal(|ui| {
                                    ui.add(egui::Label::new(egui::RichText::new("Value:").strong()));
                                    ui.add(egui::TextEdit::singleline(&mut "42".to_string())
                                        .desired_width(50.0));
                                });
                            });
                        },
                        _ if node.labels.contains(&"image".to_string()) => {
                            ui.vertical(|ui| {
                                // Show URL input for image node
                                ui.horizontal(|ui| {
                                    ui.label("URL:");
                                    ui.text_edit_singleline(&mut "https://www.rust-lang.org/static/images/rust-logo-blk.svg".to_string());
                                });
                            });
                        },
                        _ if node.labels.contains(&"decision".to_string()) => {
                            ui.vertical(|ui| {
                                ui.heading("Condition");
                                ui.add_space(2.0);
                                // Allow condition input
                                ui.horizontal(|ui| {
                                    ui.label("Expression:");
                                    ui.text_edit_singleline(&mut "value > 10".to_string());
                                });
                            });
                        },
                        _ if node.labels.contains(&"process".to_string()) || 
                                node.labels.contains(&"start".to_string()) || 
                                node.labels.contains(&"end".to_string()) => {
                            ui.vertical(|ui| {
                                ui.heading("Properties");
                                ui.add_space(2.0);
                                // Add a fixed position toggle
                                let mut fixed = true;
                                ui.checkbox(&mut fixed, "Fixed Position");
                                
                                if node.labels.contains(&"process".to_string()) {
                                    ui.add_space(4.0);
                                    ui.horizontal(|ui| {
                                        ui.label("Process Type:");
                                        let mut selected = "Standard".to_string();
                                        egui::ComboBox::from_id_source("process_type")
                                            .selected_text(&selected)
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(&mut selected, "Standard".to_string(), "Standard");
                                                ui.selectable_value(&mut selected, "Fast".to_string(), "Fast");
                                                ui.selectable_value(&mut selected, "Thorough".to_string(), "Thorough");
                                            });
                                    });
                                }
                            });
                        },
                        _ => {
                            // Default node display
                            ui.vertical(|ui| {
                                ui.heading("Properties");
                                ui.add_space(2.0);
                                for (key, value) in &node.properties {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{}:", key));
                                        ui.text_edit_singleline(&mut value.clone());
                                    });
                                }
                            });
                        }
                    }
                });
        }
    }
    
    fn show_input(
        &mut self, 
        pin: &InPin, 
        ui: &mut egui::Ui, 
        _scale: f32, 
        snarl: &mut Snarl<GraphNodeData>
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        // Get the node to determine pin styling
        let node_data = snarl.get_node(pin.id.node);
        
        // Label inputs differently based on their index and node type
        let (label, color) = if let Some(node) = node_data {
            if node.labels.contains(&"math".to_string()) {
                match pin.id.input {
                    0 => ("a", egui::Color32::RED),
                    1 => ("b", egui::Color32::RED),
                    _ => ("In", egui::Color32::WHITE),
                }
            } else if node.labels.contains(&"multiply".to_string()) {
                match pin.id.input {
                    0 => ("foo", egui::Color32::RED),
                    1 => ("bar", egui::Color32::RED),
                    _ => ("In", egui::Color32::WHITE),
                }
            } else if node.labels.contains(&"division".to_string()) {
                match pin.id.input {
                    0 => ("x", egui::Color32::RED),
                    1 => ("y", egui::Color32::RED),
                    _ => ("In", egui::Color32::WHITE),
                }
            } else if node.labels.contains(&"decision".to_string()) {
                ("In", egui::Color32::RED)
            } else if node.labels.contains(&"process".to_string()) {
                ("In", egui::Color32::RED)
            } else if node.labels.contains(&"end".to_string()) {
                ("In", egui::Color32::RED)
            } else {
                match pin.id.input {
                    0 => ("In", egui::Color32::RED),
                    1 => ("Alt", egui::Color32::YELLOW),
                    _ => ("In", egui::Color32::WHITE),
                }
            }
        } else {
            ("In", egui::Color32::WHITE)
        };
        
        ui.colored_label(color, label);
        PinInfo::square().with_wire_color(color)
    }
    
    fn show_output(
        &mut self, 
        pin: &OutPin, 
        ui: &mut egui::Ui, 
        _scale: f32, 
        snarl: &mut Snarl<GraphNodeData>
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        // Get the node to determine pin styling
        let node_data = snarl.get_node(pin.id.node);
        
        // Label outputs differently based on their index and node type
        let (label, color) = if let Some(node) = node_data {
            if node.labels.contains(&"decision".to_string()) {
                match pin.id.output {
                    0 => ("True", egui::Color32::GREEN),
                    1 => ("False", egui::Color32::RED),
                    _ => ("Out", egui::Color32::WHITE),
                }
            } else if node.labels.contains(&"input".to_string()) {
                ("Out", egui::Color32::RED)
            } else if node.labels.contains(&"math".to_string()) || 
                      node.labels.contains(&"multiply".to_string()) ||
                      node.labels.contains(&"division".to_string()) {
                ("Out", egui::Color32::RED)
            } else if node.labels.contains(&"image".to_string()) {
                ("Out", egui::Color32::YELLOW)
            } else if node.labels.contains(&"start".to_string()) {
                ("Out", egui::Color32::RED)
            } else if node.labels.contains(&"process".to_string()) {
                ("Out", egui::Color32::RED)
            } else {
                match pin.id.output {
                    0 => ("Out", egui::Color32::RED),
                    1 => ("Alt", egui::Color32::YELLOW),
                    _ => ("Out", egui::Color32::WHITE),
                }
            }
        } else {
            ("Out", egui::Color32::WHITE)
        };
        
        ui.colored_label(color, label);
        PinInfo::circle().with_wire_color(color)
    }
}

 