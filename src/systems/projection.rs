use bevy_ecs::prelude::*;
use uuid::Uuid;

use crate::{
    components::{
        IdentityEntity, IdentityVerification,
        IdentityProjection, ProjectionType, ProjectionContext,
    },
    events::{
        ProjectionCreated,
        IdentityLinkedToPerson, IdentityLinkedToOrganization,
    },
    commands::{CreateProjectionCommand},
}; 