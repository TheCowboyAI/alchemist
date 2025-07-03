//! Enhanced Visualization Plugin for CIM
//! 
//! Provides advanced 3D visualization capabilities for the graph editor including:
//! - Real-time event visualization
//! - Advanced visual effects and animations
//! - Performance optimizations with LOD
//! - Interactive data overlays

use bevy::prelude::*;
use bevy::pbr::{CascadeShadowConfigBuilder, ScreenSpaceAmbientOcclusion};
use bevy::core_pipeline::bloom::{Bloom, BloomCompositeMode};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::render::view::RenderLayers;
use crate::{
    components::{NodeEntity, EdgeEntity, Selected},
    events::{NodeAdded, EdgeAdded, NodeRemoved, EdgeRemoved},
    value_objects::{NodeType, EdgeRelationship},
};
use std::collections::HashMap;
use tracing::info;

/// Enhanced visualization plugin
pub struct EnhancedVisualizationPlugin;

impl Plugin for EnhancedVisualizationPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<VisualizationSettings>()
            .init_resource::<EventVisualizationQueue>()
            .init_resource::<PerformanceMetrics>()
            .add_systems(Startup, setup_enhanced_scene)
            .add_systems(Update, (
                // Visual effects
                update_node_glow,
                animate_event_ripples,
                update_edge_flow,
                update_particle_systems,
                
                // Event visualization
                visualize_domain_events,
                process_event_queue,
                
                // Performance
                update_lod_system,
                frustum_culling,
                
                // Interactive features
                highlight_connections,
                show_data_overlays,
                
                // Camera enhancements
                smooth_camera_transitions,
                
                // Input handling
                handle_visualization_input,
            ).chain())
            .add_systems(PostUpdate, cleanup_expired_effects);
    }
}

/// Settings for visualization features
#[derive(Resource)]
pub struct VisualizationSettings {
    /// Enable particle effects
    pub particles_enabled: bool,
    /// Enable glow effects
    pub glow_enabled: bool,
    /// Enable edge flow animation
    pub edge_flow_enabled: bool,
    /// LOD distance thresholds
    pub lod_distances: [f32; 3],
    /// Maximum visible nodes
    pub max_visible_nodes: usize,
}

impl Default for VisualizationSettings {
    fn default() -> Self {
        Self {
            particles_enabled: true,
            glow_enabled: true,
            edge_flow_enabled: true,
            lod_distances: [20.0, 50.0, 100.0],
            max_visible_nodes: 1000,
        }
    }
}

/// Queue for visualizing domain events
#[derive(Resource, Default)]
pub struct EventVisualizationQueue {
    /// Pending events to visualize
    pub events: Vec<EventVisualization>,
}

/// Visual representation of a domain event
pub struct EventVisualization {
    /// Type of event
    pub event_type: EventType,
    /// Position in world space
    pub position: Vec3,
    /// Time when event occurred
    pub timestamp: f32,
    /// Duration of the visualization
    pub duration: f32,
    /// Color of the effect
    pub color: Color,
}

#[derive(Debug, Clone)]
pub enum EventType {
    NodeCreated,
    NodeUpdated,
    NodeDeleted,
    EdgeCreated,
    EdgeDeleted,
    WorkflowStarted,
    WorkflowCompleted,
}

/// Performance tracking
#[derive(Resource, Default)]
pub struct PerformanceMetrics {
    /// Current FPS
    pub fps: f32,
    /// Number of visible nodes
    pub visible_nodes: usize,
    /// Number of active effects
    pub active_effects: usize,
}

/// Component for node glow effect
#[derive(Component)]
pub struct NodeGlow {
    /// Base emissive color
    pub base_color: Color,
    /// Current intensity
    pub intensity: f32,
    /// Target intensity
    pub target_intensity: f32,
    /// Pulse frequency
    pub pulse_frequency: f32,
}

