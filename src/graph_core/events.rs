use super::components::{DomainEdgeType, DomainNodeType};
use bevy::prelude::*;
use uuid::Uuid;

/// Event for creating a new graph node
#[derive(Event)]
pub struct CreateNodeEvent {
    pub id: Uuid,
    pub position: Vec3,
    pub domain_type: DomainNodeType,
    pub name: String,
    pub subgraph_id: Option<Uuid>,
}

/// Event for moving a node
#[derive(Event)]
pub struct MoveNodeEvent {
    pub entity: Entity,
    pub from: Vec3,
    pub to: Vec3,
}

/// Event for creating a new edge
#[derive(Event)]
pub struct CreateEdgeEvent {
    pub id: Uuid,
    pub source: Entity,
    pub target: Entity,
    pub edge_type: DomainEdgeType,
}

/// Event for deleting nodes
#[derive(Event)]
pub struct DeleteNodeEvent {
    pub entity: Entity,
}

/// Event for deleting edges
#[derive(Event)]
pub struct DeleteEdgeEvent {
    pub entity: Entity,
}

/// Event for selecting entities
#[derive(Event)]
pub struct SelectEvent {
    pub entity: Entity,
    pub multi_select: bool,
}

/// Event for deselecting all
#[derive(Event)]
pub struct DeselectAllEvent;

/// Event for hovering over entities
#[derive(Event)]
pub struct HoverEvent {
    pub entity: Option<Entity>,
}

/// Event for graph layout updates
#[derive(Event)]
pub struct LayoutUpdateEvent {
    pub layout_type: LayoutType,
}

#[derive(Debug, Clone)]
pub enum LayoutType {
    ForceDirected,
    Hierarchical,
    Circular,
    Grid,
}

/// Event for graph validation
#[derive(Event)]
pub struct ValidateGraphEvent;

/// Event for graph persistence
#[derive(Event)]
pub struct SaveGraphEvent {
    pub path: String,
}

/// Event for graph loading
#[derive(Event)]
pub struct LoadGraphEvent {
    pub path: String,
}

/// Event for subgraph creation
#[derive(Event)]
pub struct CreateSubgraphEvent {
    pub id: Uuid,
    pub name: String,
    pub nodes: Vec<Entity>,
}

/// Event for undo operations
#[derive(Event)]
pub struct UndoEvent;

/// Event for redo operations
#[derive(Event)]
pub struct RedoEvent;

/// Event for tracking graph modifications (for undo/redo)
#[derive(Event, Debug, Clone)]
pub enum GraphModificationEvent {
    NodeCreated {
        id: Uuid,
        position: Vec3,
        domain_type: DomainNodeType,
        name: String,
    },
    NodeMoved {
        id: Uuid,
        from: Vec3,
        to: Vec3,
    },
    NodeDeleted {
        id: Uuid,
    },
    EdgeCreated {
        id: Uuid,
        source_id: Uuid,
        target_id: Uuid,
        edge_type: DomainEdgeType,
    },
    EdgeDeleted {
        id: Uuid,
    },
    GraphCleared,
}

impl GraphModificationEvent {
    /// Get a timestamp for the event
    pub fn timestamp(&self) -> std::time::SystemTime {
        std::time::SystemTime::now()
    }

    /// Convert to a serializable format for persistence
    pub fn to_persistence_format(&self) -> String {
        // This would be implemented with proper serialization
        format!("{:?}", self)
    }
}

// TODO: Implement graph_patterns module
/*
#[derive(Event)]
pub struct CreatePatternEvent {
    pub pattern: crate::graph_patterns::GraphPattern,
    pub name: String,
    pub position: Vec3,
}

#[derive(Event)]
pub struct ApplyPatternEvent {
    pub position: Vec3,
    pub pattern: crate::graph_patterns::GraphPattern,
}
*/
