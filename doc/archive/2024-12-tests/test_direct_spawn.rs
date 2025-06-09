//! Test direct node spawning

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera, setup_lighting, spawn_test_nodes))
        .add_systems(Update, check_nodes)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn setup_lighting(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
        affects_lightmapped_meshes: false,
    });

    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_test_nodes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("Spawning test nodes...");

    // Spawn a simple sphere at origin
    let sphere_mesh = meshes.add(Sphere::new(0.5));
    let red_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        ..default()
    });

    commands.spawn((
        Mesh3d(sphere_mesh.clone()),
        MeshMaterial3d(red_material),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Name::new("Red Sphere"),
    ));

    // Spawn another sphere
    let blue_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.0, 1.0),
        ..default()
    });

    commands.spawn((
        Mesh3d(sphere_mesh),
        MeshMaterial3d(blue_material),
        Transform::from_xyz(3.0, 0.0, 0.0),
        Name::new("Blue Sphere"),
    ));

    // Spawn a cube for reference
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let green_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.0),
        ..default()
    });

    commands.spawn((
        Mesh3d(cube_mesh),
        MeshMaterial3d(green_material),
        Transform::from_xyz(0.0, 2.0, 0.0),
        Name::new("Green Cube"),
    ));

    println!("Spawned 3 test objects");
}

fn check_nodes(
    query: Query<(&Name, &Transform)>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        println!("\n=== ENTITIES ===");
        for (name, transform) in query.iter() {
            println!("{}: pos=({:.2}, {:.2}, {:.2}), scale=({:.2}, {:.2}, {:.2})",
                name,
                transform.translation.x, transform.translation.y, transform.translation.z,
                transform.scale.x, transform.scale.y, transform.scale.z
            );
        }
    }
}
