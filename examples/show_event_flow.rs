//! Example demonstrating event flow in CIM
//!
//! This shows how events flow through the system:
//! 1. User input → AgentQuestionEvent
//! 2. Agent processes → AgentResponseEvent
//! 3. All events are logged and can be tracked

use bevy::prelude::*;
use ia::simple_agent::{
    AgentErrorEvent, AgentQuestionEvent, AgentResponseEvent, SimpleAgentPlugin,
};

fn main() {
    println!("=== CIM Event Flow Demonstration ===\n");
    println!("This example shows how events flow through the CIM system.");
    println!("Events are the foundation of CIM - everything is an event!\n");

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(SimpleAgentPlugin)
        .init_resource::<EventLog>()
        .add_systems(Startup, send_test_questions)
        .add_systems(
            Update,
            (
                log_question_events,
                log_response_events,
                log_error_events,
                print_event_summary.after(log_response_events),
            ),
        )
        .run();
}

#[derive(Resource, Default)]
struct EventLog {
    questions_sent: Vec<(String, std::time::Instant)>,
    responses_received: Vec<(String, std::time::Instant)>,
    errors_received: Vec<(String, std::time::Instant)>,
}

fn send_test_questions(mut questions: EventWriter<AgentQuestionEvent>) {
    println!("📤 Sending test questions to demonstrate event flow:\n");

    let test_questions = vec![
        "What is CIM?",
        "Tell me about domains",
        "How do events work?",
        "What is a graph?",
    ];

    for (i, question) in test_questions.iter().enumerate() {
        println!("  {}. Sending: '{}'", i + 1, question);
        questions.write(AgentQuestionEvent {
            question: question.to_string(),
        });
    }

    println!("\n🔄 Events are now flowing through the system...\n");
}

fn log_question_events(mut events: EventReader<AgentQuestionEvent>, mut log: ResMut<EventLog>) {
    for event in events.read() {
        let timestamp = std::time::Instant::now();
        println!("📨 EVENT: AgentQuestionEvent");
        println!("   Timestamp: {:?}", timestamp);
        println!("   Question: '{}'\n", event.question);

        log.questions_sent.push((event.question.clone(), timestamp));
    }
}

fn log_response_events(mut events: EventReader<AgentResponseEvent>, mut log: ResMut<EventLog>) {
    for event in events.read() {
        let timestamp = std::time::Instant::now();
        println!("✅ EVENT: AgentResponseEvent");
        println!("   Timestamp: {:?}", timestamp);
        println!(
            "   Response: '{}'\n",
            if event.response.len() > 100 {
                format!("{}...", &event.response[..100])
            } else {
                event.response.clone()
            }
        );

        log.responses_received
            .push((event.response.clone(), timestamp));
    }
}

fn log_error_events(mut events: EventReader<AgentErrorEvent>, mut log: ResMut<EventLog>) {
    for event in events.read() {
        let timestamp = std::time::Instant::now();
        println!("❌ EVENT: AgentErrorEvent");
        println!("   Timestamp: {:?}", timestamp);
        println!("   Error: '{}'\n", event.error);

        log.errors_received.push((event.error.clone(), timestamp));
    }
}

fn print_event_summary(log: Res<EventLog>, mut exit: EventWriter<AppExit>) {
    // Print summary after processing some events
    if log.questions_sent.len() > 0
        && (log.responses_received.len() + log.errors_received.len()) >= log.questions_sent.len()
    {
        println!("\n📊 === EVENT FLOW SUMMARY ===");
        println!("Questions sent: {}", log.questions_sent.len());
        println!("Responses received: {}", log.responses_received.len());
        println!("Errors received: {}", log.errors_received.len());

        println!("\n🔍 Event Details:");
        for (i, (question, q_time)) in log.questions_sent.iter().enumerate() {
            println!("\nQuestion {}: '{}'", i + 1, question);

            // Find corresponding response or error
            if let Some((response, r_time)) = log.responses_received.get(i) {
                let latency = r_time.duration_since(*q_time);
                println!("  ✅ Response received after {:?}", latency);
                println!(
                    "  Response preview: '{}'",
                    if response.len() > 80 {
                        format!("{}...", &response[..80])
                    } else {
                        response.clone()
                    }
                );
            } else if let Some((error, e_time)) = log.errors_received.get(i) {
                let latency = e_time.duration_since(*q_time);
                println!("  ❌ Error received after {:?}", latency);
                println!("  Error: '{}'", error);
            }
        }

        println!("\n✨ This demonstrates CIM's event-driven architecture:");
        println!("   - All interactions are events");
        println!("   - Events flow through the system asynchronously");
        println!("   - Every event is tracked and can be queried");
        println!("   - Events enable loose coupling between components");

        // Exit after showing the summary
        exit.send(AppExit::Success);
    }
}
