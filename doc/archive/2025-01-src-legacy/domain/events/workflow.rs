//! Workflow domain events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::{
    aggregates::workflow::{WorkflowResult, WorkflowStep},
    value_objects::{EdgeId, StepId, UserId, WorkflowId},
};

/// All workflow-related events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowEvent {
    WorkflowCreated(WorkflowCreated),
    StepAdded(StepAdded),
    StepsConnected(StepsConnected),
    WorkflowValidated(WorkflowValidated),
    WorkflowStarted(WorkflowStarted),
    StepCompleted(StepCompleted),
    WorkflowPaused(WorkflowPaused),
    WorkflowResumed(WorkflowResumed),
    WorkflowCompleted(WorkflowCompleted),
    WorkflowFailed(WorkflowFailed),
}

/// Workflow was created
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowCreated {
    pub workflow_id: WorkflowId,
    pub name: String,
    pub description: String,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

/// Step was added to workflow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepAdded {
    pub workflow_id: WorkflowId,
    pub step: WorkflowStep,
}

/// Steps were connected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepsConnected {
    pub workflow_id: WorkflowId,
    pub from_step: StepId,
    pub to_step: StepId,
    pub edge_id: EdgeId,
    pub condition: Option<String>,
}

/// Workflow validation result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Workflow was validated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowValidated {
    pub workflow_id: WorkflowId,
    pub validated_by: UserId,
    pub validated_at: DateTime<Utc>,
    pub validation_result: ValidationResult,
}

/// Workflow execution started
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowStarted {
    pub workflow_id: WorkflowId,
    pub instance_id: String,
    pub started_at: DateTime<Utc>,
    pub started_by: UserId,
    pub initial_inputs: HashMap<String, serde_json::Value>,
    pub start_step: StepId,
}

/// Step completed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepCompleted {
    pub workflow_id: WorkflowId,
    pub step_id: StepId,
    pub completed_at: DateTime<Utc>,
    pub outputs: HashMap<String, serde_json::Value>,
    pub next_step: Option<StepId>,
}

/// Workflow was paused
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowPaused {
    pub workflow_id: WorkflowId,
    pub paused_at: DateTime<Utc>,
    pub paused_by: UserId,
    pub reason: String,
    pub resume_point: StepId,
}

/// Workflow was resumed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowResumed {
    pub workflow_id: WorkflowId,
    pub resumed_at: DateTime<Utc>,
    pub resumed_by: UserId,
    pub resume_point: StepId,
}

/// Workflow completed successfully
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowCompleted {
    pub workflow_id: WorkflowId,
    pub completed_at: DateTime<Utc>,
    pub result: WorkflowResult,
}

/// Workflow failed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowFailed {
    pub workflow_id: WorkflowId,
    pub failed_at: DateTime<Utc>,
    pub error: String,
    pub failed_step: StepId,
    pub recovery_point: Option<StepId>,
}
