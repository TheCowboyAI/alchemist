//! Conceptual visualization systems
//!
//! Systems for rendering and managing conceptual graphs in 3D space,
//! including quality dimension mapping and interactive features.

use bevy::prelude::*;
use crate::presentation::components::{
    ConceptualNodeVisual, QualityDimensionAxis,
    ConceptualSpaceVisual, NodeVisualStyle, NodeShape, TransitionAnimation, EasingFunction,
    ConceptualVisualizationSettings, Highlighted, DraggableNode,
};
use crate::domain::conceptual_graph::{ConceptNode as DomainConceptNode, ConceptualPoint};
use crate::domain::value_objects::NodeId;

/// Wrapper component for domain ConceptNode
#[derive(Component)]
pub struct ConceptNodeEntity {
    pub node: DomainConceptNode,
}

/// Maps a conceptual point to 3D visual space
pub fn map_to_visual_space(
    point: &ConceptualPoint,
    space: &ConceptualSpaceVisual,
) -> Vec3 {
    let mut position = space.origin;

    // Map each dimension to its corresponding axis
    for (i, coord) in point.coordinates.iter().enumerate() {
        if let Some(axis) = space.dimensions.get(i) {
            position += axis.axis_direction * (*coord as f32) * axis.scale;
        }
    }

    // Clamp to space bounds
    position = position.clamp(space.bounds.min, space.bounds.max);

    position
}

/// Creates visual representation for concept nodes
pub fn visualize_conceptual_nodes(
    mut commands: Commands,
    new_concepts: Query<(Entity, &ConceptNodeEntity), Added<ConceptNodeEntity>>,
    space_query: Query<&ConceptualSpaceVisual>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<ConceptualVisualizationSettings>,
) {
    // Get the first conceptual space (for now)
    let space = match space_query.iter().next() {
        Some(s) => s,
        None => return,
    };

    for (entity, concept_entity) in new_concepts.iter() {
        let concept = &concept_entity.node;

        // Map conceptual position to visual space
        let visual_position = map_to_visual_space(concept.quality_position(), space);

        // Determine visual style based on node type
        let mut visual_style = NodeVisualStyle::default();
        visual_style.scale *= settings.node_scale;

        match concept {
            DomainConceptNode::Atom { .. } => {
                visual_style.shape = NodeShape::Sphere;
                visual_style.base_color = Color::srgb(0.3, 0.7, 0.9);
            }
            DomainConceptNode::Composite { .. } => {
                visual_style.shape = NodeShape::Cube;
                visual_style.base_color = Color::srgb(0.7, 0.5, 0.3);
            }
            DomainConceptNode::Function { .. } => {
                visual_style.shape = NodeShape::Cone;
                visual_style.base_color = Color::srgb(0.9, 0.3, 0.5);
            }
        }

        // Create mesh based on shape
        let mesh = match visual_style.shape {
            NodeShape::Sphere => meshes.add(Sphere::new(0.5 * visual_style.scale)),
            NodeShape::Cube => meshes.add(Cuboid::new(
                visual_style.scale,
                visual_style.scale,
                visual_style.scale,
            )),
            NodeShape::Cylinder => meshes.add(Cylinder::new(
                0.5 * visual_style.scale,
                visual_style.scale,
            )),
            NodeShape::Cone => meshes.add(Cone {
                radius: 0.5 * visual_style.scale,
                height: visual_style.scale,
            }),
            NodeShape::Torus => meshes.add(Torus {
                minor_radius: 0.2 * visual_style.scale,
                major_radius: 0.5 * visual_style.scale,
            }),
            NodeShape::Icosahedron => meshes.add(Sphere::new(0.5 * visual_style.scale)), // Fallback to sphere
        };

        // Create material
        let material = materials.add(StandardMaterial {
            base_color: visual_style.base_color,
            emissive: visual_style.emissive_color.into(),
            alpha_mode: if visual_style.alpha < 1.0 {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            },
            ..default()
        });

        // Add visual components to entity
        commands.entity(entity).insert((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(visual_position),
            ConceptualNodeVisual {
                concept_id: NodeId::new(),
                node_type: crate::presentation::components::ConceptNodeType::Atom {
                    category: "default".to_string(),
                    properties: Default::default(),
                },
                quality_position: concept.quality_position().clone(),
                visual_style,
                selected: false,
                hovered: false,
            },
            DraggableNode::default(),
        ));

        // Add smooth transition if enabled
        if settings.smooth_transitions {
            commands.entity(entity).insert(TransitionAnimation {
                start: visual_position + Vec3::Y * 5.0, // Start above
                end: visual_position,
                duration: 1.0,
                elapsed: 0.0,
                easing: EasingFunction::EaseOut,
            });
        }
    }
}

