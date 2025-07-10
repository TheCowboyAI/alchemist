//! ECS and Event Sourcing Integration Demo
//! 
//! Shows how Alchemist uses:
//! - ECS (Entity Component System) for state management
//! - Event Sourcing for persistence
//! - DDD aggregates as components

use anyhow::Result;
use bevy::prelude::*;
use cim_domain::{
    Component as DomainComponent,
    events::{Event, EventMetadata},
    commands::Command,
    DomainEvent,
};
use uuid::Uuid;
use chrono::Utc;

// ECS Components representing domain aggregates
#[derive(Component, Debug)]
struct WorkflowAggregate {
    pub id: Uuid,
    pub name: String,
    pub current_state: String,
    pub steps_completed: u32,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Component, Debug)]
struct DialogAggregate {
    pub id: Uuid,
    pub title: String,
    pub model: String,
    pub message_count: u32,
    pub total_tokens: u32,
}

#[derive(Component, Debug)]
struct PolicyAggregate {
    pub id: Uuid,
    pub name: String,
    pub rules: Vec<PolicyRule>,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
struct PolicyRule {
    pub condition: String,
    pub action: String,
}

// Event components that get applied to entities
#[derive(Component)]
struct PendingEvents {
    events: Vec<DomainEvent>,
}

// Resources
#[derive(Resource)]
struct EventStore {
    client: Option<async_nats::Client>,
}

// Systems
fn workflow_command_handler(
    mut commands: Commands,
    mut workflows: Query<(&mut WorkflowAggregate, &mut PendingEvents)>,
) {
    for (mut workflow, mut pending) in workflows.iter_mut() {
        // Simulate state transition
        if workflow.current_state == "pending" {
            let old_state = workflow.current_state.clone();
            workflow.current_state = "processing".to_string();
            
            // Create event
            let event = DomainEvent::WorkflowStateChanged {
                id: workflow.id,
                from: old_state,
                to: workflow.current_state.clone(),
                metadata: EventMetadata {
                    event_id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    causation_id: Some(workflow.id),
                    correlation_id: Some(workflow.id),
                    actor: Some("system".to_string()),
                },
            };
            
            // Queue event for publishing
            pending.events.push(event);
            
            println!("Workflow {} transitioned to processing", workflow.id);
        }
    }
}

fn dialog_message_handler(
    mut dialogs: Query<(&mut DialogAggregate, &mut PendingEvents)>,
) {
    for (mut dialog, mut pending) in dialogs.iter_mut() {
        // Simulate message being added
        dialog.message_count += 1;
        dialog.total_tokens += 50;
        
        // Create event
        let event = DomainEvent::DialogMessageAdded {
            id: dialog.id,
            role: "assistant".to_string(),
            content: "Here's my response...".to_string(),
            tokens: 50,
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                causation_id: Some(dialog.id),
                correlation_id: Some(dialog.id),
                actor: Some("ai-model".to_string()),
            },
        };
        
        pending.events.push(event);
        
        println!("Dialog {} now has {} messages", dialog.id, dialog.message_count);
    }
}

fn event_publisher(
    mut query: Query<(Entity, &mut PendingEvents)>,
    event_store: Res<EventStore>,
    mut commands: Commands,
) {
    for (entity, mut pending) in query.iter_mut() {
        if !pending.events.is_empty() {
            println!("Publishing {} events for entity {:?}", pending.events.len(), entity);
            
            // In a real system, we'd publish to NATS here
            for event in pending.events.drain(..) {
                println!("  - Event: {:?}", event);
                
                // TODO: Publish to JetStream
                // if let Some(client) = &event_store.client {
                //     let subject = format!("events.domain.{}", event.domain());
                //     client.publish(subject, event.to_bytes()).await?;
                // }
            }
        }
    }
}

fn projection_updater(
    workflows: Query<&WorkflowAggregate>,
    dialogs: Query<&DialogAggregate>,
    policies: Query<&PolicyAggregate>,
) {
    // This system would update read-model projections
    let workflow_count = workflows.iter().count();
    let dialog_count = dialogs.iter().count();
    let policy_count = policies.iter().count();
    
    println!("\n📊 Current State:");
    println!("  Workflows: {}", workflow_count);
    println!("  Dialogs: {}", dialog_count);
    println!("  Policies: {}", policy_count);
}

fn main() {
    println!("🎮 ECS + Event Sourcing Demo");
    println!("============================\n");
    
    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(EventStore { client: None })
        .add_systems(Startup, setup)
        .add_systems(Update, (
            workflow_command_handler,
            dialog_message_handler,
            event_publisher,
            projection_updater,
        ).chain())
        .run();
}

fn setup(mut commands: Commands) {
    println!("Setting up domain aggregates...\n");
    
    // Create a workflow aggregate
    commands.spawn((
        WorkflowAggregate {
            id: Uuid::new_v4(),
            name: "Order Processing".to_string(),
            current_state: "pending".to_string(),
            steps_completed: 0,
            created_at: Utc::now(),
        },
        PendingEvents { events: Vec::new() },
    ));
    
    // Create a dialog aggregate
    commands.spawn((
        DialogAggregate {
            id: Uuid::new_v4(),
            title: "Customer Support".to_string(),
            model: "gpt-4".to_string(),
            message_count: 0,
            total_tokens: 0,
        },
        PendingEvents { events: Vec::new() },
    ));
    
    // Create a policy aggregate
    commands.spawn((
        PolicyAggregate {
            id: Uuid::new_v4(),
            name: "api-rate-limit".to_string(),
            rules: vec![
                PolicyRule {
                    condition: "requests_per_minute > 100".to_string(),
                    action: "deny".to_string(),
                },
            ],
            enabled: true,
        },
        PendingEvents { events: Vec::new() },
    ));
    
    println!("✅ Aggregates created\n");
    println!("The ECS systems will now:");
    println!("1. Process commands against aggregates");
    println!("2. Generate domain events");
    println!("3. Queue events for publishing");
    println!("4. Update projections\n");
}