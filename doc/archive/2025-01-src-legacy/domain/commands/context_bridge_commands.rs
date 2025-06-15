//! Commands for managing context bridges between bounded contexts

use serde::{Deserialize, Serialize};

use crate::domain::conceptual_graph::{
    ConceptGraph, ConceptId, ContextBridgeId, ContextMappingType, TranslationDirection,
    TranslationRule,
};

/// Commands for managing context bridges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextBridgeCommand {
    /// Create a new bridge between contexts
    CreateBridge {
        source_context: ConceptId,
        target_context: ConceptId,
        mapping_type: ContextMappingType,
    },

    /// Add a translation rule to an existing bridge
    AddTranslationRule {
        bridge_id: ContextBridgeId,
        rule: TranslationRule,
    },

    /// Remove a translation rule from a bridge
    RemoveTranslationRule {
        bridge_id: ContextBridgeId,
        source_concept: ConceptId,
        target_concept: ConceptId,
    },

    /// Translate a concept through a bridge
    TranslateConcept {
        bridge_id: ContextBridgeId,
        concept: ConceptGraph,
        direction: TranslationDirection,
    },

    /// Update the mapping type of a bridge
    UpdateMappingType {
        bridge_id: ContextBridgeId,
        new_mapping_type: ContextMappingType,
    },

    /// Delete a context bridge
    DeleteBridge { bridge_id: ContextBridgeId },
}

impl ContextBridgeCommand {
    /// Get a description of the command
    pub fn description(&self) -> &'static str {
        match self {
            Self::CreateBridge { .. } => "Create a new context bridge",
            Self::AddTranslationRule { .. } => "Add translation rule to bridge",
            Self::RemoveTranslationRule { .. } => "Remove translation rule from bridge",
            Self::TranslateConcept { .. } => "Translate concept through bridge",
            Self::UpdateMappingType { .. } => "Update bridge mapping type",
            Self::DeleteBridge { .. } => "Delete context bridge",
        }
    }
}
