use crate::contexts::graph_management::domain::*;
use crate::contexts::graph_management::events::*;
use bevy::prelude::*;
use bevy::text::{Text2d, TextFont};
use bevy_panorbit_camera::PanOrbitCamera;

// ============= Visualization Services =============
// Services that handle visual representation

// Simple random number generator to avoid dependency conflicts
fn random_f32() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    ((nanos % 1000000) as f32) / 1000000.0
}

fn random_bool() -> bool {
    random_f32() > 0.5
}

fn random_usize(max: usize) -> usize {
    (random_f32() * max as f32) as usize
}

// ============= Visual Components =============

/// Different rendering modes for graph elements
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum RenderMode {
    Mesh,       // Traditional 3D mesh rendering
    PointCloud, // Point cloud visualization
    Wireframe,  // Wireframe rendering
    Billboard,  // Always-facing-camera sprites
}

impl Default for RenderMode {
    fn default() -> Self {
        Self::Mesh
    }
}

/// Component to track visualization capabilities
#[derive(Component, Clone, Default)]
pub struct VisualizationCapability {
    pub render_mode: RenderMode,
    pub supports_instancing: bool,
    pub level_of_detail: Option<u8>,
    pub point_cloud_density: Option<f32>, // Points per unit for point cloud mode
}

/// Point cloud data for nodes
#[derive(Component, Clone)]
pub struct NodePointCloud {
    pub points: Vec<Vec3>,
    pub colors: Vec<Color>,
    pub sizes: Vec<f32>,
}

/// Point cloud data for edges
#[derive(Component, Clone)]
pub struct EdgePointCloud {
    pub points: Vec<Vec3>,
    pub colors: Vec<Color>,
    pub sizes: Vec<f32>,
    pub interpolation_samples: u32, // Number of points along edge
}

/// Different visual styles for edges
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum EdgeType {
    Line,     // Simple straight line
    Cylinder, // 3D cylinder (current implementation)
    Arc,      // Curved arc
    Bezier,   // Bezier curve
}

impl Default for EdgeType {
    fn default() -> Self {
        Self::Cylinder
    }
}

/// Visual representation of an edge
#[derive(Component, Clone)]
pub struct EdgeVisual {
    pub color: Color,
    pub thickness: f32,
    pub edge_type: EdgeType,
}

impl Default for EdgeVisual {
    fn default() -> Self {
        Self {
            color: Color::srgb(0.8, 0.8, 0.8), // Light gray
            thickness: 0.05,
            edge_type: EdgeType::default(),
        }
    }
}

/// Bundle for spawning edge visual entities
#[derive(Bundle)]
pub struct EdgeVisualBundle {
    pub edge_visual: EdgeVisual,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visualization_capability: VisualizationCapability,
}

/// Bundle for spawning node visual entities
#[derive(Bundle)]
pub struct NodeVisualBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visualization_capability: VisualizationCapability,
}

// ============= Configuration Resources =============

// Removed EdgeConfiguration and VisualizationConfiguration resources
// These will be replaced with Components and Events

// ============= Visualization Events =============

/// Request to change the edge type for future edges
#[derive(Event, Debug, Clone)]
pub struct EdgeTypeChanged {
    pub new_edge_type: EdgeType,
}

/// Request to change the render mode for future nodes
#[derive(Event, Debug, Clone)]
pub struct RenderModeChanged {
    pub new_render_mode: RenderMode,
}

/// Request to update visualization for a specific entity
#[derive(Event, Debug, Clone)]
pub struct VisualizationUpdateRequested {
    pub entity: Entity,
    pub render_mode: RenderMode,
}

/// Request to convert entity to point cloud
#[derive(Event, Debug, Clone)]
pub struct ConvertToPointCloud {
    pub entity: Entity,
    pub density: f32,
}
// ============= State Components =============

/// Current visualization settings (attached to a settings entity)
#[derive(Component, Clone)]
pub struct CurrentVisualizationSettings {
    pub edge_type: EdgeType,
    pub render_mode: RenderMode,
}

impl Default for CurrentVisualizationSettings {
    fn default() -> Self {
        Self {
            edge_type: EdgeType::Cylinder,
            render_mode: RenderMode::Mesh,
        }
    }
}

// ============= Motion Components =============
// Components to track motion and dynamics at different hierarchy levels

/// Motion dynamics for entire graphs
#[derive(Component)]
pub struct GraphMotion {
    pub rotation_speed: f32,
    pub oscillation_amplitude: f32,
    pub oscillation_frequency: f32,
    pub scale_factor: f32,
}

impl Default for GraphMotion {
    fn default() -> Self {
        Self {
            rotation_speed: 0.0,
            oscillation_amplitude: 0.0,
            oscillation_frequency: 0.0,
            scale_factor: 1.0,
        }
    }
}

/// Orbital dynamics for subgraphs
#[derive(Component)]
pub struct SubgraphOrbit {
    pub local_rotation_speed: f32,
    pub orbit_radius: f32,
    pub orbit_speed: f32,
}

/// Animation component for node pulsing effects
#[derive(Component)]
pub struct NodePulse {
    pub bounce_height: f32,
    pub bounce_speed: f32,
    pub pulse_scale: f32,
    pub pulse_speed: f32,
}

/// Animation component for edge pulsing effects
#[derive(Component)]
pub struct EdgePulse {
    pub pulse_scale: f32,
    pub pulse_speed: f32,
    pub color_intensity: f32,
    pub phase_offset: f32,
}

impl Default for EdgePulse {
    fn default() -> Self {
        Self {
            pulse_scale: 0.2,     // 20% scale variation
            pulse_speed: 2.0,     // 2 Hz
            color_intensity: 0.3, // 30% brightness variation
            phase_offset: 0.0,    // Random phase for variety
        }
    }
}

/// Animation component for directional flow along edges
#[derive(Component)]
pub struct EdgeFlow {
    pub flow_speed: f32,
    pub particle_density: f32,
    pub particle_size: f32,
    pub flow_direction: bool, // true = source to target, false = reverse
}

impl Default for EdgeFlow {
    fn default() -> Self {
        Self {
            flow_speed: 5.0,
            particle_density: 10.0,
            particle_size: 0.02,
            flow_direction: true,
        }
    }
}

/// Animation component for wave effects along edges
#[derive(Component)]
pub struct EdgeWave {
    pub wave_speed: f32,
    pub wave_amplitude: f32,
    pub wave_frequency: f32,
    pub wave_offset: f32,
}

impl Default for EdgeWave {
    fn default() -> Self {
        Self {
            wave_speed: 3.0,
            wave_amplitude: 0.1,
            wave_frequency: 2.0,
            wave_offset: 0.0,
        }
    }
}

/// Animation component for color cycling effects
#[derive(Component)]
pub struct EdgeColorCycle {
    pub cycle_speed: f32,
    pub color_range: (Color, Color),
    pub current_phase: f32,
}

