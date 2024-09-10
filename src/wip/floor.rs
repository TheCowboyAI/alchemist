use bevy::prelude::*;
use bevy::color::palettes::css::*;

pub struct FloorPlugin;

impl Plugin for FloorPlugin {
  fn build(&self, app: &mut App) {
      app.add_systems(Startup, setup_material)
  }
}


pub struct FloorMaterialHandle {
  pub handle: Handle<StandardMaterial>,
}

fn setup_material(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  // Create the StandardMaterial and add it to the assets
  let material_handle = materials.add(StandardMaterial {
      base_color: DARK_GREEN.into(),
      ..default()
  });

  // Insert the handle as a resource
  commands.insert_resource(FloorMaterialHandle {
      handle: material_handle,
  });
}

fn spawn_floor(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  floor_material: Res<FloorMaterialHandle>,
) {
  let floor = PbrBundle {
      mesh: meshes.add(Mesh::from(Plane::default())),
      material: floor_material.handle.clone(),
      ..default()
  };

  commands.spawn(floor);
}
