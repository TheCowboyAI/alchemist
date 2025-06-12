//! Mapping module for converting between domain-specific types and base graph types

use crate::base_types::*;
use std::error::Error;
use std::fmt;

/// Error type for mapping operations
#[derive(Debug, Clone)]
pub struct MappingError {
    message: String,
}

impl fmt::Display for MappingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mapping error: {}", self.message)
    }
}

impl Error for MappingError {}

/// Example mapping for domain-specific node types
pub struct DomainNodeMapping;

impl DomainNodeMapping {
    /// Convert a domain-specific node type string to BaseNodeType
    pub fn from_string(type_str: &str) -> BaseNodeType {
        match type_str {
            "value" | "value_object" => BaseNodeType::Value,
            "entity_reference" => BaseNodeType::EntityReference,
            "aggregate" => BaseNodeType::Aggregate,
            "service" => BaseNodeType::Service,
            "event" => BaseNodeType::Event,
            "command" => BaseNodeType::Command,
            _ => BaseNodeType::Custom(type_str.to_string()),
        }
    }

    /// Convert BaseNodeType to domain-specific string
    pub fn to_string(node_type: &BaseNodeType) -> String {
        match node_type {
            BaseNodeType::Value => "value_object".to_string(),
            BaseNodeType::EntityReference => "entity_reference".to_string(),
            BaseNodeType::Aggregate => "aggregate".to_string(),
            BaseNodeType::Service => "service".to_string(),
            BaseNodeType::Event => "event".to_string(),
            BaseNodeType::Command => "command".to_string(),
            BaseNodeType::Custom(s) => s.clone(),
        }
    }
}

/// Example mapping for domain-specific relationship types
pub struct DomainRelationshipMapping;

impl DomainRelationshipMapping {
    /// Convert a domain-specific relationship type string to BaseRelationshipType
    pub fn from_string(type_str: &str) -> BaseRelationshipType {
        match type_str {
            "contains" => BaseRelationshipType::Contains,
            "references" => BaseRelationshipType::References,
            "depends_on" => BaseRelationshipType::DependsOn,
            "sequence" => BaseRelationshipType::Sequence,
            "parallel" => BaseRelationshipType::Parallel,
            "choice" => BaseRelationshipType::Choice,

            _ => BaseRelationshipType::Custom(type_str.to_string()),
        }
    }

    /// Convert BaseRelationshipType to domain-specific string
    pub fn to_string(rel_type: &BaseRelationshipType) -> String {
        match rel_type {
            BaseRelationshipType::Contains => "contains".to_string(),
            BaseRelationshipType::References => "references".to_string(),
            BaseRelationshipType::DependsOn => "depends_on".to_string(),
            BaseRelationshipType::Sequence => "sequence".to_string(),
            BaseRelationshipType::Parallel => "parallel".to_string(),
            BaseRelationshipType::Choice => "choice".to_string(),

            BaseRelationshipType::Custom(s) => s.clone(),
        }
    }
}

/// Trait for types that can be mapped to/from domain types
pub trait DomainMappable<T> {
    type Error;

    fn to_domain(&self) -> Result<T, Self::Error>;
    fn from_domain(domain: T) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_mapping() {
        assert!(matches!(
            DomainNodeMapping::from_string("entity_reference"),
            BaseNodeType::EntityReference
        ));
        assert!(matches!(
            DomainNodeMapping::from_string("custom_type"),
            BaseNodeType::Custom(_)
        ));
    }

    #[test]
    fn test_relationship_type_mapping() {
        assert!(matches!(
            DomainRelationshipMapping::from_string("contains"),
            BaseRelationshipType::Contains
        ));
        assert!(matches!(
            DomainRelationshipMapping::from_string("custom_rel"),
            BaseRelationshipType::Custom(_)
        ));
    }
}
