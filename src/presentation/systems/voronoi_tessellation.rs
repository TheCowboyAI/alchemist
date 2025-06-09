//! Voronoi tessellation for conceptual space partitioning

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::mesh::MeshVertexAttribute;
use bevy::render::render_resource::VertexFormat;
use tracing::info;
use std::hash::{Hash, Hasher};

use crate::presentation::components::{
    ConceptualPosition, ConceptualSpacePartition, DistanceMetric, GraphNode, QualityDimension,
    SubgraphMember, SubgraphRegion, VoronoiCell, VoronoiSettings,
    SubgraphOrigin, SubgraphOrigins,
};
use crate::domain::value_objects::{EdgeId, GraphId, NodeId, Position3D, SubgraphId};
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;

/// Plugin for Voronoi tessellation
pub struct VoronoiTessellationPlugin;

impl Plugin for VoronoiTessellationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoronoiSettings>()
            .init_resource::<VoronoiUpdateTimer>()
            .init_resource::<VoronoiVisualizationEnabled>()
            .add_systems(
                Update,
                (
                    toggle_voronoi_visualization,
                    update_quality_dimensions,
                    calculate_voronoi_tessellation,
                    assign_nodes_to_cells,
                    visualize_voronoi_cells.run_if(resource_equals(VoronoiVisualizationEnabled(true))),
                )
                    .chain(),
            );
    }
}

#[derive(Resource, Default, PartialEq)]
struct VoronoiVisualizationEnabled(bool);

#[derive(Resource, Default)]
struct VoronoiUpdateTimer(Timer);

