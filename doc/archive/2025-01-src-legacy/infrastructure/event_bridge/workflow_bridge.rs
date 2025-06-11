//! Workflow event bridge
//!
//! Translates workflow domain events to presentation events

use bevy::prelude::*;
use crossbeam::channel::{Receiver, Sender};
use std::sync::Arc;

use cim_domain::workflow::{WorkflowEvent as DomainWorkflowEvent, WorkflowState};
use crate::domain::value_objects::{WorkflowId, StepId};
use crate::presentation::events::WorkflowEvent as PresentationWorkflowEvent;
use crate::infrastructure::event_bridge::{BridgeEvent, BridgeCommand};

/// Bridge for workflow events between domain and presentation
pub struct WorkflowEventBridge {
    /// Receiver for domain events
    domain_receiver: Receiver<BridgeEvent>,

    /// Sender for presentation events
    presentation_sender: Sender<BridgeCommand>,
}

impl WorkflowEventBridge {
    /// Create a new workflow event bridge
    pub fn new(
        domain_receiver: Receiver<BridgeEvent>,
        presentation_sender: Sender<BridgeCommand>,
    ) -> Self {
        Self {
            domain_receiver,
            presentation_sender,
        }
    }

    /// Process workflow events from domain to presentation
    pub fn process_events(&self, mut events: EventWriter<PresentationWorkflowEvent>) {
        // Process all pending domain events
        while let Ok(bridge_event) = self.domain_receiver.try_recv() {
            match bridge_event {
                BridgeEvent::WorkflowStarted { workflow_id, instance_id } => {
                    events.send(PresentationWorkflowEvent::WorkflowStarted {
                        workflow_id,
                        instance_id,
                    });
                }
                BridgeEvent::WorkflowStepStarted { step_id } => {
                    events.send(PresentationWorkflowEvent::StepStarted {
                        step_id,
                    });
                }
                BridgeEvent::WorkflowStepCompleted { step_id, duration } => {
                    events.send(PresentationWorkflowEvent::StepCompleted {
                        step_id,
                        duration,
                    });
                }
                BridgeEvent::WorkflowStepFailed { step_id, error } => {
                    events.send(PresentationWorkflowEvent::StepFailed {
                        step_id,
                        error,
                    });
                }
                BridgeEvent::WorkflowCompleted { workflow_id, instance_id, total_duration } => {
                    events.send(PresentationWorkflowEvent::WorkflowCompleted {
                        workflow_id,
                        instance_id,
                        total_duration,
                    });
                }
                _ => {
                    // Other event types not related to workflows
                }
            }
        }
    }
}

/// Plugin for workflow event bridging
pub struct WorkflowEventBridgePlugin {
    pub bridge: Arc<WorkflowEventBridge>,
}

impl Plugin for WorkflowEventBridgePlugin {
    fn build(&self, app: &mut App) {
        let bridge = self.bridge.clone();

        app.add_systems(Update, move |events: EventWriter<PresentationWorkflowEvent>| {
            bridge.process_events(events);
        });
    }
}

/// Convert domain workflow events to bridge events
pub fn convert_domain_workflow_event<S, I, O>(
    event: DomainWorkflowEvent<S, I, O>,
) -> Option<BridgeEvent>
where
    S: WorkflowState,
    I: cim_domain::workflow::TransitionInput,
    O: cim_domain::workflow::TransitionOutput,
{
    match event {
        DomainWorkflowEvent::WorkflowStarted { workflow_id, .. } => {
            Some(BridgeEvent::WorkflowStarted {
                workflow_id: convert_workflow_id(workflow_id),
                instance_id: uuid::Uuid::new_v4(),
            })
        }
        DomainWorkflowEvent::TransitionExecuted { .. } => {
            // Could map to step events if needed
            None
        }
        DomainWorkflowEvent::WorkflowCompleted { workflow_id, duration } => {
            Some(BridgeEvent::WorkflowCompleted {
                workflow_id: convert_workflow_id(workflow_id),
                instance_id: uuid::Uuid::new_v4(),
                total_duration: duration.as_secs_f32(),
            })
        }
        DomainWorkflowEvent::WorkflowSuspended { .. } => {
            // Could add suspended event to presentation
            None
        }
        DomainWorkflowEvent::WorkflowResumed { .. } => {
            // Could add resumed event to presentation
            None
        }
        DomainWorkflowEvent::WorkflowCancelled { .. } => {
            // Could add cancelled event to presentation
            None
        }
        DomainWorkflowEvent::WorkflowFailed { .. } => {
            // Could add failed event to presentation
            None
        }
        _ => None,
    }
}

/// Convert domain WorkflowId to presentation WorkflowId
fn convert_workflow_id(domain_id: cim_domain::identifiers::WorkflowId) -> WorkflowId {
    // Assuming both use UUID internally
    WorkflowId::from(domain_id.to_string().parse::<uuid::Uuid>().unwrap())
}

/// Convert domain StepId to presentation StepId
fn convert_step_id(domain_id: cim_domain::identifiers::StateId) -> StepId {
    // Convert StateId to StepId
    StepId::from(domain_id.to_string())
}
