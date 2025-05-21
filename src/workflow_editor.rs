use crate::graph::{GraphWorkflow};
use crate::models::GraphNodeData;
use bevy::prelude::*;
use egui_snarl::ui::{SnarlViewer, SnarlStyle, PinInfo};
use egui_snarl::{Snarl, NodeId, InPin, OutPin};
use uuid::Uuid;
use egui::Ui;
use std::collections::HashMap;

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
#[derive(Resource)]
pub struct WorkflowEditor {
    pub workflow: GraphWorkflow,
    pub selected_node: Option<Uuid>,
    pub show_context_menu: bool,
    pub context_menu_pos: egui::Pos2,
    pub node_sizes: HashMap<Uuid, egui::Vec2>,
    pub collapsed_nodes: HashMap<Uuid, bool>,
    pub node_contents: HashMap<Uuid, HashMap<String, String>>,
}

impl Default for WorkflowEditor {
    fn default() -> Self {
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
}

impl WorkflowEditor {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn ui(&mut self, ui: &mut egui::Context) {
        egui::Window::new("Workflow Editor")
            .default_size([800.0, 600.0])
            .show(ui, |ui| {
            self.show_editor_content(ui);
        });
    }

    fn show_editor_content(&mut self, ui: &mut egui::Ui) {
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
                    });
                });
            }
            
            // Show properties panel if a node is selected
            self.show_node_properties(ui);
        }
    }

    fn get_uuid_from_node_id(&self, node_id: NodeId, snarl: &Snarl<GraphNodeData>) -> Option<Uuid> {
        snarl.get_node(node_id).map(|node_data| node_data.uuid)
    }
    
    pub fn show_node_properties(&mut self, ui: &mut Ui) {
        if let Some(selected_node) = self.selected_node {
            // First, create a clone of the node name to avoid borrowing workflow multiple times
            let mut node_name = String::new();
            let mut node_labels = Vec::new();
            
            // Get node data in a scope to avoid borrowing self.workflow
            if let Some(node) = self.workflow.graph.nodes.get(&selected_node) {
                node_name = node.name.clone();
                node_labels = node.labels.clone();
            }
            
            ui.separator();
            
            ui.vertical(|ui| {
                ui.heading("Node Properties");
                
                // Node name
                if ui.text_edit_singleline(&mut node_name).changed() {
                    if let Some(node) = self.workflow.graph.nodes.get_mut(&selected_node) {
                        node.name = node_name.clone();
                    }
                    self.workflow.sync_to_snarl();
                }
                
                // Node type
                let mut node_type = if node_labels.contains(&"input".to_string()) {
                    "Input"
                } else if node_labels.contains(&"math".to_string()) {
                    "Math"
                } else if node_labels.contains(&"multiply".to_string()) {
                    "Multiply"
                } else if node_labels.contains(&"division".to_string()) {
                    "Division"
                } else if node_labels.contains(&"decision".to_string()) {
                    "Decision"
                } else if node_labels.contains(&"start".to_string()) {
                    "Start"
                } else if node_labels.contains(&"end".to_string()) {
                    "End"
                } else {
                    "Standard"
                }.to_string();
                
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::new("node_type", "")
                        .selected_text(&node_type)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut node_type, "Standard".to_string(), "Standard");
                            ui.selectable_value(&mut node_type, "Input".to_string(), "Input");
                            ui.selectable_value(&mut node_type, "Math".to_string(), "Math");
                            ui.selectable_value(&mut node_type, "Multiply".to_string(), "Multiply");
                            ui.selectable_value(&mut node_type, "Division".to_string(), "Division");
                            ui.selectable_value(&mut node_type, "Decision".to_string(), "Decision");
                            ui.selectable_value(&mut node_type, "Start".to_string(), "Start");
                            ui.selectable_value(&mut node_type, "End".to_string(), "End");
                        });
                });
                
                // Node parameters based on type
                if node_labels.contains(&"input".to_string()) {
                    ui.label("Input value:");
                    let node_contents = self.node_contents.entry(selected_node).or_insert_with(|| {
                        let mut map = HashMap::new();
                        map.insert("value".to_string(), "0".to_string());
                        map
                    });
                    
                    let mut value = node_contents.get("value").unwrap_or(&"0".to_string()).clone();
                    if ui.text_edit_singleline(&mut value).changed() {
                        node_contents.insert("value".to_string(), value);
                    }
                } else if node_labels.contains(&"math".to_string()) {
                    ui.label("Math parameters:");
                    let node_contents = self.node_contents.entry(selected_node).or_insert_with(|| {
                        let mut map = HashMap::new();
                        map.insert("a".to_string(), "0".to_string());
                        map.insert("b".to_string(), "0".to_string());
                        map
                    });
                    
                    let mut a = node_contents.get("a").unwrap_or(&"0".to_string()).clone();
                    let mut b = node_contents.get("b").unwrap_or(&"0".to_string()).clone();
                    
                    ui.horizontal(|ui| {
                        ui.label("a:");
                        if ui.text_edit_singleline(&mut a).changed() {
                            node_contents.insert("a".to_string(), a);
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("b:");
                        if ui.text_edit_singleline(&mut b).changed() {
                            node_contents.insert("b".to_string(), b);
                        }
                    });
                }
            });
        }
    }

    fn draw_sphere_header(&self, ui: &mut egui::Ui, title: &str, color: egui::Color32) {
        ui.horizontal(|ui| {
            let desired_size = egui::vec2(16.0, 16.0);
            let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::hover());
            let rect = response.rect;
            let center = rect.center();
            let radius = rect.width() / 2.0 - 1.0;
            
            let base_color = color;
            let highlight_color = lighten_color(base_color, 0.3);
            let shadow_color = darken_color(base_color, 0.3);
            
            // Draw base circle
            painter.circle_filled(center, radius, base_color);
            
            // Draw highlight
            painter.circle_stroke(center + egui::vec2(-1.0, -1.0), radius - 1.0, egui::Stroke::new(1.0, highlight_color));
            
            // Draw shadow
            painter.circle_stroke(center + egui::vec2(1.0, 1.0), radius - 1.0, egui::Stroke::new(1.0, shadow_color));
            
            ui.label(title);
        });
    }
}

