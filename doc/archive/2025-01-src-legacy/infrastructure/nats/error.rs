//! NATS error types

use thiserror::Error;

/// NATS-related errors
#[derive(Debug, Error)]
pub enum NatsError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Publish error: {0}")]
    Publish(String),

    #[error("Subscribe error: {0}")]
    Subscribe(String),

    #[error("Request error: {0}")]
    Request(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),
}

/// Result type for NATS operations
pub type Result<T> = std::result::Result<T, NatsError>;