impl Default for EdgeColorCycle {
    fn default() -> Self {
        Self {
            cycle_speed: 1.0,
            color_range: (
                Color::srgb(0.3, 0.5, 0.9), // Blue
                Color::srgb(0.9, 0.3, 0.5), // Red
            ),
            current_phase: 0.0,
        }
    }
}

/// Component to mark entities that should always face the camera
#[derive(Component)]
pub struct Billboard;

/// Component to mark edge segment children (for Arc and Bezier edges)
#[derive(Component)]
pub struct EdgeSegment;

/// Service to render graph elements in 3D space
pub struct RenderGraphElements;

impl RenderGraphElements {
    /// Generates point cloud data for a node
    pub fn generate_node_point_cloud(position: Vec3, density: f32, radius: f32) -> NodePointCloud {
        let mut points = Vec::new();
        let mut colors = Vec::new();
        let mut sizes = Vec::new();

        // Generate points in a spherical pattern
        let num_points = (density * 4.0 * std::f32::consts::PI * radius * radius) as usize;

        for _ in 0..num_points {
            // Random point on sphere surface
            let theta = random_f32() * 2.0 * std::f32::consts::PI;
            let phi = (random_f32() * 2.0 - 1.0).acos();
            let r = radius * random_f32().powf(1.0 / 3.0);

            let x = r * phi.sin() * theta.cos();
            let y = r * phi.sin() * theta.sin();
            let z = r * phi.cos();

            points.push(position + Vec3::new(x, y, z));
            colors.push(Color::srgb(0.3, 0.5, 0.9));
            sizes.push(0.02 + random_f32() * 0.03);
        }

        NodePointCloud {
            points,
            colors,
            sizes,
        }
    }

    /// Generates point cloud data for an edge
    pub fn generate_edge_point_cloud(
        source: Vec3,
        target: Vec3,
        samples: u32,
        density: f32,
    ) -> EdgePointCloud {
        let mut points = Vec::new();
        let mut colors = Vec::new();
        let mut sizes = Vec::new();

        // Generate points along the edge
        for i in 0..samples {
            let t = i as f32 / (samples - 1) as f32;
            let base_point = source.lerp(target, t);

            // Add some jitter for point cloud effect
            let jitter_points = (density * 0.5) as usize;
            for _ in 0..jitter_points {
                let jitter = Vec3::new(
                    (random_f32() - 0.5) * 0.1,
                    (random_f32() - 0.5) * 0.1,
                    (random_f32() - 0.5) * 0.1,
                );

                points.push(base_point + jitter);
                colors.push(Color::srgb(0.7, 0.7, 0.7));
                sizes.push(0.01 + random_f32() * 0.02);
            }
        }

        EdgePointCloud {
            points,
            colors,
            sizes,
            interpolation_samples: samples,
        }
    }

    /// Creates visual representation for edges
    pub fn render_edge(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        source_pos: Vec3,
        target_pos: Vec3,
        edge_entity: Entity,
        edge_type: EdgeType,
    ) {
        let edge_visual = EdgeVisual {
            edge_type,
            ..default()
        };

        // Create material for all edge types
        let material = materials.add(StandardMaterial {
            base_color: edge_visual.color,
            metallic: 0.1,
            perceptual_roughness: 0.8,
            ..default()
        });

        match edge_type {
            EdgeType::Line => Self::render_line_edge(
                commands,
                meshes,
                material,
                source_pos,
                target_pos,
                edge_entity,
                &edge_visual,
            ),
            EdgeType::Cylinder => Self::render_cylinder_edge(
                commands,
                meshes,
                material,
                source_pos,
                target_pos,
                edge_entity,
                &edge_visual,
            ),
            EdgeType::Arc => Self::render_arc_edge(
                commands,
                meshes,
                material,
                source_pos,
                target_pos,
                edge_entity,
                &edge_visual,
            ),
            EdgeType::Bezier => Self::render_bezier_edge(
                commands,
                meshes,
                material,
                source_pos,
                target_pos,
                edge_entity,
                &edge_visual,
            ),
        }

        // Randomly add animation components to make edges more dynamic
        let random = random_f32();

        if random < 0.3 {
            // 30% chance of pulse animation
            commands.entity(edge_entity).insert(EdgePulse {
                phase_offset: random_f32() * std::f32::consts::PI * 2.0,
                pulse_speed: 1.5 + random_f32() * 2.0, // Vary speed between 1.5-3.5 Hz
                color_intensity: 0.2 + random_f32() * 0.3, // Vary intensity 0.2-0.5
                ..default()
            });
        } else if random < 0.5 {
            // 20% chance of wave animation
            commands.entity(edge_entity).insert(EdgeWave {
                wave_offset: random_f32() * std::f32::consts::PI * 2.0,
                wave_speed: 2.0 + random_f32() * 2.0, // Vary speed
                wave_amplitude: 0.05 + random_f32() * 0.1, // Vary amplitude
                ..default()
            });
        } else if random < 0.7 {
            // 20% chance of color cycle
            let color_pairs = [
                (Color::srgb(0.3, 0.5, 0.9), Color::srgb(0.9, 0.3, 0.5)), // Blue to Red
                (Color::srgb(0.2, 0.8, 0.4), Color::srgb(0.8, 0.8, 0.2)), // Green to Yellow
                (Color::srgb(0.7, 0.3, 0.9), Color::srgb(0.3, 0.9, 0.7)), // Purple to Cyan
            ];
            let color_pair = color_pairs[random_usize(color_pairs.len())];

            commands.entity(edge_entity).insert(EdgeColorCycle {
                cycle_speed: 0.5 + random_f32() * 1.5, // Vary speed 0.5-2.0
                color_range: color_pair,
                current_phase: random_f32(),
            });
        } else if random < 0.85 {
            // 15% chance of flow animation
            commands.entity(edge_entity).insert(EdgeFlow {
                flow_speed: 2.0 + random_f32() * 4.0, // Vary speed 2.0-6.0
                flow_direction: random_bool(),
                ..default()
            });
        }
        // 15% chance of no animation (reduced from 30%)
    }

    /// Renders a simple line edge
    fn render_line_edge(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        material: Handle<StandardMaterial>,
        source_pos: Vec3,
        target_pos: Vec3,
        edge_entity: Entity,
        edge_visual: &EdgeVisual,
    ) {
        // Create a thin box to represent a line
        let direction = target_pos - source_pos;
        let length = direction.length();
        let midpoint = source_pos + direction * 0.5;

        // Use a thin cuboid for the line
        let mesh = meshes.add(Cuboid::new(
            edge_visual.thickness * 0.5, // Make it thinner than cylinder
            edge_visual.thickness * 0.5,
            length,
        ));

        // Calculate rotation to align the cuboid along the edge
        // The cuboid's default orientation is along Z axis
        let mut rotation = Quat::IDENTITY;
        if direction.length_squared() > 0.0001 {
            let normalized_dir = direction.normalize();
            // Only rotate if not already aligned with Z
            if (normalized_dir - Vec3::Z).length_squared() > 0.0001 {
                rotation = Quat::from_rotation_arc(Vec3::Z, normalized_dir);
            }
        }

        commands
            .entity(edge_entity)
            .insert(Mesh3d(mesh))
            .insert(MeshMaterial3d(material))
            .insert(Transform {
                translation: midpoint,
                rotation,
                scale: Vec3::ONE,
            })
            .insert(edge_visual.clone());
    }

