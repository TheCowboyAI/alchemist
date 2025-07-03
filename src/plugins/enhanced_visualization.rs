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

/// Temporary GraphEdge component for demo
#[derive(Component)]
pub struct GraphEdge;

/// Enhanced visualization plugin
pub struct EnhancedVisualizationPlugin;

impl Plugin for EnhancedVisualizationPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<VisualizationSettings>()
            .init_resource::<EventVisualizationQueue>()
            .init_resource::<PerformanceMetrics>()
            .add_systems(Startup, (setup_visualization_resources, setup_volumetric_fog))
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
    pub frequency: f32,
    /// Pulse amount
    pub pulse_amount: f32,
    /// Time
    pub time: f32,
}

impl Default for NodeGlow {
    fn default() -> Self {
        Self {
            base_color: Color::srgb(0.5, 0.7, 1.0),
            intensity: 0.0,
            target_intensity: 0.0,
            frequency: 1.0,
            pulse_amount: 0.0,
            time: 0.0,
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
    /// Ripple radius
    pub radius: f32,
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
    /// Time
    pub time: f32,
    /// Particle count
    pub particle_count: i32,
    /// Particle size
    pub particle_size: f32,
    /// Flow intensity
    pub intensity: f32,
}

impl Default for EdgeFlow {
    fn default() -> Self {
        Self {
            speed: 1.0,
            offset: 0.0,
            color: Color::srgb(0.3, 0.7, 1.0),
            density: 1.0,
            time: 0.0,
            particle_count: 5,
            particle_size: 0.05,
            intensity: 0.8,
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

#[derive(Component, Clone)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub color: Color,
    pub size: f32,
}

#[derive(Component, Clone)]
pub struct ParticleTemplate {
    pub initial_velocity: Vec3,
    pub velocity_variance: Vec3,
    pub lifetime: f32,
    pub lifetime_variance: f32,
    pub initial_color: Color,
    pub final_color: Color,
    pub initial_size: f32,
    pub final_size: f32,
    pub max_lifetime: f32,
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

/// Volumetric fog configuration
#[derive(Component, Default)]
pub struct VolumetricFog {
    pub density: f32,
    pub color: Color,
    pub start_distance: f32,
    pub falloff: f32,
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

/// Setup visualization resources
fn setup_visualization_resources(
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
    mut glows: Query<(&mut NodeGlow, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();
    
    for (mut glow, material_handle) in glows.iter_mut() {
        // Update glow animation
        glow.time += delta * glow.frequency;
        
        // Lerp intensity
        glow.intensity = glow.intensity.lerp(glow.target_intensity, delta * 3.0);
        
        // Update material
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let pulse = (glow.time.sin() * 0.5 + 0.5) * glow.pulse_amount;
            let final_intensity = glow.intensity + pulse;
            
            // Set emissive color with intensity
            let emissive_linear = LinearRgba::from(glow.base_color);
            material.emissive = LinearRgba::new(
                emissive_linear.red * final_intensity,
                emissive_linear.green * final_intensity,
                emissive_linear.blue * final_intensity,
                emissive_linear.alpha,
            );
            material.emissive_exposure_weight = final_intensity.clamp(0.0, 1.0);
        }
    }
}

/// Animate event ripple effects
fn animate_event_ripples(
    time: Res<Time>,
    mut commands: Commands,
    mut ripples: Query<(Entity, &mut EventRipple, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();
    
    for (entity, mut ripple, material_handle) in ripples.iter_mut() {
        ripple.age += delta;
        
        let progress = ripple.age / ripple.lifetime;
        if progress >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        
        // Update material
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let alpha = 1.0 - progress;
            
            // Scale ripple
            ripple.radius = ripple.max_radius * progress;
            
            // Update material opacity
            material.base_color = ripple.color.with_alpha(alpha * 0.5);
            let emissive_linear = LinearRgba::from(ripple.color);
            material.emissive = LinearRgba::new(
                emissive_linear.red * alpha,
                emissive_linear.green * alpha,
                emissive_linear.blue * alpha,
                emissive_linear.alpha,
            );
            material.alpha_mode = AlphaMode::Blend;
        }
    }
}

/// Update edge flow animations
fn update_edge_flow(
    time: Res<Time>,
    mut flows: Query<&mut EdgeFlow>,
    mut gizmos: Gizmos,
    edges: Query<(&GraphEdge, &Transform)>,
) {
    let delta = time.delta_secs();
    
    for mut flow in flows.iter_mut() {
        flow.time += delta * flow.speed;
        
        // Draw flow particles along edges
        for (_edge, transform) in edges.iter() {
            let start = transform.translation;
            let end = start + Vec3::new(10.0, 0.0, 0.0); // Simplified for example
            
            // Draw multiple particles along the edge
            for i in 0..flow.particle_count {
                let offset = (flow.time + i as f32 * 0.2) % 1.0;
                let pos = start.lerp(end, offset);
                
                let alpha = 1.0 - (offset - 0.5).abs() * 2.0;
                let color = flow.color.with_alpha(alpha * flow.intensity);
                
                gizmos.sphere(
                    Isometry3d::from_translation(pos),
                    flow.particle_size,
                    color,
                );
            }
        }
    }
}

/// Update particle systems
fn update_particle_systems(
    time: Res<Time>,
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Particle)>,
    mut systems: Query<&mut ParticleSystem>,
) {
    let delta = time.delta_secs();
    
    // Update individual particles
    for (entity, mut particle) in particles.iter_mut() {
        particle.lifetime += delta;
        
        // Simple physics update
        let gravity = Vec3::new(0.0, -9.81 * delta, 0.0);
        particle.velocity += gravity;
        let velocity = particle.velocity;
        particle.position += velocity * delta;
        
        // Remove old particles
        if particle.lifetime > 5.0 {
            commands.entity(entity).despawn();
        }
    }
    
    // Update particle systems
    for mut system in systems.iter_mut() {
        system.spawn_timer += delta;
        
        // Update existing particles in the system
        let max_lifetime = system.template.max_lifetime;
        system.particles.retain_mut(|particle| {
            particle.lifetime += delta;
            let velocity = particle.velocity;
            particle.position += velocity * delta;
            particle.lifetime < max_lifetime
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
                radius: 0.0,
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
            commands.entity(entity).despawn();
        }
    }
}

/// Setup volumetric fog
fn setup_volumetric_fog(
    mut commands: Commands,
) {
    // Volumetric fog is not directly supported in Bevy 0.16
    // We'll use a marker component instead
    commands.spawn((
        VolumetricFog {
            density: 0.1,
            color: Color::srgba(0.5, 0.6, 0.7, 0.3),
            start_distance: 10.0,
            falloff: 0.1,
        },
        Name::new("Volumetric Fog"),
    ));
}

/// Cleanup despawned entities
fn cleanup_despawned_entities(
    mut commands: Commands,
    ripples: Query<(Entity, &EventRipple)>,
) {
    for (entity, ripple) in ripples.iter() {
        if ripple.age >= ripple.lifetime {
            commands.entity(entity).despawn();
        }
    }
}

/// Helper function to lerp between colors
fn lerp_color(c1: Color, c2: Color, t: f32) -> Color {
    let LinearRgba { red: r1, green: g1, blue: b1, alpha: a1 } = LinearRgba::from(c1);
    let LinearRgba { red: r2, green: g2, blue: b2, alpha: a2 } = LinearRgba::from(c2);
    
    Color::LinearRgba(LinearRgba::new(
        r1.lerp(r2, t),
        g1.lerp(g2, t),
        b1.lerp(b2, t),
        a1.lerp(a2, t),
    ))
} 