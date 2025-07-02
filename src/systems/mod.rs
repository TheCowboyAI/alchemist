/// Lifecycle management systems
pub mod lifecycle;
/// Relationship management systems
pub mod relationship;
/// Workflow processing systems
pub mod workflow;
/// Verification systems
pub mod verification;
/// Projection synchronization systems
pub mod projection;

use bevy::prelude::*;

// Re-export specific items to avoid name conflicts
pub use lifecycle::{
    create_identity_system,
    update_identity_system,
    merge_identities_system,
    archive_identity_system,
};

pub use relationship::{
    establish_relationship_system,
    validate_relationship_system,
    traverse_relationships_system,
    expire_relationships_system,
};

pub use workflow::{
    start_workflow_system,
    process_workflow_steps_system,
    complete_workflow_system,
    handle_workflow_timeouts_system,
};

pub use verification::{
    start_verification_system,
    process_verification_system,
    complete_verification_system,
    expire_verifications_system,
};

pub use projection::{
    create_projection_system,
    sync_projections_system,
    validate_projections_system,
}; 