use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use std::collections::HashMap;
use uuid::Uuid;

use crate::dashboard_ui::ToggleDddEditorEvent;
use crate::graph::{AlchemistGraph, GraphEdge, GraphNode};
use crate::graph_patterns::{GraphPattern, PatternCatalog, generate_pattern};

// Events for the DDD editor
#[derive(Event)]
pub struct UpdateDDDGraphEvent(pub AlchemistGraph);

#[derive(Event)]
pub struct CreateDDDPatternEvent {
    pub pattern: GraphPattern,
}

// Interaction state for the UI
#[derive(PartialEq)]
pub enum DddInteractionState {
    Viewing,
    Editing,
    Creating,
}

// DDD Editor is the main editor component for Domain-Driven Design
#[derive(Resource)]
pub struct DddEditor {
    pub graph: AlchemistGraph,
    pub selected_node: Option<Uuid>,
    pub selected_edge: Option<Uuid>,
    pub interaction_state: DddInteractionState,
    pub node_positions: HashMap<Uuid, Vec3>,
    pub pattern_catalog: PatternCatalog,
    pub bounded_contexts: Vec<String>,
    pub aggregates: Vec<String>,
    pub entities: Vec<String>,
    pub value_objects: Vec<String>,
    pub visible: bool,
    pub window_pos: Option<egui::Pos2>,
}

impl Default for DddEditor {
    fn default() -> Self {
        let mut catalog = PatternCatalog::new();

        // Add DDD-specific patterns to catalog
        catalog.add_pattern("bounded_context", GraphPattern::Complete { nodes: 3 });
        catalog.add_pattern("aggregate", GraphPattern::Star { points: 5 });
        catalog.add_pattern(
            "entity",
            GraphPattern::Tree {
                branch_factor: 2,
                depth: 2,
            },
        );
        catalog.add_pattern("value_object", GraphPattern::Complete { nodes: 4 });

        Self {
            graph: AlchemistGraph::new(),
            selected_node: None,
            selected_edge: None,
            interaction_state: DddInteractionState::Viewing,
            node_positions: HashMap::new(),
            pattern_catalog: catalog,
            bounded_contexts: vec![
                "Core Domain".to_string(),
                "Supporting".to_string(),
                "Generic".to_string(),
            ],
            aggregates: vec![
                "User".to_string(),
                "Order".to_string(),
                "Product".to_string(),
            ],
            entities: vec![
                "Customer".to_string(),
                "Order".to_string(),
                "LineItem".to_string(),
                "Product".to_string(),
            ],
            value_objects: vec![
                "Address".to_string(),
                "Money".to_string(),
                "Email".to_string(),
                "PhoneNumber".to_string(),
            ],
            visible: false,
            window_pos: None,
        }
    }
}

impl DddEditor {
    pub fn create_default_graph(&mut self) {
        let pattern = GraphPattern::Tree {
            branch_factor: 2,
            depth: 2,
        };
        self.graph = generate_pattern(pattern);

        // Modify the graph to represent a DDD structure
        self.transform_to_ddd_structure();
    }

