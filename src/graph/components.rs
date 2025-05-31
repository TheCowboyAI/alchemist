//! Graph component definitions for ECS
//!
//! This module defines the core components for the graph aggregate.
//! Following Domain-Driven Design principles, graphs are treated as
//! first-class entities with their own identity and metadata.

use bevy::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a graph aggregate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct GraphId(pub Uuid);

impl GraphId {
    /// Create a new unique graph identifier
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for GraphId {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata associated with a graph
#[derive(Debug, Clone, Component)]
pub struct GraphMetadata {
    /// Human-readable name of the graph
    pub name: String,
    /// Description of the graph's purpose
    pub description: String,
    /// Domain type (e.g., "knowledge", "workflow", "supply-chain")
    pub domain_type: String,
    /// Version number for tracking changes
    pub version: u64,
    /// Creation timestamp
    pub created_at: std::time::SystemTime,
    /// Last modification timestamp
    pub modified_at: std::time::SystemTime,
    /// Custom properties for domain-specific needs
    pub properties: HashMap<String, String>,
}

impl Default for GraphMetadata {
    fn default() -> Self {
        let now = std::time::SystemTime::now();
        Self {
            name: "Untitled Graph".to_string(),
            description: String::new(),
            domain_type: "generic".to_string(),
            version: 1,
            created_at: now,
            modified_at: now,
            properties: HashMap::new(),
        }
    }
}

/// Component marking an entity as a graph aggregate root
#[derive(Debug, Component)]
pub struct Graph;

/// Unique identifier for nodes within a graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
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

/// Component representing a node in the graph
#[derive(Debug, Clone, Component)]
pub struct GraphNode {
    /// The graph this node belongs to
    pub graph_id: GraphId,
    /// Position in 3D space for visualization
    pub position: Vec3,
    /// Node-specific properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Unique identifier for edges within a graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
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

/// Component representing an edge in the graph
#[derive(Debug, Clone, Component)]
pub struct GraphEdge {
    /// The graph this edge belongs to
    pub graph_id: GraphId,
    /// Source node ID
    pub source: NodeId,
    /// Target node ID
    pub target: NodeId,
    /// Edge weight or strength
    pub weight: f32,
    /// Edge-specific properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Visual state for graph elements
#[derive(Debug, Clone, Component)]
pub enum ElementState {
    /// Normal, unselected state
    Normal,
    /// Currently selected
    Selected,
    /// Mouse hovering over element
    Hovered,
    /// Element is being dragged
    Dragging,
    /// Element is highlighted for some reason
    Highlighted(Color),
}

impl Default for ElementState {
    fn default() -> Self {
        Self::Normal
    }
}

/// Component for elements that can be selected
#[derive(Debug, Component)]
pub struct Selectable;

/// Component for elements that can be dragged
#[derive(Debug, Component)]
pub struct Draggable;

/// Bundle for creating a graph entity
#[derive(Bundle)]
pub struct GraphBundle {
    pub graph: Graph,
    pub id: GraphId,
    pub metadata: GraphMetadata,
}

impl Default for GraphBundle {
    fn default() -> Self {
        Self {
            graph: Graph,
            id: GraphId::default(),
            metadata: GraphMetadata::default(),
        }
    }
}

/// Bundle for creating a node entity
#[derive(Bundle)]
pub struct NodeBundle {
    pub node: GraphNode,
    pub id: NodeId,
    pub state: ElementState,
    pub selectable: Selectable,
    pub draggable: Draggable,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
}

/// Bundle for creating an edge entity
#[derive(Bundle)]
pub struct EdgeBundle {
    pub edge: GraphEdge,
    pub id: EdgeId,
    pub state: ElementState,
    pub selectable: Selectable,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
}
