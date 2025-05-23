use crate::graph::{AlchemistGraph, GraphNode, GraphEdge};
use crate::events::{GraphEvent, GraphEventType, Model};
use std::collections::HashMap;
use uuid::Uuid;
use rand::prelude::*;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::dashboard_ui::ToggleEcsEditorEvent;

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
                    
                    // Update the position by removing and re-adding
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
                    let component = PositionComponent {
                        x: rng.random_range(-5.0..5.0),
                        y: rng.random_range(-5.0..5.0),
                        z: 0.0,
                    };
                    self.registry.add_component(entity_id, "position", component);
                    
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

// ECS-specific edge types
enum EcsRelationType {
    HasComponent,
    MemberOf,
    AccessedBy,
    Owns,
}

// ECS Editor is the main editor component for Entity-Component-System design
#[derive(Resource)]
pub struct EcsEditor {
    pub graph: AlchemistGraph,
    pub entities: Vec<String>,
    pub components: Vec<String>,
    pub systems: Vec<String>,
    pub selected_node: Option<Uuid>,
    pub visible: bool,
    pub window_pos: Option<egui::Pos2>,
}

impl Default for EcsEditor {
    fn default() -> Self {
        Self {
            graph: AlchemistGraph::new(),
            entities: vec!["Player".to_string(), "Enemy".to_string(), "Camera".to_string()],
            components: vec!["Position".to_string(), "Velocity".to_string(), "Health".to_string(), "Damage".to_string()],
            systems: vec!["Movement".to_string(), "Collision".to_string(), "Rendering".to_string()],
            selected_node: None,
            visible: false,
            window_pos: None,
        }
    }
}

impl EcsEditor {
    pub fn create_default_graph(&mut self) {
        self.graph = AlchemistGraph::new();
        
        // Create entity nodes
        let player_id = self.graph.add_node("Player", vec!["entity".to_string()]);
        let enemy_id = self.graph.add_node("Enemy", vec!["entity".to_string()]);
        
        // Create component nodes
        let position_id = self.graph.add_node("Position", vec!["component".to_string()]);
        let velocity_id = self.graph.add_node("Velocity", vec!["component".to_string()]);
        let health_id = self.graph.add_node("Health", vec!["component".to_string()]);
        let damage_id = self.graph.add_node("Damage", vec!["component".to_string()]);
        
        // Create system nodes
        let movement_id = self.graph.add_node("Movement System", vec!["system".to_string()]);
        let collision_id = self.graph.add_node("Collision System", vec!["system".to_string()]);
        
        // Connect entities to components
        self.graph.add_edge(player_id, position_id, vec!["has".to_string()]);
        self.graph.add_edge(player_id, velocity_id, vec!["has".to_string()]);
        self.graph.add_edge(player_id, health_id, vec!["has".to_string()]);
        
        self.graph.add_edge(enemy_id, position_id, vec!["has".to_string()]);
        self.graph.add_edge(enemy_id, velocity_id, vec!["has".to_string()]);
        self.graph.add_edge(enemy_id, health_id, vec!["has".to_string()]);
        self.graph.add_edge(enemy_id, damage_id, vec!["has".to_string()]);
        
        // Connect systems to components they access
        self.graph.add_edge(movement_id, position_id, vec!["accesses".to_string()]);
        self.graph.add_edge(movement_id, velocity_id, vec!["accesses".to_string()]);
        
        self.graph.add_edge(collision_id, position_id, vec!["accesses".to_string()]);
        self.graph.add_edge(collision_id, health_id, vec!["accesses".to_string()]);
        self.graph.add_edge(collision_id, damage_id, vec!["accesses".to_string()]);
    }
    
    pub fn add_entity(&mut self, name: String) {
        let id = self.graph.add_node(&name, vec!["entity".to_string()]);
        self.selected_node = Some(id);
    }
    
    pub fn add_component(&mut self, name: String) {
        let id = self.graph.add_node(&name, vec!["component".to_string()]);
        self.selected_node = Some(id);
    }
    
    pub fn add_system(&mut self, name: String) {
        let id = self.graph.add_node(&name, vec!["system".to_string()]);
        self.selected_node = Some(id);
    }
    
    pub fn connect_entity_to_component(&mut self, entity_id: Uuid, component_id: Uuid) {
        self.graph.add_edge(entity_id, component_id, vec!["has".to_string()]);
    }
    
    pub fn connect_system_to_component(&mut self, system_id: Uuid, component_id: Uuid) {
        self.graph.add_edge(system_id, component_id, vec!["accesses".to_string()]);
    }
}

// Plugin for the ECS Editor
#[derive(Default)]
pub struct EcsEditorPlugin;

impl Plugin for EcsEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<EcsEditor>()
            .add_systems(Startup, setup_ecs_editor)
            .add_systems(Update, (
                handle_ecs_editor_ui,
                handle_ecs_editor_visibility,
            ));
    }
}

fn setup_ecs_editor(mut ecs_editor: ResMut<EcsEditor>) {
    ecs_editor.create_default_graph();
}