    fn transform_to_ddd_structure(&mut self) {
        // Set node labels for DDD concepts
        for (_id, node) in self.graph.nodes.iter_mut() {
            // First node is a bounded context
            if node.name == "Root" {
                node.name = "Core Domain".to_string();
                node.labels.push("BoundedContext".to_string());
                node.properties
                    .insert("type".to_string(), "Domain".to_string());
            }
            // Second level nodes are aggregates
            else if node.name.starts_with("Child") {
                node.name = format!("Aggregate_{}", node.name.replace("Child", ""));
                node.labels.push("Aggregate".to_string());
                node.properties
                    .insert("type".to_string(), "Aggregate".to_string());
            }
            // Leaf nodes are entities or value objects
            else {
                if rand::random::<bool>() {
                    node.name = format!("Entity_{}", node.name.replace("Grandchild", ""));
                    node.labels.push("Entity".to_string());
                    node.properties
                        .insert("type".to_string(), "Entity".to_string());
                } else {
                    node.name = format!("ValueObject_{}", node.name.replace("Grandchild", ""));
                    node.labels.push("ValueObject".to_string());
                    node.properties
                        .insert("type".to_string(), "ValueObject".to_string());
                }
            }
        }

        // Set edge labels for DDD relationships
        for (_, edge) in self.graph.edges.iter_mut() {
            if let Some(source_node) = self.graph.nodes.get(&edge.source) {
                if let Some(target_node) = self.graph.nodes.get(&edge.target) {
                    // Bounded Context to Aggregate
                    if source_node.labels.contains(&"BoundedContext".to_string())
                        && target_node.labels.contains(&"Aggregate".to_string())
                    {
                        edge.labels.push("Contains".to_string());
                    }
                    // Aggregate to Entity
                    else if source_node.labels.contains(&"Aggregate".to_string())
                        && target_node.labels.contains(&"Entity".to_string())
                    {
                        edge.labels.push("Contains".to_string());
                    }
                    // Aggregate to ValueObject
                    else if source_node.labels.contains(&"Aggregate".to_string())
                        && target_node.labels.contains(&"ValueObject".to_string())
                    {
                        edge.labels.push("Uses".to_string());
                    }
                    // Entity to ValueObject
                    else if source_node.labels.contains(&"Entity".to_string())
                        && target_node.labels.contains(&"ValueObject".to_string())
                    {
                        edge.labels.push("Has".to_string());
                    }
                    // Entity to Entity
                    else if source_node.labels.contains(&"Entity".to_string())
                        && target_node.labels.contains(&"Entity".to_string())
                    {
                        edge.labels.push("References".to_string());
                    }
                }
            }
        }
    }

    pub fn add_bounded_context(&mut self, name: String) {
        let id = Uuid::new_v4();
        let node = GraphNode {
            id,
            name,
            radius: 1.0,
            properties: HashMap::new(),
            labels: vec!["BoundedContext".to_string()],
        };
        self.graph.nodes.insert(id, node);
    }

    pub fn add_aggregate(&mut self, name: String, context_id: Uuid) {
        let id = Uuid::new_v4();
        let node = GraphNode {
            id,
            name,
            radius: 0.8,
            properties: HashMap::new(),
            labels: vec!["Aggregate".to_string()],
        };
        self.graph.nodes.insert(id, node);

        // Connect to bounded context
        let edge_id = Uuid::new_v4();
        let edge = GraphEdge {
            id: edge_id,
            source: context_id,
            target: id,
            weight: 1.0,
            properties: HashMap::new(),
            labels: vec!["Contains".to_string()],
        };
        self.graph.edges.insert(edge_id, edge);
    }

    pub fn add_entity(&mut self, name: String, aggregate_id: Uuid) {
        let id = Uuid::new_v4();
        let node = GraphNode {
            id,
            name,
            radius: 0.6,
            properties: HashMap::new(),
            labels: vec!["Entity".to_string()],
        };
        self.graph.nodes.insert(id, node);

        // Connect to aggregate
        let edge_id = Uuid::new_v4();
        let edge = GraphEdge {
            id: edge_id,
            source: aggregate_id,
            target: id,
            weight: 1.0,
            properties: HashMap::new(),
            labels: vec!["Contains".to_string()],
        };
        self.graph.edges.insert(edge_id, edge);
    }

    pub fn add_value_object(&mut self, name: String, parent_id: Uuid) {
        let id = Uuid::new_v4();
        let node = GraphNode {
            id,
            name,
            radius: 0.5,
            properties: HashMap::new(),
            labels: vec!["ValueObject".to_string()],
        };
        self.graph.nodes.insert(id, node);

        // Connect to parent (aggregate or entity)
        let edge_id = Uuid::new_v4();
        let edge = GraphEdge {
            id: edge_id,
            source: parent_id,
            target: id,
            weight: 1.0,
            properties: HashMap::new(),
            labels: vec!["Has".to_string()],
        };
        self.graph.edges.insert(edge_id, edge);
    }
}

