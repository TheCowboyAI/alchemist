//! Graph Commands

use crate::domain::value_objects::GraphId;
use serde::{Deserialize, Serialize};

/// Commands for Graph aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphCommand {
    /// Create a new graph
    CreateGraph {
        id: GraphId,
        name: String,
        description: Option<String>,
    },
    /// Update graph metadata
    UpdateGraphMetadata {
        id: GraphId,
        name: Option<String>,
        description: Option<String>,
    },
    /// Delete a graph
    DeleteGraph { id: GraphId },
}
