//! Visual Workflow Designer Plugin
//!
//! Provides drag-and-drop workflow creation and editing capabilities

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use std::collections::HashMap;
use uuid::Uuid;
use tracing::info;

use crate::domain::value_objects::{WorkflowId, StepId};
use crate::presentation::components::workflow_visualization::*;
use crate::presentation::events::WorkflowEvent;
use crate::presentation::systems::workflow_visualization::WorkflowVisualizationPlugin;

/// Plugin for visual workflow design
pub struct WorkflowDesignerPlugin;

impl Plugin for WorkflowDesignerPlugin {
    fn build(&self, app: &mut App) {
        // Only add EguiPlugin if not already added
        if !app.is_plugin_added::<bevy_egui::EguiPlugin>() {
            app.add_plugins(bevy_egui::EguiPlugin {
                enable_multipass_for_primary_context: false,
            });
        }

        app
            // Add workflow visualization
            .add_plugins(WorkflowVisualizationPlugin)
            // Resources
            .init_resource::<WorkflowDesignerState>()
            .init_resource::<WorkflowPalette>()
            .init_resource::<WorkflowTemplates>()
            // Events
            .add_event::<DesignerEvent>()
            // Systems
            .add_systems(Update, (
                workflow_designer_ui,
                handle_designer_events,
                drag_drop_workflow_steps,
                connect_workflow_steps,
                validate_workflow_connections,
                update_workflow_preview,
                toggle_workflow_designer,
            ).chain());
    }
}

/// State of the workflow designer
#[derive(Resource, Default)]
pub struct WorkflowDesignerState {
    /// Currently selected workflow
    pub selected_workflow: Option<WorkflowId>,

    /// Currently selected step
    pub selected_step: Option<StepId>,

    /// Currently selected transition
    pub selected_transition: Option<(StepId, StepId)>,

    /// Designer mode
    pub mode: DesignerMode,

    /// Drag state
    pub drag_state: Option<DragState>,

    /// Connection state
    pub connection_state: Option<ConnectionState>,

    /// Workflow being edited
    pub editing_workflow: Option<WorkflowDefinition>,

    /// Validation errors
    pub validation_errors: Vec<String>,

    /// Visibility of the workflow designer
    pub visible: bool,
}

/// Designer modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DesignerMode {
    #[default]
    Edit,
    Test,
    Debug,
    Connect,
}

/// Drag state for drag-and-drop
#[derive(Debug, Clone)]
pub struct DragState {
    pub item_type: DragItemType,
    pub start_position: Vec2,
    pub current_position: Vec2,
}

/// Types of draggable items
#[derive(Debug, Clone)]
pub enum DragItemType {
    PaletteStep(StepType),
    ExistingStep(StepId),
}

/// Connection state for creating transitions
#[derive(Debug, Clone)]
pub struct ConnectionState {
    pub from_step: StepId,
    pub preview_position: Vec2,
}

/// Workflow definition being edited
#[derive(Debug, Clone)]
pub struct WorkflowDefinition {
    pub id: WorkflowId,
    pub name: String,
    pub description: String,
    pub steps: HashMap<StepId, WorkflowStepDefinition>,
    pub transitions: Vec<WorkflowTransitionDefinition>,
}

/// Step definition in workflow
#[derive(Debug, Clone)]
pub struct WorkflowStepDefinition {
    pub id: StepId,
    pub step_type: StepType,
    pub name: String,
    pub position: Vec2,
    pub properties: HashMap<String, String>,
}

/// Transition definition in workflow
#[derive(Debug, Clone)]
pub struct WorkflowTransitionDefinition {
    pub from_step: StepId,
    pub to_step: StepId,
    pub transition_type: TransitionType,
    pub condition: Option<String>,
}

/// Palette of workflow step types
#[derive(Resource)]
pub struct WorkflowPalette {
    pub categories: Vec<PaletteCategory>,
}