    /// Renders a cylinder edge (existing implementation)
    fn render_cylinder_edge(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        material: Handle<StandardMaterial>,
        source_pos: Vec3,
        target_pos: Vec3,
        edge_entity: Entity,
        edge_visual: &EdgeVisual,
    ) {
        let direction = target_pos - source_pos;
        let length = direction.length();
        let midpoint = source_pos + direction * 0.5;

        let mesh = meshes.add(Cylinder::new(edge_visual.thickness, length));

        let up = Vec3::Y;
        let rotation = Quat::from_rotation_arc(up, direction.normalize());

        commands
            .entity(edge_entity)
            .insert(Mesh3d(mesh))
            .insert(MeshMaterial3d(material))
            .insert(Transform {
                translation: midpoint,
                rotation,
                scale: Vec3::ONE,
            })
            .insert(edge_visual.clone());
    }

    /// Renders an arc edge
    fn render_arc_edge(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        material: Handle<StandardMaterial>,
        source_pos: Vec3,
        target_pos: Vec3,
        edge_entity: Entity,
        edge_visual: &EdgeVisual,
    ) {
        // Calculate arc parameters
        let chord = target_pos - source_pos;
        let chord_length = chord.length();
        let arc_height = chord_length * 0.3; // Arc height as 30% of chord length

        // Create arc using cylinder segments
        let segments = 20;
        let segment_commands = &mut commands.entity(edge_entity);

        // We'll create a parent entity and add cylinder segments as children
        segment_commands
            .insert(Transform::default())
            .insert(GlobalTransform::default())
            .insert(edge_visual.clone())
            .with_children(|parent| {
                for i in 0..segments {
                    let t1 = i as f32 / segments as f32;
                    let t2 = (i + 1) as f32 / segments as f32;
                    let angle1 = t1 * std::f32::consts::PI;
                    let angle2 = t2 * std::f32::consts::PI;

                    // Calculate positions along the arc
                    let pos1 = source_pos + chord * t1 + Vec3::Y * (angle1.sin() * arc_height);
                    let pos2 = source_pos + chord * t2 + Vec3::Y * (angle2.sin() * arc_height);

                    let segment_dir = pos2 - pos1;
                    let segment_length = segment_dir.length();
                    let segment_midpoint = pos1 + segment_dir * 0.5;

                    if segment_length > 0.001 {
                        // Create a small cylinder for this segment
                        let segment_mesh =
                            meshes.add(Cylinder::new(edge_visual.thickness * 0.8, segment_length));

                        let rotation = if segment_dir.normalize() != Vec3::Y {
                            Quat::from_rotation_arc(Vec3::Y, segment_dir.normalize())
                        } else {
                            Quat::IDENTITY
                        };

                        parent.spawn((
                            Mesh3d(segment_mesh),
                            MeshMaterial3d(material.clone()),
                            Transform {
                                translation: segment_midpoint,
                                rotation,
                                scale: Vec3::ONE,
                            },
                            EdgeSegment,
                        ));
                    }
                }
            });
    }

    /// Renders a bezier curve edge
    fn render_bezier_edge(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        material: Handle<StandardMaterial>,
        source_pos: Vec3,
        target_pos: Vec3,
        edge_entity: Entity,
        edge_visual: &EdgeVisual,
    ) {
        // Calculate control points for bezier curve
        let midpoint = (source_pos + target_pos) * 0.5;
        let direction = (target_pos - source_pos).normalize();

        // Create perpendicular offset for curve
        let offset = if direction.dot(Vec3::Y).abs() < 0.99 {
            direction.cross(Vec3::Y).normalize() * 1.0
        } else {
            direction.cross(Vec3::X).normalize() * 1.0
        };

        let control1 = midpoint + offset + Vec3::Y * 0.5;
        let control2 = midpoint - offset + Vec3::Y * 0.5;

        // Generate bezier curve using cylinder segments
        let segments = 30;
        let segment_commands = &mut commands.entity(edge_entity);

        segment_commands
            .insert(Transform::default())
            .insert(GlobalTransform::default())
            .insert(edge_visual.clone())
            .with_children(|parent| {
                for i in 0..segments {
                    let t1 = i as f32 / segments as f32;
                    let t2 = (i + 1) as f32 / segments as f32;

                    // Cubic bezier curve formula
                    let pos1 = (1.0 - t1).powi(3) * source_pos
                        + 3.0 * (1.0 - t1).powi(2) * t1 * control1
                        + 3.0 * (1.0 - t1) * t1.powi(2) * control2
                        + t1.powi(3) * target_pos;

                    let pos2 = (1.0 - t2).powi(3) * source_pos
                        + 3.0 * (1.0 - t2).powi(2) * t2 * control1
                        + 3.0 * (1.0 - t2) * t2.powi(2) * control2
                        + t2.powi(3) * target_pos;

                    let segment_dir = pos2 - pos1;
                    let segment_length = segment_dir.length();
                    let segment_midpoint = pos1 + segment_dir * 0.5;

                    if segment_length > 0.001 {
                        // Create a small cylinder for this segment
                        let segment_mesh =
                            meshes.add(Cylinder::new(edge_visual.thickness * 0.7, segment_length));

                        let rotation = if segment_dir.normalize() != Vec3::Y {
                            Quat::from_rotation_arc(Vec3::Y, segment_dir.normalize())
                        } else {
                            Quat::IDENTITY
                        };

                        parent.spawn((
                            Mesh3d(segment_mesh),
                            MeshMaterial3d(material.clone()),
                            Transform {
                                translation: segment_midpoint,
                                rotation,
                                scale: Vec3::ONE,
                            },
                            EdgeSegment,
                        ));
                    }
                }
            });
    }

