//! Context Bridge - Relationships between bounded contexts
//!
//! ContextBridge represents the various ways that different bounded contexts
//! can relate to and communicate with each other in a domain-driven design.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

use crate::domain::conceptual_graph::concept::{ConceptGraph, ConceptId};

/// Unique identifier for a context bridge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextBridgeId(Uuid);

impl ContextBridgeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ContextBridgeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ContextBridgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a relationship between two bounded contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBridge {
    pub id: ContextBridgeId,
    pub source_context: ConceptId,
    pub target_context: ConceptId,
    pub mapping_type: ContextMappingType,
    pub translation_rules: Vec<TranslationRule>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ContextBridge {
    /// Create a new context bridge
    pub fn new(
        source_context: ConceptId,
        target_context: ConceptId,
        mapping_type: ContextMappingType,
    ) -> Self {
        Self {
            id: ContextBridgeId::new(),
            source_context,
            target_context,
            mapping_type,
            translation_rules: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a translation rule to the bridge
    pub fn add_translation_rule(&mut self, rule: TranslationRule) {
        self.translation_rules.push(rule);
    }

    /// Apply translation rules to a concept
    fn apply_translation_rules(
        &self,
        concept: &ConceptGraph,
        direction: TranslationDirection,
    ) -> Result<ConceptGraph, String> {
        // Find applicable rules
        let applicable_rules: Vec<_> = self
            .translation_rules
            .iter()
            .filter(|rule| match direction {
                TranslationDirection::Forward => rule.source_pattern.matches(concept),
                TranslationDirection::Backward => {
                    rule.bidirectional && rule.target_pattern.matches(concept)
                }
            })
            .collect();

        if applicable_rules.is_empty() {
            return Err("No applicable translation rules found".to_string());
        }

        // Apply the first matching rule (could be extended to handle multiple rules)
        let rule = applicable_rules[0];
        rule.transformation.apply(concept, direction)
    }

    /// Translate a concept through the bridge
    pub fn translate(
        &self,
        concept: &ConceptGraph,
        direction: TranslationDirection,
    ) -> Result<ConceptGraph, String> {
        match &self.mapping_type {
            ContextMappingType::SharedKernel { shared_concepts } => {
                // Shared kernel - concept passes through if it's shared
                if shared_concepts.contains(&concept.id) {
                    Ok(concept.clone())
                } else {
                    Err("Concept not in shared kernel".to_string())
                }
            }
            ContextMappingType::CustomerSupplier {
                upstream,
                downstream,
            } => {
                // Customer-supplier - translate based on direction
                match direction {
                    TranslationDirection::Forward => {
                        if &self.source_context == upstream {
                            self.apply_translation_rules(concept, direction)
                        } else {
                            Err("Invalid translation direction for customer-supplier".to_string())
                        }
                    }
                    TranslationDirection::Backward => {
                        if &self.target_context == downstream {
                            self.apply_translation_rules(concept, direction)
                        } else {
                            Err("Invalid translation direction for customer-supplier".to_string())
                        }
                    }
                }
            }
            ContextMappingType::Conformist {
                upstream,
                downstream,
            } => {
                // Conformist - downstream conforms to upstream
                if direction == TranslationDirection::Forward && &self.source_context == upstream {
                    Ok(concept.clone()) // Pass through unchanged
                } else {
                    Err("Conformist only allows forward translation from upstream".to_string())
                }
            }
            ContextMappingType::AntiCorruptionLayer {
                internal_context,
                external_context,
            } => {
                // Anti-corruption layer - apply translation rules
                self.apply_translation_rules(concept, direction)
            }
            ContextMappingType::OpenHostService {
                host,
                service_interface,
            } => {
                // Open host service - translate according to published interface
                self.apply_translation_rules(concept, direction)
            }
            ContextMappingType::Partnership {
                context_a,
                context_b,
            } => {
                // Partnership - bidirectional translation
                self.apply_translation_rules(concept, direction)
            }
            ContextMappingType::PublishedLanguage {
                publisher,
                language_spec,
            } => {
                // Published language - translate according to spec
                self.apply_translation_rules(concept, direction)
            }
        }
    }

    /// Get a human-readable description of the mapping type
    pub fn mapping_description(&self) -> &'static str {
        match &self.mapping_type {
            ContextMappingType::SharedKernel { .. } => "Shared kernel with common concepts",
            ContextMappingType::CustomerSupplier { .. } => "Customer-supplier relationship",
            ContextMappingType::Conformist { .. } => "Conformist - downstream follows upstream",
            ContextMappingType::AntiCorruptionLayer { .. } => {
                "Anti-corruption layer for translation"
            }
            ContextMappingType::OpenHostService { .. } => {
                "Open host service with published interface"
            }
            ContextMappingType::Partnership { .. } => "Partnership with mutual influence",
            ContextMappingType::PublishedLanguage { .. } => "Published language specification",
        }
    }
}

/// Types of relationships between bounded contexts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContextMappingType {
    /// Customer-Supplier: Upstream context provides services to downstream
    CustomerSupplier {
        upstream: ConceptId,
        downstream: ConceptId,
    },

    /// Conformist: Downstream conforms to upstream's model
    Conformist {
        upstream: ConceptId,
        downstream: ConceptId,
    },

    /// Anti-Corruption Layer: Translation layer protects from external models
    AntiCorruptionLayer {
        internal_context: ConceptId,
        external_context: ConceptId,
    },

    /// Shared Kernel: Shared subset of domain model
    SharedKernel { shared_concepts: Vec<ConceptId> },

    /// Partnership: Mutual dependency between contexts
    Partnership {
        context_a: ConceptId,
        context_b: ConceptId,
    },

    /// Published Language: Well-documented shared language
    PublishedLanguage {
        publisher: ConceptId,
        language_spec: PublishedLanguage,
    },

    /// Open Host Service: Public API for multiple consumers
    OpenHostService {
        host: ConceptId,
        service_interface: InterfaceContract,
    },
}

impl ContextMappingType {
    /// Get a description of this mapping type
    pub fn description(&self) -> &'static str {
        match self {
            Self::SharedKernel { .. } => "Shared kernel with common concepts",
            Self::CustomerSupplier { .. } => "Customer-supplier with defined contract",
            Self::Conformist { .. } => "Conformist - downstream follows upstream",
            Self::AntiCorruptionLayer { .. } => "Anti-corruption layer for translation",
            Self::OpenHostService { .. } => "Open host service with published language",
            Self::Partnership { .. } => "Partnership with mutual influence",
            Self::PublishedLanguage { .. } => "Published language specification",
        }
    }
}

