//! Value Objects for the Domain Layer

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for a graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphId(pub Uuid);

impl GraphId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for GraphId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for GraphId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for GraphId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Unique identifier for a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Component)]
pub struct NodeId(pub Uuid);

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

/// Unique identifier for an edge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Component)]
pub struct EdgeId(pub Uuid);

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

/// 3D position in space
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Component)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl Default for Position3D {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl From<Position3D> for Vec3 {
    fn from(pos: Position3D) -> Self {
        Vec3::new(pos.x, pos.y, pos.z)
    }
}

impl From<Vec3> for Position3D {
    fn from(vec: Vec3) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
            z: vec.z,
        }
    }
}

/// Content of a node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeContent {
    pub label: String,
    pub node_type: NodeType,
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

/// Types of nodes in the graph
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    // DDD types
    Entity,
    ValueObject,
    Aggregate,
    Service,
    Repository,
    Factory,
    Event,
    Command,
    Query,
    Policy,

    // Progress tracking types
    Milestone,
    Phase,
    Task,

    // Git types
    GitCommit,
    GitBranch,
    GitTag,
    GitMerge,

    // Generic
    Custom(String),
}

/// Relationship between nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeRelationship {
    pub relationship_type: RelationshipType,
    pub properties: std::collections::HashMap<String, serde_json::Value>,
    pub bidirectional: bool,
}

/// Types of relationships
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipType {
    // DDD relationships
    Contains,
    References,
    DependsOn,
    Publishes,
    Subscribes,
    Implements,
    Extends,

    // Git relationships
    Parent,
    Merged,
    Branched,
    Tagged,

    // Progress relationships
    Sequence,
    Hierarchy,
    Blocks,

    // Generic
    Custom(String),
}

/// Metadata for a graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub name: String,
    pub bounded_context: String,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
    pub tags: Vec<String>,
}

impl GraphMetadata {
    pub fn new(name: String) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            name,
            bounded_context: String::from("default"),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
        }
    }
}

impl Default for GraphMetadata {
    fn default() -> Self {
        Self::new(String::from("default"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_id_creation() {
        let id1 = GraphId::new();
        let id2 = GraphId::new();

        // Each ID should be unique
        assert_ne!(id1, id2);

        // Display should work
        assert!(!id1.to_string().is_empty());
    }

    #[test]
    fn test_graph_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id = GraphId::from(uuid);

        assert_eq!(id.0, uuid);
        assert_eq!(id.to_string(), uuid.to_string());
    }

    #[test]
    fn test_node_id_uniqueness() {
        let id1 = NodeId::new();
        let id2 = NodeId::new();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_edge_id_default() {
        let id1 = EdgeId::default();
        let id2 = EdgeId::default();

        // Default should create new unique IDs
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_position3d_creation() {
        let pos = Position3D::new(1.0, 2.0, 3.0);

        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
        assert_eq!(pos.z, 3.0);
    }

    #[test]
    fn test_position3d_default() {
        let pos = Position3D::default();

        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, 0.0);
        assert_eq!(pos.z, 0.0);
    }

    #[test]
    fn test_position3d_vec3_conversion() {
        let pos = Position3D::new(1.0, 2.0, 3.0);
        let vec: Vec3 = pos.into();

        assert_eq!(vec.x, 1.0);
        assert_eq!(vec.y, 2.0);
        assert_eq!(vec.z, 3.0);

        let pos2: Position3D = vec.into();
        assert_eq!(pos2.x, 1.0);
        assert_eq!(pos2.y, 2.0);
        assert_eq!(pos2.z, 3.0);
    }

    #[test]
    fn test_node_content_creation() {
        let mut properties = std::collections::HashMap::new();
        properties.insert("key".to_string(), serde_json::json!("value"));

        let content = NodeContent {
            label: "Test Node".to_string(),
            node_type: NodeType::Entity,
            properties: properties.clone(),
        };

        assert_eq!(content.label, "Test Node");
        assert_eq!(content.node_type, NodeType::Entity);
        assert_eq!(content.properties, properties);
    }

    #[test]
    fn test_node_type_variants() {
        // Test equality
        assert_eq!(NodeType::Entity, NodeType::Entity);
        assert_ne!(NodeType::Entity, NodeType::ValueObject);

        // Test custom type
        let custom = NodeType::Custom("MyType".to_string());
        match custom {
            NodeType::Custom(name) => assert_eq!(name, "MyType"),
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn test_edge_relationship_creation() {
        let mut properties = std::collections::HashMap::new();
        properties.insert("weight".to_string(), serde_json::json!(1.0));

        let rel = EdgeRelationship {
            relationship_type: RelationshipType::DependsOn,
            properties: properties.clone(),
            bidirectional: false,
        };

        assert_eq!(rel.relationship_type, RelationshipType::DependsOn);
        assert_eq!(rel.properties, properties);
        assert!(!rel.bidirectional);
    }

    #[test]
    fn test_relationship_type_variants() {
        // Test DDD relationships
        assert_eq!(RelationshipType::Contains, RelationshipType::Contains);
        assert_ne!(RelationshipType::Contains, RelationshipType::References);

        // Test custom relationship
        let custom = RelationshipType::Custom("MyRelation".to_string());
        match custom {
            RelationshipType::Custom(name) => assert_eq!(name, "MyRelation"),
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn test_graph_metadata_creation() {
        let name = "Test Graph".to_string();
        let metadata = GraphMetadata::new(name.clone());

        assert_eq!(metadata.name, name);
        assert_eq!(metadata.bounded_context, "default");
        assert!(metadata.tags.is_empty());

        // Times should be set
        assert!(metadata.created_at <= std::time::SystemTime::now());
        assert_eq!(metadata.created_at, metadata.updated_at);
    }

    #[test]
    fn test_graph_metadata_default() {
        let metadata = GraphMetadata::default();

        assert_eq!(metadata.name, "default");
        assert_eq!(metadata.bounded_context, "default");
        assert!(metadata.tags.is_empty());
    }

    #[test]
    fn test_value_object_immutability() {
        // Value objects should be immutable after creation
        let id = GraphId::new();
        let id_copy = id; // Copy semantics

        assert_eq!(id, id_copy);

        // Position is also a value object
        let pos = Position3D::new(1.0, 2.0, 3.0);
        let pos_copy = pos;

        assert_eq!(pos, pos_copy);
    }

    #[test]
    fn test_serialization_roundtrip() {
        // Test GraphId serialization
        let id = GraphId::new();
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: GraphId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);

        // Test Position3D serialization
        let pos = Position3D::new(1.5, 2.5, 3.5);
        let serialized = serde_json::to_string(&pos).unwrap();
        let deserialized: Position3D = serde_json::from_str(&serialized).unwrap();
        assert_eq!(pos, deserialized);

        // Test NodeType serialization
        let node_type = NodeType::Custom("TestType".to_string());
        let serialized = serde_json::to_string(&node_type).unwrap();
        let deserialized: NodeType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(node_type, deserialized);
    }
}