/// Creates quality dimension axes visualization
pub fn create_quality_dimension_axes(
    mut commands: Commands,
    space_query: Query<(Entity, &ConceptualSpaceVisual), Added<ConceptualSpaceVisual>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<ConceptualVisualizationSettings>,
) {
    if !settings.show_axes {
        return;
    }

    for (_space_entity, space) in space_query.iter() {
        // Create axis for each dimension
        for dimension in space.dimensions.iter() {
            let axis_direction = dimension.axis_direction;
            let axis_length = (space.bounds.max - space.bounds.min).length() * 0.5;

            // Create axis line mesh
            let axis_mesh = meshes.add(Cylinder::new(0.02, axis_length));

            // Create axis material with dimension color
            let axis_material = materials.add(StandardMaterial {
                base_color: dimension.color,
                emissive: LinearRgba::from(dimension.color) * 0.5,
                ..default()
            });

            // Calculate axis transform
            let axis_position = space.origin + axis_direction * (axis_length * 0.5);
            let mut transform = Transform::from_translation(axis_position);

            // Rotate cylinder to align with axis direction
            if axis_direction != Vec3::Y {
                let rotation = Quat::from_rotation_arc(Vec3::Y, axis_direction);
                transform.rotation = rotation;
            }

            // Spawn axis entity
            commands.spawn((
                Mesh3d(axis_mesh),
                MeshMaterial3d(axis_material.clone()),
                transform,
                Name::new(format!("Axis_{}", dimension.dimension.name)),
            ));

            // Create axis end marker (arrow head)
            let arrow_mesh = meshes.add(Cone {
                radius: 0.1,
                height: 0.2,
            });

            let arrow_position = space.origin + axis_direction * axis_length;
            let mut arrow_transform = Transform::from_translation(arrow_position);
            arrow_transform.rotation = transform.rotation;

            commands.spawn((
                Mesh3d(arrow_mesh),
                MeshMaterial3d(axis_material),
                arrow_transform,
                Name::new(format!("AxisArrow_{}", dimension.dimension.name)),
            ));

            // TODO: Add axis labels using text
        }
    }
}

/// Creates grid visualization for conceptual space
pub fn create_conceptual_grid(
    mut commands: Commands,
    space_query: Query<&ConceptualSpaceVisual, Added<ConceptualSpaceVisual>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<ConceptualVisualizationSettings>,
) {
    if !settings.show_grid {
        return;
    }

    for space in space_query.iter() {
        if !space.grid_settings.visible {
            continue;
        }

        // Create grid lines
        let grid_size = space.bounds.max - space.bounds.min;
        let subdivisions = space.grid_settings.subdivisions;

        // Grid material
        let grid_material = materials.add(StandardMaterial {
            base_color: space.grid_settings.color,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        // Create grid lines in XZ plane at origin Y
        for i in 0..=subdivisions {
            let t = i as f32 / subdivisions as f32;

            // X-direction lines
            let x_line_start = Vec3::new(
                space.bounds.min.x + t * grid_size.x,
                space.origin.y,
                space.bounds.min.z,
            );
            let x_line_end = Vec3::new(
                space.bounds.min.x + t * grid_size.x,
                space.origin.y,
                space.bounds.max.z,
            );

            // Z-direction lines
            let z_line_start = Vec3::new(
                space.bounds.min.x,
                space.origin.y,
                space.bounds.min.z + t * grid_size.z,
            );
            let z_line_end = Vec3::new(
                space.bounds.max.x,
                space.origin.y,
                space.bounds.min.z + t * grid_size.z,
            );

            // Create line meshes (using thin cylinders for now)
            let x_line_length = (x_line_end - x_line_start).length();
            let z_line_length = (z_line_end - z_line_start).length();

            let x_line_mesh = meshes.add(Cylinder::new(
                space.grid_settings.line_width,
                x_line_length,
            ));
            let z_line_mesh = meshes.add(Cylinder::new(
                space.grid_settings.line_width,
                z_line_length,
            ));

            // X-line transform
            let x_line_center = (x_line_start + x_line_end) * 0.5;
            let mut x_transform = Transform::from_translation(x_line_center);
            x_transform.rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);

            // Z-line transform
            let z_line_center = (z_line_start + z_line_end) * 0.5;
            let mut z_transform = Transform::from_translation(z_line_center);
            z_transform.rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);

            // Spawn grid lines
            commands.spawn((
                Mesh3d(x_line_mesh),
                MeshMaterial3d(grid_material.clone()),
                x_transform,
                Name::new(format!("GridLineX_{}", i)),
            ));

            commands.spawn((
                Mesh3d(z_line_mesh),
                MeshMaterial3d(grid_material.clone()),
                z_transform,
                Name::new(format!("GridLineZ_{}", i)),
            ));
        }
    }
}

