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

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub id: String,
    pub name: String,
    pub node_type: String,
}

#[derive(Debug, Clone)]
pub struct EdgeInfo {
    pub source: String,
    pub target: String,
}

impl GraphState {
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn selected_node(&self) -> Option<&NodeInfo> {
        self.selected_node.and_then(|idx| self.nodes.get(idx))
    }

    pub fn add_annotation(&mut self, annotation: String) {
        self.annotations.push(annotation);
    }
} 