/// Interface contract for open host service
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InterfaceContract {
    pub operations: Vec<ContractOperation>,
    pub data_schemas: HashMap<String, DataSchema>,
    pub version: String,
}

/// Operation in an interface contract
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContractOperation {
    pub name: String,
    pub input_schema: String,
    pub output_schema: String,
    pub description: String,
}

/// Data schema definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataSchema {
    pub name: String,
    pub fields: HashMap<String, FieldType>,
    pub required: Vec<String>,
}

/// Field type in a schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Object(String), // Reference to another schema
    Array(Box<FieldType>),
}

/// Translation graph for anti-corruption layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationGraph {
    pub name: String,
    pub nodes: HashMap<ConceptId, TranslationNode>,
    pub edges: Vec<TranslationEdge>,
}

impl TranslationGraph {
    /// Create a new translation graph
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Translate a concept through the graph
    pub fn translate(
        &self,
        concept: &ConceptGraph,
        direction: TranslationDirection,
    ) -> Result<ConceptGraph, String> {
        // Find the starting node for this concept
        if let Some(node) = self.nodes.get(&concept.id) {
            node.translate(concept, direction)
        } else {
            Err("No translation node found for concept".to_string())
        }
    }
}

/// Node in a translation graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationNode {
    pub concept_id: ConceptId,
    pub transformations: Vec<ConceptTransformation>,
}

impl TranslationNode {
    /// Translate a concept using this node's transformations
    pub fn translate(
        &self,
        concept: &ConceptGraph,
        direction: TranslationDirection,
    ) -> Result<ConceptGraph, String> {
        let mut result = concept.clone();

        // Apply transformations in order (or reverse for backward)
        let transformations: Box<dyn Iterator<Item = &ConceptTransformation>> = match direction {
            TranslationDirection::Forward => Box::new(self.transformations.iter()),
            TranslationDirection::Backward => Box::new(self.transformations.iter().rev()),
        };

        for transformation in transformations {
            result = transformation.apply(&result, direction)?;
        }

        Ok(result)
    }
}

/// Edge in a translation graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationEdge {
    pub from: ConceptId,
    pub to: ConceptId,
    pub transformation: ConceptTransformation,
}

/// Published language specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublishedLanguage {
    pub name: String,
    pub version: String,
    pub concepts: Vec<PublishedConcept>,
    pub relationships: Vec<PublishedRelationship>,
    pub constraints: Vec<String>,
}

/// Concept in a published language
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublishedConcept {
    pub name: String,
    pub attributes: Vec<AttributeDefinition>,
    pub invariants: Vec<String>,
}

/// Attribute definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttributeDefinition {
    pub name: String,
    pub attribute_type: String,
    pub required: bool,
    pub constraints: Vec<String>,
}

/// Relationship in a published language
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublishedRelationship {
    pub name: String,
    pub source_concept: String,
    pub target_concept: String,
    pub cardinality: String,
}

/// Translation rule between contexts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslationRule {
    pub source_concept: ConceptId,
    pub target_concept: ConceptId,
    pub transformation: ConceptTransformation,
    pub source_pattern: ConceptPattern,
    pub target_pattern: ConceptPattern,
    pub bidirectional: bool,
}

