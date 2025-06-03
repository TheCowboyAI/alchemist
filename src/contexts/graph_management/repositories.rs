use crate::contexts::graph_management::{domain::*, events::*};
use std::collections::HashMap;

// ============= Repository Layer =============
// Repositories are named as plural domain terms (Rule #4)

/// Repository for storing and retrieving graphs
pub struct Graphs {
    storage: HashMap<GraphIdentity, GraphData>,
}

/// Repository for storing graph events (event store)
pub struct GraphEvents {
    events: Vec<(GraphIdentity, GraphEvent)>,
    snapshots: HashMap<GraphIdentity, GraphSnapshot>,
}

/// Repository for quick node lookups across all graphs
pub struct Nodes {
    index: HashMap<NodeIdentity, NodeLocation>,
}

/// Repository for edge queries and traversal
pub struct Edges {
    adjacency: HashMap<NodeIdentity, Vec<EdgeReference>>,
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

    /// Store or update a graph
    pub fn store(&mut self, graph: GraphData) {
        self.storage.insert(graph.identity, graph);
    }

    /// Find a graph by its identity
    pub fn find(&self, id: GraphIdentity) -> Option<&GraphData> {
        self.storage.get(&id)
    }

    /// Find a mutable reference to a graph
    pub fn find_mut(&mut self, id: GraphIdentity) -> Option<&mut GraphData> {
        self.storage.get_mut(&id)
    }

    /// List all graphs
    pub fn list(&self) -> Vec<&GraphData> {
        self.storage.values().collect()
    }

    /// Check if a graph exists
    pub fn exists(&self, id: GraphIdentity) -> bool {
        self.storage.contains_key(&id)
    }

    /// Remove a graph
    pub fn remove(&mut self, id: GraphIdentity) -> Option<GraphData> {
        self.storage.remove(&id)
    }

    /// Count total graphs
    pub fn count(&self) -> usize {
        self.storage.len()
    }

    /// Find graphs by domain
    pub fn find_by_domain(&self, domain: &str) -> Vec<&GraphData> {
        self.storage
            .values()
            .filter(|graph| graph.metadata.domain == domain)
            .collect()
    }

    /// Update graph metadata
    pub fn update_metadata(&mut self, id: GraphIdentity, metadata: GraphMetadata) -> bool {
        if let Some(graph) = self.storage.get_mut(&id) {
            graph.metadata = metadata;
            true
        } else {
            false
        }
    }
}

impl GraphEvents {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            snapshots: HashMap::new(),
        }
    }

    /// Append a new event
    pub fn append(&mut self, event: GraphEvent) {
        let graph_id = match &event {
            GraphEvent::Created(e) => e.graph,
            GraphEvent::NodeAdded(e) => e.graph,
            GraphEvent::EdgeConnected(e) => e.graph,
            GraphEvent::NodeRemoved(e) => e.graph,
            GraphEvent::EdgeDisconnected(e) => e.graph,
            GraphEvent::NodeMoved(e) => e.graph,
            GraphEvent::PropertyUpdated(e) => e.graph,
            GraphEvent::LabelApplied(e) => e.graph,
            GraphEvent::Deleted(e) => e.graph,
        };
        self.events.push((graph_id, event));
    }

    /// Get all events for a graph
    pub fn events_for_graph(&self, graph_id: GraphIdentity) -> Vec<&GraphEvent> {
        self.events
            .iter()
            .filter(|(id, _)| *id == graph_id)
            .map(|(_, event)| event)
            .collect()
    }

    /// Get events since a specific version
    pub fn events_since(&self, graph_id: GraphIdentity, version: u64) -> Vec<&GraphEvent> {
        let graph_events: Vec<_> = self.events_for_graph(graph_id);
        graph_events.into_iter().skip(version as usize).collect()
    }

    /// Store a snapshot
    pub fn store_snapshot(&mut self, snapshot: GraphSnapshot) {
        self.snapshots.insert(snapshot.graph_id, snapshot);
    }

    /// Get the latest snapshot for a graph
    pub fn latest_snapshot(&self, graph_id: GraphIdentity) -> Option<&GraphSnapshot> {
        self.snapshots.get(&graph_id)
    }

    /// Count total events
    pub fn total_events(&self) -> usize {
        self.events.len()
    }

    /// Get event count for a specific graph
    pub fn event_count_for_graph(&self, graph_id: GraphIdentity) -> usize {
        self.events.iter().filter(|(id, _)| *id == graph_id).count()
    }

    /// Clear events older than snapshot
    pub fn compact_events(&mut self, graph_id: GraphIdentity) {
        if let Some(snapshot) = self.snapshots.get(&graph_id) {
            let cutoff_version = snapshot.version as usize;

            // Count events for the graph before compaction
            let graph_event_count = self.events.iter().filter(|(id, _)| *id == graph_id).count();

            // Only compact if we have more events than the snapshot version
            if graph_event_count > cutoff_version {
                // Calculate how many events to keep
                let _events_to_keep = graph_event_count - cutoff_version;

                // Collect indices of events to remove
                let mut remove_count = 0;
                let indices_to_remove: Vec<usize> = self
                    .events
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, (id, _))| {
                        if *id == graph_id && remove_count < cutoff_version {
                            remove_count += 1;
                            Some(idx)
                        } else {
                            None
                        }
                    })
                    .collect();

                // Remove events in reverse order to maintain indices
                for idx in indices_to_remove.into_iter().rev() {
                    self.events.remove(idx);
                }
            }
        }
    }
}

