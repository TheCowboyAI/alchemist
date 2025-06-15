use crate::domain::{
    DomainError,
    commands::ContentGraphCommand,
    events::{DomainEvent, content_graph::*},
    value_objects::{EdgeId, GraphId, NodeId, NodeType, Position3D, RelatedBy, RelationshipType},
};
use cim_ipld::{
    traits::TypedContent,
    types::{Cid, ContentType},
};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    sync::OnceLock,
    time::SystemTime,
};
use thiserror::Error;

/// Errors that can occur in ContentGraph operations
#[derive(Debug, Error)]
pub enum ContentGraphError {
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),

    #[error("Edge not found: {0}")]
    EdgeNotFound(EdgeId),

    #[error("Invalid position: {0}")]
    InvalidPosition(String),

    #[error("Duplicate node: {0}")]
    DuplicateNode(NodeId),

    #[error("Duplicate edge: {0}")]
    DuplicateEdge(EdgeId),

    #[error("Self-referential edge not allowed")]
    SelfReferentialEdge,

    #[error("CID calculation failed: {0}")]
    CidCalculationFailed(String),

    #[error("Pattern detection failed: {0}")]
    PatternDetectionFailed(String),

    #[error("View not found: {0}")]
    ViewNotFound(String),
}

/// Lazy CID wrapper for deferred calculation
#[derive(Debug, Clone)]
pub struct LazyCid<T: TypedContent> {
    content: T,
    cid_cache: OnceLock<Cid>,
}

impl<T: TypedContent> LazyCid<T> {
    pub fn new(content: T) -> Self {
        Self {
            content,
            cid_cache: OnceLock::new(),
        }
    }

    /// Get the CID, calculating it if necessary
    pub fn cid(&self) -> &Cid {
        self.cid_cache.get_or_init(|| {
            self.content
                .calculate_cid()
                .expect("CID calculation should not fail")
        })
    }

    /// Get the content
    pub fn content(&self) -> &T {
        &self.content
    }

    /// Replace content (creates new instance)
    pub fn replace_content(self, new_content: T) -> Self {
        Self::new(new_content)
    }
}

/// Versioned cache for memoized calculations
#[derive(Debug)]
pub struct VersionedCache<T> {
    version: u64,
    cache: RefCell<Option<(u64, T)>>,
}

impl<T> VersionedCache<T> {
    pub fn new() -> Self {
        Self {
            version: 0,
            cache: RefCell::new(None),
        }
    }

    pub fn get_or_compute<F>(&self, version: u64, f: F) -> T
    where
        F: FnOnce() -> T,
        T: Clone,
    {
        let mut cache = self.cache.borrow_mut();

        match &*cache {
            Some((cached_version, value)) if *cached_version == version => value.clone(),
            _ => {
                let value = f();
                *cache = Some((version, value.clone()));
                value
            }
        }
    }

    pub fn invalidate(&mut self) {
        self.version = self.version.wrapping_add(1);
    }
}

/// Content node with lazy CID evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentNode {
    pub id: NodeId,
    pub content: NodeContent,
    pub position: Position3D,
    pub conceptual_coordinates: Vec<f64>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: SystemTime,
}

/// The content of a node - can be a value or another graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeContent {
    /// Simple value content
    Value {
        content_type: ContentType,
        data: serde_json::Value,
    },

    /// This node is itself a graph (recursive structure)
    Graph {
        graph_id: GraphId,
        graph_type: GraphType,
        summary: String,
    },

    /// Reference to content in another graph
    Reference {
        graph_id: GraphId,
        node_id: NodeId,
        relationship: String,
    },
}

/// Types of graphs based on DDD patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphType {
    /// Entity graph - has identity
    Entity { entity_type: String },

    /// Value object graph - immutable, no identity
    ValueObject { value_type: String },

    /// Aggregate graph - consistency boundary
    Aggregate { aggregate_type: String },

    /// Service graph - operations/processes
    Service { service_type: String },

    /// Event graph - captures state changes
    Event { event_type: String },

    /// Command graph - expresses intent
    Command { command_type: String },

    /// Generic concept graph
    Concept { concept_type: String },
}

