//! User interaction events that may aggregate into domain commands
//!
//! These events capture user input but don't immediately translate to
//! domain state changes. They are aggregated and converted when appropriate.

use bevy::prelude::*;
use crate::domain::value_objects::{NodeId, EdgeId, GraphId};
use super::PresentationEvent;

/// Mouse hover state changes
#[derive(Event, Clone, Debug)]
pub struct HoverStateChanged {
    pub entity: Entity,
    pub hovered: bool,
    pub world_position: Vec3,
}

impl PresentationEvent for HoverStateChanged {
    fn requires_aggregation(&self) -> bool {
        false // Hover never affects domain
    }
}

/// Drag operation in progress
#[derive(Event, Clone, Debug)]
pub struct DragUpdate {
    pub entity: Entity,
    pub node_id: Option<NodeId>,
    pub start_position: Vec3,
    pub current_position: Vec3,
    pub world_delta: Vec3,
}

impl PresentationEvent for DragUpdate {
    fn requires_aggregation(&self) -> bool {
        true // Aggregate into final position change
    }
}

/// Drag operation completed
#[derive(Event, Clone, Debug)]
pub struct DragComplete {
    pub entity: Entity,
    pub node_id: Option<NodeId>,
    pub start_position: Vec3,
    pub final_position: Vec3,
    pub total_distance: f32,
}

impl PresentationEvent for DragComplete {}

/// Individual entity selection state changes
#[derive(Event, Clone, Debug)]
pub struct EntitySelectionChanged {
    pub entity: Entity,
    pub selected: bool,
    pub selection_type: SelectionType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SelectionType {
    Single,
    AddToSelection,
    RemoveFromSelection,
    Toggle,
    RectangleSelect,
}

impl PresentationEvent for EntitySelectionChanged {}

/// Aggregated selection changes for multiple nodes
#[derive(Event, Clone, Debug)]
pub struct SelectionChanged {
    pub selected_nodes: Vec<NodeId>,
    pub selection_mode: crate::presentation::aggregators::selection::SelectionMode,
}

impl PresentationEvent for SelectionChanged {
    fn requires_aggregation(&self) -> bool {
        true
    }
}

/// Preview state for potential operations
#[derive(Event, Clone, Debug)]
pub struct PreviewStateChanged {
    pub preview_type: PreviewType,
    pub active: bool,
}

#[derive(Clone, Debug)]
pub enum PreviewType {
    EdgeConnection { source: NodeId, target_position: Vec3 },
    NodeCreation { position: Vec3, node_type: String },
    Deletion { entities: Vec<Entity> },
    Morphism { graph_id: GraphId, morphism_type: String },
}

impl PresentationEvent for PreviewStateChanged {
    fn requires_aggregation(&self) -> bool {
        false // Previews don't affect domain until confirmed
    }
}

/// Temporary visual feedback
#[derive(Event, Clone, Debug)]
pub struct VisualFeedback {
    pub feedback_type: FeedbackType,
    pub position: Vec3,
    pub duration: f32,
}

#[derive(Clone, Debug)]
pub enum FeedbackType {
    Click,
    DoubleClick,
    RightClick,
    InvalidOperation,
    ValidTarget,
    InvalidTarget,
}

impl PresentationEvent for VisualFeedback {
    fn requires_aggregation(&self) -> bool {
        false // Pure visual feedback
    }
}

/// UI panel state changes
#[derive(Event, Clone, Debug)]
pub struct UIPanelStateChanged {
    pub panel_id: String,
    pub state: UIPanelState,
}

#[derive(Clone, Debug)]
pub enum UIPanelState {
    Opened,
    Closed,
    Minimized,
    Maximized,
    Docked { position: String },
}

impl PresentationEvent for UIPanelStateChanged {
    fn requires_aggregation(&self) -> bool {
        false // UI state is presentation-only
    }
}

/// Context menu events
#[derive(Event, Clone, Debug)]
pub struct ContextMenuEvent {
    pub menu_type: ContextMenuType,
    pub position: Vec2,
    pub world_position: Vec3,
    pub target_entity: Option<Entity>,
}

#[derive(Clone, Debug)]
pub enum ContextMenuType {
    NodeMenu { node_id: NodeId },
    EdgeMenu { edge_id: EdgeId },
    CanvasMenu,
    SelectionMenu { selected_count: usize },
}

impl PresentationEvent for ContextMenuEvent {}

/// Drag operation started
#[derive(Event, Clone, Debug)]
pub struct DragStart {
    pub entity: Entity,
    pub node_id: Option<NodeId>,
    pub start_position: Vec3,
    pub is_multi_select: bool,
}

impl PresentationEvent for DragStart {
    fn requires_aggregation(&self) -> bool {
        true
    }
}

/// Drag operation ended
#[derive(Event, Clone, Debug)]
pub struct DragEnd {
    pub entity: Entity,
    pub node_id: Option<NodeId>,
    pub end_position: Vec3,
    pub cancelled: bool,
}

impl PresentationEvent for DragEnd {
    fn requires_aggregation(&self) -> bool {
        true
    }
}

/// Selection cleared event
#[derive(Event, Clone, Debug)]
pub struct SelectionCleared;

impl PresentationEvent for SelectionCleared {
    fn requires_aggregation(&self) -> bool {
        true
    }
}
