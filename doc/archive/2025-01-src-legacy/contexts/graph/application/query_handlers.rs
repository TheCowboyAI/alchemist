//! Graph query handlers
//!
//! Query handlers provide read-only access to graph data through
//! optimized read models and projections.

use crate::shared::types::{GraphId, NodeId, Result};
use async_trait::async_trait;

/// Graph query handler for read operations
pub struct GraphQueryHandler {
    // Will be implemented with read model dependencies
}

/// Query for finding graphs
pub struct FindGraphQuery {
    pub graph_id: GraphId,
}

/// Query for finding nodes in a graph
pub struct FindNodesQuery {
    pub graph_id: GraphId,
    pub node_type: Option<String>,
}

#[async_trait]
pub trait QueryHandler<Q, R>: Send + Sync {
    async fn handle(&self, query: Q) -> Result<R>;
}