// Implementation for SnarlViewer trait
impl SnarlViewer<GraphNodeData> for AlchemistSnarlViewer {
    fn title(&mut self, node: &GraphNodeData) -> String {
        node.name.clone()
    }
    
    fn has_body(&mut self, _node: &GraphNodeData) -> bool {
        true // All nodes have a body
    }
    
    fn inputs(&mut self, node: &GraphNodeData) -> usize {
        if node.labels.contains(&"input".to_string()) {
            0 // Input nodes don't have inputs
        } else if node.labels.contains(&"math".to_string()) || node.labels.contains(&"multiply".to_string()) || node.labels.contains(&"division".to_string()) {
            2 // Math operations have two inputs
        } else if node.labels.contains(&"decision".to_string()) {
            1 // Decision nodes have one input
        } else if node.labels.contains(&"start".to_string()) {
            0 // Start nodes don't have inputs
        } else if node.labels.contains(&"end".to_string()) {
            1 // End nodes have one input
        } else {
            1 // Default is one input
        }
    }
    
    fn outputs(&mut self, node: &GraphNodeData) -> usize {
        if node.labels.contains(&"input".to_string()) {
            1 // Input nodes have one output
        } else if node.labels.contains(&"math".to_string()) || node.labels.contains(&"multiply".to_string()) || node.labels.contains(&"division".to_string()) {
            1 // Math operations have one output
        } else if node.labels.contains(&"decision".to_string()) {
            2 // Decision nodes have two outputs (true/false)
        } else if node.labels.contains(&"start".to_string()) {
            1 // Start nodes have one output
        } else if node.labels.contains(&"end".to_string()) {
            0 // End nodes don't have outputs
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
                
                if node_data.labels.contains(&"input".to_string()) {
                    ui.label("Constant Input Value");
                } else if node_data.labels.contains(&"math".to_string()) {
                    ui.label("Addition Operation");
                    ui.label("Computes: a + b");
                } else if node_data.labels.contains(&"multiply".to_string()) {
                    ui.label("Multiplication Operation");
                    ui.label("Computes: foo * bar");
                } else if node_data.labels.contains(&"division".to_string()) {
                    ui.label("Division Operation");
                    ui.label("Computes: x / y");
                } else if node_data.labels.contains(&"decision".to_string()) {
                    ui.label("Decision Node");
                    ui.label("Routes based on condition");
                } else if node_data.labels.contains(&"start".to_string()) {
                    ui.label("Workflow Start");
                } else if node_data.labels.contains(&"end".to_string()) {
                    ui.label("Workflow End");
                } else {
                    ui.label("Standard Process Node");
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
        
        let (_pin_name, _pin_color) = {
            match pin.id.input {
                0 => ("Value", egui::Color32::from_rgb(100, 150, 255)),
                1 => ("Second", egui::Color32::from_rgb(200, 120, 255)),
                2 => ("Third", egui::Color32::from_rgb(255, 150, 100)),
                _ => ("Input", egui::Color32::WHITE),
            }
        };
        
        let pin_color = {
            match pin.id.input {
                0 => egui::Color32::from_rgb(100, 150, 255),
                1 => egui::Color32::from_rgb(200, 120, 255),
                _ => egui::Color32::from_rgb(150, 150, 150),
            }
        };
        
        let mut pin_info = PinInfo::circle();
        pin_info.wire_color = Some(pin_color);
        PinInfo::circle().with_wire_color(pin_color)
    }
    
    fn show_output(
        &mut self, 
        pin: &OutPin, 
        _ui: &mut egui::Ui, 
        _scale: f32, 
        snarl: &mut Snarl<GraphNodeData>
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        let node_data = snarl.get_node(pin.id.node).unwrap();
        
        let _pin_name = {
            if node_data.labels.contains(&"decision".to_string()) {
                match pin.id.output {
                    0 => "True".to_string(),
                    1 => "False".to_string(),
                    _ => "Output".to_string(),
                }
            } else {
                "Output".to_string()
            }
        };
        
        let pin_color = egui::Color32::from_rgb(100, 230, 150);
        
        let mut pin_info = PinInfo::circle();
        pin_info.wire_color = Some(pin_color);
        PinInfo::circle().with_wire_color(pin_color)
    }
}

 