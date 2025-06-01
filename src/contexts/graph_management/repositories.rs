use crate::contexts::graph_management::{domain::*, events::*};
use std::collections::HashMap;
use uuid::Uuid;

// ============= Repository Layer =============
// Repositories are named as plural domain terms (Rule #4)

/// Repository for storing and retrieving graphs
pub struct Graphs {
    storage: HashMap<Uuid, GraphData>,
}

/// Repository for storing graph events (event store)
pub struct GraphEvents {
    events: Vec<GraphEvent>,
    snapshots: HashMap<Uuid, GraphSnapshot>,
}

/// Repository for quick node lookups across all graphs
pub struct Nodes {
    index: HashMap<Uuid, NodeLocation>,
}

/// Repository for edge queries and traversal
pub struct Edges {
    adjacency: HashMap<Uuid, Vec<EdgeReference>>,
}

// ============= Data Transfer Objects =============

/// Complete graph data for persistence
#[derive(Clone, Debug)]
pub struct GraphData {
    pub identity: GraphIdentity,
    pub metadata: GraphMetadata,
    pub journey: GraphJourney,
    pub nodes: Vec<NodeData>,
    pub edges: Vec<EdgeData>,
}

/// Node data for persistence
#[derive(Clone, Debug)]
pub struct NodeData {
    pub identity: NodeIdentity,
    pub content: NodeContent,
    pub position: SpatialPosition,
}

/// Edge data for persistence
#[derive(Clone, Debug)]
pub struct EdgeData {
    pub identity: EdgeIdentity,
    pub relationship: EdgeRelationship,
}

/// Location of a node within the system
#[derive(Clone, Debug)]
pub struct NodeLocation {
    pub graph_id: GraphIdentity,
    pub node_id: NodeIdentity,
}

/// Reference to an edge for traversal
#[derive(Clone, Debug)]
pub struct EdgeReference {
    pub edge_id: EdgeIdentity,
    pub target_node: NodeIdentity,
    pub category: String,
}

/// Event wrapper for event store
#[derive(Clone, Debug)]
pub enum GraphEvent {
    Created(GraphCreated),
    NodeAdded(NodeAdded),
    EdgeConnected(EdgeConnected),
    NodeRemoved(NodeRemoved),
    EdgeDisconnected(EdgeDisconnected),
    NodeMoved(NodeMoved),
    PropertyUpdated(PropertyUpdated),
    LabelApplied(LabelApplied),
    Deleted(GraphDeleted),
}

/// Snapshot of graph state at a point in time
#[derive(Clone, Debug)]
pub struct GraphSnapshot {
    pub graph_id: GraphIdentity,
    pub version: u64,
    pub timestamp: std::time::SystemTime,
    pub data: GraphData,
}

// ============= Repository Implementations =============

impl Graphs {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    /// Store a graph
    pub fn store(&mut self, data: GraphData) {
        self.storage.insert(data.identity.0, data);
    }

    /// Retrieve a graph by identity
    pub fn find(&self, id: GraphIdentity) -> Option<&GraphData> {
        self.storage.get(&id.0)
    }

    /// List all graphs
    pub fn list(&self) -> Vec<&GraphData> {
        self.storage.values().collect()
    }

    /// Remove a graph
    pub fn remove(&mut self, id: GraphIdentity) -> Option<GraphData> {
        self.storage.remove(&id.0)
    }

    /// Check if a graph exists
    pub fn exists(&self, id: GraphIdentity) -> bool {
        self.storage.contains_key(&id.0)
    }
}

impl GraphEvents {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            snapshots: HashMap::new(),
        }
    }

    /// Append an event to the store
    pub fn append(&mut self, event: GraphEvent) {
        self.events.push(event);
    }

    /// Get all events for a graph
    pub fn events_for_graph(&self, graph_id: GraphIdentity) -> Vec<&GraphEvent> {
        self.events
            .iter()
            .filter(|event| match event {
                GraphEvent::Created(e) => e.graph == graph_id,
                GraphEvent::NodeAdded(e) => e.graph == graph_id,
                GraphEvent::EdgeConnected(e) => e.graph == graph_id,
                GraphEvent::NodeRemoved(e) => e.graph == graph_id,
                GraphEvent::EdgeDisconnected(e) => e.graph == graph_id,
                GraphEvent::NodeMoved(e) => e.graph == graph_id,
                GraphEvent::PropertyUpdated(e) => e.graph == graph_id,
                GraphEvent::LabelApplied(e) => e.graph == graph_id,
                GraphEvent::Deleted(e) => e.graph == graph_id,
            })
            .collect()
    }

    /// Get events since a specific version
    pub fn events_since(&self, graph_id: GraphIdentity, version: u64) -> Vec<&GraphEvent> {
        // In a real implementation, events would have sequence numbers
        self.events_for_graph(graph_id)
            .into_iter()
            .skip(version as usize)
            .collect()
    }

    /// Store a snapshot
    pub fn store_snapshot(&mut self, snapshot: GraphSnapshot) {
        self.snapshots.insert(snapshot.graph_id.0, snapshot);
    }

    /// Get latest snapshot for a graph
    pub fn latest_snapshot(&self, graph_id: GraphIdentity) -> Option<&GraphSnapshot> {
        self.snapshots.get(&graph_id.0)
    }
}

impl Nodes {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    /// Index a node location
    pub fn index_node(&mut self, node_id: NodeIdentity, location: NodeLocation) {
        self.index.insert(node_id.0, location);
    }

    /// Find where a node is located
    pub fn locate(&self, node_id: NodeIdentity) -> Option<&NodeLocation> {
        self.index.get(&node_id.0)
    }

    /// Remove a node from the index
    pub fn remove(&mut self, node_id: NodeIdentity) -> Option<NodeLocation> {
        self.index.remove(&node_id.0)
    }
}

impl Edges {
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
        }
    }

    /// Adds an edge reference from a source node
    pub fn add_edge(&mut self, source: NodeIdentity, reference: EdgeReference) {
        self.adjacency.entry(source.0).or_default().push(reference);
    }

    /// Get all edges from a node
    pub fn edges_from(&self, node: NodeIdentity) -> Vec<&EdgeReference> {
        self.adjacency
            .get(&node.0)
            .map(|edges| edges.iter().collect())
            .unwrap_or_default()
    }

    /// Remove all edges from a node
    pub fn remove_edges_from(&mut self, node: NodeIdentity) -> Vec<EdgeReference> {
        self.adjacency.remove(&node.0).unwrap_or_default()
    }
}

// ============= Default Implementations =============

impl Default for Graphs {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GraphEvents {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Nodes {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Edges {
    fn default() -> Self {
        Self::new()
    }
}
