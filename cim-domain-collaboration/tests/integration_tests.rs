//! Integration tests for collaboration domain

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use futures;

use cim_domain::{GraphId, NodeId};
use cim_domain_collaboration::{
    UserId,
    commands::{CollaborationCommand, CollaborationCommandError},
    events::{CollaborationEvent, CursorPosition, SelectionState, ElementType},
    handlers::CollaborationCommandHandler,
    projections::ActiveSessionsProjection,
    queries::{CollaborationQuery, CollaborationQueryHandler, CollaborationQueryResult},
};
// InMemoryEventStore would come from cim_infrastructure

#[tokio::test]
async fn test_collaboration_session_lifecycle() {
    let handler = CollaborationCommandHandler::new();
    let projection = ActiveSessionsProjection::new();

    let graph_id = GraphId::new();
    let user1 = UserId::new();
    let user2 = UserId::new();

    // User 1 joins
    let events = handler.handle(CollaborationCommand::JoinSession {
        graph_id: graph_id.clone(),
        user_id: user1.clone(),
        user_name: "User1".to_string(),
    }).await.unwrap();

    assert_eq!(events.len(), 1);
    let session_id = match &events[0] {
        CollaborationEvent::UserJoinedSession { session_id, .. } => *session_id,
        _ => panic!("Expected UserJoinedSession event"),
    };

    for event in &events {
        projection.apply_event(event);
    }

    // Verify session created
    assert_eq!(projection.session_count(), 1);
    assert_eq!(projection.total_user_count(), 1);

    // User 2 joins same graph (should join existing session)
    let events = handler.handle(CollaborationCommand::JoinSession {
        graph_id: graph_id.clone(),
        user_id: user2.clone(),
        user_name: "User2".to_string(),
    }).await.unwrap();

    for event in &events {
        projection.apply_event(event);
    }

    assert_eq!(projection.session_count(), 1);
    assert_eq!(projection.total_user_count(), 2);

    // Users leave
    let events = handler.handle(CollaborationCommand::LeaveSession {
        session_id,
        user_id: user1,
    }).await.unwrap();

    for event in &events {
        projection.apply_event(event);
    }

    assert_eq!(projection.session_count(), 1);
    assert_eq!(projection.total_user_count(), 1);

    let events = handler.handle(CollaborationCommand::LeaveSession {
        session_id,
        user_id: user2,
    }).await.unwrap();

    for event in &events {
        projection.apply_event(event);
    }

    // Session should be removed when empty
    assert_eq!(projection.session_count(), 0);
    assert_eq!(projection.total_user_count(), 0);
}

#[tokio::test]
async fn test_cursor_and_selection_updates() {
    let handler = CollaborationCommandHandler::new();
    let projection = ActiveSessionsProjection::new();

    let graph_id = GraphId::new();
    let user_id = UserId::new();

    // Join session
    let events = handler.handle(CollaborationCommand::JoinSession {
        graph_id,
        user_id: user_id.clone(),
        user_name: "TestUser".to_string(),
    }).await.unwrap();

    let session_id = match &events[0] {
        CollaborationEvent::UserJoinedSession { session_id, .. } => *session_id,
        _ => panic!("Expected UserJoinedSession event"),
    };

    for event in &events {
        projection.apply_event(event);
    }

    // Update cursor
    let events = handler.handle(CollaborationCommand::UpdateCursor {
        session_id,
        user_id: user_id.clone(),
        position: CursorPosition { x: 100.0, y: 200.0, z: 50.0 },
    }).await.unwrap();

    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], CollaborationEvent::CursorMoved { .. }));

    for event in &events {
        projection.apply_event(event);
    }

    // Update selection
    let node1 = NodeId::new();
    let node2 = NodeId::new();
    
    let events = handler.handle(CollaborationCommand::UpdateSelection {
        session_id,
        user_id: user_id.clone(),
        selection: SelectionState {
            nodes: vec![node1, node2],
            edges: vec![],
        },
    }).await.unwrap();

    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], CollaborationEvent::SelectionChanged { .. }));

    for event in &events {
        projection.apply_event(event);
    }

    // Verify updates in projection
    let session = projection.get_session(&session_id).unwrap();
    let user_presence = &session.active_users[0];
    
    assert!(user_presence.cursor_position.is_some());
    assert_eq!(user_presence.cursor_position.as_ref().unwrap().x, 100.0);
    assert_eq!(user_presence.selected_nodes.len(), 2);
}

