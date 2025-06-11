//! Workflow visualization components
//!
//! Components for visualizing workflow execution, steps, and transitions

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::value_objects::{WorkflowId, StepId};

/// Visual representation of a workflow
#[derive(Component, Debug, Clone)]
pub struct WorkflowVisual {
    /// Workflow being visualized
    pub workflow_id: WorkflowId,

    /// Visual style
    pub style: WorkflowStyle,

    /// Layout algorithm
    pub layout: WorkflowLayout,

    /// Animation settings
    pub animation: WorkflowAnimation,
}

/// Visual style for workflows
#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowStyle {
    /// BPMN-style visualization
    Bpmn {
        color_scheme: ColorScheme,
        show_swimlanes: bool,
    },
    /// Flowchart style
    Flowchart {
        node_shape: WorkflowNodeShape,
        edge_style: EdgeStyle,
    },
    /// State machine style
    StateMachine {
        show_transitions: bool,
        highlight_current: bool,
    },
    /// Pipeline style (left to right)
    Pipeline {
        stage_width: f32,
        show_parallel: bool,
    },
}

/// Color schemes for workflow visualization
#[derive(Debug, Clone, PartialEq)]
pub enum ColorScheme {
    Default,
    Monochrome,
    HighContrast,
    Custom(HashMap<String, Color>),
}

/// Node shapes for workflow steps
#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowNodeShape {
    Rectangle,
    RoundedRectangle,
    Circle,
    Diamond,
    Hexagon,
}

/// Edge styles for workflow transitions
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeStyle {
    Straight,
    Curved,
    Orthogonal,
    Bezier,
}

/// Layout algorithms for workflows
#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowLayout {
    /// Hierarchical top-down layout
    Hierarchical { spacing: f32, level_gap: f32 },

    /// Left-to-right flow
    Horizontal { spacing: f32 },

    /// Circular layout for cyclic workflows
    Circular { radius: f32 },

    /// Force-directed for complex flows
    ForceDirected { iterations: u32 },

    /// Grid-based for structured workflows
    Grid { columns: u32, cell_size: f32 },
}

/// Animation settings for workflows
#[derive(Debug, Clone)]
pub struct WorkflowAnimation {
    /// Duration of step transitions
    pub transition_duration: f32,

    /// Show execution flow
    pub show_flow: bool,

    /// Particle effects for active transitions
    pub particle_effects: bool,

    /// Pulse effect for active steps
    pub pulse_active: bool,
}

impl Default for WorkflowAnimation {
    fn default() -> Self {
        Self {
            transition_duration: 0.5,
            show_flow: true,
            particle_effects: true,
            pulse_active: true,
        }
    }
}

/// Visual representation of a workflow step
#[derive(Component, Debug, Clone)]
pub struct WorkflowStepVisual {
    /// Step being visualized
    pub step_id: StepId,

    /// Step type for visual representation
    pub step_type: StepType,

    /// Current execution state
    pub state: StepState,

    /// Visual properties
    pub visual_props: StepVisualProperties,
}

/// Types of workflow steps
#[derive(Debug, Clone, PartialEq)]
pub enum StepType {
    /// Start of workflow
    Start,

    /// End of workflow
    End,

    /// User task
    UserTask {
        assignee: Option<String>,
        form_id: Option<String>,
    },

    /// System task
    SystemTask {
        service: String,
        operation: String,
    },

    /// Decision point
    Decision {
        condition: String,
    },

    /// Parallel gateway
    ParallelGateway {
        join_type: JoinType,
    },

    /// Timer event
    Timer {
        duration: f32,
    },

    /// Sub-workflow
    SubWorkflow {
        workflow_id: WorkflowId,
    },
}

/// Join types for parallel gateways
#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    /// All branches must complete
    And,
    /// Any branch completion continues
    Or,
    /// Specific number of branches
    Quorum(u32),
}

/// Execution state of a workflow step
#[derive(Debug, Clone, PartialEq)]
pub enum StepState {
    /// Not yet reached
    Pending,

    /// Currently executing
    Active {
        started_at: f32,
        progress: f32,
    },

    /// Completed successfully
    Completed {
        duration: f32,
    },

    /// Failed execution
    Failed {
        error: String,
        retry_count: u32,
    },

    /// Skipped due to conditions
    Skipped {
        reason: String,
    },

    /// Waiting for input
    Waiting {
        since: f32,
        timeout: Option<f32>,
    },
}

/// Visual properties for workflow steps
#[derive(Debug, Clone)]
pub struct StepVisualProperties {
    /// Base color
    pub color: Color,

    /// Icon to display
    pub icon: Option<String>,

    /// Size multiplier
    pub scale: f32,

    /// Glow intensity for active steps
    pub glow_intensity: f32,

    /// Custom shader parameters
    pub shader_params: HashMap<String, f32>,
}

