//! Graph commands - Intent to change state
//!
//! Commands represent user intent and are processed by the aggregate.
//! Each command implements the Command trait for dependency injection.

use crate::shared::types::{GraphId, NodeId, EdgeId, Result};
use crate::shared::events::{EventMetadata, DomainEvent};
use crate::contexts::graph::domain::context_graph::{ContextGraph, Command};
use crate::contexts::graph::domain::events::{GraphEvent, NodeAdded, EdgeAdded};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Create a new graph command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGraph {
    pub graph_id: GraphId,
    pub name: String,
    pub context_type: crate::contexts::graph::domain::context_graph::ContextType,
    pub root_node_id: NodeId,
    pub root_node_type: String,
}

impl Command for CreateGraph {
    fn execute(&self, graph: &mut ContextGraph) -> Result<Vec<Box<dyn DomainEvent>>> {
        // Graph creation is handled at the factory level
        // This command would be processed by a command handler
        Ok(vec![])
    }
}

/// Add a node to the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddNode {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub node_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Command for AddNode {
    fn execute(&self, graph: &mut ContextGraph) -> Result<Vec<Box<dyn DomainEvent>>> {
        // Validate graph ID matches
        if graph.id() != self.graph_id {
            return Err(crate::shared::types::Error::InvalidOperation(
                "Graph ID mismatch".to_string()
            ));
        }

        // Add the node (validation happens inside)
        graph.add_node(self.node_id, self.node_type.clone(), self.metadata.clone())?;

        // Create event
        let event = NodeAdded {
            graph_id: self.graph_id,
            node_id: self.node_id,
            node_type: self.node_type.clone(),
            metadata: self.metadata.clone(),
            event_metadata: EventMetadata::new(),
        };

        Ok(vec![Box::new(event)])
    }
}

/// Connect two nodes with an edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectNodes {
    pub graph_id: GraphId,
    pub edge_id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub edge_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Command for ConnectNodes {
    fn execute(&self, graph: &mut ContextGraph) -> Result<Vec<Box<dyn DomainEvent>>> {
        // Validate graph ID matches
        if graph.id() != self.graph_id {
            return Err(crate::shared::types::Error::InvalidOperation(
                "Graph ID mismatch".to_string()
            ));
        }

        // Add the edge (validation happens inside)
        graph.add_edge(
            self.edge_id,
            self.source,
            self.target,
            self.edge_type.clone(),
            self.metadata.clone()
        )?;

        // Create event
        let event = EdgeAdded {
            graph_id: self.graph_id,
            edge_id: self.edge_id,
            source: self.source,
            target: self.target,
            edge_type: self.edge_type.clone(),
            metadata: self.metadata.clone(),
            event_metadata: EventMetadata::new(),
        };

        Ok(vec![Box::new(event)])
    }
}

/// Command handler trait for processing commands outside the aggregate
pub trait CommandHandler<C: Command>: Send + Sync {
    /// Handle the command and return events
    fn handle(&self, command: C) -> Result<Vec<Box<dyn DomainEvent>>>;
}

/// Factory for creating graphs with proper dependency injection
pub trait GraphFactory: Send + Sync {
    /// Create a new graph with the given parameters
    fn create_graph(&self, command: CreateGraph) -> Result<ContextGraph>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::graph::domain::context_graph::{DefaultInvariantValidator, DefaultPositionCalculator};

    #[test]
    fn test_add_node_command() {
        let graph_id = GraphId::new();
        let root_id = NodeId::new();

        let mut graph = ContextGraph::new(
            graph_id,
            "Test Graph".to_string(),
            crate::contexts::graph::domain::context_graph::ContextType::BoundedContext {
                name: "Test".to_string(),
                domain: "Testing".to_string(),
            },
            root_id,
            Box::new(DefaultInvariantValidator),
            Box::new(DefaultPositionCalculator),
        ).unwrap();

        // Create and execute command
        let node_id = NodeId::new();
        let command = AddNode {
            graph_id,
            node_id,
            node_type: "TestNode".to_string(),
            metadata: HashMap::new(),
        };

        let events = graph.handle_command(command).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(graph.node_count(), 1);
        assert!(graph.has_node(&node_id));
    }

    #[test]
    fn test_connect_nodes_command() {
        let graph_id = GraphId::new();
        let root_id = NodeId::new();

        let mut graph = ContextGraph::new(
            graph_id,
            "Test Graph".to_string(),
            crate::contexts::graph::domain::context_graph::ContextType::BoundedContext {
                name: "Test".to_string(),
                domain: "Testing".to_string(),
            },
            root_id,
            Box::new(DefaultInvariantValidator),
            Box::new(DefaultPositionCalculator),
        ).unwrap();

        // Add two nodes first
        let node1 = NodeId::new();
        let node2 = NodeId::new();

        graph.add_node(node1, "Node1".to_string(), HashMap::new()).unwrap();
        graph.add_node(node2, "Node2".to_string(), HashMap::new()).unwrap();

        // Connect them
        let edge_id = EdgeId::new();
        let command = ConnectNodes {
            graph_id,
            edge_id,
            source: node1,
            target: node2,
            edge_type: "TestEdge".to_string(),
            metadata: HashMap::new(),
        };

        let events = graph.handle_command(command).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(graph.edge_count(), 1);
    }
}
