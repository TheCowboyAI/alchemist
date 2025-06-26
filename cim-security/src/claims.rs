//! Claims and identity abstractions

use crate::{
    error::{Result, SecurityError},
    types::*,
};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A security claim
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Claim {
    /// The issuer of the claim
    pub issuer: String,
    /// The subject of the claim
    pub subject: String,
    /// The type of claim
    pub claim_type: String,
    /// The claim value
    pub value: serde_json::Value,
    /// When the claim was issued
    pub issued_at: DateTime<Utc>,
    /// When the claim expires (if applicable)
    pub expires_at: Option<DateTime<Utc>>,
}

impl Claim {
    /// Create a new claim
    pub fn new(
        issuer: impl Into<String>,
        subject: impl Into<String>,
        claim_type: impl Into<String>,
        value: serde_json::Value,
    ) -> Self {
        Self {
            issuer: issuer.into(),
            subject: subject.into(),
            claim_type: claim_type.into(),
            value,
            issued_at: Utc::now(),
            expires_at: None,
        }
    }

    /// Set expiration time
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Set expiration duration from now
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.expires_at = Some(Utc::now() + duration);
        self
    }

    /// Check if the claim is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check if the claim is valid (not expired)
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}

/// A collection of claims forming an identity
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ClaimsIdentity {
    /// The claims in this identity
    pub claims: Vec<Claim>,
    /// The type of authentication used
    pub authentication_type: Option<String>,
    /// The claim type used for the name
    pub name_claim_type: String,
    /// The claim type used for roles
    pub role_claim_type: String,
}

impl ClaimsIdentity {
    /// Create a new claims identity
    pub fn new() -> Self {
        Self {
            claims: Vec::new(),
            authentication_type: None,
            name_claim_type: "name".to_string(),
            role_claim_type: "role".to_string(),
        }
    }

    /// Add a claim to the identity
    pub fn add_claim(&mut self, claim: Claim) {
        self.claims.push(claim);
    }

    /// Get all claims of a specific type
    pub fn get_claims(&self, claim_type: &str) -> Vec<&Claim> {
        self.claims
            .iter()
            .filter(|c| c.claim_type == claim_type)
            .collect()
    }

    /// Get the first claim of a specific type
    pub fn get_claim(&self, claim_type: &str) -> Option<&Claim> {
        self.claims.iter().find(|c| c.claim_type == claim_type)
    }

    /// Get the name claim
    pub fn name(&self) -> Option<String> {
        self.get_claim(&self.name_claim_type)
            .and_then(|c| c.value.as_str())
            .map(|s| s.to_string())
    }

    /// Get all role claims
    pub fn roles(&self) -> Vec<String> {
        self.get_claims(&self.role_claim_type)
            .iter()
            .filter_map(|c| c.value.as_str())
            .map(|s| s.to_string())
            .collect()
    }

    /// Check if the identity has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles().contains(&role.to_string())
    }

    /// Check if all claims are valid (not expired)
    pub fn is_valid(&self) -> bool {
        self.claims.iter().all(|c| c.is_valid())
    }

    /// Remove expired claims
    pub fn remove_expired(&mut self) {
        self.claims.retain(|c| c.is_valid());
    }
}

/// Trait for claim validation
#[async_trait]
pub trait ClaimValidator: Send + Sync {
    /// Validate a claim
    async fn validate(&self, claim: &Claim) -> Result<bool>;

