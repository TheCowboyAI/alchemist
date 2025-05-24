use bevy::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Core node component
#[derive(Component, Debug, Clone)]
pub struct GraphNode {
    pub id: Uuid,
    pub domain_type: DomainNodeType,
    pub name: String,
    pub labels: Vec<String>,
    pub properties: HashMap<String, String>,
}

/// Core edge component
#[derive(Component, Debug, Clone)]
pub struct GraphEdge {
    pub id: Uuid,
    pub source: Entity,
    pub target: Entity,
    pub edge_type: DomainEdgeType,
    pub labels: Vec<String>,
    pub properties: HashMap<String, String>,
}

/// Component to store node position in graph space
#[derive(Component, Debug, Clone, Copy)]
pub struct GraphPosition(pub Vec3);

/// Component to mark a node as selected
#[derive(Component)]
pub struct Selected;

/// Component to mark a node as hovered
#[derive(Component)]
pub struct Hovered;

/// Component for visual representation of nodes
#[derive(Component)]
pub struct NodeVisual {
    pub base_color: Color,
    pub current_color: Color,
}

/// Component for visual representation of edges
#[derive(Component)]
pub struct EdgeVisual {
    pub width: f32,
    pub color: Color,
}

/// Component to track which subgraph a node belongs to
#[derive(Component, Debug, Clone)]
pub struct SubgraphMember {
    pub subgraph_id: Uuid,
}

/// Domain node types for business logic
#[derive(Debug, Clone, PartialEq)]
pub enum DomainNodeType {
    Process,
    Decision,
    Event,
    Storage,
    Interface,
    Custom(String),
}

/// Domain edge types for relationships
#[derive(Debug, Clone, PartialEq)]
pub enum DomainEdgeType {
    DataFlow,
    ControlFlow,
    Dependency,
    Association,
    Custom(String),
}

/// Resource to track the overall graph state
#[derive(Resource, Default)]
pub struct GraphState {
    pub node_count: usize,
    pub edge_count: usize,
    pub selected_nodes: Vec<Entity>,
    pub selected_edges: Vec<Entity>,
    pub hovered_entity: Option<Entity>,
}

/// Resource for graph metadata
#[derive(Resource, Default)]
pub struct GraphMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub domain: String,
}

/// Component bundle for spawning graph nodes
#[derive(Bundle)]
pub struct GraphNodeBundle {
    pub node: GraphNode,
    pub position: GraphPosition,
    pub visual: NodeVisual,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl GraphNodeBundle {
    pub fn new(
        id: Uuid,
        domain_type: DomainNodeType,
        position: Vec3,
        color: Color,
        name: String,
        labels: Vec<String>,
        properties: HashMap<String, String>,
    ) -> Self {
        Self {
            node: GraphNode {
                id,
                domain_type,
                name,
                labels,
                properties,
            },
            position: GraphPosition(position),
            visual: NodeVisual {
                base_color: color,
                current_color: color,
            },
            transform: Transform::from_translation(position),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            view_visibility: ViewVisibility::default(),
            inherited_visibility: InheritedVisibility::default(),
        }
    }
}

/// Component bundle for spawning graph edges
#[derive(Bundle)]
pub struct GraphEdgeBundle {
    pub edge: GraphEdge,
    pub visual: EdgeVisual,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl GraphEdgeBundle {
    pub fn new(id: Uuid, source: Entity, target: Entity, edge_type: DomainEdgeType) -> Self {
        Self {
            edge: GraphEdge {
                id,
                source,
                target,
                edge_type,
                labels: Vec::new(),
                properties: HashMap::new(),
            },
            visual: EdgeVisual {
                width: 2.0,
                color: Color::srgb(0.6, 0.6, 0.6),
            },
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}
