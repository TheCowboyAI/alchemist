//! Workflow content type for CIM-IPLD

use cim_ipld::{ContentType, TypedContent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Content representing a workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContent {
    /// Unique identifier for the workflow
    pub id: String,
    /// Workflow name
    pub name: String,
    /// Workflow description
    pub description: String,
    /// Workflow steps
    pub steps: Vec<WorkflowStep>,
    /// Transitions between steps
    pub transitions: Vec<WorkflowTransition>,
    /// Workflow metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A step in the workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Step ID
    pub id: String,
    /// Step name
    pub name: String,
    /// Step type
    pub step_type: StepType,
    /// Step configuration
    pub config: HashMap<String, serde_json::Value>,
    /// Position in visual representation
    pub position: Option<(f64, f64)>,
}

/// Types of workflow steps
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepType {
    /// Start of workflow
    Start,
    /// End of workflow
    End,
    /// Process/action step
    Process,
    /// Decision point
    Decision,
    /// Parallel execution
    Parallel,
    /// Join parallel branches
    Join,
    /// External integration
    Integration,
    /// Custom step type
    Custom(String),
}

/// Transition between workflow steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTransition {
    /// Source step ID
    pub from: String,
    /// Target step ID
    pub to: String,
    /// Condition for transition (optional)
    pub condition: Option<String>,
    /// Transition metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TypedContent for WorkflowContent {
    const CODEC: u64 = 0x300104;
    const CONTENT_TYPE: ContentType = ContentType::Custom(0x300104);
}

impl WorkflowContent {
    /// Create a new workflow
    pub fn new(id: String, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
            steps: Vec::new(),
            transitions: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a step to the workflow
    pub fn add_step(&mut self, step: WorkflowStep) {
        self.steps.push(step);
    }

    /// Add a transition
    pub fn add_transition(&mut self, transition: WorkflowTransition) {
        self.transitions.push(transition);
    }

    /// Get start steps
    pub fn get_start_steps(&self) -> Vec<&WorkflowStep> {
        self.steps
            .iter()
            .filter(|s| matches!(s.step_type, StepType::Start))
            .collect()
    }

    /// Get next steps from a given step
    pub fn get_next_steps(&self, step_id: &str) -> Vec<&WorkflowStep> {
        let next_ids: Vec<&str> = self
            .transitions
            .iter()
            .filter(|t| t.from == step_id)
            .map(|t| t.to.as_str())
            .collect();

        self.steps
            .iter()
            .filter(|s| next_ids.contains(&s.id.as_str()))
            .collect()
    }

    /// Validate workflow structure
    pub fn validate(&self) -> Result<(), String> {
        // Check for at least one start
        if self.get_start_steps().is_empty() {
            return Err("Workflow must have at least one start step".to_string());
        }

        // Check all transitions reference valid steps
        let step_ids: Vec<&str> = self.steps.iter().map(|s| s.id.as_str()).collect();
        for transition in &self.transitions {
            if !step_ids.contains(&transition.from.as_str()) {
                return Err(format!(
                    "Transition references unknown source step: {}",
                    transition.from
                ));
            }
            if !step_ids.contains(&transition.to.as_str()) {
                return Err(format!(
                    "Transition references unknown target step: {}",
                    transition.to
                ));
            }
        }

        Ok(())
    }
}
