//! NATS client implementation

use async_nats::{Client, Subscriber};
use serde::{Deserialize, Serialize};

use super::{NatsConfig, Result};
use super::error::NatsError;

/// NATS client wrapper
#[derive(Clone)]
pub struct NatsClient {
    /// The underlying NATS client
    client: Client,

    /// Configuration
    config: NatsConfig,
}

impl NatsClient {
    /// Create a new NATS client
    pub async fn new(config: NatsConfig) -> Result<Self> {
        let client = async_nats::connect(&config.url)
            .await
            .map_err(|e| NatsError::Connection(e.to_string()))?;

        Ok(Self { client, config })
    }

    /// Publish a message
    pub async fn publish<T>(&self, subject: &str, message: &T) -> Result<()>
    where
        T: Serialize,
    {
        let payload = serde_json::to_vec(message)
            .map_err(|e| NatsError::Serialization(e.to_string()))?;

        self.client
            .publish(subject.to_string(), payload.into())
            .await
            .map_err(|e| NatsError::Publish(e.to_string()))?;

        Ok(())
    }

    /// Subscribe to a subject
    pub async fn subscribe(&self, subject: &str) -> Result<Subscriber> {
        self.client
            .subscribe(subject.to_string())
            .await
            .map_err(|e| NatsError::Subscribe(e.to_string()))
    }

    /// Request-reply pattern
    pub async fn request<T, R>(&self, subject: &str, message: &T) -> Result<R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let payload = serde_json::to_vec(message)
            .map_err(|e| NatsError::Serialization(e.to_string()))?;

        let response = self.client
            .request(subject.to_string(), payload.into())
            .await
            .map_err(|e| NatsError::Request(e.to_string()))?;

        serde_json::from_slice(&response.payload)
            .map_err(|e| NatsError::Deserialization(e.to_string()))
    }
}
