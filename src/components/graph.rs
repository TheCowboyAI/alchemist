//! Graph components for the editor

use bevy::prelude::*;
use crate::value_objects::{NodeId, EdgeId, GraphId};

/// Node entity component
#[derive(Component, Debug, Clone)]
pub struct NodeEntity {
    /// Node ID
    pub node_id: NodeId,
    /// Graph ID this node belongs to
    pub graph_id: GraphId,
}

/// Edge entity component
#[derive(Component, Debug, Clone)]
pub struct EdgeEntity {
    /// Edge ID
    pub edge_id: EdgeId,
    /// Source node ID
    pub source: NodeId,
    /// Target node ID
    pub target: NodeId,
} 