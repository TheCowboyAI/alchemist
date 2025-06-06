//! Event-driven animation system for graph visualization

use bevy::prelude::*;
use tracing::info;
use std::time::Duration;

use crate::presentation::components::{GraphNode, GraphEdge, AnimationProgress};
use crate::domain::value_objects::{NodeId, EdgeId, GraphId};
use super::nats_replay::{NatsGraphEvent, RecordedEvent, StartNatsReplay};
use super::force_layout::ForceNode;

/// Event to schedule a command for future execution
#[derive(Event)]
pub struct ScheduledCommand {
    pub delay: Duration,
    pub command: GraphCommand,
}

/// Commands that can be scheduled
#[derive(Clone, Debug)]
pub enum GraphCommand {
    SpawnNode {
        node_id: NodeId,
        position: Vec3,
        label: String,
    },
    SpawnEdge {
        edge_id: EdgeId,
        source: Entity,
        target: Entity,
    },
    UpdateAnimation {
        entity: Entity,
        progress: f32,
    },
}

/// Component to track scheduled commands
#[derive(Component)]
pub struct ScheduledCommandTimer {
    pub timer: Timer,
    pub command: GraphCommand,
}

/// System to process scheduled commands
pub fn process_scheduled_commands(
    mut commands: Commands,
    time: Res<Time>,
    mut scheduled: Query<(Entity, &mut ScheduledCommandTimer)>,
    mut event_writer: EventWriter<ExecuteGraphCommand>,
) {
    for (entity, mut scheduled_cmd) in scheduled.iter_mut() {
        scheduled_cmd.timer.tick(time.delta());

        if scheduled_cmd.timer.finished() {
            // Send command for execution
            event_writer.write(ExecuteGraphCommand(scheduled_cmd.command.clone()));

            // Remove the scheduled command
            commands.entity(entity).despawn();
        }
    }
}

/// Event to execute a graph command immediately
#[derive(Event)]
pub struct ExecuteGraphCommand(pub GraphCommand);

/// System to execute graph commands and record them
pub fn execute_graph_commands(
    mut commands: Commands,
    mut events: EventReader<ExecuteGraphCommand>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.read() {
        match &event.0 {
            GraphCommand::SpawnNode { node_id, position, label } => {
                info!("Spawning node: {} at {:?}", label, position);

                // Record as NATS event
                commands.spawn(RecordedEvent(NatsGraphEvent::NodeAdded {
                    node_id: node_id.to_string(),
                    position: position.to_array(),
                    label: label.clone(),
                }));

                commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.5))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.3, 0.5, 0.8),
                        metallic: 0.8,
                        perceptual_roughness: 0.2,
                        ..default()
                    })),
                    Transform::from_translation(*position)
                        .with_scale(Vec3::ZERO), // Start at zero scale
                    GraphNode {
                        node_id: *node_id,
                        graph_id: GraphId::new(),
                    },
                    AnimationProgress(0.0),
                    Name::new(label.clone()),
                    ForceNode::default(),
                ));
            }

            GraphCommand::SpawnEdge { edge_id, source, target } => {
                info!("Spawning edge from {:?} to {:?}", source, target);

                // Record as NATS event (need to get node IDs)
                commands.spawn(RecordedEvent(NatsGraphEvent::EdgeAdded {
                    edge_id: edge_id.to_string(),
                    source_id: format!("{source:?}"), // Simplified for demo
                    target_id: format!("{target:?}"),
                }));

                commands.spawn((
                    GraphEdge {
                        edge_id: *edge_id,
                        graph_id: GraphId::new(),
                        source: *source,
                        target: *target,
                    },
                    AnimationProgress(0.0),
                    Name::new(format!("Edge_{edge_id:?}")),
                ));
            }

            GraphCommand::UpdateAnimation { entity, progress } => {
                // Record animation progress
                commands.spawn(RecordedEvent(NatsGraphEvent::AnimationProgress {
                    entity_type: "unknown".to_string(),
                    entity_id: format!("{entity:?}"),
                    progress: *progress,
                }));

                commands.entity(*entity).insert(AnimationProgress(*progress));
            }
        }
    }
}

/// System to handle scheduled command events
pub fn handle_scheduled_commands(
    mut commands: Commands,
    mut events: EventReader<ScheduledCommand>,
) {
    for event in events.read() {
        commands.spawn(ScheduledCommandTimer {
            timer: Timer::new(event.delay, TimerMode::Once),
            command: event.command.clone(),
        });
    }
}

// Type alias to reduce complexity
type AnimatedNodeQuery<'w, 's> = Query<'w, 's, (&'static AnimationProgress, &'static mut Transform), (With<GraphNode>, Without<GraphEdge>)>;

/// System to animate nodes and edges based on AnimationProgress
pub fn animate_graph_elements(
    mut nodes: AnimatedNodeQuery,
) {
    for (progress, mut transform) in nodes.iter_mut() {
        if progress.0 < 1.0 {
            // Smooth ease-out animation
            let t = ease_out_cubic(progress.0);
            transform.scale = Vec3::splat(t);
        }
    }
}

/// Ease-out cubic function for smooth animation
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// System to update animation progress over time
pub fn update_animation_progress(
    mut query: Query<(Entity, &mut AnimationProgress)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let delta = time.delta_secs() * 2.0; // Animation speed

    for (entity, mut progress) in query.iter_mut() {
        if progress.0 < 1.0 {
            progress.0 = (progress.0 + delta).min(1.0);

            // Record progress updates periodically
            if (progress.0 * 10.0) as i32 % 2 == 0 {
                commands.spawn(RecordedEvent(NatsGraphEvent::AnimationProgress {
                    entity_type: "graph_element".to_string(),
                    entity_id: format!("{entity:?}"),
                    progress: progress.0,
                }));
            }
        }
    }
}

/// System to handle keyboard input for NATS replay
pub fn handle_nats_replay_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut replay_events: EventWriter<StartNatsReplay>,
) {
    if keyboard.just_pressed(KeyCode::KeyN) {
        info!("N key pressed - Starting NATS replay");
        replay_events.write(StartNatsReplay);
    }
}
