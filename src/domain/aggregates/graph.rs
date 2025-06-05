//! Graph Aggregate - The core domain model

use petgraph::stable_graph::StableGraph;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::domain::{
    events::{DomainEvent, GraphEvent},
    value_objects::*,
};

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
}

pub type Result<T> = std::result::Result<T, GraphError>;

/// The Graph aggregate root
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub id: GraphId,
    pub metadata: GraphMetadata,
    pub version: u64,

    // Petgraph for efficient graph operations
    #[serde(skip)]
    #[allow(dead_code)]
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

    /// Emit an event
    fn emit_event(&mut self, event: DomainEvent) {
        self.apply_event(&event);
        self.uncommitted_events.push(event);
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
            DomainEvent::Node(_node_event) => {
                // TODO: Handle node events
            }
            DomainEvent::Edge(_edge_event) => {
                // TODO: Handle edge events
            }
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
        let description = Some("Test Description".to_string());

        // When
        let graph = Graph::new(id, name.clone(), description.clone());

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
                assert!(metadata.tags.contains(&description.unwrap()));
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
