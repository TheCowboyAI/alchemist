//! Query systems for agent information

use bevy_ecs::prelude::*;
use bevy_app::prelude::*;
use crate::components::*;
use uuid::Uuid;

/// Query for finding agents by type
///
/// ```mermaid
/// graph LR
///     A[QueryByType] --> B[find_agents_by_type]
///     B --> C[Filter Agents]
///     C --> D[Return Results]
/// ```
pub fn find_agents_by_type(
    agent_type: AgentTypeComponent,
    agent_query: &Query<(&AgentEntity, &AgentTypeComponent, &AgentStatus)>,
) -> Vec<Uuid> {
    agent_query
        .iter()
        .filter(|(_, t, _)| **t == agent_type)
        .map(|(entity, _, _)| entity.agent_id)
        .collect()
}

/// Query for finding active agents
pub fn find_active_agents(
    agent_query: &Query<(&AgentEntity, &AgentStatus)>,
) -> Vec<Uuid> {
    agent_query
        .iter()
        .filter(|(_, status)| status.state == AgentState::Active)
        .map(|(entity, _)| entity.agent_id)
        .collect()
}

/// Query for finding agents with specific capability
pub fn find_agents_with_capability(
    capability: &str,
    agent_query: &Query<(&AgentEntity, &AgentCapabilities)>,
) -> Vec<Uuid> {
    agent_query
        .iter()
        .filter(|(_, caps)| caps.has(capability))
        .map(|(entity, _)| entity.agent_id)
        .collect()
} 