/// Lifecycle management systems
pub mod lifecycle;
/// Projection synchronization systems
pub mod projection;
/// Relationship management systems
pub mod relationship;
/// Verification systems
pub mod verification;
/// Workflow processing systems
pub mod workflow;

use bevy::prelude::*;

// Re-export specific items to avoid name conflicts
pub use lifecycle::{
    archive_identity_system, create_identity_system, merge_identities_system,
    update_identity_system,
};

pub use relationship::{
    establish_relationship_system, expire_relationships_system, traverse_relationships_system,
    validate_relationship_system,
};

pub use workflow::{
    complete_workflow_system, handle_workflow_timeouts_system, process_workflow_steps_system,
    start_workflow_system,
};

pub use verification::{
    complete_verification_system, expire_verifications_system, process_verification_system,
    start_verification_system,
};

pub use projection::{
    create_projection_system, sync_projections_system, validate_projections_system,
};
