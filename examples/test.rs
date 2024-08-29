use bevy::prelude::*;

// Constants
const MAX_SPHERES: usize = 13;
const TARGET_RADIUS: f32 = 1.0;
const GROWTH_RATE: f32 = 0.01;

// SphereComponent to store size and growth state
#[derive(Component)]
struct SphereComponent {
    radius: f32,
}

// AxisAssignment keeps track of which axis the sphere is on
#[derive(Component)]
struct AxisAssignment {
    axis: Vec3,
}

// Bevy application entry point
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (check_spheres, update_spheres))
        .run();
}

// Initial setup for the scene
fn setup(mut commands: Commands) {
    // Spawn light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(10.0, 10.0, 10.0),
        ..Default::default()
    });

    // Spawn camera looking at origin
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn spawn_sphere() {
    // Spawn the sphere with a starting radius of 0 at 0
    commands.spawn((
      PbrBundle {
          mesh: meshes.add(Mesh::from(Sphere { radius: 0.0, ..Default::default() })),
          material: materials.add(StandardMaterial {
              base_color: Srgba::BLUE.into(), // Blue color
              ..Default::default()
          }),
          ..Default::default()
      },
      SphereComponent {
          radius: 0.1,
      }))  
}

// System to spawn spheres
fn check_spheres(
    mut commands: Commands,
    query: Query<&SphereComponent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // when to stop
    if query.iter().len() >= MAX_SPHERES {
        return;
    }

    if query.iter().len() == 0 {
        commands.spawn_sphere();
        return; 
    }
}

// System to update and grow spheres over time
fn update_spheres_system(mut query: Query<(&mut SphereComponent, &mut Transform)>) {
    for (mut sphere, mut transform) in query.iter_mut() {
        if sphere.growing {
            sphere.radius += GROWTH_RATE;
            if sphere.radius >= TARGET_RADIUS {
                sphere.radius = TARGET_RADIUS;
                sphere.growing = false;
            }
            transform.scale = Vec3::splat(sphere.radius);
        }
    }
}
