//! Error types for subject operations

use thiserror::Error;

/// Result type alias for subject operations
pub type Result<T> = std::result::Result<T, SubjectError>;

/// Errors that can occur during subject operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SubjectError {
    /// Invalid subject format
    #[error("Invalid subject format: {0}")]
    InvalidFormat(String),

    /// Invalid pattern
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Translation error
    #[error("Translation error: {0}")]
    TranslationError(String),

    /// Composition error
    #[error("Composition error: {0}")]
    CompositionError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Not found
    #[error("Not found: {0}")]
    NotFound(String),
}

impl SubjectError {
    /// Create an invalid format error
    pub fn invalid_format(msg: impl Into<String>) -> Self {
        Self::InvalidFormat(msg.into())
    }

    /// Create an invalid pattern error
    pub fn invalid_pattern(msg: impl Into<String>) -> Self {
        Self::InvalidPattern(msg.into())
    }

    /// Create a parse error
    pub fn parse_error(msg: impl Into<String>) -> Self {
        Self::ParseError(msg.into())
    }

    /// Create a permission denied error
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        Self::PermissionDenied(msg.into())
    }

    /// Create a translation error
    pub fn translation_error(msg: impl Into<String>) -> Self {
        Self::TranslationError(msg.into())
    }

    /// Create a composition error
    pub fn composition_error(msg: impl Into<String>) -> Self {
        Self::CompositionError(msg.into())
    }

    /// Create a validation error
    pub fn validation_error(msg: impl Into<String>) -> Self {
        Self::ValidationError(msg.into())
    }

    /// Create a not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
}
