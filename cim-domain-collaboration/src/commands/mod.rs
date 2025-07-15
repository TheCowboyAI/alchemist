//! Collaboration domain commands

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use cim_domain::GraphId;
use crate::aggregate::UserId;

use crate::events::{CursorPosition, SelectionState, ElementType};

/// Errors that can occur when processing collaboration commands
#[derive(Debug, Error)]
pub enum CollaborationCommandError {
    #[error("Session not found: {0}")]
    SessionNotFound(Uuid),
    
    #[error("User not in session: {0}")]
    UserNotInSession(UserId),
    
    #[error("Element is locked by another user: {element_id} (locked by {user_id})")]
    ElementLocked { element_id: String, user_id: UserId },
    
    #[error("Invalid graph ID: {0}")]
    InvalidGraphId(String),
    
    #[error("Session full (max {max} users)")]
    SessionFull { max: usize },
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Commands for the collaboration domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborationCommand {
    /// Join a collaborative session for a graph
    JoinSession {
        graph_id: GraphId,
        user_id: UserId,
        user_name: String,
    },

    /// Leave a collaborative session
    LeaveSession {
        session_id: Uuid,
        user_id: UserId,
    },

    /// Update cursor position
    UpdateCursor {
        session_id: Uuid,
        user_id: UserId,
        position: CursorPosition,
    },

    /// Update selection
    UpdateSelection {
        session_id: Uuid,
        user_id: UserId,
        selection: SelectionState,
    },

    /// Request to start editing an element
    StartEditing {
        session_id: Uuid,
        user_id: UserId,
        element_type: ElementType,
        element_id: String,
    },

    /// Finish editing an element
    FinishEditing {
        session_id: Uuid,
        user_id: UserId,
        element_type: ElementType,
        element_id: String,
    },

    /// Synchronize session state
    SynchronizeSession {
        session_id: Uuid,
    },

    /// Clean up inactive sessions
    CleanupInactiveSessions {
        inactive_threshold_minutes: i64,
    },
}

impl CollaborationCommand {
    /// Get the session ID if this command is session-specific
    pub fn session_id(&self) -> Option<Uuid> {
        match self {
            Self::JoinSession { .. } => None,
            Self::LeaveSession { session_id, .. } => Some(*session_id),
            Self::UpdateCursor { session_id, .. } => Some(*session_id),
            Self::UpdateSelection { session_id, .. } => Some(*session_id),
            Self::StartEditing { session_id, .. } => Some(*session_id),
            Self::FinishEditing { session_id, .. } => Some(*session_id),
            Self::SynchronizeSession { session_id } => Some(*session_id),
            Self::CleanupInactiveSessions { .. } => None,
        }
    }

    /// Get the user ID if this command is user-specific
    pub fn user_id(&self) -> Option<&UserId> {
        match self {
            Self::JoinSession { user_id, .. } => Some(user_id),
            Self::LeaveSession { user_id, .. } => Some(user_id),
            Self::UpdateCursor { user_id, .. } => Some(user_id),
            Self::UpdateSelection { user_id, .. } => Some(user_id),
            Self::StartEditing { user_id, .. } => Some(user_id),
            Self::FinishEditing { user_id, .. } => Some(user_id),
            Self::SynchronizeSession { .. } => None,
            Self::CleanupInactiveSessions { .. } => None,
        }
    }
}