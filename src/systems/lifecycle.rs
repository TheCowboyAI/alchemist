use bevy_ecs::prelude::*;
use uuid::Uuid;

use crate::{
    components::{
        IdentityEntity, IdentityVerification, IdentityMetadata,
        VerificationLevel, IdentityType,
    },
    events::{IdentityCreated, IdentityUpdated, IdentityMerged, IdentityArchived},
    commands::{CreateIdentityCommand, UpdateIdentityCommand, MergeIdentitiesCommand, ArchiveIdentityCommand},
    aggregate::IdentityAggregate,
}; 