    /// Renders a node with the specified render mode
    pub fn render_node(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        node_entity: Entity,
        position: Vec3,
        _label: &str,
        render_mode: RenderMode,
    ) {
        match render_mode {
            RenderMode::Mesh => {
                // Traditional mesh rendering
                let mesh = meshes.add(Sphere::new(0.3).mesh());

                let material = materials.add(StandardMaterial {
                    base_color: Color::srgb(0.3, 0.5, 0.9),
                    metallic: 0.3,
                    perceptual_roughness: 0.5,
                    ..default()
                });

                // Don't add Transform as it's already added in graph_management
                // Just insert visual components
                commands
                    .entity(node_entity)
                    .insert(Mesh3d(mesh))
                    .insert(MeshMaterial3d(material))
                    .insert(VisualizationCapability {
                        render_mode,
                        ..default()
                    });
            }
            RenderMode::PointCloud => {
                // Generate point cloud data
                let point_cloud = Self::generate_node_point_cloud(position, 50.0, 0.3);

                commands
                    .entity(node_entity)
                    .insert(point_cloud)
                    .insert(VisualizationCapability {
                        render_mode,
                        point_cloud_density: Some(50.0),
                        ..default()
                    });

                // Note: Actual point cloud rendering would be implemented in a dedicated plugin
                info!(
                    "Point cloud data generated for node - rendering requires point cloud plugin"
                );
            }
            RenderMode::Wireframe => {
                // Wireframe rendering - use lines to show mesh edges
                let mesh = meshes.add(Sphere::new(0.3).mesh().ico(2).unwrap());

                let material = materials.add(StandardMaterial {
                    base_color: Color::srgb(0.3, 0.5, 0.9),
                    metallic: 0.0,
                    perceptual_roughness: 1.0,
                    alpha_mode: AlphaMode::Opaque,
                    emissive: Color::srgb(0.1, 0.2, 0.4).into(),
                    ..default()
                });

                commands
                    .entity(node_entity)
                    .insert(Mesh3d(mesh))
                    .insert(MeshMaterial3d(material))
                    .insert(VisualizationCapability {
                        render_mode,
                        ..default()
                    });

                info!("Wireframe mode enabled for node");
            }
            RenderMode::Billboard => {
                // Billboard rendering - brightly colored spheres that are easy to spot
                // TODO: Add proper 3D text rendering in Phase 2

                // Create a smaller sphere for billboard mode
                let mesh = meshes.add(Sphere::new(0.25).mesh());

                // Use different bright colors based on node label hash for variety
                let color_index = _label.len() % 5;
                let base_color = match color_index {
                    0 => Color::srgb(1.0, 0.2, 0.2), // Red
                    1 => Color::srgb(0.2, 1.0, 0.2), // Green
                    2 => Color::srgb(0.2, 0.2, 1.0), // Blue
                    3 => Color::srgb(1.0, 1.0, 0.2), // Yellow
                    _ => Color::srgb(1.0, 0.2, 1.0), // Magenta
                };

                let material = materials.add(StandardMaterial {
                    base_color,
                    emissive: LinearRgba::from(base_color) * 0.3, // Emissive glow for visibility
                    metallic: 0.0,
                    perceptual_roughness: 0.4,
                    ..default()
                });

                commands
                    .entity(node_entity)
                    .insert(Mesh3d(mesh))
                    .insert(MeshMaterial3d(material))
                    .insert(Billboard) // Mark as billboard for potential future use
                    .insert(VisualizationCapability {
                        render_mode,
                        ..default()
                    });

                info!(
                    "Billboard mode enabled for node '{}' with color index {}",
                    _label, color_index
                );
            }
        }
    }

    /// System that listens for NodeAdded events and creates visual representations
    pub fn visualize_new_nodes(
        mut commands: Commands,
        mut events: EventReader<NodeAdded>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        nodes: Query<(Entity, &NodeIdentity, &NodeContent, &SpatialPosition)>,
        settings: Query<&CurrentVisualizationSettings>,
    ) {
        let event_count = events.len();
        if event_count > 0 {
            info!("visualize_new_nodes: Processing {} NodeAdded events", event_count);
        }

        // Get current settings or use defaults
        let render_mode = settings
            .single()
            .map(|s| s.render_mode)
            .unwrap_or(RenderMode::Mesh);

        for event in events.read() {
            info!("Visualizing node: {} with ID {:?}", event.content.label, event.node);

            // Find the entity that was just created
            let mut found = false;
            for (entity, identity, content, position) in nodes.iter() {
                if identity.0 == event.node.0 {
                    info!("Found node entity {:?} at position {:?}", entity, position.coordinates_3d);
                    found = true;
                    Self::render_node(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        entity,
                        position.coordinates_3d,
                        &content.label,
                        render_mode,
                    );
                    break;
                }
            }
            if !found {
                warn!("Could not find entity for node {:?} in query!", event.node);
            }
        }
    }

    /// System that listens for EdgeConnected events and creates visual representations
    pub fn visualize_new_edges(
        mut commands: Commands,
        mut events: EventReader<EdgeConnected>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        edges: Query<(Entity, &EdgeIdentity, &EdgeRelationship)>,
        nodes: Query<(&NodeIdentity, &SpatialPosition)>,
        settings: Query<&CurrentVisualizationSettings>,
    ) {
        let event_count = events.len();
        if event_count > 0 {
            info!("visualize_new_edges: Processing {} EdgeConnected events", event_count);
        }

        // Get current settings or use defaults
        let edge_type = settings
            .single()
            .map(|s| s.edge_type)
            .unwrap_or(EdgeType::Cylinder);

        for event in events.read() {
            info!("Visualizing edge: {:?} from {:?} to {:?}", event.edge, event.relationship.source, event.relationship.target);

            // Find the edge entity that was just created
            let mut found = false;
            for (edge_entity, edge_identity, relationship) in edges.iter() {
                if edge_identity.0 == event.edge.0 {
                    info!("Found edge entity {:?}", edge_entity);
                    found = true;
                    // Find source and target node positions
                    let mut source_pos = None;
                    let mut target_pos = None;

                    for (node_id, position) in nodes.iter() {
                        if node_id.0 == relationship.source.0 {
                            source_pos = Some(position.coordinates_3d);
                        }
                        if node_id.0 == relationship.target.0 {
                            target_pos = Some(position.coordinates_3d);
                        }
                    }

                    // Render edge if both positions found
                    if let (Some(source), Some(target)) = (source_pos, target_pos) {
                        Self::render_edge(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            source,
                            target,
                            edge_entity,
                            edge_type,
                        );
                    } else {
                        warn!("Could not find positions for edge endpoints");
                    }
                    break;
                }
            }
            if !found {
                warn!("Could not find entity for edge {:?} in query!", event.edge);
            }
        }
    }

    /// Handles visualization update requests for specific entities
    pub fn handle_visualization_update_requests(
        mut commands: Commands,
        mut events: EventReader<VisualizationUpdateRequested>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        nodes: Query<(&NodeContent, &SpatialPosition), With<crate::contexts::graph_management::domain::Node>>,
        children_query: Query<&Children>,
    ) {
        for event in events.read() {
            // Find if this is a node entity
            if let Ok((content, position)) = nodes.get(event.entity) {
                // First, despawn all children
                if let Ok(children) = children_query.get(event.entity) {
                    for child in children.iter() {
                        commands.entity(child).despawn();
                    }
                }

                // Remove all visual components
                commands
                    .entity(event.entity)
                    .remove::<Mesh3d>()
                    .remove::<MeshMaterial3d<StandardMaterial>>()
                    .remove::<Billboard>()
                    .remove::<NodePointCloud>()
                    .remove::<VisualizationCapability>();

                // Re-render with requested render mode
                RenderGraphElements::render_node(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    event.entity,
                    position.coordinates_3d,
                    &content.label,
                    event.render_mode,
                );

                info!(
                    "Updated entity {:?} to render mode: {:?}",
                    event.entity, event.render_mode
                );
            }
        }
    }

