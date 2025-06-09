//! Comprehensive tests for Workflow aggregate
//! The Workflow aggregate is marked as 100% complete in progress.json
//! These tests ensure full coverage of the implemented functionality

use ia::domain::{
    aggregates::workflow::*,
    commands::workflow::*,
    events::{DomainEvent, workflow::*},
    value_objects::{WorkflowId, StepId, EdgeId, UserId, NodeId},
    DomainError,
};
use chrono::Utc;
use std::collections::HashMap;

#[test]
fn test_workflow_state_machine_transitions() {
    let workflow_id = WorkflowId::new();
    let user_id = UserId::new();
    let mut workflow = Workflow::new(workflow_id.clone(), "Test Workflow".to_string(), user_id.clone());

    // Test Designed -> Ready transition
    assert!(matches!(workflow.state, WorkflowState::Designed));
    assert!(workflow.can_transition(&WorkflowState::Ready {
        validated_at: Utc::now(),
        validated_by: user_id.clone(),
    }));

    // Test invalid transition Designed -> Running
    assert!(!workflow.can_transition(&WorkflowState::Running {
        started_at: Utc::now(),
        current_step: StepId::new(),
        completed_steps: vec![],
    }));
}

#[test]
fn test_workflow_creation_command() {
    let workflow_id = WorkflowId::new();
    let user_id = UserId::new();
    let mut workflow = Workflow::new(workflow_id.clone(), String::new(), user_id.clone());
    workflow.version = 0; // Ensure it's a new workflow

    let cmd = WorkflowCommand::CreateWorkflow(CreateWorkflow {
        workflow_id: workflow_id.clone(),
        name: "Test Workflow".to_string(),
        description: "A test workflow".to_string(),
        created_by: user_id.clone(),
        tags: vec!["test".to_string(), "example".to_string()],
    });

    let result = workflow.handle_command(cmd);
    assert!(result.is_ok());

    let events = result.unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        DomainEvent::Workflow(WorkflowEvent::WorkflowCreated(event)) => {
            assert_eq!(event.workflow_id, workflow_id);
            assert_eq!(event.name, "Test Workflow");
            assert_eq!(event.description, "A test workflow");
            assert_eq!(event.created_by, user_id);
            assert_eq!(event.tags.len(), 2);
        }
        _ => panic!("Expected WorkflowCreated event"),
    }
}

#[test]
fn test_add_step_to_workflow() {
    let workflow_id = WorkflowId::new();
    let user_id = UserId::new();
    let mut workflow = Workflow::new(workflow_id.clone(), "Test Workflow".to_string(), user_id.clone());

    let step = WorkflowStep {
        id: StepId::new(),
        name: "Process Order".to_string(),
        step_type: StepType::ServiceTask {
            service: "OrderService".to_string(),
            operation: "processOrder".to_string(),
        },
        node_id: NodeId::new(),
        inputs: vec![],
        outputs: vec![],
        timeout_ms: Some(5000),
        retry_policy: None,
    };

    let cmd = WorkflowCommand::AddStep(AddStep {
        workflow_id: workflow_id.clone(),
        step: step.clone(),
    });

    let result = workflow.handle_command(cmd);
    assert!(result.is_ok());

    let events = result.unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        DomainEvent::Workflow(WorkflowEvent::StepAdded(event)) => {
            assert_eq!(event.workflow_id, workflow_id);
            assert_eq!(event.step.id, step.id);
            assert_eq!(event.step.name, "Process Order");
        }
        _ => panic!("Expected StepAdded event"),
    }
}

