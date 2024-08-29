use bevy::prelude::*;
use bevy::pbr::{wireframe, AmbientLight};
use bevy::render::mesh::Mesh;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::pbr::wireframe::WireframePlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, WireframePlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Central sphere material using srgb color
    let sphere_material = materials.add(StandardMaterial {
        base_color: bevy::color::palettes::basic::PURPLE.into(),
        ..Default::default()
    });

    // Line material (white)
    let line_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..Default::default()
    });

    // Sphere mesh generated using Bevy's UVSphere
    let sphere_mesh = meshes.add(Mesh::from(Sphere {
        radius: 1.0,
    }));

    // Store sphere positions to later connect them with lines
    let mut sphere_positions = Vec::new();

    // Central sphere at origin
    let central_position = Vec3::new(0.0, 0.0, 0.0);
    commands.spawn(PbrBundle {
        mesh: sphere_mesh.clone(),
        material: sphere_material.clone(),
        transform: Transform::from_translation(central_position),
        ..Default::default()
    });
    sphere_positions.push(central_position);

    // Iterate from -3 to 3 to place spheres along x, y, z axes
    for i in -2..=2 {
        if i == 0 {
            continue; // Skip the origin as we already added it
        }

        // Spheres along the x-axis
        let position_x = Vec3::new(i as f32 * 2.0, 0.0, 0.0);
        commands.spawn(PbrBundle {
            mesh: sphere_mesh.clone(),
            material: sphere_material.clone(),
            transform: Transform::from_translation(position_x),
            ..Default::default()
        });
        sphere_positions.push(position_x);

        // Spheres along the y-axis
        let position_y = Vec3::new(0.0, i as f32 * 2.0, 0.0);
        commands.spawn(PbrBundle {
            mesh: sphere_mesh.clone(),
            material: sphere_material.clone(),
            transform: Transform::from_translation(position_y),
            ..Default::default()
        });
        sphere_positions.push(position_y);

        // Spheres along the z-axis
        let position_z = Vec3::new(0.0, 0.0, i as f32 * 2.0);
        commands.spawn(PbrBundle {
            mesh: sphere_mesh.clone(),
            material: sphere_material.clone(),
            transform: Transform::from_translation(position_z),
            ..Default::default()
        });
        sphere_positions.push(position_z);
    }

    // Generate lines between sphere positions
    for i in 0..sphere_positions.len() {
        for j in i + 1..sphere_positions.len() {
            let start = sphere_positions[i];
            let end = sphere_positions[j];

            let line_mesh = generate_line_mesh(start, end);
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(line_mesh),
                    material: line_material.clone(),
                    transform: Transform::default(),
                    ..Default::default()
                },
                wireframe::Wireframe, // Add wireframe component to the line mesh
            ));
        }
    }

    // Add a light source
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 5000.0,
            range: 100.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..Default::default()
    });

    // Add a camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(20.0, 20.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    // Add some ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.4,
    });
}

fn generate_line_mesh(start: Vec3, end: Vec3) -> Mesh {
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::LineList, RenderAssetUsages::empty());

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![start.to_array(), end.to_array()]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 0.0]; 2]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 2]);

    mesh
}
