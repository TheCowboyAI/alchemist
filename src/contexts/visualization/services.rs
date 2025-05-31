use bevy::prelude::*;
use crate::contexts::graph_management::domain::*;
use crate::contexts::graph_management::events::*;

// ============= Visualization Services =============
// Services that handle visual representation

// ============= Animation Components =============
// Components to track animations at different hierarchy levels

/// Animation state for entire graphs
#[derive(Component)]
pub struct GraphAnimation {
    pub rotation_speed: f32,
    pub oscillation_amplitude: f32,
    pub oscillation_frequency: f32,
    pub scale_factor: f32,
}

impl Default for GraphAnimation {
    fn default() -> Self {
        Self {
            rotation_speed: 0.0,
            oscillation_amplitude: 0.0,
            oscillation_frequency: 0.0,
            scale_factor: 1.0,
        }
    }
}

/// Animation state for subgraphs
#[derive(Component)]
pub struct SubgraphAnimation {
    pub local_rotation_speed: f32,
    pub orbit_radius: f32,
    pub orbit_speed: f32,
}

/// Animation state for individual nodes
#[derive(Component)]
pub struct NodeAnimation {
    pub bounce_height: f32,
    pub bounce_speed: f32,
    pub pulse_scale: f32,
    pub pulse_speed: f32,
}

/// Service to render graph elements in 3D space
pub struct RenderGraphElements;

impl RenderGraphElements {
    /// Creates visual representation for nodes
    pub fn render_node(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        node_entity: Entity,
        position: Vec3,
        _label: &str,
    ) {
        // Create a sphere mesh for the node
        let mesh = meshes.add(Sphere::new(0.3).mesh());

        // Create a blue material
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.9),
            metallic: 0.3,
            perceptual_roughness: 0.5,
            ..default()
        });

        // Add visual components to the node entity
        commands.entity(node_entity).insert(Mesh3d(mesh));
        commands.entity(node_entity).insert(MeshMaterial3d(material));
        commands.entity(node_entity).insert(Transform::from_translation(position));
    }

    /// System that listens for NodeAdded events and creates visual representations
    pub fn visualize_new_nodes(
        mut commands: Commands,
        mut events: EventReader<NodeAdded>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        nodes: Query<(Entity, &NodeIdentity, &NodeContent, &SpatialPosition)>,
    ) {
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
                    );
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
}

/// Service to animate graph elements at all hierarchy levels
pub struct AnimateGraphElements;

impl AnimateGraphElements {
    /// Animates entire graphs (rotation, oscillation, scaling)
    pub fn animate_graphs(
        time: Res<Time>,
        mut graphs: Query<(&mut Transform, &GraphAnimation), With<Graph>>,
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
        mut subgraphs: Query<(&mut Transform, &SubgraphAnimation), Without<Graph>>,
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
        mut nodes: Query<(&mut Transform, &NodeAnimation), With<crate::contexts::graph_management::domain::Node>>,
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
                    commands.entity(entity).insert(GraphAnimation {
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
