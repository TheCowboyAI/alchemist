//! Document Approval Workflow Example
//!
//! Demonstrates a complete workflow implementation with:
//! - States: Draft, Review, Approved, Rejected
//! - Transitions with guards
//! - NATS integration for distributed execution
//! - Bevy visualization of workflow state

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use cim_domain::workflow::{
    WorkflowState, TransitionInput, TransitionOutput,
    WorkflowContext, WorkflowTransition,
};
use cim_workflow_graph::WorkflowGraph;

use information_alchemist::{
    application::services::WorkflowExecutionService,
    infrastructure::{
        nats::{NatsClient, NatsConfig},
        event_store::EventStore,
        event_bridge::{EventBridge, EventBridgePlugin, WorkflowEventBridgePlugin},
    },
    presentation::{
        components::workflow_visualization::*,
        systems::workflow_visualization::WorkflowVisualizationPlugin,
        events::WorkflowEvent,
    },
    domain::value_objects::{WorkflowId, StepId, GraphId},
};

/// Document workflow states
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum DocumentState {
    Draft,
    UnderReview,
    Approved,
    Rejected,
}

impl WorkflowState for DocumentState {
    fn id(&self) -> cim_domain::identifiers::StateId {
        cim_domain::identifiers::StateId::from(format!("{:?}", self))
    }

    fn name(&self) -> &str {
        match self {
            DocumentState::Draft => "Draft",
            DocumentState::UnderReview => "Under Review",
            DocumentState::Approved => "Approved",
            DocumentState::Rejected => "Rejected",
        }
    }

    fn is_terminal(&self) -> bool {
        matches!(self, DocumentState::Approved | DocumentState::Rejected)
    }

    fn metadata(&self) -> std::collections::HashMap<String, serde_json::Value> {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("state_type".to_string(), serde_json::json!(format!("{:?}", self)));
        metadata
    }
}

/// Input for document transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
enum DocumentInput {
    SubmitForReview { author: String },
    Approve { reviewer: String },
    Reject { reviewer: String, reason: String },
    Revise { author: String },
}

impl TransitionInput for DocumentInput {
    fn name(&self) -> &str {
        match self {
            DocumentInput::SubmitForReview { .. } => "Submit for Review",
            DocumentInput::Approve { .. } => "Approve",
            DocumentInput::Reject { .. } => "Reject",
            DocumentInput::Revise { .. } => "Revise",
        }
    }
}

/// Output from document transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
enum DocumentOutput {
    Submitted { timestamp: String },
    Approved { timestamp: String, reviewer: String },
    Rejected { timestamp: String, reviewer: String, reason: String },
    Revised { timestamp: String, author: String },
}

impl TransitionOutput for DocumentOutput {
    fn name(&self) -> &str {
        match self {
            DocumentOutput::Submitted { .. } => "Submitted",
            DocumentOutput::Approved { .. } => "Approved",
            DocumentOutput::Rejected { .. } => "Rejected",
            DocumentOutput::Revised { .. } => "Revised",
        }
    }
}

/// Document approval transition
struct ApprovalTransition {
    source: DocumentState,
    target: DocumentState,
    input_type: String,
}

impl WorkflowTransition<DocumentState, DocumentInput, DocumentOutput> for ApprovalTransition {
    fn source(&self) -> &DocumentState {
        &self.source
    }

    fn target(&self) -> &DocumentState {
        &self.target
    }

    fn name(&self) -> &str {
        &self.input_type
    }

    fn guard(&self, context: &WorkflowContext) -> bool {
        // Check if user has appropriate permissions
        match &self.input_type.as_str() {
            &"approve" | &"reject" => {
                // Only reviewers can approve/reject
                context.get::<String>("role")
                    .map(|role| role == "reviewer")
                    .unwrap_or(false)
            }
            _ => true,
        }
    }

    fn accepts_input(&self, input: &DocumentInput) -> bool {
        match (&self.source, &self.target, input) {
            (DocumentState::Draft, DocumentState::UnderReview, DocumentInput::SubmitForReview { .. }) => true,
            (DocumentState::UnderReview, DocumentState::Approved, DocumentInput::Approve { .. }) => true,
            (DocumentState::UnderReview, DocumentState::Rejected, DocumentInput::Reject { .. }) => true,
            (DocumentState::Rejected, DocumentState::Draft, DocumentInput::Revise { .. }) => true,
            _ => false,
        }
    }