/// Animates quality dimension transitions
pub fn animate_quality_dimensions(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &QualityDimensionAxis)>,
    settings: Res<ConceptualVisualizationSettings>,
) {
    let delta = time.delta_secs() * settings.animation_speed;

    for (mut transform, _axis) in query.iter_mut() {
        // Gentle rotation animation for visibility
        let rotation_speed = 0.1;
        transform.rotate_local_y(rotation_speed * delta);
    }
}

/// Updates transition animations
pub fn update_transition_animations(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut TransitionAnimation)>,
    settings: Res<ConceptualVisualizationSettings>,
) {
    let delta = time.delta_secs() * settings.animation_speed;

    for (entity, mut transform, mut animation) in query.iter_mut() {
        animation.elapsed += delta;

        if animation.elapsed >= animation.duration {
            // Animation complete
            transform.translation = animation.end;
            commands.entity(entity).remove::<TransitionAnimation>();
        } else {
            // Interpolate position
            let t = animation.elapsed / animation.duration;
            let eased_t = animation.easing.apply(t);
            transform.translation = animation.start.lerp(animation.end, eased_t);
        }
    }
}

/// Highlights nodes on hover
pub fn highlight_hovered_nodes(
    mut commands: Commands,
    mut node_query: Query<(
        Entity,
        &mut ConceptualNodeVisual,
        &MeshMaterial3d<StandardMaterial>,
        Option<&Highlighted>,
        &Transform,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    _mouse_button: Res<ButtonInput<MouseButton>>,
) {
    // Get cursor position
    let (camera, camera_transform) = match camera_query.get_single() {
        Ok(c) => c,
        Err(_) => return,
    };

    let window = match windows.get_single() {
        Ok(w) => w,
        Err(_) => return,
    };

    let cursor_position = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    // Cast ray from camera through cursor
    let ray = match camera.viewport_to_world(camera_transform, cursor_position) {
        Ok(r) => r,
        Err(_) => return,
    };

    // Simple distance-based hover detection (should use proper raycasting)
    let mut closest_entity = None;
    let mut closest_distance = f32::MAX;

    for (entity, node, _, _, transform) in node_query.iter() {
        // Use the node's quality position and transform to calculate distance
        let node_position = transform.translation;

        // Calculate distance from ray to node position
        // This is a simplified check - in production, use proper raycasting
        let ray_to_node = node_position - ray.origin;
        let ray_direction_normalized = ray.direction.normalize();
        let projection_length = ray_to_node.dot(ray_direction_normalized);

        if projection_length > 0.0 {
            let closest_point_on_ray = ray.origin + ray_direction_normalized * projection_length;
            let distance = (node_position - closest_point_on_ray).length();

            if distance < closest_distance && distance < 2.0 {
                closest_distance = distance;
                closest_entity = Some(entity);
            }
        }
    }

    // Update hover states
    for (entity, mut node, material_handle, highlighted, _) in node_query.iter_mut() {
        let should_hover = Some(entity) == closest_entity;

        if should_hover != node.hovered {
            node.hovered = should_hover;

            if should_hover {
                // Add highlight component
                commands.entity(entity).insert(Highlighted {
                    color: Color::srgb(1.0, 1.0, 0.0),
                    intensity: 0.5,
                });

                // Update material
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    material.emissive = LinearRgba::from(Color::srgb(0.5, 0.5, 0.0));
                }
            } else if highlighted.is_some() {
                // Remove highlight
                commands.entity(entity).remove::<Highlighted>();

                // Reset material
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    material.emissive = LinearRgba::BLACK;
                }
            }
        }
    }
}

/// Plugin for conceptual visualization
pub struct ConceptualVisualizationPlugin;

impl Plugin for ConceptualVisualizationPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ConceptualVisualizationSettings>()
            .add_systems(
                Update,
                (
                    visualize_conceptual_nodes,
                    create_quality_dimension_axes,
                    create_conceptual_grid,
                    animate_quality_dimensions,
                    update_transition_animations,
                    highlight_hovered_nodes,
                ),
            );
    }
}