impl Default for NodeGlow {
    fn default() -> Self {
        Self {
            base_color: Color::srgb(0.5, 0.7, 1.0),
            intensity: 0.0,
            target_intensity: 0.0,
            pulse_frequency: 1.0,
        }
    }
}

/// Component for event ripple effects
#[derive(Component)]
pub struct EventRipple {
    /// Time since creation
    pub age: f32,
    /// Maximum lifetime
    pub lifetime: f32,
    /// Ripple color
    pub color: Color,
    /// Maximum radius
    pub max_radius: f32,
}

/// Component for edge flow animation
#[derive(Component)]
pub struct EdgeFlow {
    /// Flow speed
    pub speed: f32,
    /// Current offset
    pub offset: f32,
    /// Flow color
    pub color: Color,
    /// Particle density
    pub density: f32,
}

impl Default for EdgeFlow {
    fn default() -> Self {
        Self {
            speed: 1.0,
            offset: 0.0,
            color: Color::srgb(0.3, 0.7, 1.0),
            density: 5.0,
        }
    }
}

/// Level of detail component
#[derive(Component)]
pub struct LevelOfDetail {
    /// Current LOD level (0 = highest detail)
    pub level: u8,
    /// Distance from camera
    pub distance: f32,
}

/// Particle system component
#[derive(Component)]
pub struct ParticleSystem {
    /// Particles in this system
    pub particles: Vec<Particle>,
    /// Spawn rate (particles per second)
    pub spawn_rate: f32,
    /// Time since last spawn
    pub spawn_timer: f32,
    /// Particle template
    pub template: ParticleTemplate,
}

#[derive(Clone)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub color: Color,
    pub size: f32,
}

#[derive(Clone)]
pub struct ParticleTemplate {
    pub initial_velocity: Vec3,
    pub velocity_variance: Vec3,
    pub lifetime: f32,
    pub lifetime_variance: f32,
    pub initial_color: Color,
    pub final_color: Color,
    pub initial_size: f32,
    pub final_size: f32,
}

/// Data overlay component
#[derive(Component)]
pub struct DataOverlay {
    /// Text to display
    pub text: String,
    /// Background color
    pub background: Color,
    /// Offset from entity
    pub offset: Vec3,
}

/// Handle keyboard input for visualization settings
fn handle_visualization_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<VisualizationSettings>,
) {
    // Toggle particles
    if keyboard.just_pressed(KeyCode::KeyP) {
        settings.particles_enabled = !settings.particles_enabled;
        info!("Particles: {}", if settings.particles_enabled { "ON" } else { "OFF" });
    }
    
    // Toggle glow effects
    if keyboard.just_pressed(KeyCode::KeyL) {
        settings.glow_enabled = !settings.glow_enabled;
        info!("Glow effects: {}", if settings.glow_enabled { "ON" } else { "OFF" });
    }
    
    // Toggle edge flow
    if keyboard.just_pressed(KeyCode::KeyF) {
        settings.edge_flow_enabled = !settings.edge_flow_enabled;
        info!("Edge flow: {}", if settings.edge_flow_enabled { "ON" } else { "OFF" });
    }
}

