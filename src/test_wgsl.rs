use bevy::prelude::*;
use bevy::color::palettes::css::*;

pub struct TestWgslPlugin;

impl Plugin for TestWgslPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_wgsl); // Register the setup system
    }
}

#[derive(Component)]
struct Entity;

fn setup_wgsl(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
){
  // create a shape in  3d
  spawn_entity(commands, meshes, materials);

  // apply the shader
}


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
          transform: Transform::from_xyz(0.0, 0.0, 0.0),
          ..default()
      },
      Entity,
  );

  commands.spawn(entity);
}

