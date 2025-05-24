use bevy::prelude::*;
use daggy::{Dag, EdgeIndex, NodeIndex, Walker};
use petgraph::graph::NodeIndex as PetNodeIndex;
use petgraph::visit::{IntoEdgeReferences, IntoNodeIdentifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Content identifier for Merkle DAG nodes
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Cid(String);

/// Merkle DAG node data with cryptographic properties
#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct MerkleNode {
    pub cid: Cid,
    pub links: Vec<Cid>,
    pub position: Vec3,
    pub render_state: NodeRenderState,
    pub metadata: HashMap<String, String>,
    pub uuid: Uuid,
}

/// Visual state for nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRenderState {
    pub color: [f32; 4],
    pub size: f32,
    pub visible: bool,
}

/// Merkle edge with cryptographic proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleEdge {
    pub weight: f64,
    pub proof: Vec<u8>,
    pub thickness: f32,
    pub style: EdgeStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeStyle {
    Solid,
    Dashed,
    Dotted,
}

/// Resource holding the Merkle DAG
#[derive(Resource)]
pub struct MerkleDag {
    dag: Dag<MerkleNode, MerkleEdge>,
    cid_to_node: HashMap<Cid, NodeIndex>,
    node_to_entity: HashMap<NodeIndex, Entity>,
    subgraph_roots: HashMap<String, Vec<NodeIndex>>,
}

// Schema for serialization
#[derive(serde::Serialize, serde::Deserialize)]
struct DagSchema {
    nodes: Vec<(NodeIndex, MerkleNode)>,
    edges: Vec<(NodeIndex, NodeIndex, MerkleEdge)>,
}

impl Default for MerkleDag {
    fn default() -> Self {
        Self {
            dag: Dag::new(),
            cid_to_node: HashMap::new(),
            node_to_entity: HashMap::new(),
            subgraph_roots: HashMap::new(),
        }
    }
}

impl MerkleDag {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the Merkle DAG
    pub fn add_node(&mut self, node: MerkleNode) -> NodeIndex {
        let cid = node.cid.clone();
        let idx = self.dag.add_node(node);
        self.cid_to_node.insert(cid, idx);
        idx
    }

    /// Add an edge with Merkle proof
    pub fn add_edge(
        &mut self,
        source: NodeIndex,
        target: NodeIndex,
        edge: MerkleEdge,
    ) -> Result<EdgeIndex, daggy::WouldCycle<MerkleEdge>> {
        self.dag.add_edge(source, target, edge)
    }

    /// Validate Merkle path between two nodes
    pub fn validate_merkle_path(&self, start: NodeIndex, end: NodeIndex) -> Result<bool, String> {
        let mut visited = Vec::new();
        visited.push(start);

        // Use Daggy's walker for traversal - iterate directly
        for (edge_idx, node_idx) in self.dag.children(start).iter(&self.dag) {
            visited.push(node_idx);

            if node_idx == end {
                // Verify all proofs along the path
                return Ok(self.verify_path_proofs(&visited));
            }

            // Continue searching depth-first
            if let Some(mut path_node) = self.find_path_to(node_idx, end) {
                visited.extend(path_node);
                return Ok(self.verify_path_proofs(&visited));
            }
        }

        Err("No path found".to_string())
    }

    /// Find path using Daggy's recursive walker
    fn find_path_to(&self, start: NodeIndex, end: NodeIndex) -> Option<Vec<NodeIndex>> {
        let mut path = Vec::new();
        let mut stack = vec![start];
        let mut visited = std::collections::HashSet::new();

        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);
            path.push(current);

            if current == end {
                return Some(path);
            }

