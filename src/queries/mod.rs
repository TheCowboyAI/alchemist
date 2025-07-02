use bevy::prelude::*;
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
    projections::{IdentityView, RelationshipView},
};

/// Find an identity by its unique ID
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

/// Find all identities of a specific type
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

/// Find all relationships associated with a specific identity
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

/// Find all identities with a minimum verification level
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

/// Find all identities with expired verifications
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

/// Agent query type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentQuery {
    /// Find by ID
    FindById(Uuid),
    /// Find by type
    FindByType(crate::value_objects::AgentType),
    /// Find by status
    FindByStatus(crate::aggregate::AgentStatus),
}

/// Agent query handler
pub struct AgentQueryHandler;

impl AgentQueryHandler {
    /// Execute a query
    pub fn execute(query: AgentQuery, world: &mut World) -> Vec<crate::projections::AgentView> {
        use crate::components::{AgentEntity, AgentOwner};
        
        match query {
            AgentQuery::FindById(agent_id) => {
                // Query for agents with the matching ID
                // In a real implementation, we would query from a proper read model/projection
                // For now, we create a basic view from the entity components
                world.query::<&AgentEntity>()
                    .iter(world)
                    .filter(|entity| entity.id.0 == agent_id)
                    .map(|entity| {
                        // Create view with default data
                        // In production, this would come from a proper projection
                        crate::projections::AgentView {
                            id: entity.id,
                            agent_type: entity.agent_type.clone(),
                            status: crate::aggregate::AgentStatus::Active, // Default status
                            name: format!("Agent {}", entity.id.0),
                            capabilities: vec!["basic".to_string()], // Default capabilities
                            permissions: vec!["read".to_string()], // Default permissions
                        }
                    })
                    .collect()
            }
            AgentQuery::FindByType(agent_type) => {
                world.query::<&AgentEntity>()
                    .iter(world)
                    .filter(|entity| entity.agent_type == agent_type)
                    .map(|entity| {
                        crate::projections::AgentView {
                            id: entity.id,
                            agent_type: entity.agent_type.clone(),
                            status: crate::aggregate::AgentStatus::Active,
                            name: format!("Agent {}", entity.id.0),
                            capabilities: vec!["basic".to_string()],
                            permissions: vec!["read".to_string()],
                        }
                    })
                    .collect()
            }
            AgentQuery::FindByStatus(status) => {
                // Since we don't store status in the component, we return empty for non-Active status
                if status != crate::aggregate::AgentStatus::Active {
                    return Vec::new();
                }
                
                // Return all agents as "Active" for now
                world.query::<&AgentEntity>()
                    .iter(world)
                    .map(|entity| {
                        crate::projections::AgentView {
                            id: entity.id,
                            agent_type: entity.agent_type.clone(),
                            status: crate::aggregate::AgentStatus::Active,
                            name: format!("Agent {}", entity.id.0),
                            capabilities: vec!["basic".to_string()],
                            permissions: vec!["read".to_string()],
                        }
                    })
                    .collect()
            }
        }
    }
} 