impl Default for WorkflowPalette {
    fn default() -> Self {
        Self {
            categories: vec![
                PaletteCategory {
                    name: "Basic".to_string(),
                    items: vec![
                        PaletteItem {
                            name: "Start".to_string(),
                            step_type: StepType::Start,
                            icon: "⭕".to_string(),
                        },
                        PaletteItem {
                            name: "End".to_string(),
                            step_type: StepType::End,
                            icon: "🏁".to_string(),
                        },
                    ],
                },
                PaletteCategory {
                    name: "Tasks".to_string(),
                    items: vec![
                        PaletteItem {
                            name: "User Task".to_string(),
                            step_type: StepType::UserTask {
                                assignee: None,
                                form_id: None,
                            },
                            icon: "👤".to_string(),
                        },
                        PaletteItem {
                            name: "System Task".to_string(),
                            step_type: StepType::SystemTask {
                                service: String::new(),
                                operation: String::new(),
                            },
                            icon: "⚙️".to_string(),
                        },
                    ],
                },
                PaletteCategory {
                    name: "Control Flow".to_string(),
                    items: vec![
                        PaletteItem {
                            name: "Decision".to_string(),
                            step_type: StepType::Decision {
                                condition: String::new(),
                            },
                            icon: "❓".to_string(),
                        },
                        PaletteItem {
                            name: "Parallel Gateway".to_string(),
                            step_type: StepType::ParallelGateway {
                                join_type: JoinType::And,
                            },
                            icon: "⧉".to_string(),
                        },
                    ],
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct PaletteCategory {
    pub name: String,
    pub items: Vec<PaletteItem>,
}

#[derive(Debug, Clone)]
pub struct PaletteItem {
    pub name: String,
    pub step_type: StepType,
    pub icon: String,
}

/// Workflow templates
#[derive(Resource, Default)]
pub struct WorkflowTemplates {
    pub templates: Vec<WorkflowTemplate>,
}

#[derive(Debug, Clone)]
pub struct WorkflowTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: String,
    pub definition: WorkflowDefinition,
}

/// Designer events
#[derive(Event, Debug, Clone)]
pub enum DesignerEvent {
    /// Create new workflow
    NewWorkflow,

    /// Save current workflow
    SaveWorkflow,

    /// Load workflow
    LoadWorkflow,

    /// Undo last action
    Undo,

    /// Redo last undone action
    Redo,

    /// Validate workflow
    ValidateWorkflow,

    /// Deploy workflow
    DeployWorkflow,

    /// Start dragging a step
    StartDragStep {
        step_type: StepType,
    },

    /// Drop step at position
    DropStep {
        step_type: StepType,
        position: Vec2,
    },

    /// Select step
    SelectStep {
        step_id: StepId,
    },

    /// Delete step
    DeleteStep {
        step_id: StepId,
    },

    /// Connect steps
    ConnectSteps {
        from: StepId,
        to: StepId,
    },

    /// Delete transition
    DeleteTransition {
        from: StepId,
        to: StepId,
    },

    /// Load template
    LoadTemplate {
        template_id: Uuid,
    },
}

/// System to toggle workflow designer visibility with keyboard shortcut
fn toggle_workflow_designer(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<WorkflowDesignerState>,
) {
    // Press 'W' to toggle workflow designer
    if keyboard.just_pressed(KeyCode::KeyW) {
        state.visible = !state.visible;
        if state.visible {
            info!("Workflow Designer opened - Press 'W' to close");
        } else {
            info!("Workflow Designer closed - Press 'W' to open");
        }
    }
}

/// Main UI system for the workflow designer
fn workflow_designer_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<WorkflowDesignerState>,
    palette: Res<WorkflowPalette>,
    templates: Res<WorkflowTemplates>,
    mut events: EventWriter<DesignerEvent>,
) {
    // Only show UI if visible
    if !state.visible {
        return;
    }

    let ctx = contexts.ctx_mut();

    // Top toolbar
    egui::TopBottomPanel::top("workflow_toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("New Workflow").clicked() {
                events.send(DesignerEvent::NewWorkflow);
            }

            if ui.button("Save").clicked() {
                events.send(DesignerEvent::SaveWorkflow);
            }

            if ui.button("Load").clicked() {
                events.send(DesignerEvent::LoadWorkflow);
            }

            ui.separator();

            if ui.button("Undo").clicked() {
                events.send(DesignerEvent::Undo);
            }

            if ui.button("Redo").clicked() {
                events.send(DesignerEvent::Redo);
            }

            ui.separator();

            ui.label("Mode:");
            ui.selectable_value(&mut state.mode, DesignerMode::Edit, "Edit");
            ui.selectable_value(&mut state.mode, DesignerMode::Test, "Test");
            ui.selectable_value(&mut state.mode, DesignerMode::Debug, "Debug");
        });
    });

    // Left panel - Step palette
    egui::SidePanel::left("step_palette").show(ctx, |ui| {
        ui.heading("Workflow Steps");

        for category in &palette.categories {
            ui.collapsing(&category.name, |ui| {
                for item in &category.items {
                    let response = ui.button(&item.name);
                    if response.clicked() {
                        events.send(DesignerEvent::StartDragStep {
                            step_type: item.step_type.clone(),
                        });
                    }
                }
            });
        }

        ui.separator();
        ui.heading("Templates");

        for template in &templates.templates {
            if ui.button(&template.name).clicked() {
                events.send(DesignerEvent::LoadTemplate {
                    template_id: template.id,
                });
            }
        }
    });

    // Right panel - Properties
    egui::SidePanel::right("properties_panel").show(ctx, |ui| {
        ui.heading("Properties");

        if let Some(selected_step) = &state.selected_step {
            ui.label(format!("Step: {}", selected_step));

            // Step-specific properties would go here
            ui.separator();

            ui.label("Conditions:");
            // Condition editor would go here

            ui.separator();

            ui.label("Actions:");
            // Action editor would go here

            ui.separator();

            if ui.button("Delete Step").clicked() {
                events.send(DesignerEvent::DeleteStep {
                    step_id: *selected_step,
                });
            }
        } else if let Some(selected_transition) = &state.selected_transition {
            ui.label(format!("Transition: {:?}", selected_transition));

            // Transition properties would go here
            ui.separator();

            if ui.button("Delete Transition").clicked() {
                events.send(DesignerEvent::DeleteTransition {
                    from: selected_transition.0,
                    to: selected_transition.1,
                });
            }
        } else {
            ui.label("Select a step or transition to edit properties");
        }
    });

    // Bottom panel - Validation messages
    egui::TopBottomPanel::bottom("validation_panel").show(ctx, |ui| {
        if !state.validation_errors.is_empty() {
            ui.heading("Validation Errors");
            for error in &state.validation_errors {
                ui.colored_label(egui::Color32::RED, error);
            }
        }
    });
}

