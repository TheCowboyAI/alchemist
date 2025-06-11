//! Graph domain events - Facts about what happened
//!
//! Events represent things that have happened in the domain.
//! They are immutable and used for event sourcing.

use crate::shared::types::{GraphId, NodeId, EdgeId};
use crate::shared::events::{EventMetadata, DomainEvent};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Macro to implement DomainEvent trait with common functionality
macro_rules! impl_domain_event {
    ($event_type:ty, $aggregate_id_expr:expr, $event_name:expr) => {
        impl DomainEvent for $event_type {
            fn aggregate_id(&self) -> String {
                $aggregate_id_expr(self)
            }

            fn event_type(&self) -> &'static str {
                $event_name
            }

            fn metadata(&self) -> &EventMetadata {
                &self.event_metadata
            }

            fn to_json(&self) -> crate::shared::types::Result<serde_json::Value> {
                serde_json::to_value(self).map_err(|e| crate::shared::types::Error::Serialization(e))
            }

            fn clone_box(&self) -> Box<dyn DomainEvent> {
                Box::new(self.clone())
            }
        }
    };
}

/// Base trait for all graph events
pub trait GraphEvent: DomainEvent {
    /// Get the graph ID this event belongs to
    fn graph_id(&self) -> GraphId;
}

/// Graph was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphCreated {
    pub graph_id: GraphId,
    pub name: String,
    pub context_type: crate::contexts::graph::domain::context_graph::ContextType,
    pub context_root: NodeId,
    pub event_metadata: EventMetadata,
}

impl_domain_event!(GraphCreated, |e: &GraphCreated| e.graph_id.to_string(), "graph.created");

impl GraphEvent for GraphCreated {
    fn graph_id(&self) -> GraphId {
        self.graph_id
    }
}

/// Node was added to the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAdded {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub node_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub event_metadata: EventMetadata,
}

impl_domain_event!(NodeAdded, |e: &NodeAdded| e.graph_id.to_string(), "graph.node_added");

impl GraphEvent for NodeAdded {
    fn graph_id(&self) -> GraphId {
        self.graph_id
    }
}

/// Edge was added to the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeAdded {
    pub graph_id: GraphId,
    pub edge_id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub edge_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub event_metadata: EventMetadata,
}

impl_domain_event!(EdgeAdded, |e: &EdgeAdded| e.graph_id.to_string(), "graph.edge_added");

impl GraphEvent for EdgeAdded {
    fn graph_id(&self) -> GraphId {
        self.graph_id
    }
}

/// Node was removed from the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRemoved {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub event_metadata: EventMetadata,
}

impl_domain_event!(NodeRemoved, |e: &NodeRemoved| e.graph_id.to_string(), "graph.node_removed");

impl GraphEvent for NodeRemoved {
    fn graph_id(&self) -> GraphId {
        self.graph_id
    }
}

/// Edge was removed from the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRemoved {
    pub graph_id: GraphId,
    pub edge_id: EdgeId,
    pub event_metadata: EventMetadata,
}

impl_domain_event!(EdgeRemoved, |e: &EdgeRemoved| e.graph_id.to_string(), "graph.edge_removed");

impl GraphEvent for EdgeRemoved {
    fn graph_id(&self) -> GraphId {
        self.graph_id
    }
}

/// Event handler trait for processing graph events
pub trait GraphEventHandler: Send + Sync {
    /// Handle a graph created event
    fn handle_graph_created(&mut self, event: &GraphCreated) -> crate::shared::types::Result<()>;

    /// Handle a node added event
    fn handle_node_added(&mut self, event: &NodeAdded) -> crate::shared::types::Result<()>;

    /// Handle an edge added event
    fn handle_edge_added(&mut self, event: &EdgeAdded) -> crate::shared::types::Result<()>;

    /// Handle a node removed event
    fn handle_node_removed(&mut self, event: &NodeRemoved) -> crate::shared::types::Result<()>;

    /// Handle an edge removed event
    fn handle_edge_removed(&mut self, event: &EdgeRemoved) -> crate::shared::types::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_subjects() {
        let graph_id = GraphId::new();

        let event = NodeAdded {
            graph_id,
            node_id: NodeId::new(),
            node_type: "TestNode".to_string(),
            metadata: HashMap::new(),
            event_metadata: EventMetadata::new(),
        };

        assert_eq!(event.event_type(), "graph.node_added");
        assert_eq!(event.subject(), format!("events.graph.node_added.{}", graph_id));
    }

    #[test]
    fn test_event_serialization() {
        let event = GraphCreated {
            graph_id: GraphId::new(),
            name: "Test Graph".to_string(),
            context_type: crate::contexts::graph::domain::context_graph::ContextType::BoundedContext {
                name: "Test".to_string(),
                domain: "Testing".to_string(),
            },
            context_root: NodeId::new(),
            event_metadata: EventMetadata::new(),
        };

        // Should be serializable
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: GraphCreated = serde_json::from_str(&json).unwrap();

        assert_eq!(event.graph_id, deserialized.graph_id);
        assert_eq!(event.name, deserialized.name);
    }
}