/// Setup enhanced scene with advanced lighting and effects
fn setup_enhanced_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Enhanced ambient lighting
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.15, 0.15, 0.2),
        brightness: 0.3,
        affects_lightmapped_meshes: false,
    });

    // Primary directional light with shadows
    commands.spawn((
        DirectionalLight {
            illuminance: 25000.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.95, 0.8),
            ..default()
        },
        Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 10.0,
            maximum_distance: 100.0,
            ..default()
        }.build(),
    ));

    // Add volumetric fog for atmosphere
    commands.spawn((
        FogVolume {
            absorption: 0.1,
            scattering: 0.3,
            density_factor: 0.05,
            density_texture_offset: Vec3::ZERO,
            ..default()
        },
        Transform::from_scale(Vec3::splat(50.0)),
    ));

    // Create grid floor with emissive lines
    let grid_size = 100.0;
    let grid_divisions = 20;
    let line_width = 0.05;
    
    // Main grid lines
    let grid_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.15),
        emissive: Color::srgb(0.0, 0.2, 0.4).into(),
        emissive_exposure_weight: 0.5,
        metallic: 0.8,
        perceptual_roughness: 0.2,
        ..default()
    });

    // Create grid
    for i in 0..=grid_divisions {
        let offset = (i as f32 / grid_divisions as f32 - 0.5) * grid_size;
        
        // X-direction lines
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(line_width, 0.01, grid_size))),
            MeshMaterial3d(grid_material.clone()),
            Transform::from_xyz(offset, -5.0, 0.0),
        ));
        
        // Z-direction lines
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(grid_size, 0.01, line_width))),
            MeshMaterial3d(grid_material.clone()),
            Transform::from_xyz(0.0, -5.0, offset),
        ));
    }

    // Add accent lights at grid intersections
    for i in [0, grid_divisions / 2, grid_divisions] {
        for j in [0, grid_divisions / 2, grid_divisions] {
            let x = (i as f32 / grid_divisions as f32 - 0.5) * grid_size;
            let z = (j as f32 / grid_divisions as f32 - 0.5) * grid_size;
            
            commands.spawn((
                PointLight {
                    intensity: 1000.0,
                    color: Color::srgb(0.0, 0.5, 1.0),
                    shadows_enabled: false,
                    radius: 10.0,
                    ..default()
                },
                Transform::from_xyz(x, -4.5, z),
            ));
        }
    }
}

