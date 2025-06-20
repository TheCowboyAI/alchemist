//! Information Alchemist - Main Application
//!
//! A graph editor and workflow manager with AI assistance

use bevy::prelude::*;
use ia::{
    graph::GraphState,
    plugins::{AgentIntegrationPlugin, AgentUiPlugin, NatsEventBridgePlugin},
    simple_agent::SimpleAgentPlugin,
    workflow::WorkflowState,
};
use tracing::info;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,cim_agent_alchemist=debug")
        .init();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Information Alchemist".to_string(),
                ..default()
            }),
            ..default()
        }))
        // Add resources
        .init_resource::<GraphState>()
        .init_resource::<WorkflowState>()
        // Add agent plugins
        .add_plugins(SimpleAgentPlugin)
        .add_plugins(AgentUiPlugin)
        .add_plugins(AgentIntegrationPlugin)
        .add_plugins(NatsEventBridgePlugin)
        // Add systems
        .add_systems(Startup, setup)
        .add_systems(Update, (show_help,))
        .run();
}

/// Set up a simple 3D scene with camera and plane
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 10.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            ..default()
        })),
        Transform::default(),
    ));

    // Add a light
    commands.spawn((
        PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Add a camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    info!("Information Alchemist started");
    info!("Press F1 to open the AI Assistant");
    info!("Press H for help");
}

/// Show help text
fn show_help(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::KeyH) {
        info!("=== Information Alchemist Help ===");
        info!("F1 - Open AI Assistant");
        info!("F2 - Ask about current selection");
        info!("F3 - Ask about workflow");
        info!("H - Show this help");
        info!("ESC - Exit");
        info!("================================");
    }
}
