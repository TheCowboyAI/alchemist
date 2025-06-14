#!/usr/bin/env bash
set -euo pipefail

# Extract Policy Domain Script
# This script extracts policy-related code from cim-domain into cim-domain-policy

echo "=== Extracting Policy Domain ==="

# 1. Create the new policy domain directory structure
echo "Creating cim-domain-policy directory structure..."
mkdir -p cim-domain-policy/{src,tests}
mkdir -p cim-domain-policy/src/{aggregate,commands,events,handlers,projections,queries,value_objects}

# 2. Create Cargo.toml for policy domain
echo "Creating Cargo.toml..."
cat > cim-domain-policy/Cargo.toml << 'EOF'
[package]
name = "cim-domain-policy"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core dependencies
uuid = { version = "1.11", features = ["v4", "serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2.0"
async-trait = "0.1"

# Domain dependencies
cim-core-domain = { path = "../cim-core-domain" }

[dev-dependencies]
tokio = { version = "1.42", features = ["full"] }
EOF

# 3. Create lib.rs
echo "Creating lib.rs..."
cat > cim-domain-policy/src/lib.rs << 'EOF'
//! Policy domain module
//!
//! This module contains all policy-related domain logic including:
//! - Policy aggregate and components
//! - Policy commands and events
//! - Policy command and query handlers

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod value_objects;

// Re-export main types
pub use aggregate::{
    Policy, PolicyMarker, PolicyMetadata, PolicyStatus, PolicyType,
    PolicyScope, PolicyException, ViolationSeverity, EnforcementMode,
    RulesComponent, ApprovalRequirementsComponent, ApprovalStateComponent,
    EnforcementComponent, ExternalApprovalRequirement, Approval, Rejection,
    PendingExternalApproval, ExternalVerification, ViolationAction,
};

pub use commands::{
    EnactPolicy, UpdatePolicyRules, SubmitPolicyForApproval,
    ApprovePolicy, RejectPolicy, SuspendPolicy, ReactivatePolicy,
    SupersedePolicy, ArchivePolicy, RequestPolicyExternalApproval,
    RecordPolicyExternalApproval,
};

pub use events::{
    PolicyEnacted, PolicySubmittedForApproval, PolicyApproved,
    PolicyRejected, PolicySuspended, PolicyReactivated,
    PolicySuperseded, PolicyArchived, PolicyExternalApprovalRequested,
    PolicyExternalApprovalReceived,
};

pub use handlers::{PolicyCommandHandler, PolicyEventHandler};
pub use projections::PolicyView;
pub use queries::{PolicyQuery, PolicyQueryHandler, FindActivePolicies};
EOF

# 4. Move policy aggregate
echo "Moving policy aggregate..."
cp cim-domain/src/policy.rs cim-domain-policy/src/aggregate/mod.rs

# 5. Extract policy commands
echo "Extracting policy commands..."
cat > cim-domain-policy/src/commands/mod.rs << 'EOF'
//! Policy commands

use cim_core_domain::command::Command;
use cim_core_domain::entity::EntityId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Enact a new policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnactPolicy {
    /// Policy ID
    pub policy_id: Uuid,
    /// Policy type
    pub policy_type: crate::PolicyType,
    /// Policy scope
    pub scope: crate::PolicyScope,
    /// Owner ID
    pub owner_id: Uuid,
    /// Policy metadata
    pub metadata: crate::PolicyMetadata,
}

impl Command for EnactPolicy {
    type Aggregate = crate::Policy;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(self.policy_id))
    }
}

/// Update policy rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePolicyRules {
    /// Policy ID
    pub policy_id: Uuid,
    /// New rules
    pub rules: serde_json::Value,
    /// Rule engine type
    pub engine: String,
    /// Rule version
    pub version: String,
}

impl Command for UpdatePolicyRules {
    type Aggregate = crate::Policy;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(self.policy_id))
    }
}

/// Submit policy for approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitPolicyForApproval {
    /// Policy ID
    pub policy_id: Uuid,
    /// Submission notes
    pub notes: Option<String>,
}

impl Command for SubmitPolicyForApproval {
    type Aggregate = crate::Policy;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(self.policy_id))
    }
}

/// Approve a policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovePolicy {
    /// Policy ID
    pub policy_id: Uuid,
    /// Approver ID
    pub approver_id: Uuid,
    /// Approval comments
    pub comments: Option<String>,
    /// External verification if required
    pub external_verification: Option<crate::ExternalVerification>,
}

impl Command for ApprovePolicy {
    type Aggregate = crate::Policy;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(self.policy_id))
    }
}

/// Reject a policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectPolicy {
    /// Policy ID
    pub policy_id: Uuid,
    /// Rejector ID
    pub rejector_id: Uuid,
    /// Rejection reason
    pub reason: String,
}

