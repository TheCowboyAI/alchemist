//! Workflow Aggregate - Domain logic for workflow operations
//!
//! Implements a state machine for workflow execution with transactional guarantees.
//! Workflows represent business processes that can be designed, validated, and executed.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::domain::{
    commands::workflow::*,
    events::{
        DomainEvent,
        workflow::{
            WorkflowEvent, WorkflowCreated, StepAdded, StepsConnected, WorkflowValidated,
            WorkflowStarted, StepCompleted, WorkflowPaused, WorkflowResumed,
            WorkflowCompleted, WorkflowFailed, ValidationResult,
        },
    },
    value_objects::{WorkflowId, StepId, EdgeId, UserId, NodeId},
    DomainError,
};

/// Workflow state machine states
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowState {
    /// Initial state - workflow is being designed
    Designed,

    /// Workflow has been validated and is ready to run
    Ready {
        validated_at: DateTime<Utc>,
        validated_by: UserId,
    },

    /// Workflow is currently executing
    Running {
        started_at: DateTime<Utc>,
        current_step: StepId,
        completed_steps: Vec<StepId>,
    },

    /// Workflow execution is paused
    Paused {
        paused_at: DateTime<Utc>,
        paused_by: UserId,
        resume_point: StepId,
    },

    /// Workflow completed successfully
    Completed {
        completed_at: DateTime<Utc>,
        result: WorkflowResult,
    },

    /// Workflow failed during execution
    Failed {
        failed_at: DateTime<Utc>,
        error: String,
        failed_step: StepId,
        recovery_point: Option<StepId>,
    },
}

/// Result of a completed workflow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub outputs: HashMap<String, serde_json::Value>,
    pub metrics: WorkflowMetrics,
}

/// Metrics collected during workflow execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowMetrics {
    pub total_duration_ms: u64,
    pub steps_executed: usize,
    pub steps_skipped: usize,
    pub retry_count: usize,
}

/// A step in the workflow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: StepId,
    pub name: String,
    pub step_type: StepType,
    pub node_id: NodeId,
    pub inputs: Vec<StepInput>,
    pub outputs: Vec<StepOutput>,
    pub timeout_ms: Option<u64>,
    pub retry_policy: Option<RetryPolicy>,
}

/// Type of workflow step
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StepType {
    /// User task requiring manual action
    UserTask,

    /// Automated service task
    ServiceTask {
        service: String,
        operation: String,
    },

    /// Decision point with conditions
    Decision {
        conditions: Vec<DecisionCondition>,
    },

    /// Parallel execution gateway
    ParallelGateway {
        branches: Vec<StepId>,
    },

    /// Wait for external event
    EventWait {
        event_type: String,
        timeout_ms: u64,
    },

    /// Script execution
    Script {
        language: String,
        code: String,
    },
}

/// Input for a workflow step
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepInput {
    pub name: String,
    pub source: DataSource,
    pub required: bool,
}

/// Output from a workflow step
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepOutput {
    pub name: String,
    pub data_type: String,
}

/// Source of data for step inputs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataSource {
    /// Constant value
    Constant(serde_json::Value),

    /// Output from previous step
    StepOutput {
        step_id: StepId,
        output_name: String,
    },

    /// Workflow input parameter
    WorkflowInput(String),

    /// External data source
    External {
        source: String,
        query: String,
    },
}

/// Decision condition for branching
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionCondition {
    pub expression: String,
    pub target_step: StepId,
}

/// Retry policy for failed steps
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_ms: u64,
    pub backoff_multiplier: f32,
}

/// Workflow transition between steps
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowTransition {
    pub from_step: StepId,
    pub to_step: StepId,
    pub edge_id: EdgeId,
    pub condition: Option<String>,
}

/// Workflow aggregate root
#[derive(Debug, Clone)]
pub struct Workflow {
    // Identity
    pub id: WorkflowId,

    // State machine
    pub state: WorkflowState,

    // Structure
    pub name: String,
    pub description: String,
    pub graph_id: Option<NodeId>, // Reference to visual graph representation
    pub steps: HashMap<StepId, WorkflowStep>,
    pub transitions: Vec<WorkflowTransition>,
    pub start_step: Option<StepId>,
    pub end_steps: Vec<StepId>,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub created_by: UserId,
    pub version: u64,
    pub tags: Vec<String>,

    // Runtime data (only populated when running)
    pub execution_context: Option<ExecutionContext>,
}

/// Runtime execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub instance_id: String,
    pub inputs: HashMap<String, serde_json::Value>,
    pub variables: HashMap<String, serde_json::Value>,
    pub step_outputs: HashMap<StepId, HashMap<String, serde_json::Value>>,
}

impl Workflow {
    /// Create a new workflow in Designed state
    pub fn new(id: WorkflowId, name: String, created_by: UserId) -> Self {
        Self {
            id,
            state: WorkflowState::Designed,
            name,
            description: String::new(),
            graph_id: None,
            steps: HashMap::new(),
            transitions: Vec::new(),
            start_step: None,
            end_steps: Vec::new(),
            created_at: Utc::now(),
            created_by,
            version: 0,
            tags: Vec::new(),
            execution_context: None,
        }
    }