/// Content edge representing relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEdge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub relationship: RelatedBy,
    pub strength: f64,
    pub discovered_at: SystemTime,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Semantic cluster of related content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticCluster {
    pub id: String,
    pub nodes: Vec<NodeId>,
    pub centroid: Vec<f64>,
    pub coherence: f64,
}

/// Detected pattern in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub pattern_type: PatternType,
    pub nodes: Vec<NodeId>,
    pub edges: Vec<EdgeId>,
    pub confidence: f64,
    pub detected_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    Hub,
    Cluster,
    Bridge,
    Cycle,
    Hierarchy,
    Custom(String),
}

/// Business metrics for the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetrics {
    pub node_count: usize,
    pub edge_count: usize,
    pub average_degree: f64,
    pub clustering_coefficient: f64,
    pub semantic_coherence: f64,
    pub pattern_count: usize,
}

/// Graph state for CID calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphState {
    pub id: GraphId,
    pub nodes: Vec<(NodeId, ContentNode)>,
    pub edges: Vec<(EdgeId, ContentEdge)>,
    pub version: u64,
}

impl TypedContent for GraphState {
    const CODEC: u64 = 0x300102; // Custom codec for graph content
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x300102);

    fn to_bytes(&self) -> Result<Vec<u8>, cim_ipld::Error> {
        Ok(serde_json::to_vec(self)?)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, cim_ipld::Error> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

/// Content-addressable graph aggregate
#[derive(Debug)]
pub struct ContentGraph {
    // Identity
    pub id: GraphId,

    // Lazy CID evaluation for performance
    state_cid: LazyCid<GraphState>,
    pub previous_cid: Option<Cid>,

    // Content nodes and edges
    pub nodes: HashMap<NodeId, ContentNode>,
    pub edges: HashMap<EdgeId, ContentEdge>,

    // Named views into this graph (selections of nodes/edges)
    pub views: HashMap<String, GraphView>,

    // Memoized calculations
    semantic_clusters: VersionedCache<Vec<SemanticCluster>>,
    relationship_strengths: VersionedCache<HashMap<(NodeId, NodeId), f64>>,
    patterns: VersionedCache<Vec<DetectedPattern>>,
    metrics: VersionedCache<BusinessMetrics>,

    // Version for cache invalidation
    version: u64,
}

/// A named view/selection of nodes and edges within a graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphView {
    pub name: String,
    pub description: String,
    pub node_ids: HashSet<NodeId>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: SystemTime,
}

impl GraphView {
    /// Check if a node belongs to this view
    pub fn contains_node(&self, node_id: &NodeId) -> bool {
        self.node_ids.contains(node_id)
    }

    /// Get the induced edges (all edges between nodes in this view)
    pub fn induced_edges(&self, graph: &ContentGraph) -> HashSet<EdgeId> {
        graph
            .edges
            .iter()
            .filter(|(_, edge)| {
                self.node_ids.contains(&edge.source) && self.node_ids.contains(&edge.target)
            })
            .map(|(id, _)| *id)
            .collect()
    }
}

impl ContentGraph {
    /// Create a new ContentGraph
    pub fn new(id: GraphId) -> Self {
        let initial_state = GraphState {
            id,
            nodes: vec![],
            edges: vec![],
            version: 0,
        };

        Self {
            id,
            state_cid: LazyCid::new(initial_state),
            previous_cid: None,
            nodes: HashMap::new(),
            edges: HashMap::new(),
            views: HashMap::new(),
            semantic_clusters: VersionedCache::new(),
            relationship_strengths: VersionedCache::new(),
            patterns: VersionedCache::new(),
            metrics: VersionedCache::new(),
            version: 0,
        }
    }

