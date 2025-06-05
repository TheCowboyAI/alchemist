//! Components for graph visualization

use crate::domain::value_objects::{EdgeId, GraphId, NodeId};
use bevy::prelude::*;

/// Component marking an entity as a graph node
#[derive(Component)]
pub struct GraphNode {
    pub node_id: NodeId,
    pub graph_id: GraphId,
}

/// Component marking an entity as a graph edge
#[derive(Component)]
pub struct GraphEdge {
    pub edge_id: EdgeId,
    pub graph_id: GraphId,
    pub source: NodeId,
    pub target: NodeId,
}

/// Component for node labels
#[derive(Component)]
pub struct NodeLabel {
    pub text: String,
}

/// Component marking the graph container
#[derive(Component)]
pub struct GraphContainer {
    pub graph_id: GraphId,
    pub name: String,
}
