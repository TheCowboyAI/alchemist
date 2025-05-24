use bevy::prelude::*;
use daggy::{Dag, EdgeIndex, NodeIndex, Walker};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
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

/// Schema for native MerkleDag JSON format
#[derive(Serialize, Deserialize, Debug)]
pub struct DagSchema {
    nodes: Vec<MerkleNode>,
    edges: Vec<MerkleEdge>,
    metadata: HashMap<String, String>,
}

/// Schema for arrows.app compatibility
#[derive(Serialize, Deserialize, Debug)]
pub struct ArrowsSchema {
    nodes: Vec<ArrowsNode>,
    relationships: Vec<ArrowsRelationship>,
    style: ArrowsStyle,
}

/// Style for arrows.app graph
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ArrowsStyle {
    #[serde(flatten)]
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct ArrowsNode {
    id: String,
    caption: String,
    labels: Vec<String>,
    #[serde(default)]
    properties: HashMap<String, serde_json::Value>,
    position: ArrowsPosition,
    #[serde(default)]
    style: HashMap<String, serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct ArrowsPosition {
    x: f32,
    y: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct ArrowsRelationship {
    id: String,
    #[serde(rename = "type")]
    rel_type: String,
    #[serde(rename = "fromId")]
    from_id: String,
    #[serde(rename = "toId")]
    to_id: String,
    properties: HashMap<String, serde_json::Value>,
    #[serde(default)]
    style: HashMap<String, serde_json::Value>,
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
        for (_edge_idx, node_idx) in self.dag.children(start).iter(&self.dag) {
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
        let nodes: Vec<MerkleNode> = self
            .dag
            .raw_nodes()
            .iter()
            .map(|node| node.weight.clone())
            .collect();

        let edges: Vec<MerkleEdge> = self
            .dag
            .raw_edges()
            .iter()
            .map(|edge| edge.weight.clone())
            .collect();

        let metadata = HashMap::new(); // Add any metadata if needed
        let schema = DagSchema { nodes, edges, metadata };
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

    /// Export to arrows.app compatible JSON format
    pub fn to_arrows_json(&self) -> Result<String, serde_json::Error> {
        let arrows_schema = self.to_arrows_schema();
        serde_json::to_string_pretty(&arrows_schema)
    }

    /// Convert to arrows.app compatible schema
    pub fn to_arrows_schema(&self) -> ArrowsSchema {
        let mut nodes = Vec::new();
        let mut relationships = Vec::new();

        // Convert nodes
        for (_idx, node) in self.dag.raw_nodes().iter().enumerate() {
            let merkle_node = &node.weight;

            // Build properties map
            let mut properties = HashMap::new();
            properties.insert(
                "cid".to_string(),
                serde_json::Value::String(merkle_node.cid.0.clone()),
            );
            properties.insert(
                "uuid".to_string(),
                serde_json::Value::String(merkle_node.uuid.to_string()),
            );
            properties.insert(
                "size".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(merkle_node.render_state.size as f64)
                        .unwrap_or(serde_json::Number::from(0)),
                ),
            );
            properties.insert(
                "visible".to_string(),
                serde_json::Value::Bool(merkle_node.render_state.visible),
            );

            // Add metadata fields with meta_ prefix
            for (key, value) in &merkle_node.metadata {
                properties.insert(
                    format!("meta_{key}"),
                    serde_json::Value::String(value.clone()),
                );
            }

            // Add color as hex
            let color_hex = format!(
                "#{:02x}{:02x}{:02x}",
                (merkle_node.render_state.color[0] * 255.0) as u8,
                (merkle_node.render_state.color[1] * 255.0) as u8,
                (merkle_node.render_state.color[2] * 255.0) as u8
            );
            properties.insert("color".to_string(), serde_json::Value::String(color_hex));

            // Use name from metadata as caption if available, otherwise use CID
            let caption = merkle_node.metadata.get("name")
                .or_else(|| merkle_node.metadata.get("caption"))
                .cloned()
                .unwrap_or_else(|| merkle_node.cid.0.clone());

            nodes.push(ArrowsNode {
                id: merkle_node.cid.0.clone(),
                caption,
                labels: vec!["MerkleNode".to_string()],
                properties,
                position: ArrowsPosition {
                    x: merkle_node.position.x,
                    y: merkle_node.position.z, // Using z as y for 2D projection
                },
                style: HashMap::new(),
            });
        }

        // Convert edges
        for (edge_idx, edge) in self.dag.raw_edges().iter().enumerate() {
            let merkle_edge = &edge.weight;
            let source_idx = edge.source();
            let target_idx = edge.target();

            if let (Some(source), Some(target)) = (
                self.dag.node_weight(source_idx),
                self.dag.node_weight(target_idx),
            ) {
                let mut properties = HashMap::new();
                properties.insert(
                    "weight".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(merkle_edge.weight)
                            .unwrap_or(serde_json::Number::from(1)),
                    ),
                );
                properties.insert(
                    "thickness".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(merkle_edge.thickness as f64)
                            .unwrap_or(serde_json::Number::from(0)),
                    ),
                );
                properties.insert(
                    "style".to_string(),
                    serde_json::Value::String(format!("{:?}", merkle_edge.style)),
                );
                properties.insert(
                    "has_proof".to_string(),
                    serde_json::Value::Bool(!merkle_edge.proof.is_empty()),
                );

                relationships.push(ArrowsRelationship {
                    id: format!("edge_{edge_idx}"),
                    rel_type: "LINKS_TO".to_string(),
                    from_id: source.cid.0.clone(),
                    to_id: target.cid.0.clone(),
                    properties,
                    style: HashMap::new(),
                });
            }
        }

        ArrowsSchema {
            style: ArrowsStyle { properties: HashMap::new() },
            nodes,
            relationships,
        }
    }

    /// Import from arrows.app compatible JSON
    pub fn from_arrows_json(json: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let arrows_schema: ArrowsSchema = serde_json::from_str(json)?;
        Self::from_arrows_schema(arrows_schema)
    }

    /// Convert from arrows.app schema
    fn from_arrows_schema(schema: ArrowsSchema) -> Result<Self, Box<dyn std::error::Error>> {
        let mut dag = MerkleDag::new();
        let mut cid_to_idx = HashMap::new();

        // Add nodes
        for arrows_node in schema.nodes {
            let cid = Cid(arrows_node.id.clone());

            // Extract properties
            let uuid = arrows_node
                .properties
                .get("uuid")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(Uuid::new_v4);

            let size = arrows_node
                .properties
                .get("size")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5) as f32;

            let visible = arrows_node
                .properties
                .get("visible")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            // Parse color from hex
            let color = if let Some(color_str) =
                arrows_node.properties.get("color").and_then(|v| v.as_str())
            {
                Self::parse_hex_color(color_str)
            } else {
                [1.0, 1.0, 1.0, 1.0]
            };

            // Extract metadata
            let mut metadata = HashMap::new();

            // Store caption if it's different from the id
            if arrows_node.caption != arrows_node.id {
                metadata.insert("caption".to_string(), arrows_node.caption.clone());
            }

            for (key, value) in &arrows_node.properties {
                if key.starts_with("meta_") {
                    if let Some(val_str) = value.as_str() {
                        metadata.insert(
                            key.strip_prefix("meta_").unwrap().to_string(),
                            val_str.to_string(),
                        );
                    }
                } else if key == "name" || key == "type" {
                    // Store common properties directly in metadata
                    if let Some(val_str) = value.as_str() {
                        metadata.insert(key.clone(), val_str.to_string());
                    }
                }
            }

            let node = MerkleNode {
                cid: cid.clone(),
                links: vec![], // Will be populated from relationships
                position: Vec3::new(arrows_node.position.x, 0.0, arrows_node.position.y),
                render_state: NodeRenderState {
                    color,
                    size,
                    visible,
                },
                metadata,
                uuid,
            };

            let idx = dag.add_node(node);
            cid_to_idx.insert(arrows_node.id, idx);
        }

        // Add edges and update links
        for relationship in schema.relationships {
            if let (Some(&source_idx), Some(&target_idx)) = (
                cid_to_idx.get(&relationship.from_id),
                cid_to_idx.get(&relationship.to_id),
            ) {
                // Update source node's links
                if let Some(source_node) = dag.dag.node_weight_mut(source_idx) {
                    source_node.links.push(Cid(relationship.to_id.clone()));
                }

                // Create edge
                let weight = relationship
                    .properties
                    .get("weight")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0);

                let thickness = relationship
                    .properties
                    .get("thickness")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.1) as f32;

                let style = relationship
                    .properties
                    .get("style")
                    .and_then(|v| v.as_str())
                    .and_then(|s| match s {
                        "Dashed" => Some(EdgeStyle::Dashed),
                        "Dotted" => Some(EdgeStyle::Dotted),
                        _ => Some(EdgeStyle::Solid),
                    })
                    .unwrap_or(EdgeStyle::Solid);

                let edge = MerkleEdge {
                    weight,
                    proof: vec![], // Empty proof by default
                    thickness,
                    style,
                };

                dag.add_edge(source_idx, target_idx, edge)?;
            }
        }

        Ok(dag)
    }

    /// Parse hex color string to RGBA array
    fn parse_hex_color(hex: &str) -> [f32; 4] {
        let hex = hex.trim_start_matches('#');
        if hex.len() >= 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0];
            }
        }
        [1.0, 1.0, 1.0, 1.0] // Default white
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
        let nodes: Vec<MerkleNode> = self
            .dag
            .raw_nodes()
            .iter()
            .map(|node| node.weight.clone())
            .collect();

        let edges: Vec<MerkleEdge> = self
            .dag
            .raw_edges()
            .iter()
            .map(|edge| edge.weight.clone())
            .collect();

        let metadata = HashMap::new(); // Add any metadata if needed
        DagSchema { nodes, edges, metadata }
    }
}

