//! Agent lifecycle systems

use bevy::prelude::*;
use std::time::SystemTime;

use crate::components::{
    AgentEntity, AgentOwner, AgentTypeComponent, CreatedAt, IdentityEntity, IdentityMetadata,
    IdentityVerification, LastActive, UpdatedAt,
};
use crate::events::AgentDeployed;

/// Plugin for agent lifecycle systems
pub struct AgentLifecyclePlugin;

impl Plugin for AgentLifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_agent, update_agent_activity, cleanup_inactive_agents),
        );
    }
}

/// Spawns a new agent entity in the ECS world
fn spawn_agent(mut commands: Commands, mut spawn_events: EventReader<AgentDeployed>) {
    for event in spawn_events.read() {
        commands.spawn((
            AgentEntity {
                id: event.agent_id,
                agent_type: event.agent_type.clone(),
            },
            AgentOwner {
                owner_id: event.owner_id,
            },
            AgentTypeComponent(event.agent_type.clone()),
            CreatedAt(SystemTime::now()),
            UpdatedAt(SystemTime::now()),
            LastActive(SystemTime::now()),
        ));
    }
}

/// Update agent activity timestamps
fn update_agent_activity(
    time: Res<Time>,
    mut query: Query<(&mut LastActive, &mut UpdatedAt), With<AgentEntity>>,
) {
    // Only update activity periodically, not every frame
    let update_interval = 1.0; // Update every second
    let elapsed = time.elapsed_secs();

    // Check if we should update based on elapsed time
    if elapsed % update_interval < time.delta_secs() {
        for (mut last_active, mut updated_at) in query.iter_mut() {
            // Update activity timestamp
            let now = SystemTime::now();
            last_active.0 = now;
            updated_at.0 = now;
        }
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
        if let Ok(duration) = now.duration_since(last_active.0) {
            if duration > max_inactive_duration {
                // Remove inactive agent
                commands.entity(entity).despawn();
            }
        }
    }
}

// Identity lifecycle systems
/// Create identity system
pub fn create_identity_system(
    mut commands: Commands,
    mut create_events: EventReader<crate::commands::CreateIdentityCommand>,
    mut created_events: EventWriter<crate::events::IdentityLinkedToPerson>,
) {
    use crate::components::{IdentityEntity, IdentityMetadata, IdentityVerification};
    use crate::value_objects::{VerificationLevel, VerificationMethod};

    for event in create_events.read() {
        let entity_id = commands
            .spawn((
                IdentityEntity {
                    id: event.identity_id,
                    identity_type: event.identity_type.clone(),
                    external_reference: event.external_reference.clone(),
                },
                IdentityVerification {
                    verification_level: VerificationLevel::None,
                    verification_method: None,
                    last_verified: None,
                },
                IdentityMetadata {
                    created_at: SystemTime::now(),
                    updated_at: SystemTime::now(),
                    metadata: std::collections::HashMap::new(),
                },
            ))
            .id();

        // Emit event if linked to person
        if let Some(person_id) = event.person_id {
            created_events.write(crate::events::IdentityLinkedToPerson {
                identity_id: event.identity_id.to_string(),
                person_id: person_id.to_string(),
            });
        }
    }
}

/// Update identity system
pub fn update_identity_system(
    mut query: Query<(&mut IdentityEntity, &mut IdentityMetadata), Changed<IdentityEntity>>,
) {
    for (_identity, mut metadata) in query.iter_mut() {
        // Update the metadata timestamp whenever the identity changes
        metadata.updated_at = SystemTime::now();
    }
}

/// Merge identities system
pub fn merge_identities_system(
    mut commands: Commands,
    mut merge_events: EventReader<crate::commands::MergeIdentitiesCommand>,
    identities: Query<(Entity, &IdentityEntity, &IdentityMetadata)>,
) {
    for event in merge_events.read() {
        // Find the source and target identities
        let source = identities.iter().find(|(_, i, _)| i.id == event.source_id);
        let target = identities.iter().find(|(_, i, _)| i.id == event.target_id);

        if let (
            Some((source_entity, source_identity, source_meta)),
            Some((_, target_identity, target_meta)),
        ) = (source, target)
        {
            // Merge metadata
            let mut merged_metadata = target_meta.metadata.clone();
            for (key, value) in &source_meta.metadata {
                merged_metadata.entry(key.clone()).or_insert(value.clone());
            }

            // Remove the source identity
            commands.entity(source_entity).despawn();

            // Log the merge (in production, emit an event)
            info!(
                "Merged identity {} into {}",
                source_identity.id, target_identity.id
            );
        }
    }
}

/// Archive identity system
pub fn archive_identity_system(
    commands: Commands,
    mut archive_events: EventReader<crate::commands::ArchiveIdentityCommand>,
    mut identities: Query<(Entity, &mut IdentityMetadata), With<IdentityEntity>>,
) {
    for event in archive_events.read() {
        if let Some((_entity, mut metadata)) = identities.iter_mut().find(|(_, m)| {
            m.metadata
                .get("id")
                .map(|id| id == &event.identity_id.to_string())
                .unwrap_or(false)
        }) {
            // Mark as archived
            metadata
                .metadata
                .insert("archived".to_string(), "true".to_string());
            metadata.metadata.insert(
                "archived_at".to_string(),
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string(),
            );
            metadata.updated_at = SystemTime::now();

            // In production, you might move to a different storage or add an Archived component
            info!("Archived identity {}", event.identity_id);
        }
    }
}
