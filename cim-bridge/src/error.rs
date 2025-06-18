//! Error types for CIM Bridge

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("NATS connection error: {0}")]
    NatsConnect(String),
    
    #[error("NATS subscribe error: {0}")]
    NatsSubscribe(String),
    
    #[error("NATS publish error: {0}")]
    NatsPublish(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Provider error: {0}")]
    Provider(String),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("No active provider")]
    NoActiveProvider,
    
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Timeout waiting for response")]
    Timeout,
    
    #[error("Stream error: {0}")]
    Stream(String),
    
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

impl From<async_nats::SubscribeError> for BridgeError {
    fn from(err: async_nats::SubscribeError) -> Self {
        BridgeError::NatsSubscribe(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, BridgeError>; 