fn handle_ecs_editor_ui(
    mut contexts: EguiContexts,
    mut ecs_editor: ResMut<EcsEditor>,
) {
    if !ecs_editor.visible {
        return;
    }
    
    let mut window = egui::Window::new("ECS Editor")
        .default_size([800.0, 600.0]);
        
    if let Some(pos) = ecs_editor.window_pos {
        window = window.default_pos(pos);
    }
    
    let response = window.show(contexts.ctx_mut(), |ui| {
        ui.heading("Entity-Component-System Editor");
        
        // Top toolbar for ECS actions
        ui.horizontal(|ui| {
            if ui.button("New Graph").clicked() {
                ecs_editor.graph = AlchemistGraph::new();
            }
            
            if ui.button("Default Graph").clicked() {
                ecs_editor.create_default_graph();
            }
            
            ui.separator();
            
            if ui.button("Add Entity").clicked() {
                ecs_editor.add_entity("New Entity".to_string());
            }
            
            if ui.button("Add Component").clicked() {
                ecs_editor.add_component("New Component".to_string());
            }
            
            if ui.button("Add System").clicked() {
                ecs_editor.add_system("New System".to_string());
            }
        });
        
        ui.separator();
        
        // Split view: Graph visualization on the left, properties on the right
        ui.horizontal(|ui| {
            // Graph visualization (left side)
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.heading("Graph Visualization");
                    ui.label("Visualization would be displayed here");
                    
                    // Display node counts
                    let entity_count = ecs_editor.graph.nodes.values()
                        .filter(|n| n.labels.contains(&"entity".to_string()))
                        .count();
                    
                    let component_count = ecs_editor.graph.nodes.values()
                        .filter(|n| n.labels.contains(&"component".to_string()))
                        .count();
                    
                    let system_count = ecs_editor.graph.nodes.values()
                        .filter(|n| n.labels.contains(&"system".to_string()))
                        .count();
                    
                    ui.label(format!("Entities: {}", entity_count));
                    ui.label(format!("Components: {}", component_count));
                    ui.label(format!("Systems: {}", system_count));
                    ui.label(format!("Total Edges: {}", ecs_editor.graph.edges.len()));
                });
            });
            
            // Properties panel (right side)
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.heading("Entity List");
                    // Collect entity IDs first to avoid borrowing conflict
                    let entity_nodes: Vec<(Uuid, &str)> = ecs_editor.graph.nodes.iter()
                        .filter(|(_, node)| node.labels.contains(&"entity".to_string()))
                        .map(|(id, node)| (*id, node.name.as_str()))
                        .collect();
                    
                    // Store ID to select after the loop if needed
                    let mut entity_to_select = None;
                    
                    for (id, name) in entity_nodes {
                        if ui.selectable_label(ecs_editor.selected_node == Some(id), name).clicked() {
                            entity_to_select = Some(id);
                        }
                    }
                    
                    // Update selection outside the iteration
                    if let Some(id) = entity_to_select {
                        ecs_editor.selected_node = Some(id);
                    }
                });
                
                ui.group(|ui| {
                    ui.heading("Component List");
                    // Collect component IDs first to avoid borrowing conflict
                    let component_nodes: Vec<(Uuid, &str)> = ecs_editor.graph.nodes.iter()
                        .filter(|(_, node)| node.labels.contains(&"component".to_string()))
                        .map(|(id, node)| (*id, node.name.as_str()))
                        .collect();
                    
                    // Store ID to select after the loop if needed
                    let mut component_to_select = None;
                    
                    for (id, name) in component_nodes {
                        if ui.selectable_label(ecs_editor.selected_node == Some(id), name).clicked() {
                            component_to_select = Some(id);
                        }
                    }
                    
                    // Update selection outside the iteration
                    if let Some(id) = component_to_select {
                        ecs_editor.selected_node = Some(id);
                    }
                });
                
                ui.group(|ui| {
                    ui.heading("System List");
                    // Collect system IDs first to avoid borrowing conflict
                    let system_nodes: Vec<(Uuid, &str)> = ecs_editor.graph.nodes.iter()
                        .filter(|(_, node)| node.labels.contains(&"system".to_string()))
                        .map(|(id, node)| (*id, node.name.as_str()))
                        .collect();
                    
                    // Store ID to select after the loop if needed
                    let mut system_to_select = None;
                    
                    for (id, name) in system_nodes {
                        if ui.selectable_label(ecs_editor.selected_node == Some(id), name).clicked() {
                            system_to_select = Some(id);
                        }
                    }
                    
                    // Update selection outside the iteration
                    if let Some(id) = system_to_select {
                        ecs_editor.selected_node = Some(id);
                    }
                });
            });
        });
    });
    
    // Save window position for next frame
    if let Some(inner_response) = response {
        ecs_editor.window_pos = Some(inner_response.response.rect.min);
    }
}

// System to handle toggling the ECS Editor visibility
fn handle_ecs_editor_visibility(
    mut events: EventReader<ToggleEcsEditorEvent>,
    mut ecs_editor: ResMut<EcsEditor>,
) {
    for event in events.read() {
        ecs_editor.visible = event.0;
        
        // If we're making it visible and it has no content, create default graph
        if ecs_editor.visible && ecs_editor.graph.nodes.is_empty() {
            ecs_editor.create_default_graph();
        }
    }
} 