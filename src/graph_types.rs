use bevy::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Base component for any graph entity in the world
#[derive(Component)]
pub struct Graph {
    pub name: String,
    pub description: String,
    pub graph_type: GraphType,
}

/// Enum defining the different types of graphs available
#[derive(Debug, Clone, PartialEq)]
pub enum GraphType {
    Workflow,
    DomainModel,
    Network,
    Tree,
    Star,
    Cycle,
    Grid,
    Custom(String),
}

/// Component for workflow graphs
#[derive(Component)]
pub struct WorkflowGraph {
    pub start_nodes: Vec<Uuid>,
    pub end_nodes: Vec<Uuid>,
    pub decision_nodes: Vec<Uuid>,
}

/// Component for domain-driven design graphs
#[derive(Component)]
pub struct DomainModelGraph {
    pub bounded_contexts: Vec<Uuid>,
    pub aggregates: Vec<Uuid>,
    pub entities: Vec<Uuid>,
    pub value_objects: Vec<Uuid>,
}

/// Component for network topology graphs
#[derive(Component)]
pub struct NetworkGraph {
    pub servers: Vec<Uuid>,
    pub clients: Vec<Uuid>,
    pub routers: Vec<Uuid>,
}

/// Component for hierarchical tree graphs
#[derive(Component)]
pub struct TreeGraph {
    pub root: Option<Uuid>,
    pub branch_factor: usize,
    pub max_depth: usize,
}

/// Component for star topology graphs
#[derive(Component)]
pub struct StarGraph {
    pub center: Option<Uuid>,
    pub points: Vec<Uuid>,
}

/// Generic component for custom graph types
#[derive(Component)]
pub struct CustomGraph {
    pub graph_type_name: String,
    pub properties: HashMap<String, String>,
}

/// Component that defines how nodes should be rendered for a specific graph type
#[derive(Component)]
pub struct GraphRenderingRules {
    pub node_rules: HashMap<String, NodeRenderRule>,
    pub edge_rules: HashMap<String, EdgeRenderRule>,
    pub default_node_rule: NodeRenderRule,
    pub default_edge_rule: EdgeRenderRule,
}

/// Rendering rules for different node types
#[derive(Debug, Clone)]
pub struct NodeRenderRule {
    pub shape: NodeShape,
    pub color: Color,
    pub size: f32,
    pub label_color: Color,
    pub label_size: f32,
}

/// Rendering rules for different edge types
#[derive(Debug, Clone)]
pub struct EdgeRenderRule {
    pub color: Color,
    pub thickness: f32,
    pub style: EdgeStyle,
}

/// Available node shapes
#[derive(Debug, Clone)]
pub enum NodeShape {
    Sphere,
    Cube,
    Cylinder,
    Icosphere,
    Capsule,
}

/// Available edge styles
#[derive(Debug, Clone)]
pub enum EdgeStyle {
    Solid,
    Dashed,
    Dotted,
    Arrow,
}

/// Component for cross-graph connections
#[derive(Component)]
pub struct CrossGraphEdge {
    pub source_graph: Entity,
    pub target_graph: Entity,
    pub source_node: Uuid,
    pub target_node: Uuid,
    pub relationship_type: String,
}

/// Bundle for creating a complete graph entity
#[derive(Bundle)]
pub struct GraphBundle {
    pub graph: Graph,
    pub rendering_rules: GraphRenderingRules,
    pub name: Name,
}

impl GraphBundle {
    pub fn workflow(name: String, description: String) -> (Self, WorkflowGraph) {
        let bundle = Self {
            graph: Graph {
                name: name.clone(),
                description,
                graph_type: GraphType::Workflow,
            },
            rendering_rules: GraphRenderingRules::workflow_default(),
            name: Name::new(format!("Workflow: {}", name)),
        };

        let workflow = WorkflowGraph {
            start_nodes: Vec::new(),
            end_nodes: Vec::new(),
            decision_nodes: Vec::new(),
        };

        (bundle, workflow)
    }

    pub fn domain_model(name: String, description: String) -> (Self, DomainModelGraph) {
        let bundle = Self {
            graph: Graph {
                name: name.clone(),
                description,
                graph_type: GraphType::DomainModel,
            },
            rendering_rules: GraphRenderingRules::domain_model_default(),
            name: Name::new(format!("Domain Model: {}", name)),
        };

        let domain_model = DomainModelGraph {
            bounded_contexts: Vec::new(),
            aggregates: Vec::new(),
            entities: Vec::new(),
            value_objects: Vec::new(),
        };

        (bundle, domain_model)
    }

    pub fn new_star(name: String, description: String, _points: usize) -> (Self, StarGraph) {
        let bundle = Self {
            graph: Graph {
                name: name.clone(),
                description,
                graph_type: GraphType::Star,
            },
            rendering_rules: GraphRenderingRules::star_default(),
            name: Name::new(format!("Star Graph: {}", name)),
        };

        let star = StarGraph {
            center: None,
            points: Vec::new(),
        };

        (bundle, star)
    }
}

