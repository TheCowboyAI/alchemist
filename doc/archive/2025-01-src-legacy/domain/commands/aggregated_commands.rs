//! Aggregated commands from presentation layer
//!
//! These commands represent business-meaningful state changes that
//! result from aggregating multiple presentation events.

use serde::{Deserialize, Serialize};
use crate::domain::value_objects::{NodeId, Position3D, GraphModel};

/// Update multiple node positions at once
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNodePositions {
    /// List of node updates (node_id, new_position)
    pub updates: Vec<(NodeId, Position3D)>,
    /// Reason for the update (e.g., "User drag operation", "Force-directed layout")
    pub reason: String,
}

/// Update the current selection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGraphSelection {
    /// Currently selected nodes
    pub selected_nodes: Vec<NodeId>,
    /// Reason for the update
    pub reason: String,
}

/// Recognize a graph as a specific model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognizeGraphModel {
    /// The recognized model
    pub model: GraphModel,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
}

/// Apply a structure-preserving morphism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyGraphMorphism {
    /// Source model
    pub from_model: GraphModel,
    /// Target model
    pub to_model: GraphModel,
    /// Morphism type
    pub morphism_type: MorphismType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MorphismType {
    /// Edge subdivision (e.g., K3 → C6)
    EdgeSubdivision { subdivisions: usize },
    /// Node duplication
    NodeDuplication { factor: usize },
    /// Complement graph
    Complement,
    /// Line graph transformation
    LineGraph,
    /// Custom transformation
    Custom(String),
}

/// Aggregated domain commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainCommand {
    UpdateNodePositions(UpdateNodePositions),
    UpdateGraphSelection(UpdateGraphSelection),
    RecognizeGraphModel(RecognizeGraphModel),
    ApplyGraphMorphism(ApplyGraphMorphism),
}

impl DomainCommand {
    pub fn command_type(&self) -> &'static str {
        match self {
            DomainCommand::UpdateNodePositions(_) => "UpdateNodePositions",
            DomainCommand::UpdateGraphSelection(_) => "UpdateGraphSelection",
            DomainCommand::RecognizeGraphModel(_) => "RecognizeGraphModel",
            DomainCommand::ApplyGraphMorphism(_) => "ApplyGraphMorphism",
        }
    }
}
