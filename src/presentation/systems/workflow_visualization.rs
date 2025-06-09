//! Workflow visualization systems
//!
//! Systems for rendering and animating workflow execution

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::presentation::components::workflow_visualization::*;
use crate::presentation::events::WorkflowEvent;

/// Plugin for workflow visualization
pub struct WorkflowVisualizationPlugin;

impl Plugin for WorkflowVisualizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            visualize_workflow_steps,
            visualize_workflow_transitions,
            animate_workflow_tokens,
            update_step_states,
            apply_workflow_layout,
        ).chain());
    }
}

/// System to visualize workflow steps
pub fn visualize_workflow_steps(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &WorkflowStepVisual), Added<WorkflowStepVisual>>,
) {
    for (entity, step_visual) in query.iter() {
        let (mesh, material) = create_step_mesh_and_material(
            &step_visual.step_type,
            &step_visual.state,
            &step_visual.visual_props,
            &mut meshes,
            &mut materials,
        );

        commands.entity(entity).insert((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::default(),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));
    }
}

/// Create mesh and material for a workflow step
fn create_step_mesh_and_material(
    step_type: &StepType,
    state: &StepState,
    props: &StepVisualProperties,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> (Handle<Mesh>, Handle<StandardMaterial>) {
    // Create mesh based on step type
    let mesh = match step_type {
        StepType::Start | StepType::End => {
            meshes.add(Circle::new(props.scale * 0.5))
        }
        StepType::Decision { .. } => {
            meshes.add(RegularPolygon::new(props.scale * 0.6, 4)) // Diamond
        }
        StepType::ParallelGateway { .. } => {
            meshes.add(RegularPolygon::new(props.scale * 0.5, 6)) // Hexagon
        }
        _ => {
            meshes.add(Rectangle::new(props.scale * 1.2, props.scale * 0.8))
        }
    };

    // Create material based on state
    let base_color = match state {
        StepState::Pending => {
            let [r, g, b, a] = props.color.to_srgba().to_f32_array();
            Color::srgba(r * 0.5, g * 0.5, b * 0.5, a)
        }
        StepState::Active { .. } => props.color,
        StepState::Completed { .. } => Color::srgb(0.0, 0.8, 0.0),
        StepState::Failed { .. } => Color::srgb(0.8, 0.0, 0.0),
        StepState::Skipped { .. } => Color::srgb(0.5, 0.5, 0.5),
        StepState::Waiting { .. } => Color::srgb(1.0, 0.8, 0.0),
    };

    let material = materials.add(StandardMaterial {
        base_color,
        emissive: LinearRgba::from(base_color) * props.glow_intensity,
        ..default()
    });

    (mesh, material)
}

/// System to visualize workflow transitions
pub fn visualize_workflow_transitions(
    _commands: Commands,
    mut gizmos: Gizmos,
    step_query: Query<(&WorkflowStepVisual, &Transform)>,
    transition_query: Query<&WorkflowTransitionVisual>,
) {
    // Create a map of step IDs to positions
    let mut step_positions = HashMap::new();
    for (step_visual, transform) in step_query.iter() {
        step_positions.insert(step_visual.step_id, transform.translation);
    }

    // Draw transitions
    for transition in transition_query.iter() {
        if let (Some(&from_pos), Some(&to_pos)) = (
            step_positions.get(&transition.from_step),
            step_positions.get(&transition.to_step),
        ) {
            let color = match &transition.state {
                TransitionState::Inactive => Color::srgb(0.3, 0.3, 0.3),
                TransitionState::Available => Color::srgb(0.0, 0.5, 0.8),
                TransitionState::Active { .. } => Color::srgb(0.0, 0.8, 1.0),
                TransitionState::Blocked { .. } => Color::srgb(0.8, 0.3, 0.0),
            };

            // Draw the transition line
            match &transition.transition_type {
                TransitionType::Sequential => {
                    gizmos.line(from_pos, to_pos, color);
                }
                TransitionType::Conditional { .. } => {
                    // Draw dashed line for conditional
                    draw_dashed_line(&mut gizmos, from_pos, to_pos, color, 0.1);
                }
                TransitionType::Loop { .. } => {
                    // Draw curved line for loops
                    draw_curved_line(&mut gizmos, from_pos, to_pos, color);
                }
                _ => {
                    gizmos.line(from_pos, to_pos, color);
                }
            }

            // Draw arrowhead
            draw_arrowhead(&mut gizmos, from_pos, to_pos, color);
        }
    }
}

/// Draw a dashed line
fn draw_dashed_line(
    gizmos: &mut Gizmos,
    from: Vec3,
    to: Vec3,
    color: Color,
    dash_length: f32,
) {
    let direction = (to - from).normalize();
    let distance = from.distance(to);
    let num_dashes = (distance / (dash_length * 2.0)) as i32;

    for i in 0..num_dashes {
        let start = from + direction * (i as f32 * dash_length * 2.0);
        let end = from + direction * ((i as f32 * dash_length * 2.0) + dash_length);
        if end.distance(from) <= distance {
            gizmos.line(start, end.min(to), color);
        }
    }
}

/// Draw a curved line
fn draw_curved_line(
    gizmos: &mut Gizmos,
    from: Vec3,
    to: Vec3,
    color: Color,
) {
    let mid = (from + to) * 0.5;
    let offset = Vec3::new(0.0, 0.5, 0.0);
    let control = mid + offset;

    // Simple bezier curve approximation
    let segments = 20;
    for i in 0..segments {
        let t1 = i as f32 / segments as f32;
        let t2 = (i + 1) as f32 / segments as f32;

        let p1 = bezier_point(from, control, to, t1);
        let p2 = bezier_point(from, control, to, t2);

        gizmos.line(p1, p2, color);
    }
}

/// Calculate bezier curve point
fn bezier_point(p0: Vec3, p1: Vec3, p2: Vec3, t: f32) -> Vec3 {
    let t2 = t * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;

    p0 * mt2 + p1 * 2.0 * mt * t + p2 * t2
}

/// Draw an arrowhead
fn draw_arrowhead(
    gizmos: &mut Gizmos,
    from: Vec3,
    to: Vec3,
    color: Color,
) {
    let direction = (to - from).normalize();
    let arrow_size = 0.2;
    let arrow_angle = 0.5;

    let right = Vec3::new(-direction.z, 0.0, direction.x) * arrow_size;
    let back = -direction * arrow_size;

    let arrow_point1 = to + back + right * arrow_angle;
    let arrow_point2 = to + back - right * arrow_angle;

    gizmos.line(to, arrow_point1, color);
    gizmos.line(to, arrow_point2, color);
}

/// System to animate workflow tokens
pub fn animate_workflow_tokens(
    mut token_query: Query<(&mut Transform, &mut WorkflowToken)>,
    transition_query: Query<&WorkflowTransitionVisual>,
    step_query: Query<(&WorkflowStepVisual, &Transform), Without<WorkflowToken>>,
    time: Res<Time>,
) {
    // Create step position map
    let mut step_positions = HashMap::new();
    for (step_visual, transform) in step_query.iter() {
        step_positions.insert(step_visual.step_id, transform.translation);
    }

    for (mut transform, mut token) in token_query.iter_mut() {
        if let Ok(transition) = transition_query.get(token.transition) {
            if let (Some(&from_pos), Some(&to_pos)) = (
                step_positions.get(&transition.from_step),
                step_positions.get(&transition.to_step),
            ) {
                // Update token position
                token.position += time.delta_secs() * 0.5; // Speed

                if token.position >= 1.0 {
                    // Token reached destination
                    token.position = 1.0;
                }

                // Interpolate position
                transform.translation = from_pos.lerp(to_pos, token.position);
            }
        }
    }
}

/// System to update step states based on events
pub fn update_step_states(
    mut step_query: Query<(&mut WorkflowStepVisual, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut events: EventReader<WorkflowEvent>,
    time: Res<Time>,
) {
    for event in events.read() {
        match event {
            WorkflowEvent::StepStarted { step_id } => {
                for (mut step_visual, material_handle) in step_query.iter_mut() {
                    if step_visual.step_id == *step_id {
                        step_visual.state = StepState::Active {
                            started_at: time.elapsed_secs(),
                            progress: 0.0,
                        };

                        // Update material to active color
                        if let Some(material) = materials.get_mut(&material_handle.0) {
                            material.base_color = Color::srgb(0.0, 0.8, 1.0);
                            material.emissive = LinearRgba::from(Color::srgb(0.0, 0.8, 1.0)) * 0.5;
                        }
                    }
                }
            }
            WorkflowEvent::StepCompleted { step_id, duration } => {
                for (mut step_visual, material_handle) in step_query.iter_mut() {
                    if step_visual.step_id == *step_id {
                        step_visual.state = StepState::Completed {
                            duration: *duration,
                        };

                        // Update material to completed color
                        if let Some(material) = materials.get_mut(&material_handle.0) {
                            material.base_color = Color::srgb(0.0, 0.8, 0.0);
                            material.emissive = LinearRgba::from(Color::srgb(0.0, 0.8, 0.0)) * 0.2;
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// System to apply workflow layout algorithms
pub fn apply_workflow_layout(
    mut step_query: Query<(Entity, &WorkflowStepVisual, &mut Transform), Changed<WorkflowStepVisual>>,
    _workflow_query: Query<(Entity, &WorkflowVisual)>,
) {
    // For now, apply a simple layout to all steps
    // TODO: Implement proper workflow-step relationships

    // Collect all steps first to avoid borrow checker issues
    let all_steps: Vec<(Entity, StepType)> = step_query.iter()
        .map(|(entity, step_visual, _)| (entity, step_visual.step_type.clone()))
        .collect();

    // Apply a default layout if we have steps
    if !all_steps.is_empty() {
        // Simple horizontal layout
        for (i, (entity, _)) in all_steps.iter().enumerate() {
            if let Ok((_, _, mut transform)) = step_query.get_mut(*entity) {
                transform.translation.x = i as f32 * 2.0;
                transform.translation.y = 0.0;
            }
        }
    }
}

/// Apply hierarchical layout to workflow steps
fn apply_hierarchical_layout(
    step_query: &mut Query<(Entity, &WorkflowStepVisual, &mut Transform), Changed<WorkflowStepVisual>>,
    steps: &[(Entity, &WorkflowStepVisual)],
    spacing: f32,
    level_gap: f32,
) {
    // Simple hierarchical layout - arrange by levels
    let mut levels: HashMap<i32, Vec<Entity>> = HashMap::new();

    // Assign levels based on step type (simplified)
    for (entity, step_visual) in steps {
        let level = match step_visual.step_type {
            StepType::Start => 0,
            StepType::End => 10,
            _ => 5, // Middle level for now
        };
        levels.entry(level).or_insert_with(Vec::new).push(*entity);
    }

    // Position steps
    for (level, entities) in levels {
        let y = -(level as f32) * level_gap;
        let count = entities.len() as f32;
        let start_x = -(count - 1.0) * spacing * 0.5;

        for (i, entity) in entities.iter().enumerate() {
            if let Ok((_, _, mut transform)) = step_query.get_mut(*entity) {
                transform.translation.x = start_x + (i as f32) * spacing;
                transform.translation.y = y;
            }
        }
    }
}

/// Apply horizontal layout to workflow steps
fn apply_horizontal_layout(
    step_query: &mut Query<(Entity, &WorkflowStepVisual, &mut Transform), Changed<WorkflowStepVisual>>,
    steps: &[(Entity, &WorkflowStepVisual)],
    spacing: f32,
) {
    // Simple left-to-right layout
    for (i, (entity, _)) in steps.iter().enumerate() {
        if let Ok((_, _, mut transform)) = step_query.get_mut(*entity) {
            transform.translation.x = i as f32 * spacing;
            transform.translation.y = 0.0;
        }
    }
}

/// Apply circular layout to workflow steps
fn apply_circular_layout(
    step_query: &mut Query<(Entity, &WorkflowStepVisual, &mut Transform), Changed<WorkflowStepVisual>>,
    steps: &[(Entity, &WorkflowStepVisual)],
    radius: f32,
) {
    let count = steps.len() as f32;
    let angle_step = std::f32::consts::TAU / count;

    for (i, (entity, _)) in steps.iter().enumerate() {
        if let Ok((_, _, mut transform)) = step_query.get_mut(*entity) {
            let angle = i as f32 * angle_step;
            transform.translation.x = angle.cos() * radius;
            transform.translation.y = angle.sin() * radius;
        }
    }
}

/// Calculate hierarchical layout positions
fn calculate_hierarchical_layout(
    children: &Children,
    step_query: &Query<(&WorkflowStepVisual, &mut Transform)>,
    layer_spacing: f32,
    node_spacing: f32,
) -> HashMap<Entity, Vec3> {
    let mut positions = HashMap::new();
    let mut layers: Vec<Vec<Entity>> = Vec::new();
    let mut visited = HashSet::new();

    // Simple BFS to assign layers
    let mut current_layer = vec![];
    for child in children.iter() {
        if let Ok((step_visual, _)) = step_query.get(child) {
            if matches!(step_visual.step_type, StepType::Start) {
                current_layer.push(child);
                visited.insert(child);
            }
        }
    }

    if !current_layer.is_empty() {
        layers.push(current_layer);
    }

    // Assign positions based on layers
    for (layer_idx, layer) in layers.iter().enumerate() {
        let y = -(layer_idx as f32) * layer_spacing;
        let total_width = (layer.len() as f32 - 1.0) * node_spacing;

        for (node_idx, &entity) in layer.iter().enumerate() {
            let x = (node_idx as f32) * node_spacing - total_width / 2.0;
            positions.insert(entity, Vec3::new(x, y, 0.0));
        }
    }

    positions
}

/// Calculate horizontal layout positions
fn calculate_horizontal_layout(
    children: &Children,
    spacing: f32,
) -> HashMap<Entity, Vec3> {
    let mut positions = HashMap::new();

    for (idx, child) in children.iter().enumerate() {
        let x = idx as f32 * spacing;
        positions.insert(child, Vec3::new(x, 0.0, 0.0));
    }

    positions
}

/// Calculate circular layout positions
fn calculate_circular_layout(
    children: &Children,
    radius: f32,
) -> HashMap<Entity, Vec3> {
    let mut positions = HashMap::new();
    let count = children.len() as f32;

    for (idx, child) in children.iter().enumerate() {
        let angle = (idx as f32 / count) * std::f32::consts::TAU;
        let x = angle.cos() * radius;
        let y = angle.sin() * radius;
        positions.insert(child, Vec3::new(x, y, 0.0));
    }

    positions
}

/// System to layout workflow steps
pub fn layout_workflow_steps(
    mut workflow_query: Query<(&WorkflowVisual, &Children)>,
    mut step_query: Query<(&WorkflowStepVisual, &mut Transform)>,
) {
    for (workflow, children) in workflow_query.iter_mut() {
        let layout = &workflow.layout;

        // Collect step positions based on layout algorithm
        let positions = match layout {
            WorkflowLayout::Hierarchical { spacing: node_spacing, level_gap: layer_spacing } => {
                calculate_hierarchical_layout(children, &step_query, *layer_spacing, *node_spacing)
            }
            WorkflowLayout::Horizontal { spacing } => {
                calculate_horizontal_layout(children, *spacing)
            }
            WorkflowLayout::Circular { radius } => {
                calculate_circular_layout(children, *radius)
            }
            WorkflowLayout::ForceDirected { .. } => {
                // Force-directed layout would be more complex
                HashMap::new()
            }
            WorkflowLayout::Grid { .. } => {
                // Grid layout would need more implementation
                HashMap::new()
            }
        };

        // Apply positions to transforms
        for child in children.iter() {
            if let Ok((_, mut transform)) = step_query.get_mut(child) {
                if let Some(position) = positions.get(&child) {
                    transform.translation = *position;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bezier_point_calculation() {
        let p0 = Vec3::new(0.0, 0.0, 0.0);
        let p1 = Vec3::new(0.5, 1.0, 0.0);
        let p2 = Vec3::new(1.0, 0.0, 0.0);

        let mid = bezier_point(p0, p1, p2, 0.5);
        assert!(mid.y > 0.0); // Should be above the baseline
    }
}