// Plugin for the DDD Editor
#[derive(Default)]
pub struct DddEditorPlugin;

impl Plugin for DddEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DddEditor>()
            .add_event::<UpdateDDDGraphEvent>()
            .add_event::<CreateDDDPatternEvent>()
            .add_systems(Startup, setup_ddd_editor)
            .add_systems(
                Update,
                (
                    handle_ddd_editor_ui,
                    handle_ddd_pattern_creation,
                    handle_ddd_graph_update,
                    handle_ddd_editor_visibility,
                ),
            );
    }
}

fn setup_ddd_editor(mut ddd_editor: ResMut<DddEditor>) {
    ddd_editor.create_default_graph();
}

fn handle_ddd_editor_ui(
    mut contexts: EguiContexts,
    mut ddd_editor: ResMut<DddEditor>,
    mut pattern_event: EventWriter<CreateDDDPatternEvent>,
) {
    if !ddd_editor.visible {
        return;
    }

    let mut window = egui::Window::new("DDD Editor");

    if let Some(pos) = ddd_editor.window_pos {
        window = window.default_pos(pos);
    }

    let response = window.show(contexts.ctx_mut(), |ui| {
        ui.heading("Domain-Driven Design Editor");

        // Tabs for different DDD concepts
        ui.horizontal(|ui| {
            if ui.button("Bounded Contexts").clicked() {
                // Show bounded context UI
            }

            if ui.button("Aggregates").clicked() {
                // Show aggregates UI
            }

            if ui.button("Entities").clicked() {
                // Show entities UI
            }

            if ui.button("Value Objects").clicked() {
                // Show value objects UI
            }
        });

        ui.separator();

        // Pattern selection
        ui.heading("DDD Patterns");
        ui.horizontal(|ui| {
            // Patterns menu
            ui.menu_button("Patterns", |ui| {
                if ui.button("Aggregate").clicked() {
                    pattern_event.write(CreateDDDPatternEvent {
                        pattern: GraphPattern::Star { points: 5 },
                    });
                }

                if ui.button("Bounded Context").clicked() {
                    pattern_event.write(CreateDDDPatternEvent {
                        pattern: GraphPattern::Complete { nodes: 3 },
                    });
                }

                if ui.button("Entity-ValueObject").clicked() {
                    pattern_event.write(CreateDDDPatternEvent {
                        pattern: GraphPattern::Tree {
                            branch_factor: 2,
                            depth: 2,
                        },
                    });
                }
            });
        });

        // Graph statistics
        ui.separator();
        ui.label(format!("Nodes: {}", ddd_editor.graph.nodes.len()));
        ui.label(format!("Edges: {}", ddd_editor.graph.edges.len()));

        if ui.button("Create Default Graph").clicked() {
            ddd_editor.create_default_graph();
        }
    });

    // Save window position for next frame
    if let Some(inner_response) = response {
        ddd_editor.window_pos = Some(inner_response.response.rect.min);
    }
}

fn handle_ddd_pattern_creation(
    mut events: EventReader<CreateDDDPatternEvent>,
    mut ddd_editor: ResMut<DddEditor>,
) {
    for event in events.read() {
        let new_graph = generate_pattern(event.pattern.clone());
        ddd_editor.graph = new_graph;
        ddd_editor.transform_to_ddd_structure();
    }
}

fn handle_ddd_graph_update(
    mut events: EventReader<UpdateDDDGraphEvent>,
    mut ddd_editor: ResMut<DddEditor>,
) {
    for event in events.read() {
        ddd_editor.graph = event.0.clone();
    }
}

// System to handle toggling the DDD Editor visibility
fn handle_ddd_editor_visibility(
    mut events: EventReader<ToggleDddEditorEvent>,
    mut ddd_editor: ResMut<DddEditor>,
) {
    for event in events.read() {
        ddd_editor.visible = event.0;

        // If we're making it visible and it has no content, create default graph
        if ddd_editor.visible && ddd_editor.graph.nodes.is_empty() {
            ddd_editor.create_default_graph();
        }
    }
}