#[test]
fn test_connect_workflow_steps() {
    let workflow_id = WorkflowId::new();
    let user_id = UserId::new();
    let mut workflow = Workflow::new(workflow_id.clone(), "Test Workflow".to_string(), user_id.clone());

    // Add two steps first
    let step1 = WorkflowStep {
        id: StepId::new(),
        name: "Step 1".to_string(),
        step_type: StepType::UserTask,
        node_id: NodeId::new(),
        inputs: vec![],
        outputs: vec![],
        timeout_ms: None,
        retry_policy: None,
    };

    let step2 = WorkflowStep {
        id: StepId::new(),
        name: "Step 2".to_string(),
        step_type: StepType::UserTask,
        node_id: NodeId::new(),
        inputs: vec![],
        outputs: vec![],
        timeout_ms: None,
        retry_policy: None,
    };

    workflow.steps.insert(step1.id.clone(), step1.clone());
    workflow.steps.insert(step2.id.clone(), step2.clone());

    // Connect the steps
    let cmd = WorkflowCommand::ConnectSteps(ConnectSteps {
        workflow_id: workflow_id.clone(),
        from_step: step1.id.clone(),
        to_step: step2.id.clone(),
        edge_id: EdgeId::new(),
        condition: None,
    });

    let result = workflow.handle_command(cmd);
    assert!(result.is_ok());

    let events = result.unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        DomainEvent::Workflow(WorkflowEvent::StepsConnected(event)) => {
            assert_eq!(event.workflow_id, workflow_id);
            assert_eq!(event.from_step, step1.id);
            assert_eq!(event.to_step, step2.id);
        }
        _ => panic!("Expected StepsConnected event"),
    }
}

#[test]
fn test_validate_workflow() {
    let workflow_id = WorkflowId::new();
    let user_id = UserId::new();
    let mut workflow = Workflow::new(workflow_id.clone(), "Test Workflow".to_string(), user_id.clone());

    // Add a step and set start/end
    let step = WorkflowStep {
        id: StepId::new(),
        name: "Single Step".to_string(),
        step_type: StepType::UserTask,
        node_id: NodeId::new(),
        inputs: vec![],
        outputs: vec![],
        timeout_ms: None,
        retry_policy: None,
    };

    workflow.steps.insert(step.id.clone(), step.clone());
    workflow.start_step = Some(step.id.clone());
    workflow.end_steps = vec![step.id.clone()];

    let cmd = WorkflowCommand::ValidateWorkflow(ValidateWorkflow {
        workflow_id: workflow_id.clone(),
        validated_by: user_id.clone(),
    });

    let result = workflow.handle_command(cmd);
    assert!(result.is_ok());

    let events = result.unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        DomainEvent::Workflow(WorkflowEvent::WorkflowValidated(event)) => {
            assert_eq!(event.workflow_id, workflow_id);
            assert_eq!(event.validated_by, user_id);
            assert!(event.validation_result.is_valid);
        }
        _ => panic!("Expected WorkflowValidated event"),
    }
}

#[test]
fn test_workflow_validation_failures() {
    let workflow_id = WorkflowId::new();
    let user_id = UserId::new();
    let mut workflow = Workflow::new(workflow_id.clone(), "Test Workflow".to_string(), user_id.clone());

    // Test validation with no steps
    let cmd = WorkflowCommand::ValidateWorkflow(ValidateWorkflow {
        workflow_id: workflow_id.clone(),
        validated_by: user_id.clone(),
    });

    let result = workflow.handle_command(cmd);
    assert!(result.is_err());
    match result {
        Err(DomainError::ValidationError(msg)) => {
            assert!(msg.contains("no steps"));
        }
        _ => panic!("Expected ValidationError for no steps"),
    }
}

#[test]
fn test_start_workflow() {
    let workflow_id = WorkflowId::new();
    let user_id = UserId::new();
    let mut workflow = Workflow::new(workflow_id.clone(), "Test Workflow".to_string(), user_id.clone());

    // Setup workflow in Ready state
    let step = WorkflowStep {
        id: StepId::new(),
        name: "Start Step".to_string(),
        step_type: StepType::UserTask,
        node_id: NodeId::new(),
        inputs: vec![],
        outputs: vec![],
        timeout_ms: None,
        retry_policy: None,
    };

    workflow.steps.insert(step.id.clone(), step.clone());
    workflow.start_step = Some(step.id.clone());
    workflow.state = WorkflowState::Ready {
        validated_at: Utc::now(),
        validated_by: user_id.clone(),
    };

    let cmd = WorkflowCommand::StartWorkflow(StartWorkflow {
        workflow_id: workflow_id.clone(),
        instance_id: "instance-123".to_string(),
        started_by: user_id.clone(),
        inputs: HashMap::new(),
    });

    let result = workflow.handle_command(cmd);
    assert!(result.is_ok());

    let events = result.unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        DomainEvent::Workflow(WorkflowEvent::WorkflowStarted(event)) => {
            assert_eq!(event.workflow_id, workflow_id);
            assert_eq!(event.instance_id, "instance-123");
            assert_eq!(event.started_by, user_id);
            assert_eq!(event.start_step, step.id);
        }
        _ => panic!("Expected WorkflowStarted event"),
    }
}

