//! Permissions management systems

use crate::components::*;
use crate::events::*;
use bevy::prelude::*;
use bevy_app::prelude::*;
use uuid::Uuid;

/// System for granting permissions to agents
///
/// ```mermaid
/// graph LR
///     A[GrantPermissionsCommand] --> B[grant_permissions_system]
///     B --> C{Agent Found?}
///     C -->|Yes| D[Update Permissions]
///     C -->|No| E[Log Error]
///     D --> F[PermissionsGrantedEvent]
/// ```
pub fn grant_permissions_system(
    mut grant_commands: EventReader<GrantPermissionsCommand>,
    mut agent_query: Query<(
        &AgentEntity,
        &mut AgentPermissions,
        Option<&mut PermissionAudit>,
    )>,
    mut granted_events: EventWriter<AgentPermissionsGranted>,
) {
    for grant_cmd in grant_commands.read() {
        // Find the agent
        let agent_found = agent_query
            .iter_mut()
            .find(|(entity, _, _)| entity.agent_id == grant_cmd.agent_id);

        if let Some((_, mut permissions, audit)) = agent_found {
            let mut actually_granted = Vec::new();

            // Grant each permission
            for permission in &grant_cmd.permissions {
                if !permissions.has(permission) {
                    permissions.grant(permission.clone());
                    actually_granted.push(permission.clone());

                    // Add audit entry
                    if let Some(mut audit_log) = audit.as_mut() {
                        audit_log.add_change(PermissionChange {
                            change_type: PermissionChangeType::Granted,
                            permission: permission.clone(),
                            changed_by: grant_cmd.granted_by,
                            timestamp: chrono::Utc::now(),
                            reason: grant_cmd.reason.clone(),
                        });
                    }
                }
            }

            // Emit event if permissions were actually granted
            if !actually_granted.is_empty() {
                granted_events.write(AgentPermissionsGranted {
                    agent_id: grant_cmd.agent_id,
                    permissions: actually_granted,
                    granted_at: chrono::Utc::now(),
                    event_metadata: cim_domain::EventMetadata::default(),
                });
            }
        }
    }
}

/// System for revoking permissions from agents
///
/// ```mermaid
/// graph LR
///     A[RevokePermissionsCommand] --> B[revoke_permissions_system]
///     B --> C{Agent Found?}
///     C -->|Yes| D[Remove Permissions]
///     C -->|No| E[Log Error]
///     D --> F[PermissionsRevokedEvent]
/// ```
pub fn revoke_permissions_system(
    mut revoke_commands: EventReader<RevokePermissionsCommand>,
    mut agent_query: Query<(
        &AgentEntity,
        &mut AgentPermissions,
        Option<&mut PermissionAudit>,
    )>,
    mut revoked_events: EventWriter<AgentPermissionsRevoked>,
) {
    for revoke_cmd in revoke_commands.read() {
        // Find the agent
        let agent_found = agent_query
            .iter_mut()
            .find(|(entity, _, _)| entity.agent_id == revoke_cmd.agent_id);

        if let Some((_, mut permissions, audit)) = agent_found {
            let mut actually_revoked = Vec::new();

            // Revoke each permission
            for permission in &revoke_cmd.permissions {
                if permissions.has(permission) {
                    permissions.deny(permission.clone());
                    actually_revoked.push(permission.clone());

                    // Add audit entry
                    if let Some(mut audit_log) = audit.as_mut() {
                        audit_log.add_change(PermissionChange {
                            change_type: PermissionChangeType::Revoked,
                            permission: permission.clone(),
                            changed_by: revoke_cmd.revoked_by,
                            timestamp: chrono::Utc::now(),
                            reason: revoke_cmd.reason.clone(),
                        });
                    }
                }
            }

            // Emit event if permissions were actually revoked
            if !actually_revoked.is_empty() {
                revoked_events.write(AgentPermissionsRevoked {
                    agent_id: revoke_cmd.agent_id,
                    permissions: actually_revoked,
                    revoked_at: chrono::Utc::now(),
                    event_metadata: cim_domain::EventMetadata::default(),
                });
            }
        }
    }
}

