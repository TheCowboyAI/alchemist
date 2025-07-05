//! ECS components for the application

use bevy::prelude::*;
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

pub mod visualization_3d;
pub use visualization_3d::*;

pub mod graph;
pub use graph::*;

use crate::aggregate::AgentId;
use crate::value_objects::{
    AgentType, IdentityType, ProjectionContext, ProjectionType, RelationshipType,
    VerificationLevel, VerificationMethod, WorkflowStatus, WorkflowType,
};

// Agent components
/// Agent entity component
#[derive(Component, Debug, Clone)]
pub struct AgentEntity {
    /// Agent ID
    pub id: AgentId,
    /// Agent type
    pub agent_type: AgentType,
}

/// Agent owner component
#[derive(Component, Debug, Clone)]
pub struct AgentOwner {
    /// Owner ID
    pub owner_id: Uuid,
}

/// Agent type component
#[derive(Component, Debug, Clone)]
pub struct AgentTypeComponent(pub AgentType);

/// Created at component
#[derive(Component, Debug, Clone)]
pub struct CreatedAt(pub SystemTime);

/// Last active component
#[derive(Component, Debug, Clone)]
pub struct LastActive(pub SystemTime);

/// Updated at component
#[derive(Component, Debug, Clone)]
pub struct UpdatedAt(pub SystemTime);

// Identity components
/// Identity entity component
#[derive(Component, Debug, Clone)]
pub struct IdentityEntity {
    /// Identity ID
    pub id: Uuid,
    /// Identity type
    pub identity_type: IdentityType,
    /// External reference
    pub external_reference: Option<String>,
}

/// Identity verification component
#[derive(Component, Debug, Clone)]
pub struct IdentityVerification {
    /// Verification level
    pub verification_level: VerificationLevel,
    /// Verification method
    pub verification_method: Option<VerificationMethod>,
    /// Last verified
    pub last_verified: Option<SystemTime>,
}

/// Identity relationship component
#[derive(Component, Debug, Clone)]
pub struct IdentityRelationship {
    /// Relationship ID
    pub id: Uuid,
    /// Source identity
    pub source_identity: Uuid,
    /// Target identity
    pub target_identity: Uuid,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Established at
    pub established_at: SystemTime,
    /// Expires at
    pub expires_at: Option<SystemTime>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Identity workflow component
#[derive(Component, Debug, Clone)]
pub struct IdentityWorkflow {
    /// Workflow ID
    pub id: Uuid,
    /// Workflow type
    pub workflow_type: WorkflowType,
    /// Status
    pub status: WorkflowStatus,
}

/// Identity projection component
#[derive(Component, Debug, Clone)]
pub struct IdentityProjection {
    /// Projection type
    pub projection_type: ProjectionType,
    /// Context
    pub context: ProjectionContext,
}

/// Identity metadata component
#[derive(Component, Debug, Clone)]
pub struct IdentityMetadata {
    /// Created at
    pub created_at: SystemTime,
    /// Updated at
    pub updated_at: SystemTime,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Relationship graph component
#[derive(Component, Debug, Clone)]
pub struct RelationshipGraph {
    /// Graph data
    pub nodes: Vec<Uuid>,
    /// Edges
    pub edges: Vec<(Uuid, Uuid)>,
}

/// Selected component for graph editor
#[derive(Component)]
pub struct Selected;