    /// Validate a full identity
    async fn validate_identity(&self, identity: &ClaimsIdentity) -> Result<bool> {
        // Default implementation validates all claims
        for claim in &identity.claims {
            if !self.validate(claim).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// Trait for claim issuance
#[async_trait]
pub trait ClaimIssuer: Send + Sync {
    /// Issue a new claim
    async fn issue_claim(
        &self,
        subject: &str,
        claim_type: &str,
        value: serde_json::Value,
        duration: Option<Duration>,
    ) -> Result<Claim>;

    /// Sign a claim
    async fn sign_claim(&self, claim: &Claim) -> Result<SignedClaim>;

    /// Get the issuer identifier
    fn issuer(&self) -> &str;
}

/// A signed claim with proof
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedClaim {
    /// The claim
    pub claim: Claim,
    /// The signature
    pub signature: Vec<u8>,
    /// The algorithm used
    pub algorithm: String,
}

/// Trait for signed claim verification
#[async_trait]
pub trait SignedClaimVerifier: Send + Sync {
    /// Verify a signed claim
    async fn verify(&self, signed_claim: &SignedClaim) -> Result<bool>;
}

/// Standard claim types
pub mod claim_types {
    /// Subject identifier
    pub const SUBJECT: &str = "sub";
    /// Name
    pub const NAME: &str = "name";
    /// Email address
    pub const EMAIL: &str = "email";
    /// Role
    pub const ROLE: &str = "role";
    /// Group membership
    pub const GROUP: &str = "group";
    /// Issued at time
    pub const ISSUED_AT: &str = "iat";
    /// Expiration time
    pub const EXPIRATION: &str = "exp";
    /// Not before time
    pub const NOT_BEFORE: &str = "nbf";
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_claim_creation() {
        let claim = Claim::new(
            "test-issuer",
            "test-subject",
            claim_types::NAME,
            json!("Test User"),
        );

        assert_eq!(claim.issuer, "test-issuer");
        assert_eq!(claim.subject, "test-subject");
        assert_eq!(claim.claim_type, claim_types::NAME);
        assert_eq!(claim.value, json!("Test User"));
        assert!(claim.is_valid());
    }

    #[test]
    fn test_claim_expiration() {
        let expired_claim = Claim::new(
            "test-issuer",
            "test-subject",
            claim_types::NAME,
            json!("Test User"),
        )
        .with_expiration(Utc::now() - Duration::hours(1));

        assert!(expired_claim.is_expired());
        assert!(!expired_claim.is_valid());

        let valid_claim = Claim::new(
            "test-issuer",
            "test-subject",
            claim_types::NAME,
            json!("Test User"),
        )
        .with_duration(Duration::hours(1));

        assert!(!valid_claim.is_expired());
        assert!(valid_claim.is_valid());
    }

    #[test]
    fn test_claims_identity() {
        let mut identity = ClaimsIdentity::new();

        identity.add_claim(Claim::new(
            "test-issuer",
            "user123",
            claim_types::NAME,
            json!("John Doe"),
        ));

        identity.add_claim(Claim::new(
            "test-issuer",
            "user123",
            claim_types::ROLE,
            json!("admin"),
        ));

        identity.add_claim(Claim::new(
            "test-issuer",
            "user123",
            claim_types::ROLE,
            json!("user"),
        ));

        assert_eq!(identity.name(), Some("John Doe".to_string()));
        assert_eq!(identity.roles(), vec!["admin", "user"]);
        assert!(identity.has_role("admin"));
        assert!(identity.has_role("user"));
        assert!(!identity.has_role("guest"));
        assert!(identity.is_valid());
    }

    #[test]
    fn test_expired_claims_removal() {
        let mut identity = ClaimsIdentity::new();

        // Add valid claim
        identity.add_claim(
            Claim::new(
                "test-issuer",
                "user123",
                claim_types::NAME,
                json!("John Doe"),
            )
            .with_duration(Duration::hours(1)),
        );

        // Add expired claim
        identity.add_claim(
            Claim::new(
                "test-issuer",
                "user123",
                claim_types::ROLE,
                json!("temp-admin"),
            )
            .with_expiration(Utc::now() - Duration::hours(1)),
        );

        assert_eq!(identity.claims.len(), 2);

        identity.remove_expired();

        assert_eq!(identity.claims.len(), 1);
        assert_eq!(identity.claims[0].claim_type, claim_types::NAME);
    }
}