    fn execute(&self, input: &DocumentInput, _context: &WorkflowContext) -> Result<DocumentOutput, cim_domain::workflow::TransitionError> {
        let timestamp = chrono::Utc::now().to_rfc3339();

        match input {
            DocumentInput::SubmitForReview { .. } => {
                Ok(DocumentOutput::Submitted { timestamp })
            }
            DocumentInput::Approve { reviewer } => {
                Ok(DocumentOutput::Approved {
                    timestamp,
                    reviewer: reviewer.clone(),
                })
            }
            DocumentInput::Reject { reviewer, reason } => {
                Ok(DocumentOutput::Rejected {
                    timestamp,
                    reviewer: reviewer.clone(),
                    reason: reason.clone(),
                })
            }
            DocumentInput::Revise { author } => {
                Ok(DocumentOutput::Revised {
                    timestamp,
                    author: author.clone(),
                })
            }
        }
    }
}

/// Create the approval workflow graph
fn create_approval_workflow() -> WorkflowGraph<DocumentState, DocumentInput, DocumentOutput, f32> {
    let mut workflow = WorkflowGraph::new(
        GraphId::new(),
        cim_workflow_graph::WorkflowType::Sequential,
    );

    // Add states
    let draft = workflow.add_state(DocumentState::Draft);
    let review = workflow.add_state(DocumentState::UnderReview);
    let approved = workflow.add_state(DocumentState::Approved);
    let rejected = workflow.add_state(DocumentState::Rejected);

    // Add transitions
    workflow.add_transition(
        draft,
        review,
        Box::new(ApprovalTransition {
            source: DocumentState::Draft,
            target: DocumentState::UnderReview,
            input_type: "submit".to_string(),
        }),
        1.0, // enrichment value (cost/time)
    );

    workflow.add_transition(
        review,
        approved,
        Box::new(ApprovalTransition {
            source: DocumentState::UnderReview,
            target: DocumentState::Approved,
            input_type: "approve".to_string(),
        }),
        0.5,
    );

    workflow.add_transition(
        review,
        rejected,
        Box::new(ApprovalTransition {
            source: DocumentState::UnderReview,
            target: DocumentState::Rejected,
            input_type: "reject".to_string(),
        }),
        0.5,
    );

    workflow.add_transition(
        rejected,
        draft,
        Box::new(ApprovalTransition {
            source: DocumentState::Rejected,
            target: DocumentState::Draft,
            input_type: "revise".to_string(),
        }),
        2.0,
    );

    workflow
}

/// Bevy app for workflow visualization
fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,approval_workflow=debug")
        .init();

    // Create NATS client
    let nats_config = NatsConfig::default();
    let nats_client = Arc::new(
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(NatsClient::new(nats_config))
            .expect("Failed to create NATS client")
    );

    // Create event store
    let event_store = Arc::new(EventStore::new());

    // Create workflow execution service
    let workflow_service = Arc::new(WorkflowExecutionService::<DocumentState, DocumentInput, DocumentOutput>::new(
        nats_client.clone(),
        event_store.clone(),
        "workflow.document".to_string(),
    ));

    // Register the workflow definition
    let workflow_def = create_approval_workflow();
    let workflow_def_id = workflow_def.id.clone();

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(workflow_service.register_definition(workflow_def))
        .expect("Failed to register workflow");

    // Create Bevy app
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EventBridgePlugin)
        .add_plugins(WorkflowVisualizationPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_input,
            update_workflow_display,
        ))
        .insert_resource(WorkflowService {
            service: workflow_service,
            definition_id: workflow_def_id,
            current_workflow: None,
        })
        .run();
}

/// Resource to hold workflow service
#[derive(Resource)]
struct WorkflowService {
    service: Arc<WorkflowExecutionService<DocumentState, DocumentInput, DocumentOutput>>,
    definition_id: GraphId,
    current_workflow: Option<WorkflowId>,
}

