//! Graph Context Domain Layer
//!
//! This module contains the pure domain logic for the graph context.
//! No external dependencies, no Bevy, no NATS - just business logic.

pub mod context_graph;
pub mod node;
pub mod edge;
pub mod events;
pub mod commands;
pub mod invariants;

pub use context_graph::{ContextGraph, ContextType};
pub use node::{Node, NodeData};
pub use edge::{Edge, EdgeData};
pub use events::GraphEvent;
pub use commands::{AddNode, ConnectNodes, CreateGraph};