    /// Handles requests to convert entities to point clouds
    pub fn handle_convert_to_point_cloud(
        mut commands: Commands,
        mut events: EventReader<ConvertToPointCloud>,
        nodes: Query<&SpatialPosition, With<crate::contexts::graph_management::domain::Node>>,
        edges: Query<(&EdgeRelationship,), With<crate::contexts::graph_management::domain::Edge>>,
        node_positions: Query<(&NodeIdentity, &SpatialPosition)>,
    ) {
        for event in events.read() {
            // Check if it's a node
            if let Ok(position) = nodes.get(event.entity) {
                let point_cloud = RenderGraphElements::generate_node_point_cloud(
                    position.coordinates_3d,
                    event.density,
                    0.3, // Default radius
                );

                commands
                    .entity(event.entity)
                    .insert(point_cloud)
                    .insert(VisualizationCapability {
                        render_mode: RenderMode::PointCloud,
                        point_cloud_density: Some(event.density),
                        ..default()
                    });

                info!("Converted node {:?} to point cloud", event.entity);
            }
            // Check if it's an edge
            else if let Ok((relationship,)) = edges.get(event.entity) {
                // Find source and target positions
                let mut source_pos = None;
                let mut target_pos = None;

                for (node_id, position) in node_positions.iter() {
                    if node_id.0 == relationship.source.0 {
                        source_pos = Some(position.coordinates_3d);
                    }
                    if node_id.0 == relationship.target.0 {
                        target_pos = Some(position.coordinates_3d);
                    }
                }

                if let (Some(source), Some(target)) = (source_pos, target_pos) {
                    let point_cloud = RenderGraphElements::generate_edge_point_cloud(
                        source,
                        target,
                        50, // Default samples
                        event.density,
                    );

                    commands
                        .entity(event.entity)
                        .insert(point_cloud)
                        .insert(VisualizationCapability {
                            render_mode: RenderMode::PointCloud,
                            point_cloud_density: Some(event.density),
                            ..default()
                        });

                    info!("Converted edge {:?} to point cloud", event.entity);
                }
            }
        }
    }

    /// Renders flow particles for edges with EdgeFlow component
    pub fn render_edge_flow_particles(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        time: Res<Time>,
        edges: Query<(Entity, &EdgeRelationship, &EdgeFlow), With<EdgeVisual>>,
        nodes: Query<(&NodeIdentity, &SpatialPosition)>,
        mut flow_particles: Query<(&mut Transform, &FlowParticle)>,
    ) {
        let elapsed = time.elapsed_secs();

        for (edge_entity, relationship, flow) in edges.iter() {
            // Find source and target positions
            let mut source_pos = None;
            let mut target_pos = None;

            for (node_id, position) in nodes.iter() {
                if node_id.0 == relationship.source.0 {
                    source_pos = Some(position.coordinates_3d);
                }
                if node_id.0 == relationship.target.0 {
                    target_pos = Some(position.coordinates_3d);
                }
            }

            if let (Some(source), Some(target)) = (source_pos, target_pos) {
                // Spawn particles if they don't exist yet
                let particle_count = (flow.particle_density as usize).max(1);

                // Check if we need to spawn particles for this edge
                let mut has_particles = false;
                for (_, particle) in flow_particles.iter() {
                    if particle.edge_entity == edge_entity {
                        has_particles = true;
                        break;
                    }
                }

                if !has_particles {
                    // Spawn flow particles
                    for i in 0..particle_count {
                        let offset = i as f32 / particle_count as f32;

                        let particle_mesh = meshes.add(Sphere::new(flow.particle_size).mesh());
                        let particle_material = materials.add(StandardMaterial {
                            base_color: Color::srgb(0.9, 0.9, 1.0),
                            emissive: LinearRgba::rgb(0.3, 0.3, 0.5),
                            ..default()
                        });

                        commands.spawn((
                            Mesh3d(particle_mesh),
                            MeshMaterial3d(particle_material),
                            Transform::from_translation(source),
                            GlobalTransform::default(),
                            FlowParticle {
                                edge_entity,
                                offset,
                                speed: flow.flow_speed,
                            },
                        ));
                    }
                }

                // Update existing particles
                for (mut transform, particle) in flow_particles.iter_mut() {
                    if particle.edge_entity == edge_entity {
                        // Calculate position along edge
                        let mut t = (elapsed * particle.speed / 10.0 + particle.offset) % 1.0;
                        if !flow.flow_direction {
                            t = 1.0 - t;
                        }

                        transform.translation = source.lerp(target, t);
                    }
                }
            }
        }
    }
}

/// Component to mark flow particles and track their parent edge
#[derive(Component)]
pub struct FlowParticle {
    pub edge_entity: Entity,
    pub offset: f32,
    pub speed: f32,
}

/// Service to perform raycasting for selection
pub struct PerformRaycast;

impl PerformRaycast {
    /// Converts screen coordinates to a ray in world space
    pub fn screen_to_ray(
        camera: &Camera,
        camera_transform: &GlobalTransform,
        screen_pos: Vec2,
    ) -> Option<Ray3d> {
        camera.viewport_to_world(camera_transform, screen_pos).ok()
    }

    /// Checks ray-sphere intersection
    pub fn ray_intersects_sphere(
        ray: &Ray3d,
        sphere_center: Vec3,
        sphere_radius: f32,
    ) -> Option<f32> {
        // Vector from ray origin to sphere center
        let oc = ray.origin - sphere_center;

        // Coefficients for quadratic equation
        let direction = ray.direction.as_vec3();
        let a = direction.dot(direction);
        let b = 2.0 * oc.dot(direction);
        let c = oc.dot(oc) - sphere_radius * sphere_radius;

        // Discriminant
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            // No intersection
            None
        } else {
            // Calculate the two intersection points
            let sqrt_discriminant = discriminant.sqrt();
            let t1 = (-b - sqrt_discriminant) / (2.0 * a);
            let t2 = (-b + sqrt_discriminant) / (2.0 * a);

            // Return the closest positive intersection
            if t1 > 0.0 {
                Some(t1)
            } else if t2 > 0.0 {
                Some(t2)
            } else {
                None
            }
        }
    }
}

/// Service to handle user input and interactions
pub struct HandleUserInput;

impl HandleUserInput {
    /// Change edge rendering type with keyboard
    pub fn change_edge_type(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut events: EventWriter<EdgeTypeChanged>,
    ) {
        if keyboard.just_pressed(KeyCode::Digit1) {
            events.write(EdgeTypeChanged {
                new_edge_type: EdgeType::Line,
            });
            info!("Edge type changed to: Line");
        } else if keyboard.just_pressed(KeyCode::Digit2) {
            events.write(EdgeTypeChanged {
                new_edge_type: EdgeType::Cylinder,
            });
            info!("Edge type changed to: Cylinder");
        } else if keyboard.just_pressed(KeyCode::Digit3) {
            events.write(EdgeTypeChanged {
                new_edge_type: EdgeType::Arc,
            });
            info!("Edge type changed to: Arc");
        } else if keyboard.just_pressed(KeyCode::Digit4) {
            events.write(EdgeTypeChanged {
                new_edge_type: EdgeType::Bezier,
            });
            info!("Edge type changed to: Bezier");
        }
    }

