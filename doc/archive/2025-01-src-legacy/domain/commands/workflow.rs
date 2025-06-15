//! Workflow commands for the workflow aggregate

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::{
    aggregates::workflow::WorkflowStep,
    value_objects::{EdgeId, StepId, UserId, WorkflowId},
};

/// Commands that can be sent to a workflow aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowCommand {
    CreateWorkflow(CreateWorkflow),
    AddStep(AddStep),
    ConnectSteps(ConnectSteps),
    ValidateWorkflow(ValidateWorkflow),
    StartWorkflow(StartWorkflow),
    CompleteStep(CompleteStep),
    PauseWorkflow(PauseWorkflow),
    ResumeWorkflow(ResumeWorkflow),
    FailWorkflow(FailWorkflow),
}

/// Create a new workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkflow {
    pub workflow_id: WorkflowId,
    pub name: String,
    pub description: String,
    pub created_by: UserId,
    pub tags: Vec<String>,
}

/// Add a step to the workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddStep {
    pub workflow_id: WorkflowId,
    pub step: WorkflowStep,
}

/// Connect two steps in the workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectSteps {
    pub workflow_id: WorkflowId,
    pub from_step: StepId,
    pub to_step: StepId,
    pub edge_id: EdgeId,
    pub condition: Option<String>,
}

/// Validate the workflow structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateWorkflow {
    pub workflow_id: WorkflowId,
    pub validated_by: UserId,
}

/// Start workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWorkflow {
    pub workflow_id: WorkflowId,
    pub instance_id: String,
    pub started_by: UserId,
    pub inputs: HashMap<String, serde_json::Value>,
}

/// Mark a step as completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteStep {
    pub workflow_id: WorkflowId,
    pub step_id: StepId,
    pub outputs: HashMap<String, serde_json::Value>,
    pub next_step: Option<StepId>,
}

/// Pause workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PauseWorkflow {
    pub workflow_id: WorkflowId,
    pub paused_by: UserId,
    pub reason: String,
}

/// Resume a paused workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeWorkflow {
    pub workflow_id: WorkflowId,
    pub resumed_by: UserId,
}

/// Mark workflow as failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailWorkflow {
    pub workflow_id: WorkflowId,
    pub error: String,
    pub recovery_point: Option<StepId>,
}
