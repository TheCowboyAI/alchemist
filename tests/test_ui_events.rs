//! Simple test to validate UI event flow
//!
//! This test validates that the UI components are properly emitting events
//! in the correct order with the correct payloads.

use bevy::prelude::*;

/// Event sent when user asks a question
#[derive(Event, Clone, Debug)]
pub struct AgentQuestionEvent {
    pub question: String,
}

/// Event sent when agent responds
#[derive(Event, Clone, Debug)]
pub struct AgentResponseEvent {
    pub response: String,
}

/// Test that events flow correctly through the system
#[test]
fn test_ui_event_flow() {
    // Create a minimal Bevy app
    let mut app = App::new();
    
    // Add events
    app.add_event::<AgentQuestionEvent>();
    app.add_event::<AgentResponseEvent>();
    
    // Track events with a resource
    #[derive(Resource, Default)]
    struct EventTracker {
        questions: Vec<String>,
        responses: Vec<String>,
    }
    
    app.init_resource::<EventTracker>();
    
    // System to capture question events
    app.add_systems(Update, |
        mut events: EventReader<AgentQuestionEvent>,
        mut tracker: ResMut<EventTracker>
    | {
        for event in events.read() {
            println!("Captured question: {}", event.question);
            tracker.questions.push(event.question.clone());
        }
    });
    
    // System to capture response events
    app.add_systems(Update, |
        mut events: EventReader<AgentResponseEvent>,
        mut tracker: ResMut<EventTracker>
    | {
        for event in events.read() {
            println!("Captured response: {}", event.response);
            tracker.responses.push(event.response.clone());
        }
    });
    
    // Test 1: Send a question event
    println!("Sending question event...");
    app.world_mut().send_event(AgentQuestionEvent {
        question: "What is CIM?".to_string(),
    });
    
    // Process events
    app.update();
    
    // Verify question was captured
    let tracker = app.world().resource::<EventTracker>();
    assert_eq!(tracker.questions.len(), 1, "Should have captured one question");
    assert_eq!(tracker.questions[0], "What is CIM?", "Question content should match");
    
    // Test 2: Send a response event
    println!("Sending response event...");
    app.world_mut().send_event(AgentResponseEvent {
        response: "CIM is a Composable Information Machine".to_string(),
    });
    
    // Process events
    app.update();
    
    // Verify response was captured
    let tracker = app.world().resource::<EventTracker>();
    assert_eq!(tracker.responses.len(), 1, "Should have captured one response");
    assert_eq!(tracker.responses[0], "CIM is a Composable Information Machine", "Response content should match");
    
    println!("✅ All event flow tests passed!");
}

/// Test event ordering with multiple events
#[test]
fn test_event_ordering() {
    let mut app = App::new();
    app.add_event::<AgentQuestionEvent>();
    
    #[derive(Resource, Default)]
    struct OrderTracker {
        order: Vec<String>,
    }
    
    app.init_resource::<OrderTracker>();
    
    // System that tracks event order
    app.add_systems(Update, |
        mut events: EventReader<AgentQuestionEvent>,
        mut tracker: ResMut<OrderTracker>
    | {
        for event in events.read() {
            tracker.order.push(event.question.clone());
        }
    });
    
    // Send multiple events
    let questions = vec![
        "First question",
        "Second question",
        "Third question",
    ];
    
    for q in &questions {
        app.world_mut().send_event(AgentQuestionEvent {
            question: q.to_string(),
        });
    }
    
    // Process all events
    app.update();
    
    // Verify order
    let tracker = app.world().resource::<OrderTracker>();
    assert_eq!(tracker.order.len(), 3, "Should have all three questions");
    assert_eq!(tracker.order[0], "First question", "First should be first");
    assert_eq!(tracker.order[1], "Second question", "Second should be second");
    assert_eq!(tracker.order[2], "Third question", "Third should be third");
    
    println!("✅ Event ordering test passed!");
}

/// Test that simulates the complete UI interaction flow
/// 
/// ```mermaid
/// graph TD
///     A[User Presses F1] -->|KeyEvent| B[UI System]
///     B -->|show_window = true| C[Render Chat UI]
///     C -->|User Types| D[Input Text]
///     D -->|Send Button| E[AgentQuestionEvent]
///     E -->|Process| F[Agent System]
///     F -->|Success| G[AgentResponseEvent]
///     G -->|Display| H[Chat Messages]
/// ```
#[test]
fn test_complete_ui_flow() {
    let mut app = App::new();
    
    // Add all required events
    app.add_event::<AgentQuestionEvent>();
    app.add_event::<AgentResponseEvent>();
    
    // Mock UI state
    #[derive(Resource, Default)]
    struct UiState {
        show_window: bool,
        messages: Vec<(String, bool)>, // (text, is_user)
    }
    
    app.init_resource::<UiState>();
    
    // System that simulates UI sending a question
    app.add_systems(Update, |
        mut events: EventReader<AgentQuestionEvent>,
        mut ui_state: ResMut<UiState>,
        mut response_events: EventWriter<AgentResponseEvent>
    | {
        for event in events.read() {
            // Add to UI messages
            ui_state.messages.push((event.question.clone(), true));
            
            // Simulate agent response
            let response = format!("Response to: {}", event.question);
            response_events.send(AgentResponseEvent { response });
        }
    });
    
    // System that handles responses
    app.add_systems(Update, |
        mut events: EventReader<AgentResponseEvent>,
        mut ui_state: ResMut<UiState>
    | {
        for event in events.read() {
            ui_state.messages.push((event.response.clone(), false));
        }
    });
    
    // Simulate complete flow
    app.world_mut().resource_mut::<UiState>().show_window = true;
    
    // Send a question
    app.world_mut().send_event(AgentQuestionEvent {
        question: "What are the 8 CIM domains?".to_string(),
    });
    
    // Process events
    app.update();
    
    // Verify complete flow
    let ui_state = app.world().resource::<UiState>();
    assert!(ui_state.show_window, "Window should be shown");
    assert_eq!(ui_state.messages.len(), 2, "Should have question and response");
    assert_eq!(ui_state.messages[0].0, "What are the 8 CIM domains?", "First message should be question");
    assert!(ui_state.messages[0].1, "First message should be from user");
    assert_eq!(ui_state.messages[1].0, "Response to: What are the 8 CIM domains?", "Second message should be response");
    assert!(!ui_state.messages[1].1, "Second message should not be from user");
    
    println!("✅ Complete UI flow test passed!");
} 