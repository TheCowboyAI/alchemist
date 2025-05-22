use crate::graph::{AlchemistGraph, GraphNode, GraphEdge};
use crate::events::{GraphEvent, GraphEventType, Model};
use std::collections::HashMap;
use uuid::Uuid;
use rand::prelude::*;

// Components (Values)
#[derive(Debug, Clone)]
pub struct NodeComponent {
    pub name: String,
    pub radius: f32,
    pub properties: HashMap<String, String>,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EdgeComponent {
    pub source: Uuid,
    pub target: Uuid,
    pub weight: f32,
    pub properties: HashMap<String, String>,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PositionComponent {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone)]
pub struct VisibilityComponent {
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct WorkflowComponent {
    pub is_workflow_node: bool,
    pub step_order: Option<u32>,
}

// Entity Registry
pub struct EntityRegistry {
    entities: HashMap<Uuid, HashMap<String, Box<dyn std::any::Any + Send + Sync>>>,
}

impl EntityRegistry {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
        }
    }

    pub fn create_entity(&mut self) -> Uuid {
        let entity_id = Uuid::new_v4();
        self.entities.insert(entity_id, HashMap::new());
        entity_id
    }

    pub fn add_component<T: 'static + Send + Sync>(&mut self, entity: Uuid, component_name: &str, component: T) {
        if let Some(components) = self.entities.get_mut(&entity) {
            components.insert(component_name.to_string(), Box::new(component));
        }
    }

    pub fn get_component<T: 'static + Clone>(&self, entity: Uuid, component_name: &str) -> Option<T> {
        if let Some(components) = self.entities.get(&entity) {
            if let Some(component) = components.get(component_name) {
                if let Some(typed_component) = component.downcast_ref::<T>() {
                    return Some(typed_component.clone());
                }
            }
        }
        None
    }

    pub fn remove_component(&mut self, entity: Uuid, component_name: &str) {
        if let Some(components) = self.entities.get_mut(&entity) {
            components.remove(component_name);
        }
    }

    pub fn remove_entity(&mut self, entity: Uuid) {
        self.entities.remove(&entity);
    }

    pub fn has_component(&self, entity: Uuid, component_name: &str) -> bool {
        if let Some(components) = self.entities.get(&entity) {
            return components.contains_key(component_name);
        }
        false
    }
    
    // Get the number of entities in the registry
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }
}

// Systems
pub struct GraphSystem {
    pub registry: EntityRegistry,
}

impl GraphSystem {
    pub fn new() -> Self {
        Self {
            registry: EntityRegistry::new(),
        }
    }

    // Convert an AlchemistGraph to ECS entities with components
    pub fn import_graph(&mut self, graph: &AlchemistGraph) {
        // Create entities for nodes
        for (id, node) in &graph.nodes {
            let entity_id = *id; // Use the same ID from the graph
            
            // Add components
            self.registry.add_component(entity_id, "node", NodeComponent {
                name: node.name.clone(),
                radius: node.radius,
                properties: node.properties.clone(),
                labels: node.labels.clone(),
            });
            
            // Add a default position
            let mut rng = rand::rng();
            self.registry.add_component(entity_id, "position", PositionComponent {
                x: rng.random_range(-5.0..5.0),
                y: rng.random_range(-5.0..5.0),
                z: 0.0,
            });
            
            // Add visibility
            self.registry.add_component(entity_id, "visibility", VisibilityComponent {
                visible: true,
            });
            
            // Check if it's a workflow node
            if node.labels.contains(&"workflow".to_string()) {
                self.registry.add_component(entity_id, "workflow", WorkflowComponent {
                    is_workflow_node: true,
                    step_order: None,
                });
            }
        }
        
        // Create entities for edges
        for (id, edge) in &graph.edges {
            let entity_id = *id; // Use the same ID from the graph
            
            // Add edge component
            self.registry.add_component(entity_id, "edge", EdgeComponent {
                source: edge.source,
                target: edge.target,
                weight: edge.weight,
                properties: edge.properties.clone(),
                labels: edge.labels.clone(),
            });
            
            // Add visibility
            self.registry.add_component(entity_id, "visibility", VisibilityComponent {
                visible: true,
            });
        }
    }
    
