use super::components::{DomainEdgeType, DomainNodeType};
use bevy::prelude::*;
use petgraph::Direction;
use petgraph::graph::{DiGraph, EdgeIndex as PetEdgeIndex, NodeIndex as PetNodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use uuid::Uuid;

/// The actual graph data structure using petgraph
#[derive(Resource)]
pub struct GraphData {
    /// The petgraph directed graph
    pub(super) graph: DiGraph<NodeData, EdgeData>,
    /// Map from UUID to petgraph NodeIndex
    pub(super) uuid_to_node: HashMap<Uuid, PetNodeIndex>,
    /// Map from petgraph NodeIndex to ECS Entity
    pub(super) node_to_entity: HashMap<PetNodeIndex, Entity>,
    /// Map from petgraph EdgeIndex to ECS Entity
    pub(super) edge_to_entity: HashMap<PetEdgeIndex, Entity>,
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

impl GraphData {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, data: NodeData) -> PetNodeIndex {
        let id = data.id;
        let idx = self.graph.add_node(data);
        self.uuid_to_node.insert(id, idx);
        idx
    }

    /// Add an edge between two nodes
    pub fn add_edge(
        &mut self,
        source_id: Uuid,
        target_id: Uuid,
        data: EdgeData,
    ) -> Result<PetEdgeIndex, String> {
        let source = self
            .uuid_to_node
            .get(&source_id)
            .ok_or_else(|| format!("Source node {} not found", source_id))?;
        let target = self
            .uuid_to_node
            .get(&target_id)
            .ok_or_else(|| format!("Target node {} not found", target_id))?;

        Ok(self.graph.add_edge(*source, *target, data))
    }

    /// Get node data by UUID
    pub fn get_node(&self, id: Uuid) -> Option<&NodeData> {
        self.uuid_to_node
            .get(&id)
            .and_then(|idx| self.graph.node_weight(*idx))
    }

    /// Get all edges for a node
    pub fn get_edges(
        &self,
        id: Uuid,
        direction: Direction,
    ) -> Vec<(PetEdgeIndex, &EdgeData, PetNodeIndex)> {
        if let Some(&node_idx) = self.uuid_to_node.get(&id) {
            self.graph
                .edges_directed(node_idx, direction)
                .map(|edge| (edge.id(), edge.weight(), edge.target()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Associate a petgraph node with an ECS entity
    pub fn set_node_entity(&mut self, node_idx: PetNodeIndex, entity: Entity) {
        self.node_to_entity.insert(node_idx, entity);
    }

    /// Associate a petgraph edge with an ECS entity
    pub fn set_edge_entity(&mut self, edge_idx: PetEdgeIndex, entity: Entity) {
        self.edge_to_entity.insert(edge_idx, entity);
    }

    /// Get the ECS entity for a node
    pub fn get_node_entity(&self, node_idx: PetNodeIndex) -> Option<Entity> {
        self.node_to_entity.get(&node_idx).copied()
    }

    /// Get the ECS entity for an edge
    pub fn get_edge_entity(&self, edge_idx: PetEdgeIndex) -> Option<Entity> {
        self.edge_to_entity.get(&edge_idx).copied()
    }

    /// Get source and target entities for an edge
    pub fn get_edge_entities(&self, edge_idx: PetEdgeIndex) -> Option<(Entity, Entity)> {
        let (source_idx, target_idx) = self.graph.edge_endpoints(edge_idx)?;
        let source_entity = self.get_node_entity(source_idx)?;
        let target_entity = self.get_node_entity(target_idx)?;
        Some((source_entity, target_entity))
    }

    /// Iterate over all nodes
    pub fn nodes(&self) -> impl Iterator<Item = (PetNodeIndex, &NodeData)> {
        self.graph
            .node_indices()
            .filter_map(move |idx| self.graph.node_weight(idx).map(|data| (idx, data)))
    }

    /// Iterate over all edges
    pub fn edges(
        &self,
    ) -> impl Iterator<Item = (PetEdgeIndex, &EdgeData, PetNodeIndex, PetNodeIndex)> {
        self.graph.edge_indices().filter_map(move |idx| {
            let (source, target) = self.graph.edge_endpoints(idx)?;
            let data = self.graph.edge_weight(idx)?;
            Some((idx, data, source, target))
        })
    }

    /// Get total node count
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Get total edge count
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}