            // Add children to the stack - iterate directly
            for (_, child) in self.dag.children(current).iter(&self.dag) {
                if !visited.contains(&child) {
                    stack.push(child);
                }
            }
        }

        None
    }

    /// Verify cryptographic proofs along a path
    fn verify_path_proofs(&self, path: &[NodeIndex]) -> bool {
        for window in path.windows(2) {
            if let (Some(source), Some(target)) = (window.get(0), window.get(1)) {
                if let Some(edge_idx) = self.dag.find_edge(*source, *target) {
                    let edge = &self.dag[edge_idx];
                    if !self.verify_edge_proof(edge) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Verify a single edge proof (placeholder - implement actual crypto)
    fn verify_edge_proof(&self, edge: &MerkleEdge) -> bool {
        // TODO: Implement actual cryptographic verification
        !edge.proof.is_empty()
    }

    /// Get node by CID
    pub fn get_node_by_cid(&self, cid: &Cid) -> Option<&MerkleNode> {
        self.cid_to_node
            .get(cid)
            .and_then(|idx| self.dag.node_weight(*idx))
    }

    /// Associate a DAG node with an ECS entity
    pub fn set_node_entity(&mut self, node_idx: NodeIndex, entity: Entity) {
        self.node_to_entity.insert(node_idx, entity);
    }

    /// Get all nodes in BFS order from root
    pub fn bfs_from(&self, start: NodeIndex) -> Vec<NodeIndex> {
        let mut visited = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        let mut seen = std::collections::HashSet::new();

        queue.push_back(start);
        seen.insert(start);

        while let Some(current) = queue.pop_front() {
            visited.push(current);

            // Add all children to the queue - iterate directly
            for (_, child) in self.dag.children(current).iter(&self.dag) {
                if seen.insert(child) {
                    queue.push_back(child);
                }
            }
        }

        visited
    }

    /// Serialize the DAG for persistence
    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        let nodes: Vec<(NodeIndex, MerkleNode)> = self
            .dag
            .raw_nodes()
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| node.weight.clone().map(|w| (NodeIndex::new(idx), w)))
            .collect();

        let edges: Vec<(NodeIndex, NodeIndex, MerkleEdge)> = self
            .dag
            .raw_edges()
            .iter()
            .filter_map(|edge| {
                edge.weight
                    .clone()
                    .map(|w| (edge.source(), edge.target(), w))
            })
            .collect();

        let schema = DagSchema { nodes, edges };
        serde_json::to_string(&schema)
    }

    /// Get all ancestors of a node
    pub fn ancestors(&self, start: NodeIndex) -> Vec<NodeIndex> {
        let mut ancestors = Vec::new();

        // Iterate directly over parents
        for (_, parent) in self.dag.parents(start).iter(&self.dag) {
            ancestors.push(parent);
        }

        ancestors
    }

    /// Get all descendants of a node
    pub fn descendants(&self, start: NodeIndex) -> Vec<NodeIndex> {
        let mut descendants = Vec::new();
        let mut stack = vec![start];
        let mut visited = std::collections::HashSet::new();

        while let Some(current) = stack.pop() {
            if current != start && visited.insert(current) {
                descendants.push(current);
            }

            // Add all children to the stack - iterate directly
            for (_, child) in self.dag.children(current).iter(&self.dag) {
                if !visited.contains(&child) {
                    stack.push(child);
                }
            }
        }

        descendants
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        self.serialize()
    }

    pub fn to_graphml(&self) -> String {
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<graphml>\n");

        // Write nodes
        for (idx, node) in self.dag.raw_nodes().iter().enumerate() {
            if let Some(ref weight) = node.weight {
                xml.push_str(&format!("  <node id=\"{}\"/>\n", weight.cid.0));
            }
        }

        // Write edges
        for edge in self.dag.raw_edges().iter() {
            if let Some(ref weight) = edge.weight {
                let source_idx = edge.source();
                let target_idx = edge.target();

                if let (Some(source), Some(target)) = (
                    self.dag.node_weight(source_idx),
                    self.dag.node_weight(target_idx),
                ) {
                    xml.push_str(&format!(
                        "  <edge source=\"{}\" target=\"{}\"/>\n",
                        source.cid.0, target.cid.0
                    ));
                }
            }
        }

        xml.push_str("</graphml>");
        xml
    }

    /// Commit a subgraph and generate proof
    pub fn commit_subgraph(&mut self, name: String, root: NodeIndex) -> Result<Cid, String> {
        // Get all descendants
        let mut nodes = vec![root];
        nodes.extend(self.descendants(root));

        self.subgraph_roots.insert(name.clone(), nodes.clone());

        // TODO: Generate actual Merkle proof
        let proof_cid = Cid(format!(
            "Qm{}",
            uuid::Uuid::new_v4().to_string().replace("-", "")
        ));

        info!("Committed subgraph '{}' with {} nodes", name, nodes.len());

        Ok(proof_cid)
    }

    /// Convert the DAG to a serializable format
    pub fn to_schema(&self) -> DagSchema {
        let nodes: Vec<(NodeIndex, MerkleNode)> = self
            .dag
            .raw_nodes()
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| node.weight.clone().map(|w| (NodeIndex::new(idx), w)))
            .collect();

        let edges: Vec<(NodeIndex, NodeIndex, MerkleEdge)> = self
            .dag
            .raw_edges()
            .iter()
            .filter_map(|edge| {
                edge.weight
                    .clone()
                    .map(|w| (edge.source(), edge.target(), w))
            })
            .collect();

        DagSchema { nodes, edges }
    }
}

