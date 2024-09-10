use bevy::prelude::*;
use glam::Vec3;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 3000.0,
            ..default()
        },
        transform: Transform::from_xyz(5.0, 8.0, 2.0),
        ..default()
    });

    // Material for the points
    let material = materials.add(Color::rgb(0.3, 0.5, 0.7).into());

    // Create points for the point cloud
    let points = generate_points();

    for point in points {
        // Spawn a small sphere at each point
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.1, // Tiny radius for each point
                subdivisions: 2,
            })),
            material: material.clone(),
            transform: Transform::from_translation(point),
            ..default()
        });
    }
}

// Function to generate Vec3 points
fn generate_points() -> Vec<Vec3> {
    let mut points = Vec::new();
    for i in 0..5 {
        let x = i as f32 * 2.0;
        let y = i as f32 * 2.0;
        let z = i as f32 * 2.0;
        points.push(Vec3::new(x, y, z));
    }
    points
}