/// Pattern for matching concepts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConceptPattern {
    pub concept_type: Option<String>,
    pub required_attributes: Vec<String>,
    pub metadata_patterns: HashMap<String, serde_json::Value>,
}

impl ConceptPattern {
    /// Check if a concept matches this pattern
    pub fn matches(&self, concept: &ConceptGraph) -> bool {
        // Check concept type if specified
        if let Some(expected_type) = &self.concept_type {
            if let Some(actual_type) = concept.metadata.get("type") {
                if actual_type.as_str() != Some(expected_type) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check required attributes
        for attr in &self.required_attributes {
            if !concept.metadata.contains_key(attr) {
                return false;
            }
        }

        // Check metadata patterns
        for (key, expected_value) in &self.metadata_patterns {
            if let Some(actual_value) = concept.metadata.get(key) {
                if actual_value != expected_value {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

/// Transformation to apply when translating concepts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConceptTransformation {
    /// Direct mapping - concept passes through unchanged
    Identity,

    /// Rename concept
    Rename { new_name: String },

    /// Map attributes
    AttributeMapping { mappings: HashMap<String, String> },

    /// Filter attributes
    FilterAttributes { keep: Vec<String> },

    /// Add attributes with default values
    AddAttributes {
        attributes: HashMap<String, serde_json::Value>,
    },

    /// Custom transformation function
    Custom {
        name: String,
        parameters: HashMap<String, serde_json::Value>,
    },

    /// Composite transformation
    Composite {
        transformations: Vec<ConceptTransformation>,
    },
}

impl ConceptTransformation {
    /// Apply the transformation to a concept
    pub fn apply(
        &self,
        concept: &ConceptGraph,
        direction: TranslationDirection,
    ) -> Result<ConceptGraph, String> {
        match self {
            Self::Identity => Ok(concept.clone()),

            Self::Rename { new_name } => {
                let mut result = concept.clone();
                match direction {
                    TranslationDirection::Forward => {
                        result.name = new_name.clone();
                    }
                    TranslationDirection::Backward => {
                        // For backward, we'd need to store the original name
                        // This is a simplified implementation
                    }
                }
                Ok(result)
            }

            Self::AttributeMapping { mappings } => {
                let result = concept.clone();
                // Apply attribute mappings
                // This would map attributes based on the mappings HashMap
                Ok(result)
            }

            Self::FilterAttributes { keep } => {
                let result = concept.clone();
                // Keep only specified attributes
                // This would filter the concept's attributes
                Ok(result)
            }

            Self::AddAttributes { attributes } => {
                let result = concept.clone();
                // Add new attributes with default values
                // This would add the specified attributes to the concept
                Ok(result)
            }

            Self::Custom { name, parameters } => {
                // Apply custom transformation based on name and parameters
                // This would look up and apply a custom transformation function
                Err(format!("Custom transformation '{}' not implemented", name))
            }

            Self::Composite { transformations } => {
                let mut result = concept.clone();
                for transformation in transformations {
                    result = transformation.apply(&result, direction)?;
                }
                Ok(result)
            }
        }
    }
}

/// Direction of translation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TranslationDirection {
    /// From source context to target context
    Forward,
    /// From target context back to source context
    Backward,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_bridge_creation() {
        let source = ConceptId::new();
        let target = ConceptId::new();
        let bridge = ContextBridge::new(
            source,
            target,
            ContextMappingType::Conformist {
                upstream: source,
                downstream: target,
            },
        );

        assert_eq!(bridge.source_context, source);
        assert_eq!(bridge.target_context, target);
        assert!(bridge.translation_rules.is_empty());
    }

    #[test]
    fn test_translation_rule() {
        let source = ConceptId::new();
        let target = ConceptId::new();
        let transformation = ConceptTransformation::Rename {
            new_name: "TranslatedConcept".to_string(),
        };

        let rule = TranslationRule {
            source_concept: source,
            target_concept: target,
            transformation,
            source_pattern: ConceptPattern {
                concept_type: None,
                required_attributes: vec![],
                metadata_patterns: HashMap::new(),
            },
            target_pattern: ConceptPattern {
                concept_type: None,
                required_attributes: vec![],
                metadata_patterns: HashMap::new(),
            },
            bidirectional: false,
        };
        assert_eq!(rule.source_concept, source);
        assert_eq!(rule.target_concept, target);
    }

    #[test]
    fn test_mapping_type_descriptions() {
        let shared_kernel = ContextMappingType::SharedKernel {
            shared_concepts: vec![],
        };
        assert_eq!(
            shared_kernel.description(),
            "Shared kernel with common concepts"
        );

        let conformist = ContextMappingType::Conformist {
            upstream: ConceptId::new(),
            downstream: ConceptId::new(),
        };
        assert_eq!(
            conformist.description(),
            "Conformist - downstream follows upstream"
        );
    }

    #[test]
    fn test_translation_graph() {
        let mut graph = TranslationGraph::new("TestTranslator");
        assert_eq!(graph.name, "TestTranslator");
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }
}
