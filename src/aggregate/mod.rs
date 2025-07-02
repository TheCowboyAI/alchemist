//! Aggregate definitions for the application

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

use crate::value_objects::*;

/// Agent ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

impl AgentId {
    /// Create a new agent ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

/// Agent aggregate
#[derive(Debug, Clone)]
pub struct Agent {
    /// Agent ID
    pub id: AgentId,
    /// Agent type
    pub agent_type: AgentType,
    /// Agent status
    pub status: AgentStatus,
    /// Agent metadata
    pub metadata: AgentMetadata,
}

/// Agent marker component
#[derive(Component, Debug, Clone)]
pub struct AgentMarker {
    /// Agent ID
    pub id: AgentId,
}

/// Agent metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Created at
    pub created_at: SystemTime,
    /// Updated at
    pub updated_at: SystemTime,
}

/// Agent status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Active status
    Active,
    /// Inactive status
    Inactive,
    /// Suspended status
    Suspended,
    /// Decommissioned status
    Decommissioned,
}

/// Authentication component
#[derive(Component, Debug, Clone)]
pub struct AuthenticationComponent {
    /// Auth method
    pub method: AuthMethod,
    /// Credentials
    pub credentials: HashMap<String, String>,
}

/// Capabilities component
#[derive(Component, Debug, Clone)]
pub struct CapabilitiesComponent {
    /// Capabilities list
    pub capabilities: Vec<String>,
}

/// Configuration component
#[derive(Component, Debug, Clone)]
pub struct ConfigurationComponent {
    /// Configuration settings
    pub settings: HashMap<String, String>,
}

/// Permissions component
#[derive(Component, Debug, Clone)]
pub struct PermissionsComponent {
    /// Permissions list
    pub permissions: Vec<String>,
}

/// Tool access component
#[derive(Component, Debug, Clone)]
pub struct ToolAccessComponent {
    /// Enabled tools
    pub enabled_tools: Vec<String>,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool version
    pub version: String,
}

/// Tool usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageStats {
    /// Tool name
    pub tool_name: String,
    /// Usage count
    pub usage_count: u64,
    /// Last used
    pub last_used: Option<SystemTime>,
}

/// Identity aggregate
#[derive(Debug, Clone)]
pub struct IdentityAggregate {
    /// Identity ID
    pub id: Uuid,
    /// Identity type
    pub identity_type: IdentityType,
    /// Verification level
    pub verification_level: VerificationLevel,
}

impl IdentityAggregate {
    /// Validate a relationship between two identities
    pub fn validate_relationship(
        source_id: Uuid,
        target_id: Uuid,
        relationship_type: &crate::value_objects::RelationshipType,
    ) -> Result<(), String> {
        // Basic validation
        if source_id == target_id {
            return Err("Cannot create relationship with self".to_string());
        }

        // Additional validation could go here based on relationship type
        match relationship_type {
            crate::value_objects::RelationshipType::ParentChild => {
                // Could add age/type validation here
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
