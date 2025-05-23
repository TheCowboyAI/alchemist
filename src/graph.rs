use egui_snarl::Snarl;
use std::collections::HashMap;
use uuid::Uuid;
use crate::events::{GraphEvent, GraphEventType, Model};

use crate::models::GraphNodeData;

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: Uuid,
    pub name: String,
    pub properties: HashMap<String, String>,
    pub labels: Vec<String>,
    pub radius: f32,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub id: Uuid,
    pub source: Uuid,
    pub target: Uuid,
    pub properties: HashMap<String, String>,
    pub labels: Vec<String>,
    pub weight: f32,
}

#[derive(Debug, Clone)]
pub struct AlchemistGraph {
    pub nodes: HashMap<Uuid, GraphNode>,
    pub edges: HashMap<Uuid, GraphEdge>,
    pub node_positions: HashMap<Uuid, egui::Pos2>,
}

impl AlchemistGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            node_positions: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, name: &str, labels: Vec<String>) -> Uuid {
        let id = Uuid::new_v4();
        let node = GraphNode {
            id,
            name: name.to_string(),
            properties: HashMap::new(),
            labels,
            radius: 1.0,
        };
        self.nodes.insert(id, node);
        id
    }

    pub fn add_edge(&mut self, source: Uuid, target: Uuid, labels: Vec<String>) -> Uuid {
        let id = Uuid::new_v4();
        let edge = GraphEdge {
            id,
            source,
            target,
            properties: HashMap::new(),
            labels,
            weight: 1.0,
        };
        self.edges.insert(id, edge);
        id
    }

    /// Set the weight of an edge
    pub fn set_edge_weight(&mut self, edge_id: Uuid, weight: f32) {
        if let Some(edge) = self.edges.get_mut(&edge_id) {
            edge.weight = weight;
        }
    }

    pub fn to_snarl_graph(&self) -> Snarl<GraphNodeData> {
        let mut graph = Snarl::default();
        
        // Create a map to store node UUIDs to Snarl NodeIds
        let mut node_id_map = HashMap::new();
        
        // Add all nodes to the Snarl graph
        for (uuid, node) in &self.nodes {
            // Convert GraphNode to GraphNodeData
            let node_data = GraphNodeData {
                uuid: *uuid,
                name: node.name.clone(),
                properties: node.properties.clone(),
                labels: node.labels.clone(),
                radius: node.radius,
            };
            
            // Calculate a position based on the node's current position in the graph
            let position = if let Some(pos) = self.node_positions.get(uuid) {
                egui::pos2(pos.x, pos.y)
            } else {
                // Default position if none exists
                egui::pos2(0.0, 0.0)
            };
            
            // Add the node to the Snarl graph with a position
            let node_id = graph.insert_node(position, node_data);
            
            // Store the mapping between UUID and Snarl NodeId
            node_id_map.insert(*uuid, node_id);
        }
        
        // Add all edges as wires between nodes
        for edge in self.edges.values() {
            if let (Some(source_id), Some(target_id)) = (
                node_id_map.get(&edge.source), 
                node_id_map.get(&edge.target)
            ) {
                // Create OutPinId and InPinId for the source and target nodes
                let out_pin_id = egui_snarl::OutPinId { node: *source_id, output: 0 };
                let in_pin_id = egui_snarl::InPinId { node: *target_id, input: 0 };
                
                // Connect the nodes with a wire
                graph.connect(out_pin_id, in_pin_id);
            }
        }
        
        graph
    }

    // Get a specific node by ID
    pub fn get_node(&self, id: Uuid) -> Option<&GraphNode> {
        self.nodes.get(&id)
    }
    
    // Get a specific edge by ID
    pub fn get_edge(&self, id: Uuid) -> Option<&GraphEdge> {
        self.edges.get(&id)
    }
    
    // Add a property to a node
    pub fn add_property(&mut self, node_id: Uuid, key: String, value: String) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.properties.insert(key, value);
        }
    }
}

