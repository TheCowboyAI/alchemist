use bevy::prelude::*;
use crate::contexts::graph_management::domain::*;
use crate::contexts::graph_management::events::*;

// ============= Visualization Services =============
// Services that handle visual representation

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

/// Service to animate transitions between states
pub struct AnimateTransitions;

impl AnimateTransitions {
    /// Animates node movement - will need to track movement state separately
    pub fn animate_position(
        _time: Res<Time>,
        // TODO: Create a MovementAnimation component to track animations
    ) {
        // TODO: Implement smooth position interpolation
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
