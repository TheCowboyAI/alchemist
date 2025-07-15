//! Demonstration of real-time collaboration features

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize
    let command_handler = Arc::new(CollaborationCommandHandler::new());
    let projection = ActiveSessionsProjection::new();
    let query_handler = CollaborationQueryHandler::new(projection.clone());

    println!("=== Real-Time Collaboration Demo ===\n");

    // Create a graph to collaborate on
    let graph_id = GraphId::new();
    println!("Created graph for collaboration: {}", graph_id);

    // User 1 joins
    let user1_id = UserId::new();
    let user1_name = "Alice".to_string();
    println!("\n1. {} joining collaboration session...", user1_name);
    
    let events = command_handler.handle(CollaborationCommand::JoinSession {
        graph_id: graph_id.clone(),
        user_id: user1_id.clone(),
        user_name: user1_name.clone(),
    }).await?;

    let session_id = if let Some(CollaborationEvent::UserJoinedSession { session_id, color, .. }) = events.first() {
        println!("   ✓ {} joined with color {}", user1_name, color);
        projection.apply_event(&events[0]);
        *session_id
    } else {
        panic!("Failed to join session");
    };

    // User 2 joins
    let user2_id = UserId::new();
    let user2_name = "Bob".to_string();
    println!("\n2. {} joining collaboration session...", user2_name);
    
    let events = command_handler.handle(CollaborationCommand::JoinSession {
        graph_id: graph_id.clone(),
        user_id: user2_id.clone(),
        user_name: user2_name.clone(),
    }).await?;

    if let Some(CollaborationEvent::UserJoinedSession { color, .. }) = events.first() {
        println!("   ✓ {} joined with color {}", user2_name, color);
        projection.apply_event(&events[0]);
    }

    // Show current session state
    println!("\n3. Current session state:");
    if let CollaborationQueryResult::Session(session) = 
        query_handler.handle(CollaborationQuery::GetSession { session_id }) {
        println!("   Session ID: {}", session.session_id);
        println!("   Graph ID: {}", session.graph_id);
        println!("   Active users: {}", session.active_users.len());
        for user in &session.active_users {
            println!("     - {} ({})", user.user_name, user.color);
        }
    }

    // User 1 moves cursor
    println!("\n4. {} moving cursor...", user1_name);
    let events = command_handler.handle(CollaborationCommand::UpdateCursor {
        session_id,
        user_id: user1_id.clone(),
        position: CursorPosition { x: 100.0, y: 200.0, z: 0.0 },
    }).await?;
    
    if events.len() > 0 {
        println!("   ✓ Cursor moved to (100, 200, 0)");
        projection.apply_event(&events[0]);
    }

    // User 2 selects some nodes
    println!("\n5. {} selecting nodes...", user2_name);
    let node1 = NodeId::new();
    let node2 = NodeId::new();
    
    let events = command_handler.handle(CollaborationCommand::UpdateSelection {
        session_id,
        user_id: user2_id.clone(),
        selection: SelectionState {
            nodes: vec![node1.clone(), node2.clone()],
            edges: vec![],
        },
    }).await?;
    
    if events.len() > 0 {
        println!("   ✓ Selected 2 nodes");
        projection.apply_event(&events[0]);
    }

    // User 1 tries to edit a node
    println!("\n6. {} requesting to edit node...", user1_name);
    let events = command_handler.handle(CollaborationCommand::StartEditing {
        session_id,
        user_id: user1_id.clone(),
        element_type: ElementType::Node,
        element_id: node1.to_string(),
    }).await?;
    
    if events.len() > 0 {
        println!("   ✓ Started editing node {}", node1);
        projection.apply_event(&events[0]);
    }

    // User 2 tries to edit the same node (should fail)
    println!("\n7. {} trying to edit the same node...", user2_name);
    match command_handler.handle(CollaborationCommand::StartEditing {
        session_id,
        user_id: user2_id.clone(),
        element_type: ElementType::Node,
        element_id: node1.to_string(),
    }).await {
        Err(CollaborationCommandError::ElementLocked { element_id, user_id }) => {
            println!("   ✗ Cannot edit - node is locked by another user");
        }
        _ => println!("   Unexpected result"),
    }

    // User 1 finishes editing
    println!("\n8. {} finishing edit...", user1_name);
    let events = command_handler.handle(CollaborationCommand::FinishEditing {
        session_id,
        user_id: user1_id.clone(),
        element_type: ElementType::Node,
        element_id: node1.to_string(),
    }).await?;
    
    if events.len() > 0 {
        println!("   ✓ Finished editing node");
        projection.apply_event(&events[0]);
    }

    // Now User 2 can edit
    println!("\n9. {} now editing the node...", user2_name);
    let events = command_handler.handle(CollaborationCommand::StartEditing {
        session_id,
        user_id: user2_id.clone(),
        element_type: ElementType::Node,
        element_id: node1.to_string(),
    }).await?;
    
    if events.len() > 0 {
        println!("   ✓ Started editing node {}", node1);
        projection.apply_event(&events[0]);
    }

    // Show collaboration statistics
    println!("\n10. Collaboration statistics:");
    if let CollaborationQueryResult::Statistics(stats) = 
        query_handler.handle(CollaborationQuery::GetStatistics) {
        println!("    Active sessions: {}", stats.active_sessions);
        println!("    Total users: {}", stats.total_users);
        println!("    Graphs with collaboration: {}", stats.graphs_with_collaboration);
        println!("    Average users per session: {:.1}", stats.average_users_per_session);
    }

    // Simulate some activity
    println!("\n11. Simulating real-time activity...");
    for i in 0..5 {
        let x = (i as f64 * 50.0) + 100.0;
        let y = (i as f64 * 30.0) + 200.0;
        
        let events = command_handler.handle(CollaborationCommand::UpdateCursor {
            session_id,
            user_id: user1_id.clone(),
            position: CursorPosition { x, y, z: 0.0 },
        }).await?;
        
        if events.len() > 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout())?;
            projection.apply_event(&events[0]);
        }
        
        sleep(Duration::from_millis(200)).await;
    }
    println!(" Done!");

    // Users leave
    println!("\n12. Users leaving session...");
    
    let events = command_handler.handle(CollaborationCommand::LeaveSession {
        session_id,
        user_id: user1_id.clone(),
    }).await?;
    
    if events.len() > 0 {
        println!("   ✓ {} left", user1_name);
        projection.apply_event(&events[0]);
    }

    let events = command_handler.handle(CollaborationCommand::LeaveSession {
        session_id,
        user_id: user2_id.clone(),
    }).await?;
    
    if events.len() > 0 {
        println!("   ✓ {} left", user2_name);
        projection.apply_event(&events[0]);
    }

    // Show final state
    println!("\n13. Final state:");
    if let CollaborationQueryResult::Statistics(stats) = 
        query_handler.handle(CollaborationQuery::GetStatistics) {
        println!("    Active sessions: {}", stats.active_sessions);
        println!("    Total users: {}", stats.total_users);
    }

    println!("\n=== Demo Complete ===");
    
    Ok(())
}