    // Export ECS entities back to an AlchemistGraph
    pub fn export_to_graph(&self) -> AlchemistGraph {
        let mut graph = AlchemistGraph::new();
        
        // Process all entities
        for (entity_id, components) in &self.registry.entities {
            // Check if it's a node
            if components.contains_key("node") {
                if let Some(node_comp) = self.registry.get_component::<NodeComponent>(*entity_id, "node") {
                    let node = GraphNode {
                        id: *entity_id,
                        name: node_comp.name,
                        properties: node_comp.properties,
                        labels: node_comp.labels,
                        radius: node_comp.radius,
                    };
                    graph.nodes.insert(*entity_id, node);
                }
            }
            
            // Check if it's an edge
            if components.contains_key("edge") {
                if let Some(edge_comp) = self.registry.get_component::<EdgeComponent>(*entity_id, "edge") {
                    let edge = GraphEdge {
                        id: *entity_id,
                        source: edge_comp.source,
                        target: edge_comp.target,
                        properties: edge_comp.properties,
                        labels: edge_comp.labels,
                        weight: edge_comp.weight,
                    };
                    graph.edges.insert(*entity_id, edge);
                }
            }
        }
        
        graph
    }
    
    // A simple system that updates node positions based on a force-directed algorithm
    pub fn update_positions(&mut self) {
        // Get all entities with node and position components
        let entity_ids: Vec<Uuid> = self.registry.entities.keys().cloned().collect();
        
        // For each entity with both node and position
        let mut rng = rand::rng();
        for entity_id in entity_ids {
            if self.registry.has_component(entity_id, "node") && self.registry.has_component(entity_id, "position") {
                if let Some(mut pos) = self.registry.get_component::<PositionComponent>(entity_id, "position") {
                    // Apply some force or movement (simplified)
                    pos.x += (rng.random::<f32>() - 0.5) * 0.1;
                    pos.y += (rng.random::<f32>() - 0.5) * 0.1;
                    
                    // Update the position
                    self.registry.add_component(entity_id, "position", pos);
                }
            }
        }
    }
    
    // A system that processes workflow nodes in order
    pub fn process_workflow(&mut self) {
        // Get all entities with workflow components
        let mut workflow_nodes = Vec::new();
        
        for (entity_id, components) in &self.registry.entities {
            if components.contains_key("workflow") && components.contains_key("node") {
                if let Some(workflow) = self.registry.get_component::<WorkflowComponent>(*entity_id, "workflow") {
                    if workflow.is_workflow_node {
                        workflow_nodes.push(*entity_id);
                    }
                }
            }
        }
        
        // Process each workflow node (simplified)
        for node_id in workflow_nodes {
            if let Some(node) = self.registry.get_component::<NodeComponent>(node_id, "node") {
                println!("Processing workflow node: {}", node.name);
                // Here you would add actual workflow processing logic
            }
        }
    }
}

