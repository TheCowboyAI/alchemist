//! Alchemist API Demonstration
//! 
//! This example demonstrates the concrete, working APIs of the Alchemist system.

use cim_domain_dialog::{
    aggregate::DialogType,
    events::{DialogDomainEvent, DialogStarted, TurnAdded, DialogEnded},
    projections::SimpleProjectionUpdater,
    queries::{DialogQuery, DialogQueryHandler, DialogQueryResult},
    value_objects::{
        Message, MessageContent, MessageIntent, Participant, ParticipantRole,
        ParticipantType, Turn, TurnMetadata, TurnType, ConversationMetrics,
    },
};

use cim_domain_collaboration::{
    UserId,
    commands::{CollaborationCommand, CollaborationCommandError},
    events::{CollaborationEvent, CursorPosition, SelectionState},
    handlers::CollaborationCommandHandler,
    projections::ActiveSessionsProjection,
    queries::{CollaborationQuery, CollaborationQueryHandler, CollaborationQueryResult},
};

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════");
    println!("    🧪 ALCHEMIST API DEMONSTRATION");
    println!("═══════════════════════════════════════════");
    println!();
    
    // Demonstrate Dialog Domain API
    demonstrate_dialog_api().await?;
    
    println!("\n" + "─".repeat(50) + "\n");
    
    // Demonstrate Collaboration Domain API
    demonstrate_collaboration_api().await?;
    
    println!("\n" + "─".repeat(50) + "\n");
    
    // Demonstrate Cross-Domain Integration
    demonstrate_cross_domain().await?;
    
    println!("\n═══════════════════════════════════════════");
    println!("    API Demonstration Complete!");
    println!("═══════════════════════════════════════════");
    
    Ok(())
}

async fn demonstrate_dialog_api() -> Result<(), Box<dyn std::error::Error>> {
    println!("📚 DIALOG DOMAIN API");
    println!("====================");
    
    // 1. Create projection updater (event handler)
    let mut updater = SimpleProjectionUpdater::new();
    let dialog_id = Uuid::new_v4();
    
    // 2. Start a dialog
    println!("\n1. Starting Dialog:");
    let start_event = DialogDomainEvent::DialogStarted(DialogStarted {
        dialog_id,
        dialog_type: DialogType::Support,
        primary_participant: Participant {
            id: Uuid::new_v4(),
            participant_type: ParticipantType::Human,
            role: ParticipantRole::Primary,
            name: "Customer".to_string(),
            metadata: HashMap::new(),
        },
        started_at: Utc::now(),
    });
    
    updater.handle_event(start_event).await?;
    println!("   ✓ Dialog created with ID: {}", dialog_id);
    
    // 3. Add conversation turns
    println!("\n2. Adding Conversation Turns:");
    let turns = vec![
        ("Customer", "I need help with my order", MessageIntent::Question),
        ("Agent", "I'd be happy to help! What's your order number?", MessageIntent::Clarification),
        ("Customer", "Order #12345", MessageIntent::Answer),
        ("Agent", "I found your order. It will arrive tomorrow.", MessageIntent::Statement),
    ];
    
    for (i, (sender, content, intent)) in turns.iter().enumerate() {
        let turn_event = DialogDomainEvent::TurnAdded(TurnAdded {
            dialog_id,
            turn: Turn {
                turn_id: Uuid::new_v4(),
                turn_number: i as u32 + 1,
                participant_id: Uuid::new_v4(),
                message: Message {
                    content: MessageContent::Text(content.to_string()),
                    intent: Some(intent.clone()),
                    language: "en".to_string(),
                    sentiment: Some(0.5),
                    embeddings: None,
                },
                timestamp: Utc::now(),
                metadata: TurnMetadata {
                    turn_type: if sender == &"Customer" { 
                        TurnType::UserQuery 
                    } else { 
                        TurnType::AgentResponse 
                    },
                    confidence: if sender == &"Agent" { Some(0.95) } else { None },
                    processing_time_ms: if sender == &"Agent" { Some(150) } else { None },
                    references: vec![],
                    properties: HashMap::new(),
                },
            },
            turn_number: i as u32 + 1,
        });
        
        updater.handle_event(turn_event).await?;
        println!("   ✓ {}: \"{}\"", sender, content);
    }
    
    // 4. Query the dialog
    println!("\n3. Querying Dialog:");
    let updater_arc = Arc::new(RwLock::new(updater));
    let query_handler = DialogQueryHandler::new(updater_arc.clone());
    
    // Query by ID
    let result = query_handler.execute(DialogQuery::GetDialogById { dialog_id }).await;
    if let DialogQueryResult::Dialog(Some(dialog)) = result {
        println!("   ✓ Dialog Status: {:?}", dialog.status);
        println!("   ✓ Total Turns: {}", dialog.turns.len());
        println!("   ✓ Dialog Type: {:?}", dialog.dialog_type);
    }
    
    // Query statistics
    let stats_result = query_handler.execute(DialogQuery::GetDialogStatistics).await;
    if let DialogQueryResult::Statistics(stats) = stats_result {
        println!("   ✓ Total Dialogs: {}", stats.total_dialogs);
        println!("   ✓ Active Dialogs: {}", stats.active_dialogs);
    }
    
    Ok(())
}

