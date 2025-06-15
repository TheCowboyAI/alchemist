//! Presentation-layer events that stay within Bevy
//!
//! These events are ephemeral, UI-specific, and NEVER sent to NATS.
//! They represent visual state changes, animations, and user interactions
//! that don't constitute business-meaningful state changes.

pub mod animation;
pub mod interaction;
pub mod layout;

pub use animation::*;
pub use interaction::*;
pub use layout::*;

use crate::domain::commands::graph_commands::GraphCommand;
use crate::domain::events::DomainEvent;
use crate::domain::value_objects::{StepId, WorkflowId};
use bevy::prelude::*;

/// Marker trait for presentation events
pub trait PresentationEvent: Event + Clone + Send + Sync + 'static {
    /// Whether this event should be aggregated before domain conversion
    fn requires_aggregation(&self) -> bool {
        true
    }
}

/// Event fired when multiple presentation events should be aggregated
/// into a single domain command
#[derive(Event, Clone, Debug)]
pub struct AggregationComplete {
    pub aggregation_type: AggregationType,
    pub entity_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AggregationType {
    DragOperation,
    LayoutCalculation,
    BatchSelection,
    AnimationSequence,
}

/// Event for import results that need to be processed
#[derive(Event, Debug, Clone)]
pub struct ImportResultEvent {
    pub event: DomainEvent,
}

/// Event for import requests that need to be processed
#[derive(Event, Debug, Clone)]
pub struct ImportRequestEvent {
    pub event: DomainEvent,
}

/// Wrapper for domain commands to be sent through Bevy's event system
#[derive(Event, Debug, Clone)]
pub struct PresentationCommand {
    pub command: GraphCommand,
}

impl PresentationCommand {
    pub fn new(command: GraphCommand) -> Self {
        Self { command }
    }
}

/// Events related to workflow execution and visualization
#[derive(Event, Debug, Clone)]
pub enum WorkflowEvent {
    /// Workflow instance started
    WorkflowStarted {
        workflow_id: WorkflowId,
        instance_id: uuid::Uuid,
    },

    /// Workflow step started execution
    StepStarted { step_id: StepId },

    /// Workflow step completed
    StepCompleted { step_id: StepId, duration: f32 },

    /// Workflow step failed
    StepFailed { step_id: StepId, error: String },

    /// Workflow instance completed
    WorkflowCompleted {
        workflow_id: WorkflowId,
        instance_id: uuid::Uuid,
        total_duration: f32,
    },
}
