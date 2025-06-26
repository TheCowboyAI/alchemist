//! Authentication and authorization abstractions

use crate::{claims::ClaimsIdentity, error::Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Authentication context
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthContext {
    /// The authenticated identity
    pub identity: ClaimsIdentity,
    /// The method used for authentication
    pub authentication_method: String,
    /// When authentication occurred
    pub authenticated_at: DateTime<Utc>,
    /// Additional properties
    pub properties: HashMap<String, String>,
    /// Session identifier
    pub session_id: Option<String>,
}

impl AuthContext {
    /// Create a new authentication context
    pub fn new(identity: ClaimsIdentity, method: impl Into<String>) -> Self {
        Self {
            identity,
            authentication_method: method.into(),
            authenticated_at: Utc::now(),
            properties: HashMap::new(),
            session_id: None,
        }
    }

    /// Add a property to the context
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// Set the session ID
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Check if the context is still valid
    pub fn is_valid(&self) -> bool {
        self.identity.is_valid()
    }
}

/// Trait for authentication
#[async_trait]
pub trait Authenticator: Send + Sync {
    /// Authenticate with credentials
    async fn authenticate(&self, credentials: &dyn Credentials) -> Result<AuthContext>;

    /// Validate an existing auth context
    async fn validate(&self, context: &AuthContext) -> Result<bool>;

    /// Refresh authentication
    async fn refresh(&self, context: &AuthContext) -> Result<AuthContext>;

    /// Revoke authentication
    async fn revoke(&self, context: &AuthContext) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }

    /// Get supported authentication methods
    fn supported_methods(&self) -> Vec<String>;
}

/// Trait for authorization decisions
#[async_trait]
pub trait Authorizer: Send + Sync {
    /// Check if a principal is authorized for an action
    async fn authorize(
        &self,
        context: &AuthContext,
        resource: &str,
        action: &str,
    ) -> Result<AuthDecision>;

    /// Check multiple permissions at once
    async fn authorize_many(
        &self,
        context: &AuthContext,
        permissions: &[(String, String)], // (resource, action) pairs
    ) -> Result<Vec<AuthDecision>> {
        let mut decisions = Vec::new();
        for (resource, action) in permissions {
            decisions.push(self.authorize(context, resource, action).await?);
        }
        Ok(decisions)
    }
}

/// Authorization decision
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthDecision {
    /// Whether the action is allowed
    pub allowed: bool,
    /// Reason for the decision
    pub reason: Option<String>,
    /// Any constraints that apply
    pub constraints: Vec<String>,
    /// Time when the decision was made
    pub decided_at: DateTime<Utc>,
}

impl AuthDecision {
    /// Create an allowed decision
    pub fn allow() -> Self {
        Self {
            allowed: true,
            reason: None,
            constraints: Vec::new(),
            decided_at: Utc::now(),
        }
    }

    /// Create a denied decision
    pub fn deny(reason: impl Into<String>) -> Self {
        Self {
            allowed: false,
            reason: Some(reason.into()),
            constraints: Vec::new(),
            decided_at: Utc::now(),
        }
    }

    /// Add a constraint to the decision
    pub fn with_constraint(mut self, constraint: impl Into<String>) -> Self {
        self.constraints.push(constraint.into());
        self
    }
}

/// Trait for credentials
pub trait Credentials: Send + Sync {
    /// Get credential type
    fn credential_type(&self) -> &str;

    /// Convert to bytes for processing
    fn as_bytes(&self) -> Vec<u8>;
}

/// Username/password credentials
#[derive(Clone, Debug)]
pub struct PasswordCredentials {
    pub username: String,
    pub password: String,
}

impl Credentials for PasswordCredentials {
    fn credential_type(&self) -> &str {
        "password"
    }

    fn as_bytes(&self) -> Vec<u8> {
        format!("{}:{}", self.username, self.password).into_bytes()
    }
}

/// Token-based credentials
#[derive(Clone, Debug)]
pub struct TokenCredentials {
    pub token: String,
    pub token_type: String,
}

impl TokenCredentials {
    /// Create bearer token credentials
    pub fn bearer(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            token_type: "Bearer".to_string(),
        }
    }
}

impl Credentials for TokenCredentials {
    fn credential_type(&self) -> &str {
        "token"
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.token.as_bytes().to_vec()
    }
}

/// Certificate-based credentials
#[derive(Clone, Debug)]
pub struct CertificateCredentials {
    pub certificate: Vec<u8>,
    pub private_key: Option<Vec<u8>>,
}

impl Credentials for CertificateCredentials {
    fn credential_type(&self) -> &str {
        "certificate"
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.certificate.clone()
    }
}

/// Multi-factor authentication support
#[async_trait]
pub trait MultiFactorAuthenticator: Send + Sync {
    /// Start MFA challenge
    async fn start_challenge(&self, context: &AuthContext) -> Result<MfaChallenge>;

    /// Verify MFA response
    async fn verify_response(&self, challenge: &MfaChallenge, response: &str) -> Result<bool>;
}

/// MFA challenge
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MfaChallenge {
    /// Challenge ID
    pub id: String,
    /// Challenge type (totp, sms, email, etc.)
    pub challenge_type: String,
    /// When the challenge was issued
    pub issued_at: DateTime<Utc>,
    /// When the challenge expires
    pub expires_at: DateTime<Utc>,
    /// Additional challenge data
    pub data: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claims::{Claim, claim_types};
    use serde_json::json;

    #[test]
    fn test_auth_context() {
        let mut identity = ClaimsIdentity::new();
        identity.add_claim(Claim::new(
            "test-issuer",
            "user123",
            claim_types::NAME,
            json!("John Doe"),
        ));

        let context = AuthContext::new(identity, "password")
            .with_property("ip_address", "192.168.1.1")
            .with_session_id("session123");

        assert_eq!(context.authentication_method, "password");
        assert_eq!(
            context.properties.get("ip_address"),
            Some(&"192.168.1.1".to_string())
        );
        assert_eq!(context.session_id, Some("session123".to_string()));
        assert!(context.is_valid());
    }

    #[test]
    fn test_auth_decision() {
        let allow = AuthDecision::allow().with_constraint("read-only");

        assert!(allow.allowed);
        assert_eq!(allow.constraints, vec!["read-only"]);

        let deny = AuthDecision::deny("insufficient permissions");

        assert!(!deny.allowed);
        assert_eq!(deny.reason, Some("insufficient permissions".to_string()));
    }

    #[test]
    fn test_password_credentials() {
        let creds = PasswordCredentials {
            username: "alice".to_string(),
            password: "secret123".to_string(),
        };

        assert_eq!(creds.credential_type(), "password");
        assert_eq!(creds.as_bytes(), b"alice:secret123");
    }

    #[test]
    fn test_token_credentials() {
        let creds = TokenCredentials::bearer("my-token");

        assert_eq!(creds.credential_type(), "token");
        assert_eq!(creds.token_type, "Bearer");
        assert_eq!(creds.as_bytes(), b"my-token");
    }
}