    /// Check if a state transition is valid
    pub fn can_transition(&self, to: &WorkflowState) -> bool {
        match (&self.state, to) {
            // From Designed
            (WorkflowState::Designed, WorkflowState::Ready { .. }) => true,

            // From Ready
            (WorkflowState::Ready { .. }, WorkflowState::Running { .. }) => true,
            (WorkflowState::Ready { .. }, WorkflowState::Designed) => true, // Allow going back to design

            // From Running
            (WorkflowState::Running { .. }, WorkflowState::Paused { .. }) => true,
            (WorkflowState::Running { .. }, WorkflowState::Completed { .. }) => true,
            (WorkflowState::Running { .. }, WorkflowState::Failed { .. }) => true,

            // From Paused
            (WorkflowState::Paused { .. }, WorkflowState::Running { .. }) => true,
            (WorkflowState::Paused { .. }, WorkflowState::Failed { .. }) => true,

            // From Failed (with recovery point)
            (WorkflowState::Failed { recovery_point: Some(_), .. }, WorkflowState::Running { .. }) => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    /// Handle workflow commands
    pub fn handle_command(&mut self, command: WorkflowCommand) -> Result<Vec<DomainEvent>, DomainError> {
        match command {
            WorkflowCommand::CreateWorkflow(cmd) => self.handle_create_workflow(cmd),
            WorkflowCommand::AddStep(cmd) => self.handle_add_step(cmd),
            WorkflowCommand::ConnectSteps(cmd) => self.handle_connect_steps(cmd),
            WorkflowCommand::ValidateWorkflow(cmd) => self.handle_validate_workflow(cmd),
            WorkflowCommand::StartWorkflow(cmd) => self.handle_start_workflow(cmd),
            WorkflowCommand::CompleteStep(cmd) => self.handle_complete_step(cmd),
            WorkflowCommand::PauseWorkflow(cmd) => self.handle_pause_workflow(cmd),
            WorkflowCommand::ResumeWorkflow(cmd) => self.handle_resume_workflow(cmd),
            WorkflowCommand::FailWorkflow(cmd) => self.handle_fail_workflow(cmd),
        }
    }

    /// Apply events to update state
    pub fn apply_event(&mut self, event: &DomainEvent) -> Result<(), DomainError> {
        match event {
            DomainEvent::Workflow(workflow_event) => match workflow_event {
                WorkflowEvent::WorkflowCreated(e) => self.apply_workflow_created(e),
                WorkflowEvent::StepAdded(e) => self.apply_step_added(e),
                WorkflowEvent::StepsConnected(e) => self.apply_steps_connected(e),
                WorkflowEvent::WorkflowValidated(e) => self.apply_workflow_validated(e),
                WorkflowEvent::WorkflowStarted(e) => self.apply_workflow_started(e),
                WorkflowEvent::StepCompleted(e) => self.apply_step_completed(e),
                WorkflowEvent::WorkflowPaused(e) => self.apply_workflow_paused(e),
                WorkflowEvent::WorkflowResumed(e) => self.apply_workflow_resumed(e),
                WorkflowEvent::WorkflowCompleted(e) => self.apply_workflow_completed(e),
                WorkflowEvent::WorkflowFailed(e) => self.apply_workflow_failed(e),
            },
            _ => Ok(()), // Ignore non-workflow events
        }
    }

    // Command handlers

    fn handle_create_workflow(&mut self, cmd: CreateWorkflow) -> Result<Vec<DomainEvent>, DomainError> {
        if self.version > 0 {
            return Err(DomainError::AggregateAlreadyExists);
        }

        let event = WorkflowCreated {
            workflow_id: cmd.workflow_id,
            name: cmd.name,
            description: cmd.description,
            created_by: cmd.created_by,
            created_at: Utc::now(),
            tags: cmd.tags,
        };

        self.apply_workflow_created(&event)?;

        Ok(vec![DomainEvent::Workflow(WorkflowEvent::WorkflowCreated(event))])
    }

    fn handle_add_step(&mut self, cmd: AddStep) -> Result<Vec<DomainEvent>, DomainError> {
        // Can only add steps in Designed state
        if !matches!(self.state, WorkflowState::Designed) {
            return Err(DomainError::InvalidState("Can only add steps to workflows in Designed state".into()));
        }

        // Check for duplicate step ID
        if self.steps.contains_key(&cmd.step.id) {
            return Err(DomainError::DuplicateEntity(format!("Step {} already exists", cmd.step.id)));
        }

        let event = StepAdded {
            workflow_id: self.id,
            step: cmd.step,
        };

        self.apply_step_added(&event)?;

        Ok(vec![DomainEvent::Workflow(WorkflowEvent::StepAdded(event))])
    }

    fn handle_connect_steps(&mut self, cmd: ConnectSteps) -> Result<Vec<DomainEvent>, DomainError> {
        // Can only connect steps in Designed state
        if !matches!(self.state, WorkflowState::Designed) {
            return Err(DomainError::InvalidState("Can only connect steps in Designed state".into()));
        }

        // Validate both steps exist
        if !self.steps.contains_key(&cmd.from_step) {
            return Err(DomainError::EntityNotFound(format!("Step {} not found", cmd.from_step)));
        }
        if !self.steps.contains_key(&cmd.to_step) {
            return Err(DomainError::EntityNotFound(format!("Step {} not found", cmd.to_step)));
        }

        // Check for duplicate transition
        let duplicate = self.transitions.iter().any(|t| {
            t.from_step == cmd.from_step && t.to_step == cmd.to_step
        });
        if duplicate {
            return Err(DomainError::DuplicateEntity("Transition already exists".into()));
        }

        let event = StepsConnected {
            workflow_id: self.id,
            from_step: cmd.from_step,
            to_step: cmd.to_step,
            edge_id: cmd.edge_id,
            condition: cmd.condition,
        };

        self.apply_steps_connected(&event)?;

        Ok(vec![DomainEvent::Workflow(WorkflowEvent::StepsConnected(event))])
    }

    fn handle_validate_workflow(&mut self, cmd: ValidateWorkflow) -> Result<Vec<DomainEvent>, DomainError> {
        // Can only validate from Designed state
        if !matches!(self.state, WorkflowState::Designed) {
            return Err(DomainError::InvalidState("Can only validate workflows in Designed state".into()));
        }

        // Validation rules
        if self.steps.is_empty() {
            return Err(DomainError::ValidationError("Workflow has no steps".into()));
        }

        if self.start_step.is_none() {
            return Err(DomainError::ValidationError("Workflow has no start step".into()));
        }

        if self.end_steps.is_empty() {
            return Err(DomainError::ValidationError("Workflow has no end steps".into()));
        }

        // TODO: Add more validation (reachability, cycles, etc.)

        let event = WorkflowValidated {
            workflow_id: self.id,
            validated_by: cmd.validated_by,
            validated_at: Utc::now(),
            validation_result: ValidationResult {
                is_valid: true,
                errors: vec![],
                warnings: vec![],
            },
        };

        self.apply_workflow_validated(&event)?;

        Ok(vec![DomainEvent::Workflow(WorkflowEvent::WorkflowValidated(event))])
    }

    fn handle_start_workflow(&mut self, cmd: StartWorkflow) -> Result<Vec<DomainEvent>, DomainError> {
        // Can only start from Ready state
        if !matches!(self.state, WorkflowState::Ready { .. }) {
            return Err(DomainError::InvalidState("Can only start workflows in Ready state".into()));
        }

        let start_step = self.start_step
            .ok_or_else(|| DomainError::ValidationError("No start step defined".into()))?;

        let event = WorkflowStarted {
            workflow_id: self.id,
            instance_id: cmd.instance_id,
            started_at: Utc::now(),
            started_by: cmd.started_by,
            initial_inputs: cmd.inputs,
            start_step,
        };

        self.apply_workflow_started(&event)?;

        Ok(vec![DomainEvent::Workflow(WorkflowEvent::WorkflowStarted(event))])
    }

    fn handle_complete_step(&mut self, cmd: CompleteStep) -> Result<Vec<DomainEvent>, DomainError> {
        // Can only complete steps when Running
        if !matches!(self.state, WorkflowState::Running { .. }) {
            return Err(DomainError::InvalidState("Can only complete steps when workflow is Running".into()));
        }

        // Verify step exists
        if !self.steps.contains_key(&cmd.step_id) {
            return Err(DomainError::EntityNotFound(format!("Step {} not found", cmd.step_id)));
        }

        // TODO: Verify this is the current step or a parallel step

        let event = StepCompleted {
            workflow_id: self.id,
            step_id: cmd.step_id,
            completed_at: Utc::now(),
            outputs: cmd.outputs,
            next_step: cmd.next_step,
        };

        self.apply_step_completed(&event)?;

        // Check if workflow is complete
        let mut events = vec![DomainEvent::Workflow(WorkflowEvent::StepCompleted(event.clone()))];

        if let WorkflowState::Running { completed_steps, .. } = &self.state {
            // Check if all paths lead to end steps and all are completed
            let all_complete = self.end_steps.iter().all(|end_step| {
                completed_steps.contains(end_step)
            });

            if all_complete {
                let completion_event = WorkflowCompleted {
                    workflow_id: self.id,
                    completed_at: Utc::now(),
                    result: WorkflowResult {
                        outputs: self.execution_context
                            .as_ref()
                            .map(|ctx| ctx.variables.clone())
                            .unwrap_or_default(),
                        metrics: WorkflowMetrics {
                            total_duration_ms: 0, // TODO: Calculate
                            steps_executed: completed_steps.len(),
                            steps_skipped: 0,
                            retry_count: 0,
                        },
                    },
                };

                self.apply_workflow_completed(&completion_event)?;
                events.push(DomainEvent::Workflow(WorkflowEvent::WorkflowCompleted(completion_event)));
            }
        }

        Ok(events)
    }

    fn handle_pause_workflow(&mut self, cmd: PauseWorkflow) -> Result<Vec<DomainEvent>, DomainError> {
        // Can only pause when Running
        let current_step = match &self.state {
            WorkflowState::Running { current_step, .. } => *current_step,
            _ => return Err(DomainError::InvalidState("Can only pause running workflows".into())),
        };

        let event = WorkflowPaused {
            workflow_id: self.id,
            paused_at: Utc::now(),
            paused_by: cmd.paused_by,
            reason: cmd.reason,
            resume_point: current_step,
        };

        self.apply_workflow_paused(&event)?;

        Ok(vec![DomainEvent::Workflow(WorkflowEvent::WorkflowPaused(event))])
    }

    fn handle_resume_workflow(&mut self, cmd: ResumeWorkflow) -> Result<Vec<DomainEvent>, DomainError> {
        // Can only resume when Paused
        let resume_point = match &self.state {
            WorkflowState::Paused { resume_point, .. } => *resume_point,
            _ => return Err(DomainError::InvalidState("Can only resume paused workflows".into())),
        };

        let event = WorkflowResumed {
            workflow_id: self.id,
            resumed_at: Utc::now(),
            resumed_by: cmd.resumed_by,
            resume_point,
        };

        self.apply_workflow_resumed(&event)?;

        Ok(vec![DomainEvent::Workflow(WorkflowEvent::WorkflowResumed(event))])
    }

    fn handle_fail_workflow(&mut self, cmd: FailWorkflow) -> Result<Vec<DomainEvent>, DomainError> {
        // Can fail from Running or Paused states
        let failed_step = match &self.state {
            WorkflowState::Running { current_step, .. } => *current_step,
            WorkflowState::Paused { resume_point, .. } => *resume_point,
            _ => return Err(DomainError::InvalidState("Can only fail running or paused workflows".into())),
        };

        let event = WorkflowFailed {
            workflow_id: self.id,
            failed_at: Utc::now(),
            error: cmd.error,
            failed_step,
            recovery_point: cmd.recovery_point,
        };

        self.apply_workflow_failed(&event)?;

        Ok(vec![DomainEvent::Workflow(WorkflowEvent::WorkflowFailed(event))])
    }

    // Event application methods

    fn apply_workflow_created(&mut self, event: &WorkflowCreated) -> Result<(), DomainError> {
        self.id = event.workflow_id;
        self.name = event.name.clone();
        self.description = event.description.clone();
        self.created_by = event.created_by;
        self.created_at = event.created_at;
        self.tags = event.tags.clone();
        self.version = 1;
        Ok(())
    }

    fn apply_step_added(&mut self, event: &StepAdded) -> Result<(), DomainError> {
        self.steps.insert(event.step.id, event.step.clone());

        // Set start step if this is the first step
        if self.steps.len() == 1 {
            self.start_step = Some(event.step.id);
        }

        self.version += 1;
        Ok(())
    }

    fn apply_steps_connected(&mut self, event: &StepsConnected) -> Result<(), DomainError> {
        self.transitions.push(WorkflowTransition {
            from_step: event.from_step,
            to_step: event.to_step,
            edge_id: event.edge_id,
            condition: event.condition.clone(),
        });

        self.version += 1;
        Ok(())
    }

    fn apply_workflow_validated(&mut self, event: &WorkflowValidated) -> Result<(), DomainError> {
        self.state = WorkflowState::Ready {
            validated_at: event.validated_at,
            validated_by: event.validated_by,
        };

        self.version += 1;
        Ok(())
    }

    fn apply_workflow_started(&mut self, event: &WorkflowStarted) -> Result<(), DomainError> {
        self.state = WorkflowState::Running {
            started_at: event.started_at,
            current_step: event.start_step,
            completed_steps: vec![],
        };

        self.execution_context = Some(ExecutionContext {
            instance_id: event.instance_id.clone(),
            inputs: event.initial_inputs.clone(),
            variables: HashMap::new(),
            step_outputs: HashMap::new(),
        });

        self.version += 1;
        Ok(())
    }

    fn apply_step_completed(&mut self, event: &StepCompleted) -> Result<(), DomainError> {
        if let WorkflowState::Running { current_step, completed_steps, started_at } = &mut self.state {
            // Add to completed steps
            completed_steps.push(event.step_id);

            // Update current step if next step is provided
            if let Some(next) = &event.next_step {
                *current_step = *next;
            }

            // Store step outputs
            if let Some(ctx) = &mut self.execution_context {
                ctx.step_outputs.insert(event.step_id, event.outputs.clone());
            }

            // Update state with new values
            self.state = WorkflowState::Running {
                started_at: *started_at,
                current_step: *current_step,
                completed_steps: completed_steps.clone(),
            };
        }

        self.version += 1;
        Ok(())
    }

    fn apply_workflow_paused(&mut self, event: &WorkflowPaused) -> Result<(), DomainError> {
        self.state = WorkflowState::Paused {
            paused_at: event.paused_at,
            paused_by: event.paused_by,
            resume_point: event.resume_point,
        };

        self.version += 1;
        Ok(())
    }

    fn apply_workflow_resumed(&mut self, event: &WorkflowResumed) -> Result<(), DomainError> {
        // Get completed steps from execution context
        let completed_steps = if let WorkflowState::Paused { .. } = &self.state {
            self.execution_context
                .as_ref()
                .map(|ctx| {
                    ctx.step_outputs.keys().cloned().collect()
                })
                .unwrap_or_default()
        } else {
            vec![]
        };

        self.state = WorkflowState::Running {
            started_at: event.resumed_at, // Use resumed time as new start
            current_step: event.resume_point,
            completed_steps,
        };

        self.version += 1;
        Ok(())
    }

    fn apply_workflow_completed(&mut self, event: &WorkflowCompleted) -> Result<(), DomainError> {
        self.state = WorkflowState::Completed {
            completed_at: event.completed_at,
            result: event.result.clone(),
        };

        self.version += 1;
        Ok(())
    }

    fn apply_workflow_failed(&mut self, event: &WorkflowFailed) -> Result<(), DomainError> {
        self.state = WorkflowState::Failed {
            failed_at: event.failed_at,
            error: event.error.clone(),
            failed_step: event.failed_step,
            recovery_point: event.recovery_point,
        };

        self.version += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_workflow() -> Workflow {
        Workflow::new(
            WorkflowId::new(),
            "Test Workflow".to_string(),
            UserId::new(),
        )
    }

    fn create_test_step(name: &str) -> WorkflowStep {
        WorkflowStep {
            id: StepId::new(),
            name: name.to_string(),
            step_type: StepType::UserTask,
            node_id: NodeId::new(),
            inputs: vec![],
            outputs: vec![],
            timeout_ms: None,
            retry_policy: None,
        }
    }

    #[test]
    fn test_workflow_creation() {
        let mut workflow = Workflow::new(WorkflowId::new(), "Test".to_string(), UserId::new());

        let cmd = CreateWorkflow {
            workflow_id: workflow.id.clone(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            created_by: UserId::new(),
            tags: vec!["test".to_string()],
        };

        let events = workflow.handle_command(WorkflowCommand::CreateWorkflow(cmd)).unwrap();

        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], DomainEvent::Workflow(WorkflowEvent::WorkflowCreated(_))));
        assert_eq!(workflow.version, 1);
    }

    #[test]
    fn test_state_transitions() {
        let workflow = create_test_workflow();

        // Valid transitions from Designed
        assert!(workflow.can_transition(&WorkflowState::Ready {
            validated_at: Utc::now(),
            validated_by: UserId::new(),
        }));

        // Invalid transitions from Designed
        assert!(!workflow.can_transition(&WorkflowState::Running {
            started_at: Utc::now(),
            current_step: StepId::new(),
            completed_steps: vec![],
        }));
    }

    #[test]
    fn test_add_step() {
        let mut workflow = create_test_workflow();
        workflow.version = 1; // Simulate created workflow

        let step = create_test_step("Step 1");
        let cmd = AddStep {
            workflow_id: workflow.id.clone(),
            step: step.clone(),
        };

        let events = workflow.handle_command(WorkflowCommand::AddStep(cmd)).unwrap();

        assert_eq!(events.len(), 1);
        assert!(workflow.steps.contains_key(&step.id));
        assert_eq!(workflow.start_step, Some(step.id));
    }

    #[test]
    fn test_workflow_validation() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        // Add steps
        let step1 = create_test_step("Step 1");
        let step2 = create_test_step("Step 2");

        workflow.steps.insert(step1.id.clone(), step1.clone());
        workflow.steps.insert(step2.id.clone(), step2.clone());
        workflow.start_step = Some(step1.id.clone());
        workflow.end_steps = vec![step2.id.clone()];

        let cmd = ValidateWorkflow {
            workflow_id: workflow.id.clone(),
            validated_by: UserId::new(),
        };

        let events = workflow.handle_command(WorkflowCommand::ValidateWorkflow(cmd)).unwrap();

        assert_eq!(events.len(), 1);
        assert!(matches!(workflow.state, WorkflowState::Ready { .. }));
    }

