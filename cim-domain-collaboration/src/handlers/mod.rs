//! Collaboration command handlers

use chrono::{Duration, Utc};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use cim_domain::GraphId;
use crate::aggregate::UserId;
// EventStore would come from cim_infrastructure
// For now, using a simple trait
pub trait EventStore: Send + Sync {
    // Event store methods would go here
}

use crate::{
    aggregate::CollaborationSession,
    commands::{CollaborationCommand, CollaborationCommandError},
    events::CollaborationEvent,
};

/// Handler for collaboration commands
pub struct CollaborationCommandHandler {
    /// Active collaboration sessions
    sessions: Arc<DashMap<Uuid, CollaborationSession>>,
    /// Map from graph ID to session ID
    graph_sessions: Arc<DashMap<GraphId, Uuid>>,
    // Event store would be used for persistence
    // For now, we'll just store in memory
    /// Maximum users per session
    max_users_per_session: usize,
}

impl CollaborationCommandHandler {
    /// Create a new collaboration command handler
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            graph_sessions: Arc::new(DashMap::new()),
            max_users_per_session: 50,
        }
    }

    /// Handle a collaboration command
    pub async fn handle(&self, command: CollaborationCommand) -> Result<Vec<CollaborationEvent>, CollaborationCommandError> {
        match command {
            CollaborationCommand::JoinSession { graph_id, user_id, user_name } => {
                self.handle_join_session(graph_id, user_id, user_name).await
            }
            CollaborationCommand::LeaveSession { session_id, user_id } => {
                self.handle_leave_session(session_id, user_id).await
            }
            CollaborationCommand::UpdateCursor { session_id, user_id, position } => {
                self.handle_update_cursor(session_id, user_id, position).await
            }
            CollaborationCommand::UpdateSelection { session_id, user_id, selection } => {
                self.handle_update_selection(session_id, user_id, selection).await
            }
            CollaborationCommand::StartEditing { session_id, user_id, element_type, element_id } => {
                self.handle_start_editing(session_id, user_id, element_type, element_id).await
            }
            CollaborationCommand::FinishEditing { session_id, user_id, element_type, element_id } => {
                self.handle_finish_editing(session_id, user_id, element_type, element_id).await
            }
            CollaborationCommand::SynchronizeSession { session_id } => {
                self.handle_synchronize_session(session_id).await
            }
            CollaborationCommand::CleanupInactiveSessions { inactive_threshold_minutes } => {
                self.handle_cleanup_inactive_sessions(inactive_threshold_minutes).await
            }
        }
    }

    async fn handle_join_session(
        &self,
        graph_id: GraphId,
        user_id: UserId,
        user_name: String,
    ) -> Result<Vec<CollaborationEvent>, CollaborationCommandError> {
        // Check if a session already exists for this graph
        let session_id = if let Some(existing_session_id) = self.graph_sessions.get(&graph_id) {
            let session_id = *existing_session_id;
            
            // Check if session is full
            if let Some(session) = self.sessions.get(&session_id) {
                if session.user_count() >= self.max_users_per_session {
                    return Err(CollaborationCommandError::SessionFull {
                        max: self.max_users_per_session,
                    });
                }
            }
            
            session_id
        } else {
            // Create new session
            let session = CollaborationSession::new(graph_id.clone());
            let session_id = session.session_id;
            self.sessions.insert(session_id, session);
            self.graph_sessions.insert(graph_id.clone(), session_id);
            info!("Created new collaboration session {} for graph {}", session_id, graph_id);
            session_id
        };

        // Get session and generate color
        let color = self.sessions
            .get(&session_id)
            .map(|s| s.generate_user_color())
            .unwrap_or_else(|| "#FF6B6B".to_string());

        let event = CollaborationEvent::UserJoinedSession {
            session_id,
            graph_id,
            user_id,
            user_name,
            color,
            timestamp: Utc::now(),
        };

        // Apply event to session
        if let Some(mut session) = self.sessions.get_mut(&session_id) {
            session.apply_event(&event);
        }

        // Store event
        self.store_event(&event).await?;

        Ok(vec![event])
    }

    async fn handle_leave_session(
        &self,
        session_id: Uuid,
        user_id: UserId,
    ) -> Result<Vec<CollaborationEvent>, CollaborationCommandError> {
        let mut session = self.sessions
            .get_mut(&session_id)
            .ok_or(CollaborationCommandError::SessionNotFound(session_id))?;

        if !session.has_user(&user_id) {
            return Err(CollaborationCommandError::UserNotInSession(user_id));
        }

        let event = CollaborationEvent::UserLeftSession {
            session_id,
            graph_id: session.graph_id.clone(),
            user_id,
            timestamp: Utc::now(),
        };

        session.apply_event(&event);

        // Remove session if no users left
        if session.user_count() == 0 {
            let graph_id = session.graph_id.clone();
            drop(session); // Release the lock
            self.sessions.remove(&session_id);
            self.graph_sessions.remove(&graph_id);
            info!("Removed empty collaboration session {}", session_id);
        }

        self.store_event(&event).await?;

        Ok(vec![event])
    }

    async fn handle_update_cursor(
        &self,
        session_id: Uuid,
        user_id: UserId,
        position: crate::events::CursorPosition,
    ) -> Result<Vec<CollaborationEvent>, CollaborationCommandError> {
        let mut session = self.sessions
            .get_mut(&session_id)
            .ok_or(CollaborationCommandError::SessionNotFound(session_id))?;

        if !session.has_user(&user_id) {
            return Err(CollaborationCommandError::UserNotInSession(user_id));
        }

        let event = CollaborationEvent::CursorMoved {
            session_id,
            graph_id: session.graph_id.clone(),
            user_id,
            position,
            timestamp: Utc::now(),
        };

        session.apply_event(&event);
        
        // Don't persist cursor events - they're too frequent
        // Just broadcast them

        Ok(vec![event])
    }

    async fn handle_update_selection(
        &self,
        session_id: Uuid,
        user_id: UserId,
        selection: crate::events::SelectionState,
    ) -> Result<Vec<CollaborationEvent>, CollaborationCommandError> {
        let mut session = self.sessions
            .get_mut(&session_id)
            .ok_or(CollaborationCommandError::SessionNotFound(session_id))?;

        if !session.has_user(&user_id) {
            return Err(CollaborationCommandError::UserNotInSession(user_id));
        }

        let event = CollaborationEvent::SelectionChanged {
            session_id,
            graph_id: session.graph_id.clone(),
            user_id,
            selection,
            timestamp: Utc::now(),
        };

        session.apply_event(&event);
        self.store_event(&event).await?;

        Ok(vec![event])
    }

    async fn handle_start_editing(
        &self,
        session_id: Uuid,
        user_id: UserId,
        element_type: crate::events::ElementType,
        element_id: String,
    ) -> Result<Vec<CollaborationEvent>, CollaborationCommandError> {
        let mut session = self.sessions
            .get_mut(&session_id)
            .ok_or(CollaborationCommandError::SessionNotFound(session_id))?;

        if !session.has_user(&user_id) {
            return Err(CollaborationCommandError::UserNotInSession(user_id));
        }

        // Check if element is already locked
        if let Some(locked_by) = session.is_locked(&element_id) {
            if locked_by != &user_id {
                return Err(CollaborationCommandError::ElementLocked {
                    element_id,
                    user_id: locked_by.clone(),
                });
            }
        }

        let event = CollaborationEvent::EditingStarted {
            session_id,
            graph_id: session.graph_id.clone(),
            user_id,
            element_type,
            element_id,
            timestamp: Utc::now(),
        };

        session.apply_event(&event);
        self.store_event(&event).await?;

        Ok(vec![event])
    }

    async fn handle_finish_editing(
        &self,
        session_id: Uuid,
        user_id: UserId,
        element_type: crate::events::ElementType,
        element_id: String,
    ) -> Result<Vec<CollaborationEvent>, CollaborationCommandError> {
        let mut session = self.sessions
            .get_mut(&session_id)
            .ok_or(CollaborationCommandError::SessionNotFound(session_id))?;

        if !session.has_user(&user_id) {
            return Err(CollaborationCommandError::UserNotInSession(user_id));
        }

        let event = CollaborationEvent::EditingFinished {
            session_id,
            graph_id: session.graph_id.clone(),
            user_id,
            element_type,
            element_id,
            timestamp: Utc::now(),
        };

        session.apply_event(&event);
        self.store_event(&event).await?;

        Ok(vec![event])
    }

    async fn handle_synchronize_session(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<CollaborationEvent>, CollaborationCommandError> {
        let session = self.sessions
            .get(&session_id)
            .ok_or(CollaborationCommandError::SessionNotFound(session_id))?;

        let event = CollaborationEvent::SessionSynchronized {
            session_id,
            graph_id: session.graph_id.clone(),
            active_users: session.get_presences(),
            timestamp: Utc::now(),
        };

        self.store_event(&event).await?;

        Ok(vec![event])
    }

    async fn handle_cleanup_inactive_sessions(
        &self,
        inactive_threshold_minutes: i64,
    ) -> Result<Vec<CollaborationEvent>, CollaborationCommandError> {
        let threshold = Utc::now() - Duration::minutes(inactive_threshold_minutes);
        let mut removed_sessions = Vec::new();

        self.sessions.retain(|session_id, session| {
            if session.last_activity < threshold && session.user_count() == 0 {
                removed_sessions.push((*session_id, session.graph_id.clone()));
                info!("Cleaning up inactive session {}", session_id);
                false
            } else {
                true
            }
        });

        // Clean up graph_sessions mapping
        for (_, graph_id) in &removed_sessions {
            self.graph_sessions.remove(graph_id);
        }

        debug!("Cleaned up {} inactive sessions", removed_sessions.len());

        Ok(vec![]) // No events generated for cleanup
    }

    async fn store_event(&self, event: &CollaborationEvent) -> Result<(), CollaborationCommandError> {
        // Convert to domain event format for storage
        // This would integrate with the existing event store
        // For now, we'll just log it
        debug!("Storing collaboration event: {:?}", event);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Mock event store for tests
    pub struct InMemoryEventStore;
    impl InMemoryEventStore {
        pub fn new() -> Self { Self }
    }

    #[tokio::test]
    async fn test_join_leave_session() {
        let handler = CollaborationCommandHandler::new();

        let graph_id = GraphId::new();
        let user_id = UserId::new();

        // Join session
        let join_result = handler.handle(CollaborationCommand::JoinSession {
            graph_id: graph_id.clone(),
            user_id: user_id.clone(),
            user_name: "Alice".to_string(),
        }).await;

        assert!(join_result.is_ok());
        let events = join_result.unwrap();
        assert_eq!(events.len(), 1);

        let session_id = match &events[0] {
            CollaborationEvent::UserJoinedSession { session_id, .. } => *session_id,
            _ => panic!("Expected UserJoinedSession event"),
        };

        // Leave session
        let leave_result = handler.handle(CollaborationCommand::LeaveSession {
            session_id,
            user_id,
        }).await;

        assert!(leave_result.is_ok());
    }

    #[tokio::test]
    async fn test_editing_locks() {
        let handler = CollaborationCommandHandler::new();

        let graph_id = GraphId::new();
        let user1 = UserId::new();
        let user2 = UserId::new();

        // Both users join
        let join1 = handler.handle(CollaborationCommand::JoinSession {
            graph_id: graph_id.clone(),
            user_id: user1.clone(),
            user_name: "User1".to_string(),
        }).await.unwrap();

        let session_id = match &join1[0] {
            CollaborationEvent::UserJoinedSession { session_id, .. } => *session_id,
            _ => panic!("Expected UserJoinedSession event"),
        };

        handler.handle(CollaborationCommand::JoinSession {
            graph_id,
            user_id: user2.clone(),
            user_name: "User2".to_string(),
        }).await.unwrap();

        // User1 starts editing
        let start_result = handler.handle(CollaborationCommand::StartEditing {
            session_id,
            user_id: user1.clone(),
            element_type: crate::events::ElementType::Node,
            element_id: "node1".to_string(),
        }).await;

        assert!(start_result.is_ok());

        // User2 tries to edit same element
        let start_result2 = handler.handle(CollaborationCommand::StartEditing {
            session_id,
            user_id: user2,
            element_type: crate::events::ElementType::Node,
            element_id: "node1".to_string(),
        }).await;

        assert!(matches!(start_result2, Err(CollaborationCommandError::ElementLocked { .. })));
    }
}