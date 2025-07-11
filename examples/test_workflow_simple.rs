use alchemist::workflow::{Workflow, WorkflowStep, WorkflowAction, WorkflowManager};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Testing workflow system...");
    
    // Create workflow manager
    let manager = WorkflowManager::new(None).await?;
    println!("✓ WorkflowManager created");
    
    // Create a simple workflow
    let workflow = Workflow {
        id: String::new(),
        name: "Test Workflow".to_string(),
        description: Some("Simple test".to_string()),
        steps: vec![
            WorkflowStep {
                id: "step1".to_string(),
                name: "Step 1".to_string(),
                description: None,
                action: WorkflowAction::Command {
                    command: "echo".to_string(),
                    args: vec!["Hello from workflow!".to_string()],
                    env: HashMap::new(),
                },
                dependencies: vec![],
                conditions: vec![],
                retry_config: None,
                timeout_seconds: Some(5),
            },
        ],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    };
    
    let workflow_id = manager.create_workflow(workflow).await?;
    println!("✓ Workflow created: {}", workflow_id);
    
    // List workflows
    let workflows = manager.list_workflows().await?;
    println!("✓ Found {} workflow(s)", workflows.len());
    
    // Execute workflow
    let execution_id = manager.execute_workflow(&workflow_id, HashMap::new()).await?;
    println!("✓ Workflow execution started: {}", execution_id);
    
    // Wait briefly
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    // Check status
    if let Some(execution) = manager.get_execution(&execution_id).await? {
        println!("✓ Execution status: {:?}", execution.state);
    }
    
    println!("\nWorkflow system test completed successfully!");
    Ok(())
}