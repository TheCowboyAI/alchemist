//! Subgraph-related commands

use crate::domain::value_objects::{GraphId, NodeId, Position3D, SubgraphId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Commands for subgraph operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubgraphCommand {
    /// Create a new subgraph within a graph
    CreateSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        name: String,
        base_position: Position3D,
        metadata: HashMap<String, serde_json::Value>,
    },

    /// Remove a subgraph and all its nodes
    RemoveSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
    },

    /// Move a subgraph to a new position (moves all contained nodes)
    MoveSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        new_position: Position3D,
    },

    /// Add a node to a subgraph
    AddNodeToSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        node_id: NodeId,
        relative_position: Position3D,
    },

    /// Remove a node from a subgraph (node still exists in graph)
    RemoveNodeFromSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        node_id: NodeId,
    },

    /// Move a node between subgraphs
    MoveNodeBetweenSubgraphs {
        graph_id: GraphId,
        node_id: NodeId,
        from_subgraph: SubgraphId,
        to_subgraph: SubgraphId,
        new_relative_position: Position3D,
    },
}

impl SubgraphCommand {
    pub fn command_type(&self) -> &'static str {
        match self {
            SubgraphCommand::CreateSubgraph { .. } => "create_subgraph",
            SubgraphCommand::RemoveSubgraph { .. } => "remove_subgraph",
            SubgraphCommand::MoveSubgraph { .. } => "move_subgraph",
            SubgraphCommand::AddNodeToSubgraph { .. } => "add_node_to_subgraph",
            SubgraphCommand::RemoveNodeFromSubgraph { .. } => "remove_node_from_subgraph",
            SubgraphCommand::MoveNodeBetweenSubgraphs { .. } => "move_node_between_subgraphs",
        }
    }
}
