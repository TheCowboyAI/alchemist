use bevy::prelude::*;
use bevy::render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_flycam::prelude::*;
use std::f32::consts::PI;

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Node;

#[derive(Component)]
struct Edge;

#[bevy_main]
fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((DefaultPlugins.set(ImagePlugin::default_nearest()), PlayerPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, rotate)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 250000.0,
            ..default()
        },
        transform: Transform::from_xyz(25.0, 28.0, 22.0),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 150000.0,
            ..default()
        },
        transform: Transform::from_xyz(-3.0, 3.0, 3.0),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            illuminance: 1_500.,
            ..default()
        },
        ..default()
    });

    // Material for the points
    let material = materials.add(Color::srgb(0.3, 0.5, 0.7));

    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    // metal material
    let metal_material = materials.add(StandardMaterial {
        base_color: Srgba::hex("#ffd891").unwrap().into(),
        // vary key PBR parameters on a grid of spheres to show the effect
        metallic: 1.0,
        perceptual_roughness: 0.30,
        ..default()
    });


    // Create points for the point cloud
    let points = generate_points();

    for point in points.clone() {
        // Spawn a small sphere at each point
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Sphere::new(0.25)),
                material: metal_material.clone(),
                transform: Transform::from_translation(point)
                .with_rotation(Quat::from_rotation_x(-PI / 4.)),
                ..default()
            },
            Node,
        ));
    }

    connect_points_with_cylinders(points, commands, meshes, materials);
}

fn rotate(mut query: Query<&mut Transform, With<Node>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() / 2.);
    }
}

fn connect_points_with_cylinders(
    points: Vec<Vec3>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cylinder_mesh = meshes.add(Cylinder::new(0.05, 1.0));
    let pipe_material = materials.add(Color::srgb(0.8, 0.1, 0.1)); // Red material for the cylinders
        
    // Loop through all pairs of points to connect them with cylinders
    for i in 0..points.len() {
        for j in i + 1..points.len() {
            let point_a = points[i];
            let point_b = points[j];

            // Midpoint between the two points
            let midpoint = (point_a + point_b) / 2.0;

            // Direction vector from point_a to point_b
            let direction = point_b - point_a;

            // Compute the length (distance) between the points
            let length = direction.length();

            // Create a transform to align the cylinder between point_a and point_b
            let transform = Transform {
                translation: midpoint,
                rotation: Quat::from_rotation_arc(Vec3::Y, direction.normalize()),
                scale: Vec3::new(1.0, length, 1.0), // Scale to match the distance
                ..default()
            };

            // Spawn the cylinder connecting point_a and point_b
            commands.spawn(PbrBundle {
                mesh: cylinder_mesh.clone(),
                material: pipe_material.clone(),
                transform,
                ..default()
            });
        }
    }
}

// Function to generate Vec3 points
fn generate_points() -> Vec<Vec3> {
    let mut points = Vec::new();

    // Create points spaced 2 units apart along the x, y, and z axes centered around 0
    // aka Metatron's Cube if these are unit spheres

    let offsets = [-4.0, -2.0, 0.0, 2.0, 4.0]; // 5 points symmetrically spaced

    for &offset in &offsets {
        // Points along the x-axis, y-axis, and z-axis, centered around 0
        points.push(Vec3::new(offset, 0.0, 0.0)); // Points along the x-axis
        points.push(Vec3::new(0.0, offset, 0.0)); // Points along the y-axis
        points.push(Vec3::new(0.0, 0.0, offset)); // Points along the z-axis
    }

    points
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

