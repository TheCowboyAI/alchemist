//! Graph command handler for async/await context
//!
//! This handler processes commands through the domain aggregate
//! and persists events to the event store.

use crate::domain::aggregates::Graph;
use crate::domain::commands::{Command, GraphCommand, NodeCommand, EdgeCommand};
use crate::domain::events::DomainEvent;
use crate::infrastructure::event_store::EventStore;
use std::sync::Arc;


/// Command handler error type
#[derive(Debug, thiserror::Error)]
pub enum CommandHandlerError {
    #[error("Event store error: {0}")]
    EventStore(#[from] crate::infrastructure::event_store::EventStoreError),

    #[error("Graph error: {0}")]
    Graph(#[from] crate::domain::aggregates::graph::GraphError),

    #[error("Aggregate not found")]
    AggregateNotFound,

    #[error("Other error: {0}")]
    Other(String),
}

impl From<Box<dyn std::error::Error + Send + Sync>> for CommandHandlerError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        CommandHandlerError::Other(err.to_string())
    }
}

/// Trait for command handlers
#[async_trait::async_trait]
pub trait CommandHandler: Send + Sync {
    type Command;
    type Error: std::error::Error + Send + Sync + 'static;

    async fn handle(&self, command: Self::Command) -> Result<Vec<DomainEvent>, Self::Error>;
}

/// Graph command handler that uses event sourcing
pub struct GraphCommandHandler {
    event_store: Arc<dyn EventStore>,
}

impl GraphCommandHandler {
    pub fn new(event_store: Arc<dyn EventStore>) -> Self {
        Self { event_store }
    }

    async fn load_or_create_aggregate(&self, graph_id: crate::domain::value_objects::GraphId) -> Result<Graph, CommandHandlerError> {
        // Try to load existing aggregate
        let events = self.event_store.get_events(graph_id.to_string()).await?;

        let aggregate = if events.is_empty() {
            // This shouldn't happen for existing aggregates
            return Err(CommandHandlerError::AggregateNotFound);
        } else {
            // Rebuild from events
            Graph::from_events(graph_id, events)
        };

        Ok(aggregate)
    }
}

#[async_trait::async_trait]
impl CommandHandler for GraphCommandHandler {
    type Command = Command;
    type Error = CommandHandlerError;

    async fn handle(&self, command: Self::Command) -> Result<Vec<DomainEvent>, Self::Error> {
        match command {
            Command::Graph(graph_cmd) => self.handle_graph_command(graph_cmd).await,
            Command::Node(node_cmd) => self.handle_node_command(node_cmd).await,
            Command::Edge(edge_cmd) => self.handle_edge_command(edge_cmd).await,
        }
    }
}

impl GraphCommandHandler {
    async fn handle_graph_command(&self, command: GraphCommand) -> Result<Vec<DomainEvent>, CommandHandlerError> {
        match command {
            GraphCommand::CreateGraph { id, name } => {
                // For create, we start with a new aggregate
                let mut aggregate = Graph::new(id, name.clone(), None);
                let events = aggregate.handle_command(Command::Graph(GraphCommand::CreateGraph { id, name }))?;

                // Store events
                self.event_store.append_events(id.to_string(), events.clone()).await?;

                Ok(events)
            }
            _ => {
                // For other commands, load the aggregate first
                let graph_id = match &command {
                    GraphCommand::RenameGraph { id, .. } => *id,
                    GraphCommand::TagGraph { id, .. } => *id,
                    GraphCommand::UntagGraph { id, .. } => *id,
                    GraphCommand::DeleteGraph { id } => *id,
                    GraphCommand::CreateGraph { .. } => unreachable!(),
                };

                let mut aggregate = self.load_or_create_aggregate(graph_id).await?;
                let events = aggregate.handle_command(Command::Graph(command))?;

                // Store events
                if !events.is_empty() {
                    self.event_store.append_events(graph_id.to_string(), events.clone()).await?;
                }

                Ok(events)
            }
        }
    }

    async fn handle_node_command(&self, command: NodeCommand) -> Result<Vec<DomainEvent>, CommandHandlerError> {
        let graph_id = match &command {
            NodeCommand::AddNode { graph_id, .. } => *graph_id,
            NodeCommand::RemoveNode { graph_id, .. } => *graph_id,
            NodeCommand::UpdateNode { graph_id, .. } => *graph_id,
            NodeCommand::MoveNode { graph_id, .. } => *graph_id,
            NodeCommand::SelectNode { graph_id, .. } => *graph_id,
            NodeCommand::DeselectNode { graph_id, .. } => *graph_id,
        };

        let mut aggregate = self.load_or_create_aggregate(graph_id).await?;
        let events = aggregate.handle_command(Command::Node(command))?;

        // Store events
        if !events.is_empty() {
            self.event_store.append_events(graph_id.to_string(), events.clone()).await?;
        }

        Ok(events)
    }

    async fn handle_edge_command(&self, command: EdgeCommand) -> Result<Vec<DomainEvent>, CommandHandlerError> {
        let graph_id = match &command {
            EdgeCommand::ConnectEdge { graph_id, .. } => *graph_id,
            EdgeCommand::DisconnectEdge { graph_id, .. } => *graph_id,
            EdgeCommand::SelectEdge { graph_id, .. } => *graph_id,
            EdgeCommand::DeselectEdge { graph_id, .. } => *graph_id,
        };

        let mut aggregate = self.load_or_create_aggregate(graph_id).await?;
        let events = aggregate.handle_command(Command::Edge(command))?;

        // Store events
        if !events.is_empty() {
            self.event_store.append_events(graph_id.to_string(), events.clone()).await?;
        }

        Ok(events)
    }
}
