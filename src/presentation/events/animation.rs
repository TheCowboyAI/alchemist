//! Animation events that remain in the presentation layer
//!
//! These events handle visual transitions, interpolations, and effects
//! without affecting domain state.

use bevy::prelude::*;
use crate::domain::value_objects::{NodeId, EdgeId};
use super::PresentationEvent;

/// Frame update for ongoing animations
#[derive(Event, Clone, Debug)]
pub struct AnimationFrame {
    pub animation_id: u64,
    pub progress: f32,
    pub delta_time: f32,
}

impl PresentationEvent for AnimationFrame {
    fn requires_aggregation(&self) -> bool {
        false // Never sent to domain
    }
}

/// Node position interpolation during animations
#[derive(Event, Clone, Debug)]
pub struct NodeAnimationUpdate {
    pub node_id: NodeId,
    pub entity: Entity,
    pub from_position: Vec3,
    pub to_position: Vec3,
    pub progress: f32,
}

impl PresentationEvent for NodeAnimationUpdate {}

/// Edge animation for connection/disconnection effects
#[derive(Event, Clone, Debug)]
pub struct EdgeAnimationUpdate {
    pub edge_id: EdgeId,
    pub entity: Entity,
    pub animation_type: EdgeAnimationType,
    pub progress: f32,
}

#[derive(Clone, Debug)]
pub enum EdgeAnimationType {
    Connecting { opacity: f32 },
    Disconnecting { opacity: f32 },
    Highlighting { color: Color },
    Pulsing { scale: f32 },
}

impl PresentationEvent for EdgeAnimationUpdate {}

/// Camera animation events
#[derive(Event, Clone, Debug)]
pub struct CameraAnimation {
    pub animation_type: CameraAnimationType,
    pub progress: f32,
}

#[derive(Clone, Debug)]
pub enum CameraAnimationType {
    ZoomTo { target_scale: f32 },
    PanTo { target_position: Vec3 },
    OrbitTo { target_rotation: Quat },
    FocusOn { target_entity: Entity, zoom_level: f32 },
}

impl PresentationEvent for CameraAnimation {
    fn requires_aggregation(&self) -> bool {
        false // Camera movements don't affect domain
    }
}

/// Particle effect events
#[derive(Event, Clone, Debug)]
pub struct ParticleEffect {
    pub effect_type: ParticleEffectType,
    pub position: Vec3,
    pub duration: f32,
}

#[derive(Clone, Debug)]
pub enum ParticleEffectType {
    NodeCreation,
    NodeDeletion,
    EdgeConnection,
    EdgeDisconnection,
    SelectionHighlight,
}

impl PresentationEvent for ParticleEffect {
    fn requires_aggregation(&self) -> bool {
        false // Pure visual effect
    }
}

/// Layout animation progress
#[derive(Event, Clone, Debug)]
pub struct LayoutTransition {
    pub from_layout: String,
    pub to_layout: String,
    pub affected_nodes: Vec<(NodeId, Vec3, Vec3)>, // (id, from, to)
    pub progress: f32,
}

impl PresentationEvent for LayoutTransition {}
