//! Example showing how CIM domains publish events to JetStream
//! 
//! This demonstrates the Event Sourcing pattern where each domain
//! publishes its events to specific JetStream subjects.

use anyhow::Result;
use async_nats::jetstream;
use chrono::Utc;
use cim_domain::{
    DomainEvent, EventMetadata, EventStore,
    commands::Command,
    events::Event,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to NATS
    let client = async_nats::connect("nats://localhost:4222").await?;
    let jetstream = jetstream::new(client);
    
    println!("🚀 Domain Event Publisher Example");
    println!("=================================\n");
    
    // Create event stream if it doesn't exist
    create_event_stream(&jetstream).await?;
    
    // Simulate events from different domains
    println!("Publishing domain events...\n");
    
    // 1. Workflow domain events
    publish_workflow_events(&jetstream).await?;
    
    // 2. Dialog domain events
    publish_dialog_events(&jetstream).await?;
    
    // 3. Agent domain events
    publish_agent_events(&jetstream).await?;
    
    // 4. Policy domain events
    publish_policy_events(&jetstream).await?;
    
    println!("\n✅ All events published successfully!");
    println!("\nThese events are now available for:");
    println!("- Dashboard consumption");
    println!("- Event replay");
    println!("- Projections");
    println!("- Analytics");
    
    Ok(())
}

async fn create_event_stream(js: &jetstream::Context) -> Result<()> {
    let stream_name = "CIM-EVENTS";
    
    match js.get_stream(stream_name).await {
        Ok(_) => {
            println!("✓ Event stream '{}' already exists", stream_name);
        }
        Err(_) => {
            println!("Creating event stream '{}'...", stream_name);
            
            js.create_stream(jetstream::stream::Config {
                name: stream_name.to_string(),
                subjects: vec![
                    "events.>".to_string(), // All events
                ],
                retention: jetstream::stream::RetentionPolicy::Limits,
                max_messages: 10_000_000,
                max_age: std::time::Duration::from_secs(86400 * 30), // 30 days
                storage: jetstream::stream::StorageType::File,
                num_replicas: 1,
                ..Default::default()
            }).await?;
            
            println!("✓ Event stream created");
        }
    }
    
    Ok(())
}

async fn publish_workflow_events(js: &jetstream::Context) -> Result<()> {
    let workflow_id = Uuid::new_v4();
    
    // Workflow created event
    let event = DomainEvent::WorkflowCreated {
        id: workflow_id,
        name: "Order Processing".to_string(),
        initial_state: "pending".to_string(),
        metadata: EventMetadata {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            causation_id: Some(Uuid::new_v4()),
            correlation_id: Some(Uuid::new_v4()),
            actor: Some("system".to_string()),
        },
    };
    
    publish_event(js, "events.domain.workflow.created", &event).await?;
    
    // Workflow state changed event
    let event = DomainEvent::WorkflowStateChanged {
        id: workflow_id,
        from: "pending".to_string(),
        to: "processing".to_string(),
        metadata: EventMetadata {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            causation_id: Some(Uuid::new_v4()),
            correlation_id: Some(Uuid::new_v4()),
            actor: Some("system".to_string()),
        },
    };
    
    publish_event(js, "events.domain.workflow.state_changed", &event).await?;
    
    println!("✓ Published workflow events");
    Ok(())
}

async fn publish_dialog_events(js: &jetstream::Context) -> Result<()> {
    let dialog_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    
    // Dialog created event
    let event = DomainEvent::DialogCreated {
        id: dialog_id,
        title: "System Architecture Discussion".to_string(),
        model: "claude-3".to_string(),
        user_id,
        metadata: EventMetadata {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            causation_id: Some(Uuid::new_v4()),
            correlation_id: Some(Uuid::new_v4()),
            actor: Some(user_id.to_string()),
        },
    };
    
    publish_event(js, "events.domain.dialog.created", &event).await?;
    
    // Message added event
    let event = DomainEvent::DialogMessageAdded {
        id: dialog_id,
        role: "user".to_string(),
        content: "Explain the event sourcing architecture".to_string(),
        tokens: 10,
        metadata: EventMetadata {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            causation_id: Some(Uuid::new_v4()),
            correlation_id: Some(Uuid::new_v4()),
            actor: Some(user_id.to_string()),
        },
    };
    
    publish_event(js, "events.domain.dialog.message_added", &event).await?;
    
    println!("✓ Published dialog events");
    Ok(())
}

async fn publish_agent_events(js: &jetstream::Context) -> Result<()> {
    let query_id = Uuid::new_v4();
    
    // Agent query executed event
    let event = DomainEvent::AgentQueryExecuted {
        id: query_id,
        model: "gpt-4".to_string(),
        prompt: "Analyze customer sentiment from recent feedback".to_string(),
        tokens_used: 150,
        cost: 0.0045,
        metadata: EventMetadata {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            causation_id: Some(Uuid::new_v4()),
            correlation_id: Some(Uuid::new_v4()),
            actor: Some("sentiment-analyzer".to_string()),
        },
    };
    
    publish_event(js, "events.domain.agent.query_executed", &event).await?;
    
    println!("✓ Published agent events");
    Ok(())
}

async fn publish_policy_events(js: &jetstream::Context) -> Result<()> {
    let policy_id = Uuid::new_v4();
    let subject_id = Uuid::new_v4();
    
    // Policy evaluated event
    let event = DomainEvent::PolicyEvaluated {
        id: policy_id,
        subject: subject_id.to_string(),
        decision: "allow".to_string(),
        rules_evaluated: 3,
        metadata: EventMetadata {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            causation_id: Some(Uuid::new_v4()),
            correlation_id: Some(Uuid::new_v4()),
            actor: Some("policy-engine".to_string()),
        },
    };
    
    publish_event(js, "events.domain.policy.evaluated", &event).await?;
    
    println!("✓ Published policy events");
    Ok(())
}

async fn publish_event(
    js: &jetstream::Context,
    subject: &str,
    event: &DomainEvent,
) -> Result<()> {
    let payload = serde_json::to_vec(event)?;
    
    js.publish(subject, payload.into()).await?;
    
    Ok(())
}

// Define domain events (normally these would be in each domain crate)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum DomainEvent {
    // Workflow events
    WorkflowCreated {
        id: Uuid,
        name: String,
        initial_state: String,
        metadata: EventMetadata,
    },
    WorkflowStateChanged {
        id: Uuid,
        from: String,
        to: String,
        metadata: EventMetadata,
    },
    
    // Dialog events
    DialogCreated {
        id: Uuid,
        title: String,
        model: String,
        user_id: Uuid,
        metadata: EventMetadata,
    },
    DialogMessageAdded {
        id: Uuid,
        role: String,
        content: String,
        tokens: u32,
        metadata: EventMetadata,
    },
    
    // Agent events
    AgentQueryExecuted {
        id: Uuid,
        model: String,
        prompt: String,
        tokens_used: u32,
        cost: f64,
        metadata: EventMetadata,
    },
    
    // Policy events
    PolicyEvaluated {
        id: Uuid,
        subject: String,
        decision: String,
        rules_evaluated: u32,
        metadata: EventMetadata,
    },
}