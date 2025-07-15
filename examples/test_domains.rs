//! Test basic functionality of each domain
//!
//! This example demonstrates that each domain can be used and
//! provides basic functionality.

use alchemist::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Alchemist Domain Functionality ===\n");
    
    // Test Graph domain
    println!("1. Testing Graph Domain:");
    test_graph_domain()?;
    
    // Test Workflow domain
    println!("\n2. Testing Workflow Domain:");
    test_workflow_domain().await?;
    
    // Test Document domain
    println!("\n3. Testing Document Domain:");
    test_document_domain()?;
    
    println!("\n=== All domain tests completed successfully! ===");
    Ok(())
}

fn test_graph_domain() -> Result<(), Box<dyn std::error::Error>> {
    use cim_domain_graph::{GraphAggregate, commands::GraphCommand, events::GraphEvent};
    use uuid::Uuid;
    
    let graph_id = Uuid::new_v4();
    let mut graph = GraphAggregate::new(graph_id);
    
    // Create a graph
    let cmd = GraphCommand::CreateGraph {
        id: graph_id,
        name: "Test Graph".to_string(),
        graph_type: cim_domain_graph::GraphType::Directed,
        metadata: Default::default(),
    };
    
    let events = graph.handle_command(cmd)?;
    println!("  ✓ Created graph with {} event(s)", events.len());
    
    // Apply events
    for event in &events {
        graph.apply_event(event);
    }
    
    // Add a node
    let node_id = Uuid::new_v4();
    let cmd = GraphCommand::AddNode {
        graph_id,
        node_id,
        node_type: cim_domain_graph::NodeType::Process,
        properties: Default::default(),
        position: None,
    };
    
    let events = graph.handle_command(cmd)?;
    println!("  ✓ Added node with {} event(s)", events.len());
    
    Ok(())
}

async fn test_workflow_domain() -> Result<(), Box<dyn std::error::Error>> {
    use cim_domain_workflow::{WorkflowAggregate, commands::WorkflowCommand};
    use uuid::Uuid;
    
    let workflow_id = Uuid::new_v4();
    let mut workflow = WorkflowAggregate::new(workflow_id);
    
    // Create workflow
    let cmd = WorkflowCommand::CreateWorkflow {
        id: workflow_id,
        name: "Test Workflow".to_string(),
        description: Some("A test workflow".to_string()),
        metadata: Default::default(),
    };
    
    let events = workflow.handle_command(cmd)?;
    println!("  ✓ Created workflow with {} event(s)", events.len());
    
    // Apply events
    for event in &events {
        workflow.apply_event(event);
    }
    
    // Add a step
    let step_id = Uuid::new_v4();
    let cmd = WorkflowCommand::AddStep {
        workflow_id,
        step_id,
        name: "Process Data".to_string(),
        step_type: cim_domain_workflow::StepType::Task,
        config: Default::default(),
        dependencies: vec![],
    };
    
    let events = workflow.handle_command(cmd)?;
    println!("  ✓ Added workflow step with {} event(s)", events.len());
    
    Ok(())
}

fn test_document_domain() -> Result<(), Box<dyn std::error::Error>> {
    use cim_domain_document::{DocumentAggregate, commands::DocumentCommand};
    use uuid::Uuid;
    
    let doc_id = Uuid::new_v4();
    let mut document = DocumentAggregate::new(doc_id);
    
    // Create document
    let cmd = DocumentCommand::CreateDocument {
        id: doc_id,
        title: "Test Document".to_string(),
        content: "This is a test document.".to_string(),
        document_type: cim_domain_document::DocumentType::Markdown,
        author: "Test User".to_string(),
        tags: vec!["test".to_string()],
        metadata: Default::default(),
    };
    
    let events = document.handle_command(cmd)?;
    println!("  ✓ Created document with {} event(s)", events.len());
    
    // Apply events
    for event in &events {
        document.apply_event(event);
    }
    
    // Update content
    let cmd = DocumentCommand::UpdateContent {
        id: doc_id,
        content: "Updated content.".to_string(),
        editor: "Test User".to_string(),
    };
    
    let events = document.handle_command(cmd)?;
    println!("  ✓ Updated document content with {} event(s)", events.len());
    
    Ok(())
}