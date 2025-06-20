//! Test that UI events are properly published to NATS JetStream
//!
//! This example:
//! 1. Starts the app with NATS bridge
//! 2. Sends a test question
//! 3. Verifies the event appears in NATS with proper metadata

use bevy::prelude::*;
use ia::{
    plugins::{AgentUiPlugin, NatsEventBridgePlugin},
    simple_agent::{AgentQuestionEvent, AgentResponseEvent, SimpleAgentPlugin},
};
use std::time::Duration;

fn main() {
    println!("=== Testing UI → NATS Event Flow ===\n");

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(SimpleAgentPlugin)
        .add_plugins(NatsEventBridgePlugin)
        .add_systems(Startup, send_test_question)
        .add_systems(Update, (log_responses, exit_after_delay))
        .run();
}

fn send_test_question(mut events: EventWriter<AgentQuestionEvent>) {
    println!("📤 Sending test question to UI event system...");

    events.write(AgentQuestionEvent {
        question: "What is event sourcing in CIM?".to_string(),
    });

    println!("✅ Question event sent!");
    println!("\n🔍 Check NATS JetStream for:");
    println!("   - Stream: CIM-UI-EVENTS");
    println!("   - Subject: cim.ui.agent.question");
    println!("   - Event should have:");
    println!("     • event_id (UUID)");
    println!("     • correlation_id (UUID)");
    println!("     • causation_id (None for questions)");
    println!("     • timestamp");
    println!("     • payload with question text");
}

fn log_responses(mut events: EventReader<AgentResponseEvent>) {
    for event in events.read() {
        println!("\n📨 Received response: {}", event.response);
        println!("\n🔍 Check NATS JetStream for:");
        println!("   - Subject: cim.ui.agent.response");
        println!("   - Event should have:");
        println!("     • causation_id pointing to question's correlation_id");
    }
}

fn exit_after_delay(time: Res<Time>, mut exit: EventWriter<AppExit>) {
    // Exit after 5 seconds to give time for processing
    if time.elapsed_secs() > 5.0 {
        println!("\n✨ Test complete! Check NATS streams for events.");
        exit.write(AppExit::Success);
    }
}
