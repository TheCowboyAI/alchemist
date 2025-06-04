//! Bevy Plugins for the Presentation Layer

use bevy::prelude::*;
use crate::application::{CommandEvent, EventNotification};
use crate::application::command_handlers::process_commands;
use tracing::info;

/// Main plugin for the graph editor
pub struct GraphEditorPlugin;

impl Plugin for GraphEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register events
            .add_event::<CommandEvent>()
            .add_event::<EventNotification>()

            // Add systems
            .add_systems(Startup, setup_camera)
            .add_systems(Update, (
                // Command processing
                process_commands,

                // Event handling
                handle_domain_events,
            ));
    }
}

/// Setup basic 3D camera
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Handle domain events and update the world
fn handle_domain_events(
    mut events: EventReader<EventNotification>,
) {
    for event in events.read() {
        info!("Received domain event: {:?}", event.event);
        // TODO: Update ECS entities based on domain events
    }
}
