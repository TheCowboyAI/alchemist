//! Tool management systems

use bevy_ecs::prelude::*;
use bevy_app::prelude::*;
use crate::components::*;
use crate::events::*;
use uuid::Uuid;

/// System for enabling/disabling agent tools
///
/// ```mermaid
/// graph LR
///     A[ToggleToolCommand] --> B[toggle_tools_system]
///     B --> C{Enable/Disable?}
///     C -->|Enable| D[AgentToolsEnabled]
///     C -->|Disable| E[AgentToolsDisabled]
/// ```
pub fn toggle_tools_system(
    mut toggle_commands: EventReader<ToggleToolCommand>,
    mut agent_query: Query<(&AgentEntity, &mut AgentToolAccess)>,
    mut enabled_events: EventWriter<AgentToolsEnabled>,
    mut disabled_events: EventWriter<AgentToolsDisabled>,
) {
    for toggle_cmd in toggle_commands.read() {
        let agent_found = agent_query.iter_mut()
            .find(|(entity, _)| entity.agent_id == toggle_cmd.agent_id);

        if let Some((_, mut tool_access)) = agent_found {
            if let Some(tool) = tool_access.tools.get_mut(&toggle_cmd.tool_id) {
                tool.enabled = toggle_cmd.enable;

                if toggle_cmd.enable {
                    enabled_events.write(AgentToolsEnabled {
                        agent_id: toggle_cmd.agent_id,
                        tools: vec![toggle_cmd.tool_id.clone()],
                        enabled_at: chrono::Utc::now(),
                        event_metadata: cim_domain::EventMetadata::default(),
                    });
                } else {
                    disabled_events.write(AgentToolsDisabled {
                        agent_id: toggle_cmd.agent_id,
                        tools: vec![toggle_cmd.tool_id.clone()],
                        disabled_at: chrono::Utc::now(),
                        event_metadata: cim_domain::EventMetadata::default(),
                    });
                }
            }
        }
    }
}

#[derive(Event)]
pub struct ToggleToolCommand {
    pub agent_id: Uuid,
    pub tool_id: String,
    pub enable: bool,
} 