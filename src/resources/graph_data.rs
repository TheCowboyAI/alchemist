use bevy::prelude::*;
use petgraph::Direction;
use petgraph::graph::{DiGraph, EdgeIndex as PetEdgeIndex, NodeIndex as PetNodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use uuid::Uuid;

use crate::components::{DomainEdgeType, DomainNodeType};

/// The actual graph data structure using petgraph
#[derive(Resource)]
pub struct GraphData {
    /// The petgraph directed graph
    pub graph: DiGraph<NodeData, EdgeData>,
    /// Map from UUID to petgraph NodeIndex
    pub uuid_to_node: HashMap<Uuid, PetNodeIndex>,
    /// Map from petgraph NodeIndex to ECS Entity
    pub node_to_entity: HashMap<PetNodeIndex, Entity>,
    /// Map from petgraph EdgeIndex to ECS Entity
    pub edge_to_entity: HashMap<PetEdgeIndex, Entity>,
}

#[derive(Debug, Clone)]
pub struct NodeData {
    pub id: Uuid,
    pub name: String,
    pub domain_type: DomainNodeType,
    pub position: Vec3,
    pub labels: Vec<String>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct EdgeData {
    pub id: Uuid,
    pub edge_type: DomainEdgeType,
    pub labels: Vec<String>,
    pub properties: HashMap<String, String>,
}

impl Default for GraphData {
    fn default() -> Self {
        Self {
            graph: DiGraph::new(),
            uuid_to_node: HashMap::new(),
            node_to_entity: HashMap::new(),
            edge_to_entity: HashMap::new(),
        }
    }
}

// Note: Implementation methods would be moved to systems
// This resource should primarily hold data, with logic in systems
