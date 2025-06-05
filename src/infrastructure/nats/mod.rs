//! NATS integration for CIM event-driven architecture

use async_nats::Client;
use std::time::Duration;
use thiserror::Error;
use tracing::info;

pub mod client;
pub mod config;

#[cfg(test)]
mod tests;

pub use client::NatsClient;
pub use config::NatsConfig;

#[derive(Error, Debug)]
pub enum NatsError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("JetStream error: {0}")]
    JetStreamError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Subscription error: {0}")]
    SubscriptionError(String),

    #[error("Timeout error: operation timed out after {0:?}")]
    TimeoutError(Duration),
}

// Generic implementation for all async_nats errors
impl<T> From<async_nats::error::Error<T>> for NatsError
where
    T: Clone + std::fmt::Display + std::fmt::Debug + PartialEq,
{
    fn from(err: async_nats::error::Error<T>) -> Self {
        NatsError::ConnectionError(format!("{}", err))
    }
}

impl From<async_nats::jetstream::Error> for NatsError {
    fn from(err: async_nats::jetstream::Error) -> Self {
        NatsError::JetStreamError(err.to_string())
    }
}

impl From<serde_json::Error> for NatsError {
    fn from(err: serde_json::Error) -> Self {
        NatsError::SerializationError(err.to_string())
    }
}

impl From<async_nats::SubscribeError> for NatsError {
    fn from(err: async_nats::SubscribeError) -> Self {
        NatsError::SubscriptionError(err.to_string())
    }
}

/// Initialize NATS connection with default configuration for localhost
pub async fn connect_localhost() -> Result<Client, NatsError> {
    let config = NatsConfig::localhost();
    connect_with_config(&config).await
}

/// Connect to NATS with custom configuration
pub async fn connect_with_config(config: &NatsConfig) -> Result<Client, NatsError> {
    info!("Connecting to NATS at {}", config.url);

    let mut options = async_nats::ConnectOptions::new().name(&config.client_name);

    // Set max reconnects if specified
    if let Some(max_reconnects) = config.max_reconnects {
        options = options.max_reconnects(max_reconnects);
    }

    // Set connection timeout
    options = options.connection_timeout(config.connection_timeout);

    let client = options.connect(&config.url).await?;

    info!("Successfully connected to NATS");
    Ok(client)
}
