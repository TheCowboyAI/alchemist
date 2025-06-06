//! Domain Commands

use serde::{Deserialize, Serialize};

pub mod aggregated_commands;
pub mod edge_commands;
pub mod graph_commands;
pub mod node_commands;

pub use aggregated_commands::{
    DomainCommand, UpdateNodePositions, UpdateGraphSelection,
    RecognizeGraphModel, ApplyGraphMorphism, MorphismType
};
pub use edge_commands::EdgeCommand;
pub use graph_commands::GraphCommand;
pub use node_commands::NodeCommand;

/// All commands in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Graph(GraphCommand),
    Node(NodeCommand),
    Edge(EdgeCommand),
}

impl Command {
    pub fn command_type(&self) -> &'static str {
        match self {
            Command::Graph(c) => c.command_type(),
            Command::Node(c) => c.command_type(),
            Command::Edge(c) => c.command_type(),
        }
    }
}
