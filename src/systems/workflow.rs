//! Workflow systems

use bevy::prelude::*;
use std::time::SystemTime;

/// Start workflow system
pub fn start_workflow_system(
    _commands: Commands,
    mut workflow_state: ResMut<crate::workflow::WorkflowState>,
    mut start_events: EventReader<StartWorkflowEvent>,
) {
    for event in start_events.read() {
        // Clear previous workflow by resetting state
        *workflow_state = crate::workflow::WorkflowState::default();

        // Add steps from the event
        for step_name in &event.steps {
            workflow_state.add_step(step_name.clone());
        }

        // Advance to the first step
        workflow_state.advance_to_next_step();

        info!("Started workflow with {} steps", event.steps.len());
    }
}

/// Process workflow steps system
pub fn process_workflow_steps_system(
    mut workflow_state: ResMut<crate::workflow::WorkflowState>,
    mut step_events: EventReader<CompleteStepEvent>,
) {
    for event in step_events.read() {
        // Complete the current step
        workflow_state.complete_step(&event.step_name);

        // Advance to next step
        workflow_state.advance_to_next_step();

        // Log progress
        let progress = workflow_state.completion_percentage();
        info!("Workflow progress: {:.1}%", progress);
    }
}

/// Complete workflow system
pub fn complete_workflow_system(
    workflow_state: Res<crate::workflow::WorkflowState>,
    mut complete_events: EventWriter<WorkflowCompletedEvent>,
) {
    // Check if all steps are completed
    let all_completed = workflow_state.steps().iter().all(|s| s.completed);

    if all_completed && !workflow_state.steps().is_empty() {
        // Workflow is already completed, advance_to_next_step will set current_step to None

        // Emit completion event
        complete_events.write(WorkflowCompletedEvent {
            completed_at: SystemTime::now(),
            total_steps: workflow_state.steps().len(),
        });

        info!("Workflow completed!");
    }
}

/// Handle workflow timeouts system
pub fn handle_workflow_timeouts_system(
    time: Res<Time>,
    mut workflow_state: ResMut<crate::workflow::WorkflowState>,
    mut timeout_events: EventWriter<WorkflowTimeoutEvent>,
) {
    // Check for workflow timeout (example: 30 minutes)
    let timeout_duration = 30.0 * 60.0; // 30 minutes in seconds

    // This is a simplified timeout check
    // In production, you'd track when the workflow started
    if workflow_state.current_step().is_some() {
        // For now, we'll use the time resource to track elapsed time
        // In a real implementation, you'd store the workflow start time
        let elapsed = time.elapsed_secs();

        if elapsed > timeout_duration {
            timeout_events.write(WorkflowTimeoutEvent {
                timed_out_at: SystemTime::now(),
                current_step: workflow_state.current_step().map(std::string::ToString::to_string),
            });

            // Clear the workflow by resetting to default
            *workflow_state = crate::workflow::WorkflowState::default();

            warn!("Workflow timed out!");
        }
    }
}

// Event types for workflow systems
/// Event to start a workflow
#[derive(Event)]
pub struct StartWorkflowEvent {
    /// Steps in the workflow
    pub steps: Vec<String>,
}

/// Event when a step is completed
#[derive(Event)]
pub struct CompleteStepEvent {
    /// Name of the completed step
    pub step_name: String,
}

/// Event when workflow is completed
#[derive(Event)]
pub struct WorkflowCompletedEvent {
    /// When the workflow completed
    pub completed_at: SystemTime,
    /// Total number of steps
    pub total_steps: usize,
}

/// Event when workflow times out
#[derive(Event)]
pub struct WorkflowTimeoutEvent {
    /// When the timeout occurred
    pub timed_out_at: SystemTime,
    /// Current step when timeout occurred
    pub current_step: Option<String>,
}
