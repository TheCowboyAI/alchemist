//! Graph visualization plugin for Bevy

use bevy::prelude::*;

use crate::presentation::bevy_systems::{
    setup_3d_scene,
    create_demo_graph,
    draw_edges,
    apply_force_directed_layout,
    process_scheduled_commands,
    handle_scheduled_commands,
    execute_graph_commands,
    animate_graph_elements,
    update_animation_progress,
    handle_nats_replay_input,
    ExecuteGraphCommand,
    ScheduledCommand,
    NatsReplayPlugin,
    ForceLayoutSettings,
};

pub struct GraphVisualizationPlugin;

impl Plugin for GraphVisualizationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add NATS replay functionality
            .add_plugins(NatsReplayPlugin)
            // Resources
            .insert_resource(ForceLayoutSettings::default())
            // Events
            .add_event::<ScheduledCommand>()
            .add_event::<ExecuteGraphCommand>()
            // Startup systems
            .add_systems(Startup, (setup_3d_scene, create_demo_graph))
            // Update systems
            .add_systems(Update, (
                // Event processing
                handle_scheduled_commands,
                process_scheduled_commands,
                execute_graph_commands,
                // Animation
                animate_graph_elements,
                update_animation_progress,
                // Rendering
                draw_edges,
                // Physics
                apply_force_directed_layout,
                // Input
                handle_nats_replay_input,
            ).chain());
    }
}