// Implement Model for GraphSystem to handle events and keep ECS in sync
impl Model for GraphSystem {
    fn apply_event(&mut self, event: &GraphEvent) {
        match event.event_type {
            GraphEventType::NodeCreated => {
                if let Some(entity_id) = event.entity_id {
                    // Create the entity in our ECS with the same ID
                    // The entity was already created in the AlchemistGraph
                    
                    let name = event.payload.get("name")
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "Unnamed Node".to_string());
                    
                    let labels = event.payload.get("labels")
                        .map(|s| s.split(',').map(|label| label.trim().to_string()).collect())
                        .unwrap_or_else(Vec::new);
                    
                    // Add node component
                    self.registry.add_component(entity_id, "node", NodeComponent {
                        name: name.clone(),
                        radius: 1.0,
                        properties: HashMap::new(),
                        labels: labels.clone(),
                    });
                    
                    // Add position component with random position
                    let mut rng = rand::rng();
                    self.registry.add_component(entity_id, "position", PositionComponent {
                        x: rng.random_range(-5.0..5.0),
                        y: rng.random_range(-5.0..5.0),
                        z: 0.0,
                    });
                    
                    // Add visibility component
                    self.registry.add_component(entity_id, "visibility", VisibilityComponent {
                        visible: true,
                    });
                    
                    // Check if it's a workflow node
                    if labels.contains(&"workflow".to_string()) {
                        self.registry.add_component(entity_id, "workflow", WorkflowComponent {
                            is_workflow_node: true,
                            step_order: None,
                        });
                    }
                }
            },
            GraphEventType::NodeUpdated => {
                if let Some(entity_id) = event.entity_id {
                    if self.registry.has_component(entity_id, "node") {
                        if let Some(mut node_comp) = self.registry.get_component::<NodeComponent>(entity_id, "node") {
                            // Update properties from payload
                            for (key, value) in &event.payload {
                                match key.as_str() {
                                    "name" => node_comp.name = value.clone(),
                                    "radius" => {
                                        if let Ok(radius) = value.parse::<f32>() {
                                            node_comp.radius = radius;
                                        }
                                    },
                                    "labels" => {
                                        node_comp.labels = value.split(',')
                                            .map(|s| s.trim().to_string())
                                            .collect();
                                    },
                                    _ => {
                                        // Add as regular property
                                        node_comp.properties.insert(key.clone(), value.clone());
                                    }
                                }
                            }
                            
                            // Update the component
                            self.registry.add_component(entity_id, "node", node_comp);
                        }
                    }
                }
            },
            GraphEventType::NodeDeleted => {
                if let Some(entity_id) = event.entity_id {
                    // Remove the entity from ECS
                    self.registry.remove_entity(entity_id);
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
                            
                            // Add edge component
                            self.registry.add_component(entity_id, "edge", EdgeComponent {
                                source,
                                target,
                                weight: 1.0,
                                properties: HashMap::new(),
                                labels,
                            });
                            
                            // Add visibility component
                            self.registry.add_component(entity_id, "visibility", VisibilityComponent {
                                visible: true,
                            });
                        }
                    }
                }
            },
            GraphEventType::EdgeUpdated => {
                if let Some(entity_id) = event.entity_id {
                    if self.registry.has_component(entity_id, "edge") {
                        if let Some(mut edge_comp) = self.registry.get_component::<EdgeComponent>(entity_id, "edge") {
                            // Update properties from payload
                            for (key, value) in &event.payload {
                                match key.as_str() {
                                    "weight" => {
                                        if let Ok(weight) = value.parse::<f32>() {
                                            edge_comp.weight = weight;
                                        }
                                    },
                                    "labels" => {
                                        edge_comp.labels = value.split(',')
                                            .map(|s| s.trim().to_string())
                                            .collect();
                                    },
                                    _ => {
                                        // Add as regular property
                                        edge_comp.properties.insert(key.clone(), value.clone());
                                    }
                                }
                            }
                            
                            // Update the component
                            self.registry.add_component(entity_id, "edge", edge_comp);
                        }
                    }
                }
            },
            GraphEventType::EdgeDeleted => {
                if let Some(entity_id) = event.entity_id {
                    // Remove the entity from ECS
                    self.registry.remove_entity(entity_id);
                }
            },
            GraphEventType::GraphCleared => {
                // Clear all entities
                for entity_id in self.registry.entities.keys().cloned().collect::<Vec<_>>() {
                    self.registry.remove_entity(entity_id);
                }
            },
            _ => { /* Ignore other event types */ }
        }
    }
} 