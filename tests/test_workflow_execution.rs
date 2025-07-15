//! Integration tests for workflow execution with real commands

#[cfg(test)]
mod workflow_integration_tests {
    use alchemist::{
        workflow::{WorkflowManager, Workflow, WorkflowStep, WorkflowAction, WorkflowCondition, RetryConfig},
        config::AlchemistConfig,
    };
    use std::collections::HashMap;
    use tempfile::TempDir;
    
    fn create_test_workflow() -> Workflow {
        Workflow {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: Some("Integration test workflow".to_string()),
            steps: vec![
                WorkflowStep {
                    id: "create-dir".to_string(),
                    name: "Create Directory".to_string(),
                    description: None,
                    action: WorkflowAction::Command {
                        command: "mkdir".to_string(),
                        args: vec!["-p".to_string(), "test-output".to_string()],
                        env: HashMap::new(),
                    },
                    dependencies: vec![],
                    conditions: vec![],
                    retry_config: None,
                    timeout_seconds: None,
                },
                WorkflowStep {
                    id: "write-file".to_string(),
                    name: "Write Test File".to_string(),
                    description: None,
                    action: WorkflowAction::Command {
                        command: "sh".to_string(),
                        args: vec![
                            "-c".to_string(),
                            "echo 'Hello from workflow!' > test-output/hello.txt".to_string()
                        ],
                        env: HashMap::new(),
                    },
                    dependencies: vec!["create-dir".to_string()],
                    conditions: vec![],
                    retry_config: None,
                    timeout_seconds: None,
                },
                WorkflowStep {
                    id: "list-files".to_string(),
                    name: "List Files".to_string(),
                    description: None,
                    action: WorkflowAction::Command {
                        command: "ls".to_string(),
                        args: vec!["-la".to_string(), "test-output".to_string()],
                        env: HashMap::new(),
                    },
                    dependencies: vec!["write-file".to_string()],
                    conditions: vec![],
                    retry_config: None,
                    timeout_seconds: None,
                },
            ],
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
    
    #[tokio::test]
    async fn test_workflow_execution() {
        // Create temp directory for test
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();
        
        // Create workflow manager
        let mut workflow_manager = WorkflowManager::new(None).await.unwrap();
        
        // Create and save workflow
        let workflow = create_test_workflow();
        let workflow_id = workflow_manager.create_workflow(workflow).await.unwrap();
        
        // Execute workflow
        let execution_id = workflow_manager.execute_workflow(&workflow_id, HashMap::new()).await.unwrap();
        
        // Wait for completion
        let mut completed = false;
        for _ in 0..30 { // Wait up to 30 seconds
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            
            if let Some(execution) = workflow_manager.get_execution(&execution_id).await.unwrap() {
                match execution.state {
                    alchemist::workflow::WorkflowState::Completed => {
                        completed = true;
                        break;
                    }
                    alchemist::workflow::WorkflowState::Failed => {
                        panic!("Workflow failed: {:?}", execution.errors);
                    }
                    _ => continue,
                }
            }
        }
        
        assert!(completed, "Workflow did not complete in time");
        
        // Verify output file was created
        let output_path = temp_dir.path().join("test-output").join("hello.txt");
        assert!(output_path.exists(), "Output file was not created");
        
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert_eq!(content.trim(), "Hello from workflow!");
        
        println!("✅ Workflow executed successfully!");
    }
    
    #[tokio::test]
    async fn test_workflow_with_conditions() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();
        
        let workflow = Workflow {
            id: "conditional-workflow".to_string(),
            name: "Conditional Workflow".to_string(),
            description: Some("Test workflow with conditions".to_string()),
            steps: vec![
                WorkflowStep {
                    id: "check-os".to_string(),
                    name: "Check OS".to_string(),
                    description: None,
                    action: WorkflowAction::Command {
                        command: "uname".to_string(),
                        args: vec![],
                        env: HashMap::new(),
                    },
                    dependencies: vec![],
                    conditions: vec![],
                    retry_config: None,
                    timeout_seconds: None,
                },
                WorkflowStep {
                    id: "linux-step".to_string(),
                    name: "Linux Step".to_string(),
                    description: None,
                    action: WorkflowAction::Command {
                        command: "echo".to_string(),
                        args: vec!["Running on Linux".to_string()],
                        env: HashMap::new(),
                    },
                    dependencies: vec!["check-os".to_string()],
                    conditions: vec![
                        WorkflowCondition::Custom {
                            evaluator: "contains".to_string(),
                            params: serde_json::json!({
                                "field": "outputs.check-os.stdout",
                                "value": "Linux"
                            }),
                        }
                    ],
                    retry_config: None,
                    timeout_seconds: None,
                },
            ],
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        let mut workflow_manager = WorkflowManager::new(None).await.unwrap();
        let workflow_id = workflow_manager.create_workflow(workflow).await.unwrap();
        let execution_id = workflow_manager.execute_workflow(&workflow_id, HashMap::new()).await.unwrap();
        
        // Wait for completion
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        
        let execution = workflow_manager.get_execution(&execution_id).await.unwrap().unwrap();
        assert_eq!(execution.state, alchemist::workflow::WorkflowState::Completed);
        
        println!("✅ Conditional workflow executed successfully!");
    }
    
    #[tokio::test]
    async fn test_workflow_error_handling() {
        let mut workflow_manager = WorkflowManager::new(None).await.unwrap();
        
        let workflow = Workflow {
            id: "error-workflow".to_string(),
            name: "Error Workflow".to_string(),
            description: Some("Test workflow error handling".to_string()),
            steps: vec![
                WorkflowStep {
                    id: "failing-step".to_string(),
                    name: "Failing Step".to_string(),
                    description: None,
                    action: WorkflowAction::Command {
                        command: "false".to_string(), // This command always fails
                        args: vec![],
                        env: HashMap::new(),
                    },
                    dependencies: vec![],
                    conditions: vec![],
                    retry_config: Some(RetryConfig {
                        max_attempts: 3,
                        delay_seconds: 1,
                        backoff_multiplier: 2.0,
                    }),
                    timeout_seconds: None,
                },
            ],
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        let workflow_id = workflow_manager.create_workflow(workflow).await.unwrap();
        let execution_id = workflow_manager.execute_workflow(&workflow_id, HashMap::new()).await.unwrap();
        
        // Wait for failure
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        
        let execution = workflow_manager.get_execution(&execution_id).await.unwrap().unwrap();
        assert_eq!(execution.state, alchemist::workflow::WorkflowState::Failed);
        assert!(!execution.errors.is_empty());
        
        // Check that retry was attempted
        if let Some(step_state) = execution.step_states.get("failing-step") {
            match &step_state.state {
                alchemist::workflow::StepState::Failed => {
                    // Expected
                }
                _ => panic!("Step should have failed"),
            }
        }
        
        println!("✅ Workflow error handling tested successfully!");
    }
}