/// Setup the 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 4.0)),
    ));

    // Create workflow visualization
    commands.spawn((
        WorkflowVisual {
            workflow_id: WorkflowId::new(),
            style: WorkflowStyle::Bpmn {
                color_scheme: ColorScheme::Default,
                show_swimlanes: false,
            },
            layout: WorkflowLayout::Hierarchical {
                spacing: 3.0,
                level_gap: 2.0,
            },
            animation: WorkflowAnimation::default(),
        },
        Transform::default(),
        GlobalTransform::default(),
    ));

    // Add workflow steps
    let states = vec![
        (DocumentState::Draft, Vec3::new(-6.0, 0.0, 0.0)),
        (DocumentState::UnderReview, Vec3::new(-2.0, 0.0, 0.0)),
        (DocumentState::Approved, Vec3::new(2.0, 2.0, 0.0)),
        (DocumentState::Rejected, Vec3::new(2.0, -2.0, 0.0)),
    ];

    for (state, position) in states {
        let step_type = match state {
            DocumentState::Draft => StepType::Start,
            DocumentState::Approved | DocumentState::Rejected => StepType::End,
            _ => StepType::UserTask {
                assignee: Some("Reviewer".to_string()),
                form_id: None,
            },
        };

        commands.spawn((
            WorkflowStepVisual {
                step_id: StepId::from(format!("{:?}", state)),
                step_type,
                state: StepState::Pending,
                visual_props: StepVisualProperties::default(),
            },
            Transform::from_translation(position),
            GlobalTransform::default(),
        ));
    }

    // Instructions
    info!("Document Approval Workflow Example");
    info!("Press SPACE to start a new workflow");
    info!("Press 1 to submit for review");
    info!("Press 2 to approve");
    info!("Press 3 to reject");
    info!("Press 4 to revise");
}

/// Handle keyboard input
fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut workflow_service: ResMut<WorkflowService>,
    runtime: Res<Runtime>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        // Start new workflow
        let service = workflow_service.service.clone();
        let def_id = workflow_service.definition_id.clone();

        let workflow_id = runtime.block_on(async {
            let mut context = WorkflowContext::new();
            context.set("role", "author".to_string());

            service.start_workflow(def_id, context).await
        }).expect("Failed to start workflow");

        workflow_service.current_workflow = Some(workflow_id);
        info!("Started workflow: {}", workflow_id);
    }

    if let Some(workflow_id) = &workflow_service.current_workflow {
        if keyboard.just_pressed(KeyCode::Digit1) {
            // Submit for review
            let service = workflow_service.service.clone();
            let wf_id = workflow_id.clone();

            runtime.block_on(async {
                service.execute_transition(
                    &wf_id,
                    DocumentInput::SubmitForReview {
                        author: "John Doe".to_string(),
                    },
                ).await
            }).ok();
        }

        if keyboard.just_pressed(KeyCode::Digit2) {
            // Approve
            let service = workflow_service.service.clone();
            let wf_id = workflow_id.clone();

            runtime.block_on(async {
                // Change role to reviewer
                let mut context = WorkflowContext::new();
                context.set("role", "reviewer".to_string());

                service.execute_transition(
                    &wf_id,
                    DocumentInput::Approve {
                        reviewer: "Jane Smith".to_string(),
                    },
                ).await
            }).ok();
        }

        if keyboard.just_pressed(KeyCode::Digit3) {
            // Reject
            let service = workflow_service.service.clone();
            let wf_id = workflow_id.clone();

            runtime.block_on(async {
                // Change role to reviewer
                let mut context = WorkflowContext::new();
                context.set("role", "reviewer".to_string());

                service.execute_transition(
                    &wf_id,
                    DocumentInput::Reject {
                        reviewer: "Jane Smith".to_string(),
                        reason: "Needs more detail".to_string(),
                    },
                ).await
            }).ok();
        }

        if keyboard.just_pressed(KeyCode::Digit4) {
            // Revise
            let service = workflow_service.service.clone();
            let wf_id = workflow_id.clone();

            runtime.block_on(async {
                service.execute_transition(
                    &wf_id,
                    DocumentInput::Revise {
                        author: "John Doe".to_string(),
                    },
                ).await
            }).ok();
        }
    }
}

/// Update workflow display based on events
fn update_workflow_display(
    mut events: EventReader<WorkflowEvent>,
    mut step_query: Query<(&mut WorkflowStepVisual, &mut Transform)>,
) {
    for event in events.read() {
        match event {
            WorkflowEvent::StepStarted { step_id } => {
                for (mut step, _) in step_query.iter_mut() {
                    if step.step_id == *step_id {
                        step.state = StepState::Active {
                            started_at: 0.0,
                            progress: 0.0,
                        };
                        info!("Step started: {:?}", step_id);
                    }
                }
            }
            WorkflowEvent::StepCompleted { step_id, duration } => {
                for (mut step, _) in step_query.iter_mut() {
                    if step.step_id == *step_id {
                        step.state = StepState::Completed {
                            duration: *duration,
                        };
                        info!("Step completed: {:?} in {}s", step_id, duration);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Runtime resource for async operations
#[derive(Resource)]
struct Runtime(tokio::runtime::Runtime);

impl Default for Runtime {
    fn default() -> Self {
        Self(tokio::runtime::Runtime::new().unwrap())
    }
}
