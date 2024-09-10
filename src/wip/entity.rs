use bevy::prelude::*;
use bevy::color::palettes::css::*;
use bevy_third_person_camera::ThirdPersonCameraTarget;


pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_entity)
            .add_systems(Update, entity_movement);
    }
}

#[derive(Component)]
struct Entity;

#[derive(Component)]
struct Speed(f32);

fn spawn_entity(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = (
        PbrBundle {
            mesh: meshes.add(Sphere::new(1.0)),
            material: materials.add(StandardMaterial {
                base_color: BLUE.into(),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Entity,
        ThirdPersonCameraTarget,
        Speed(2.5),
    );

    commands.spawn(entity);
}

fn entity_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut entity_q: Query<(&mut Transform, &Speed), With<Entity>>,
    cam_q: Query<&Transform, (With<Camera3d>, Without<Entity>)>,
) {
    for (mut entity_transform, entity_speed) in entity_q.iter_mut() {
        let cam = match cam_q.get_single() {
            Ok(c) => c,
            Err(e) => Err(format!("Error retrieving camera: {}", e)).unwrap(),
        };

        let mut direction = Vec3::ZERO;

        // forward
        if keys.pressed(KeyCode::ArrowUp) {
            direction += *cam.forward();
        }

        // back
        if keys.pressed(KeyCode::ArrowDown) {
            direction += *cam.back();
        }

        // left
        if keys.pressed(KeyCode::ArrowLeft) {
            direction += *cam.left();
        }

        // right
        if keys.pressed(KeyCode::ArrowRight) {
            direction += *cam.right();
        }

        // don't move on Y
        direction.y = 0.0;

        // up
        if keys.pressed(KeyCode::KeyA) {
        // don't move on X
        direction.x = 0.0;
            direction += *cam.up();
        }

        // down
        if keys.pressed(KeyCode::KeyZ) {
        // don't move on X
        direction.x = 0.0;
            direction += *cam.down();
        }

        let movement = direction.normalize_or_zero() * entity_speed.0 * time.delta_seconds();
        entity_transform.translation += movement;
    }
}
