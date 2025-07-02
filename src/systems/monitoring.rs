//! Agent monitoring systems

use crate::components::*;
use bevy::prelude::*;
use bevy_app::prelude::*;

/// System for updating agent activity
///
/// ```mermaid
/// graph LR
///     A[Timer] --> B[update_agent_activity_system]
///     B --> C[Check Activity]
///     C --> D[Update Status]
/// ```
pub fn update_agent_activity_system(mut agent_query: Query<(&AgentEntity, &mut AgentActivity)>) {
    let now = chrono::Utc::now();

    for (entity, mut activity) in agent_query.iter_mut() {
        // Check if agent has been idle too long
        if activity.time_since_activity() > chrono::Duration::minutes(5) {
            activity.activity_type = ActivityType::Idle;
            activity.is_active = false;
        }
    }
}