    /// Change render mode with keyboard
    pub fn change_render_mode(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut events: EventWriter<RenderModeChanged>,
    ) {
        if keyboard.just_pressed(KeyCode::KeyM) {
            events.write(RenderModeChanged {
                new_render_mode: RenderMode::Mesh,
            });
            info!("Render mode changed to: Mesh");
        } else if keyboard.just_pressed(KeyCode::KeyP) {
            events.write(RenderModeChanged {
                new_render_mode: RenderMode::PointCloud,
            });
            info!("Render mode changed to: PointCloud (requires point cloud plugin)");
        } else if keyboard.just_pressed(KeyCode::KeyW) {
            events.write(RenderModeChanged {
                new_render_mode: RenderMode::Wireframe,
            });
            info!("Render mode changed to: Wireframe");
        } else if keyboard.just_pressed(KeyCode::KeyB) {
            events.write(RenderModeChanged {
                new_render_mode: RenderMode::Billboard,
            });
            info!("Render mode changed to: Billboard");
        }
    }

    /// Trigger force-directed layout with keyboard
    pub fn trigger_layout(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut layout_events: EventWriter<crate::contexts::visualization::layout::LayoutRequested>,
        graphs: Query<&GraphIdentity>,
    ) {
        if keyboard.just_pressed(KeyCode::KeyL) {
            let graph_count = graphs.iter().count();
            info!("Layout key pressed. Found {} graphs", graph_count);

            // Find the first graph (in a real app, you might want to target a specific graph)
            if let Some(graph_id) = graphs.iter().next() {
                layout_events.write(crate::contexts::visualization::layout::LayoutRequested {
                    graph: *graph_id,
                    algorithm:
                        crate::contexts::visualization::layout::LayoutAlgorithm::ForceDirected,
                });
                info!("Force-directed layout requested for graph: {:?}", graph_id);
            } else {
                warn!("No graph found to apply layout to");
            }
        }
    }

    /// Trigger visualization update for selected entities
    pub fn trigger_visualization_update(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut events: EventWriter<VisualizationUpdateRequested>,
        selected_nodes: Query<Entity, (With<crate::contexts::selection::domain::Selected>, With<crate::contexts::graph_management::domain::Node>)>,
    ) {
        // Check for key combinations to change render mode for selected nodes
        if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
            if keyboard.just_pressed(KeyCode::Digit1) {
                // Ctrl+1: Change selected nodes to Mesh mode
                for entity in selected_nodes.iter() {
                    events.write(VisualizationUpdateRequested {
                        entity,
                        render_mode: RenderMode::Mesh,
                    });
                }
                info!("Changed selected nodes to Mesh mode");
            } else if keyboard.just_pressed(KeyCode::Digit2) {
                // Ctrl+2: Change selected nodes to PointCloud mode
                for entity in selected_nodes.iter() {
                    events.write(VisualizationUpdateRequested {
                        entity,
                        render_mode: RenderMode::PointCloud,
                    });
                }
                info!("Changed selected nodes to PointCloud mode");
            } else if keyboard.just_pressed(KeyCode::Digit3) {
                // Ctrl+3: Change selected nodes to Wireframe mode
                for entity in selected_nodes.iter() {
                    events.write(VisualizationUpdateRequested {
                        entity,
                        render_mode: RenderMode::Wireframe,
                    });
                }
                info!("Changed selected nodes to Wireframe mode");
            } else if keyboard.just_pressed(KeyCode::Digit4) {
                // Ctrl+4: Change selected nodes to Billboard mode
                for entity in selected_nodes.iter() {
                    events.write(VisualizationUpdateRequested {
                        entity,
                        render_mode: RenderMode::Billboard,
                    });
                }
                info!("Changed selected nodes to Billboard mode");
            }
        }
    }

    /// Trigger point cloud conversion for selected entities
    pub fn trigger_point_cloud_conversion(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut events: EventWriter<ConvertToPointCloud>,
        selected_nodes: Query<Entity, (With<crate::contexts::selection::domain::Selected>, With<crate::contexts::graph_management::domain::Node>)>,
        selected_edges: Query<Entity, (With<crate::contexts::selection::domain::Selected>, With<crate::contexts::graph_management::domain::Edge>)>,
    ) {
        if keyboard.just_pressed(KeyCode::KeyC) {
            // C key: Convert selected entities to point clouds
            let mut converted_count = 0;

            // Convert selected nodes
            for entity in selected_nodes.iter() {
                events.write(ConvertToPointCloud {
                    entity,
                    density: 50.0, // Default density
                });
                converted_count += 1;
            }

            // Convert selected edges
            for entity in selected_edges.iter() {
                events.write(ConvertToPointCloud {
                    entity,
                    density: 30.0, // Lower density for edges
                });
                converted_count += 1;
            }

            if converted_count > 0 {
                info!("Converting {} selected entities to point clouds", converted_count);
            } else {
                info!("No entities selected for point cloud conversion. Select nodes or edges first.");
            }
        }
    }
}

/// Service to handle visualization state updates
pub struct UpdateVisualizationState;

impl UpdateVisualizationState {
    /// Updates settings based on EdgeTypeChanged events
    pub fn handle_edge_type_changed(
        mut events: EventReader<EdgeTypeChanged>,
        mut settings: Query<&mut CurrentVisualizationSettings>,
    ) {
        for event in events.read() {
            for mut setting in settings.iter_mut() {
                setting.edge_type = event.new_edge_type;
            }
        }
    }

    /// Updates settings based on RenderModeChanged events
    pub fn handle_render_mode_changed(
        mut events: EventReader<RenderModeChanged>,
        mut settings: Query<&mut CurrentVisualizationSettings>,
    ) {
        for event in events.read() {
            for mut setting in settings.iter_mut() {
                setting.render_mode = event.new_render_mode;
            }
        }
    }

