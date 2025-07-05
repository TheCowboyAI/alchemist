use bevy::prelude::*;
use uuid::Uuid;

use crate::{
    commands::CreateProjectionCommand,
    components::{IdentityEntity, IdentityProjection, IdentityVerification},
    events::{IdentityLinkedToOrganization, IdentityLinkedToPerson, ProjectionCreated},
    value_objects::{ProjectionContext, ProjectionType},
};

/// Create projection system
pub fn create_projection_system(
    mut commands: Commands,
    mut create_events: EventReader<CreateProjectionCommand>,
    mut created_events: EventWriter<ProjectionCreated>,
    identities: Query<Entity, With<IdentityEntity>>,
) {
    for event in create_events.read() {
        // Create projection component
        let projection = IdentityProjection {
            projection_type: event.projection_type.clone(),
            context: event.context.clone(),
        };

        // Add projection to all identities (simplified - in production you'd be more selective)
        for entity in identities.iter() {
            commands.entity(entity).insert(projection.clone());
        }

        // Emit created event
        created_events.write(ProjectionCreated {
            projection_id: Uuid::new_v4().to_string(),
            projection_type: format!("{:?}", event.projection_type),
        });

        info!("Created projection of type {:?}", event.projection_type);
    }
}

/// Sync projections system
pub fn sync_projections_system(
    mut link_person_events: EventReader<IdentityLinkedToPerson>,
    mut link_org_events: EventReader<IdentityLinkedToOrganization>,
    mut identities: Query<(&IdentityEntity, &mut IdentityProjection)>,
) {
    // Sync person links
    for event in link_person_events.read() {
        if let Ok(identity_id) = Uuid::parse_str(&event.identity_id) {
            for (entity, mut projection) in &mut identities {
                if entity.id == identity_id {
                    // Update projection context with person link
                    projection.context = ProjectionContext {
                        id: event.person_id.clone(),
                        name: format!("Person {}", event.person_id),
                    };

                    info!(
                        "Synced identity {} with person {}",
                        event.identity_id, event.person_id
                    );
                }
            }
        }
    }

    // Sync organization links
    for event in link_org_events.read() {
        if let Ok(identity_id) = Uuid::parse_str(&event.identity_id) {
            for (entity, mut projection) in &mut identities {
                if entity.id == identity_id {
                    // Update projection context with org link
                    projection.context = ProjectionContext {
                        id: event.organization_id.clone(),
                        name: format!("Organization {}", event.organization_id),
                    };

                    info!(
                        "Synced identity {} with organization {}",
                        event.identity_id, event.organization_id
                    );
                }
            }
        }
    }
}

/// Validate projections system
pub fn validate_projections_system(
    identities: Query<(&IdentityEntity, &IdentityProjection, &IdentityVerification)>,
    mut validation_events: EventWriter<ProjectionValidationEvent>,
) {
    for (entity, projection, verification) in identities.iter() {
        let is_valid = match &projection.projection_type {
            ProjectionType::Summary => true, // Summary projections are always valid
            ProjectionType::Detail => {
                // Detail projections require at least basic verification
                verification.verification_level >= crate::value_objects::VerificationLevel::Basic
            }
            ProjectionType::Analytics => {
                // Analytics projections require advanced verification
                verification.verification_level >= crate::value_objects::VerificationLevel::Advanced
            }
        };

        if !is_valid {
            validation_events.write(ProjectionValidationEvent {
                identity_id: entity.id,
                projection_type: projection.projection_type.clone(),
                is_valid,
                reason: format!(
                    "Insufficient verification level: {:?}",
                    verification.verification_level
                ),
            });
        }
    }
}

/// Event for projection validation results
#[derive(Event)]
pub struct ProjectionValidationEvent {
    /// Identity ID
    pub identity_id: Uuid,
    /// Projection type
    pub projection_type: ProjectionType,
    /// Whether the projection is valid
    pub is_valid: bool,
    /// Reason for validation result
    pub reason: String,
}
