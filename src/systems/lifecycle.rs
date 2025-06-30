//! Agent lifecycle systems

use bevy::prelude::*;
use std::time::SystemTime;

use crate::components::{
    AgentEntity, AgentOwner, AgentTypeComponent, CreatedAt, LastActive, UpdatedAt,
};
use crate::events::AgentDeployed;

/// Plugin for agent lifecycle systems
pub struct AgentLifecyclePlugin;

impl Plugin for AgentLifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            spawn_agent,
            update_agent_activity,
            cleanup_inactive_agents,
        ));
    }
}

/// Spawns a new agent entity in the ECS world
fn spawn_agent(
    mut commands: Commands,
    mut spawn_events: EventReader<AgentDeployed>,
) {
    for event in spawn_events.read() {
        commands.spawn((
            AgentEntity {
                id: event.agent_id.0,  // Extract Uuid from AgentId
            },
            AgentOwner {
                owner_id: event.owner_id,
            },
            AgentTypeComponent {
                agent_type: event.agent_type.clone(),
            },
            CreatedAt {
                timestamp: SystemTime::now(),
            },
            UpdatedAt {
                timestamp: SystemTime::now(),
            },
            LastActive {
                timestamp: SystemTime::now(),
            },
        ));
    }
}

/// Update agent activity timestamps
fn update_agent_activity(
    time: Res<Time>,
    mut query: Query<&mut LastActive, With<AgentEntity>>,
) {
    for mut last_active in query.iter_mut() {
        // Update activity every frame for now
        // In a real system, this would be triggered by actual agent activity
        last_active.timestamp = SystemTime::now();
    }
}

/// Clean up agents that have been inactive for too long
fn cleanup_inactive_agents(
    mut commands: Commands,
    query: Query<(Entity, &LastActive), With<AgentEntity>>,
) {
    let now = SystemTime::now();
    let max_inactive_duration = std::time::Duration::from_secs(3600); // 1 hour
    
    for (entity, last_active) in query.iter() {
        if let Ok(duration) = now.duration_since(last_active.timestamp) {
            if duration > max_inactive_duration {
                // Remove inactive agent
                commands.entity(entity).despawn();
            }
        }
    }
} 