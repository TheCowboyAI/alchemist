//! Graph state management for the application

use bevy::prelude::*;

/// Resource that tracks the current graph state
#[derive(Resource, Default)]
pub struct GraphState {
    nodes: Vec<NodeInfo>,
    edges: Vec<EdgeInfo>,
    selected_node: Option<usize>,
    annotations: Vec<String>,
}

/// Information about a node in the graph
#[derive(Debug, Clone)]
pub struct NodeInfo {
    /// Unique identifier for the node
    pub id: String,
    /// Display name of the node
    pub name: String,
    /// Type of the node (e.g., "concept", "workflow", "event")
    pub node_type: String,
}

/// Information about an edge connecting two nodes
#[derive(Debug, Clone)]
pub struct EdgeInfo {
    /// ID of the source node
    pub source: String,
    /// ID of the target node
    pub target: String,
}

impl GraphState {
    /// Returns the total number of nodes in the graph
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the total number of edges in the graph
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Returns information about the currently selected node, if any
    pub fn selected_node(&self) -> Option<&NodeInfo> {
        self.selected_node.and_then(|idx| self.nodes.get(idx))
    }

    /// Adds an annotation to the graph
    pub fn add_annotation(&mut self, annotation: String) {
        self.annotations.push(annotation);
    }
}