/// System for checking permission requirements
///
/// ```mermaid
/// graph LR
///     A[CheckPermissionCommand] --> B[check_permission_system]
///     B --> C{Has Permission?}
///     C -->|Yes| D[PermissionAllowed]
///     C -->|No| E[PermissionDenied]
/// ```
pub fn check_permission_system(
    mut check_commands: EventReader<CheckPermissionCommand>,
    agent_query: Query<(
        &AgentEntity,
        &AgentPermissions,
        Option<&PermissionInheritance>,
    )>,
    mut allowed_events: EventWriter<PermissionAllowedEvent>,
    mut denied_events: EventWriter<PermissionDeniedEvent>,
) {
    for check_cmd in check_commands.read() {
        // Find the agent
        let agent_found = agent_query
            .iter()
            .find(|(entity, _, _)| entity.agent_id == check_cmd.agent_id);

        if let Some((_, permissions, inheritance)) = agent_found {
            let mut has_permission = permissions.has(&check_cmd.permission);

            // Check inherited permissions if not found directly
            if !has_permission {
                if let Some(inherit) = inheritance {
                    // In a real implementation, we would check parent roles
                    // For now, we'll just check if inheritance is enabled
                    has_permission = inherit.allow_override
                        && !permissions.denied.contains(&check_cmd.permission);
                }
            }

            if has_permission {
                allowed_events.write(PermissionAllowedEvent {
                    agent_id: check_cmd.agent_id,
                    permission: check_cmd.permission.clone(),
                    context: check_cmd.context.clone(),
                    timestamp: chrono::Utc::now(),
                });
            } else {
                denied_events.write(PermissionDeniedEvent {
                    agent_id: check_cmd.agent_id,
                    permission: check_cmd.permission.clone(),
                    reason: "Permission not granted or explicitly denied".to_string(),
                    context: check_cmd.context.clone(),
                    timestamp: chrono::Utc::now(),
                });
            }
        }
    }
}

/// System for managing permission roles
///
/// ```mermaid
/// graph LR
///     A[AssignRoleCommand] --> B[manage_roles_system]
///     B --> C{Agent Found?}
///     C -->|Yes| D[Update Roles]
///     C -->|No| E[Log Error]
///     D --> F[RoleAssignedEvent]
/// ```
pub fn manage_roles_system(
    mut role_commands: EventReader<AssignRoleCommand>,
    mut agent_query: Query<(
        &AgentEntity,
        &mut AgentPermissions,
        Option<&mut PermissionAudit>,
    )>,
    mut role_events: EventWriter<RoleAssignedEvent>,
) {
    for role_cmd in role_commands.read() {
        // Find the agent
        let agent_found = agent_query
            .iter_mut()
            .find(|(entity, _, _)| entity.agent_id == role_cmd.agent_id);

        if let Some((_, mut permissions, audit)) = agent_found {
            // Add role
            permissions.add_role(role_cmd.role.clone());

            // Add audit entry
            if let Some(mut audit_log) = audit.as_mut() {
                audit_log.add_change(PermissionChange {
                    change_type: PermissionChangeType::RoleAdded,
                    permission: format!("role:{}", role_cmd.role),
                    changed_by: role_cmd.assigned_by,
                    timestamp: chrono::Utc::now(),
                    reason: role_cmd.reason.clone(),
                });
            }

            // Emit event
            role_events.write(RoleAssignedEvent {
                agent_id: role_cmd.agent_id,
                role: role_cmd.role.clone(),
                assigned_by: role_cmd.assigned_by,
                timestamp: chrono::Utc::now(),
            });
        }
    }
}

// Command and event types
#[derive(Event)]
pub struct GrantPermissionsCommand {
    pub agent_id: Uuid,
    pub permissions: Vec<String>,
    pub granted_by: Uuid,
    pub reason: Option<String>,
}

#[derive(Event)]
pub struct RevokePermissionsCommand {
    pub agent_id: Uuid,
    pub permissions: Vec<String>,
    pub revoked_by: Uuid,
    pub reason: Option<String>,
}

#[derive(Event)]
pub struct CheckPermissionCommand {
    pub agent_id: Uuid,
    pub permission: String,
    pub context: Option<String>,
}

#[derive(Event)]
pub struct AssignRoleCommand {
    pub agent_id: Uuid,
    pub role: String,
    pub assigned_by: Uuid,
    pub reason: Option<String>,
}

#[derive(Event)]
pub struct PermissionAllowedEvent {
    pub agent_id: Uuid,
    pub permission: String,
    pub context: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Event)]
pub struct PermissionDeniedEvent {
    pub agent_id: Uuid,
    pub permission: String,
    pub reason: String,
    pub context: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Event)]
pub struct RoleAssignedEvent {
    pub agent_id: Uuid,
    pub role: String,
    pub assigned_by: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
