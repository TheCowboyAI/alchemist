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

/// Struct for edge visual properties (not a component since edges aren't entities)
#[derive(Debug, Clone)]
pub struct EdgeVisual {
    pub width: f32,
    pub color: Color,
}

impl Default for EdgeVisual {
    fn default() -> Self {
        Self {
            width: 2.0,
            color: Color::srgb(0.255, 0.412, 0.882), // Royal blue
        }
    }
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

/// Component to track outgoing edges from a node
/// `id` is the UUID of the edge (matches GraphData and events)
#[derive(Component, Debug, Clone)]
pub struct OutgoingEdge {
    /// The UUID of the edge (unique identifier)
    pub id: Uuid,
    /// The ECS entity of the target node
    pub target: Entity,
    pub edge_type: DomainEdgeType,
    pub labels: Vec<String>,
    pub properties: HashMap<String, String>,
}

/// Component to track multiple outgoing edges from a node
#[derive(Component, Debug, Clone, Default)]
pub struct OutgoingEdges {
    pub edges: Vec<OutgoingEdge>,
}
