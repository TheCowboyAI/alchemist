use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    // Camera setup
    commands.spawn(Camera2dBundle::default());

    let num_sections = 5;
    let radius = 200.0;
    let colors = [
        Color::RED, Color::GREEN, Color::BLUE, Color::YELLOW, Color::PINK,
    ];
    let labels = [
        "Section 1", "Section 2", "Section 3", "Section 4", "Section 5",
    ];

    // Add each section of the pie chart
    for i in 0..num_sections {
        let start_angle = (i as f32 / num_sections as f32) * std::f32::consts::PI * 2.0;
        let end_angle = ((i + 1) as f32 / num_sections as f32) * std::f32::consts::PI * 2.0;

        let mesh = create_pie_section(radius, start_angle, end_angle);
        let material = materials.add(colors[i].into());

        // Spawn the pie section entity
        commands.spawn(MaterialMesh2dBundle {
            mesh: meshes.add(mesh).into(),
            material,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        });

        // Spawn text labels at the center of each pie section
        let label_angle = (start_angle + end_angle) / 2.0;
        let label_x = label_angle.cos() * (radius / 2.0);
        let label_y = label_angle.sin() * (radius / 2.0);

        commands.spawn(Text2dBundle {
            text: Text::from_section(
                labels[i],
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_translation(Vec3::new(label_x, label_y, 1.0)),
            ..default()
        });
    }
}

/// Function to create a pie chart section as a mesh
fn create_pie_section(radius: f32, start_angle: f32, end_angle: f32) -> Mesh {
    let mut vertices = vec![[0.0, 0.0, 0.0]]; // Center of the pie section

    // Number of segments for smoothness
    let num_segments = 30;

    for i in 0..=num_segments {
        let t = i as f32 / num_segments as f32;
        let angle = start_angle + t * (end_angle - start_angle);
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        vertices.push([x, y, 0.0]);
    }

    // Triangles to connect center to outer points
    let mut indices = Vec::new();
    for i in 1..vertices.len() - 1 {
        indices.push(0);
        indices.push(i as u32);
        indices.push((i + 1) as u32);
    }

    let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    mesh
}
