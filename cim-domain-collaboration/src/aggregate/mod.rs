//! Collaboration session aggregate

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;
use cim_domain::GraphId;

/// User ID type for collaboration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

use crate::events::{CollaborationEvent, UserPresence, ElementType};

/// A collaborative editing session for a graph
#[derive(Debug, Clone)]
pub struct CollaborationSession {
    /// Unique session ID
    pub session_id: Uuid,
    /// Graph being edited
    pub graph_id: GraphId,
    /// Active users in the session
    pub active_users: HashMap<UserId, UserPresence>,
    /// Elements currently being edited by users
    pub editing_locks: HashMap<String, UserId>,
    /// Session creation time
    pub created_at: DateTime<Utc>,
    /// Last activity time
    pub last_activity: DateTime<Utc>,
}

impl CollaborationSession {
    /// Create a new collaboration session
    pub fn new(graph_id: GraphId) -> Self {
        let now = Utc::now();
        Self {
            session_id: Uuid::new_v4(),
            graph_id,
            active_users: HashMap::new(),
            editing_locks: HashMap::new(),
            created_at: now,
            last_activity: now,
        }
    }

    /// Apply an event to the session
    pub fn apply_event(&mut self, event: &CollaborationEvent) {
        self.last_activity = event.timestamp();
        
        match event {
            CollaborationEvent::UserJoinedSession { user_id, user_name, color, timestamp, .. } => {
                let presence = UserPresence {
                    user_id: user_id.clone(),
                    user_name: user_name.clone(),
                    color: color.clone(),
                    cursor_position: None,
                    selected_nodes: Vec::new(),
                    selected_edges: Vec::new(),
                    last_activity: *timestamp,
                };
                self.active_users.insert(user_id.clone(), presence);
            }

            CollaborationEvent::UserLeftSession { user_id, .. } => {
                self.active_users.remove(user_id);
                // Release any editing locks held by this user
                self.editing_locks.retain(|_, locked_by| locked_by != user_id);
            }

            CollaborationEvent::CursorMoved { user_id, position, timestamp, .. } => {
                if let Some(presence) = self.active_users.get_mut(user_id) {
                    presence.cursor_position = Some(position.clone());
                    presence.last_activity = *timestamp;
                }
            }

            CollaborationEvent::SelectionChanged { user_id, selection, timestamp, .. } => {
                if let Some(presence) = self.active_users.get_mut(user_id) {
                    presence.selected_nodes = selection.nodes.clone();
                    presence.selected_edges = selection.edges.clone();
                    presence.last_activity = *timestamp;
                }
            }

            CollaborationEvent::EditingStarted { user_id, element_id, .. } => {
                self.editing_locks.insert(element_id.clone(), user_id.clone());
            }

            CollaborationEvent::EditingFinished { element_id, .. } => {
                self.editing_locks.remove(element_id);
            }

            _ => {}
        }
    }

    /// Check if a user is in the session
    pub fn has_user(&self, user_id: &UserId) -> bool {
        self.active_users.contains_key(user_id)
    }

    /// Get the number of active users
    pub fn user_count(&self) -> usize {
        self.active_users.len()
    }

    /// Check if an element is locked for editing
    pub fn is_locked(&self, element_id: &str) -> Option<&UserId> {
        self.editing_locks.get(element_id)
    }

    /// Get all active user presences
    pub fn get_presences(&self) -> Vec<UserPresence> {
        self.active_users.values().cloned().collect()
    }

    /// Generate a unique color for a new user
    pub fn generate_user_color(&self) -> String {
        const COLORS: &[&str] = &[
            "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FECA57",
            "#FF9FF3", "#54A0FF", "#48DBFB", "#0ABDE3", "#00D2D3",
        ];
        let index = self.active_users.len() % COLORS.len();
        COLORS[index].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cim_domain::NodeId;

    #[test]
    fn test_session_creation() {
        let graph_id = GraphId::new();
        let session = CollaborationSession::new(graph_id.clone());
        
        assert_eq!(session.graph_id, graph_id);
        assert_eq!(session.user_count(), 0);
        assert!(session.editing_locks.is_empty());
    }

    #[test]
    fn test_user_join_leave() {
        let graph_id = GraphId::new();
        let mut session = CollaborationSession::new(graph_id.clone());
        let user_id = UserId::new();
        
        // User joins
        let join_event = CollaborationEvent::UserJoinedSession {
            session_id: session.session_id,
            graph_id: graph_id.clone(),
            user_id: user_id.clone(),
            user_name: "Alice".to_string(),
            color: "#FF6B6B".to_string(),
            timestamp: Utc::now(),
        };
        session.apply_event(&join_event);
        
        assert!(session.has_user(&user_id));
        assert_eq!(session.user_count(), 1);
        
        // User leaves
        let leave_event = CollaborationEvent::UserLeftSession {
            session_id: session.session_id,
            graph_id,
            user_id: user_id.clone(),
            timestamp: Utc::now(),
        };
        session.apply_event(&leave_event);
        
        assert!(!session.has_user(&user_id));
        assert_eq!(session.user_count(), 0);
    }

    #[test]
    fn test_editing_locks() {
        let graph_id = GraphId::new();
        let mut session = CollaborationSession::new(graph_id.clone());
        let user_id = UserId::new();
        let node_id = NodeId::new();
        
        // Start editing
        let start_event = CollaborationEvent::EditingStarted {
            session_id: session.session_id,
            graph_id: graph_id.clone(),
            user_id: user_id.clone(),
            element_type: ElementType::Node,
            element_id: node_id.to_string(),
            timestamp: Utc::now(),
        };
        session.apply_event(&start_event);
        
        assert_eq!(session.is_locked(&node_id.to_string()), Some(&user_id));
        
        // Finish editing
        let finish_event = CollaborationEvent::EditingFinished {
            session_id: session.session_id,
            graph_id,
            user_id,
            element_type: ElementType::Node,
            element_id: node_id.to_string(),
            timestamp: Utc::now(),
        };
        session.apply_event(&finish_event);
        
        assert_eq!(session.is_locked(&node_id.to_string()), None);
    }
}