// Implement Model trait to handle events
impl Model for AlchemistGraph {
    fn apply_event(&mut self, event: &GraphEvent) {
        match event.event_type {
            GraphEventType::NodeCreated => {
                if let Some(entity_id) = event.entity_id {
                    let name = event.payload.get("name")
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "Unnamed Node".to_string());
                    
                    let labels = event.payload.get("labels")
                        .map(|s| s.split(',').map(|label| label.trim().to_string()).collect())
                        .unwrap_or_default();
                    
                    let node = GraphNode {
                        id: entity_id,
                        name,
                        properties: HashMap::new(),
                        labels,
                        radius: 1.0,
                    };
                    
                    self.nodes.insert(entity_id, node);
                }
            },
            GraphEventType::NodeUpdated => {
                if let Some(entity_id) = event.entity_id {
                    if let Some(node) = self.nodes.get_mut(&entity_id) {
                        // Update properties from payload
                        for (key, value) in &event.payload {
                            // Handle special properties
                            match key.as_str() {
                                "name" => node.name = value.clone(),
                                "radius" => {
                                    if let Ok(radius) = value.parse::<f32>() {
                                        node.radius = radius;
                                    }
                                },
                                "labels" => {
                                    node.labels = value.split(',')
                                        .map(|s| s.trim().to_string())
                                        .collect();
                                },
                                _ => { 
                                    // Add as regular property
                                    node.properties.insert(key.clone(), value.clone());
                                }
                            }
                        }
                    }
                }
            },
            GraphEventType::NodeDeleted => {
                if let Some(entity_id) = event.entity_id {
                    self.nodes.remove(&entity_id);
                    
                    // Also remove any edges connected to this node
                    self.edges.retain(|_, edge| {
                        edge.source != entity_id && edge.target != entity_id
                    });
                }
            },
            GraphEventType::EdgeCreated => {
                if let Some(entity_id) = event.entity_id {
                    if let (Some(source_str), Some(target_str)) = 
                        (event.payload.get("source"), event.payload.get("target")) {
                        
                        if let (Ok(source), Ok(target)) = 
                            (Uuid::parse_str(source_str), Uuid::parse_str(target_str)) {
                            
                            let labels = event.payload.get("labels")
                                .map(|s| s.split(',').map(|label| label.trim().to_string()).collect())
                                .unwrap_or_default();
                            
                            let edge = GraphEdge {
                                id: entity_id,
                                source,
                                target,
                                properties: HashMap::new(),
                                labels,
                                weight: 1.0,
                            };
                            
                            self.edges.insert(entity_id, edge);
                        }
                    }
                }
            },
            GraphEventType::EdgeUpdated => {
                if let Some(entity_id) = event.entity_id {
                    if let Some(edge) = self.edges.get_mut(&entity_id) {
                        // Update properties from payload
                        for (key, value) in &event.payload {
                            // Handle special properties
                            match key.as_str() {
                                "weight" => {
                                    if let Ok(weight) = value.parse::<f32>() {
                                        edge.weight = weight;
                                    }
                                },
                                "labels" => {
                                    edge.labels = value.split(',')
                                        .map(|s| s.trim().to_string())
                                        .collect();
                                },
                                _ => { 
                                    // Add as regular property
                                    edge.properties.insert(key.clone(), value.clone());
                                }
                            }
                        }
                    }
                }
            },
            GraphEventType::EdgeDeleted => {
                if let Some(entity_id) = event.entity_id {
                    self.edges.remove(&entity_id);
                }
            },
            GraphEventType::GraphCleared => {
                self.nodes.clear();
                self.edges.clear();
            },
            _ => { /* Ignore other event types */ }
        }
    }
}

pub struct GraphWorkflow {
    pub graph: AlchemistGraph,
    pub snarl_graph: Option<Snarl<GraphNodeData>>,
    pub current_node: Option<Uuid>,
}

impl GraphWorkflow {
    pub fn new() -> Self {
        Self {
            graph: AlchemistGraph::new(),
            snarl_graph: Some(Snarl::default()),
            current_node: None,
        }
    }
    
    pub fn sync_to_snarl(&mut self) {
        self.snarl_graph = Some(self.graph.to_snarl_graph());
    }
    
    // Method to create a simple workflow graph
    pub fn create_example_workflow(&mut self) {
        // Create some basic nodes
        let node1 = self.graph.add_node("Number", vec!["input".to_string()]);
        let node2 = self.graph.add_node("Expression", vec!["math".to_string()]);
        let node3 = self.graph.add_node("Output", vec!["sink".to_string()]);
        
        // Connect nodes
        self.graph.add_edge(node1, node2, vec!["flows_to".to_string()]);
        self.graph.add_edge(node2, node3, vec!["flows_to".to_string()]);
    }
    
    // Create a workflow that matches the reference image
    pub fn create_decision_workflow(&mut self) {
        // Create nodes matching the reference image
        let start_node = self.graph.add_node("Start", vec!["start".to_string()]);
        let process_node = self.graph.add_node("Process Data", vec!["process".to_string()]);
        let decision_node = self.graph.add_node("Decision", vec!["decision".to_string()]);
        let end_node = self.graph.add_node("End", vec!["end".to_string()]);
        
        // Connect nodes with edges
        self.graph.add_edge(start_node, process_node, vec!["flows_to".to_string()]);
        self.graph.add_edge(process_node, decision_node, vec!["flows_to".to_string()]);
        self.graph.add_edge(decision_node, end_node, vec!["true_path".to_string()]);
        
        // Set positions for better layout
        self.graph.node_positions.insert(start_node, egui::pos2(200.0, 300.0));
        self.graph.node_positions.insert(process_node, egui::pos2(450.0, 300.0));
        self.graph.node_positions.insert(decision_node, egui::pos2(700.0, 300.0));
        self.graph.node_positions.insert(end_node, egui::pos2(950.0, 300.0));
    }
} 