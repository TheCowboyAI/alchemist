use bevy::prelude::*;
use bevy::color::Srgba;
//use bevy_render::render_asset::RenderAssetUsages;
use crate::chart::Triangle3d;

use serde::*;
use serde_yaml::*;

pub struct ChartPlugin;

impl Plugin for ChartPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_chart); // Register the setup system
    }
}

#[derive(Component, Serialize, Deserialize, Debug)]
pub struct Sectors {
    pub value: f32,            // Percentage value for the sector (0 to 1)
    pub height: Option<f32>,   // Optional height for the sector (Z-axis)
    pub color: Srgba,          // Color for the sector
    pub label: Option<String>, // Optional label for the sector
}

#[derive(Component, Serialize, Deserialize, Debug)]
pub struct Chart {
    pub radius: f32,           // Radius of the chart cylinder
    pub label: Option<String>, // Optional label for the whole chart
    pub sectors: Vec<Sectors>, // List of sectors
}

fn setup_chart(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Load the YAML chart data
    let yaml_path = "data/3d_pie_chart.yaml";
    let yaml_content = std::fs::read_to_string(yaml_path).expect("Failed to load the YAML file.");
    
    // Parse the YAML file into chart structure
    let chart_data: Chart = parse_yaml_to_chart(yaml_content).expect("Failed to parse YAML.");

    // Generate the pie chart with the parsed data
    spawn_chart(commands, meshes, materials, asset_server, chart_data);
}

fn parse_yaml_to_chart(yaml_content: String) -> Result<Chart> {
    serde_yaml::from_str::<Chart>(&yaml_content)
}

fn spawn_chart(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    chart: Chart,
) {
    for sector in chart.sectors.iter() {
        let sector_mesh = create_cylinder_slice(chart.radius, sector.value, sector.height.unwrap_or(10.0));

        let material = materials.add(StandardMaterial {
            base_color: sector.color.into(),
            unlit: true,
            ..default()
        });

        // Spawn the sector
        commands.spawn(PbrBundle {
            mesh: meshes.add(sector_mesh),
            material,
            transform: Transform::default(),
            ..default()
        });

        if let Some(label) = &sector.label {
            let label_position = calculate_label_position(sector.value, chart.radius);
            spawn_sector_label(commands.reborrow(), label.clone(), &asset_server, label_position);
        }
    }

    // Add a label for the chart itself
    if let Some(chart_label) = &chart.label {
        let label_position = Vec3::new(0.0, chart.radius * 1.1, 0.0);
        spawn_chart_label(commands, chart_label.clone(), &asset_server, label_position);
    }
}

fn create_cylinder_slice(radius: f32, value: f32, height: f32) -> Mesh {
    let angle = value * std::f32::consts::PI * 2.0;
    let num_segments = 30;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=num_segments {
        let t = i as f32 / num_segments as f32;
        let current_angle = t * angle;
        let x = radius * current_angle.cos();
        let y = radius * current_angle.sin();

        vertices.push([x, y, 0.0]);
        vertices.push([x, y, height]);
    }

    for i in 0..num_segments {
        let bottom_start = i * 2;
        let top_start = bottom_start + 1;
        let bottom_next = (i + 1) * 2;
        let top_next = bottom_next + 1;

        indices.push(bottom_start);
        indices.push(bottom_next);
        indices.push(top_start);

        indices.push(top_start);
        indices.push(bottom_next);
        indices.push(top_next);
    }

    let mut mesh = Mesh::new(Triangle3d::new(1,), RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.set_indices(Some(mesh::Indices::U32(indices)));

    mesh
}

fn calculate_label_position(value: f32, radius: f32) -> Vec3 {
    let angle = value * std::f32::consts::PI;
    let x = angle.cos() * radius * 0.75;
    let y = angle.sin() * radius * 0.75;
    Vec3::new(x, y, 10.0)
}

fn spawn_sector_label(
    mut commands: Commands,
    label: String,
    asset_server: &Res<AssetServer>,
    position: Vec3,
) {
    commands.spawn(TextBundle {
        text: Text::from_section(
            label,
            TextStyle {
                font: asset_server.load("fonts/FiraCodeNerdFont-Regular.ttf"),
                font_size: 48.0,
                color: Color::WHITE,
            },
        ),
        transform: Transform::from_xyz(position.x, position.y, position.z),
        ..default()
    });
}

fn spawn_chart_label(
    mut commands: Commands,
    label: String,
    asset_server: &Res<AssetServer>,
    position: Vec3,
) {
    commands.spawn(TextBundle {
        text: Text::from_section(
            label,
            TextStyle {
                font: asset_server.load("fonts/FiraCodeNerdFont-Regular.ttf"),
                font_size: 72.0,
                color: Color::WHITE,
            },
        ),
        transform: Transform::from_xyz(position.x, position.y, position.z),
        ..default()
    });
}
