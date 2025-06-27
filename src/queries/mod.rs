use bevy_ecs::prelude::*;
use uuid::Uuid;
use std::time::{SystemTime, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    components::{
        IdentityEntity, IdentityVerification, IdentityRelationship,
        IdentityWorkflow, IdentityProjection, IdentityMetadata,
    },
    value_objects::{
        IdentityType, VerificationLevel, RelationshipType,
        WorkflowType, WorkflowStatus, ProjectionType,
        RelationshipId,
    },
};

pub fn find_identity_by_id(
    world: &mut World,
    identity_id: Uuid,
) -> Option<IdentityView> {
    world.query_filtered::<(&IdentityEntity, &IdentityMetadata, &IdentityVerification), ()>()
        .iter(world)
        .find(|(entity, _, _)| entity.id == identity_id)
        .map(|(entity, metadata, verification)| IdentityView {
            id: entity.id,
            identity_type: entity.identity_type.clone(),
            external_reference: entity.external_reference.clone(),
            verification_level: verification.verification_level.clone(),
            last_verified: verification.last_verified,
            created_at: metadata.created_at,
            updated_at: metadata.updated_at,
        })
}

pub fn find_identities_by_type(
    world: &mut World,
    identity_type: IdentityType,
) -> Vec<IdentityView> {
    world.query_filtered::<(&IdentityEntity, &IdentityMetadata, &IdentityVerification), ()>()
        .iter(world)
        .filter(|(entity, _, _)| entity.identity_type == identity_type)
        .map(|(entity, metadata, verification)| IdentityView {
            id: entity.id,
            identity_type: entity.identity_type.clone(),
            external_reference: entity.external_reference.clone(),
            verification_level: verification.verification_level.clone(),
            last_verified: verification.last_verified,
            created_at: metadata.created_at,
            updated_at: metadata.updated_at,
        })
        .collect()
}

pub fn find_relationships_by_identity(
    world: &mut World,
    identity_id: Uuid,
) -> Vec<RelationshipView> {
    world.query_filtered::<&IdentityRelationship, ()>()
        .iter(world)
        .filter(|rel| rel.source_identity == identity_id || rel.target_identity == identity_id)
        .map(|rel| RelationshipView {
            id: rel.id,
            source_identity: rel.source_identity,
            target_identity: rel.target_identity,
            relationship_type: rel.relationship_type.clone(),
            established_at: rel.established_at,
            expires_at: rel.expires_at,
            metadata: rel.metadata.clone(),
        })
        .collect()
}

pub fn find_identities_by_verification_level(
    world: &mut World,
    min_level: VerificationLevel,
) -> Vec<IdentityView> {
    world.query_filtered::<(&IdentityEntity, &IdentityVerification, &IdentityMetadata), ()>()
        .iter(world)
        .filter(|(_, verification, _)| verification.verification_level >= min_level)
        .map(|(entity, verification, metadata)| IdentityView {
            id: entity.id,
            identity_type: entity.identity_type.clone(),
            external_reference: entity.external_reference.clone(),
            verification_level: verification.verification_level.clone(),
            last_verified: verification.last_verified,
            created_at: metadata.created_at,
            updated_at: metadata.updated_at,
        })
        .collect()
}

pub fn find_expired_verifications(
    world: &mut World,
    current_time: SystemTime,
) -> Vec<IdentityView> {
    world.query_filtered::<(&IdentityEntity, &IdentityVerification, &IdentityMetadata), ()>()
        .iter(world)
        .filter(|(_, verification, _)| {
            verification.last_verified
                .map(|last| current_time.duration_since(last)
                    .map(|duration| duration > Duration::from_secs(30 * 24 * 60 * 60))
                    .unwrap_or(false))
                .unwrap_or(true)
        })
        .map(|(entity, verification, metadata)| IdentityView {
            id: entity.id,
            identity_type: entity.identity_type.clone(),
            external_reference: entity.external_reference.clone(),
            verification_level: verification.verification_level.clone(),
            last_verified: verification.last_verified,
            created_at: metadata.created_at,
            updated_at: metadata.updated_at,
        })
        .collect()
} 