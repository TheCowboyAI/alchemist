//! Context bridge visualization system
//!
//! Visualizes relationships and mappings between different bounded contexts
//! in the conceptual graph system.

use bevy::prelude::*;
use crate::presentation::components::{
    ConceptualSpaceVisual, ConceptRelationship, EdgeVisualStyle, ContextBridgeComponent,
};
use crate::domain::conceptual_graph::{ContextBridge, ContextMappingType};
use crate::domain::value_objects::{NodeId, EdgeId};

/// Visual representation of a context bridge
#[derive(Component, Debug, Clone)]
pub struct ContextBridgeVisual {
    /// Unique identifier for this bridge
    pub bridge_id: BridgeId,

    /// Source context entity
    pub source_context: Entity,

    /// Target context entity
    pub target_context: Entity,

    /// Start position in 3D space
    pub source_position: Vec3,

    /// End position in 3D space
    pub target_position: Vec3,

    /// Type of context mapping
    pub mapping_type: ContextMappingType,

    /// Visual style for the bridge
    pub visual_style: BridgeVisualStyle,

    /// Animation state
    pub animation_state: BridgeAnimationState,
}

/// Unique identifier for a context bridge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BridgeId(pub uuid::Uuid);

impl BridgeId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

/// Visual style for context bridges
#[derive(Debug, Clone)]
pub struct BridgeVisualStyle {
    /// Primary color
    pub color: Color,

    /// Secondary color for bidirectional bridges
    pub secondary_color: Option<Color>,

    /// Width of the bridge visualization
    pub width: f32,

    /// Pattern for the bridge (solid, dashed, etc.)
    pub pattern: BridgePattern,

    /// Glow intensity
    pub glow_intensity: f32,

    /// Whether to show flow direction
    pub show_flow: bool,
}

impl Default for BridgeVisualStyle {
    fn default() -> Self {
        Self {
            color: Color::srgb(0.5, 0.7, 0.9),
            secondary_color: None,
            width: 0.2,
            pattern: BridgePattern::Solid,
            glow_intensity: 0.3,
            show_flow: true,
        }
    }
}

/// Bridge visual patterns
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BridgePattern {
    Solid,
    Dashed,
    Dotted,
    Wave,
    Pulse,
}

/// Animation state for bridges
#[derive(Debug, Clone)]
pub struct BridgeAnimationState {
    /// Current animation progress (0.0 to 1.0)
    pub progress: f32,

    /// Animation speed
    pub speed: f32,

    /// Whether animation is active
    pub active: bool,

    /// Flow particles
    pub flow_particles: Vec<FlowParticle>,
}

impl Default for BridgeAnimationState {
    fn default() -> Self {
        Self {
            progress: 0.0,
            speed: 1.0,
            active: true,
            flow_particles: Vec::new(),
        }
    }
}

/// Particle for visualizing data flow
#[derive(Debug, Clone)]
pub struct FlowParticle {
    /// Position along the bridge (0.0 to 1.0)
    pub position: f32,

    /// Particle size
    pub size: f32,

    /// Particle color
    pub color: Color,

    /// Particle speed multiplier
    pub speed: f32,
}

/// Creates visual representations for context bridges
pub fn visualize_context_bridges(
    mut commands: Commands,
    bridges: Query<&ContextBridgeComponent, Added<ContextBridgeComponent>>,
    contexts: Query<(&ConceptualSpaceVisual, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for bridge_component in bridges.iter() {
        let bridge = &bridge_component.bridge;

        // Find source and target context positions
        let (source_pos, target_pos) = match find_context_positions(&contexts, bridge_component) {
            Some(positions) => positions,
            None => continue,
        };

        // Determine visual style based on mapping type
        let visual_style = create_bridge_style(&bridge.mapping_type);

        // Create bridge visual component
        let bridge_visual = ContextBridgeVisual {
            bridge_id: BridgeId::new(),
            source_context: Entity::PLACEHOLDER, // Would be set properly
            target_context: Entity::PLACEHOLDER, // Would be set properly
            source_position: source_pos,
            target_position: target_pos,
            mapping_type: bridge.mapping_type.clone(),
            visual_style,
            animation_state: BridgeAnimationState::default(),
        };

        // Create bridge mesh
        let bridge_mesh = create_bridge_mesh(source_pos, target_pos, &bridge_visual.visual_style);
        let mesh_handle = meshes.add(bridge_mesh);

        // Create bridge material
        let material = materials.add(StandardMaterial {
            base_color: bridge_visual.visual_style.color,
            emissive: LinearRgba::from(bridge_visual.visual_style.color)
                * bridge_visual.visual_style.glow_intensity,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        // Spawn bridge entity
        commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material),
            Transform::from_translation((source_pos + target_pos) * 0.5),
            bridge_visual,
            Name::new(format!("ContextBridge_{:?}", bridge.mapping_type)),
        ));
    }
}