#[tokio::test]
async fn test_editing_locks() {
    let handler = CollaborationCommandHandler::new();

    let graph_id = GraphId::new();
    let user1 = UserId::new();
    let user2 = UserId::new();

    // Both users join
    let events = handler.handle(CollaborationCommand::JoinSession {
        graph_id: graph_id.clone(),
        user_id: user1.clone(),
        user_name: "User1".to_string(),
    }).await.unwrap();

    let session_id = match &events[0] {
        CollaborationEvent::UserJoinedSession { session_id, .. } => *session_id,
        _ => panic!("Expected UserJoinedSession event"),
    };

    handler.handle(CollaborationCommand::JoinSession {
        graph_id,
        user_id: user2.clone(),
        user_name: "User2".to_string(),
    }).await.unwrap();

    let node_id = NodeId::new();

    // User1 starts editing
    let result = handler.handle(CollaborationCommand::StartEditing {
        session_id,
        user_id: user1.clone(),
        element_type: ElementType::Node,
        element_id: node_id.to_string(),
    }).await;

    assert!(result.is_ok());

    // User2 tries to edit same node (should fail)
    let result = handler.handle(CollaborationCommand::StartEditing {
        session_id,
        user_id: user2.clone(),
        element_type: ElementType::Node,
        element_id: node_id.to_string(),
    }).await;

    assert!(matches!(
        result,
        Err(CollaborationCommandError::ElementLocked { .. })
    ));

    // User1 finishes editing
    handler.handle(CollaborationCommand::FinishEditing {
        session_id,
        user_id: user1,
        element_type: ElementType::Node,
        element_id: node_id.to_string(),
    }).await.unwrap();

    // Now User2 can edit
    let result = handler.handle(CollaborationCommand::StartEditing {
        session_id,
        user_id: user2,
        element_type: ElementType::Node,
        element_id: node_id.to_string(),
    }).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_session_cleanup() {
    let handler = CollaborationCommandHandler::new();
    let projection = ActiveSessionsProjection::new();

    // Create multiple sessions
    for i in 0..3 {
        let graph_id = GraphId::new();
        let user_id = UserId::new();

        let events = handler.handle(CollaborationCommand::JoinSession {
            graph_id,
            user_id: user_id.clone(),
            user_name: format!("User{}", i),
        }).await.unwrap();

        let session_id = match &events[0] {
            CollaborationEvent::UserJoinedSession { session_id, .. } => *session_id,
            _ => panic!("Expected UserJoinedSession event"),
        };

        for event in &events {
            projection.apply_event(event);
        }

        // Leave session to make it inactive
        let events = handler.handle(CollaborationCommand::LeaveSession {
            session_id,
            user_id,
        }).await.unwrap();

        for event in &events {
            projection.apply_event(event);
        }
    }

    // All sessions should be removed (empty sessions are auto-removed)
    assert_eq!(projection.session_count(), 0);

    // Cleanup should handle this gracefully
    let result = handler.handle(CollaborationCommand::CleanupInactiveSessions {
        inactive_threshold_minutes: 10,
    }).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_collaboration_queries() {
    let handler = CollaborationCommandHandler::new();
    let projection = ActiveSessionsProjection::new();
    let query_handler = CollaborationQueryHandler::new(projection.clone());

    let graph_id = GraphId::new();
    let user_id = UserId::new();

    // Join session
    let events = handler.handle(CollaborationCommand::JoinSession {
        graph_id: graph_id.clone(),
        user_id: user_id.clone(),
        user_name: "QueryTest".to_string(),
    }).await.unwrap();

    let session_id = match &events[0] {
        CollaborationEvent::UserJoinedSession { session_id, .. } => *session_id,
        _ => panic!("Expected UserJoinedSession event"),
    };

    for event in &events {
        projection.apply_event(event);
    }

    // Test GetSession query
    match query_handler.handle(CollaborationQuery::GetSession { session_id }) {
        CollaborationQueryResult::Session(session) => {
            assert_eq!(session.session_id, session_id);
            assert_eq!(session.active_users.len(), 1);
        }
        _ => panic!("Expected Session result"),
    }

    // Test GetUserSession query
    match query_handler.handle(CollaborationQuery::GetUserSession { user_id }) {
        CollaborationQueryResult::Session(session) => {
            assert_eq!(session.session_id, session_id);
        }
        _ => panic!("Expected Session result"),
    }

    // Test GetGraphSessions query
    match query_handler.handle(CollaborationQuery::GetGraphSessions { graph_id }) {
        CollaborationQueryResult::Sessions(sessions) => {
            assert_eq!(sessions.len(), 1);
            assert_eq!(sessions[0].session_id, session_id);
        }
        _ => panic!("Expected Sessions result"),
    }

    // Test GetStatistics query
    match query_handler.handle(CollaborationQuery::GetStatistics) {
        CollaborationQueryResult::Statistics(stats) => {
            assert_eq!(stats.active_sessions, 1);
            assert_eq!(stats.total_users, 1);
            assert_eq!(stats.graphs_with_collaboration, 1);
            assert_eq!(stats.average_users_per_session, 1.0);
        }
        _ => panic!("Expected Statistics result"),
    }
}

#[tokio::test]
async fn test_concurrent_operations() {
    let handler = Arc::new(CollaborationCommandHandler::new());
    let projection = Arc::new(ActiveSessionsProjection::new());

    let graph_id = GraphId::new();

    // Spawn multiple tasks to join sessions concurrently
    let mut handles = vec![];
    
    for i in 0..10 {
        let handler = handler.clone();
        let projection = projection.clone();
        let graph_id = graph_id.clone();
        
        let handle = tokio::spawn(async move {
            let user_id = UserId::new();
            
            let events = handler.handle(CollaborationCommand::JoinSession {
                graph_id,
                user_id: user_id.clone(),
                user_name: format!("User{}", i),
            }).await.unwrap();

            for event in &events {
                projection.apply_event(event);
            }

            (user_id, events)
        });
        
        handles.push(handle);
    }

    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles).await;
    
    // All should have joined the same session
    let session_ids: Vec<_> = results.iter()
        .filter_map(|r| r.as_ref().ok())
        .filter_map(|(_, events)| match &events[0] {
            CollaborationEvent::UserJoinedSession { session_id, .. } => Some(session_id),
            _ => None,
        })
        .collect();

    // All users should be in the same session
    let first_session = session_ids[0];
    assert!(session_ids.iter().all(|&id| id == first_session));
    
    // Verify projection state
    assert_eq!(projection.session_count(), 1);
    assert_eq!(projection.total_user_count(), 10);
}