    /// Get the current state CID (lazy evaluation)
    pub fn cid(&self) -> &Cid {
        self.state_cid.cid()
    }

    /// Get semantic clusters (memoized calculation)
    pub fn semantic_clusters(&self) -> Vec<SemanticCluster> {
        self.semantic_clusters
            .get_or_compute(self.version, || self.calculate_semantic_clusters())
    }

    /// Get business metrics (memoized calculation)
    pub fn metrics(&self) -> BusinessMetrics {
        self.metrics
            .get_or_compute(self.version, || self.calculate_business_metrics())
    }

    /// Get detected patterns (memoized calculation)
    pub fn patterns(&self) -> Vec<DetectedPattern> {
        self.patterns
            .get_or_compute(self.version, || self.detect_patterns())
    }

    /// Define a named view into this graph
    pub fn define_view(
        &mut self,
        name: String,
        description: String,
        node_ids: HashSet<NodeId>,
    ) -> Result<(), ContentGraphError> {
        // Verify all nodes exist
        for node_id in &node_ids {
            if !self.nodes.contains_key(node_id) {
                return Err(ContentGraphError::NodeNotFound(*node_id));
            }
        }

        let view = GraphView {
            name: name.clone(),
            description,
            node_ids,
            metadata: HashMap::new(),
            created_at: SystemTime::now(),
        };

        self.views.insert(name, view);
        Ok(())
    }

    /// Get a named view
    pub fn get_view(&self, name: &str) -> Option<&GraphView> {
        self.views.get(name)
    }

