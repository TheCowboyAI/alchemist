//! Event definitions for the application

use serde::{Deserialize, Serialize};
use bevy::prelude::*;
use uuid::Uuid;

/// AI-related events
pub mod ai_events;
pub use ai_events::*;

// Agent-related events
/// Agent activated event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct AgentActivated {
    /// Agent ID
    pub agent_id: String,
}

/// Agent capabilities added event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct AgentCapabilitiesAdded {
    /// Agent ID
    pub agent_id: String,
    /// Capabilities added
    pub capabilities: Vec<String>,
}

/// Agent capabilities removed event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct AgentCapabilitiesRemoved {
    /// Agent ID
    pub agent_id: String,
    /// Capabilities removed
    pub capabilities: Vec<String>,
}

/// Agent configuration removed event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct AgentConfigurationRemoved {
    /// Agent ID
    pub agent_id: String,
}

/// Agent configuration set event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct AgentConfigurationSet {
    /// Agent ID
    pub agent_id: String,
}

/// Agent decommissioned event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct AgentDecommissioned {
    /// Agent ID
    pub agent_id: String,
}

/// Agent deployed event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct AgentDeployed {
    /// Agent ID
    pub agent_id: crate::aggregate::AgentId,
    /// Owner ID
    pub owner_id: Uuid,
    /// Agent type
    pub agent_type: crate::value_objects::AgentType,
}

/// Agent permissions granted event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct AgentPermissionsGranted {
    /// Agent ID
    pub agent_id: String,
    /// Permissions granted
    pub permissions: Vec<String>,
}

/// Agent permissions revoked event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct AgentPermissionsRevoked {
    /// Agent ID
    pub agent_id: String,
    /// Permissions revoked
    pub permissions: Vec<String>,
}

/// Agent suspended event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct AgentSuspended {
    /// Agent ID
    pub agent_id: String,
}

/// Agent tools disabled event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct AgentToolsDisabled {
    /// Agent ID
    pub agent_id: String,
    /// Tools disabled
    pub tools: Vec<String>,
}

/// Agent tools enabled event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct AgentToolsEnabled {
    /// Agent ID
    pub agent_id: String,
    /// Tools enabled
    pub tools: Vec<String>,
}

/// Agent went offline event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct AgentWentOffline {
    /// Agent ID
    pub agent_id: String,
}

// Relationship events
/// Relationship established event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct RelationshipEstablished {
    /// Relationship ID
    pub relationship_id: Uuid,
    /// Source identity
    pub source_identity: Uuid,
    /// Target identity
    pub target_identity: Uuid,
    /// Relationship type
    pub relationship_type: crate::value_objects::RelationshipType,
    /// Established at
    pub established_at: std::time::SystemTime,
}

/// Relationship validated event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct RelationshipValidated {
    /// Relationship ID
    pub relationship_id: Uuid,
    /// Is valid
    pub is_valid: bool,
    /// Validated at
    pub validated_at: std::time::SystemTime,
    /// Validation reason
    pub validation_reason: String,
}

/// Relationship expired event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct RelationshipExpired {
    /// Relationship ID
    pub relationship_id: Uuid,
    /// Expired at
    pub expired_at: std::time::SystemTime,
    /// Reason
    pub reason: String,
}

// Projection events
/// Projection created event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct ProjectionCreated {
    /// Projection ID
    pub projection_id: String,
    /// Projection type
    pub projection_type: String,
}

// Identity link events
/// Identity linked to person event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct IdentityLinkedToPerson {
    /// Identity ID
    pub identity_id: String,
    /// Person ID
    pub person_id: String,
}

/// Identity linked to organization event
#[derive(Debug, Clone, Serialize, Deserialize, Event)]
pub struct IdentityLinkedToOrganization {
    /// Identity ID
    pub identity_id: String,
    /// Organization ID
    pub organization_id: String,
}

// Import value objects for graph events
use crate::value_objects::{NodeId, EdgeId, GraphId, NodeType, EdgeRelationship, Position3D};

// Graph events
/// Node added event
#[derive(Event, Debug, Clone)]
pub struct NodeAdded {
    /// Node ID
    pub node_id: NodeId,
    /// Graph ID
    pub graph_id: GraphId,
    /// Node type
    pub node_type: NodeType,
    /// Position
    pub position: Position3D,
    /// Metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Edge added event
#[derive(Event, Debug, Clone)]
pub struct EdgeAdded {
    /// Edge ID
    pub edge_id: EdgeId,
    /// Graph ID
    pub graph_id: GraphId,
    /// Source node
    pub source: NodeId,
    /// Target node
    pub target: NodeId,
    /// Relationship type
    pub relationship: EdgeRelationship,
}

/// Node removed event
#[derive(Event, Debug, Clone)]
pub struct NodeRemoved {
    /// Node ID
    pub node_id: NodeId,
    /// Graph ID
    pub graph_id: GraphId,
}

/// Edge removed event
#[derive(Event, Debug, Clone)]
pub struct EdgeRemoved {
    /// Edge ID
    pub edge_id: EdgeId,
    /// Graph ID
    pub graph_id: GraphId,
} 