use super::domain::SelectionMode;
use crate::contexts::graph_management::domain::{EdgeIdentity, NodeIdentity};
use bevy::prelude::*;

// ============= Selection Events =============

/// Event fired when selection state changes
#[derive(Event, Debug, Clone)]
pub struct SelectionChanged {
    pub added_nodes: Vec<NodeIdentity>,
    pub removed_nodes: Vec<NodeIdentity>,
    pub added_edges: Vec<EdgeIdentity>,
    pub removed_edges: Vec<EdgeIdentity>,
}

/// Event fired when all selections are cleared
#[derive(Event, Debug, Clone)]
pub struct SelectionCleared;

/// Event fired when selection mode changes
#[derive(Event, Debug, Clone)]
pub struct SelectionModeChanged {
    pub new_mode: SelectionMode,
    pub previous_mode: SelectionMode,
}

/// Event fired when a node was selected
#[derive(Event, Debug, Clone)]
pub struct NodeSelected {
    pub entity: Entity,
    pub node: NodeIdentity,
    pub add_to_selection: bool, // If true, add to existing selection
}

/// Event fired when a node was deselected
#[derive(Event, Debug, Clone)]
pub struct NodeDeselected {
    pub entity: Entity,
    pub node: NodeIdentity,
}

/// Event fired when an edge was selected
#[derive(Event, Debug, Clone)]
pub struct EdgeSelected {
    pub entity: Entity,
    pub edge: EdgeIdentity,
    pub add_to_selection: bool,
}

/// Event fired when an edge was deselected
#[derive(Event, Debug, Clone)]
pub struct EdgeDeselected {
    pub entity: Entity,
    pub edge: EdgeIdentity,
}

/// Event fired when box selection started
#[derive(Event, Debug, Clone)]
pub struct BoxSelectionStarted {
    pub start_position: Vec2,
}

/// Event fired when box selection was updated
#[derive(Event, Debug, Clone)]
pub struct BoxSelectionUpdated {
    pub current_position: Vec2,
}

/// Event fired when box selection completed
#[derive(Event, Debug, Clone)]
pub struct BoxSelectionCompleted {
    pub end_position: Vec2,
}

/// Event fired when box selection was cancelled
#[derive(Event, Debug, Clone)]
pub struct BoxSelectionCancelled;

/// Event fired when an entity is hovered
#[derive(Event, Debug, Clone)]
pub struct EntityHovered {
    pub entity: Entity,
    pub is_node: bool,
    pub is_edge: bool,
}

/// Event fired when hover ends
#[derive(Event, Debug, Clone)]
pub struct EntityUnhovered {
    pub entity: Entity,
}

/// Event fired when all visible entities were selected
#[derive(Event, Debug, Clone)]
pub struct AllSelected;

/// Event fired when selection was inverted
#[derive(Event, Debug, Clone)]
pub struct SelectionInverted;

/// Event fired when connected nodes were selected
#[derive(Event, Debug, Clone)]
pub struct ConnectedNodesSelected {
    pub from_node: NodeIdentity,
    pub depth: u32, // How many hops to select
}
