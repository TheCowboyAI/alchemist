//! Identity relationship systems

use bevy::prelude::*;
use std::time::SystemTime;
use uuid::Uuid;

use crate::{
    aggregate::IdentityAggregate,
    commands::{
        EstablishRelationshipCommand, ExpireRelationshipCommand, ValidateRelationshipCommand,
    },
    components::{IdentityEntity, IdentityRelationship},
    events::{RelationshipEstablished, RelationshipExpired, RelationshipValidated},
    value_objects::RelationshipType,
};

/// System to establish relationships between identities
pub fn establish_relationship_system(
    mut commands: Commands,
    mut events: EventReader<EstablishRelationshipCommand>,
    mut established_events: EventWriter<RelationshipEstablished>,
    identities: Query<&IdentityEntity>,
    existing_relationships: Query<&IdentityRelationship>,
) {
    for event in events.read() {
        // Validate identities exist
        let source_exists = identities.iter().any(|i| i.id == event.source_identity);
        let target_exists = identities.iter().any(|i| i.id == event.target_identity);

        if !source_exists || !target_exists {
            eprintln!("Cannot establish relationship: one or both identities don't exist");
            continue;
        }

        // Check for duplicate relationships
        let duplicate = existing_relationships.iter().any(|r| {
            r.source_identity == event.source_identity
                && r.target_identity == event.target_identity
                && r.relationship_type == event.relationship_type
        });

        if duplicate {
            eprintln!("Relationship already exists");
            continue;
        }

        // Validate through aggregate
        match IdentityAggregate::validate_relationship(
            event.source_identity,
            event.target_identity,
            &event.relationship_type,
        ) {
            Ok(_) => {
                let rel_id = Uuid::new_v4();

                // Spawn the relationship entity
                commands.spawn(IdentityRelationship {
                    id: rel_id,
                    source_identity: event.source_identity,
                    target_identity: event.target_identity,
                    relationship_type: event.relationship_type.clone(),
                    established_at: SystemTime::now(),
                    expires_at: event.expires_at,
                    metadata: event.metadata.clone(),
                });

                // Emit established event
                established_events.write(RelationshipEstablished {
                    relationship_id: rel_id,
                    source_identity: event.source_identity,
                    target_identity: event.target_identity,
                    relationship_type: event.relationship_type.clone(),
                    established_at: SystemTime::now(),
                });
            }
            Err(e) => {
                eprintln!("Failed to establish relationship: {}", e);
            }
        }
    }
}

/// System to validate relationships
pub fn validate_relationship_system(
    mut events: EventReader<ValidateRelationshipCommand>,
    mut validated_events: EventWriter<RelationshipValidated>,
    relationships: Query<&IdentityRelationship>,
    identities: Query<&IdentityEntity>,
) {
    for event in events.read() {
        if let Some(relationship) = relationships.iter().find(|r| r.id == event.relationship_id) {
            // Check if both identities still exist
            let source_exists = identities
                .iter()
                .any(|i| i.id == relationship.source_identity);
            let target_exists = identities
                .iter()
                .any(|i| i.id == relationship.target_identity);

            let is_valid = source_exists && target_exists;

            validated_events.write(RelationshipValidated {
                relationship_id: event.relationship_id,
                is_valid,
                validated_at: SystemTime::now(),
                validation_reason: if is_valid {
                    "Both identities exist".to_string()
                } else {
                    "One or both identities missing".to_string()
                },
            });
        }
    }
}

/// System to traverse relationships (find connected identities)
pub fn traverse_relationships_system(
    relationships: Query<&IdentityRelationship>,
    identities: Query<&IdentityEntity>,
) -> Vec<(Uuid, Uuid, RelationshipType)> {
    relationships
        .iter()
        .filter(|r| {
            // Only include relationships where both identities exist
            identities.iter().any(|i| i.id == r.source_identity)
                && identities.iter().any(|i| i.id == r.target_identity)
        })
        .map(|r| {
            (
                r.source_identity,
                r.target_identity,
                r.relationship_type.clone(),
            )
        })
        .collect()
}

/// System to expire relationships
pub fn expire_relationships_system(
    mut commands: Commands,
    mut expired_events: EventWriter<RelationshipExpired>,
    relationships: Query<(Entity, &IdentityRelationship)>,
    time: Res<Time>,
) {
    // Only check for expiration periodically to improve performance
    let check_interval = 5.0; // Check every 5 seconds
    let elapsed = time.elapsed_secs();

    // Skip if not time to check
    if elapsed % check_interval > time.delta_secs() {
        return;
    }

    let current_time = SystemTime::now();

    for (entity, relationship) in relationships.iter() {
        if let Some(expires_at) = relationship.expires_at {
            if current_time > expires_at {
                // Remove the expired relationship
                commands.entity(entity).despawn();

                // Emit expiration event
                expired_events.write(RelationshipExpired {
                    relationship_id: relationship.id,
                    expired_at: current_time,
                    reason: format!(
                        "Expired after {} seconds",
                        current_time
                            .duration_since(relationship.established_at)
                            .unwrap_or_default()
                            .as_secs()
                    ),
                });
            }
        }
    }
}
