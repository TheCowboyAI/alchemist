use daggy::{Dag, NodeIndex, EdgeIndex, Walker};
use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::collections::HashMap;

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
}

impl Default for MerkleDag {
    fn default() -> Self {
        Self {
            dag: Dag::new(),
            cid_to_node: HashMap::new(),
            node_to_entity: HashMap::new(),
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
    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex, edge: MerkleEdge) -> Result<EdgeIndex, daggy::WouldCycle<MerkleEdge>> {
        self.dag.add_edge(source, target, edge)
    }

    /// Validate Merkle path between two nodes
    pub fn validate_merkle_path(&self, start: NodeIndex, end: NodeIndex) -> Result<bool, String> {
        let mut current = start;
        let mut visited = Vec::new();

        // Use Daggy's walker for traversal
        let walker = self.dag.children(start);

        for (edge_idx, node_idx) in walker.iter(&self.dag) {
            visited.push(node_idx);

            if node_idx == end {
                // Verify all proofs along the path
                return Ok(self.verify_path_proofs(&visited));
            }

            // Continue searching depth-first
            if let Some(path_node) = self.find_path_to(node_idx, end) {
                visited.extend(path_node);
                return Ok(self.verify_path_proofs(&visited));
            }
        }

        Err("No path found".to_string())
    }

    /// Find path using Daggy's recursive walker
    fn find_path_to(&self, start: NodeIndex, end: NodeIndex) -> Option<Vec<NodeIndex>> {
        let mut path = Vec::new();

        for (_, node) in self.dag.recursive_walk(start, |g, n| g.children(n).iter(g)).iter(&self.dag) {
            path.push(node);
            if node == end {
                return Some(path);
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
        self.cid_to_node.get(cid)
            .and_then(|idx| self.dag.node_weight(*idx))
    }

    /// Associate a DAG node with an ECS entity
    pub fn set_node_entity(&mut self, node_idx: NodeIndex, entity: Entity) {
        self.node_to_entity.insert(node_idx, entity);
    }

    /// Serialize the DAG for persistence
    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        #[derive(Serialize)]
        struct DagSchema {
            nodes: Vec<(NodeIndex, MerkleNode)>,
            edges: Vec<(NodeIndex, NodeIndex, MerkleEdge)>,
        }

        let schema = DagSchema {
            nodes: self.dag.node_indices()
                .filter_map(|idx| self.dag.node_weight(idx).map(|n| (idx, n.clone())))
                .collect(),
            edges: self.dag.edge_indices()
                .filter_map(|idx| {
                    let (a, b) = self.dag.edge_endpoints(idx)?;
                    let weight = self.dag.edge_weight(idx)?;
                    Some((a, b, weight.clone()))
                })
                .collect(),
        };

        serde_json::to_string(&schema)
    }
}

/// System to sync Merkle DAG changes to ECS
pub fn sync_merkle_dag_to_ecs(
    mut commands: Commands,
    merkle_dag: Res<MerkleDag>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<Entity, Without<MerkleNode>>,
) {
    // Check for new nodes that need ECS entities
    for node_idx in merkle_dag.dag.node_indices() {
        if !merkle_dag.node_to_entity.contains_key(&node_idx) {
            if let Some(node) = merkle_dag.dag.node_weight(node_idx) {
                // Create visual representation
                let mesh = meshes.add(Sphere::new(node.render_state.size));
                let material = materials.add(StandardMaterial {
                    base_color: Color::srgba(
                        node.render_state.color[0],
                        node.render_state.color[1],
                        node.render_state.color[2],
                        node.render_state.color[3],
                    ),
                    ..default()
                });

                let entity = commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(node.position),
                    node.clone(),
                )).id();

                // This would need mutable access to merkle_dag
                // In practice, you'd handle this differently
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
                    render_events.send(RenderUpdate::Node(node.uuid));
                }
            }
            DagEvent::EdgeAdded(idx) => {
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