    // New comprehensive tests for better coverage

    #[test]
    fn test_duplicate_step_error() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        let step = create_test_step("Step 1");

        // Add step first time
        workflow.handle_command(WorkflowCommand::AddStep(AddStep {
            workflow_id: workflow.id.clone(),
            step: step.clone(),
        })).unwrap();

        // Try to add same step again
        let result = workflow.handle_command(WorkflowCommand::AddStep(AddStep {
            workflow_id: workflow.id.clone(),
            step: step.clone(),
        }));

        assert!(result.is_err());
        match result {
            Err(DomainError::DuplicateEntity(_)) => {},
            _ => panic!("Expected DuplicateEntity error"),
        }
    }

    #[test]
    fn test_connect_steps() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        // Add two steps
        let step1 = create_test_step("Step 1");
        let step2 = create_test_step("Step 2");

        workflow.handle_command(WorkflowCommand::AddStep(AddStep {
            workflow_id: workflow.id.clone(),
            step: step1.clone(),
        })).unwrap();

        workflow.handle_command(WorkflowCommand::AddStep(AddStep {
            workflow_id: workflow.id.clone(),
            step: step2.clone(),
        })).unwrap();

        // Connect steps
        let result = workflow.handle_command(WorkflowCommand::ConnectSteps(ConnectSteps {
            workflow_id: workflow.id.clone(),
            from_step: step1.id,
            to_step: step2.id,
            edge_id: EdgeId::new(),
            condition: Some("success".to_string()),
        }));

        assert!(result.is_ok());
        assert_eq!(workflow.transitions.len(), 1);
        assert_eq!(workflow.transitions[0].from_step, step1.id);
        assert_eq!(workflow.transitions[0].to_step, step2.id);
    }

    #[test]
    fn test_connect_nonexistent_steps_error() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        let step1 = create_test_step("Step 1");
        workflow.handle_command(WorkflowCommand::AddStep(AddStep {
            workflow_id: workflow.id.clone(),
            step: step1.clone(),
        })).unwrap();

        // Try to connect to non-existent step
        let result = workflow.handle_command(WorkflowCommand::ConnectSteps(ConnectSteps {
            workflow_id: workflow.id.clone(),
            from_step: step1.id,
            to_step: StepId::new(), // Non-existent
            edge_id: EdgeId::new(),
            condition: None,
        }));

        assert!(result.is_err());
        match result {
            Err(DomainError::EntityNotFound(_)) => {},
            _ => panic!("Expected EntityNotFound error"),
        }
    }

    #[test]
    fn test_duplicate_transition_error() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        // Add two steps
        let step1 = create_test_step("Step 1");
        let step2 = create_test_step("Step 2");

        workflow.handle_command(WorkflowCommand::AddStep(AddStep {
            workflow_id: workflow.id.clone(),
            step: step1.clone(),
        })).unwrap();

        workflow.handle_command(WorkflowCommand::AddStep(AddStep {
            workflow_id: workflow.id.clone(),
            step: step2.clone(),
        })).unwrap();

        // Connect steps first time
        workflow.handle_command(WorkflowCommand::ConnectSteps(ConnectSteps {
            workflow_id: workflow.id.clone(),
            from_step: step1.id,
            to_step: step2.id,
            edge_id: EdgeId::new(),
            condition: None,
        })).unwrap();

        // Try to connect same steps again
        let result = workflow.handle_command(WorkflowCommand::ConnectSteps(ConnectSteps {
            workflow_id: workflow.id.clone(),
            from_step: step1.id,
            to_step: step2.id,
            edge_id: EdgeId::new(),
            condition: None,
        }));

        assert!(result.is_err());
        match result {
            Err(DomainError::DuplicateEntity(_)) => {},
            _ => panic!("Expected DuplicateEntity error"),
        }
    }

    #[test]
    fn test_validate_empty_workflow_error() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        // Try to validate empty workflow
        let result = workflow.handle_command(WorkflowCommand::ValidateWorkflow(ValidateWorkflow {
            workflow_id: workflow.id.clone(),
            validated_by: UserId::new(),
        }));

        assert!(result.is_err());
        match result {
            Err(DomainError::ValidationError(_)) => {},
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_validate_workflow_no_start_step_error() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        // Add step but don't set start
        let step = create_test_step("Step 1");
        workflow.steps.insert(step.id, step);
        workflow.start_step = None; // Explicitly no start

        let result = workflow.handle_command(WorkflowCommand::ValidateWorkflow(ValidateWorkflow {
            workflow_id: workflow.id.clone(),
            validated_by: UserId::new(),
        }));

        assert!(result.is_err());
        match result {
            Err(DomainError::ValidationError(msg)) if msg.contains("no start step") => {},
            _ => panic!("Expected ValidationError about no start step"),
        }
    }

    #[test]
    fn test_validate_workflow_no_end_steps_error() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        // Add step with start but no end
        let step = create_test_step("Step 1");
        workflow.steps.insert(step.id, step.clone());
        workflow.start_step = Some(step.id);
        workflow.end_steps = vec![]; // No end steps

        let result = workflow.handle_command(WorkflowCommand::ValidateWorkflow(ValidateWorkflow {
            workflow_id: workflow.id.clone(),
            validated_by: UserId::new(),
        }));

        assert!(result.is_err());
        match result {
            Err(DomainError::ValidationError(msg)) if msg.contains("no end steps") => {},
            _ => panic!("Expected ValidationError about no end steps"),
        }
    }

    #[test]
    fn test_start_workflow() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        // Setup valid workflow
        let step = create_test_step("Step 1");
        workflow.steps.insert(step.id, step.clone());
        workflow.start_step = Some(step.id);
        workflow.end_steps = vec![step.id];
        workflow.state = WorkflowState::Ready {
            validated_at: Utc::now(),
            validated_by: UserId::new(),
        };

        // Start workflow
        let result = workflow.handle_command(WorkflowCommand::StartWorkflow(StartWorkflow {
            workflow_id: workflow.id.clone(),
            instance_id: "instance-1".to_string(),
            started_by: UserId::new(),
            inputs: HashMap::new(),
        }));

        assert!(result.is_ok());
        assert!(matches!(workflow.state, WorkflowState::Running { .. }));
        assert!(workflow.execution_context.is_some());
    }

    #[test]
    fn test_start_workflow_invalid_state_error() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        // Try to start from Designed state (not Ready)
        let result = workflow.handle_command(WorkflowCommand::StartWorkflow(StartWorkflow {
            workflow_id: workflow.id.clone(),
            instance_id: "instance-1".to_string(),
            started_by: UserId::new(),
            inputs: HashMap::new(),
        }));

        assert!(result.is_err());
        match result {
            Err(DomainError::InvalidState(_)) => {},
            _ => panic!("Expected InvalidState error"),
        }
    }

    #[test]
    fn test_complete_step() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        // Setup running workflow
        let step1 = create_test_step("Step 1");
        let step2 = create_test_step("Step 2");

        workflow.steps.insert(step1.id, step1.clone());
        workflow.steps.insert(step2.id, step2.clone());
        workflow.start_step = Some(step1.id);
        workflow.end_steps = vec![step2.id];

        workflow.state = WorkflowState::Running {
            started_at: Utc::now(),
            current_step: step1.id,
            completed_steps: vec![],
        };

        workflow.execution_context = Some(ExecutionContext {
            instance_id: "instance-1".to_string(),
            inputs: HashMap::new(),
            variables: HashMap::new(),
            step_outputs: HashMap::new(),
        });

        // Complete step
        let result = workflow.handle_command(WorkflowCommand::CompleteStep(CompleteStep {
            workflow_id: workflow.id.clone(),
            step_id: step1.id,
            outputs: HashMap::new(),
            next_step: Some(step2.id),
        }));

        assert!(result.is_ok());

        // Check state updated
        if let WorkflowState::Running { current_step, completed_steps, .. } = &workflow.state {
            assert_eq!(*current_step, step2.id);
            assert!(completed_steps.contains(&step1.id));
        } else {
            panic!("Expected Running state");
        }
    }

    #[test]
    fn test_complete_nonexistent_step_error() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        workflow.state = WorkflowState::Running {
            started_at: Utc::now(),
            current_step: StepId::new(),
            completed_steps: vec![],
        };

        // Try to complete non-existent step
        let result = workflow.handle_command(WorkflowCommand::CompleteStep(CompleteStep {
            workflow_id: workflow.id.clone(),
            step_id: StepId::new(), // Non-existent
            outputs: HashMap::new(),
            next_step: None,
        }));

        assert!(result.is_err());
        match result {
            Err(DomainError::EntityNotFound(_)) => {},
            _ => panic!("Expected EntityNotFound error"),
        }
    }

    #[test]
    fn test_workflow_completion() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        // Setup single-step workflow
        let step = create_test_step("Step 1");
        workflow.steps.insert(step.id, step.clone());
        workflow.start_step = Some(step.id);
        workflow.end_steps = vec![step.id];

        workflow.state = WorkflowState::Running {
            started_at: Utc::now(),
            current_step: step.id,
            completed_steps: vec![],
        };

        workflow.execution_context = Some(ExecutionContext {
            instance_id: "instance-1".to_string(),
            inputs: HashMap::new(),
            variables: HashMap::new(),
            step_outputs: HashMap::new(),
        });

        // Complete the only step
        let result = workflow.handle_command(WorkflowCommand::CompleteStep(CompleteStep {
            workflow_id: workflow.id.clone(),
            step_id: step.id,
            outputs: HashMap::new(),
            next_step: None,
        }));

        assert!(result.is_ok());

        // Should generate two events: StepCompleted and WorkflowCompleted
        let events = result.unwrap();
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], DomainEvent::Workflow(WorkflowEvent::StepCompleted(_))));
        assert!(matches!(events[1], DomainEvent::Workflow(WorkflowEvent::WorkflowCompleted(_))));

        // Check final state
        assert!(matches!(workflow.state, WorkflowState::Completed { .. }));
    }

    #[test]
    fn test_pause_workflow() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        let step = StepId::new();
        workflow.state = WorkflowState::Running {
            started_at: Utc::now(),
            current_step: step,
            completed_steps: vec![],
        };

        // Pause workflow
        let result = workflow.handle_command(WorkflowCommand::PauseWorkflow(PauseWorkflow {
            workflow_id: workflow.id.clone(),
            paused_by: UserId::new(),
            reason: "Manual pause".to_string(),
        }));

        assert!(result.is_ok());

        if let WorkflowState::Paused { resume_point, .. } = &workflow.state {
            assert_eq!(*resume_point, step);
        } else {
            panic!("Expected Paused state");
        }
    }

    #[test]
    fn test_pause_non_running_workflow_error() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        // Try to pause from Designed state
        let result = workflow.handle_command(WorkflowCommand::PauseWorkflow(PauseWorkflow {
            workflow_id: workflow.id.clone(),
            paused_by: UserId::new(),
            reason: "Test".to_string(),
        }));

        assert!(result.is_err());
        match result {
            Err(DomainError::InvalidState(_)) => {},
            _ => panic!("Expected InvalidState error"),
        }
    }

    #[test]
    fn test_resume_workflow() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        let step = StepId::new();
        workflow.state = WorkflowState::Paused {
            paused_at: Utc::now(),
            paused_by: UserId::new(),
            resume_point: step,
        };

        workflow.execution_context = Some(ExecutionContext {
            instance_id: "instance-1".to_string(),
            inputs: HashMap::new(),
            variables: HashMap::new(),
            step_outputs: HashMap::new(),
        });

        // Resume workflow
        let result = workflow.handle_command(WorkflowCommand::ResumeWorkflow(ResumeWorkflow {
            workflow_id: workflow.id.clone(),
            resumed_by: UserId::new(),
        }));

        assert!(result.is_ok());

        if let WorkflowState::Running { current_step, .. } = &workflow.state {
            assert_eq!(*current_step, step);
        } else {
            panic!("Expected Running state");
        }
    }

    #[test]
    fn test_fail_workflow() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        let step = StepId::new();
        workflow.state = WorkflowState::Running {
            started_at: Utc::now(),
            current_step: step,
            completed_steps: vec![],
        };

        // Fail workflow
        let result = workflow.handle_command(WorkflowCommand::FailWorkflow(FailWorkflow {
            workflow_id: workflow.id.clone(),
            error: "Test error".to_string(),
            recovery_point: Some(step),
        }));

        assert!(result.is_ok());

        if let WorkflowState::Failed { failed_step, error, .. } = &workflow.state {
            assert_eq!(*failed_step, step);
            assert_eq!(error, "Test error");
        } else {
            panic!("Expected Failed state");
        }
    }

    #[test]
    fn test_fail_paused_workflow() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        let step = StepId::new();
        workflow.state = WorkflowState::Paused {
            paused_at: Utc::now(),
            paused_by: UserId::new(),
            resume_point: step,
        };

        // Fail paused workflow
        let result = workflow.handle_command(WorkflowCommand::FailWorkflow(FailWorkflow {
            workflow_id: workflow.id.clone(),
            error: "Timeout".to_string(),
            recovery_point: None,
        }));

        assert!(result.is_ok());
        assert!(matches!(workflow.state, WorkflowState::Failed { .. }));
    }

    #[test]
    fn test_invalid_state_operations() {
        let mut workflow = create_test_workflow();
        workflow.version = 1;

        // Set to completed state
        workflow.state = WorkflowState::Completed {
            completed_at: Utc::now(),
            result: WorkflowResult {
                outputs: HashMap::new(),
                metrics: WorkflowMetrics {
                    total_duration_ms: 1000,
                    steps_executed: 1,
                    steps_skipped: 0,
                    retry_count: 0,
                },
            },
        };

        // Try to add step to completed workflow
        let result = workflow.handle_command(WorkflowCommand::AddStep(AddStep {
            workflow_id: workflow.id.clone(),
            step: create_test_step("New Step"),
        }));

        assert!(result.is_err());
        match result {
            Err(DomainError::InvalidState(_)) => {},
            _ => panic!("Expected InvalidState error"),
        }
    }

    #[test]
    fn test_step_types() {
        // Test different step type creation
        let user_task = WorkflowStep {
            id: StepId::new(),
            name: "User Task".to_string(),
            step_type: StepType::UserTask,
            node_id: NodeId::new(),
            inputs: vec![],
            outputs: vec![],
            timeout_ms: None,
            retry_policy: None,
        };

        let service_task = WorkflowStep {
            id: StepId::new(),
            name: "Service Task".to_string(),
            step_type: StepType::ServiceTask {
                service: "EmailService".to_string(),
                operation: "sendEmail".to_string(),
            },
            node_id: NodeId::new(),
            inputs: vec![],
            outputs: vec![],
            timeout_ms: Some(5000),
            retry_policy: Some(RetryPolicy {
                max_attempts: 3,
                backoff_ms: 1000,
                backoff_multiplier: 2.0,
            }),
        };

        let decision = WorkflowStep {
            id: StepId::new(),
            name: "Decision".to_string(),
            step_type: StepType::Decision {
                conditions: vec![
                    DecisionCondition {
                        expression: "amount > 1000".to_string(),
                        target_step: StepId::new(),
                    },
                ],
            },
            node_id: NodeId::new(),
            inputs: vec![],
            outputs: vec![],
            timeout_ms: None,
            retry_policy: None,
        };

        // Verify they can be created
        assert!(matches!(user_task.step_type, StepType::UserTask));
        assert!(matches!(service_task.step_type, StepType::ServiceTask { .. }));
        assert!(matches!(decision.step_type, StepType::Decision { .. }));
    }
}
