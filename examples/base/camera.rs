use bevy::prelude::*;

pub struct CameraPlugin;

#[derive(Component)]
struct DefaultCamera;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_camera);
    }
}

fn add_camera(mut commands: Commands) {
    let camera = (Camera3dBundle {
        transform: Transform::from_xyz(10.0, 12.0, 16.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },
    DefaultCamera,);
    commands.spawn(camera);
}
