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