/// System to create ECS entities for DAG nodes
pub fn sync_dag_to_ecs(
    mut commands: Commands,
    mut merkle_dag: ResMut<MerkleDag>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Check for new nodes that need ECS entities
    for (idx, node) in merkle_dag.dag.raw_nodes().iter().enumerate() {
        let node_idx = NodeIndex::new(idx);

        if let Some(ref weight) = node.weight {
            if !merkle_dag.node_to_entity.contains_key(&node_idx) {
                // Create visual representation
                let mesh = meshes.add(Sphere::new(weight.render_state.size));
                let material = materials.add(StandardMaterial {
                    base_color: Color::srgba(
                        weight.render_state.color[0],
                        weight.render_state.color[1],
                        weight.render_state.color[2],
                        weight.render_state.color[3],
                    ),
                    ..default()
                });

                let entity = commands
                    .spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        Transform::from_translation(weight.position),
                        weight.clone(),
                    ))
                    .id();

                merkle_dag.set_node_entity(node_idx, entity);
            }
        }
    }
}

/// Event for DAG modifications
#[derive(Event)]
pub enum DagEvent {
    NodeUpdated(NodeIndex),
    EdgeAdded(EdgeIndex),
    SubgraphCommitted(Vec<NodeIndex>),
}

/// System to handle DAG events
pub fn handle_dag_events(
    mut events: EventReader<DagEvent>,
    merkle_dag: Res<MerkleDag>,
    mut render_events: EventWriter<RenderUpdate>,
) {
    for event in events.read() {
        match event {
            DagEvent::NodeUpdated(idx) => {
                if let Some(node) = merkle_dag.dag.node_weight(*idx) {
                    render_events.write(RenderUpdate::Node(node.uuid));
                }
            }
            DagEvent::EdgeAdded(_idx) => {
                // Handle edge addition
            }
            DagEvent::SubgraphCommitted(nodes) => {
                // Handle subgraph commit
                info!("Subgraph committed with {} nodes", nodes.len());
            }
        }
    }
}

/// Render update event
#[derive(Event)]
pub enum RenderUpdate {
    Node(Uuid),
    Edge(Uuid),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_dag_creation() {
        let mut dag = MerkleDag::new();

        let node1 = MerkleNode {
            cid: Cid("QmNode1".to_string()),
            links: vec![],
            position: Vec3::ZERO,
            render_state: NodeRenderState {
                color: [1.0, 0.0, 0.0, 1.0],
                size: 0.5,
                visible: true,
            },
            metadata: HashMap::new(),
            uuid: Uuid::new_v4(),
        };

        let idx = dag.add_node(node1);
        assert_eq!(dag.dag.node_count(), 1);
    }
}