impl Nodes {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    /// Index a node's location
    pub fn index_node(&mut self, node_id: NodeIdentity, location: NodeLocation) {
        self.index.insert(node_id, location);
    }

    /// Find where a node is located
    pub fn locate(&self, node_id: NodeIdentity) -> Option<&NodeLocation> {
        self.index.get(&node_id)
    }

    /// Remove a node from the index
    pub fn remove(&mut self, node_id: NodeIdentity) -> Option<NodeLocation> {
        self.index.remove(&node_id)
    }

    /// Find all nodes in a graph
    pub fn nodes_in_graph(&self, graph_id: GraphIdentity) -> Vec<NodeIdentity> {
        self.index
            .iter()
            .filter(|(_, loc)| loc.graph_id == graph_id)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Count total indexed nodes
    pub fn count(&self) -> usize {
        self.index.len()
    }

    /// Check if a node is indexed
    pub fn contains(&self, node_id: NodeIdentity) -> bool {
        self.index.contains_key(&node_id)
    }
}

impl Edges {
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
        }
    }

    /// Add an edge reference
    pub fn add_edge(&mut self, source: NodeIdentity, edge_ref: EdgeReference) {
        self.adjacency.entry(source).or_default().push(edge_ref);
    }

    /// Get all edges from a node
    pub fn edges_from(&self, source: NodeIdentity) -> Vec<&EdgeReference> {
        self.adjacency
            .get(&source)
            .map(|edges| edges.iter().collect())
            .unwrap_or_default()
    }

    /// Remove all edges from a node
    pub fn remove_edges_from(&mut self, source: NodeIdentity) -> Vec<EdgeReference> {
        self.adjacency.remove(&source).unwrap_or_default()
    }

    /// Count outgoing edges from a node
    pub fn out_degree(&self, node: NodeIdentity) -> usize {
        self.adjacency
            .get(&node)
            .map(|edges| edges.len())
            .unwrap_or(0)
    }

    /// Find edges by category
    pub fn edges_by_category(&self, source: NodeIdentity, category: &str) -> Vec<&EdgeReference> {
        self.edges_from(source)
            .into_iter()
            .filter(|edge| edge.category == category)
            .collect()
    }

    /// Check if edge exists between nodes
    pub fn has_edge(&self, source: NodeIdentity, target: NodeIdentity) -> bool {
        self.adjacency
            .get(&source)
            .map(|edges| edges.iter().any(|e| e.target_node == target))
            .unwrap_or(false)
    }

    /// Get all nodes with outgoing edges
    pub fn source_nodes(&self) -> Vec<NodeIdentity> {
        self.adjacency.keys().copied().collect()
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
