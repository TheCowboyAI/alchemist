//! Projection types for the application

use bevy::prelude::*;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use crate::aggregate::{AgentId, AgentStatus};
use crate::value_objects::AgentType;

/// Agent view projection
#[derive(Debug, Clone)]
pub struct AgentView {
    /// Agent ID
    pub id: AgentId,
    /// Agent type
    pub agent_type: AgentType,
    /// Agent status
    pub status: AgentStatus,
    /// Agent name
    pub name: String,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Permissions
    pub permissions: Vec<String>,
}

/// Identity view projection
#[derive(Debug, Clone)]
pub struct IdentityView {
    /// Identity ID
    pub id: Uuid,
    /// Identity type
    pub identity_type: crate::value_objects::IdentityType,
    /// External reference
    pub external_reference: Option<String>,
    /// Verification level
    pub verification_level: crate::value_objects::VerificationLevel,
    /// Last verified
    pub last_verified: Option<SystemTime>,
    /// Created at
    pub created_at: SystemTime,
    /// Updated at
    pub updated_at: SystemTime,
}

/// Relationship view projection
#[derive(Debug, Clone)]
pub struct RelationshipView {
    /// Relationship ID
    pub id: Uuid,
    /// Source identity
    pub source_identity: Uuid,
    /// Target identity
    pub target_identity: Uuid,
    /// Relationship type
    pub relationship_type: crate::value_objects::RelationshipType,
    /// Established at
    pub established_at: SystemTime,
    /// Expires at
    pub expires_at: Option<SystemTime>,
    /// Metadata
    pub metadata: std::collections::HashMap<String, String>,
} 