//! Workflow state management for the application

use bevy::prelude::*;

/// Resource that tracks the current workflow state
#[derive(Resource, Default)]
pub struct WorkflowState {
    current_step: Option<String>,
    steps: Vec<WorkflowStep>,
    ai_guidance: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WorkflowStep {
    pub name: String,
    pub completed: bool,
}

impl WorkflowState {
    pub fn current_step(&self) -> Option<&str> {
        self.current_step.as_deref()
    }

    pub fn add_ai_guidance(&mut self, guidance: String) {
        self.ai_guidance.push(guidance);
    }
} 