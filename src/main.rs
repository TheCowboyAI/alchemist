use bevy::prelude::*;

mod graph;

use graph::GraphPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GraphPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Add a camera positioned to see the graph nodes
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add a light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-0.5)),
    ));

    // Add ambient light for better visibility
    commands.insert_resource(AmbientLight {
        brightness: 500.0,
        ..default()
    });

    // The cube has been removed - graph nodes will be visible instead

    // Add a ground plane for reference
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            ..default()
        })),
        Transform::from_xyz(0.0, -1.0, 0.0).with_scale(Vec3::splat(20.0)),
    ));
}
