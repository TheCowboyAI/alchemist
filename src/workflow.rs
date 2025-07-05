//! Workflow state management for the application

use bevy::prelude::*;

/// Resource that tracks the current workflow state
#[derive(Resource, Default)]
pub struct WorkflowState {
    current_step: Option<String>,
    steps: Vec<WorkflowStep>,
    ai_guidance: Vec<String>,
}

/// Represents a single step in a workflow
#[derive(Debug, Clone)]
pub struct WorkflowStep {
    /// Name of the workflow step
    pub name: String,
    /// Whether this step has been completed
    pub completed: bool,
}

impl WorkflowState {
    /// Returns the name of the current workflow step, if any
    #[must_use] pub fn current_step(&self) -> Option<&str> {
        self.current_step.as_deref()
    }

    /// Adds AI-generated guidance to the workflow
    pub fn add_ai_guidance(&mut self, guidance: String) {
        self.ai_guidance.push(guidance);
    }

    /// Adds a new workflow step
    pub fn add_step(&mut self, name: String) {
        self.steps.push(WorkflowStep {
            name,
            completed: false,
        });
    }

    /// Marks a step as completed
    pub fn complete_step(&mut self, name: &str) {
        if let Some(step) = self.steps.iter_mut().find(|s| s.name == name) {
            step.completed = true;
        }
    }

    /// Gets all workflow steps
    #[must_use] pub fn steps(&self) -> &[WorkflowStep] {
        &self.steps
    }

    /// Advances to the next incomplete step
    pub fn advance_to_next_step(&mut self) {
        if let Some(next_step) = self.steps.iter().find(|s| !s.completed) {
            self.current_step = Some(next_step.name.clone());
        } else {
            self.current_step = None; // All steps completed
        }
    }

    /// Gets the completion percentage of the workflow
    #[must_use] pub fn completion_percentage(&self) -> f32 {
        if self.steps.is_empty() {
            return 0.0;
        }

        let completed = self.steps.iter().filter(|s| s.completed).count() as f32;
        let total = self.steps.len() as f32;
        (completed / total) * 100.0
    }
}
