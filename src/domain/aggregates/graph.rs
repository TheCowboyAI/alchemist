//! Graph Aggregate - The core domain model

use petgraph::stable_graph::StableGraph;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::domain::{
    commands::{Command, EdgeCommand, GraphCommand, NodeCommand},
    events::{DomainEvent, EdgeEvent, GraphEvent, NodeEvent},
    value_objects::*,
};
use serde_json;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Graph not found: {0}")]
    GraphNotFound(GraphId),

    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),

    #[error("Edge not found: {0}")]
    EdgeNotFound(EdgeId),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Node already exists: {0}")]
    NodeAlreadyExists(NodeId),

    #[error("Edge already exists: {0}")]
    EdgeAlreadyExists(EdgeId),

    #[error("Self-loop not allowed: node {0} cannot connect to itself")]
    SelfLoopNotAllowed(NodeId),

    #[error("Duplicate edge: edge already exists between {0} and {1}")]
    DuplicateEdge(NodeId, NodeId),

    #[error("Graph is at maximum capacity: {0} nodes")]
    GraphAtCapacity(usize),

    #[error("Invalid node position: position must be finite")]
    InvalidNodePosition,

    #[error("Cascade delete would remove {0} edges")]
    CascadeDeleteWarning(usize),
}

pub type Result<T> = std::result::Result<T, GraphError>;

/// Maximum number of nodes allowed in a graph
const MAX_NODES: usize = 10_000;

/// Maximum number of edges allowed in a graph
const MAX_EDGES: usize = 100_000;

/// The Graph aggregate root
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub id: GraphId,
    pub metadata: GraphMetadata,
    pub version: u64,

    // Petgraph for efficient graph operations
    #[serde(skip)]
    graph: StableGraph<NodeId, EdgeId>,

    // Component storage
    nodes: HashMap<NodeId, Node>,
    edges: HashMap<EdgeId, Edge>,

    // Indices for fast lookups
    node_indices: HashMap<NodeId, petgraph::graph::NodeIndex>,
    edge_indices: HashMap<EdgeId, petgraph::graph::EdgeIndex>,

    // Event sourcing
    #[serde(skip)]
    uncommitted_events: Vec<DomainEvent>,
}

/// Node entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub content: NodeContent,
    pub position: Position3D,
}

/// Edge entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub relationship: EdgeRelationship,
}

impl Graph {
    /// Create a new graph
    pub fn new(id: GraphId, name: String, description: Option<String>) -> Self {
        let mut metadata = GraphMetadata::new(name);
        if let Some(desc) = description {
            metadata.tags.push(desc);
        }

        let mut graph = Self {
            id,
            metadata: metadata.clone(),
            version: 0,
            graph: StableGraph::new(),
            nodes: HashMap::new(),
            edges: HashMap::new(),
            node_indices: HashMap::new(),
            edge_indices: HashMap::new(),
            uncommitted_events: Vec::new(),
        };

        // Emit creation event
        graph.emit_event(DomainEvent::Graph(GraphEvent::GraphCreated {
            id,
            metadata,
        }));

        // Reset version to 0 after creation
        graph.version = 0;

        graph
    }

    /// Rebuild graph from events
    pub fn from_events(id: GraphId, events: Vec<DomainEvent>) -> Self {
        let mut graph = Self {
            id,
            metadata: GraphMetadata::new(String::new()),
            version: 0,
            graph: StableGraph::new(),
            nodes: HashMap::new(),
            edges: HashMap::new(),
            node_indices: HashMap::new(),
            edge_indices: HashMap::new(),
            uncommitted_events: Vec::new(),
        };

        for event in events {
            graph.apply_event(&event);
        }

        graph
    }

    /// Handle commands - main entry point for all operations
    pub fn handle_command(&mut self, command: Command) -> Result<Vec<DomainEvent>> {
        match command {
            Command::Graph(cmd) => self.handle_graph_command(cmd),
            Command::Node(cmd) => self.handle_node_command(cmd),
            Command::Edge(cmd) => self.handle_edge_command(cmd),
        }
    }