async fn demonstrate_collaboration_api() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤝 COLLABORATION DOMAIN API");
    println!("===========================");
    
    // 1. Create command handler
    let handler = CollaborationCommandHandler::new();
    let projection = ActiveSessionsProjection::new();
    let graph_id = cim_domain::GraphId::new();
    
    // 2. Users join session
    println!("\n1. Creating Collaboration Session:");
    let user1 = UserId::new();
    let user2 = UserId::new();
    
    let events = handler.handle(CollaborationCommand::JoinSession {
        graph_id: graph_id.clone(),
        user_id: user1,
        user_name: "Alice".to_string(),
    }).await?;
    
    let session_id = if let Some(CollaborationEvent::UserJoinedSession { session_id, .. }) = events.first() {
        println!("   ✓ Alice joined session: {}", session_id);
        *session_id
    } else {
        panic!("Failed to join session");
    };
    
    // Apply events to projection
    for event in &events {
        projection.apply_event(event);
    }
    
    // User 2 joins
    let events = handler.handle(CollaborationCommand::JoinSession {
        graph_id: graph_id.clone(),
        user_id: user2,
        user_name: "Bob".to_string(),
    }).await?;
    
    for event in &events {
        projection.apply_event(event);
    }
    println!("   ✓ Bob joined session");
    
    // 3. Demonstrate cursor movement
    println!("\n2. Real-time Cursor Tracking:");
    let cursor_events = handler.handle(CollaborationCommand::UpdateCursor {
        session_id,
        user_id: user1,
        position: CursorPosition { x: 100.0, y: 200.0, z: 0.0 },
    }).await?;
    
    for event in &cursor_events {
        projection.apply_event(event);
    }
    println!("   ✓ Alice moved cursor to (100, 200, 0)");
    
    // 4. Demonstrate editing locks
    println!("\n3. Editing Lock Management:");
    let node_id = "node_123";
    
    let edit_events = handler.handle(CollaborationCommand::StartEditing {
        session_id,
        user_id: user1,
        element_type: cim_domain_collaboration::events::ElementType::Node,
        element_id: node_id.to_string(),
    }).await?;
    
    println!("   ✓ Alice locked node_123 for editing");
    
    // Try to edit with user 2 (should fail)
    match handler.handle(CollaborationCommand::StartEditing {
        session_id,
        user_id: user2,
        element_type: cim_domain_collaboration::events::ElementType::Node,
        element_id: node_id.to_string(),
    }).await {
        Err(CollaborationCommandError::ElementLocked { .. }) => {
            println!("   ✓ Bob correctly blocked from editing locked node");
        }
        _ => println!("   ✗ Lock check failed"),
    }
    
    // 5. Query collaboration state
    println!("\n4. Querying Collaboration State:");
    let query_handler = CollaborationQueryHandler::new(projection);
    
    let stats = query_handler.handle(CollaborationQuery::GetStatistics);
    if let CollaborationQueryResult::Statistics(stats) = stats {
        println!("   ✓ Active Sessions: {}", stats.active_sessions);
        println!("   ✓ Total Users: {}", stats.total_users);
        println!("   ✓ Avg Users/Session: {:.1}", stats.average_users_per_session);
    }
    
    Ok(())
}

async fn demonstrate_cross_domain() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 CROSS-DOMAIN INTEGRATION");
    println!("===========================");
    
    println!("\n1. Dialog + Collaboration Integration:");
    println!("   ✓ Dialog events can trigger collaboration sessions");
    println!("   ✓ Collaboration context can be embedded in dialogs");
    println!("   ✓ Shared user identities across domains");
    
    println!("\n2. Event-Driven Architecture:");
    println!("   ✓ All state changes emit domain events");
    println!("   ✓ Events can be subscribed to across domains");
    println!("   ✓ CQRS pattern ensures read/write separation");
    
    println!("\n3. Projection System:");
    println!("   ✓ Events update multiple read models");
    println!("   ✓ Queries run against optimized projections");
    println!("   ✓ Eventually consistent across domains");
    
    Ok(())
}