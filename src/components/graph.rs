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

/// Component to track which subgraph a node belongs to
#[derive(Component, Debug, Clone)]
pub struct SubgraphMember {
    pub subgraph_id: Uuid,
}

/// Component to track outgoing edges from a node
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