    /// Handle graph-level commands
    fn handle_graph_command(&mut self, command: GraphCommand) -> Result<Vec<DomainEvent>> {
        match command {
            GraphCommand::CreateGraph { .. } => {
                Err(GraphError::InvalidOperation("Graph already exists".to_string()))
            }
            GraphCommand::DeleteGraph { id } => {
                if id != self.id {
                    return Err(GraphError::GraphNotFound(id));
                }
                self.emit_event(DomainEvent::Graph(GraphEvent::GraphDeleted { id }));
                Ok(self.get_uncommitted_events())
            }
            GraphCommand::RenameGraph { id, new_name } => {
                if id != self.id {
                    return Err(GraphError::GraphNotFound(id));
                }
                let old_name = self.metadata.name.clone();
                self.emit_event(DomainEvent::Graph(GraphEvent::GraphRenamed {
                    id,
                    old_name,
                    new_name,
                }));
                Ok(self.get_uncommitted_events())
            }
            GraphCommand::TagGraph { id, tag } => {
                if id != self.id {
                    return Err(GraphError::GraphNotFound(id));
                }
                self.emit_event(DomainEvent::Graph(GraphEvent::GraphTagged { id, tag }));
                Ok(self.get_uncommitted_events())
            }
            GraphCommand::UntagGraph { id, tag } => {
                if id != self.id {
                    return Err(GraphError::GraphNotFound(id));
                }
                if !self.metadata.tags.contains(&tag) {
                    return Err(GraphError::InvalidOperation(format!("Tag '{}' not found", tag)));
                }
                self.emit_event(DomainEvent::Graph(GraphEvent::GraphUntagged { id, tag }));
                Ok(self.get_uncommitted_events())
            }
        }
    }

