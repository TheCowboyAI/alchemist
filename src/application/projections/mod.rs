//! Projections (Read Models) - These ARE Resources

pub mod graph_summary;
pub mod projection_handler;
pub mod external;

pub use graph_summary::{GraphSummaryProjection, GraphSummary, Projection};
pub use projection_handler::{ProjectionHandler, ProjectionPlugin};
pub use external::{ExternalProjection, IngestHandler, BidirectionalEventManager};

use crate::domain::value_objects::{
    EdgeId, EdgeRelationship, GraphId, NodeContent, NodeId, Position3D,
};
use bevy::prelude::*;
use std::collections::HashMap;

/// Graph projection - the current state of all graphs
#[derive(Resource, Default)]
pub struct GraphProjection {
    pub graphs: HashMap<GraphId, GraphState>,
}

/// Current state of a single graph
#[derive(Debug, Clone)]
pub struct GraphState {
    pub id: GraphId,
    pub name: String,
    pub description: Option<String>,
    pub nodes: HashMap<NodeId, NodeState>,
    pub edges: HashMap<EdgeId, EdgeState>,
}

/// Current state of a node
#[derive(Debug, Clone)]
pub struct NodeState {
    pub id: NodeId,
    pub position: Position3D,
    pub content: NodeContent,
}

/// Current state of an edge
#[derive(Debug, Clone)]
pub struct EdgeState {
    pub id: EdgeId,
    pub source_id: NodeId,
    pub target_id: NodeId,
    pub relationship: EdgeRelationship,
}

impl GraphProjection {
    pub fn new() -> Self {
        Self {
            graphs: HashMap::new(),
        }
    }

    pub fn get_graph(&self, id: &GraphId) -> Option<&GraphState> {
        self.graphs.get(id)
    }

    pub fn get_graph_mut(&mut self, id: &GraphId) -> Option<&mut GraphState> {
        self.graphs.get_mut(id)
    }
}
