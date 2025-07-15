//! Collaboration domain projections

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;

use cim_domain::GraphId;
use crate::aggregate::UserId;
use crate::events::{CollaborationEvent, UserPresence};

/// A view of active collaboration sessions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionView {
    pub session_id: Uuid,
    pub graph_id: GraphId,
    pub active_users: Vec<UserPresence>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

/// Projection for active collaboration sessions
#[derive(Clone)]
pub struct ActiveSessionsProjection {
    sessions: Arc<DashMap<Uuid, SessionView>>,
    user_sessions: Arc<DashMap<UserId, Uuid>>,
}

impl ActiveSessionsProjection {
    /// Create a new projection
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            user_sessions: Arc::new(DashMap::new()),
        }
    }

    /// Apply an event to update the projection
    pub fn apply_event(&self, event: &CollaborationEvent) {
        match event {
            CollaborationEvent::UserJoinedSession {
                session_id,
                graph_id,
                user_id,
                user_name,
                color,
                timestamp,
            } => {
                let presence = UserPresence {
                    user_id: user_id.clone(),
                    user_name: user_name.clone(),
                    color: color.clone(),
                    cursor_position: None,
                    selected_nodes: Vec::new(),
                    selected_edges: Vec::new(),
                    last_activity: *timestamp,
                };

                self.sessions
                    .entry(*session_id)
                    .and_modify(|view| {
                        view.active_users.push(presence.clone());
                        view.last_activity = *timestamp;
                    })
                    .or_insert_with(|| SessionView {
                        session_id: *session_id,
                        graph_id: graph_id.clone(),
                        active_users: vec![presence],
                        created_at: *timestamp,
                        last_activity: *timestamp,
                    });

                self.user_sessions.insert(user_id.clone(), *session_id);
            }

            CollaborationEvent::UserLeftSession {
                session_id,
                user_id,
                timestamp,
                ..
            } => {
                if let Some(mut view) = self.sessions.get_mut(session_id) {
                    view.active_users.retain(|p| p.user_id != *user_id);
                    view.last_activity = *timestamp;
                    
                    if view.active_users.is_empty() {
                        drop(view);
                        self.sessions.remove(session_id);
                    }
                }
                self.user_sessions.remove(user_id);
            }

            CollaborationEvent::CursorMoved {
                session_id,
                user_id,
                position,
                timestamp,
                ..
            } => {
                if let Some(mut view) = self.sessions.get_mut(session_id) {
                    if let Some(user) = view.active_users.iter_mut().find(|u| u.user_id == *user_id) {
                        user.cursor_position = Some(position.clone());
                        user.last_activity = *timestamp;
                    }
                    view.last_activity = *timestamp;
                }
            }

            CollaborationEvent::SelectionChanged {
                session_id,
                user_id,
                selection,
                timestamp,
                ..
            } => {
                if let Some(mut view) = self.sessions.get_mut(session_id) {
                    if let Some(user) = view.active_users.iter_mut().find(|u| u.user_id == *user_id) {
                        user.selected_nodes = selection.nodes.clone();
                        user.selected_edges = selection.edges.clone();
                        user.last_activity = *timestamp;
                    }
                    view.last_activity = *timestamp;
                }
            }

            _ => {}
        }
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &Uuid) -> Option<SessionView> {
        self.sessions.get(session_id).map(|r| r.clone())
    }

    /// Get all active sessions
    pub fn get_all_sessions(&self) -> Vec<SessionView> {
        self.sessions.iter().map(|r| r.value().clone()).collect()
    }

    /// Get sessions for a specific graph
    pub fn get_graph_sessions(&self, graph_id: &GraphId) -> Vec<SessionView> {
        self.sessions
            .iter()
            .filter(|r| r.graph_id == *graph_id)
            .map(|r| r.value().clone())
            .collect()
    }

    /// Get the session a user is in
    pub fn get_user_session(&self, user_id: &UserId) -> Option<SessionView> {
        self.user_sessions
            .get(user_id)
            .and_then(|session_id| self.get_session(&session_id))
    }

    /// Get the number of active sessions
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get the total number of active users across all sessions
    pub fn total_user_count(&self) -> usize {
        self.sessions
            .iter()
            .map(|r| r.active_users.len())
            .sum()
    }
}

impl Default for ActiveSessionsProjection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_projection() {
        let projection = ActiveSessionsProjection::new();
        let session_id = Uuid::new_v4();
        let graph_id = GraphId::new();
        let user_id = UserId::new();

        // Apply join event
        let join_event = CollaborationEvent::UserJoinedSession {
            session_id,
            graph_id: graph_id.clone(),
            user_id: user_id.clone(),
            user_name: "Alice".to_string(),
            color: "#FF6B6B".to_string(),
            timestamp: Utc::now(),
        };
        projection.apply_event(&join_event);

        // Check projection state
        assert_eq!(projection.session_count(), 1);
        assert_eq!(projection.total_user_count(), 1);
        
        let session = projection.get_session(&session_id).unwrap();
        assert_eq!(session.active_users.len(), 1);
        assert_eq!(session.active_users[0].user_name, "Alice");

        // Apply leave event
        let leave_event = CollaborationEvent::UserLeftSession {
            session_id,
            graph_id: graph_id.clone(),
            user_id,
            timestamp: Utc::now(),
        };
        projection.apply_event(&leave_event);

        // Session should be removed when empty
        assert_eq!(projection.session_count(), 0);
        assert_eq!(projection.total_user_count(), 0);
    }
}