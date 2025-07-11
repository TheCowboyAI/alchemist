//! Test workflow system compilation and basic functionality

use alchemist::workflow::{
    Workflow, WorkflowStep, WorkflowAction, WorkflowManager,
    load_workflow_from_yaml,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Testing Alchemist Workflow System");
    
    // Create a simple test workflow
    let workflow = Workflow {
        id: String::new(),
        name: "Test Workflow".to_string(),
        description: Some("A simple test workflow".to_string()),
        steps: vec![
            WorkflowStep {
                id: "step1".to_string(),
                name: "Echo Hello".to_string(),
                description: Some("Print hello message".to_string()),
                action: WorkflowAction::Command {
                    command: "echo".to_string(),
                    args: vec!["Hello, Workflow!".to_string()],
                    env: HashMap::new(),
                },
                dependencies: vec![],
                conditions: vec![],
                retry_config: None,
                timeout_seconds: Some(10),
            },
            WorkflowStep {
                id: "step2".to_string(),
                name: "Echo World".to_string(),
                description: Some("Print world message".to_string()),
                action: WorkflowAction::Command {
                    command: "echo".to_string(),
                    args: vec!["Hello, World!".to_string()],
                    env: HashMap::new(),
                },
                dependencies: vec!["step1".to_string()],
                conditions: vec![],
                retry_config: None,
                timeout_seconds: Some(10),
            },
        ],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    };
    
    // Create workflow manager (without NATS for testing)
    let manager = WorkflowManager::new(None).await?;
    
    // Create the workflow
    let workflow_id = manager.create_workflow(workflow).await?;
    println!("Created workflow: {}", workflow_id);
    
    // List workflows
    let workflows = manager.list_workflows().await?;
    println!("Total workflows: {}", workflows.len());
    
    // Execute the workflow
    println!("Executing workflow...");
    let execution_id = manager.execute_workflow(&workflow_id, HashMap::new()).await?;
    println!("Started execution: {}", execution_id);
    
    // Wait a bit for execution
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Check status
    if let Some(execution) = manager.get_execution(&execution_id).await? {
        println!("Execution status: {:?}", execution.state);
        for (step_id, state) in &execution.step_states {
            println!("  Step {}: {:?}", step_id, state.state);
        }
    }
    
    // Test loading from YAML
    println!("\nTesting YAML loading...");
    match load_workflow_from_yaml("examples/workflows/data_pipeline.yaml").await {
        Ok(workflow) => {
            println!("Loaded workflow: {} ({} steps)", workflow.name, workflow.steps.len());
        }
        Err(e) => {
            println!("Could not load YAML workflow: {}", e);
        }
    }
    
    Ok(())
}