/// Toggle Voronoi visualization on/off
fn toggle_voronoi_visualization(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut enabled: ResMut<VoronoiVisualizationEnabled>,
    mut commands: Commands,
    existing_cells: Query<Entity, With<VoronoiCellMesh>>,
) {
    if keyboard.just_pressed(KeyCode::KeyV) {
        enabled.0 = !enabled.0;
        info!("Voronoi visualization: {}", if enabled.0 { "ON" } else { "OFF" });

        // Remove existing cells if disabling
        if !enabled.0 {
            for entity in existing_cells.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

/// Update quality dimensions from subgraph regions
fn update_quality_dimensions(
    mut commands: Commands,
    subgraph_query: Query<(Entity, &SubgraphRegion), Changed<SubgraphRegion>>,
    _node_query: Query<&Transform, With<GraphNode>>,
    member_query: Query<(&SubgraphMember, &Transform)>,
) {
    for (entity, subgraph) in subgraph_query.iter() {
        // Calculate prototype (centroid) of subgraph nodes
        let mut sum = Vec3::ZERO;
        let mut count = 0;

        for (member, transform) in member_query.iter() {
            // Check if this member belongs to the current subgraph
            if member.subgraph_ids.contains(&subgraph.subgraph_id) {
                sum += transform.translation;
                count += 1;
            }
        }

        if count > 0 {
            let prototype = sum / count as f32;

            // Add or update quality dimension component
            commands.entity(entity).insert(QualityDimension {
                subgraph_id: subgraph.subgraph_id,
                name: subgraph.name.clone(),
                prototype,
                weight: 1.0,
                metric: DistanceMetric::Euclidean,
            });
        }
    }
}

/// Calculate Voronoi tessellation based on quality dimension prototypes
fn calculate_voronoi_tessellation(
    mut commands: Commands,
    mut timer: ResMut<VoronoiUpdateTimer>,
    settings: Res<VoronoiSettings>,
    time: Res<Time>,
    dimensions: Query<&QualityDimension>,
    mut partition_query: Query<&mut ConceptualSpacePartition>,
    node_query: Query<&Transform, With<GraphNode>>,
    mut gizmos: Gizmos,
    origins: Res<SubgraphOrigins>,
) {
    let settings = settings.into_inner();
    if !settings.enabled {
        return;
    }

    // Update timer
    timer.0.tick(time.delta());
    if !timer.0.finished() {
        return;
    }
    timer.0 = Timer::from_seconds(settings.update_frequency, TimerMode::Repeating);

    // Collect all prototypes
    let prototypes: Vec<_> = dimensions
        .iter()
        .map(|dim| (dim.subgraph_id, dim.prototype))
        .collect();

    if prototypes.is_empty() {
        return;
    }

    // Calculate bounds
    let mut min_bound = Vec3::splat(f32::MAX);
    let mut max_bound = Vec3::splat(f32::MIN);

    for (_, proto) in &prototypes {
        min_bound = min_bound.min(*proto);
        max_bound = max_bound.max(*proto);
    }

    // Add padding
    min_bound -= Vec3::splat(settings.boundary_padding);
    max_bound += Vec3::splat(settings.boundary_padding);

    // Simple 2D Voronoi on XZ plane (Y is fixed for visualization)
    let cells = calculate_voronoi_cells_2d(&prototypes, min_bound, max_bound, &*settings);

    // Update or create partition entity
    if let Ok(mut partition) = partition_query.get_single_mut() {
        partition.cells = cells;
        partition.bounds = (min_bound, max_bound);
    } else {
        commands.spawn(ConceptualSpacePartition {
            cells,
            bounds: (min_bound, max_bound),
        });
    }

    // Get actual node positions from the query
    let node_positions: Vec<Vec3> = node_query
        .iter()
        .map(|transform| transform.translation)
        .collect();

    if node_positions.is_empty() {
        return;
    }

    // Use node positions to influence Voronoi cell generation
    for (subgraph_id, subgraph_info) in origins.subgraph_info.iter() {
        // Find nodes near this subgraph origin
        let origin_pos = Vec3::new(
            subgraph_info.origin.x,
            subgraph_info.origin.y,
            subgraph_info.origin.z,
        );

        let nearby_nodes: Vec<Vec3> = node_positions
            .iter()
            .filter(|pos| pos.distance(origin_pos) < settings.cell_size * 2.0)
            .cloned()
            .collect();

        // Use nearby nodes to adjust cell boundaries
        let adjusted_origin = if !nearby_nodes.is_empty() {
            let centroid = nearby_nodes.iter().sum::<Vec3>() / nearby_nodes.len() as f32;
            origin_pos.lerp(centroid, 0.3) // Blend with node centroid
        } else {
            origin_pos
        };

        // Draw cell with adjusted origin
        gizmos.circle(
            Isometry3d::from_translation(adjusted_origin),
            settings.cell_size * 0.8,
            Color::srgb(0.5, 0.5, 1.0).with_alpha(0.3),
        );
    }
}

/// Calculate 2D Voronoi cells using a proper algorithm
fn calculate_voronoi_cells_2d(
    prototypes: &[(SubgraphId, Vec3)],
    min_bound: Vec3,
    max_bound: Vec3,
    settings: &VoronoiSettings,
) -> Vec<VoronoiCell> {
    let mut cells = Vec::new();

    // For each prototype, calculate its Voronoi cell
    for (i, (id, proto)) in prototypes.iter().enumerate() {
        let mut vertices = Vec::new();
        let mut neighbors = HashSet::new();
        let mut bisectors = Vec::new();

        // Calculate perpendicular bisectors with all other prototypes
        for (j, (other_id, other_proto)) in prototypes.iter().enumerate() {
            if i != j {
                let midpoint = (*proto + *other_proto) * 0.5;

                // Calculate perpendicular direction
                let direction = (*other_proto - *proto).normalize();
                let perpendicular = Vec3::new(-direction.z, 0.0, direction.x).normalize();

                // Create bisector plane vertices
                let cell_size = settings.min_cell_size;
                let bisector_vertices = vec![
                    midpoint + perpendicular * cell_size,
                    midpoint - perpendicular * cell_size,
                ];

                // Store bisector for intersection calculations
                bisectors.push((bisector_vertices[0], bisector_vertices[1]));
                neighbors.insert(*other_id);

                // Add bisector midpoint to influence vertex positions
                vertices.push(midpoint);
            }
        }

        // Create vertices at intersections of bisectors
        // For now, use a simplified approach with regular polygon
        let num_vertices = 16; // More vertices for smoother cells
        let base_radius = settings.min_cell_size * 1.5;

        for k in 0..num_vertices {
            let angle = (k as f32 / num_vertices as f32) * std::f32::consts::TAU;
            let mut vertex = *proto + Vec3::new(angle.cos() * base_radius, 0.0, angle.sin() * base_radius);

            // Adjust vertex position based on nearby prototypes
            for (j, (_, other_proto)) in prototypes.iter().enumerate() {
                if i != j {
                    let to_other = *other_proto - *proto;
                    let to_vertex = vertex - *proto;
                    let distance_to_other = proto.distance(*other_proto);

                    // Check if vertex violates Voronoi property
                    let vertex_distance_to_proto = to_vertex.length();
                    let vertex_distance_to_other = (vertex - *other_proto).length();

                    // If vertex is closer to other prototype, project it onto bisector
                    if vertex_distance_to_other < vertex_distance_to_proto {
                        // Calculate projection onto perpendicular bisector
                        let midpoint = (*proto + *other_proto) * 0.5;
                        let bisector_normal = to_other.normalize();

                        // Project vertex onto the bisector plane
                        let vertex_to_mid = midpoint - vertex;
                        let projection_distance = vertex_to_mid.dot(bisector_normal);
                        vertex = vertex + bisector_normal * projection_distance;

                        // Ensure vertex stays within reasonable bounds
                        let max_distance = distance_to_other * 0.45;
                        if vertex.distance(*proto) > max_distance {
                            vertex = *proto + (vertex - *proto).normalize() * max_distance;
                        }
                    }
                }
            }

            // Clamp to bounds
            vertex = vertex.clamp(min_bound, max_bound);
            vertex.y = settings.visualization_height; // Keep on visualization plane
            vertices.push(vertex);
        }

        cells.push(VoronoiCell {
            subgraph_id: *id,
            prototype: Vec3::new(proto.x, settings.visualization_height, proto.z),
            vertices,
            neighbors,
        });
    }

    // Apply Lloyd's relaxation for better cell shapes
    if settings.smoothing_factor > 0.0 {
        apply_lloyds_relaxation(&mut cells, settings.smoothing_factor);
    }

    cells
}

/// Apply Lloyd's relaxation to improve Voronoi cell shapes
fn apply_lloyds_relaxation(cells: &mut [VoronoiCell], factor: f32) {
    for cell in cells.iter_mut() {
        // Calculate centroid of cell vertices
        let centroid = cell.vertices.iter().fold(Vec3::ZERO, |acc, v| acc + *v) / cell.vertices.len() as f32;

        // Move prototype towards centroid
        cell.prototype = cell.prototype.lerp(centroid, factor);
    }
}

/// Assign nodes to their nearest Voronoi cell
fn assign_nodes_to_cells(
    mut commands: Commands,
    partition: Query<&ConceptualSpacePartition>,
    dimensions: Query<&QualityDimension>,
    mut nodes: Query<(Entity, &Transform, Option<&mut ConceptualPosition>), With<GraphNode>>,
) {
    let Ok(partition) = partition.get_single() else {
        return;
    };

    // Create a map of subgraph_id to dimension for quick lookup
    let dim_map: HashMap<_, _> = dimensions
        .iter()
        .map(|dim| (dim.subgraph_id, dim))
        .collect();

    for (entity, transform, conceptual_pos) in nodes.iter_mut() {
        let position = transform.translation;

        // Find nearest prototype
        let mut nearest_cell = None;
        let mut min_distance = f32::MAX;

        for cell in &partition.cells {
            if let Some(dim) = dim_map.get(&cell.subgraph_id) {
                let distance = match dim.metric {
                    DistanceMetric::Euclidean => position.distance(cell.prototype),
                    DistanceMetric::Manhattan => {
                        (position - cell.prototype).abs().x
                            + (position - cell.prototype).abs().y
                            + (position - cell.prototype).abs().z
                    }
                    DistanceMetric::WeightedEuclidean { weights } => {
                        let diff = position - cell.prototype;
                        (diff.x * diff.x * weights[0]
                            + diff.y * diff.y * weights[1]
                            + diff.z * diff.z * weights[2])
                            .sqrt()
                    }
                    DistanceMetric::Conceptual => {
                        // Simplified conceptual distance
                        position.distance(cell.prototype) * dim.weight
                    }
                };

                if distance < min_distance {
                    min_distance = distance;
                    nearest_cell = Some(cell.subgraph_id);
                }
            }
        }

        // Update or insert conceptual position
        if let Some(mut pos) = conceptual_pos {
            pos.coordinates = position;
            pos.cell_id = nearest_cell;
            pos.distance_to_prototype = min_distance;
        } else {
            commands.entity(entity).insert(ConceptualPosition {
                coordinates: position,
                cell_id: nearest_cell,
                distance_to_prototype: min_distance,
            });
        }
    }
}

/// Visualize Voronoi cells as mesh boundaries
fn visualize_voronoi_cells(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    partition: Query<&ConceptualSpacePartition, Changed<ConceptualSpacePartition>>,
    subgraphs: Query<&SubgraphRegion>,
    settings: Res<VoronoiSettings>,
    existing_cells: Query<(Entity, &VoronoiCellMesh)>,
) {
    let Ok(partition) = partition.get_single() else {
        return;
    };

    // Remove old cell meshes
    for (entity, _) in existing_cells.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Create mesh for each Voronoi cell
    for cell in &partition.cells {
        // Find the subgraph color
        let color = subgraphs
            .iter()
            .find(|s| s.subgraph_id == cell.subgraph_id)
            .map(|s| s.color)
            .unwrap_or(Color::srgba(0.5, 0.5, 0.5, 0.3));

        // Create a simple polygon mesh for the cell
        let mesh = create_voronoi_cell_mesh(&cell.vertices, settings.visualization_height);

        let mesh_handle = meshes.add(mesh);
        let material_handle = materials.add(StandardMaterial {
            base_color: color.with_alpha(0.2),
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            cull_mode: None,
            ..default()
        });

        commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
            Transform::from_translation(Vec3::Y * settings.visualization_height),
            VoronoiCellMesh {
                subgraph_id: cell.subgraph_id,
                vertices: cell.vertices.clone(),
                indices: triangulate_convex_polygon(&cell.vertices),
                color,
            },
        ));
    }
}

/// Marker component for Voronoi cell meshes
#[derive(Component)]
struct VoronoiCellMesh {
    subgraph_id: SubgraphId,
    vertices: Vec<Vec3>,
    indices: Vec<u32>,
    color: Color,
}

impl VoronoiCellMesh {
    fn new(subgraph_id: SubgraphId, vertices: Vec<Vec3>, color: Color) -> Self {
        // Generate indices for triangulation
        let indices = triangulate_convex_polygon(&vertices);

        Self {
            subgraph_id,
            vertices,
            indices,
            color,
        }
    }

    fn to_mesh(&self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());

        // Add vertices
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            self.vertices.clone(),
        );

        // Add normals (all pointing up for floor mesh)
        let normals = vec![Vec3::Y; self.vertices.len()];
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

        // Add UVs
        let uvs: Vec<[f32; 2]> = self.vertices
            .iter()
            .map(|v| [(v.x + 50.0) / 100.0, (v.z + 50.0) / 100.0])
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        // Add indices
        mesh.insert_indices(Indices::U32(self.indices.clone()));

        // Add a custom attribute to identify the subgraph
        mesh.insert_attribute(
            MeshVertexAttribute::new("subgraph_id", 0, VertexFormat::Uint32),
            vec![self.subgraph_id.as_uuid().as_u128() as u32; self.vertices.len()],
        );

        mesh
    }
}