/// System to create ECS entities for DAG nodes
pub fn sync_dag_to_ecs(
    mut commands: Commands,
    mut merkle_dag: ResMut<MerkleDag>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Collect nodes that need ECS entities first to avoid borrow conflicts
    let nodes_to_create: Vec<(NodeIndex, MerkleNode)> = merkle_dag
        .dag
        .raw_nodes()
        .iter()
        .enumerate()
        .filter_map(|(idx, node)| {
            let node_idx = NodeIndex::new(idx);
            if !merkle_dag.node_to_entity.contains_key(&node_idx) {
                Some((node_idx, node.weight.clone()))
            } else {
                None
            }
        })
        .collect();

    // Now create entities for the collected nodes
    for (node_idx, node) in nodes_to_create {
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

        let entity = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_translation(node.position),
                node,
            ))
            .id();

        merkle_dag.set_node_entity(node_idx, entity);
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

    #[test]
    fn test_arrows_app_export_import() {
        let mut dag = MerkleDag::new();

        // Create test nodes
        let mut metadata1 = HashMap::new();
        metadata1.insert("name".to_string(), "Process A".to_string());
        metadata1.insert("type".to_string(), "bounded_context".to_string());

        let node1 = MerkleNode {
            cid: Cid("QmNode1".to_string()),
            links: vec![],
            position: Vec3::new(100.0, 0.0, 200.0),
            render_state: NodeRenderState {
                color: [1.0, 0.5, 0.0, 1.0],
                size: 0.75,
                visible: true,
            },
            metadata: metadata1,
            uuid: Uuid::new_v4(),
        };

        let node2 = MerkleNode {
            cid: Cid("QmNode2".to_string()),
            links: vec![],
            position: Vec3::new(300.0, 0.0, 400.0),
            render_state: NodeRenderState {
                color: [0.0, 1.0, 0.5, 1.0],
                size: 0.5,
                visible: true,
            },
            metadata: HashMap::new(),
            uuid: Uuid::new_v4(),
        };

        let idx1 = dag.add_node(node1);
        let idx2 = dag.add_node(node2);

        // Add edge
        let edge = MerkleEdge {
            weight: 2.5,
            proof: vec![1, 2, 3],
            thickness: 0.2,
            style: EdgeStyle::Dashed,
        };
        dag.add_edge(idx1, idx2, edge).unwrap();

        // Export to arrows.app format
        let arrows_json = dag.to_arrows_json().unwrap();

        // Import back
        let imported_dag = MerkleDag::from_arrows_json(&arrows_json).unwrap();

        // Verify structure
        assert_eq!(imported_dag.dag.node_count(), 2);
        assert_eq!(imported_dag.dag.edge_count(), 1);

        // Verify node properties
        let imported_node1 = imported_dag.get_node_by_cid(&Cid("QmNode1".to_string())).unwrap();
        assert_eq!(imported_node1.position.x, 100.0);
        assert_eq!(imported_node1.position.z, 200.0);
        assert_eq!(imported_node1.render_state.size, 0.75);
        assert_eq!(imported_node1.metadata.get("name"), Some(&"Process A".to_string()));
        assert_eq!(imported_node1.metadata.get("type"), Some(&"bounded_context".to_string()));

        // Verify color conversion (should be close due to float precision)
        assert!((imported_node1.render_state.color[0] - 1.0).abs() < 0.01);
        assert!((imported_node1.render_state.color[1] - 0.5).abs() < 0.01);
        assert!((imported_node1.render_state.color[2] - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_import_real_arrows_app_file() {
        // Test with a minimal arrows.app format similar to real exports
        let arrows_schema = serde_json::json!({
            "style": {
                "font-family": "sans-serif",
                "node-color": "#ffffff"
            },
            "nodes": [
                {
                    "id": "n0",
                    "position": {
                        "x": -325.846,
                        "y": -435.525
                    },
                    "caption": "Org",
                    "style": {},
                    "labels": ["Entity"],
                    "properties": {
                        "id": "",
                        "cn": ""
                    }
                },
                {
                    "id": "n1",
                    "position": {
                        "x": 249.5,
                        "y": -74.726
                    },
                    "caption": "Person",
                    "style": {},
                    "labels": ["Entity"],
                    "properties": {
                        "id": "",
                        "cn": ""
                    }
                }
            ],
            "relationships": [
                {
                    "id": "n0",
                    "type": "employs",
                    "style": {},
                    "properties": {},
                    "fromId": "n0",
                    "toId": "n1"
                }
            ]
        });

        let arrows_json = serde_json::to_string(&arrows_schema).unwrap();

        // Import the JSON
        let dag = MerkleDag::from_arrows_json(&arrows_json).unwrap();

        // Verify structure
        assert_eq!(dag.dag.node_count(), 2);
        assert_eq!(dag.dag.edge_count(), 1);

        // Verify nodes were imported with correct captions in metadata
        let node0 = dag.get_node_by_cid(&Cid("n0".to_string())).unwrap();
        assert_eq!(node0.metadata.get("caption"), Some(&"Org".to_string()));
        assert_eq!(node0.position.x, -325.846);
        assert_eq!(node0.position.z, -435.525); // y becomes z

        let node1 = dag.get_node_by_cid(&Cid("n1".to_string())).unwrap();
        assert_eq!(node1.metadata.get("caption"), Some(&"Person".to_string()));

        // Export back and verify round-trip
        let exported = dag.to_arrows_json().unwrap();
        let re_imported = MerkleDag::from_arrows_json(&exported).unwrap();
        assert_eq!(re_imported.dag.node_count(), 2);
        assert_eq!(re_imported.dag.edge_count(), 1);
    }

    #[test]
    fn test_daggy_usage() {
        let mut dag = MerkleDag::new();

        let node1 = MerkleNode {
            cid: Cid("QmNode1".to_string()),
            links: vec![],
            position: Vec3::new(100.0, 0.0, 200.0),
            render_state: NodeRenderState {
                color: [1.0, 0.5, 0.0, 1.0],
                size: 0.75,
                visible: true,
            },
            metadata: HashMap::new(),
            uuid: Uuid::new_v4(),
        };

        let _idx = dag.add_node(node1);
    }
}
