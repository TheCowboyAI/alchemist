//! Systems for camera focus and framing operations
//!
//! These systems handle:
//! - Focus on specific nodes
//! - Focus on selection
//! - Fit-to-view operations
//! - Smooth camera transitions

use bevy::prelude::*;

use crate::{
    components::*,
    events::*,
    resources::*,
};

/// System that handles focus on selection events
pub fn handle_focus_selection(
    mut events: EventReader<FocusSelectionEvent>,
    selected: Query<&Transform, With<Selected>>,
    mut camera_events: EventWriter<AnimateCameraEvent>,
) {
    for event in events.read() {
        if selected.is_empty() {
            continue;
        }

        // Calculate bounding box of selection
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);

        for transform in selected.iter() {
            min = min.min(transform.translation);
            max = max.max(transform.translation);
        }

        let center = (min + max) * 0.5;
        let size = (max - min).length();

        // Calculate camera position to frame selection
        let distance = size * 2.0; // Adjust multiplier for desired framing
        let camera_offset = Vec3::new(0.0, distance * 0.7, distance);
        let target_position = center + camera_offset;

        camera_events.send(AnimateCameraEvent {
            target_position,
            target_rotation: Some(Quat::from_rotation_x(-0.5)), // Look down at selection
            target_scale: None,
            duration: if event.instant { 0.0 } else { 1.0 },
            easing: EasingFunction::EaseInOut,
        });
    }
}

/// System that handles focus on specific node
pub fn handle_focus_node(
    mut events: EventReader<FocusNodeEvent>,
    nodes: Query<&Transform, With<NodeId>>,
    uuid_to_entity: Res<UuidToEntity>,
    mut camera_events: EventWriter<AnimateCameraEvent>,
) {
    for event in events.read() {
        if let Some(&entity) = uuid_to_entity.0.get(&event.node_id) {
            if let Ok(transform) = nodes.get(entity) {
                let target_position = transform.translation + Vec3::new(5.0, 5.0, 5.0);

                camera_events.send(AnimateCameraEvent {
                    target_position,
                    target_rotation: None,
                    target_scale: None,
                    duration: if event.instant { 0.0 } else { 0.5 },
                    easing: EasingFunction::EaseInOut,
                });
            }
        }
    }
}

/// System that handles fit-to-view events
pub fn handle_fit_to_view(
    mut events: EventReader<FitToViewEvent>,
    nodes: Query<&Transform, With<NodeId>>,
    mut camera_events: EventWriter<AnimateCameraEvent>,
) {
    for event in events.read() {
        if nodes.is_empty() {
            continue;
        }

        // Calculate bounding box of all nodes
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);

        for transform in nodes.iter() {
            min = min.min(transform.translation);
            max = max.max(transform.translation);
        }

        let center = (min + max) * 0.5;
        let size = (max - min).length();

        // Add padding
        let distance = size * (1.0 + event.padding);
        let camera_offset = Vec3::new(distance * 0.5, distance * 0.7, distance);
        let target_position = center + camera_offset;

        camera_events.send(AnimateCameraEvent {
            target_position,
            target_rotation: Some(Quat::from_rotation_x(-0.4)),
            target_scale: None,
            duration: 1.5,
            easing: EasingFunction::EaseInOut,
        });
    }
}

/// System that animates camera movements
pub fn animate_camera(
    mut events: EventReader<AnimateCameraEvent>,
    time: Res<Time>,
    mut camera_animations: ResMut<CameraAnimations>,
    mut cameras: Query<&mut Transform, With<GraphCamera>>,
    mut complete_events: EventWriter<CameraAnimationCompleteEvent>,
) {
    // Add new animations
    for event in events.read() {
        if let Ok(transform) = cameras.get_single() {
            camera_animations.0.push(CameraAnimation {
                start_position: transform.translation,
                target_position: event.target_position,
                start_rotation: transform.rotation,
                target_rotation: event.target_rotation.unwrap_or(transform.rotation),
                duration: event.duration,
                elapsed: 0.0,
                easing: event.easing,
            });
        }
    }

    // Update existing animations
    let delta = time.delta_seconds();
    camera_animations.0.retain_mut(|animation| {
        animation.elapsed += delta;

        if animation.elapsed >= animation.duration || animation.duration == 0.0 {
            // Complete immediately
            if let Ok(mut transform) = cameras.get_single_mut() {
                transform.translation = animation.target_position;
                transform.rotation = animation.target_rotation;
            }
            complete_events.send(CameraAnimationCompleteEvent);
            false // Remove animation
        } else {
            // Interpolate
            let t = animation.elapsed / animation.duration;
            let t = apply_easing(t, animation.easing);

            if let Ok(mut transform) = cameras.get_single_mut() {
                transform.translation = animation.start_position.lerp(animation.target_position, t);
                transform.rotation = animation.start_rotation.slerp(animation.target_rotation, t);
            }
            true // Keep animation
        }
    });
}

fn apply_easing(t: f32, easing: EasingFunction) -> f32 {
    match easing {
        EasingFunction::Linear => t,
        EasingFunction::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                -1.0 + (4.0 - 2.0 * t) * t
            }
        }
        EasingFunction::EaseIn => t * t,
        EasingFunction::EaseOut => t * (2.0 - t),
        EasingFunction::Spring => {
            let c4 = (2.0 * std::f32::consts::PI) / 3.0;
            if t == 0.0 || t == 1.0 {
                t
            } else {
                (2.0_f32).powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
            }
        }
    }
}
