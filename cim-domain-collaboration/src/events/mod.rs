//! Collaboration domain events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use cim_domain::{GraphId, NodeId, EdgeId};
use crate::aggregate::UserId;

/// Position of a user's cursor in the graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CursorPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// User presence information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserPresence {
    pub user_id: UserId,
    pub user_name: String,
    pub color: String,
    pub cursor_position: Option<CursorPosition>,
    pub selected_nodes: Vec<NodeId>,
    pub selected_edges: Vec<EdgeId>,
    pub last_activity: DateTime<Utc>,
}

/// Selection state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelectionState {
    pub nodes: Vec<NodeId>,
    pub edges: Vec<EdgeId>,
}

/// Collaboration events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CollaborationEvent {
    /// User joined a collaborative session
    UserJoinedSession {
        session_id: Uuid,
        graph_id: GraphId,
        user_id: UserId,
        user_name: String,
        color: String,
        timestamp: DateTime<Utc>,
    },

    /// User left a collaborative session
    UserLeftSession {
        session_id: Uuid,
        graph_id: GraphId,
        user_id: UserId,
        timestamp: DateTime<Utc>,
    },

    /// User's cursor moved
    CursorMoved {
        session_id: Uuid,
        graph_id: GraphId,
        user_id: UserId,
        position: CursorPosition,
        timestamp: DateTime<Utc>,
    },

    /// User's selection changed
    SelectionChanged {
        session_id: Uuid,
        graph_id: GraphId,
        user_id: UserId,
        selection: SelectionState,
        timestamp: DateTime<Utc>,
    },

    /// User started editing an element
    EditingStarted {
        session_id: Uuid,
        graph_id: GraphId,
        user_id: UserId,
        element_type: ElementType,
        element_id: String,
        timestamp: DateTime<Utc>,
    },

    /// User finished editing an element
    EditingFinished {
        session_id: Uuid,
        graph_id: GraphId,
        user_id: UserId,
        element_type: ElementType,
        element_id: String,
        timestamp: DateTime<Utc>,
    },

    /// Conflict detected between operations
    ConflictDetected {
        session_id: Uuid,
        graph_id: GraphId,
        user_id: UserId,
        conflict_type: ConflictType,
        resolution: ConflictResolution,
        timestamp: DateTime<Utc>,
    },

    /// Session state synchronized
    SessionSynchronized {
        session_id: Uuid,
        graph_id: GraphId,
        active_users: Vec<UserPresence>,
        timestamp: DateTime<Utc>,
    },
}

/// Type of element being edited
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ElementType {
    Node,
    Edge,
    Graph,
}

/// Types of conflicts that can occur
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    /// Two users editing the same node
    NodeEditConflict { node_id: NodeId },
    /// Two users editing the same edge
    EdgeEditConflict { edge_id: EdgeId },
    /// Node was deleted while being edited
    DeletedWhileEditing { element_id: String },
    /// Position conflict
    PositionConflict { element_id: String },
}

/// How a conflict was resolved
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictResolution {
    /// Last write wins
    LastWriteWins { winning_user: UserId },
    /// Merged using CRDT
    CrdtMerge { merged_value: serde_json::Value },
    /// User intervention required
    RequiresUserIntervention,
}

impl CollaborationEvent {
    /// Get the session ID for this event
    pub fn session_id(&self) -> Uuid {
        match self {
            Self::UserJoinedSession { session_id, .. } => *session_id,
            Self::UserLeftSession { session_id, .. } => *session_id,
            Self::CursorMoved { session_id, .. } => *session_id,
            Self::SelectionChanged { session_id, .. } => *session_id,
            Self::EditingStarted { session_id, .. } => *session_id,
            Self::EditingFinished { session_id, .. } => *session_id,
            Self::ConflictDetected { session_id, .. } => *session_id,
            Self::SessionSynchronized { session_id, .. } => *session_id,
        }
    }

    /// Get the graph ID for this event
    pub fn graph_id(&self) -> GraphId {
        match self {
            Self::UserJoinedSession { graph_id, .. } => graph_id.clone(),
            Self::UserLeftSession { graph_id, .. } => graph_id.clone(),
            Self::CursorMoved { graph_id, .. } => graph_id.clone(),
            Self::SelectionChanged { graph_id, .. } => graph_id.clone(),
            Self::EditingStarted { graph_id, .. } => graph_id.clone(),
            Self::EditingFinished { graph_id, .. } => graph_id.clone(),
            Self::ConflictDetected { graph_id, .. } => graph_id.clone(),
            Self::SessionSynchronized { graph_id, .. } => graph_id.clone(),
        }
    }

    /// Get the timestamp for this event
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::UserJoinedSession { timestamp, .. } => *timestamp,
            Self::UserLeftSession { timestamp, .. } => *timestamp,
            Self::CursorMoved { timestamp, .. } => *timestamp,
            Self::SelectionChanged { timestamp, .. } => *timestamp,
            Self::EditingStarted { timestamp, .. } => *timestamp,
            Self::EditingFinished { timestamp, .. } => *timestamp,
            Self::ConflictDetected { timestamp, .. } => *timestamp,
            Self::SessionSynchronized { timestamp, .. } => *timestamp,
        }
    }
}