    /// Get all nodes in a view
    pub fn get_view_nodes(&self, name: &str) -> Vec<&ContentNode> {
        self.views
            .get(name)
            .map(|view| {
                view.node_ids
                    .iter()
                    .filter_map(|id| self.nodes.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all edges in a view (induced edges)
    pub fn get_view_edges(&self, name: &str) -> Vec<&ContentEdge> {
        self.views
            .get(name)
            .map(|view| {
                let induced = view.induced_edges(self);
                induced.iter().filter_map(|id| self.edges.get(id)).collect()
            })
            .unwrap_or_default()
    }

    /// Extract a view as a new ContentGraph
    pub fn extract_view_as_graph(&self, name: &str) -> Option<ContentGraph> {
        self.views.get(name).map(|view| {
            let mut new_graph = ContentGraph::new(GraphId::new());

            // Copy nodes
            for node_id in &view.node_ids {
                if let Some(node) = self.nodes.get(node_id) {
                    new_graph.nodes.insert(*node_id, node.clone());
                }
            }

            // Copy induced edges
            let induced = view.induced_edges(self);
            for edge_id in induced {
                if let Some(edge) = self.edges.get(&edge_id) {
                    new_graph.edges.insert(edge_id, edge.clone());
                }
            }

            new_graph.increment_version();
            new_graph
        })
    }

    /// Get all nested graphs (nodes that contain graphs)
    pub fn get_nested_graphs(&self) -> Vec<(&NodeId, &GraphId, &GraphType)> {
        self.nodes
            .iter()
            .filter_map(|(node_id, node)| match &node.content {
                NodeContent::Graph {
                    graph_id,
                    graph_type,
                    ..
                } => Some((node_id, graph_id, graph_type)),
                _ => None,
            })
            .collect()
    }

    /// Check if this graph represents a DDD pattern
    pub fn get_graph_type(&self) -> Option<GraphType> {
        // Look for a root node that defines the graph type
        self.nodes
            .values()
            .find(|node| {
                // Root nodes typically have no incoming edges
                !self.edges.values().any(|edge| edge.target == node.id)
            })
            .and_then(|root_node| match &root_node.content {
                NodeContent::Graph { graph_type, .. } => Some(graph_type.clone()),
                _ => None,
            })
    }

    /// Handle commands
    pub fn handle_command(
        &mut self,
        command: ContentGraphCommand,
    ) -> Result<Vec<DomainEvent>, ContentGraphError> {
        match command {
            ContentGraphCommand::CreateGraph { graph_id } => self.handle_create_graph(graph_id),
            ContentGraphCommand::AddContent {
                node_id,
                content,
                position,
                metadata,
            } => self.handle_add_content(node_id, content, position, metadata),
            ContentGraphCommand::RemoveContent { node_id } => self.handle_remove_content(node_id),
            ContentGraphCommand::EstablishRelationship {
                edge_id,
                source,
                target,
                relationship,
            } => self.handle_establish_relationship(edge_id, source, target, relationship),
            ContentGraphCommand::RemoveRelationship { edge_id } => {
                self.handle_remove_relationship(edge_id)
            }
            ContentGraphCommand::DiscoverRelationships { threshold } => {
                self.handle_discover_relationships(threshold)
            }
            ContentGraphCommand::UpdateSemanticClusters => self.handle_update_semantic_clusters(),
            ContentGraphCommand::CalculateMetrics => self.handle_calculate_metrics(),
            ContentGraphCommand::DefineView {
                name,
                description,
                node_ids,
            } => self.handle_define_view(name, description, node_ids),
            ContentGraphCommand::UpdateView { name, node_ids } => {
                self.handle_update_view(name, node_ids)
            }
            ContentGraphCommand::RemoveView { name } => self.handle_remove_view(name),
        }
    }

    /// Apply events to update state
    pub fn apply_event(&mut self, event: &DomainEvent) -> Result<(), ContentGraphError> {
        match event {
            DomainEvent::ContentGraphCreated(e) => {
                self.id = e.graph_id;
                self.increment_version();
                Ok(())
            }
            DomainEvent::ContentAdded(e) => {
                let node = ContentNode {
                    id: e.node_id,
                    content: e.content.clone(),
                    position: e.position.clone(),
                    conceptual_coordinates: vec![],
                    metadata: e.metadata.clone(),
                    created_at: SystemTime::now(),
                };
                self.nodes.insert(e.node_id, node);
                self.increment_version();
                Ok(())
            }
            DomainEvent::ContentRemoved(e) => {
                self.nodes.remove(&e.node_id);
                // Remove edges connected to this node
                self.edges
                    .retain(|_, edge| edge.source != e.node_id && edge.target != e.node_id);
                self.increment_version();
                Ok(())
            }
            DomainEvent::RelationshipEstablished(e) => {
                let edge = ContentEdge {
                    id: e.edge_id,
                    source: e.source,
                    target: e.target,
                    relationship: e.relationship.clone(),
                    strength: e.strength,
                    discovered_at: SystemTime::now(),
                    metadata: HashMap::new(),
                };
                self.edges.insert(e.edge_id, edge);
                self.increment_version();
                Ok(())
            }
            DomainEvent::RelationshipRemoved(e) => {
                self.edges.remove(&e.edge_id);
                self.increment_version();
                Ok(())
            }
            _ => Ok(()), // Ignore other events
        }
    }

    // Private command handlers
    fn handle_create_graph(
        &mut self,
        graph_id: GraphId,
    ) -> Result<Vec<DomainEvent>, ContentGraphError> {
        let event = ContentGraphCreated {
            graph_id,
            created_at: SystemTime::now(),
        };
        Ok(vec![DomainEvent::ContentGraphCreated(event)])
    }

    fn handle_add_content(
        &mut self,
        node_id: NodeId,
        content: NodeContent,
        position: Position3D,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<Vec<DomainEvent>, ContentGraphError> {
        if self.nodes.contains_key(&node_id) {
            return Err(ContentGraphError::DuplicateNode(node_id));
        }

        let event = ContentAdded {
            graph_id: self.id,
            node_id,
            content,
            position,
            metadata,
            content_cid: None, // Lazy evaluation - don't calculate unless needed
        };

        Ok(vec![DomainEvent::ContentAdded(event)])
    }

    fn handle_remove_content(
        &mut self,
        node_id: NodeId,
    ) -> Result<Vec<DomainEvent>, ContentGraphError> {
        if !self.nodes.contains_key(&node_id) {
            return Err(ContentGraphError::NodeNotFound(node_id));
        }

        let event = ContentRemoved {
            graph_id: self.id,
            node_id,
        };

        Ok(vec![DomainEvent::ContentRemoved(event)])
    }

    fn handle_establish_relationship(
        &mut self,
        edge_id: EdgeId,
        source: NodeId,
        target: NodeId,
        relationship: RelatedBy,
    ) -> Result<Vec<DomainEvent>, ContentGraphError> {
        if source == target {
            return Err(ContentGraphError::SelfReferentialEdge);
        }

        if !self.nodes.contains_key(&source) {
            return Err(ContentGraphError::NodeNotFound(source));
        }

        if !self.nodes.contains_key(&target) {
            return Err(ContentGraphError::NodeNotFound(target));
        }

        if self.edges.contains_key(&edge_id) {
            return Err(ContentGraphError::DuplicateEdge(edge_id));
        }

        let event = RelationshipEstablished {
            graph_id: self.id,
            edge_id,
            source,
            target,
            relationship,
            strength: 1.0, // Default strength
        };

        Ok(vec![DomainEvent::RelationshipEstablished(event)])
    }

    fn handle_remove_relationship(
        &mut self,
        edge_id: EdgeId,
    ) -> Result<Vec<DomainEvent>, ContentGraphError> {
        if !self.edges.contains_key(&edge_id) {
            return Err(ContentGraphError::EdgeNotFound(edge_id));
        }

        let event = RelationshipRemoved {
            graph_id: self.id,
            edge_id,
        };

        Ok(vec![DomainEvent::RelationshipRemoved(event)])
    }

    fn handle_discover_relationships(
        &mut self,
        threshold: f64,
    ) -> Result<Vec<DomainEvent>, ContentGraphError> {
        let mut events = vec![];

        // Simple similarity-based relationship discovery
        let nodes: Vec<_> = self.nodes.values().collect();
        for i in 0..nodes.len() {
            for j in i + 1..nodes.len() {
                let similarity = self.calculate_similarity(&nodes[i], &nodes[j]);
                if similarity >= threshold {
                    let edge_id = EdgeId::new();
                    let event = RelationshipDiscovered {
                        graph_id: self.id,
                        edge_id,
                        source: nodes[i].id,
                        target: nodes[j].id,
                        relationship: RelatedBy::Similar,
                        confidence: similarity,
                    };
                    events.push(DomainEvent::RelationshipDiscovered(event));
                }
            }
        }

        Ok(events)
    }

    fn handle_update_semantic_clusters(&mut self) -> Result<Vec<DomainEvent>, ContentGraphError> {
        let clusters = self.calculate_semantic_clusters();

        let event = SemanticClustersUpdated {
            graph_id: self.id,
            clusters,
        };

        Ok(vec![DomainEvent::SemanticClustersUpdated(event)])
    }

    fn handle_calculate_metrics(&mut self) -> Result<Vec<DomainEvent>, ContentGraphError> {
        let metrics = self.calculate_business_metrics();

        let event = MetricsCalculated {
            graph_id: self.id,
            metrics,
        };

        Ok(vec![DomainEvent::MetricsCalculated(event)])
    }

    fn handle_define_view(
        &mut self,
        name: String,
        description: String,
        node_ids: Vec<NodeId>,
    ) -> Result<Vec<DomainEvent>, ContentGraphError> {
        // Verify all nodes exist
        for node_id in &node_ids {
            if !self.nodes.contains_key(node_id) {
                return Err(ContentGraphError::NodeNotFound(*node_id));
            }
        }

        // Views are internal to this graph, no events needed
        self.define_view(name, description, node_ids.into_iter().collect())?;
        Ok(vec![])
    }

    fn handle_update_view(
        &mut self,
        name: String,
        node_ids: Vec<NodeId>,
    ) -> Result<Vec<DomainEvent>, ContentGraphError> {
        // Verify view exists
        if !self.views.contains_key(&name) {
            return Err(ContentGraphError::ViewNotFound(name));
        }

        // Verify all nodes exist
        for node_id in &node_ids {
            if !self.nodes.contains_key(node_id) {
                return Err(ContentGraphError::NodeNotFound(*node_id));
            }
        }

        // Update the view
        if let Some(view) = self.views.get_mut(&name) {
            view.node_ids = node_ids.into_iter().collect();
        }

        Ok(vec![])
    }

    fn handle_remove_view(&mut self, name: String) -> Result<Vec<DomainEvent>, ContentGraphError> {
        if !self.views.contains_key(&name) {
            return Err(ContentGraphError::ViewNotFound(name));
        }

        self.views.remove(&name);
        Ok(vec![])
    }

    // Private helper methods
    fn increment_version(&mut self) {
        self.version = self.version.wrapping_add(1);

        // Update state CID with new graph state
        let state = GraphState {
            id: self.id,
            nodes: self.nodes.iter().map(|(k, v)| (*k, v.clone())).collect(),
            edges: self.edges.iter().map(|(k, v)| (*k, v.clone())).collect(),
            version: self.version,
        };

        // Store previous CID before creating new one
        if self.version > 1 {
            self.previous_cid = Some(self.state_cid.cid().clone());
        }

        self.state_cid = LazyCid::new(state);
    }

    fn calculate_similarity(&self, node1: &ContentNode, node2: &ContentNode) -> f64 {
        // Simple position-based similarity for now
        let dx = (node1.position.x - node2.position.x) as f64;
        let dy = (node1.position.y - node2.position.y) as f64;
        let dz = (node1.position.z - node2.position.z) as f64;
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();

        // Convert distance to similarity (closer = more similar)
        1.0 / (1.0 + distance)
    }

    fn calculate_semantic_clusters(&self) -> Vec<SemanticCluster> {
        // Placeholder implementation
        vec![]
    }

    fn calculate_business_metrics(&self) -> BusinessMetrics {
        let node_count = self.nodes.len();
        let edge_count = self.edges.len();

        let average_degree = if node_count > 0 {
            (2.0 * edge_count as f64) / node_count as f64
        } else {
            0.0
        };

        BusinessMetrics {
            node_count,
            edge_count,
            average_degree,
            clustering_coefficient: 0.0, // TODO: Implement
            semantic_coherence: 0.0,     // TODO: Implement
            pattern_count: self.patterns().len(),
        }
    }

    fn detect_patterns(&self) -> Vec<DetectedPattern> {
        // Placeholder implementation
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_graph_creation() {
        let graph_id = GraphId::new();
        let graph = ContentGraph::new(graph_id);

        assert_eq!(graph.id, graph_id);
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.edges.len(), 0);
        assert_eq!(graph.version, 0);
    }

    #[test]
    fn test_lazy_cid_evaluation() {
        let graph_id = GraphId::new();
        let graph = ContentGraph::new(graph_id);

        // CID should be calculated lazily
        let cid = graph.cid();
        assert!(!cid.to_string().is_empty());
    }

    #[test]
    fn test_add_content() {
        let graph_id = GraphId::new();
        let mut graph = ContentGraph::new(graph_id);

        let node_id = NodeId::new();
        let position = Position3D {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let metadata = HashMap::new();

        let command = ContentGraphCommand::AddContent {
            node_id,
            content: NodeContent::Value {
                content_type: ContentType::Custom(0x300103),
                data: serde_json::json!({"label": "Test Node"}),
            },
            position: position.clone(),
            metadata,
        };

        let events = graph.handle_command(command).unwrap();
        assert_eq!(events.len(), 1);

        // Apply the event
        graph.apply_event(&events[0]).unwrap();

        assert_eq!(graph.nodes.len(), 1);
        assert!(graph.nodes.contains_key(&node_id));
    }
}