/// Updates bridge animations
pub fn animate_context_bridges(
    time: Res<Time>,
    mut bridges: Query<(&mut ContextBridgeVisual, &mut Transform)>,
    mut gizmos: Gizmos,
) {
    let delta = time.delta_secs();

    for (mut bridge, _transform) in bridges.iter_mut() {
        if !bridge.animation_state.active {
            continue;
        }

        // Update animation progress
        bridge.animation_state.progress += delta * bridge.animation_state.speed;
        if bridge.animation_state.progress > 1.0 {
            bridge.animation_state.progress -= 1.0;
        }

        // Update flow particles
        update_flow_particles(&mut bridge.animation_state, delta);

        // Draw bridge with gizmos for dynamic effects
        draw_bridge_gizmos(&mut gizmos, &bridge);
    }
}

/// Draws bridge visualizations using gizmos
fn draw_bridge_gizmos(gizmos: &mut Gizmos, bridge: &ContextBridgeVisual) {
    let start = bridge.source_position;
    let end = bridge.target_position;
    let direction = (end - start).normalize();

    match bridge.visual_style.pattern {
        BridgePattern::Solid => {
            gizmos.line(start, end, bridge.visual_style.color);
        }
        BridgePattern::Dashed => {
            draw_dashed_line(gizmos, start, end, bridge.visual_style.color, 10);
        }
        BridgePattern::Dotted => {
            draw_dotted_line(gizmos, start, end, bridge.visual_style.color, 20);
        }
        BridgePattern::Wave => {
            draw_wave_line(gizmos, start, end, bridge.visual_style.color, bridge.animation_state.progress);
        }
        BridgePattern::Pulse => {
            let pulse_color = bridge.visual_style.color
                .with_alpha(0.5 + 0.5 * (bridge.animation_state.progress * std::f32::consts::TAU).sin());
            gizmos.line(start, end, pulse_color);
        }
    }

    // Draw flow particles
    for particle in &bridge.animation_state.flow_particles {
        let pos = start.lerp(end, particle.position);
        gizmos.sphere(pos, particle.size, particle.color);
    }

    // Draw mapping type indicator
    draw_mapping_indicator(gizmos, &bridge.mapping_type, start, end, direction);
}

