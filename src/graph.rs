use egui_snarl::Snarl;
use std::collections::HashMap;
use uuid::Uuid;
use crate::events::{GraphEvent, GraphEventType, Model};

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
}

impl AlchemistGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
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

    pub fn to_snarl_graph(&self) -> Snarl<GraphNodeData> {
        let graph = Snarl::default();
        
        // This is a placeholder as we don't have direct access to the snarl API
        // In a real implementation, you'd use the actual API to add nodes and wires
        // For each node, create a GraphNodeData and add it to the graph
        // For each edge, create connections between nodes
        
        graph
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
                        .unwrap_or_else(Vec::new);
                    
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
                                .unwrap_or_else(Vec::new);
                            
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

#[derive(Debug, Clone)]
pub struct GraphNodeData {
    pub uuid: Uuid,
    pub name: String,
    pub properties: HashMap<String, String>,
    pub labels: Vec<String>,
    pub radius: f32,
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
        // Create a simple workflow with 4 steps
        let start = self.graph.add_node("Start", vec!["workflow".to_string()]);
        let process = self.graph.add_node("Process Data", vec!["workflow".to_string()]);
        let decision = self.graph.add_node("Decision", vec!["workflow".to_string(), "decision".to_string()]);
        let end = self.graph.add_node("End", vec!["workflow".to_string()]);
        
        // Connect the nodes in sequence
        self.graph.add_edge(start, process, vec!["flow".to_string()]);
        self.graph.add_edge(process, decision, vec!["flow".to_string()]);
        self.graph.add_edge(decision, end, vec!["flow".to_string()]);
        
        // Update the Snarl graph
        self.sync_to_snarl();
    }
} 