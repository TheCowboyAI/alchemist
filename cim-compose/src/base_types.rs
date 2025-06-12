//! Base types for the graph composition system
//!
//! This module provides generic types that can be used without depending on
//! the main application's domain types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use uuid::Uuid;

/// A generic entity with a typed ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Entity<T> {
    pub id: EntityId<T>,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
}

impl<T> Entity<T> {
    pub fn new() -> Self {
        let now = std::time::SystemTime::now();
        Self {
            id: EntityId::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_id(id: EntityId<T>) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            id,
            created_at: now,
            updated_at: now,
        }
    }
}

/// A typed entity ID using phantom types for type safety
/// These IDs are globally unique and persistent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId<T> {
    id: Uuid,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T> EntityId<T> {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            _phantom: PhantomData,
        }
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.id
    }
}

impl<T> fmt::Display for EntityId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl<T> Default for EntityId<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Marker types for entity IDs
pub mod markers {
    use serde::{Deserialize, Serialize};

    /// Marker for graph entities
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct GraphMarker;

    /// Marker for aggregate entities
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct AggregateMarker;

    /// Marker for bounded context entities
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct BoundedContextMarker;
}

// Type aliases for common entity IDs
pub type GraphId = EntityId<markers::GraphMarker>;
pub type AggregateId = EntityId<markers::AggregateMarker>;
pub type BoundedContextId = EntityId<markers::BoundedContextMarker>;

/// Node ID - only meaningful within a graph context
/// These are NOT entities - they're local identifiers within a graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Edge ID - only meaningful within a graph context
/// These are NOT entities - they're local identifiers within a graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(Uuid);

impl EdgeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EdgeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Base node types that can be extended
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BaseNodeType {
    /// A simple value node
    Value,
    /// An entity reference node (contains EntityId)
    EntityReference,
    /// An aggregate root node
    Aggregate,
    /// A service node
    Service,
    /// A command node
    Command,
    /// An event node
    Event,
    /// A custom node type
    Custom(String),
}

impl fmt::Display for BaseNodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BaseNodeType::Value => write!(f, "Value"),
            BaseNodeType::EntityReference => write!(f, "EntityReference"),
            BaseNodeType::Aggregate => write!(f, "Aggregate"),
            BaseNodeType::Service => write!(f, "Service"),
            BaseNodeType::Command => write!(f, "Command"),
            BaseNodeType::Event => write!(f, "Event"),
            BaseNodeType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// Base relationship types that can be extended
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BaseRelationshipType {
    /// Contains relationship (parent-child)
    Contains,
    /// References relationship
    References,
    /// Depends on relationship
    DependsOn,
    /// Sequence relationship (ordered)
    Sequence,
    /// Parallel relationship
    Parallel,
    /// Choice relationship (one of)
    Choice,
    /// Custom relationship
    Custom(String),
}

impl fmt::Display for BaseRelationshipType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BaseRelationshipType::Contains => write!(f, "Contains"),
            BaseRelationshipType::References => write!(f, "References"),
            BaseRelationshipType::DependsOn => write!(f, "DependsOn"),
            BaseRelationshipType::Sequence => write!(f, "Sequence"),
            BaseRelationshipType::Parallel => write!(f, "Parallel"),
            BaseRelationshipType::Choice => write!(f, "Choice"),
            BaseRelationshipType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// A generic relationship between nodes within a graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relationship<T = BaseRelationshipType> {
    pub relationship_type: T,
    pub metadata: HashMap<String, serde_json::Value>,
    pub bidirectional: bool,
}

impl<T> Relationship<T> {
    pub fn new(relationship_type: T) -> Self {
        Self {
            relationship_type,
            metadata: HashMap::new(),
            bidirectional: false,
        }
    }

    pub fn bidirectional(mut self) -> Self {
        self.bidirectional = true;
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Generic metadata that can be attached to graph elements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Metadata {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub properties: HashMap<String, serde_json::Value>,
}

impl Metadata {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            tags: Vec::new(),
            properties: HashMap::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn with_property(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.properties.insert(key.into(), value);
        self
    }
}

/// Trait for converting between domain types and base types
pub trait DomainMapping<T> {
    fn to_base(&self) -> T;
    fn from_base(base: &T) -> Option<Self>
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_id_creation() {
        let id1: GraphId = GraphId::new();
        let id2: GraphId = GraphId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_node_id_is_not_entity() {
        // NodeId and EdgeId are simple value objects, not entities
        let node1 = NodeId::new();
        let node2 = NodeId::new();
        assert_ne!(node1, node2); // Different instances have different IDs

        // But they don't have the phantom type safety of EntityId
        // This is intentional - nodes only have meaning within their graph
    }

    #[test]
    fn test_base_node_types() {
        let value_node = BaseNodeType::Value;
        assert_eq!(value_node.to_string(), "Value");

        let entity_ref = BaseNodeType::EntityReference;
        assert_eq!(entity_ref.to_string(), "EntityReference");
    }

    #[test]
    fn test_relationship_creation() {
        let rel = Relationship::new(BaseRelationshipType::Contains)
            .bidirectional()
            .with_metadata("weight".to_string(), serde_json::json!(1.0));

        assert_eq!(rel.relationship_type, BaseRelationshipType::Contains);
        assert!(rel.bidirectional);
        assert_eq!(rel.metadata.get("weight"), Some(&serde_json::json!(1.0)));
    }

    #[test]
    fn test_metadata_builder() {
        let metadata = Metadata::new("TestNode")
            .with_description("A test node")
            .with_tag("test")
            .with_tag("example")
            .with_property("priority", serde_json::json!(5));

        assert_eq!(metadata.name, "TestNode");
        assert_eq!(metadata.description, Some("A test node".to_string()));
        assert_eq!(metadata.tags.len(), 2);
        assert_eq!(metadata.properties.get("priority"), Some(&serde_json::json!(5)));
    }
}
