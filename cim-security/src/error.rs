//! Error types for the security module

use thiserror::Error;

/// Result type alias for security operations
pub type Result<T> = std::result::Result<T, SecurityError>;

/// Security-related errors
#[derive(Debug, Error)]
pub enum SecurityError {
    /// Cryptographic operation failed
    #[error("Cryptographic operation failed: {0}")]
    CryptoError(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    /// Authorization denied
    #[error("Authorization denied: {0}")]
    AuthorizationError(String),

    /// Invalid claim
    #[error("Invalid claim: {0}")]
    InvalidClaim(String),

    /// Claim expired
    #[error("Claim expired: {claim_type} for subject {subject}")]
    ClaimExpired { claim_type: String, subject: String },

    /// Secret not found
    #[error("Secret not found: {0}")]
    SecretNotFound(String),

    /// Invalid secret reference
    #[error("Invalid secret reference: {0}")]
    InvalidSecretRef(String),

    /// Policy violation
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    /// Invalid credentials
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    /// Signature verification failed
    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    /// Decryption failed
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    /// Key derivation failed
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    /// Operation not supported
    #[error("Operation not supported: {0}")]
    NotSupported(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl SecurityError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            SecurityError::Internal(_) | SecurityError::CryptoError(_)
        )
    }

    /// Get an error code for the error type
    pub fn error_code(&self) -> &'static str {
        match self {
            SecurityError::CryptoError(_) => "SEC_CRYPTO",
            SecurityError::AuthenticationError(_) => "SEC_AUTH",
            SecurityError::AuthorizationError(_) => "SEC_AUTHZ",
            SecurityError::InvalidClaim(_) => "SEC_CLAIM_INVALID",
            SecurityError::ClaimExpired { .. } => "SEC_CLAIM_EXPIRED",
            SecurityError::SecretNotFound(_) => "SEC_SECRET_NOT_FOUND",
            SecurityError::InvalidSecretRef(_) => "SEC_SECRET_INVALID_REF",
            SecurityError::PolicyViolation(_) => "SEC_POLICY_VIOLATION",
            SecurityError::InvalidCredentials(_) => "SEC_INVALID_CREDS",
            SecurityError::SignatureVerificationFailed => "SEC_SIG_VERIFY_FAIL",
            SecurityError::DecryptionFailed(_) => "SEC_DECRYPT_FAIL",
            SecurityError::KeyDerivationFailed(_) => "SEC_KEY_DERIVE_FAIL",
            SecurityError::NotSupported(_) => "SEC_NOT_SUPPORTED",
            SecurityError::Internal(_) => "SEC_INTERNAL",
        }
    }
}