/// Draws mapping type indicators
fn draw_mapping_indicator(
    gizmos: &mut Gizmos,
    mapping_type: &ContextMappingType,
    start: Vec3,
    end: Vec3,
    direction: Vec3,
) {
    let mid_point = (start + end) * 0.5;
    let perpendicular = direction.cross(Vec3::Y).normalize();

    match mapping_type {
        ContextMappingType::SharedKernel { .. } => {
            // Draw overlapping circles
            let rotation = Quat::from_rotation_arc(Vec3::Z, perpendicular);
            gizmos.circle(
                Isometry3d::new(mid_point - perpendicular * 0.3, rotation),
                0.2,
                Color::srgb(0.8, 0.8, 0.3)
            );
            gizmos.circle(
                Isometry3d::new(mid_point + perpendicular * 0.3, rotation),
                0.2,
                Color::srgb(0.8, 0.8, 0.3)
            );
        }
        ContextMappingType::CustomerSupplier { .. } => {
            // Draw arrow from supplier to customer
            draw_arrow(gizmos, start, end, Color::srgb(0.3, 0.8, 0.3));
        }
        ContextMappingType::Conformist { .. } => {
            // Draw one-way arrow with submission indicator
            draw_arrow(gizmos, start, end, Color::srgb(0.8, 0.3, 0.3));
            let rotation = Quat::from_rotation_arc(Vec3::Z, direction);
            gizmos.circle(
                Isometry3d::new(end - direction * 0.5, rotation),
                0.1,
                Color::srgb(0.8, 0.3, 0.3)
            );
        }
        ContextMappingType::AntiCorruptionLayer { .. } => {
            // Draw shield symbol
            draw_shield(gizmos, mid_point, perpendicular, Color::srgb(0.3, 0.3, 0.8));
        }
        ContextMappingType::OpenHostService { .. } => {
            // Draw broadcast symbol
            for i in 0..3 {
                let angle = i as f32 * std::f32::consts::TAU / 3.0;
                let offset = Quat::from_rotation_y(angle) * perpendicular * 0.3;
                gizmos.line(mid_point, mid_point + offset, Color::srgb(0.5, 0.8, 0.5));
            }
        }
        ContextMappingType::PublishedLanguage { .. } => {
            // Draw book/document symbol
            gizmos.rect(
                Isometry3d::new(mid_point, Quat::from_rotation_arc(Vec3::Z, direction)),
                Vec2::new(0.3, 0.2),
                Color::srgb(0.7, 0.5, 0.3)
            );
        }
        ContextMappingType::Partnership { .. } => {
            // Draw bidirectional arrows
            draw_arrow(gizmos, start, end, Color::srgb(0.5, 0.5, 0.8));
            draw_arrow(gizmos, end, start, Color::srgb(0.5, 0.5, 0.8));
        }
    }
}

/// Helper function to draw dashed lines
fn draw_dashed_line(gizmos: &mut Gizmos, start: Vec3, end: Vec3, color: Color, segments: u32) {
    let step = 1.0 / segments as f32;
    for i in 0..segments {
        if i % 2 == 0 {
            let t1 = i as f32 * step;
            let t2 = ((i + 1) as f32 * step).min(1.0);
            gizmos.line(start.lerp(end, t1), start.lerp(end, t2), color);
        }
    }
}

/// Helper function to draw dotted lines
fn draw_dotted_line(gizmos: &mut Gizmos, start: Vec3, end: Vec3, color: Color, dots: u32) {
    for i in 0..=dots {
        let t = i as f32 / dots as f32;
        let pos = start.lerp(end, t);
        gizmos.sphere(pos, 0.02, color);
    }
}

/// Helper function to draw wave lines
fn draw_wave_line(gizmos: &mut Gizmos, start: Vec3, end: Vec3, color: Color, progress: f32) {
    let segments = 50;
    let perpendicular = (end - start).cross(Vec3::Y).normalize();

    for i in 0..segments {
        let t1 = i as f32 / segments as f32;
        let t2 = (i + 1) as f32 / segments as f32;

        let wave1 = (t1 * std::f32::consts::TAU * 4.0 + progress * std::f32::consts::TAU).sin() * 0.1;
        let wave2 = (t2 * std::f32::consts::TAU * 4.0 + progress * std::f32::consts::TAU).sin() * 0.1;

        let p1 = start.lerp(end, t1) + perpendicular * wave1;
        let p2 = start.lerp(end, t2) + perpendicular * wave2;

        gizmos.line(p1, p2, color);
    }
}

/// Helper function to draw arrows
fn draw_arrow(gizmos: &mut Gizmos, start: Vec3, end: Vec3, color: Color) {
    gizmos.line(start, end, color);

    let direction = (end - start).normalize();
    let perpendicular = direction.cross(Vec3::Y).normalize();
    let arrow_size = 0.2;

    gizmos.line(end, end - direction * arrow_size + perpendicular * arrow_size * 0.5, color);
    gizmos.line(end, end - direction * arrow_size - perpendicular * arrow_size * 0.5, color);
}

/// Helper function to draw shield symbol
fn draw_shield(gizmos: &mut Gizmos, center: Vec3, normal: Vec3, color: Color) {
    let points = 6;
    for i in 0..points {
        let angle1 = i as f32 * std::f32::consts::TAU / points as f32;
        let angle2 = (i + 1) as f32 * std::f32::consts::TAU / points as f32;

        let p1 = center + Quat::from_rotation_y(angle1) * normal * 0.2;
        let p2 = center + Quat::from_rotation_y(angle2) * normal * 0.2;

        gizmos.line(p1, p2, color);
    }
}

