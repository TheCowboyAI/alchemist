//! Test Workflow Designer
//!
//! Simple test to verify workflow designer with egui is working

use bevy::prelude::*;
use ia::presentation::plugins::WorkflowDesignerPlugin;

fn main() {
    println!("Starting Workflow Designer Test...");

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Workflow Designer Test".to_string(),
                resolution: (1024.0, 768.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(WorkflowDesignerPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, check_egui)
        .run();
}

fn setup(mut commands: Commands) {
    println!("Setting up camera...");

    // Simple 2D camera for UI
    commands.spawn(Camera2d::default());
}

fn check_egui(time: Res<Time>) {
    // Log every second to show the app is running
    if time.elapsed_secs() as u32 % 1 == 0 && time.elapsed_secs_f64().fract() < 0.016 {
        println!(
            "Workflow Designer running... Time: {:.1}s",
            time.elapsed_secs()
        );
    }
}
