use crate::graph::{GraphWorkflow, GraphNodeData};
use egui;
use egui_snarl::ui::{SnarlStyle, SnarlViewer, PinInfo};
use egui_snarl::{Snarl, NodeId, InPin, OutPin};
use uuid::Uuid;

// Create a type alias to work around the private module issue
// This aliases our public Ui type to whatever the trait expects
type UiRef = egui::Ui;

pub struct WorkflowEditor {
    pub workflow: GraphWorkflow,
    pub selected_node: Option<Uuid>,
}

impl WorkflowEditor {
    pub fn new() -> Self {
        let mut workflow = GraphWorkflow::new();
        workflow.create_example_workflow();
        
        Self {
            workflow,
            selected_node: None,
        }
    }
    
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Workflow Editor");
            
            // Top toolbar for workflow actions
            ui.horizontal(|ui| {
                if ui.button("New Workflow").clicked() {
                    self.workflow = GraphWorkflow::new();
                }
                
                if ui.button("Add Node").clicked() {
                    let node_id = self.workflow.graph.add_node("New Node", vec!["workflow".to_string()]);
                    self.selected_node = Some(node_id);
                    self.workflow.sync_to_snarl();
                }
                
                if ui.button("Example Workflow").clicked() {
                    self.workflow = GraphWorkflow::new();
                    self.workflow.create_example_workflow();
                }
            });
            
            ui.separator();
            
            // Main editor area using egui_snarl
            if let Some(_snarl_graph) = &mut self.workflow.snarl_graph {
                let _style = SnarlStyle::default();
                
                // Create snarl viewer for graph
                let _viewer = AlchemistSnarlViewer;
                
                // Show the snarl UI manually since we can't access the native API
                // In a real implementation, this would use egui_snarl's API directly
                let available_size = ui.available_size();
                let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());
                
                // Draw a placeholder for the snarl UI
                ui.painter().rect_filled(
                    rect, 
                    0.0, 
                    egui::Color32::from_rgb(30, 30, 30)
                );
                
                ui.painter().text(
                    rect.center(), 
                    egui::Align2::CENTER_CENTER, 
                    "Node Graph View (Placeholder)",
                    egui::FontId::default(),
                    egui::Color32::WHITE
                );
                
                // Simplified response handling
                let selected_node: Option<NodeId> = None;
                let context_menu_position = if response.secondary_clicked() { 
                    Some(response.interact_pointer_pos().unwrap()) 
                } else { 
                    None 
                };
                
                // Handle node selection
                if let Some(_selected_node) = selected_node {
                    // For now, skip actual node selection since we don't have the right API
                    /*
                    if let Some(node_data) = snarl_graph.node(selected_node) {
                        self.selected_node = Some(node_data.uuid);
                    }
                    */
                }
                
                // Handle node creation via context menu
                if let Some(pos) = context_menu_position {
                    egui::Area::new("context_menu".into())
                        .fixed_pos(pos)
                        .order(egui::Order::Foreground)
                        .show(ui.ctx(), |ui| {
                            egui::Frame::popup(ui.style())
                                .show(ui, |ui| {
                                    if ui.button("Add Node").clicked() {
                                        let node_id = self.workflow.graph.add_node("New Node", vec!["workflow".to_string()]);
                                        self.selected_node = Some(node_id);
                                        self.workflow.sync_to_snarl();
                                    }
                                });
                        });
                }
            } else {
                ui.label("No workflow loaded");
            }
        });
        
        // If a node is selected, show its properties
        self.show_node_properties(ui);
    }
    
    // Separate method to avoid borrowing conflicts
    fn show_node_properties(&mut self, ui: &mut egui::Ui) {
        if let Some(node_id) = self.selected_node {
            // Get a clone of the node to avoid borrow checker issues
            let mut node_clone = match self.workflow.graph.nodes.get(&node_id) {
                Some(n) => n.clone(),
                None => return,
            };
            
            // Show a window with node properties using the cloned node
            let mut node_updated = false;
            
            egui::Window::new(format!("Node: {}", node_clone.name))
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name: ");
                        let mut name = node_clone.name.clone();
                        if ui.text_edit_singleline(&mut name).changed() {
                            node_clone.name = name;
                            node_updated = true;
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Radius: ");
                        let mut radius = node_clone.radius;
                        if ui.add(egui::Slider::new(&mut radius, 0.1..=5.0)).changed() {
                            node_clone.radius = radius;
                            node_updated = true;
                        }
                    });
                    
                    ui.label("Properties:");
                    let mut keys_to_remove = Vec::new();
                    for (key, value) in &mut node_clone.properties {
                        ui.horizontal(|ui| {
                            ui.label(key);
                            let mut val = value.clone();
                            if ui.text_edit_singleline(&mut val).changed() {
                                *value = val;
                                node_updated = true;
                            }
                            if ui.button("🗑").clicked() {
                                keys_to_remove.push(key.clone());
                                node_updated = true;
                            }
                        });
                    }
                    
                    for key in keys_to_remove {
                        node_clone.properties.remove(&key);
                    }
                    
                    ui.horizontal(|ui| {
                        ui.label("Add property:");
                        static mut NEW_KEY: String = String::new();
                        static mut NEW_VALUE: String = String::new();
                        
                        unsafe {
                            ui.text_edit_singleline(&mut NEW_KEY);
                            ui.text_edit_singleline(&mut NEW_VALUE);
                            
                            if ui.button("Add").clicked() && !NEW_KEY.is_empty() {
                                node_clone.properties.insert(NEW_KEY.clone(), NEW_VALUE.clone());
                                NEW_KEY.clear();
                                NEW_VALUE.clear();
                                node_updated = true;
                            }
                        }
                    });
                });
            
            // If node was updated, update the original node in the graph
            if node_updated {
                if let Some(node) = self.workflow.graph.nodes.get_mut(&node_id) {
                    *node = node_clone;
                    self.workflow.sync_to_snarl();
                }
            }
        }
    }
}

/// Custom viewer for our snarl graph
struct AlchemistSnarlViewer;

// Implement the SnarlViewer trait with minimally required methods
impl SnarlViewer<GraphNodeData> for AlchemistSnarlViewer {
    fn title(&mut self, node: &GraphNodeData) -> String {
        node.name.clone()
    }
    
    fn has_body(&mut self, _node: &GraphNodeData) -> bool {
        true
    }
    
    fn inputs(&mut self, _node: &GraphNodeData) -> usize {
        1 // One input
    }
    
    fn outputs(&mut self, _node: &GraphNodeData) -> usize {
        1 // One output
    }
    
    fn show_body(
        &mut self, 
        _node_id: NodeId, 
        _inpins: &[InPin], 
        _outpins: &[OutPin], 
        ui: &mut UiRef, 
        _scale: f32, 
        _snarl: &mut Snarl<GraphNodeData>
    ) {
        ui.label("Node Body");
    }
    
    fn show_input(
        &mut self, 
        _pin: &InPin, 
        ui: &mut UiRef, 
        _scale: f32, 
        _snarl: &mut Snarl<GraphNodeData>
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        ui.label("Input");
        PinInfo::circle()
    }
    
    fn show_output(
        &mut self, 
        _pin: &OutPin, 
        ui: &mut UiRef, 
        _scale: f32, 
        _snarl: &mut Snarl<GraphNodeData>
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        ui.label("Output");
        PinInfo::circle()
    }
} 