//! Graph Context Domain Layer
//!
//! This module contains the pure domain logic for the graph context.
//! No external dependencies, no Bevy, no NATS - just business logic.

pub mod commands;
pub mod context_graph;
pub mod edge;
pub mod events;
pub mod invariants;
pub mod node;

pub use commands::{AddNode, ConnectNodes, CreateGraph};
pub use context_graph::{ContextGraph, ContextType};
pub use edge::{Edge, EdgeData};
pub use events::GraphEvent;
pub use node::{Node, NodeData};