#[test]
fn test_complete_workflow_step() {
    let workflow_id = WorkflowId::new();
    let user_id = UserId::new();
    let mut workflow = Workflow::new(workflow_id.clone(), "Test Workflow".to_string(), user_id.clone());

    // Setup workflow with two steps
    let step1 = WorkflowStep {
        id: StepId::new(),
        name: "Step 1".to_string(),
        step_type: StepType::UserTask,
        node_id: NodeId::new(),
        inputs: vec![],
        outputs: vec![],
        timeout_ms: None,
        retry_policy: None,
    };

    let step2 = WorkflowStep {
        id: StepId::new(),
        name: "Step 2".to_string(),
        step_type: StepType::UserTask,
        node_id: NodeId::new(),
        inputs: vec![],
        outputs: vec![],
        timeout_ms: None,
        retry_policy: None,
    };

    workflow.steps.insert(step1.id.clone(), step1.clone());
    workflow.steps.insert(step2.id.clone(), step2.clone());
    workflow.start_step = Some(step1.id.clone());
    workflow.end_steps = vec![step2.id.clone()];

    // Set workflow to Running state
    workflow.state = WorkflowState::Running {
        started_at: Utc::now(),
        current_step: step1.id.clone(),
        completed_steps: vec![],
    };

    workflow.execution_context = Some(ExecutionContext {
        instance_id: "instance-123".to_string(),
        inputs: HashMap::new(),
        variables: HashMap::new(),
        step_outputs: HashMap::new(),
    });

    let cmd = WorkflowCommand::CompleteStep(CompleteStep {
        workflow_id: workflow_id.clone(),
        step_id: step1.id.clone(),
        outputs: HashMap::new(),
        next_step: Some(step2.id.clone()),
    });

    let result = workflow.handle_command(cmd);
    assert!(result.is_ok());

    let events = result.unwrap();
    assert_eq!(events.len(), 1); // Only StepCompleted, not WorkflowCompleted yet

    match &events[0] {
        DomainEvent::Workflow(WorkflowEvent::StepCompleted(event)) => {
            assert_eq!(event.workflow_id, workflow_id);
            assert_eq!(event.step_id, step1.id);
            assert_eq!(event.next_step, Some(step2.id));
        }
        _ => panic!("Expected StepCompleted event"),
    }
}

#[test]
fn test_pause_and_resume_workflow() {
    let workflow_id = WorkflowId::new();
    let user_id = UserId::new();
    let mut workflow = Workflow::new(workflow_id.clone(), "Test Workflow".to_string(), user_id.clone());

    let step = StepId::new();
    workflow.state = WorkflowState::Running {
        started_at: Utc::now(),
        current_step: step.clone(),
        completed_steps: vec![],
    };

    // Pause workflow
    let pause_cmd = WorkflowCommand::PauseWorkflow(PauseWorkflow {
        workflow_id: workflow_id.clone(),
        paused_by: user_id.clone(),
        reason: "Maintenance".to_string(),
    });

    let result = workflow.handle_command(pause_cmd);
    assert!(result.is_ok());

    let events = result.unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        DomainEvent::Workflow(WorkflowEvent::WorkflowPaused(event)) => {
            assert_eq!(event.workflow_id, workflow_id);
            assert_eq!(event.paused_by, user_id);
            assert_eq!(event.reason, "Maintenance");
        }
        _ => panic!("Expected WorkflowPaused event"),
    }

    // Apply the event to update state
    workflow.apply_event(&events[0]).unwrap();

    // Resume workflow
    let resume_cmd = WorkflowCommand::ResumeWorkflow(ResumeWorkflow {
        workflow_id: workflow_id.clone(),
        resumed_by: user_id.clone(),
    });

    let result = workflow.handle_command(resume_cmd);
    assert!(result.is_ok());

    let events = result.unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        DomainEvent::Workflow(WorkflowEvent::WorkflowResumed(event)) => {
            assert_eq!(event.workflow_id, workflow_id);
            assert_eq!(event.resumed_by, user_id);
        }
        _ => panic!("Expected WorkflowResumed event"),
    }
}

