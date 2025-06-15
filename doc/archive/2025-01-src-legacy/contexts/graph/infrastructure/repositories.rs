//! Graph repository implementations
//!
//! Repositories provide persistence for aggregates, typically by
//! replaying events from the event store.

use crate::contexts::graph::application::command_handlers::GraphRepository;
use crate::contexts::graph::domain::ContextGraph;
use crate::shared::types::{GraphId, Result};
use async_trait::async_trait;

/// In-memory graph repository for testing
pub struct InMemoryGraphRepository {
    // Will store graphs in memory
}

#[async_trait]
impl GraphRepository for InMemoryGraphRepository {
    async fn load(&self, id: GraphId) -> Result<Option<ContextGraph>> {
        // TODO: Implement
        Ok(None)
    }

    async fn save(&self, graph: &ContextGraph) -> Result<()> {
        // TODO: Implement
        Ok(())
    }
}