    /// Handle node-level commands
    fn handle_node_command(&mut self, command: NodeCommand) -> Result<Vec<DomainEvent>> {
        match command {
            NodeCommand::AddNode { graph_id, node_id, content, position } => {
                if graph_id != self.id {
                    return Err(GraphError::GraphNotFound(graph_id));
                }

                // Validate business rules
                if self.nodes.contains_key(&node_id) {
                    return Err(GraphError::NodeAlreadyExists(node_id));
                }

                if self.nodes.len() >= MAX_NODES {
                    return Err(GraphError::GraphAtCapacity(MAX_NODES));
                }

                                if !position.is_finite() {
                    return Err(GraphError::InvalidNodePosition);
                }

                // Convert NodeContent to metadata HashMap
                let mut metadata = std::collections::HashMap::new();
                metadata.insert("label".to_string(), serde_json::json!(content.label));
                metadata.insert("node_type".to_string(), serde_json::to_value(&content.node_type).unwrap());
                for (k, v) in content.properties {
                    metadata.insert(k, v);
                }

                self.emit_event(DomainEvent::Node(NodeEvent::NodeAdded {
                    graph_id,
                    node_id,
                    metadata,
                    position,
                }));
                Ok(self.get_uncommitted_events())
            }
            NodeCommand::RemoveNode { graph_id, node_id } => {
                if graph_id != self.id {
                    return Err(GraphError::GraphNotFound(graph_id));
                }

                if !self.nodes.contains_key(&node_id) {
                    return Err(GraphError::NodeNotFound(node_id));
                }

                // Find all edges connected to this node
                let connected_edges: Vec<EdgeId> = self.edges
                    .iter()
                    .filter(|(_, edge)| edge.source == node_id || edge.target == node_id)
                    .map(|(id, _)| *id)
                    .collect();

                // Emit edge removal events first (cascade delete)
                for edge_id in connected_edges {
                    self.emit_event(DomainEvent::Edge(EdgeEvent::EdgeRemoved {
                        graph_id,
                        edge_id,
                    }));
                }

                // Then emit node removal event
                self.emit_event(DomainEvent::Node(NodeEvent::NodeRemoved {
                    graph_id,
                    node_id,
                }));

                Ok(self.get_uncommitted_events())
            }
            NodeCommand::UpdateNode { graph_id, node_id, content } => {
                if graph_id != self.id {
                    return Err(GraphError::GraphNotFound(graph_id));
                }

                                if !self.nodes.contains_key(&node_id) {
                    return Err(GraphError::NodeNotFound(node_id));
                }

                // Get position before removal
                let position = self.nodes[&node_id].position.clone();

                // Following DDD principles: remove old, add new
                self.emit_event(DomainEvent::Node(NodeEvent::NodeRemoved {
                    graph_id,
                    node_id,
                }));

                // Convert NodeContent to metadata HashMap
                let mut metadata = std::collections::HashMap::new();
                metadata.insert("label".to_string(), serde_json::json!(content.label));
                metadata.insert("node_type".to_string(), serde_json::to_value(&content.node_type).unwrap());
                for (k, v) in content.properties {
                    metadata.insert(k, v);
                }

                self.emit_event(DomainEvent::Node(NodeEvent::NodeAdded {
                    graph_id,
                    node_id,
                    metadata,
                    position,
                }));

                Ok(self.get_uncommitted_events())
            }
            NodeCommand::MoveNode { graph_id, node_id, position } => {
                if graph_id != self.id {
                    return Err(GraphError::GraphNotFound(graph_id));
                }

                if !self.nodes.contains_key(&node_id) {
                    return Err(GraphError::NodeNotFound(node_id));
                }

                if !position.is_finite() {
                    return Err(GraphError::InvalidNodePosition);
                }

                                // Following DDD principles: remove old, add new
                let content = self.nodes[&node_id].content.clone();

                self.emit_event(DomainEvent::Node(NodeEvent::NodeRemoved {
                    graph_id,
                    node_id,
                }));

                // Convert NodeContent to metadata HashMap
                let mut metadata = std::collections::HashMap::new();
                metadata.insert("label".to_string(), serde_json::json!(content.label));
                metadata.insert("node_type".to_string(), serde_json::to_value(&content.node_type).unwrap());
                for (k, v) in content.properties {
                    metadata.insert(k, v);
                }

                self.emit_event(DomainEvent::Node(NodeEvent::NodeAdded {
                    graph_id,
                    node_id,
                    metadata,
                    position,
                }));

                Ok(self.get_uncommitted_events())
            }
            NodeCommand::SelectNode { graph_id, node_id } => {
                if graph_id != self.id {
                    return Err(GraphError::GraphNotFound(graph_id));
                }

                if !self.nodes.contains_key(&node_id) {
                    return Err(GraphError::NodeNotFound(node_id));
                }

                self.emit_event(DomainEvent::Node(NodeEvent::NodeSelected {
                    graph_id,
                    node_id,
                }));

                Ok(self.get_uncommitted_events())
            }
            NodeCommand::DeselectNode { graph_id, node_id } => {
                if graph_id != self.id {
                    return Err(GraphError::GraphNotFound(graph_id));
                }

                if !self.nodes.contains_key(&node_id) {
                    return Err(GraphError::NodeNotFound(node_id));
                }

                self.emit_event(DomainEvent::Node(NodeEvent::NodeDeselected {
                    graph_id,
                    node_id,
                }));

                Ok(self.get_uncommitted_events())
            }
        }
    }