/// Update node glow effects
fn update_node_glow(
    time: Res<Time>,
    settings: Res<VisualizationSettings>,
    mut nodes: Query<(&mut NodeGlow, &MeshMaterial3d<StandardMaterial>, Option<&Selected>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !settings.glow_enabled {
        return;
    }

    for (mut glow, material_handle, selected) in nodes.iter_mut() {
        // Update target intensity based on selection
        glow.target_intensity = if selected.is_some() { 1.0 } else { 0.3 };
        
        // Smooth intensity transition
        let delta = time.delta_secs();
        glow.intensity = glow.intensity.lerp(&glow.target_intensity, delta * 3.0);
        
        // Apply pulsing effect
        let pulse = (time.elapsed_secs() * glow.pulse_frequency).sin() * 0.5 + 0.5;
        let final_intensity = glow.intensity * (0.7 + pulse * 0.3);
        
        // Update material
        if let Some(material) = materials.get_mut(material_handle) {
            material.emissive = (glow.base_color * final_intensity).into();
            material.emissive_exposure_weight = 0.5;
        }
    }
}

/// Animate event ripple effects
fn animate_event_ripples(
    time: Res<Time>,
    mut commands: Commands,
    mut ripples: Query<(Entity, &mut Transform, &mut EventRipple)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_handles: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    let delta = time.delta_secs();
    
    for (entity, mut transform, mut ripple) in ripples.iter_mut() {
        ripple.age += delta;
        
        // Calculate ripple progress
        let progress = ripple.age / ripple.lifetime;
        
        if progress >= 1.0 {
            // Remove expired ripple
            commands.entity(entity).despawn_recursive();
            continue;
        }
        
        // Expand ripple
        let radius = ripple.max_radius * progress;
        transform.scale = Vec3::new(radius * 2.0, 0.1, radius * 2.0);
        
        // Fade out
        let alpha = 1.0 - progress;
        if let Ok(material_handle) = material_handles.get(entity) {
            if let Some(material) = materials.get_mut(material_handle) {
                material.base_color = ripple.color.with_alpha(alpha * 0.5);
                material.emissive = (ripple.color * alpha).into();
            }
        }
    }
}

/// Update edge flow animations
fn update_edge_flow(
    time: Res<Time>,
    settings: Res<VisualizationSettings>,
    mut edges: Query<(&mut EdgeFlow, &Transform, &EdgeEntity)>,
    mut gizmos: Gizmos,
) {
    if !settings.edge_flow_enabled {
        return;
    }

    let delta = time.delta_secs();
    
    for (mut flow, transform, edge) in edges.iter_mut() {
        // Update flow offset
        flow.offset += flow.speed * delta;
        if flow.offset > 1.0 {
            flow.offset -= 1.0;
        }
        
        // Draw flow particles along edge
        let num_particles = (transform.scale.y * flow.density) as i32;
        for i in 0..num_particles {
            let t = (i as f32 / num_particles as f32 + flow.offset) % 1.0;
            let y_offset = (t - 0.5) * transform.scale.y;
            let particle_pos = transform.translation + transform.rotation * Vec3::new(0.0, y_offset, 0.0);
            
            // Fade particles at ends
            let fade = (0.5 - (t - 0.5).abs()) * 2.0;
            let color = flow.color.with_alpha(fade * 0.8);
            
            gizmos.sphere(particle_pos, 0.05, color);
        }
    }
}

/// Update particle systems
fn update_particle_systems(
    time: Res<Time>,
    settings: Res<VisualizationSettings>,
    mut systems: Query<(&Transform, &mut ParticleSystem)>,
    mut gizmos: Gizmos,
) {
    if !settings.particles_enabled {
        return;
    }

    let delta = time.delta_secs();
    
    for (transform, mut system) in systems.iter_mut() {
        // Spawn new particles
        system.spawn_timer += delta;
        let spawn_interval = 1.0 / system.spawn_rate;
        
        while system.spawn_timer >= spawn_interval {
            system.spawn_timer -= spawn_interval;
            
            // Create new particle
            let template = &system.template;
            let variance = Vec3::new(
                rand::random::<f32>() - 0.5,
                rand::random::<f32>() - 0.5,
                rand::random::<f32>() - 0.5,
            );
            
            system.particles.push(Particle {
                position: transform.translation,
                velocity: template.initial_velocity + variance * template.velocity_variance,
                lifetime: 0.0,
                max_lifetime: template.lifetime + (rand::random::<f32>() - 0.5) * template.lifetime_variance,
                color: template.initial_color,
                size: template.initial_size,
            });
        }
        
        // Update existing particles
        system.particles.retain_mut(|particle| {
            particle.lifetime += delta;
            
            if particle.lifetime >= particle.max_lifetime {
                return false; // Remove expired particle
            }
            
            // Update position
            particle.position += particle.velocity * delta;
            
            // Update color and size
            let t = particle.lifetime / particle.max_lifetime;
            particle.color = template.initial_color.lerp(&template.final_color, t);
            particle.size = template.initial_size.lerp(&template.final_size, t);
            
            // Draw particle
            gizmos.sphere(particle.position, particle.size, particle.color);
            
            true
        });
    }
}

/// Process domain events and create visualizations
fn visualize_domain_events(
    mut node_events: EventReader<NodeAdded>,
    mut edge_events: EventReader<EdgeAdded>,
    mut event_queue: ResMut<EventVisualizationQueue>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    
    // Visualize node creation events
    for event in node_events.read() {
        event_queue.events.push(EventVisualization {
            event_type: EventType::NodeCreated,
            position: Vec3::new(event.position.x, event.position.y, event.position.z),
            timestamp: current_time,
            duration: 2.0,
            color: Color::srgb(0.2, 0.8, 0.2),
        });
    }
    
    // Visualize edge creation events
    for event in edge_events.read() {
        // Would need to look up node positions here
        event_queue.events.push(EventVisualization {
            event_type: EventType::EdgeCreated,
            position: Vec3::ZERO, // Should be midpoint of edge
            timestamp: current_time,
            duration: 1.5,
            color: Color::srgb(0.2, 0.5, 0.8),
        });
    }
}

/// Process queued event visualizations
fn process_event_queue(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut event_queue: ResMut<EventVisualizationQueue>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    
    // Process each pending event
    for event in event_queue.events.drain(..) {
        // Create ripple effect
        commands.spawn((
            EventRipple {
                age: 0.0,
                lifetime: event.duration,
                color: event.color,
                max_radius: 5.0,
            },
            Mesh3d(meshes.add(Torus {
                minor_radius: 0.1,
                major_radius: 0.5,
            })),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: event.color.with_alpha(0.5),
                emissive: event.color.into(),
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_translation(event.position),
        ));
    }
}

/// Update LOD based on camera distance
fn update_lod_system(
    camera: Query<&Transform, With<Camera3d>>,
    mut nodes: Query<(&Transform, &mut LevelOfDetail, &mut Visibility), Without<Camera3d>>,
    settings: Res<VisualizationSettings>,
) {
    let Ok(camera_transform) = camera.get_single() else {
        return;
    };
    
    for (transform, mut lod, mut visibility) in nodes.iter_mut() {
        // Calculate distance to camera
        lod.distance = camera_transform.translation.distance(transform.translation);
        
        // Determine LOD level
        lod.level = if lod.distance < settings.lod_distances[0] {
            0 // Full detail
        } else if lod.distance < settings.lod_distances[1] {
            1 // Medium detail
        } else if lod.distance < settings.lod_distances[2] {
            2 // Low detail
        } else {
            3 // Hidden
        };
        
        // Update visibility
        *visibility = if lod.level < 3 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Frustum culling for performance
fn frustum_culling(
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut nodes: Query<(&GlobalTransform, &mut Visibility), With<NodeEntity>>,
) {
    let Ok((camera, camera_transform)) = camera.get_single() else {
        return;
    };
    
    // Simple frustum culling - would be more complex in production
    for (transform, mut visibility) in nodes.iter_mut() {
        let position = transform.translation();
        
        // Check if node is in camera view
        if let Some(ndc) = camera.world_to_ndc(camera_transform, position) {
            *visibility = if ndc.x.abs() < 1.2 && ndc.y.abs() < 1.2 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

/// Highlight connections when hovering
fn highlight_connections(
    hover_entity: Query<&NodeEntity, With<Selected>>,
    edges: Query<(&EdgeEntity, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Reset all edge materials first
    for (_, material_handle) in edges.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            material.emissive = LinearRgba::BLACK;
        }
    }
    
    // Highlight connected edges
    for selected_node in hover_entity.iter() {
        for (edge, material_handle) in edges.iter() {
            if edge.source == selected_node.node_id || edge.target == selected_node.node_id {
                if let Some(material) = materials.get_mut(material_handle) {
                    material.emissive = Color::srgb(0.5, 0.7, 1.0).into();
                }
            }
        }
    }
}

/// Show data overlays for selected nodes
fn show_data_overlays(
    selected: Query<(Entity, &NodeEntity), Added<Selected>>,
    mut commands: Commands,
) {
    for (entity, node) in selected.iter() {
        commands.entity(entity).insert(DataOverlay {
            text: format!("Node: {:?}\nType: Process", node.node_id),
            background: Color::srgba(0.0, 0.0, 0.0, 0.8),
            offset: Vec3::new(0.0, 2.0, 0.0),
        });
    }
}

/// Smooth camera transitions
fn smooth_camera_transitions(
    mut camera: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    // This would implement smooth camera movement
    // For now, just a placeholder
}

/// Clean up expired visual effects
fn cleanup_expired_effects(
    mut commands: Commands,
    ripples: Query<(Entity, &EventRipple)>,
) {
    for (entity, ripple) in ripples.iter() {
        if ripple.age >= ripple.lifetime {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// Helper function for linear interpolation
trait Lerp {
    fn lerp(&self, other: &Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Lerp for Color {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        let [r1, g1, b1, a1] = self.to_srgba().to_f32_array();
        let [r2, g2, b2, a2] = other.to_srgba().to_f32_array();
        
        Color::srgba(
            r1.lerp(&r2, t),
            g1.lerp(&g2, t),
            b1.lerp(&b2, t),
            a1.lerp(&a2, t),
        )
    }
} 