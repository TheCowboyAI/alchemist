use bevy_ecs::prelude::*;
use uuid::Uuid;
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    components::{IdentityProjection, ProjectionType, RelationshipGraph},
    events::*,
};

use crate::aggregate::{AgentId, AgentType, AgentStatus}; 