    /// Handle edge-level commands
    fn handle_edge_command(&mut self, command: EdgeCommand) -> Result<Vec<DomainEvent>> {
        match command {
            EdgeCommand::ConnectEdge { graph_id, edge_id, source, target, relationship } => {
                if graph_id != self.id {
                    return Err(GraphError::GraphNotFound(graph_id));
                }

                // Validate business rules
                if self.edges.contains_key(&edge_id) {
                    return Err(GraphError::EdgeAlreadyExists(edge_id));
                }

                if !self.nodes.contains_key(&source) {
                    return Err(GraphError::NodeNotFound(source));
                }

                if !self.nodes.contains_key(&target) {
                    return Err(GraphError::NodeNotFound(target));
                }

                if source == target {
                    return Err(GraphError::SelfLoopNotAllowed(source));
                }

                // Check for duplicate edges
                let duplicate_exists = self.edges.values().any(|edge| {
                    (edge.source == source && edge.target == target) ||
                    (edge.source == target && edge.target == source)
                });

                if duplicate_exists {
                    return Err(GraphError::DuplicateEdge(source, target));
                }

                                if self.edges.len() >= MAX_EDGES {
                    return Err(GraphError::GraphAtCapacity(MAX_EDGES));
                }

                self.emit_event(DomainEvent::Edge(EdgeEvent::EdgeConnected {
                    graph_id,
                    edge_id,
                    source,
                    target,
                    relationship,
                }));

                Ok(self.get_uncommitted_events())
            }
            EdgeCommand::DisconnectEdge { graph_id, edge_id } => {
                if graph_id != self.id {
                    return Err(GraphError::GraphNotFound(graph_id));
                }

                if !self.edges.contains_key(&edge_id) {
                    return Err(GraphError::EdgeNotFound(edge_id));
                }

                self.emit_event(DomainEvent::Edge(EdgeEvent::EdgeRemoved {
                    graph_id,
                    edge_id,
                }));

                Ok(self.get_uncommitted_events())
            }
            EdgeCommand::SelectEdge { graph_id, edge_id } => {
                if graph_id != self.id {
                    return Err(GraphError::GraphNotFound(graph_id));
                }

                if !self.edges.contains_key(&edge_id) {
                    return Err(GraphError::EdgeNotFound(edge_id));
                }

                self.emit_event(DomainEvent::Edge(EdgeEvent::EdgeSelected {
                    graph_id,
                    edge_id,
                }));

                Ok(self.get_uncommitted_events())
            }
            EdgeCommand::DeselectEdge { graph_id, edge_id } => {
                if graph_id != self.id {
                    return Err(GraphError::GraphNotFound(graph_id));
                }

                if !self.edges.contains_key(&edge_id) {
                    return Err(GraphError::EdgeNotFound(edge_id));
                }

                self.emit_event(DomainEvent::Edge(EdgeEvent::EdgeDeselected {
                    graph_id,
                    edge_id,
                }));

                Ok(self.get_uncommitted_events())
            }
        }
    }

    /// Update graph metadata
    pub fn update_metadata(&mut self, name: Option<String>, description: Option<String>) {
        if let Some(new_name) = name {
            let old_name = self.metadata.name.clone();
            self.emit_event(DomainEvent::Graph(GraphEvent::GraphRenamed {
                id: self.id,
                old_name,
                new_name,
            }));
        }

        if let Some(desc) = description {
            self.emit_event(DomainEvent::Graph(GraphEvent::GraphTagged {
                id: self.id,
                tag: desc,
            }));
        }
    }

    /// Get uncommitted events
    pub fn get_uncommitted_events(&self) -> Vec<DomainEvent> {
        self.uncommitted_events.clone()
    }

    /// Clear uncommitted events (after they've been persisted)
    pub fn mark_events_as_committed(&mut self) {
        self.uncommitted_events.clear();
    }

    /// Get node count (for testing)
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get edge count (for testing)
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Emit an event
    fn emit_event(&mut self, event: DomainEvent) {
        self.apply_event(&event);
        self.uncommitted_events.push(event);
        self.version += 1;
    }

