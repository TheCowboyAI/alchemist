use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use crate::value_objects::{
    IdentityType, VerificationLevel, VerificationMethod,
    RelationshipType, WorkflowType, ProjectionType, ProjectionContext,
};
use crate::aggregate::AgentId;

// Agent commands
/// Activate agent command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateAgent {
    /// Agent ID
    pub agent_id: AgentId,
}

/// Decommission agent command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecommissionAgent {
    /// Agent ID
    pub agent_id: AgentId,
}

/// Deploy agent command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployAgent {
    /// Agent ID
    pub agent_id: AgentId,
    /// Deployment target
    pub target: String,
}

/// Disable agent tools command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisableAgentTools {
    /// Agent ID
    pub agent_id: AgentId,
    /// Tools to disable
    pub tools: Vec<String>,
}

/// Enable agent tools command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnableAgentTools {
    /// Agent ID
    pub agent_id: AgentId,
    /// Tools to enable
    pub tools: Vec<String>,
}

/// Grant agent permissions command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantAgentPermissions {
    /// Agent ID
    pub agent_id: AgentId,
    /// Permissions to grant
    pub permissions: Vec<String>,
}

/// Remove agent configuration command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveAgentConfiguration {
    /// Agent ID
    pub agent_id: AgentId,
    /// Configuration keys to remove
    pub keys: Vec<String>,
}

/// Revoke agent permissions command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeAgentPermissions {
    /// Agent ID
    pub agent_id: AgentId,
    /// Permissions to revoke
    pub permissions: Vec<String>,
}

/// Set agent configuration command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetAgentConfiguration {
    /// Agent ID
    pub agent_id: AgentId,
    /// Configuration settings
    pub settings: std::collections::HashMap<String, String>,
}

/// Set agent offline command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetAgentOffline {
    /// Agent ID
    pub agent_id: AgentId,
}

/// Suspend agent command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspendAgent {
    /// Agent ID
    pub agent_id: AgentId,
    /// Reason for suspension
    pub reason: String,
}

/// Change agent capabilities command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeAgentCapabilities {
    /// Agent ID
    pub agent_id: AgentId,
    /// Capabilities to add
    pub add: Vec<String>,
    /// Capabilities to remove
    pub remove: Vec<String>,
}

// Relationship commands
/// Establish relationship command
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct EstablishRelationshipCommand {
    /// Source identity
    pub source_identity: uuid::Uuid,
    /// Target identity
    pub target_identity: uuid::Uuid,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Expires at
    pub expires_at: Option<SystemTime>,
    /// Metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Validate relationship command
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct ValidateRelationshipCommand {
    /// Relationship ID
    pub relationship_id: uuid::Uuid,
}

/// Expire relationship command
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct ExpireRelationshipCommand {
    /// Relationship ID
    pub relationship_id: uuid::Uuid,
}

// Identity commands
/// Create identity command
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct CreateIdentityCommand {
    /// Identity ID
    pub identity_id: uuid::Uuid,
    /// Identity type
    pub identity_type: IdentityType,
    /// External reference
    pub external_reference: Option<String>,
    /// Person ID (if linked)
    pub person_id: Option<uuid::Uuid>,
}

/// Merge identities command
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct MergeIdentitiesCommand {
    /// Source identity to merge from
    pub source_id: uuid::Uuid,
    /// Target identity to merge into
    pub target_id: uuid::Uuid,
}

/// Archive identity command
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct ArchiveIdentityCommand {
    /// Identity to archive
    pub identity_id: uuid::Uuid,
}

// Projection commands
/// Create projection command
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct CreateProjectionCommand {
    /// Projection type
    pub projection_type: ProjectionType,
    /// Context
    pub context: ProjectionContext,
} 