/// System to handle designer events
pub fn handle_designer_events(
    mut events: EventReader<DesignerEvent>,
    mut state: ResMut<WorkflowDesignerState>,
    mut commands: Commands,
) {
    for event in events.read() {
        match event {
            DesignerEvent::NewWorkflow => {
                state.editing_workflow = Some(WorkflowDefinition {
                    id: WorkflowId::new(),
                    name: "New Workflow".to_string(),
                    description: String::new(),
                    steps: HashMap::new(),
                    transitions: Vec::new(),
                });
                state.selected_step = None;
                state.validation_errors.clear();
            }
            DesignerEvent::DropStep { step_type, position } => {
                if let Some(workflow) = &mut state.editing_workflow {
                    let step_id = StepId::new();
                    let step_def = WorkflowStepDefinition {
                        id: step_id,
                        step_type: step_type.clone(),
                        name: format!("Step {}", workflow.steps.len() + 1),
                        position: *position,
                        properties: HashMap::new(),
                    };

                    workflow.steps.insert(step_id, step_def);

                    // Create visual entity
                    commands.spawn(WorkflowStepVisual {
                        step_id,
                        step_type: step_type.clone(),
                        state: StepState::Pending,
                        visual_props: StepVisualProperties::default(),
                    });
                }
            }
            DesignerEvent::ValidateWorkflow => {
                if let Some(workflow) = &state.editing_workflow {
                    state.validation_errors = validate_workflow(workflow);
                }
            }
            _ => {}
        }
    }
}