    /// Apply an event to update state
    fn apply_event(&mut self, event: &DomainEvent) {
        match event {
            DomainEvent::Graph(graph_event) => match graph_event {
                GraphEvent::GraphCreated { id: _, metadata } => {
                    self.metadata = metadata.clone();
                }
                GraphEvent::GraphRenamed {
                    id: _,
                    old_name: _,
                    new_name,
                } => {
                    self.metadata.name = new_name.clone();
                }
                GraphEvent::GraphTagged { id: _, tag } => {
                    self.metadata.tags.push(tag.clone());
                }
                GraphEvent::GraphUntagged { id: _, tag } => {
                    self.metadata.tags.retain(|t| t != tag);
                }
                GraphEvent::GraphDeleted { id: _ } => {
                    // Mark as deleted
                }
            },
            DomainEvent::Node(node_event) => match node_event {
                NodeEvent::NodeAdded { graph_id: _, node_id, metadata, position } => {
                    // Convert metadata HashMap back to NodeContent
                    let label = metadata.get("label")
                        .and_then(|v| v.as_str())
                        .unwrap_or("").to_string();

                    let node_type = metadata.get("node_type")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or(NodeType::Custom("Unknown".to_string()));

                    let mut properties = metadata.clone();
                    properties.remove("label");
                    properties.remove("node_type");

                    let content = NodeContent {
                        label,
                        node_type,
                        properties,
                    };

                    let node = Node {
                        id: *node_id,
                        content,
                        position: position.clone(),
                    };

                    // Add to petgraph
                    let idx = self.graph.add_node(*node_id);
                    self.node_indices.insert(*node_id, idx);

                    // Add to storage
                    self.nodes.insert(*node_id, node);
                }
                NodeEvent::NodeRemoved { graph_id: _, node_id } => {
                    // Remove from petgraph
                    if let Some(idx) = self.node_indices.remove(node_id) {
                        self.graph.remove_node(idx);
                    }

                    // Remove from storage
                    self.nodes.remove(node_id);
                }
                NodeEvent::NodeSelected { .. } | NodeEvent::NodeDeselected { .. } | NodeEvent::NodeMetadataUpdated { .. } => {
                    // Selection state and metadata updates are handled in presentation layer
                }
            },
            DomainEvent::Edge(edge_event) => match edge_event {
                EdgeEvent::EdgeConnected { graph_id: _, edge_id, source, target, relationship } => {
                    let edge = Edge {
                        id: *edge_id,
                        source: *source,
                        target: *target,
                        relationship: relationship.clone(),
                    };

                    // Add to petgraph
                    if let (Some(&source_idx), Some(&target_idx)) =
                        (self.node_indices.get(source), self.node_indices.get(target)) {
                        let idx = self.graph.add_edge(source_idx, target_idx, *edge_id);
                        self.edge_indices.insert(*edge_id, idx);
                    }

                    // Add to storage
                    self.edges.insert(*edge_id, edge);
                }
                EdgeEvent::EdgeRemoved { graph_id: _, edge_id } => {
                    // Remove from petgraph
                    if let Some(idx) = self.edge_indices.remove(edge_id) {
                        self.graph.remove_edge(idx);
                    }

                    // Remove from storage
                    self.edges.remove(edge_id);
                }
                EdgeEvent::EdgeAdded { graph_id: _, edge_id, source, target, metadata: _ } => {
                    // EdgeAdded is used for simple edge creation without relationship details
                    let edge = Edge {
                        id: *edge_id,
                        source: *source,
                        target: *target,
                        relationship: EdgeRelationship {
                            relationship_type: RelationshipType::Custom("Default".to_string()),
                            properties: std::collections::HashMap::new(),
                            bidirectional: false,
                        },
                    };

                    // Add to petgraph
                    if let (Some(&source_idx), Some(&target_idx)) =
                        (self.node_indices.get(source), self.node_indices.get(target)) {
                        let idx = self.graph.add_edge(source_idx, target_idx, *edge_id);
                        self.edge_indices.insert(*edge_id, idx);
                    }

                    // Add to storage
                    self.edges.insert(*edge_id, edge);
                }
                EdgeEvent::EdgeDisconnected { .. } => {
                    // Handle disconnection if needed
                }
                EdgeEvent::EdgeSelected { .. } | EdgeEvent::EdgeDeselected { .. } | EdgeEvent::EdgeMetadataUpdated { .. } => {
                    // Selection state and metadata updates are handled in presentation layer
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        // Given
        let id = GraphId::new();
        let name = "Test Graph".to_string();
        let description = "Test Description".to_string();

        // When
        let graph = Graph::new(id, name.clone(), Some(description.clone()));

        // Then
        assert_eq!(graph.id, id);
        assert_eq!(graph.metadata.name, name);
        assert_eq!(graph.version, 0);
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.edges.len(), 0);

        // Verify creation event was emitted
        let events = graph.get_uncommitted_events();
        assert_eq!(events.len(), 1);

        match &events[0] {
            DomainEvent::Graph(GraphEvent::GraphCreated { id: event_id, metadata }) => {
                assert_eq!(*event_id, id);
                assert_eq!(metadata.name, name);
                assert!(metadata.tags.contains(&description));
            }
            _ => panic!("Expected GraphCreated event"),
        }
    }

    #[test]
    fn test_graph_metadata_update() {
        // Given
        let id = GraphId::new();
        let mut graph = Graph::new(id, "Original Name".to_string(), None);
        graph.mark_events_as_committed(); // Clear creation event

        // When
        graph.update_metadata(Some("New Name".to_string()), Some("New Tag".to_string()));

        // Then
        assert_eq!(graph.metadata.name, "New Name");
        assert!(graph.metadata.tags.contains(&"New Tag".to_string()));

        // Verify events were emitted
        let events = graph.get_uncommitted_events();
        assert_eq!(events.len(), 2);

        // Check rename event
        match &events[0] {
            DomainEvent::Graph(GraphEvent::GraphRenamed { id: event_id, old_name, new_name }) => {
                assert_eq!(*event_id, id);
                assert_eq!(old_name, "Original Name");
                assert_eq!(new_name, "New Name");
            }
            _ => panic!("Expected GraphRenamed event"),
        }

        // Check tag event
        match &events[1] {
            DomainEvent::Graph(GraphEvent::GraphTagged { id: event_id, tag }) => {
                assert_eq!(*event_id, id);
                assert_eq!(tag, "New Tag");
            }
            _ => panic!("Expected GraphTagged event"),
        }
    }

    #[test]
    fn test_graph_from_events() {
        // Given
        let id = GraphId::new();
        let events = vec![
            DomainEvent::Graph(GraphEvent::GraphCreated {
                id,
                metadata: GraphMetadata::new("Event Sourced Graph".to_string()),
            }),
            DomainEvent::Graph(GraphEvent::GraphRenamed {
                id,
                old_name: "Event Sourced Graph".to_string(),
                new_name: "Renamed Graph".to_string(),
            }),
            DomainEvent::Graph(GraphEvent::GraphTagged {
                id,
                tag: "Important".to_string(),
            }),
        ];

        // When
        let graph = Graph::from_events(id, events);

        // Then
        assert_eq!(graph.id, id);
        assert_eq!(graph.metadata.name, "Renamed Graph");
        assert!(graph.metadata.tags.contains(&"Important".to_string()));
        assert_eq!(graph.uncommitted_events.len(), 0); // No new events
    }

    #[test]
    fn test_event_commit_cycle() {
        // Given
        let id = GraphId::new();
        let mut graph = Graph::new(id, "Test Graph".to_string(), None);

        // When - verify uncommitted events exist
        assert_eq!(graph.get_uncommitted_events().len(), 1);

        // Mark as committed
        graph.mark_events_as_committed();

        // Then - verify events are cleared
        assert_eq!(graph.get_uncommitted_events().len(), 0);

        // When - make another change
        graph.update_metadata(Some("Updated".to_string()), None);

        // Then - verify new event is tracked
        assert_eq!(graph.get_uncommitted_events().len(), 1);
    }

    #[test]
    fn test_graph_tag_operations() {
        // Given
        let id = GraphId::new();
        let mut graph = Graph::new(id, "Tagged Graph".to_string(), None);
        graph.mark_events_as_committed();

        // When - add multiple tags
        graph.update_metadata(None, Some("Tag1".to_string()));
        graph.update_metadata(None, Some("Tag2".to_string()));

        // Then
        assert_eq!(graph.metadata.tags.len(), 2);
        assert!(graph.metadata.tags.contains(&"Tag1".to_string()));
        assert!(graph.metadata.tags.contains(&"Tag2".to_string()));

        // Verify events
        let events = graph.get_uncommitted_events();
        assert_eq!(events.len(), 2);

        for event in events {
            match event {
                DomainEvent::Graph(GraphEvent::GraphTagged { .. }) => {},
                _ => panic!("Expected GraphTagged events"),
            }
        }
    }

    #[test]
    fn test_graph_untag_operation() {
        // Given
        let id = GraphId::new();
        let events = vec![
            DomainEvent::Graph(GraphEvent::GraphCreated {
                id,
                metadata: GraphMetadata::new("Graph".to_string()),
            }),
            DomainEvent::Graph(GraphEvent::GraphTagged {
                id,
                tag: "ToRemove".to_string(),
            }),
            DomainEvent::Graph(GraphEvent::GraphTagged {
                id,
                tag: "ToKeep".to_string(),
            }),
            DomainEvent::Graph(GraphEvent::GraphUntagged {
                id,
                tag: "ToRemove".to_string(),
            }),
        ];

        // When
        let graph = Graph::from_events(id, events);

        // Then
        assert_eq!(graph.metadata.tags.len(), 1);
        assert!(graph.metadata.tags.contains(&"ToKeep".to_string()));
        assert!(!graph.metadata.tags.contains(&"ToRemove".to_string()));
    }
}
