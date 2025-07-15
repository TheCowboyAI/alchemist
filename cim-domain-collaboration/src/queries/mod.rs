//! Collaboration domain queries

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use cim_domain::GraphId;
use crate::aggregate::UserId;

use crate::projections::{ActiveSessionsProjection, SessionView};

/// Queries for the collaboration domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborationQuery {
    /// Get a specific session by ID
    GetSession { session_id: Uuid },
    
    /// Get all active sessions
    GetAllSessions,
    
    /// Get sessions for a specific graph
    GetGraphSessions { graph_id: GraphId },
    
    /// Get the session a user is currently in
    GetUserSession { user_id: UserId },
    
    /// Get collaboration statistics
    GetStatistics,
}

/// Statistics about collaboration activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationStatistics {
    pub active_sessions: usize,
    pub total_users: usize,
    pub graphs_with_collaboration: usize,
    pub average_users_per_session: f64,
}

/// Query handler for collaboration queries
pub struct CollaborationQueryHandler {
    projection: ActiveSessionsProjection,
}

impl CollaborationQueryHandler {
    /// Create a new query handler
    pub fn new(projection: ActiveSessionsProjection) -> Self {
        Self { projection }
    }

    /// Handle a query
    pub fn handle(&self, query: CollaborationQuery) -> CollaborationQueryResult {
        match query {
            CollaborationQuery::GetSession { session_id } => {
                self.projection
                    .get_session(&session_id)
                    .map(CollaborationQueryResult::Session)
                    .unwrap_or(CollaborationQueryResult::SessionNotFound)
            }

            CollaborationQuery::GetAllSessions => {
                let sessions = self.projection.get_all_sessions();
                CollaborationQueryResult::Sessions(sessions)
            }

            CollaborationQuery::GetGraphSessions { graph_id } => {
                let sessions = self.projection.get_graph_sessions(&graph_id);
                CollaborationQueryResult::Sessions(sessions)
            }

            CollaborationQuery::GetUserSession { user_id } => {
                self.projection
                    .get_user_session(&user_id)
                    .map(CollaborationQueryResult::Session)
                    .unwrap_or(CollaborationQueryResult::UserNotInSession)
            }

            CollaborationQuery::GetStatistics => {
                let sessions = self.projection.get_all_sessions();
                let total_users = self.projection.total_user_count();
                let active_sessions = sessions.len();
                
                let graphs_with_collaboration = sessions
                    .iter()
                    .map(|s| s.graph_id.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .len();

                let average_users_per_session = if active_sessions > 0 {
                    total_users as f64 / active_sessions as f64
                } else {
                    0.0
                };

                CollaborationQueryResult::Statistics(CollaborationStatistics {
                    active_sessions,
                    total_users,
                    graphs_with_collaboration,
                    average_users_per_session,
                })
            }
        }
    }
}

/// Results from collaboration queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborationQueryResult {
    /// Single session
    Session(SessionView),
    
    /// Multiple sessions
    Sessions(Vec<SessionView>),
    
    /// Statistics
    Statistics(CollaborationStatistics),
    
    /// Session not found
    SessionNotFound,
    
    /// User not in any session
    UserNotInSession,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::CollaborationEvent;
    use chrono::Utc;

    #[test]
    fn test_collaboration_queries() {
        let projection = ActiveSessionsProjection::new();
        let handler = CollaborationQueryHandler::new(projection.clone());

        // Initially no sessions
        match handler.handle(CollaborationQuery::GetAllSessions) {
            CollaborationQueryResult::Sessions(sessions) => assert_eq!(sessions.len(), 0),
            _ => panic!("Expected Sessions result"),
        }

        // Add a session
        let session_id = Uuid::new_v4();
        let graph_id = GraphId::new();
        let user_id = UserId::new();

        let event = CollaborationEvent::UserJoinedSession {
            session_id,
            graph_id: graph_id.clone(),
            user_id: user_id.clone(),
            user_name: "Alice".to_string(),
            color: "#FF6B6B".to_string(),
            timestamp: Utc::now(),
        };
        projection.apply_event(&event);

        // Query for the session
        match handler.handle(CollaborationQuery::GetSession { session_id }) {
            CollaborationQueryResult::Session(session) => {
                assert_eq!(session.session_id, session_id);
                assert_eq!(session.active_users.len(), 1);
            }
            _ => panic!("Expected Session result"),
        }

        // Query for user's session
        match handler.handle(CollaborationQuery::GetUserSession { user_id }) {
            CollaborationQueryResult::Session(session) => {
                assert_eq!(session.session_id, session_id);
            }
            _ => panic!("Expected Session result"),
        }

        // Query statistics
        match handler.handle(CollaborationQuery::GetStatistics) {
            CollaborationQueryResult::Statistics(stats) => {
                assert_eq!(stats.active_sessions, 1);
                assert_eq!(stats.total_users, 1);
                assert_eq!(stats.graphs_with_collaboration, 1);
                assert_eq!(stats.average_users_per_session, 1.0);
            }
            _ => panic!("Expected Statistics result"),
        }
    }
}