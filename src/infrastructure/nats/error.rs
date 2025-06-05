//! NATS error types

use thiserror::Error;

/// NATS-related errors
#[derive(Error, Debug)]
pub enum NatsError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Subscription error: {0}")]
    Subscription(String),

    #[error("Publish error: {0}")]
    Publish(String),

    #[error("Request error: {0}")]
    Request(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("JetStream error: {0}")]
    JetStream(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("NATS client error: {0}")]
    Client(#[from] async_nats::Error),
}
