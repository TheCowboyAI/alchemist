//! Domain Events

use crate::domain::value_objects::*;
use serde::{Deserialize, Serialize};

pub mod edge_events;
pub mod graph_events;
pub mod node_events;

pub use edge_events::EdgeEvent;
pub use graph_events::GraphEvent;
pub use node_events::NodeEvent;

/// All domain events in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    Graph(GraphEvent),
    Node(NodeEvent),
    Edge(EdgeEvent),
}

impl DomainEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            DomainEvent::Graph(e) => e.event_type(),
            DomainEvent::Node(e) => e.event_type(),
            DomainEvent::Edge(e) => e.event_type(),
        }
    }
}
