//! Test import functionality by directly triggering the import command

use bevy::prelude::*;
use ia::application::{CommandEvent, EventNotification};
use ia::domain::{
    commands::{Command, GraphCommand, ImportOptions, ImportSource, graph_commands::MergeBehavior},
    events::DomainEvent,
    value_objects::{GraphId, Position3D},
};
use ia::presentation::{components::GraphNode, plugins::GraphEditorPlugin, systems::ImportPlugin};
use std::time::Duration;
use tracing::info;

fn main() {
    // Set headless mode
    unsafe {
        std::env::set_var("BEVY_HEADLESS", "1");
    }

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Import Test".to_string(),
                    mode: bevy::window::WindowMode::Windowed,
                    ..default()
                }),
                ..default()
            }),
            GraphEditorPlugin,
            ImportPlugin,
        ))
        .add_systems(Startup, trigger_import_after_delay)
        .add_systems(Update, (monitor_import_progress, check_nodes_created))
        .run();
}

/// Trigger import after a short delay to ensure systems are initialized
fn trigger_import_after_delay(mut commands: Commands) {
    // Schedule import command to run after 1 second
    commands.spawn(ImportTimer {
        timer: Timer::new(Duration::from_secs(1), TimerMode::Once),
    });
}

#[derive(Component)]
struct ImportTimer {
    timer: Timer,
}

/// Monitor import progress
fn monitor_import_progress(
    mut commands: Commands,
    time: Res<Time>,
    mut timers: Query<(Entity, &mut ImportTimer)>,
    mut command_events: EventWriter<CommandEvent>,
    mut triggered: Local<bool>,
) {
    for (entity, mut timer) in timers.iter_mut() {
        timer.timer.tick(time.delta());

        if timer.timer.finished() && !*triggered {
            info!("Triggering import command...");

            let graph_id = GraphId::new();

            // Send import command
            command_events.write(CommandEvent {
                command: Command::Graph(GraphCommand::ImportGraph {
                    graph_id,
                    source: ImportSource::File {
                        path: "examples/data/sample_graph.json".to_string(),
                    },
                    format: "arrows_app".to_string(),
                    options: ImportOptions {
                        merge_behavior: MergeBehavior::AlwaysCreate,
                        id_prefix: Some("test".to_string()),
                        position_offset: Some(Position3D {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        }),
                        mapping: None,
                        validate: true,
                        max_nodes: Some(1000),
                    },
                }),
            });

            info!("Import command sent for graph: {:?}", graph_id);
            *triggered = true;

            // Remove timer
            commands.entity(entity).despawn();
        }
    }
}

/// Check if nodes were created
fn check_nodes_created(
    nodes: Query<&GraphNode>,
    mut events: EventReader<EventNotification>,
    mut exit_timer: Local<Option<Timer>>,
    time: Res<Time>,
    mut exit: EventWriter<AppExit>,
) {
    // Log all events
    for event in events.read() {
        info!("Event received: {:?}", event.event.event_type());

        if let DomainEvent::Node(_) = &event.event {
            info!("Node event detected!");
        }
    }

    let node_count = nodes.iter().count();
    if node_count > 0 {
        info!("SUCCESS: {} nodes created!", node_count);

        // Exit after showing success
        if exit_timer.is_none() {
            *exit_timer = Some(Timer::new(Duration::from_secs(2), TimerMode::Once));
        }
    }

    // Update exit timer
    if let Some(timer) = exit_timer.as_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            info!("Test complete, exiting...");
            exit.write(AppExit::Success);
        }
    }

    // Also exit after timeout if no nodes created
    if time.elapsed_secs() > 10.0 && node_count == 0 {
        info!("FAILURE: No nodes created after 10 seconds");
        exit.write(AppExit::from_code(1));
    }
}