/// Updates flow particles along bridges
fn update_flow_particles(animation_state: &mut BridgeAnimationState, delta: f32) {
    // Update existing particles
    for particle in animation_state.flow_particles.iter_mut() {
        particle.position += delta * particle.speed * 0.3;
        if particle.position > 1.0 {
            particle.position -= 1.0;
        }
    }

    // Spawn new particles periodically
    if animation_state.flow_particles.len() < 5 && animation_state.progress % 0.2 < delta {
        animation_state.flow_particles.push(FlowParticle {
            position: 0.0,
            size: 0.05,
            color: Color::srgb(1.0, 1.0, 0.8),
            speed: 0.8 + rand::random::<f32>() * 0.4,
        });
    }
}

/// Creates a mesh for the bridge visualization
fn create_bridge_mesh(start: Vec3, end: Vec3, style: &BridgeVisualStyle) -> Mesh {
    // For now, create a simple cylinder mesh
    // In a full implementation, this would create more complex geometry
    let length = start.distance(end);
    Cylinder::new(style.width * 0.5, length).into()
}

/// Finds positions of contexts for bridge endpoints
fn find_context_positions(
    contexts: &Query<(&ConceptualSpaceVisual, &Transform)>,
    _bridge: &ContextBridgeComponent,
) -> Option<(Vec3, Vec3)> {
    // This is a placeholder - in a real implementation, we'd look up
    // the actual context entities based on the bridge's source and target
    let positions: Vec<Vec3> = contexts
        .iter()
        .map(|(_, transform)| transform.translation)
        .collect();

    if positions.len() >= 2 {
        Some((positions[0], positions[1]))
    } else {
        None
    }
}

/// Creates visual style based on mapping type
fn create_bridge_style(mapping_type: &ContextMappingType) -> BridgeVisualStyle {
    match mapping_type {
        ContextMappingType::SharedKernel { .. } => BridgeVisualStyle {
            color: Color::srgb(0.8, 0.8, 0.3),
            pattern: BridgePattern::Solid,
            width: 0.3,
            glow_intensity: 0.5,
            ..default()
        },
        ContextMappingType::CustomerSupplier { .. } => BridgeVisualStyle {
            color: Color::srgb(0.3, 0.8, 0.3),
            pattern: BridgePattern::Dashed,
            show_flow: true,
            ..default()
        },
        ContextMappingType::Conformist { .. } => BridgeVisualStyle {
            color: Color::srgb(0.8, 0.3, 0.3),
            pattern: BridgePattern::Dotted,
            ..default()
        },
        ContextMappingType::AntiCorruptionLayer { .. } => BridgeVisualStyle {
            color: Color::srgb(0.3, 0.3, 0.8),
            pattern: BridgePattern::Wave,
            width: 0.4,
            glow_intensity: 0.7,
            ..default()
        },
        ContextMappingType::OpenHostService { .. } => BridgeVisualStyle {
            color: Color::srgb(0.5, 0.8, 0.5),
            pattern: BridgePattern::Pulse,
            show_flow: true,
            ..default()
        },
        ContextMappingType::PublishedLanguage { .. } => BridgeVisualStyle {
            color: Color::srgb(0.7, 0.5, 0.3),
            pattern: BridgePattern::Solid,
            secondary_color: Some(Color::srgb(0.9, 0.7, 0.5)),
            ..default()
        },
        ContextMappingType::Partnership { .. } => BridgeVisualStyle {
            color: Color::srgb(0.5, 0.5, 0.8),
            pattern: BridgePattern::Solid,
            secondary_color: Some(Color::srgb(0.7, 0.7, 0.9)),
            show_flow: true,
            ..default()
        },
    }
}

/// Plugin for context bridge visualization
pub struct ContextBridgeVisualizationPlugin;

impl Plugin for ContextBridgeVisualizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                visualize_context_bridges,
                animate_context_bridges,
            ),
        );
    }
}