/// System for drag-and-drop workflow steps
pub fn drag_drop_workflow_steps(
    mut state: ResMut<WorkflowDesignerState>,
    mut events: EventWriter<DesignerEvent>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
) {
    // Check if we're dragging
    if let Some(drag_state) = state.drag_state.as_mut() {
        if let Ok(window) = windows.get_single() {
            if let Some(cursor_pos) = window.cursor_position() {
                drag_state.current_position = cursor_pos;

                // Handle drop
                if mouse.just_released(MouseButton::Left) {
                    let drop_event = match &drag_state.item_type {
                        DragItemType::PaletteStep(step_type) => {
                            Some(DesignerEvent::DropStep {
                                step_type: step_type.clone(),
                                position: cursor_pos,
                            })
                        }
                        DragItemType::ExistingStep(step_id) => {
                            // We'll handle this after clearing drag state
                            None
                        }
                    };

                    // Store the step id if we need to update position
                    let existing_step = match &drag_state.item_type {
                        DragItemType::ExistingStep(step_id) => Some((*step_id, cursor_pos)),
                        _ => None,
                    };

                    // Clear drag state first
                    state.drag_state = None;

                    // Now handle the events
                    if let Some(event) = drop_event {
                        events.send(event);
                    }

                    // Update existing step position
                    if let Some((step_id, position)) = existing_step {
                        if let Some(workflow) = &mut state.editing_workflow {
                            if let Some(step) = workflow.steps.get_mut(&step_id) {
                                step.position = position;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// System to connect workflow steps
pub fn connect_workflow_steps(
    mut state: ResMut<WorkflowDesignerState>,
    mut events: EventWriter<DesignerEvent>,
    mouse: Res<ButtonInput<MouseButton>>,
    step_query: Query<(&WorkflowStepVisual, &Transform)>,
) {
    if state.mode != DesignerMode::Connect {
        return;
    }

    if mouse.just_pressed(MouseButton::Left) {
        // Find clicked step
        // TODO: Implement ray casting to find clicked step
    }

    if let Some(connection_state) = &state.connection_state {
        if mouse.just_released(MouseButton::Left) {
            // Find target step and create connection
            // TODO: Implement target step detection
            state.connection_state = None;
        }
    }
}

/// System to validate workflow connections
pub fn validate_workflow_connections(
    state: Res<WorkflowDesignerState>,
    mut gizmos: Gizmos,
) {
    if let Some(workflow) = &state.editing_workflow {
        // Highlight invalid connections
        for transition in &workflow.transitions {
            // Check if transition is valid
            let is_valid = validate_transition(workflow, transition);
            let color = if is_valid {
                Color::srgb(0.0, 0.8, 0.0)
            } else {
                Color::srgb(0.8, 0.0, 0.0)
            };

            // Draw validation indicator
            // TODO: Get actual positions from entities
        }
    }
}

/// System to update workflow preview
pub fn update_workflow_preview(
    state: Res<WorkflowDesignerState>,
    mut step_query: Query<(&mut Transform, &WorkflowStepVisual)>,
) {
    if let Some(workflow) = &state.editing_workflow {
        for (mut transform, step_visual) in step_query.iter_mut() {
            if let Some(step_def) = workflow.steps.get(&step_visual.step_id) {
                transform.translation.x = step_def.position.x;
                transform.translation.y = step_def.position.y;
            }
        }
    }
}

/// Validate workflow definition
fn validate_workflow(workflow: &WorkflowDefinition) -> Vec<String> {
    let mut errors = Vec::new();

    // Check for start step
    let has_start = workflow.steps.values()
        .any(|s| matches!(s.step_type, StepType::Start));
    if !has_start {
        errors.push("Workflow must have a start step".to_string());
    }

    // Check for end step
    let has_end = workflow.steps.values()
        .any(|s| matches!(s.step_type, StepType::End));
    if !has_end {
        errors.push("Workflow must have an end step".to_string());
    }

    // Check for unreachable steps
    // TODO: Implement reachability analysis

    errors
}

/// Validate a single transition
fn validate_transition(
    workflow: &WorkflowDefinition,
    transition: &WorkflowTransitionDefinition,
) -> bool {
    // Check if both steps exist
    workflow.steps.contains_key(&transition.from_step) &&
    workflow.steps.contains_key(&transition.to_step)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_validation() {
        let mut workflow = WorkflowDefinition {
            id: WorkflowId::new(),
            name: "Test".to_string(),
            description: String::new(),
            steps: HashMap::new(),
            transitions: Vec::new(),
        };

        // Empty workflow should have errors
        let errors = validate_workflow(&workflow);
        assert_eq!(errors.len(), 2); // No start, no end

        // Add start step
        let start_id = StepId::new();
        workflow.steps.insert(start_id, WorkflowStepDefinition {
            id: start_id,
            step_type: StepType::Start,
            name: "Start".to_string(),
            position: Vec2::ZERO,
            properties: HashMap::new(),
        });

        // Should still have one error (no end)
        let errors = validate_workflow(&workflow);
        assert_eq!(errors.len(), 1);
    }
}
