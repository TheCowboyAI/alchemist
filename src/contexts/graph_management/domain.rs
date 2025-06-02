use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============= Value Objects =============

/// Unique identifier for a graph
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphIdentity(pub Uuid);

impl GraphIdentity {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for GraphIdentity {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for a node
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeIdentity(pub Uuid);

impl NodeIdentity {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for NodeIdentity {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for an edge
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeIdentity(pub Uuid);

impl EdgeIdentity {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EdgeIdentity {
    fn default() -> Self {
        Self::new()
    }
}

/// Graph metadata
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub name: String,
    pub description: String,
    pub domain: String,
    pub created: std::time::SystemTime,
    pub modified: std::time::SystemTime,
    pub tags: Vec<String>,
}

/// Node content and properties
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct NodeContent {
    pub label: String,
    pub category: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Edge relationship information
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRelationship {
    pub source: NodeIdentity,
    pub target: NodeIdentity,
    pub category: String,
    pub strength: f32,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Spatial position in 3D space
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SpatialPosition {
    pub coordinates_3d: Vec3,
    pub coordinates_2d: Vec2,
}

impl SpatialPosition {
    pub fn at_3d(x: f32, y: f32, z: f32) -> Self {
        Self {
            coordinates_3d: Vec3::new(x, y, z),
            coordinates_2d: Vec2::new(x, y),
        }
    }
}

/// Graph journey tracking
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct GraphJourney {
    pub version: u64,
    pub event_count: u64,
    pub last_event: Option<Uuid>, // EventIdentity
}

impl Default for GraphJourney {
    fn default() -> Self {
        Self {
            version: 1,
            event_count: 0,
            last_event: None,
        }
    }
}

// ============= Entities =============

/// Graph aggregate root
#[derive(Component, Debug, Clone)]
pub struct Graph {
    pub identity: GraphIdentity,
    pub metadata: GraphMetadata,
    pub journey: GraphJourney,
}

/// Node entity
#[derive(Component, Debug, Clone)]
pub struct Node {
    pub identity: NodeIdentity,
    pub graph: GraphIdentity,
    pub content: NodeContent,
    pub position: SpatialPosition,
}

/// Edge entity
#[derive(Component, Debug, Clone)]
pub struct Edge {
    pub identity: EdgeIdentity,
    pub graph: GraphIdentity,
    pub relationship: EdgeRelationship,
}

// ============= Component Bundles =============

/// Bundle for spawning graph entities
#[derive(Bundle)]
pub struct GraphBundle {
    pub graph: Graph,
    pub identity: GraphIdentity,
    pub metadata: GraphMetadata,
    pub journey: GraphJourney,
}

/// Bundle for spawning node entities
#[derive(Bundle)]
pub struct NodeBundle {
    pub node: Node,
    pub identity: NodeIdentity,
    pub position: SpatialPosition,
    pub content: NodeContent,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

/// Bundle for spawning edge entities
#[derive(Bundle)]
pub struct EdgeBundle {
    pub edge: Edge,
    pub identity: EdgeIdentity,
    pub relationship: EdgeRelationship,
}
