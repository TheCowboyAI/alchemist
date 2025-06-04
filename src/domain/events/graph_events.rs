//! Graph-related domain events

use serde::{Deserialize, Serialize};
use crate::domain::value_objects::{GraphId, GraphMetadata};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphEvent {
    GraphCreated {
        id: GraphId,
        metadata: GraphMetadata,
    },
    GraphDeleted {
        id: GraphId,
    },
    GraphRenamed {
        id: GraphId,
        old_name: String,
        new_name: String,
    },
    GraphTagged {
        id: GraphId,
        tag: String,
    },
    GraphUntagged {
        id: GraphId,
        tag: String,
    },
}

impl GraphEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            GraphEvent::GraphCreated { .. } => "GraphCreated",
            GraphEvent::GraphDeleted { .. } => "GraphDeleted",
            GraphEvent::GraphRenamed { .. } => "GraphRenamed",
            GraphEvent::GraphTagged { .. } => "GraphTagged",
            GraphEvent::GraphUntagged { .. } => "GraphUntagged",
        }
    }

    pub fn graph_id(&self) -> GraphId {
        match self {
            GraphEvent::GraphCreated { id, .. } |
            GraphEvent::GraphDeleted { id } |
            GraphEvent::GraphRenamed { id, .. } |
            GraphEvent::GraphTagged { id, .. } |
            GraphEvent::GraphUntagged { id, .. } => *id,
        }
    }
}