impl Default for StepVisualProperties {
    fn default() -> Self {
        Self {
            color: Color::srgb(0.5, 0.5, 0.5),
            icon: None,
            scale: 1.0,
            glow_intensity: 0.0,
            shader_params: HashMap::new(),
        }
    }
}

/// Visual representation of workflow transitions
#[derive(Component, Debug, Clone)]
pub struct WorkflowTransitionVisual {
    /// Source step
    pub from_step: StepId,

    /// Target step
    pub to_step: StepId,

    /// Transition type
    pub transition_type: TransitionType,

    /// Visual state
    pub state: TransitionState,

    /// Animation progress
    pub animation_progress: f32,
}

/// Types of workflow transitions
#[derive(Debug, Clone, PartialEq)]
pub enum TransitionType {
    /// Sequential flow
    Sequential,

    /// Conditional flow
    Conditional {
        condition: String,
        probability: Option<f32>,
    },

    /// Error handling flow
    ErrorHandler {
        error_types: Vec<String>,
    },

    /// Compensation flow
    Compensation,

    /// Loop back
    Loop {
        max_iterations: Option<u32>,
    },
}

/// Visual state of transitions
#[derive(Debug, Clone, PartialEq)]
pub enum TransitionState {
    /// Inactive transition
    Inactive,

    /// Available for execution
    Available,

    /// Currently executing
    Active {
        tokens: u32,
        speed: f32,
    },

    /// Blocked by conditions
    Blocked {
        reason: String,
    },
}

/// Workflow execution token for visualization
#[derive(Component, Debug, Clone)]
pub struct WorkflowToken {
    /// Unique token ID
    pub id: Uuid,

    /// Current position on transition
    pub position: f32,

    /// Transition being traversed
    pub transition: Entity,

    /// Token data
    pub data: HashMap<String, serde_json::Value>,

    /// Visual style
    pub style: TokenStyle,
}

/// Visual style for workflow tokens
#[derive(Debug, Clone)]
pub struct TokenStyle {
    /// Token color
    pub color: Color,

    /// Token size
    pub size: f32,

    /// Trail effect
    pub trail: bool,

    /// Particle emission
    pub particles: bool,
}

impl Default for TokenStyle {
    fn default() -> Self {
        Self {
            color: Color::srgb(0.0, 0.8, 1.0),
            size: 0.3,
            trail: true,
            particles: false,
        }
    }
}

/// Swimlane for organizing workflow steps
#[derive(Component, Debug, Clone)]
pub struct WorkflowSwimlane {
    /// Swimlane name
    pub name: String,

    /// Actor or system responsible
    pub owner: String,

    /// Visual properties
    pub color: Color,
    pub width: f32,
    pub order: i32,
}

/// Workflow metrics overlay
#[derive(Component, Debug, Clone)]
pub struct WorkflowMetrics {
    /// Show execution times
    pub show_duration: bool,

    /// Show success/failure rates
    pub show_success_rate: bool,

    /// Show throughput
    pub show_throughput: bool,

    /// Time window for metrics
    pub time_window: f32,
}

/// Debug visualization for workflows
#[derive(Component, Debug, Clone)]
pub struct WorkflowDebugVisual {
    /// Show step IDs
    pub show_ids: bool,

    /// Show transition conditions
    pub show_conditions: bool,

    /// Show data flow
    pub show_data: bool,

    /// Highlight bottlenecks
    pub highlight_bottlenecks: bool,
}

/// Workflow instance tracker
#[derive(Component, Debug, Clone)]
pub struct WorkflowInstance {
    /// Instance ID
    pub instance_id: Uuid,

    /// Current active steps
    pub active_steps: Vec<StepId>,

    /// Execution history
    pub history: Vec<ExecutionEvent>,

    /// Instance variables
    pub variables: HashMap<String, serde_json::Value>,
}

/// Execution events for history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEvent {
    pub timestamp: f32,
    pub step_id: StepId,
    pub event_type: ExecutionEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEventType {
    StepStarted,
    StepCompleted { duration: f32 },
    StepFailed { error: String },
    TransitionTaken { to_step: StepId },
    VariableUpdated { name: String, value: serde_json::Value },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_visual_creation() {
        let visual = WorkflowVisual {
            workflow_id: WorkflowId::new(),
            style: WorkflowStyle::Bpmn {
                color_scheme: ColorScheme::Default,
                show_swimlanes: true,
            },
            layout: WorkflowLayout::Hierarchical {
                spacing: 100.0,
                level_gap: 50.0,
            },
            animation: WorkflowAnimation::default(),
        };

        assert!(visual.animation.show_flow);
        assert!(visual.animation.particle_effects);
    }

    #[test]
    fn test_step_state_transitions() {
        let mut state = StepState::Pending;

        // Transition to active
        state = StepState::Active {
            started_at: 0.0,
            progress: 0.0,
        };

        // Complete the step
        state = StepState::Completed {
            duration: 5.0,
        };

        match state {
            StepState::Completed { duration } => assert_eq!(duration, 5.0),
            _ => panic!("Expected completed state"),
        }
    }
}
