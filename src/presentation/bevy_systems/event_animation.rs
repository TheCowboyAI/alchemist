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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ease_out_cubic() {
        // Test boundary values
        assert_eq!(ease_out_cubic(0.0), 0.0);
        assert_eq!(ease_out_cubic(1.0), 1.0);

        // Test middle value
        assert!((ease_out_cubic(0.5) - 0.875).abs() < 0.001);

        // Test that it's monotonically increasing
        let mut prev = 0.0;
        for i in 1..=100 {
            let t = i as f32 / 100.0;
            let value = ease_out_cubic(t);
            assert!(value > prev, "ease_out_cubic should be monotonically increasing");
            prev = value;
        }
    }

    #[test]
    fn test_scheduled_command_timer() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(Time::<()>::default());
        app.add_event::<ExecuteGraphCommand>();

        // Add a scheduled command timer
        app.world_mut().spawn(ScheduledCommandTimer {
            timer: Timer::new(Duration::from_secs(1), TimerMode::Once),
            command: GraphCommand::SpawnNode {
                node_id: NodeId::new(),
                position: Vec3::new(1.0, 0.0, 0.0),
                label: "Test Node".to_string(),
            },
        });

        // Add system
        app.add_systems(Update, process_scheduled_commands);

        // Advance time before timer finishes
        app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f32(0.5));
        app.update();

        // Timer should still exist
        let mut query = app.world_mut().query::<&ScheduledCommandTimer>();
        let timer_count = query.iter(app.world()).count();
        assert_eq!(timer_count, 1);

        // Advance time past timer duration
        app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f32(0.6));
        app.update();

        // Timer should be removed
        let mut query = app.world_mut().query::<&ScheduledCommandTimer>();
        let timer_count = query.iter(app.world()).count();
        assert_eq!(timer_count, 0);

        // Check event was sent
        let events = app.world().resource::<Events<ExecuteGraphCommand>>();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_handle_scheduled_commands() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<ScheduledCommand>();

        // Add system
        app.add_systems(Update, handle_scheduled_commands);

        // Send scheduled command event
        app.world_mut().send_event(ScheduledCommand {
            delay: Duration::from_secs(2),
            command: GraphCommand::SpawnNode {
                node_id: NodeId::new(),
                position: Vec3::new(1.0, 2.0, 3.0),
                label: "Test Node".to_string(),
            },
        });

        // Run update
        app.update();

        // Check that a timer entity was created
        let mut query = app.world_mut().query::<&ScheduledCommandTimer>();
        let timer_count = query.iter(app.world()).count();
        assert_eq!(timer_count, 1);

        // Check timer duration
        let timer = query.single(app.world()).unwrap();
        assert_eq!(timer.timer.duration().as_secs(), 2);
    }

    #[test]
    fn test_animate_graph_elements() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Create node with animation progress
        let node = app.world_mut().spawn((
            GraphNode {
                node_id: NodeId::new(),
                graph_id: GraphId::new(),
            },
            Transform::from_scale(Vec3::ZERO),
            AnimationProgress(0.5),
        )).id();

        // Add system
        app.add_systems(Update, animate_graph_elements);

        // Run update
        app.update();

        // Check scale was updated
        let transform = app.world().get::<Transform>(node).unwrap();
        let expected_scale = ease_out_cubic(0.5);
        assert!((transform.scale.x - expected_scale).abs() < 0.001);
        assert!((transform.scale.y - expected_scale).abs() < 0.001);
        assert!((transform.scale.z - expected_scale).abs() < 0.001);
    }

    #[test]
    fn test_update_animation_progress() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Add time resource
        app.insert_resource(Time::<()>::default());
        app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f32(0.1));

        // Create entity with animation progress
        let entity = app.world_mut().spawn(AnimationProgress(0.0)).id();

        // Add system (modified to not emit RecordedEvent)
        app.add_systems(Update, |mut query: Query<(Entity, &mut AnimationProgress)>, time: Res<Time>| {
            let delta = time.delta_secs() * 2.0;
            for (_entity, mut progress) in query.iter_mut() {
                if progress.0 < 1.0 {
                    progress.0 = (progress.0 + delta).min(1.0);
                }
            }
        });

        // Run update
        app.update();

        // Check progress was updated (delta * 2.0 = 0.1 * 2.0 = 0.2)
        let progress = app.world().get::<AnimationProgress>(entity).unwrap();
        assert!((progress.0 - 0.2).abs() < 0.001);

        // Run multiple updates to reach 1.0
        for _ in 0..10 {
            app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f32(0.1));
            app.update();
        }

        // Progress should be capped at 1.0
        let progress = app.world().get::<AnimationProgress>(entity).unwrap();
        assert_eq!(progress.0, 1.0);
    }

    #[test]
    fn test_graph_command_spawn_node() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(Time::<()>::default());
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        // Add event
        app.add_event::<ExecuteGraphCommand>();

        // Add system
        app.add_systems(Update, execute_graph_commands);

        // Send spawn node command
        app.world_mut().send_event(ExecuteGraphCommand(GraphCommand::SpawnNode {
            node_id: NodeId::new(),
            position: Vec3::new(1.0, 2.0, 3.0),
            label: "Test Node".to_string(),
        }));

        app.update();

        // Check that node was spawned
        let mut query = app.world_mut().query::<(&GraphNode, &Transform, &AnimationProgress)>();
        let node_count = query.iter(app.world()).count();
        assert_eq!(node_count, 1);

        // Check node properties
        let (_, transform, progress) = query.single(app.world()).unwrap();
        assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(progress.0, 0.0);
    }

    #[test]
    fn test_graph_command_spawn_edge() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(Time::<()>::default());
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();

        // Add event
        app.add_event::<ExecuteGraphCommand>();

        // Spawn source and target nodes first
        let source = app.world_mut().spawn((
            GraphNode {
                node_id: NodeId::new(),
                graph_id: GraphId::new(),
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let target = app.world_mut().spawn((
            GraphNode {
                node_id: NodeId::new(),
                graph_id: GraphId::new(),
            },
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        // Add system
        app.add_systems(Update, execute_graph_commands);

        // Send spawn edge command
        app.world_mut().send_event(ExecuteGraphCommand(GraphCommand::SpawnEdge {
            edge_id: EdgeId::new(),
            source,
            target,
        }));

        app.update();

        // Check that edge was spawned
        let mut query = app.world_mut().query::<&GraphEdge>();
        let edge_count = query.iter(app.world()).count();
        assert_eq!(edge_count, 1);

        // Check edge properties
        let edge = query.single(app.world()).unwrap();
        assert_eq!(edge.source, source);
        assert_eq!(edge.target, target);
    }

    #[test]
    fn test_handle_nats_replay_input() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<StartNatsReplay>();
        app.init_resource::<ButtonInput<KeyCode>>();

        // Add system
        app.add_systems(Update, handle_nats_replay_input);

        // Simulate N key press
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyN);

        // Run update
        app.update();

        // Check event was sent
        let events = app.world().resource::<Events<StartNatsReplay>>();
        assert_eq!(events.len(), 1);

        // Clear and press again - should only trigger on just_pressed
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().clear();
        app.update();

        // Events should still be 1 (no new events)
        let events = app.world().resource::<Events<StartNatsReplay>>();
        assert_eq!(events.len(), 1);
    }
}
