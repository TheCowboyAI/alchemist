//! Events for context bridge operations

use serde::{Deserialize, Serialize};

use crate::domain::conceptual_graph::{
    ConceptId, ContextBridgeId, ContextMappingType,
    TranslationRule
};

/// Events related to context bridges
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContextBridgeEvent {
    /// A new bridge was created between contexts
    BridgeCreated {
        bridge_id: ContextBridgeId,
        source_context: ConceptId,
        target_context: ConceptId,
        mapping_type: ContextMappingType,
    },

    /// A translation rule was added to a bridge
    TranslationRuleAdded {
        bridge_id: ContextBridgeId,
        rule: TranslationRule,
    },

    /// A translation rule was removed from a bridge
    TranslationRuleRemoved {
        bridge_id: ContextBridgeId,
        rule_id: String,
    },

    /// A concept was translated across contexts
    ConceptTranslated {
        bridge_id: ContextBridgeId,
        source_concept_id: ConceptId,
        target_concept_id: ConceptId,
    },

    /// A bridge was removed
    BridgeDeleted {
        bridge_id: ContextBridgeId,
    },

    /// Bridge mapping type was changed
    MappingTypeUpdated {
        bridge_id: ContextBridgeId,
        old_type: ContextMappingType,
        new_type: ContextMappingType,
    },

    /// Translation failed
    TranslationFailed {
        bridge_id: ContextBridgeId,
        source_concept: ConceptId,
        error: String,
    },
}

impl ContextBridgeEvent {
    /// Get a description of the event
    pub fn description(&self) -> &'static str {
        match self {
            Self::BridgeCreated { .. } => "Context bridge created",
            Self::TranslationRuleAdded { .. } => "Translation rule added",
            Self::TranslationRuleRemoved { .. } => "Translation rule removed",
            Self::ConceptTranslated { .. } => "Concept translated",
            Self::MappingTypeUpdated { .. } => "Bridge mapping type updated",
            Self::BridgeDeleted { .. } => "Context bridge deleted",
            Self::TranslationFailed { .. } => "Translation failed",
        }
    }
}
