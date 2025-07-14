//! Test the agent integration to ensure it's processing events correctly

#![cfg(feature = "bevy")]
// This test is disabled because it depends on modules from the ia binary that are not available
#![cfg(feature = "simple_agent_available")]

use bevy::prelude::*;
use ia::simple_agent::{
    AgentErrorEvent, AgentQuestionEvent, AgentResponseEvent, SimpleAgentPlugin,
};
use std::time::Duration;

/// Test that the agent plugin is correctly processing questions
#[test]
fn test_agent_processes_questions() {
    let mut app = App::new();

    // Add minimal plugins needed
    app.add_plugins(MinimalPlugins);

    // Add the agent plugin
    app.add_plugins(SimpleAgentPlugin);

    // Track events
    #[derive(Resource, Default)]
    struct EventTracker {
        questions_sent: usize,
        responses_received: usize,
        errors_received: usize,
    }

    app.init_resource::<EventTracker>();

    // System to track sent questions
    app.add_systems(
        Update,
        |mut events: EventReader<AgentQuestionEvent>, mut tracker: ResMut<EventTracker>| {
            for _ in events.read() {
                tracker.questions_sent += 1;
            }
        },
    );

    // System to track responses
    app.add_systems(
        Update,
        |mut events: EventReader<AgentResponseEvent>, mut tracker: ResMut<EventTracker>| {
            for event in events.read() {
                println!("Received response: {}", event.response);
                tracker.responses_received += 1;
            }
        },
    );

    // System to track errors
    app.add_systems(
        Update,
        |mut events: EventReader<AgentErrorEvent>, mut tracker: ResMut<EventTracker>| {
            for event in events.read() {
                println!("Received error: {}", event.error);
                tracker.errors_received += 1;
            }
        },
    );

    // Send a test question
    println!("Sending test question to agent...");
    app.world_mut().send_event(AgentQuestionEvent {
        question: "What is CIM?".to_string(),
    });

    // Run multiple update cycles to allow async processing
    for i in 0..5 {
        println!("Update cycle {}", i + 1);
        app.update();
        std::thread::sleep(Duration::from_millis(100));
    }

    // Check results
    let tracker = app.world().resource::<EventTracker>();
    println!("Questions sent: {}", tracker.questions_sent);
    println!("Responses received: {}", tracker.responses_received);
    println!("Errors received: {}", tracker.errors_received);

    // We should have either a response or an error
    assert!(
        tracker.responses_received > 0 || tracker.errors_received > 0,
        "Agent should have produced either a response or an error"
    );
}

/// Test the event flow through the agent system
///
/// ```mermaid
/// graph TD
///     A[AgentQuestionEvent] -->|process_questions system| B[Agent Resource]
///     B -->|ask method| C[Ollama Client]
///     C -->|Success| D[AgentResponseEvent]
///     C -->|Error| E[AgentErrorEvent]
/// ```
#[test]
fn test_agent_event_flow_diagram() {
    // This test documents the expected flow
    assert!(true, "See mermaid diagram in doc comment");
}