impl GraphRenderingRules {
    pub fn workflow_default() -> Self {
        let mut node_rules = HashMap::new();

        node_rules.insert(
            "start".to_string(),
            NodeRenderRule {
                shape: NodeShape::Sphere,
                color: Color::srgb(0.2, 0.8, 0.2),
                size: 0.4,
                label_color: Color::WHITE,
                label_size: 1.0,
            },
        );

        node_rules.insert(
            "end".to_string(),
            NodeRenderRule {
                shape: NodeShape::Sphere,
                color: Color::srgb(0.8, 0.2, 0.2),
                size: 0.4,
                label_color: Color::WHITE,
                label_size: 1.0,
            },
        );

        node_rules.insert(
            "decision".to_string(),
            NodeRenderRule {
                shape: NodeShape::Cube,
                color: Color::srgb(0.8, 0.8, 0.2),
                size: 0.3,
                label_color: Color::BLACK,
                label_size: 0.8,
            },
        );

        node_rules.insert(
            "process".to_string(),
            NodeRenderRule {
                shape: NodeShape::Cylinder,
                color: Color::srgb(0.2, 0.4, 0.8),
                size: 0.35,
                label_color: Color::WHITE,
                label_size: 0.9,
            },
        );

        let mut edge_rules = HashMap::new();
        edge_rules.insert(
            "flow".to_string(),
            EdgeRenderRule {
                color: Color::srgb(0.7, 0.7, 0.7),
                thickness: 0.05,
                style: EdgeStyle::Arrow,
            },
        );

        Self {
            node_rules,
            edge_rules,
            default_node_rule: NodeRenderRule {
                shape: NodeShape::Sphere,
                color: Color::srgb(0.5, 0.5, 0.8),
                size: 0.3,
                label_color: Color::WHITE,
                label_size: 1.0,
            },
            default_edge_rule: EdgeRenderRule {
                color: Color::srgb(0.6, 0.6, 0.6),
                thickness: 0.03,
                style: EdgeStyle::Solid,
            },
        }
    }

    pub fn domain_model_default() -> Self {
        let mut node_rules = HashMap::new();

        node_rules.insert(
            "BoundedContext".to_string(),
            NodeRenderRule {
                shape: NodeShape::Cube,
                color: Color::srgb(0.8, 0.4, 0.2),
                size: 0.6,
                label_color: Color::WHITE,
                label_size: 1.2,
            },
        );

        node_rules.insert(
            "Aggregate".to_string(),
            NodeRenderRule {
                shape: NodeShape::Icosphere,
                color: Color::srgb(0.2, 0.6, 0.8),
                size: 0.4,
                label_color: Color::WHITE,
                label_size: 1.0,
            },
        );

        node_rules.insert(
            "Entity".to_string(),
            NodeRenderRule {
                shape: NodeShape::Sphere,
                color: Color::srgb(0.4, 0.8, 0.4),
                size: 0.3,
                label_color: Color::BLACK,
                label_size: 0.9,
            },
        );

        node_rules.insert(
            "ValueObject".to_string(),
            NodeRenderRule {
                shape: NodeShape::Capsule,
                color: Color::srgb(0.8, 0.6, 0.8),
                size: 0.25,
                label_color: Color::BLACK,
                label_size: 0.8,
            },
        );

        let mut edge_rules = HashMap::new();
        edge_rules.insert(
            "contains".to_string(),
            EdgeRenderRule {
                color: Color::srgb(0.2, 0.8, 0.2),
                thickness: 0.08,
                style: EdgeStyle::Solid,
            },
        );

        edge_rules.insert(
            "references".to_string(),
            EdgeRenderRule {
                color: Color::srgb(0.8, 0.2, 0.8),
                thickness: 0.04,
                style: EdgeStyle::Dashed,
            },
        );

        Self {
            node_rules,
            edge_rules,
            default_node_rule: NodeRenderRule {
                shape: NodeShape::Sphere,
                color: Color::srgb(0.7, 0.7, 0.7),
                size: 0.3,
                label_color: Color::WHITE,
                label_size: 1.0,
            },
            default_edge_rule: EdgeRenderRule {
                color: Color::srgb(0.5, 0.5, 0.5),
                thickness: 0.03,
                style: EdgeStyle::Solid,
            },
        }
    }

    pub fn star_default() -> Self {
        let mut node_rules = HashMap::new();

        node_rules.insert(
            "center".to_string(),
            NodeRenderRule {
                shape: NodeShape::Icosphere,
                color: Color::srgb(0.9, 0.7, 0.2),
                size: 0.5,
                label_color: Color::BLACK,
                label_size: 1.2,
            },
        );

        node_rules.insert(
            "point".to_string(),
            NodeRenderRule {
                shape: NodeShape::Sphere,
                color: Color::srgb(0.2, 0.7, 0.9),
                size: 0.3,
                label_color: Color::WHITE,
                label_size: 0.9,
            },
        );

        let mut edge_rules = HashMap::new();
        edge_rules.insert(
            "spoke".to_string(),
            EdgeRenderRule {
                color: Color::srgb(0.8, 0.8, 0.8),
                thickness: 0.04,
                style: EdgeStyle::Solid,
            },
        );

        Self {
            node_rules,
            edge_rules,
            default_node_rule: NodeRenderRule {
                shape: NodeShape::Sphere,
                color: Color::srgb(0.6, 0.6, 0.8),
                size: 0.3,
                label_color: Color::WHITE,
                label_size: 1.0,
            },
            default_edge_rule: EdgeRenderRule {
                color: Color::srgb(0.7, 0.7, 0.7),
                thickness: 0.03,
                style: EdgeStyle::Solid,
            },
        }
    }
}
