use bevy::prelude::*;
use bevy::render::mesh::{PrimitiveTopology, Indices};
use bevy::render::render_asset::RenderAssetUsages;
use crate::contexts::graph_management::domain::*;
use crate::contexts::graph_management::events::*;

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

/// Capabilities for visualization
#[derive(Component, Clone)]
pub struct VisualizationCapability {
    pub render_mode: RenderMode,
    pub supports_instancing: bool,
    pub level_of_detail: Option<u8>,
    pub point_cloud_density: Option<f32>, // Points per unit for point cloud mode
}

impl Default for VisualizationCapability {
    fn default() -> Self {
        Self {
            render_mode: RenderMode::default(),
            supports_instancing: false,
            level_of_detail: None,
            point_cloud_density: None,
        }
    }
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
    Line,      // Simple straight line
    Cylinder,  // 3D cylinder (current implementation)
    Arc,       // Curved arc
    Bezier,    // Bezier curve
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

/// Pulse dynamics for individual nodes
#[derive(Component)]
pub struct NodePulse {
    pub bounce_height: f32,
    pub bounce_speed: f32,
    pub pulse_scale: f32,
    pub pulse_speed: f32,
}

/// Service to render graph elements in 3D space
pub struct RenderGraphElements;

impl RenderGraphElements {
    /// Generates point cloud data for a node
    pub fn generate_node_point_cloud(
        position: Vec3,
        density: f32,
        radius: f32,
    ) -> NodePointCloud {
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
                commands, meshes, material, source_pos, target_pos, edge_entity, &edge_visual
            ),
            EdgeType::Cylinder => Self::render_cylinder_edge(
                commands, meshes, material, source_pos, target_pos, edge_entity, &edge_visual
            ),
            EdgeType::Arc => Self::render_arc_edge(
                commands, meshes, material, source_pos, target_pos, edge_entity, &edge_visual
            ),
            EdgeType::Bezier => Self::render_bezier_edge(
                commands, meshes, material, source_pos, target_pos, edge_entity, &edge_visual
            ),
        }
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

        let mesh = meshes.add(Cuboid::new(edge_visual.thickness, edge_visual.thickness, length));

        // Calculate rotation to align with edge direction
        let rotation = Quat::from_rotation_arc(Vec3::Z, direction.normalize());

        commands.entity(edge_entity)
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

        commands.entity(edge_entity)
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

        commands.entity(edge_entity)
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

        commands.entity(edge_entity)
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

                commands.entity(node_entity)
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

                commands.entity(node_entity)
                    .insert(Transform::from_translation(position))
                    .insert(point_cloud)
                    .insert(VisualizationCapability {
                        render_mode,
                        point_cloud_density: Some(50.0),
                        ..default()
                    });

                // Note: Actual point cloud rendering would be implemented in a dedicated plugin
                info!("Point cloud data generated for node - rendering requires point cloud plugin");
            }
            RenderMode::Wireframe => {
                // Wireframe rendering - use mesh with wireframe material
                let mesh = meshes.add(Sphere::new(0.3).mesh());

                let material = materials.add(StandardMaterial {
                    base_color: Color::srgb(0.3, 0.5, 0.9),
                    metallic: 0.0,
                    perceptual_roughness: 1.0,
                    alpha_mode: AlphaMode::Opaque,
                    ..default()
                });

                commands.entity(node_entity)
                    .insert(Mesh3d(mesh))
                    .insert(MeshMaterial3d(material))
                    .insert(Transform::from_translation(position))
                    .insert(VisualizationCapability {
                        render_mode,
                        ..default()
                    });
            }
            RenderMode::Billboard => {
                // Billboard rendering - sprite that always faces camera
                commands.entity(node_entity)
                    .insert(Transform::from_translation(position))
                    .insert(VisualizationCapability {
                        render_mode,
                        ..default()
                    });

                // Note: Billboard rendering would need custom shader or sprite setup
                info!("Billboard mode selected - requires custom implementation");
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
            .get_single()
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
            .get_single()
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

/// Service to handle user input and interactions
pub struct HandleUserInput;

impl HandleUserInput {
    /// Process mouse clicks for selection
    pub fn process_selection(
        _windows: Query<&Window>,
        _camera: Query<(&Camera, &GlobalTransform)>,
        _nodes: Query<(Entity, &Transform, &NodeIdentity)>,
        mouse_button: Res<ButtonInput<MouseButton>>,
    ) {
        if !mouse_button.just_pressed(MouseButton::Left) {
            return;
        }

        // TODO: Implement raycasting for node selection
    }

    /// Change edge rendering type with keyboard
    pub fn change_edge_type(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut events: EventWriter<EdgeTypeChanged>,
    ) {
        if keyboard.just_pressed(KeyCode::Digit1) {
            events.send(EdgeTypeChanged { new_edge_type: EdgeType::Line });
            info!("Edge type changed to: Line");
        } else if keyboard.just_pressed(KeyCode::Digit2) {
            events.send(EdgeTypeChanged { new_edge_type: EdgeType::Cylinder });
            info!("Edge type changed to: Cylinder");
        } else if keyboard.just_pressed(KeyCode::Digit3) {
            events.send(EdgeTypeChanged { new_edge_type: EdgeType::Arc });
            info!("Edge type changed to: Arc");
        } else if keyboard.just_pressed(KeyCode::Digit4) {
            events.send(EdgeTypeChanged { new_edge_type: EdgeType::Bezier });
            info!("Edge type changed to: Bezier");
        }
    }

    /// Change render mode with keyboard
    pub fn change_render_mode(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut events: EventWriter<RenderModeChanged>,
    ) {
        if keyboard.just_pressed(KeyCode::KeyM) {
            events.send(RenderModeChanged { new_render_mode: RenderMode::Mesh });
            info!("Render mode changed to: Mesh");
        } else if keyboard.just_pressed(KeyCode::KeyP) {
            events.send(RenderModeChanged { new_render_mode: RenderMode::PointCloud });
            info!("Render mode changed to: PointCloud (requires point cloud plugin)");
        } else if keyboard.just_pressed(KeyCode::KeyW) {
            events.send(RenderModeChanged { new_render_mode: RenderMode::Wireframe });
            info!("Render mode changed to: Wireframe");
        } else if keyboard.just_pressed(KeyCode::KeyB) {
            events.send(RenderModeChanged { new_render_mode: RenderMode::Billboard });
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
        mut nodes: Query<(&mut Transform, &NodePulse), With<crate::contexts::graph_management::domain::Node>>,
    ) {
        for (mut transform, animation) in nodes.iter_mut() {
            let elapsed = time.elapsed_secs();

            // Apply bouncing
            if animation.bounce_height > 0.0 {
                let bounce = animation.bounce_height
                    * (elapsed * animation.bounce_speed).sin().abs();
                transform.translation.y += bounce;
            }

            // Apply pulsing
            if animation.pulse_scale > 0.0 {
                let pulse = 1.0 + animation.pulse_scale
                    * (elapsed * animation.pulse_speed).sin();
                transform.scale = Vec3::splat(pulse);
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
            Transform::from_xyz(0.0, 5.0, 10.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
        ));

        // Light
        commands.spawn((
            DirectionalLight {
                illuminance: 10000.0,
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 8.0, 4.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
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
}
