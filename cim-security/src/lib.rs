//! # CIM Security
//!
//! Core security abstractions for the Composable Information Machine (CIM).
//!
//! This crate provides foundational security interfaces for:
//! - Cryptographic operations
//! - Claims-based authentication
//! - Secrets management
//! - Security policies
//! - Audit trails
//!
//! ## Design Principles
//!
//! 1. **Abstract Over Concrete**: All functionality is exposed through traits
//! 2. **Async by Default**: All operations are async for scalability
//! 3. **No Implementation Details**: This crate contains only abstractions
//! 4. **Composable**: Security features can be mixed and matched
//!
//! ## Example
//!
//! ```no_run
//! use cim_security::{
//!     auth::{Authenticator, PasswordCredentials},
//!     claims::ClaimsIdentity,
//!     policy::{SecurityPolicy, PolicyEvaluator},
//! };
//!
//! async fn authenticate_user(
//!     authenticator: &impl Authenticator,
//!     username: &str,
//!     password: &str,
//! ) -> Result<ClaimsIdentity, Box<dyn std::error::Error>> {
//!     let credentials = PasswordCredentials {
//!         username: username.to_string(),
//!         password: password.to_string(),
//!     };
//!     
//!     let auth_context = authenticator.authenticate(&credentials).await?;
//!     Ok(auth_context.identity)
//! }
//! ```

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

/// Error types
pub mod error;

/// Common types used throughout the module
pub mod types;

/// Cryptographic operation traits
pub mod crypto;

/// Claims-based security
pub mod claims;

/// Secrets management
pub mod secrets;

/// Authentication and authorization
pub mod auth;

/// Security policy abstractions
pub mod policy;

/// Audit trail abstractions
pub mod audit;

/// Security context management
pub mod context;

// Re-export commonly used types
pub use error::{Result, SecurityError};
pub use types::{
    Algorithm, DerivedKey, EncryptedData, EncryptionMetadata, PrivateKey, PublicKey, Signature,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        auth::{
            AuthContext, AuthDecision, Authenticator, Authorizer, Credentials, PasswordCredentials,
            TokenCredentials,
        },
        claims::{
            Claim, ClaimIssuer, ClaimValidator, ClaimsIdentity, SignedClaim, SignedClaimVerifier,
            claim_types,
        },
        context::{SecurityContext, SecurityContextBuilder},
        crypto::{Decryptor, Encryptor, KeyDerivation, KeyGenerator, Signer, Verifier},
        error::{Result, SecurityError},
        policy::{
            DefaultPolicyEvaluator, PolicyCondition, PolicyDecision, PolicyEffect, PolicyEvaluator,
            PolicyRule, SecurityPolicy,
        },
        secrets::{RotationPolicy, SecretMetadata, SecretRef, SecretRotation, SecretStore},
        types::{Algorithm, PublicKey, Signature},
    };
}