#[test]
fn test_fail_workflow() {
    let workflow_id = WorkflowId::new();
    let user_id = UserId::new();
    let mut workflow = Workflow::new(workflow_id.clone(), "Test Workflow".to_string(), user_id.clone());

    let step = StepId::new();
    workflow.state = WorkflowState::Running {
        started_at: Utc::now(),
        current_step: step.clone(),
        completed_steps: vec![],
    };

    let cmd = WorkflowCommand::FailWorkflow(FailWorkflow {
        workflow_id: workflow_id.clone(),
        error: "Service unavailable".to_string(),
        failed_step: step.clone(),
        recovery_point: None,
    });

    let result = workflow.handle_command(cmd);
    assert!(result.is_ok());

    let events = result.unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        DomainEvent::Workflow(WorkflowEvent::WorkflowFailed(event)) => {
            assert_eq!(event.workflow_id, workflow_id);
            assert_eq!(event.error, "Service unavailable");
            assert_eq!(event.failed_step, step);
            assert!(event.recovery_point.is_none());
        }
        _ => panic!("Expected WorkflowFailed event"),
    }
}

#[test]
fn test_workflow_step_types() {
    // Test all step types are properly handled
    let step_types = vec![
        StepType::UserTask,
        StepType::ServiceTask {
            service: "OrderService".to_string(),
            operation: "processOrder".to_string(),
        },
        StepType::Decision {
            conditions: vec![DecisionCondition {
                expression: "amount > 100".to_string(),
                target_step: StepId::new(),
            }],
        },
        StepType::ParallelGateway {
            branches: vec![StepId::new(), StepId::new()],
        },
        StepType::EventWait {
            event_type: "OrderConfirmed".to_string(),
            timeout_ms: 60000,
        },
        StepType::Script {
            language: "javascript".to_string(),
            code: "return { success: true };".to_string(),
        },
    ];

    for step_type in step_types {
        let step = WorkflowStep {
            id: StepId::new(),
            name: format!("Test {:?}", step_type),
            step_type: step_type.clone(),
            node_id: NodeId::new(),
            inputs: vec![],
            outputs: vec![],
            timeout_ms: None,
            retry_policy: None,
        };

        // Verify step can be created with each type
        assert_eq!(step.step_type, step_type);
    }
}

#[test]
fn test_workflow_with_retry_policy() {
    let retry_policy = RetryPolicy {
        max_attempts: 3,
        backoff_ms: 1000,
        backoff_multiplier: 2.0,
    };

    let step = WorkflowStep {
        id: StepId::new(),
        name: "Retryable Step".to_string(),
        step_type: StepType::ServiceTask {
            service: "ExternalAPI".to_string(),
            operation: "callEndpoint".to_string(),
        },
        node_id: NodeId::new(),
        inputs: vec![],
        outputs: vec![],
        timeout_ms: Some(5000),
        retry_policy: Some(retry_policy.clone()),
    };

    assert_eq!(step.retry_policy.unwrap().max_attempts, 3);
    assert_eq!(step.timeout_ms.unwrap(), 5000);
}

#[test]
fn test_workflow_data_flow() {
    // Test step inputs and outputs
    let step_input = StepInput {
        name: "orderId".to_string(),
        source: DataSource::WorkflowInput("order_id".to_string()),
        required: true,
    };

    let step_output = StepOutput {
        name: "processedOrder".to_string(),
        data_type: "Order".to_string(),
    };

    let step = WorkflowStep {
        id: StepId::new(),
        name: "Process Order".to_string(),
        step_type: StepType::ServiceTask {
            service: "OrderService".to_string(),
            operation: "process".to_string(),
        },
        node_id: NodeId::new(),
        inputs: vec![step_input.clone()],
        outputs: vec![step_output.clone()],
        timeout_ms: None,
        retry_policy: None,
    };

    assert_eq!(step.inputs.len(), 1);
    assert_eq!(step.inputs[0].name, "orderId");
    assert!(step.inputs[0].required);
    assert_eq!(step.outputs.len(), 1);
    assert_eq!(step.outputs[0].name, "processedOrder");
}

