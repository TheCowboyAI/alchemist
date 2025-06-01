use crate::contexts::graph_management::domain::*;
use crate::contexts::graph_management::events::*;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

// ============= Visualization Services =============
// Services that handle visual representation

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

/// Event fired when a node is selected
#[derive(Event, Debug, Clone)]
pub struct NodeSelected {
    pub entity: Entity,
    pub node: NodeIdentity,
}

/// Event fired when a node is deselected
#[derive(Event, Debug, Clone)]
pub struct NodeDeselected {
    pub entity: Entity,
    pub node: NodeIdentity,
}

/// Component to mark selected entities
#[derive(Component)]
pub struct Selected;

/// Component to store original material for restoration on deselection
#[derive(Component)]
pub struct OriginalMaterial(pub Handle<StandardMaterial>);

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
            let theta = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
            let phi = (rand::random::<f32>() * 2.0 - 1.0).acos();
            let r = radius * rand::random::<f32>().powf(1.0 / 3.0);

            let x = r * phi.sin() * theta.cos();
            let y = r * phi.sin() * theta.sin();
            let z = r * phi.cos();

            points.push(position + Vec3::new(x, y, z));
            colors.push(Color::srgb(0.3, 0.5, 0.9));
            sizes.push(0.02 + rand::random::<f32>() * 0.03);
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
                    (rand::random::<f32>() - 0.5) * 0.1,
                    (rand::random::<f32>() - 0.5) * 0.1,
                    (rand::random::<f32>() - 0.5) * 0.1,
                );

                points.push(base_point + jitter);
                colors.push(Color::srgb(0.7, 0.7, 0.7));
                sizes.push(0.01 + rand::random::<f32>() * 0.02);
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
        let random = rand::random::<f32>();

        if random < 0.3 {
            // 30% chance of pulse animation
            commands.entity(edge_entity).insert(EdgePulse {
                phase_offset: rand::random::<f32>() * std::f32::consts::PI * 2.0,
                ..default()
            });
        } else if random < 0.5 {
            // 20% chance of wave animation
            commands.entity(edge_entity).insert(EdgeWave {
                wave_offset: rand::random::<f32>() * std::f32::consts::PI * 2.0,
                ..default()
            });
        } else if random < 0.7 {
            // 20% chance of color cycle
            commands
                .entity(edge_entity)
                .insert(EdgeColorCycle::default());
        }
        // 30% chance of no animation
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

        let mesh = meshes.add(Cuboid::new(
            edge_visual.thickness,
            edge_visual.thickness,
            length,
        ));

        // Calculate rotation to align with edge direction
        let rotation = Quat::from_rotation_arc(Vec3::Z, direction.normalize());

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

        // Create arc mesh using line segments
        let segments = 20;
        let mut positions = Vec::new();
        let mut indices = Vec::new();

        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let angle = t * std::f32::consts::PI;

            // Interpolate along chord and add arc height
            let pos = source_pos + chord * t + Vec3::Y * (angle.sin() * arc_height);
            positions.push([pos.x, pos.y, pos.z]);

            if i < segments {
                // Create line segments
                indices.push(i as u32);
                indices.push((i + 1) as u32);
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::RENDER_WORLD);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_indices(Indices::U32(indices));

        commands
            .entity(edge_entity)
            .insert(Mesh3d(meshes.add(mesh)))
            .insert(MeshMaterial3d(material))
            .insert(Transform::default())
            .insert(edge_visual.clone());
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
        let offset = (target_pos - source_pos).cross(Vec3::Y).normalize() * 0.5;

        let control1 = midpoint + offset + Vec3::Y * 0.3;
        let control2 = midpoint - offset + Vec3::Y * 0.3;

        // Generate bezier curve points
        let segments = 30;
        let mut positions = Vec::new();
        let mut indices = Vec::new();

        for i in 0..=segments {
            let t = i as f32 / segments as f32;

            // Cubic bezier curve formula
            let pos = (1.0 - t).powi(3) * source_pos
                + 3.0 * (1.0 - t).powi(2) * t * control1
                + 3.0 * (1.0 - t) * t.powi(2) * control2
                + t.powi(3) * target_pos;

            positions.push([pos.x, pos.y, pos.z]);

            if i < segments {
                indices.push(i as u32);
                indices.push((i + 1) as u32);
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::RENDER_WORLD);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_indices(Indices::U32(indices));

        commands
            .entity(edge_entity)
            .insert(Mesh3d(meshes.add(mesh)))
            .insert(MeshMaterial3d(material))
            .insert(Transform::default())
            .insert(edge_visual.clone());
    }

    /// Creates visual representation for nodes
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

                commands
                    .entity(node_entity)
                    .insert(Mesh3d(mesh))
                    .insert(MeshMaterial3d(material))
                    .insert(Transform::from_translation(position))
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
                    .insert(Transform::from_translation(position))
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
                    .insert(Transform::from_translation(position))
                    .insert(VisualizationCapability {
                        render_mode,
                        ..default()
                    });

                info!("Wireframe mode enabled for node");
            }
            RenderMode::Billboard => {
                // Billboard rendering - text that always faces camera
                let text_style = TextFont {
                    font_size: 20.0,
                    ..default()
                };

                commands.entity(node_entity).insert((
                    Text2d::new(_label),
                    text_style,
                    Transform::from_translation(position),
                    Billboard,
                    VisualizationCapability {
                        render_mode,
                        ..default()
                    },
                ));

                info!("Billboard mode enabled for node: {}", _label);
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
        // Get current settings or use defaults
        let render_mode = settings
            .single()
            .map(|s| s.render_mode)
            .unwrap_or(RenderMode::Mesh);

        for event in events.read() {
            info!("Visualizing node: {}", event.content.label);

            // Find the entity that was just created
            for (entity, identity, content, position) in nodes.iter() {
                if identity.0 == event.node.0 {
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
        // Get current settings or use defaults
        let edge_type = settings
            .single()
            .map(|s| s.edge_type)
            .unwrap_or(EdgeType::Cylinder);

        for event in events.read() {
            info!("Visualizing edge: {:?}", event.edge);

            // Find the edge entity that was just created
            for (edge_entity, edge_identity, relationship) in edges.iter() {
                if edge_identity.0 == event.edge.0 {
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
        }
    }
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
    /// Process mouse clicks for selection
    pub fn process_selection(
        windows: Query<&Window>,
        camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
        nodes: Query<
            (Entity, &Transform, &NodeIdentity),
            With<crate::contexts::graph_management::domain::Node>,
        >,
        mouse_button: Res<ButtonInput<MouseButton>>,
        mut events: EventWriter<NodeSelected>,
    ) {
        if mouse_button.just_pressed(MouseButton::Left) {
            // Get the primary window
            let Ok(window) = windows.single() else { return };

            // Get camera info
            let Ok((camera, camera_transform)) = camera.single() else {
                return;
            };

            // Get cursor position
            let Some(cursor_position) = window.cursor_position() else {
                return;
            };

            // Convert screen position to ray
            let Some(ray) =
                PerformRaycast::screen_to_ray(camera, camera_transform, cursor_position)
            else {
                return;
            };

            // Find the closest intersecting node
            let mut closest_hit: Option<(Entity, NodeIdentity, f32)> = None;

            for (entity, transform, node_id) in nodes.iter() {
                let sphere_center = transform.translation;
                let sphere_radius = 0.3; // Match the sphere radius used in rendering

                if let Some(distance) =
                    PerformRaycast::ray_intersects_sphere(&ray, sphere_center, sphere_radius)
                {
                    match &closest_hit {
                        None => closest_hit = Some((entity, *node_id, distance)),
                        Some((_, _, closest_distance)) => {
                            if distance < *closest_distance {
                                closest_hit = Some((entity, *node_id, distance));
                            }
                        }
                    }
                }
            }

            // Emit selection event for the closest hit
            if let Some((entity, node_id, _)) = closest_hit {
                events.write(NodeSelected {
                    entity,
                    node: node_id,
                });
                info!("Node selected: {:?}", node_id);
            }
        }
    }

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
}

/// Service to handle selection visualization
pub struct SelectionVisualization;

impl SelectionVisualization {
    /// Updates visual appearance when nodes are selected
    pub fn handle_node_selection(
        mut commands: Commands,
        mut events: EventReader<NodeSelected>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        query: Query<&MeshMaterial3d<StandardMaterial>, Without<Selected>>,
    ) {
        for event in events.read() {
            if let Ok(material_handle) = query.get(event.entity) {
                // Store original material
                commands
                    .entity(event.entity)
                    .insert(Selected)
                    .insert(OriginalMaterial(material_handle.0.clone()));

                // Create highlight material
                let highlight_material = materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.8, 0.2), // Golden highlight
                    emissive: LinearRgba::rgb(0.5, 0.4, 0.1),
                    metallic: 0.5,
                    perceptual_roughness: 0.3,
                    ..default()
                });

                // Apply highlight material
                commands
                    .entity(event.entity)
                    .insert(MeshMaterial3d(highlight_material));

                info!("Applied selection highlight to entity: {:?}", event.entity);
            }
        }
    }

    /// Removes visual feedback when nodes are deselected
    pub fn handle_node_deselection(
        mut commands: Commands,
        mut events: EventReader<NodeDeselected>,
        query: Query<&OriginalMaterial, With<Selected>>,
    ) {
        for event in events.read() {
            if let Ok(original_material) = query.get(event.entity) {
                // Restore original material
                commands
                    .entity(event.entity)
                    .remove::<Selected>()
                    .insert(MeshMaterial3d(original_material.0.clone()))
                    .remove::<OriginalMaterial>();

                info!(
                    "Removed selection highlight from entity: {:?}",
                    event.entity
                );
            }
        }
    }

    /// System to handle clicking on empty space to deselect
    pub fn handle_deselect_all(
        mouse_button: Res<ButtonInput<MouseButton>>,
        selected: Query<(Entity, &NodeIdentity), With<Selected>>,
        mut deselect_events: EventWriter<NodeDeselected>,
        // Check if we clicked on nothing
        windows: Query<&Window>,
        camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
        nodes: Query<&Transform, With<crate::contexts::graph_management::domain::Node>>,
    ) {
        if mouse_button.just_pressed(MouseButton::Right) {
            // Deselect all on right click
            for (entity, node_id) in selected.iter() {
                deselect_events.write(NodeDeselected {
                    entity,
                    node: *node_id,
                });
            }
            info!("Deselected all nodes");
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
                let wave_offset = wave_anim.wave_amplitude
                    * (elapsed * wave_anim.wave_speed + wave_anim.wave_offset).sin();
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
    /// Initialize camera for 3D graph viewing
    pub fn setup_camera(mut commands: Commands) {
        // Camera
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
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

    /// Handle camera orbit controls
    pub fn orbit_camera(
        time: Res<Time>,
        mut query: Query<&mut Transform, With<Camera3d>>,
        input: Res<ButtonInput<KeyCode>>,
    ) {
        if let Ok(mut camera_transform) = query.single_mut() {
            let rotation_speed = 2.0 * time.delta().as_secs_f32();

            if input.pressed(KeyCode::ArrowLeft) {
                let rotation = Quat::from_rotation_y(rotation_speed);
                camera_transform.translation = rotation * camera_transform.translation;
                *camera_transform = camera_transform.looking_at(Vec3::ZERO, Vec3::Y);
            }
            if input.pressed(KeyCode::ArrowRight) {
                let rotation = Quat::from_rotation_y(-rotation_speed);
                camera_transform.translation = rotation * camera_transform.translation;
                *camera_transform = camera_transform.looking_at(Vec3::ZERO, Vec3::Y);
            }
        }
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
