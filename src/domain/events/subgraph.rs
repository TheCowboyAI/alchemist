//! Subgraph-related domain events

use serde::{Deserialize, Serialize};
use crate::domain::value_objects::{GraphId, SubgraphId, NodeId, Position3D};

/// Events related to subgraph operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubgraphEvent {
    /// A new subgraph was created within a graph
    SubgraphCreated {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        name: String,
        base_position: Position3D,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    },

    /// A subgraph was removed from a graph
    SubgraphRemoved {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
    },

    /// A subgraph was moved to a new position
    SubgraphMoved {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        old_position: Position3D,
        new_position: Position3D,
    },

    /// A node was added to a subgraph
    NodeAddedToSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        node_id: NodeId,
        relative_position: Position3D,
    },

    /// A node was removed from a subgraph
    NodeRemovedFromSubgraph {
        graph_id: GraphId,
        subgraph_id: SubgraphId,
        node_id: NodeId,
    },
}