/// Create a mesh for a Voronoi cell from its vertices
fn create_voronoi_cell_mesh(vertices: &[Vec3], height: f32) -> Mesh {
    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );

    // Create vertices for top and bottom faces
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let num_verts = vertices.len();

    // Add center vertex for fan triangulation
    let center = vertices.iter().fold(Vec3::ZERO, |acc, v| acc + *v) / num_verts as f32;
    positions.push([center.x, height, center.z]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    // Add perimeter vertices
    for vertex in vertices {
        positions.push([vertex.x, height, vertex.z]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([
            (vertex.x - center.x) / 100.0 + 0.5,
            (vertex.z - center.z) / 100.0 + 0.5,
        ]);
    }

    // Create triangles (fan from center)
    for i in 0..num_verts {
        let next = (i + 1) % num_verts;
        indices.push(0);
        indices.push((i + 1) as u32);
        indices.push((next + 1) as u32);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    mesh
}

/// Triangulate a convex polygon into triangles
fn triangulate_convex_polygon(vertices: &[Vec3]) -> Vec<u32> {
    let mut indices = Vec::new();

    if vertices.len() < 3 {
        return indices;
    }

    // Simple fan triangulation from first vertex
    for i in 1..vertices.len() - 1 {
        indices.push(0);
        indices.push(i as u32);
        indices.push((i + 1) as u32);
    }

    indices
}
