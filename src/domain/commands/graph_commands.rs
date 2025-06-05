//! Graph Commands

use crate::domain::value_objects::GraphId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphCommand {
    CreateGraph { id: GraphId, name: String },
    DeleteGraph { id: GraphId },
    RenameGraph { id: GraphId, new_name: String },
    TagGraph { id: GraphId, tag: String },
    UntagGraph { id: GraphId, tag: String },
}

impl GraphCommand {
    pub fn command_type(&self) -> &'static str {
        match self {
            GraphCommand::CreateGraph { .. } => "CreateGraph",
            GraphCommand::DeleteGraph { .. } => "DeleteGraph",
            GraphCommand::RenameGraph { .. } => "RenameGraph",
            GraphCommand::TagGraph { .. } => "TagGraph",
            GraphCommand::UntagGraph { .. } => "UntagGraph",
        }
    }

    pub fn graph_id(&self) -> GraphId {
        match self {
            GraphCommand::CreateGraph { id, .. }
            | GraphCommand::DeleteGraph { id }
            | GraphCommand::RenameGraph { id, .. }
            | GraphCommand::TagGraph { id, .. }
            | GraphCommand::UntagGraph { id, .. } => *id,
        }
    }
}
