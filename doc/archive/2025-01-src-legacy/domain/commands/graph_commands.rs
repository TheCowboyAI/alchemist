//! Graph Commands

use crate::domain::value_objects::{EdgeId, GraphId, NodeId, Position3D};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphCommand {
    CreateGraph {
        id: GraphId,
        name: String,
        metadata: HashMap<String, serde_json::Value>,
    },
    DeleteGraph {
        id: GraphId,
    },
    RenameGraph {
        id: GraphId,
        new_name: String,
    },
    TagGraph {
        id: GraphId,
        tag: String,
    },
    UntagGraph {
        id: GraphId,
        tag: String,
    },
    UpdateGraph {
        id: GraphId,
        name: Option<String>,
        description: Option<String>,
    },
    ClearGraph {
        graph_id: GraphId,
    },
    AddNode {
        graph_id: GraphId,
        node_id: NodeId,
        node_type: String,
        position: Position3D,
        content: serde_json::Value,
    },
    UpdateNode {
        graph_id: GraphId,
        node_id: NodeId,
        new_position: Option<Position3D>,
        new_content: Option<serde_json::Value>,
    },
    RemoveNode {
        graph_id: GraphId,
        node_id: NodeId,
    },
    ConnectNodes {
        graph_id: GraphId,
        edge_id: EdgeId,
        source_id: NodeId,
        target_id: NodeId,
        edge_type: String,
        properties: HashMap<String, serde_json::Value>,
    },
    DisconnectNodes {
        graph_id: GraphId,
        edge_id: EdgeId,
    },
    UpdateEdge {
        graph_id: GraphId,
        edge_id: EdgeId,
        new_properties: HashMap<String, serde_json::Value>,
    },
    ImportGraph {
        graph_id: GraphId,
        source: ImportSource,
        format: String,
        options: ImportOptions,
    },
    ImportFromFile {
        graph_id: GraphId,
        file_path: String,
        format: String,
    },
    ImportFromUrl {
        graph_id: GraphId,
        url: String,
        format: String,
    },
    CreateConceptualGraph {
        graph_id: GraphId,
        name: String,
        category_type: crate::domain::conceptual_graph::CategoryType,
    },
    AddConceptualNode {
        graph_id: GraphId,
        node_id: NodeId,
        concept_type: crate::domain::conceptual_graph::ConceptType,
        conceptual_point: crate::domain::conceptual_graph::ConceptualPoint,
    },
    ApplyGraphMorphism {
        source_graph: GraphId,
        target_graph: GraphId,
        morphism: crate::domain::conceptual_graph::GraphMorphism,
    },
    ComposeConceptualGraphs {
        graph_ids: Vec<GraphId>,
        operation: crate::domain::conceptual_graph::CompositionOperation,
        result_graph_id: GraphId,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImportSource {
    File {
        path: String,
    },
    Url {
        url: String,
    },
    GitRepository {
        url: String,
        branch: Option<String>,
        path: String,
    },
    NixFlake {
        flake_ref: String,
        output: String,
    },
    InlineContent {
        content: String,
    },
}

/// How to handle conflicts when importing into existing graphs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MergeBehavior {
    /// Skip nodes/edges that already exist
    Skip,
    /// Replace existing nodes/edges
    Replace,
    /// Merge properties (existing properties take precedence)
    MergePreferExisting,
    /// Merge properties (imported properties take precedence)
    MergePreferImported,
    /// Always create new nodes/edges (may create duplicates)
    AlwaysCreate,
}

/// Import options for graph import operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportOptions {
    /// How to handle existing nodes/edges
    pub merge_behavior: MergeBehavior,

    /// Prefix to add to imported node IDs
    pub id_prefix: Option<String>,

    /// Offset to apply to imported positions
    pub position_offset: Option<Position3D>,

    /// Mapping configuration for field transformations
    pub mapping: Option<crate::domain::services::graph_import::ImportMapping>,

    /// Whether to validate imported data
    pub validate: bool,

    /// Maximum number of nodes to import (for safety)
    pub max_nodes: Option<usize>,
}

impl GraphCommand {
    pub fn command_type(&self) -> &'static str {
        match self {
            GraphCommand::CreateGraph { .. } => "CreateGraph",
            GraphCommand::DeleteGraph { .. } => "DeleteGraph",
            GraphCommand::RenameGraph { .. } => "RenameGraph",
            GraphCommand::TagGraph { .. } => "TagGraph",
            GraphCommand::UntagGraph { .. } => "UntagGraph",
            GraphCommand::UpdateGraph { .. } => "UpdateGraph",
            GraphCommand::ClearGraph { .. } => "ClearGraph",
            GraphCommand::AddNode { .. } => "AddNode",
            GraphCommand::UpdateNode { .. } => "UpdateNode",
            GraphCommand::RemoveNode { .. } => "RemoveNode",
            GraphCommand::ConnectNodes { .. } => "ConnectNodes",
            GraphCommand::DisconnectNodes { .. } => "DisconnectNodes",
            GraphCommand::UpdateEdge { .. } => "UpdateEdge",
            GraphCommand::ImportGraph { .. } => "ImportGraph",
            GraphCommand::ImportFromFile { .. } => "ImportFromFile",
            GraphCommand::ImportFromUrl { .. } => "ImportFromUrl",
            GraphCommand::CreateConceptualGraph { .. } => "CreateConceptualGraph",
            GraphCommand::AddConceptualNode { .. } => "AddConceptualNode",
            GraphCommand::ApplyGraphMorphism { .. } => "ApplyGraphMorphism",
            GraphCommand::ComposeConceptualGraphs { .. } => "ComposeConceptualGraphs",
        }
    }

    pub fn graph_id(&self) -> GraphId {
        match self {
            GraphCommand::CreateGraph { id, .. }
            | GraphCommand::DeleteGraph { id }
            | GraphCommand::RenameGraph { id, .. }
            | GraphCommand::TagGraph { id, .. }
            | GraphCommand::UntagGraph { id, .. }
            | GraphCommand::UpdateGraph { id, .. } => *id,
            GraphCommand::ClearGraph { graph_id }
            | GraphCommand::AddNode { graph_id, .. }
            | GraphCommand::UpdateNode { graph_id, .. }
            | GraphCommand::RemoveNode { graph_id, .. }
            | GraphCommand::ConnectNodes { graph_id, .. }
            | GraphCommand::DisconnectNodes { graph_id, .. }
            | GraphCommand::UpdateEdge { graph_id, .. }
            | GraphCommand::ImportGraph { graph_id, .. }
            | GraphCommand::ImportFromFile { graph_id, .. }
            | GraphCommand::ImportFromUrl { graph_id, .. }
            | GraphCommand::CreateConceptualGraph { graph_id, .. }
            | GraphCommand::AddConceptualNode { graph_id, .. } => *graph_id,
            GraphCommand::ApplyGraphMorphism { source_graph, .. } => *source_graph,
            GraphCommand::ComposeConceptualGraphs {
                result_graph_id, ..
            } => *result_graph_id,
        }
    }
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            merge_behavior: MergeBehavior::AlwaysCreate,
            id_prefix: None,
            position_offset: None,
            mapping: None,
            validate: true,
            max_nodes: None,
        }
    }
}
