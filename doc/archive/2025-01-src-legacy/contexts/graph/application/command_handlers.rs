//! Graph command handlers
//!
//! Command handlers orchestrate the processing of commands by loading
//! aggregates, executing commands, and persisting events.

use crate::contexts::graph::domain::ContextGraph;
use crate::contexts::graph::domain::commands::{CommandHandler, CreateGraph, GraphFactory};
use crate::contexts::graph::domain::events::GraphCreated;
use crate::shared::events::{DomainEvent, EventStore};
use crate::shared::types::{GraphId, Result};
use async_trait::async_trait;
use std::sync::Arc;

/// Graph command handler with injected dependencies
pub struct GraphCommandHandler {
    event_store: Arc<dyn EventStore>,
    graph_factory: Arc<dyn GraphFactory>,
    graph_repository: Arc<dyn GraphRepository>,
}

impl GraphCommandHandler {
    pub fn new(
        event_store: Arc<dyn EventStore>,
        graph_factory: Arc<dyn GraphFactory>,
        graph_repository: Arc<dyn GraphRepository>,
    ) -> Self {
        Self {
            event_store,
            graph_factory,
            graph_repository,
        }
    }
}

/// Repository trait for loading/saving graphs
#[async_trait]
pub trait GraphRepository: Send + Sync {
    /// Load a graph by ID
    async fn load(&self, id: GraphId) -> Result<Option<ContextGraph>>;

    /// Save a graph
    async fn save(&self, graph: &ContextGraph) -> Result<()>;
}

#[async_trait]
impl CommandHandler<CreateGraph> for GraphCommandHandler {
    fn handle(&self, command: CreateGraph) -> Result<Vec<Box<dyn DomainEvent>>> {
        // Use the factory to create the graph with proper dependencies
        let graph = self.graph_factory.create_graph(command.clone())?;

        // Generate creation event
        let event = GraphCreated {
            graph_id: command.graph_id,
            name: command.name,
            context_type: command.context_type,
            context_root: command.root_node_id,
            event_metadata: crate::shared::events::EventMetadata::new(),
        };

        Ok(vec![Box::new(event)])
    }
}
