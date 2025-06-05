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
