//! AI Provider implementations

pub mod ollama;

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

use crate::types::*;
use crate::error::BridgeError;

/// Provider trait - implemented by each AI provider
#[async_trait]
pub trait Provider: Send + Sync {
    /// Get the name of this provider
    fn name(&self) -> String;
    
    /// Get provider description
    fn description(&self) -> String;
    
    /// Check if provider is configured
    fn is_configured(&self) -> bool;
    
    /// Execute a query
    async fn query(
        &self,
        model: String,
        messages: Vec<Message>,
        parameters: ModelParameters,
    ) -> Result<ModelResponse, BridgeError>;
    
    /// Execute a streaming query
    async fn stream_query(
        &self,
        model: String,
        messages: Vec<Message>,
        parameters: ModelParameters,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, BridgeError>> + Send>>, BridgeError>;
    
    /// List available models
    async fn list_models(&self) -> Result<Vec<ModelInfo>, BridgeError>;
    
    /// Check provider health
    async fn health_check(&self) -> Result<HealthStatus, BridgeError>;
    
    /// Get provider capabilities
    fn capabilities(&self) -> Vec<String> {
        vec![
            "query".to_string(),
            "stream".to_string(),
            "models".to_string(),
        ]
    }
} 