impl Command for RejectPolicy {
    type Aggregate = crate::Policy;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(self.policy_id))
    }
}

/// Suspend a policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspendPolicy {
    /// Policy ID
    pub policy_id: Uuid,
    /// Suspension reason
    pub reason: String,
}

impl Command for SuspendPolicy {
    type Aggregate = crate::Policy;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(self.policy_id))
    }
}

/// Reactivate a suspended policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactivatePolicy {
    /// Policy ID
    pub policy_id: Uuid,
}

impl Command for ReactivatePolicy {
    type Aggregate = crate::Policy;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(self.policy_id))
    }
}

/// Supersede a policy with another
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupersedePolicy {
    /// Policy ID being superseded
    pub policy_id: Uuid,
    /// New policy ID that supersedes this one
    pub new_policy_id: Uuid,
}

impl Command for SupersedePolicy {
    type Aggregate = crate::Policy;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(self.policy_id))
    }
}

/// Archive a policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivePolicy {
    /// Policy ID
    pub policy_id: Uuid,
}

impl Command for ArchivePolicy {
    type Aggregate = crate::Policy;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(self.policy_id))
    }
}

/// Request external approval for a policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPolicyExternalApproval {
    /// Policy ID
    pub policy_id: Uuid,
    /// Type of approval required
    pub approval_type: String,
    /// Request metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Command for RequestPolicyExternalApproval {
    type Aggregate = crate::Policy;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(self.policy_id))
    }
}

/// Record external approval received
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordPolicyExternalApproval {
    /// Policy ID
    pub policy_id: Uuid,
    /// Request ID this approval is for
    pub request_id: Uuid,
    /// External verification details
    pub verification: crate::ExternalVerification,
}

impl Command for RecordPolicyExternalApproval {
    type Aggregate = crate::Policy;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        Some(EntityId::from_uuid(self.policy_id))
    }
}
EOF

# 6. Create handlers directory
echo "Creating handlers..."
cat > cim-domain-policy/src/handlers/mod.rs << 'EOF'
//! Policy handlers

pub mod command_handler;
pub mod event_handler;

pub use command_handler::PolicyCommandHandler;
pub use event_handler::PolicyEventHandler;
EOF

cat > cim-domain-policy/src/handlers/command_handler.rs << 'EOF'
//! Policy command handler implementation

use crate::{Policy, commands::*};
use cim_core_domain::command::CommandHandler;
use cim_core_domain::repository::AggregateRepository;
use async_trait::async_trait;

/// Policy command handler
pub struct PolicyCommandHandler<R: AggregateRepository<Policy>> {
    repository: R,
}

impl<R: AggregateRepository<Policy>> PolicyCommandHandler<R> {
    /// Create a new policy command handler
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: AggregateRepository<Policy> + Send + Sync> CommandHandler<EnactPolicy> for PolicyCommandHandler<R> {
    type Error = cim_core_domain::errors::DomainError;

    async fn handle(&self, command: EnactPolicy) -> Result<(), Self::Error> {
        let mut policy = Policy::new(
            command.policy_id,
            command.policy_type,
            command.scope,
            command.owner_id,
        );

        // Add metadata component
        policy.add_component(command.metadata)?;

        self.repository.save(&policy).await?;
        Ok(())
    }
}

// Additional command handlers would be implemented similarly...
EOF

cat > cim-domain-policy/src/handlers/event_handler.rs << 'EOF'
//! Policy event handler implementation

use crate::events::*;
use cim_core_domain::event::EventHandler;
use async_trait::async_trait;

/// Policy event handler
pub struct PolicyEventHandler;

#[async_trait]
impl EventHandler<PolicyEnacted> for PolicyEventHandler {
    type Error = cim_core_domain::errors::DomainError;

    async fn handle(&self, _event: PolicyEnacted) -> Result<(), Self::Error> {
        // Handle policy enacted event
        Ok(())
    }
}

// Additional event handlers would be implemented similarly...
EOF

# 7. Create empty directories
echo "Creating remaining directories..."
touch cim-domain-policy/src/events/mod.rs
touch cim-domain-policy/src/projections/mod.rs
touch cim-domain-policy/src/queries/mod.rs
touch cim-domain-policy/src/value_objects/mod.rs

# 8. Initialize git repository
echo "Initializing git repository..."
cd cim-domain-policy
git init
git add .
git commit -m "Initial commit: Policy domain extracted from cim-domain"

echo "=== Policy Domain Extraction Complete ==="
echo "Next steps:"
echo "1. Extract policy events from cim-domain/src/events.rs"
echo "2. Remove policy code from cim-domain"
echo "3. Push to GitHub repository"
echo "4. Add as submodule to main project"
