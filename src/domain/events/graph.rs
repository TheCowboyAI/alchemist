use crate::domain::{
    value_objects::{GraphId, GraphMetadata},
    commands::{ImportSource, ImportOptions},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    GraphUpdated {
        graph_id: GraphId,
        name: Option<String>,
        description: Option<String>,
    },
    GraphImportRequested {
        graph_id: GraphId,
        source: ImportSource,
        format: String,
        options: ImportOptions,
    },
    GraphImportCompleted {
        graph_id: GraphId,
        imported_nodes: usize,
        imported_edges: usize,
        source: ImportSource,
    },
    GraphImportFailed {
        graph_id: GraphId,
        source: ImportSource,
        error: String,
    },
}
