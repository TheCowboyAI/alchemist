//! Example demonstrating WorkflowGraph with a document approval process
//!
//! This example shows:
//! - Creating a workflow graph with multiple states
//! - Adding transitions with guards and enrichment values
//! - Finding optimal paths based on business value
//! - Validating workflow structure

use cim_workflow_graph::{
    WorkflowGraph, WorkflowType, EnrichmentType, BusinessValue,
};
use cim_domain::workflow::{
    WorkflowState, SimpleState, SimpleTransition, SimpleInput, SimpleOutput,
    WorkflowContext, ContextKeyGuard, ActorGuard,
};
use cim_domain::{GraphId, StateId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

fn main() {
    println!("=== Document Approval Workflow Example ===\n");

    // Create workflow graph
    let mut workflow = WorkflowGraph::new_with_name(
        "Document Approval Process".to_string(),
        WorkflowType::Sequential,
        EnrichmentType::BusinessValue,
    );

    // Define states
    let draft = SimpleState::new("Draft");

    let review = SimpleState::new("UnderReview");

    let approved = SimpleState::new("Approved");

    let rejected = SimpleState::new("Rejected");

    let published = SimpleState::terminal("Published");

    let archived = SimpleState::terminal("Archived");

    // Add states to workflow
    println!("Adding states to workflow...");
    workflow.add_state(draft.clone());
    workflow.add_state(review.clone());
    workflow.add_state(approved.clone());
    workflow.add_state(rejected.clone());
    workflow.add_state(published.clone());
    workflow.add_state(archived.clone());

    // Create transitions with guards
    println!("\nAdding transitions...");

    // Draft -> Review (requires author)
    let submit_for_review = SimpleTransition::new(
        "submit_for_review",
        draft.clone(),
        review.clone(),
        SimpleInput::new("submit"),
        SimpleOutput::new("submitted"),
    ).with_guard(Box::new(ContextKeyGuard::new("author")))
     .with_description("Submit document for review");

    workflow.add_transition(
        Box::new(submit_for_review),
        BusinessValue {
            monetary_value: 0.0,
            time_cost: Duration::from_secs(60), // 1 minute
            risk_factor: 0.1,
        },
    ).unwrap();

    // Review -> Approved (requires reviewer role)
    let approve = SimpleTransition::new(
        "approve",
        review.clone(),
        approved.clone(),
        SimpleInput::new("approve"),
        SimpleOutput::new("approved"),
    ).with_guard(Box::new(ActorGuard::single("reviewer")))
     .with_description("Approve the document");

    workflow.add_transition(
        Box::new(approve),
        BusinessValue {
            monetary_value: 100.0, // Value of approved document
            time_cost: Duration::from_secs(300), // 5 minutes
            risk_factor: 0.2,
        },
    ).unwrap();

    // Review -> Rejected (requires reviewer role)
    let reject = SimpleTransition::new(
        "reject",
        review.clone(),
        rejected.clone(),
        SimpleInput::new("reject"),
        SimpleOutput::new("rejected"),
    ).with_guard(Box::new(ActorGuard::single("reviewer")))
     .with_description("Reject the document");

    workflow.add_transition(
        Box::new(reject),
        BusinessValue {
            monetary_value: -50.0, // Cost of rejection
            time_cost: Duration::from_secs(180), // 3 minutes
            risk_factor: 0.5,
        },
    ).unwrap();

    // Rejected -> Draft (for revision)
    let revise = SimpleTransition::new(
        "revise",
        rejected.clone(),
        draft.clone(),
        SimpleInput::new("revise"),
        SimpleOutput::new("revision_started"),
    ).with_description("Send document back for revision");

    workflow.add_transition(
        Box::new(revise),
        BusinessValue {
            monetary_value: 0.0,
            time_cost: Duration::from_secs(600), // 10 minutes
            risk_factor: 0.3,
        },
    ).unwrap();

    // Approved -> Published (requires publisher role)
    let publish = SimpleTransition::new(
        "publish",
        approved.clone(),
        published.clone(),
        SimpleInput::new("publish"),
        SimpleOutput::new("published"),
    ).with_guard(Box::new(ActorGuard::single("publisher")))
     .with_description("Publish the approved document");

    workflow.add_transition(
        Box::new(publish),
        BusinessValue {
            monetary_value: 500.0, // Value of published document
            time_cost: Duration::from_secs(120), // 2 minutes
            risk_factor: 0.1,
        },
    ).unwrap();

    // Published -> Archived (automatic after time)
    let archive = SimpleTransition::new(
        "archive",
        published.clone(),
        archived.clone(),
        SimpleInput::new("archive"),
        SimpleOutput::new("archived"),
    ).with_description("Archive the published document");

    workflow.add_transition(
        Box::new(archive),
        BusinessValue {
            monetary_value: 0.0,
            time_cost: Duration::from_secs(30), // 30 seconds
            risk_factor: 0.0,
        },
    ).unwrap();

    // Validate workflow
    println!("\nValidating workflow...");
    match workflow.validate() {
        Ok(()) => println!("✓ Workflow is valid"),
        Err(e) => println!("✗ Workflow validation failed: {}", e),
    }

    // Display workflow statistics
    println!("\nWorkflow Statistics:");
    println!("- Total states: {}", workflow.states().count());
    println!("- Total transitions: {}", workflow.transitions().count());
    println!("- Initial states: {:?}",
        workflow.initial_states()
            .iter()
            .map(|s| s.name())
            .collect::<Vec<_>>()
    );
    println!("- Terminal states: {:?}",
        workflow.terminal_states()
            .iter()
            .map(|s| s.name())
            .collect::<Vec<_>>()
    );

    // Simulate workflow execution
    println!("\n=== Simulating Workflow Execution ===");

    // Create context with author
    let mut context = WorkflowContext::new();
    context.set("author", "John Doe");
    context.set("actor", "reviewer");

    // Find transitions from Draft state
    let draft_state = workflow.get_state(&draft.id()).unwrap();
    let submit_input = SimpleInput::new("submit");

    println!("\nFrom Draft state with 'submit' input:");
    let transitions = workflow.find_transitions(draft_state, &submit_input, &context);

    for (transition, value, _) in &transitions {
        println!("  - {} -> {} (value: ${:.2}, time: {:?}, risk: {:.2})",
            transition.source().name(),
            transition.target().name(),
            value.monetary_value,
            value.time_cost,
            value.risk_factor
        );
    }

    // Find optimal transition
    if let Some((optimal, value, _)) = workflow.find_optimal_transition(draft_state, &submit_input, &context) {
        println!("\nOptimal transition: {} -> {} (business value score: {:.2})",
            optimal.source().name(),
            optimal.target().name(),
            value.monetary_value - value.time_cost.as_secs_f64() - value.risk_factor * 1000.0
        );
    }

    // Simulate approval path
    println!("\n=== Simulating Approval Path ===");

    let review_state = workflow.get_state(&review.id()).unwrap();
    let approve_input = SimpleInput::new("approve");

    let approval_transitions = workflow.find_transitions(review_state, &approve_input, &context);

    if !approval_transitions.is_empty() {
        println!("From Review state, reviewer can:");
        for (transition, _, _) in &approval_transitions {
            println!("  - {}: {}",
                transition.id().to_string().split('_').last().unwrap_or(""),
                transition.target().name()
            );
        }
    }

    // Calculate total value of happy path
    println!("\n=== Business Value Analysis ===");

    let happy_path = vec![
        ("Draft", "UnderReview", 0.0, 60, 0.1),
        ("UnderReview", "Approved", 100.0, 300, 0.2),
        ("Approved", "Published", 500.0, 120, 0.1),
    ];

    let total_value: f64 = happy_path.iter().map(|(_, _, v, _, _)| v).sum();
    let total_time: u64 = happy_path.iter().map(|(_, _, _, t, _)| t).sum();
    let max_risk: f64 = happy_path.iter()
        .map(|(_, _, _, _, r)| *r)
        .fold(0.0, |acc, r| acc.max(r));

    println!("Happy path (Draft -> Review -> Approved -> Published):");
    println!("  - Total monetary value: ${:.2}", total_value);
    println!("  - Total time cost: {} seconds", total_time);
    println!("  - Maximum risk factor: {:.2}", max_risk);
    println!("  - Net business value score: {:.2}",
        total_value - total_time as f64 - max_risk * 1000.0
    );

    println!("\n=== Workflow Example Complete ===");
}
