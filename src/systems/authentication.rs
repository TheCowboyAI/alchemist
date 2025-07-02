//! Authentication systems

use bevy::prelude::*;
use bevy_app::prelude::*;
use crate::components::*;
use uuid::Uuid;

/// System for authenticating agents
///
/// ```mermaid
/// graph LR
///     A[AuthenticateAgentCommand] --> B[authenticate_agent_system]
///     B --> C{Valid Credentials?}
///     C -->|Yes| D[Update Auth Status]
///     C -->|No| E[Increment Failures]
///     D --> F[AuthSuccessEvent]
///     E --> G[AuthFailureEvent]
/// ```
pub fn authenticate_agent_system(
    mut auth_commands: EventReader<AuthenticateAgentCommand>,
    mut agent_query: Query<(
        &AgentEntity,
        &mut AgentAuthentication,
        Option<&mut AuthenticationAudit>,
        Option<&AuthenticationPolicy>,
    )>,
    mut success_events: EventWriter<AuthenticationSuccessEvent>,
    mut failure_events: EventWriter<AuthenticationFailureEvent>,
) {
    for auth_cmd in auth_commands.read() {
        // Find the agent
        let agent_found = agent_query.iter_mut()
            .find(|(entity, _, _, _)| entity.agent_id == auth_cmd.agent_id);

        if let Some((_, mut auth, audit, policy)) = agent_found {
            // Check if account is locked
            if let Some(pol) = policy {
                if auth.failed_attempts >= pol.max_failed_attempts {
                    // Account is locked
                    failure_events.write(AuthenticationFailureEvent {
                        agent_id: auth_cmd.agent_id,
                        reason: "Account locked due to too many failed attempts".to_string(),
                        timestamp: chrono::Utc::now(),
                    });
                    continue;
                }
            }

            // Simulate authentication (in real implementation, verify credentials)
            let auth_success = auth_cmd.validate_credentials();

            if auth_success {
                // Successful authentication
                auth.status = AuthenticationStatus::Authenticated;
                auth.last_authenticated = Some(chrono::Utc::now());
                auth.failed_attempts = 0;

                // Add audit event
                if let Some(mut audit_log) = audit {
                    audit_log.add_event(AuthenticationEvent {
                        event_type: AuthEventType::LoginSuccess,
                        timestamp: chrono::Utc::now(),
                        ip_address: auth_cmd.ip_address.clone(),
                        user_agent: auth_cmd.user_agent.clone(),
                        context: Some("Successful authentication".to_string()),
                    });
                }

                success_events.write(AuthenticationSuccessEvent {
                    agent_id: auth_cmd.agent_id,
                    method: auth.method.clone(),
                    timestamp: chrono::Utc::now(),
                });
            } else {
                // Failed authentication
                auth.failed_attempts += 1;

                // Add audit event
                if let Some(mut audit_log) = audit {
                    audit_log.add_event(AuthenticationEvent {
                        event_type: AuthEventType::LoginFailed,
                        timestamp: chrono::Utc::now(),
                        ip_address: auth_cmd.ip_address.clone(),
                        user_agent: auth_cmd.user_agent.clone(),
                        context: Some(format!("Failed attempt #{}", auth.failed_attempts)),
                    });
                }

                failure_events.write(AuthenticationFailureEvent {
                    agent_id: auth_cmd.agent_id,
                    reason: "Invalid credentials".to_string(),
                    timestamp: chrono::Utc::now(),
                });
            }
        }
    }
}

/// System for token refresh
///
/// ```mermaid
/// graph LR
///     A[RefreshTokenCommand] --> B[refresh_token_system]
///     B --> C{Token Valid?}
///     C -->|Yes| D[Generate New Token]
///     C -->|No| E[Reject Refresh]
///     D --> F[TokenRefreshedEvent]
/// ```
pub fn refresh_token_system(
    mut refresh_commands: EventReader<RefreshTokenCommand>,
    mut agent_query: Query<(
        &AgentEntity,
        &mut AgentAuthentication,
        Option<&mut AuthenticationToken>,
        Option<&mut AuthenticationAudit>,
    )>,
    mut refreshed_events: EventWriter<TokenRefreshedEvent>,
) {
    for refresh_cmd in refresh_commands.read() {
        // Find the agent
        let agent_found = agent_query.iter_mut()
            .find(|(entity, _, _, _)| entity.agent_id == refresh_cmd.agent_id);

        if let Some((_, auth, token, audit)) = agent_found {
            // Check if authenticated
            if auth.status != AuthenticationStatus::Authenticated {
                continue;
            }

            // Update token
            if let Some(mut tok) = token {
                tok.expires_at = Some(chrono::Utc::now() + chrono::Duration::hours(24));
                
                // Add audit event
                if let Some(mut audit_log) = audit {
                    audit_log.add_event(AuthenticationEvent {
                        event_type: AuthEventType::TokenRefreshed,
                        timestamp: chrono::Utc::now(),
                        ip_address: None,
                        user_agent: None,
                        context: Some("Token refreshed".to_string()),
                    });
                }

                refreshed_events.write(TokenRefreshedEvent {
                    agent_id: refresh_cmd.agent_id,
                    new_expiry: tok.expires_at.unwrap(),
                    timestamp: chrono::Utc::now(),
                });
            }
        }
    }
}

/// System for checking token expiration
///
/// ```mermaid
/// graph LR
///     A[Timer] --> B[check_token_expiration_system]
///     B --> C[For Each Agent]
///     C --> D{Token Expired?}
///     D -->|Yes| E[Update Auth Status]
///     E --> F[TokenExpiredEvent]
/// ```
pub fn check_token_expiration_system(
    mut agent_query: Query<(
        &AgentEntity,
        &mut AgentAuthentication,
        Option<&AuthenticationToken>,
    )>,
    mut expired_events: EventWriter<TokenExpiredEvent>,
) {
    let now = chrono::Utc::now();

    for (entity, mut auth, token) in agent_query.iter_mut() {
        if let Some(tok) = token {
            if let Some(expires_at) = tok.expires_at {
                if expires_at < now && auth.status == AuthenticationStatus::Authenticated {
                    // Token has expired
                    auth.status = AuthenticationStatus::Expired;

                    expired_events.write(TokenExpiredEvent {
                        agent_id: entity.agent_id,
                        expired_at: expires_at,
                        timestamp: now,
                    });
                }
            }
        }
    }
}

// Command and event types
#[derive(Event)]
pub struct AuthenticateAgentCommand {
    pub agent_id: Uuid,
    pub method: AuthenticationMethod,
    pub credentials: Vec<u8>, // Encrypted credentials
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl AuthenticateAgentCommand {
    // Placeholder for credential validation
    fn validate_credentials(&self) -> bool {
        // In real implementation, this would verify credentials
        true
    }
}

#[derive(Event)]
pub struct RefreshTokenCommand {
    pub agent_id: Uuid,
    pub refresh_token: String,
}

#[derive(Event)]
pub struct AuthenticationSuccessEvent {
    pub agent_id: Uuid,
    pub method: AuthenticationMethod,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Event)]
pub struct AuthenticationFailureEvent {
    pub agent_id: Uuid,
    pub reason: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Event)]
pub struct TokenRefreshedEvent {
    pub agent_id: Uuid,
    pub new_expiry: chrono::DateTime<chrono::Utc>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Event)]
pub struct TokenExpiredEvent {
    pub agent_id: Uuid,
    pub expired_at: chrono::DateTime<chrono::Utc>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
} 