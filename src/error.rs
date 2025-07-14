//! Error types for Alchemist

use thiserror::Error;

/// Main error type for Alchemist
#[derive(Error, Debug)]
pub enum AlchemistError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// NATS error
    #[error("NATS error: {0}")]
    Nats(#[from] async_nats::Error),

    /// NATS connection error
    #[error("NATS connection error: {0}")]
    NatsConnect(#[from] async_nats::ConnectError),

    /// NATS publish error
    #[error("NATS publish error: {0}")]
    NatsPublish(#[from] async_nats::PublishError),

    /// NATS request error
    #[error("NATS request error: {0}")]
    NatsRequest(#[from] async_nats::RequestError),
    
    /// NATS subscribe error
    #[error("NATS subscribe error: {0}")]
    NatsSubscribe(#[from] async_nats::SubscribeError),

    /// HTTP error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Deployment error
    #[error("Deployment failed: {0}")]
    DeploymentFailed(String),

    /// Deployment not found
    #[error("Deployment not found: {0}")]
    DeploymentNotFound(String),

    /// Service not found
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    /// Agent not found
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    /// Health check failed
    #[error("Health check failed for service: {0}")]
    HealthCheckFailed(String),

    /// Agent not responding
    #[error("Agent not responding: {0}")]
    AgentNotResponding(String),

    /// Validation failed
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Command execution error
    #[error("Command execution error: {0}")]
    CommandExecution(String),

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl From<String> for AlchemistError {
    fn from(err: String) -> Self {
        AlchemistError::Other(err)
    }
}

impl From<&str> for AlchemistError {
    fn from(err: &str) -> Self {
        AlchemistError::Other(err.to_string())
    }
}

/// Result type alias for Alchemist operations
pub type Result<T> = std::result::Result<T, AlchemistError>;