    /// Re-renders edges when edge type changes
    pub fn update_existing_edges(
        mut commands: Commands,
        mut edge_type_events: EventReader<EdgeTypeChanged>,
        settings: Query<&CurrentVisualizationSettings>,
        edges: Query<
            (Entity, &EdgeRelationship, &EdgeVisual),
            With<crate::contexts::graph_management::domain::Edge>,
        >,
        nodes: Query<(&NodeIdentity, &SpatialPosition)>,
        children_query: Query<&Children>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        // Only proceed if edge type changed
        if edge_type_events.is_empty() {
            return;
        }

        // Consume events to clear them
        edge_type_events.clear();

        // Get current settings
        let Ok(current_settings) = settings.single() else {
            return;
        };

        // Update all existing edges with new edge type
        for (edge_entity, edge_relationship, _edge_visual) in edges.iter() {
            // Find source and target positions
            let mut source_pos = None;
            let mut target_pos = None;

            for (node_id, position) in nodes.iter() {
                if node_id.0 == edge_relationship.source.0 {
                    source_pos = Some(position.coordinates_3d);
                }
                if node_id.0 == edge_relationship.target.0 {
                    target_pos = Some(position.coordinates_3d);
                }
            }

            if let (Some(source), Some(target)) = (source_pos, target_pos) {
                // First, despawn all children (for Arc and Bezier edges)
                if let Ok(children) = children_query.get(edge_entity) {
                    for child in children.iter() {
                        commands.entity(child).despawn();
                    }
                }

                // Remove all visual components
                commands
                    .entity(edge_entity)
                    .remove::<Mesh3d>()
                    .remove::<MeshMaterial3d<StandardMaterial>>()
                    .remove::<Transform>()
                    .remove::<GlobalTransform>()
                    .remove::<EdgeVisual>();

                // Remove any animation components that might exist
                commands
                    .entity(edge_entity)
                    .remove::<EdgePulse>()
                    .remove::<EdgeWave>()
                    .remove::<EdgeColorCycle>()
                    .remove::<EdgeFlow>();

                // Re-render with new edge type
                RenderGraphElements::render_edge(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    source,
                    target,
                    edge_entity,
                    current_settings.edge_type,
                );

                info!(
                    "Updated edge {:?} to type {:?}",
                    edge_entity, current_settings.edge_type
                );
            }
        }
    }

    /// Re-renders nodes when render mode changes
    pub fn update_existing_nodes(
        mut commands: Commands,
        mut render_mode_events: EventReader<RenderModeChanged>,
        settings: Query<&CurrentVisualizationSettings>,
        nodes: Query<
            (Entity, &NodeIdentity, &NodeContent, &SpatialPosition),
            With<crate::contexts::graph_management::domain::Node>,
        >,
        children_query: Query<&Children>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        // Only proceed if render mode changed
        if render_mode_events.is_empty() {
            return;
        }

        // Consume events to clear them
        render_mode_events.clear();

        // Get current settings
        let Ok(current_settings) = settings.single() else {
            return;
        };

        // Update all existing nodes with new render mode
        for (node_entity, _node_identity, node_content, spatial_position) in nodes.iter() {
            // First, despawn all children (in case node has child entities)
            if let Ok(children) = children_query.get(node_entity) {
                for child in children.iter() {
                    commands.entity(child).despawn();
                }
            }

            // Remove all visual components
            commands
                .entity(node_entity)
                .remove::<Mesh3d>()
                .remove::<MeshMaterial3d<StandardMaterial>>()
                .remove::<Text2d>()
                .remove::<TextFont>()
                .remove::<Billboard>()
                .remove::<NodePointCloud>()
                .remove::<VisualizationCapability>();

            // Remove all animation components (they are mode-specific)
            commands
                .entity(node_entity)
                .remove::<NodePulse>()
                .remove::<GraphMotion>()
                .remove::<SubgraphOrbit>();

            // Re-render with new render mode
            RenderGraphElements::render_node(
                &mut commands,
                &mut meshes,
                &mut materials,
                node_entity,
                spatial_position.coordinates_3d,
                &node_content.label,
                current_settings.render_mode,
            );

            info!(
                "Updated node '{}' to render mode: {:?}",
                node_content.label, current_settings.render_mode
            );
        }
    }

    /// Manages graph animations based on render mode
    pub fn manage_graph_animations_on_mode_change(
        mut render_mode_events: EventReader<RenderModeChanged>,
        mut graphs: Query<&mut GraphMotion, With<Graph>>,
    ) {
        for event in render_mode_events.read() {
            match event.new_render_mode {
                RenderMode::Billboard => {
                    // Pause all graph animations in billboard mode
                    for mut motion in graphs.iter_mut() {
                        motion.rotation_speed = 0.0;
                        motion.oscillation_amplitude = 0.0;
                        info!("Paused graph animations for billboard mode");
                    }
                }
                _ => {
                    // Resume graph animations for other modes
                    for mut motion in graphs.iter_mut() {
                        if motion.rotation_speed == 0.0 {
                            motion.rotation_speed = 0.5; // Restore default rotation
                            info!("Resumed graph animations");
                        }
                    }
                }
            }
        }
    }
}

/// Service to animate graph elements at all hierarchy levels
pub struct AnimateGraphElements;

impl AnimateGraphElements {
    /// Animates entire graphs (rotation, oscillation, scaling)
    pub fn animate_graphs(
        time: Res<Time>,
        mut graphs: Query<(&mut Transform, &GraphMotion), With<Graph>>,
    ) {
        for (mut transform, animation) in graphs.iter_mut() {
            let elapsed = time.elapsed_secs();

            // Apply continuous rotation
            if animation.rotation_speed != 0.0 {
                transform.rotate_y(animation.rotation_speed * time.delta_secs());
            }

            // Apply oscillation
            if animation.oscillation_amplitude > 0.0 {
                let oscillation = animation.oscillation_amplitude
                    * (elapsed * animation.oscillation_frequency).sin();
                transform.translation.y = oscillation;
            }

            // Apply scaling
            if animation.scale_factor != 1.0 {
                let scale = Vec3::splat(animation.scale_factor);
                transform.scale = scale;
            }
        }
    }

    /// Animates subgraphs (local rotation, orbiting)
    pub fn animate_subgraphs(
        time: Res<Time>,
        mut subgraphs: Query<(&mut Transform, &SubgraphOrbit), Without<Graph>>,
    ) {
        for (mut transform, animation) in subgraphs.iter_mut() {
            let elapsed = time.elapsed_secs();

            // Apply local rotation
            if animation.local_rotation_speed != 0.0 {
                transform.rotate_y(animation.local_rotation_speed * time.delta_secs());
            }

            // Apply orbital motion
            if animation.orbit_radius > 0.0 {
                let angle = elapsed * animation.orbit_speed;
                let x = angle.cos() * animation.orbit_radius;
                let z = angle.sin() * animation.orbit_radius;
                transform.translation.x = x;
                transform.translation.z = z;
            }
        }
    }

    /// Animates individual nodes (bouncing, pulsing)
    pub fn animate_nodes(
        time: Res<Time>,
        mut nodes: Query<
            (&mut Transform, &NodePulse),
            With<crate::contexts::graph_management::domain::Node>,
        >,
    ) {
        for (mut transform, animation) in nodes.iter_mut() {
            let elapsed = time.elapsed_secs();

            // Apply bouncing
            if animation.bounce_height > 0.0 {
                let bounce =
                    animation.bounce_height * (elapsed * animation.bounce_speed).sin().abs();
                transform.translation.y += bounce;
            }

            // Apply pulsing
            if animation.pulse_scale > 0.0 {
                let pulse = 1.0 + animation.pulse_scale * (elapsed * animation.pulse_speed).sin();
                transform.scale = Vec3::splat(pulse);
            }
        }
    }