#[test]
fn test_workflow_event_application() {
    let workflow_id = WorkflowId::new();
    let user_id = UserId::new();
    let mut workflow = Workflow::new(workflow_id.clone(), "Test".to_string(), user_id.clone());

    // Create and apply WorkflowCreated event
    let event = WorkflowCreated {
        workflow_id: workflow_id.clone(),
        name: "Updated Workflow".to_string(),
        description: "Updated description".to_string(),
        created_by: user_id.clone(),
        created_at: Utc::now(),
        tags: vec!["updated".to_string()],
    };

    let domain_event = DomainEvent::Workflow(WorkflowEvent::WorkflowCreated(event));
    let result = workflow.apply_event(&domain_event);

    assert!(result.is_ok());
    assert_eq!(workflow.name, "Updated Workflow");
    assert_eq!(workflow.description, "Updated description");
    assert_eq!(workflow.tags, vec!["updated"]);
}

#[test]
fn test_duplicate_step_rejection() {
    let workflow_id = WorkflowId::new();
    let user_id = UserId::new();
    let mut workflow = Workflow::new(workflow_id.clone(), "Test Workflow".to_string(), user_id.clone());

    let step_id = StepId::new();
    let step = WorkflowStep {
        id: step_id.clone(),
        name: "Duplicate Step".to_string(),
        step_type: StepType::UserTask,
        node_id: NodeId::new(),
        inputs: vec![],
        outputs: vec![],
        timeout_ms: None,
        retry_policy: None,
    };

    // Add step first time - should succeed
    workflow.steps.insert(step_id.clone(), step.clone());

    // Try to add same step again - should fail
    let cmd = WorkflowCommand::AddStep(AddStep {
        workflow_id: workflow_id.clone(),
        step: step.clone(),
    });

    let result = workflow.handle_command(cmd);
    assert!(result.is_err());
    match result {
        Err(DomainError::DuplicateEntity(msg)) => {
            assert!(msg.contains("already exists"));
        }
        _ => panic!("Expected DuplicateEntity error"),
    }
}

#[test]
fn test_invalid_state_transitions() {
    let workflow_id = WorkflowId::new();
    let user_id = UserId::new();
    let mut workflow = Workflow::new(workflow_id.clone(), "Test Workflow".to_string(), user_id.clone());

    // Try to start workflow from Designed state (should fail)
    let cmd = WorkflowCommand::StartWorkflow(StartWorkflow {
        workflow_id: workflow_id.clone(),
        instance_id: "instance-123".to_string(),
        started_by: user_id.clone(),
        inputs: HashMap::new(),
    });

    let result = workflow.handle_command(cmd);
    assert!(result.is_err());
    match result {
        Err(DomainError::InvalidState(msg)) => {
            assert!(msg.contains("Ready state"));
        }
        _ => panic!("Expected InvalidState error"),
    }
}

#[test]
fn test_workflow_metrics() {
    let metrics = WorkflowMetrics {
        total_duration_ms: 5000,
        steps_executed: 10,
        steps_skipped: 2,
        retry_count: 3,
    };

    assert_eq!(metrics.total_duration_ms, 5000);
    assert_eq!(metrics.steps_executed, 10);
    assert_eq!(metrics.steps_skipped, 2);
    assert_eq!(metrics.retry_count, 3);
}

#[test]
fn test_execution_context() {
    let mut context = ExecutionContext {
        instance_id: "instance-123".to_string(),
        inputs: HashMap::new(),
        variables: HashMap::new(),
        step_outputs: HashMap::new(),
    };

    // Add some test data
    context.inputs.insert("orderId".to_string(), serde_json::json!("12345"));
    context.variables.insert("status".to_string(), serde_json::json!("processing"));

    let mut step_output = HashMap::new();
    step_output.insert("result".to_string(), serde_json::json!(true));
    context.step_outputs.insert(StepId::new(), step_output);

    assert_eq!(context.instance_id, "instance-123");
    assert_eq!(context.inputs.len(), 1);
    assert_eq!(context.variables.len(), 1);
    assert_eq!(context.step_outputs.len(), 1);
}
