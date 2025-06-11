//! Information Alchemist - Main Application
//!
//! A minimal Bevy shell ready for domain integration

use bevy::prelude::*;
use ia::presentation::NatsPlugin;
use tracing::info;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NatsPlugin::new())
        .add_systems(Startup, setup)
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

    info!("Information Alchemist started with NATS integration");
}