    /// Animates edges with various effects
    pub fn animate_edges(
        time: Res<Time>,
        mut edges: Query<
            (
                &mut Transform,
                Option<&EdgePulse>,
                Option<&EdgeWave>,
                Option<&mut EdgeColorCycle>,
            ),
            With<EdgeVisual>,
        >,
    ) {
        let elapsed = time.elapsed_secs();

        for (mut transform, pulse, wave, color_cycle) in edges.iter_mut() {
            // Apply edge pulsing
            if let Some(pulse_anim) = pulse {
                let pulse_factor = 1.0
                    + pulse_anim.pulse_scale
                        * (elapsed * pulse_anim.pulse_speed + pulse_anim.phase_offset).sin();

                // Scale the edge thickness
                transform.scale.x = pulse_factor;
                transform.scale.y = pulse_factor;

                // Note: Material emissive changes would require accessing MeshMaterial3d component
                // This is handled separately if needed
            }

            // Apply wave animation
            if let Some(wave_anim) = wave {
                // Use wave frequency to create more complex wave patterns
                let wave_offset = wave_anim.wave_amplitude
                    * (elapsed * wave_anim.wave_speed + wave_anim.wave_offset).sin()
                    * (elapsed * wave_anim.wave_frequency).cos(); // Use frequency for modulation
                transform.translation.y += wave_offset;
            }

            // Apply color cycling
            if let Some(mut color_anim) = color_cycle {
                color_anim.current_phase += color_anim.cycle_speed * time.delta_secs();
                if color_anim.current_phase > 1.0 {
                    color_anim.current_phase -= 1.0;
                }

                // Note: To update material colors, we would need a separate system
                // that queries for both EdgeColorCycle and MeshMaterial3d components
            }
        }
    }

    /// Animates edge materials based on animation components
    pub fn animate_edge_materials(
        time: Res<Time>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        edges: Query<
            (
                &MeshMaterial3d<StandardMaterial>,
                Option<&EdgePulse>,
                Option<&EdgeColorCycle>,
                Option<&EdgeFlow>,
            ),
            With<EdgeVisual>,
        >,
    ) {
        let elapsed = time.elapsed_secs();

        for (material_handle, pulse, color_cycle, flow) in edges.iter() {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                // Apply pulse animation to emissive
                if let Some(pulse_anim) = pulse {
                    let intensity = pulse_anim.color_intensity
                        * (0.5
                            + 0.5
                                * (elapsed * pulse_anim.pulse_speed + pulse_anim.phase_offset)
                                    .sin());
                    material.emissive =
                        LinearRgba::rgb(intensity * 0.8, intensity * 0.8, intensity);
                }

                // Apply color cycling
                if let Some(color_anim) = color_cycle {
                    let t = color_anim.current_phase;
                    // Lerp between the two colors
                    let start_rgba = LinearRgba::from(color_anim.color_range.0);
                    let end_rgba = LinearRgba::from(color_anim.color_range.1);

                    material.base_color = Color::from(LinearRgba::new(
                        start_rgba.red + (end_rgba.red - start_rgba.red) * t,
                        start_rgba.green + (end_rgba.green - start_rgba.green) * t,
                        start_rgba.blue + (end_rgba.blue - start_rgba.blue) * t,
                        1.0,
                    ));
                }

                // Apply flow animation (visual indication through emissive pulsing along edge)
                if let Some(flow_anim) = flow {
                    // Create a moving pulse effect
                    let flow_position = (elapsed * flow_anim.flow_speed) % 1.0;
                    let pulse_intensity = if flow_anim.flow_direction {
                        (flow_position * std::f32::consts::PI * 2.0).sin().max(0.0)
                    } else {
                        ((1.0 - flow_position) * std::f32::consts::PI * 2.0)
                            .sin()
                            .max(0.0)
                    };

                    // Add to existing emissive
                    let current_emissive = material.emissive;
                    material.emissive = LinearRgba::rgb(
                        current_emissive.red + pulse_intensity * 0.3,
                        current_emissive.green + pulse_intensity * 0.3,
                        current_emissive.blue + pulse_intensity * 0.5,
                    );
                }
            }
        }
    }

    /// Start graph rotation on event
    pub fn handle_graph_animation_events(
        mut events: EventReader<GraphCreated>,
        mut graphs: Query<(Entity, &GraphIdentity)>,
        mut commands: Commands,
    ) {
        for event in events.read() {
            // Find the graph entity and add animation
            for (entity, identity) in graphs.iter_mut() {
                if identity.0 == event.graph.0 {
                    commands.entity(entity).insert(GraphMotion {
                        rotation_speed: 0.5, // Rotate at 0.5 rad/s
                        ..default()
                    });
                    info!("Started rotation animation for graph: {}", identity.0);
                    break;
                }
            }
        }
    }
}

/// Service to control camera movement
pub struct ControlCamera;

impl ControlCamera {
    /// Initialize camera for 3D graph viewing with Panorbit Camera
    pub fn setup_camera(mut commands: Commands) {
        // Camera with Panorbit controls
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            PanOrbitCamera {
                // Set the focus point
                focus: Vec3::ZERO,
                // Set camera movement settings
                radius: Some(10.0f32),
                // Control settings
                button_orbit: MouseButton::Right,
                button_pan: MouseButton::Middle,
                // Allow upside down camera
                allow_upside_down: false,
                ..default()
            },
        ));

        // Light
        commands.spawn((
            DirectionalLight {
                illuminance: 10000.0,
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ));
    }

    /// System to make billboards face camera
    pub fn update_billboards(
        camera: Query<&Transform, With<Camera3d>>,
        mut billboards: Query<&mut Transform, (With<Billboard>, Without<Camera3d>)>,
    ) {
        if let Ok(camera_transform) = camera.single() {
            for mut transform in billboards.iter_mut() {
                // Make billboard face camera while preserving its position
                let position = transform.translation;
                transform.look_at(camera_transform.translation, Vec3::Y);
                transform.translation = position;
            }
        }
    }
}

// TODO: These functions are incomplete and need to be implemented
/*
pub fn highlight_edge_on_hover(
    mut edge_hover_events: EventReader<EdgeHoverEvent>,
    mut edge_type_events: EventReader<EdgeTypeChanged>,
    mut edges: Query<(&EdgeRelationship, &EdgeVisual), With<EdgeBundle>>,
) {
    // Handle edge type changes
    for _event in edge_type_events.read() {

    }

    for (edge_entity, edge_relationship, _edge_visual) in edges.iter() {

    }
}

pub fn draw_graph_ui(
    mut contexts: EguiContexts,
    mut commands: Commands,
    _windows: Query<&Window>,
    _camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    _nodes: Query<&Transform, With<crate::contexts::graph_management::domain::Node>>,
    mut graph_state: ResMut<GraphState>,
) {
    self.reset();
}
*/
