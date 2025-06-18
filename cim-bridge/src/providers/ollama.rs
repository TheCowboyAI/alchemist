//! Ollama provider implementation

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;

use crate::error::BridgeError;
use crate::providers::Provider;
use crate::types::*;

/// Ollama provider configuration
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub host: String,
    pub port: u16,
    pub timeout: Duration,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 11434,
            timeout: Duration::from_secs(300), // 5 minutes for large models
        }
    }
}

/// Ollama provider
pub struct OllamaProvider {
    client: Client,
    config: OllamaConfig,
}

impl OllamaProvider {
    pub fn new(config: OllamaConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client, config }
    }
    
    fn base_url(&self) -> String {
        format!("http://{}:{}", self.config.host, self.config.port)
    }
}

// Ollama API types
#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: Option<OllamaChatMessage>,
    done: bool,
    #[serde(default)]
    total_duration: Option<u64>,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    #[serde(default)]
    parameter_size: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModel>,
}

#[async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> String {
        "ollama".to_string()
    }
    
    fn description(&self) -> String {
        "Local Ollama instance for running open-source models".to_string()
    }
    
    fn is_configured(&self) -> bool {
        true // Ollama doesn't require API keys
    }
    
    async fn query(
        &self,
        model: String,
        messages: Vec<Message>,
        parameters: ModelParameters,
    ) -> Result<ModelResponse, BridgeError> {
        let query_id = Uuid::new_v4();
        
        // Convert messages to Ollama format
        let ollama_messages: Vec<OllamaMessage> = messages
            .into_iter()
            .map(|msg| OllamaMessage {
                role: match msg.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::Function => "user", // Ollama doesn't have function role
                }.to_string(),
                content: msg.content,
            })
            .collect();
        
        // Build options
        let options = Some(OllamaOptions {
            temperature: parameters.temperature,
            top_p: parameters.top_p,
            seed: parameters.seed.map(|s| s as i64),
            num_predict: parameters.max_tokens.map(|t| t as i32),
        });
        
        // Make request
        let request = OllamaChatRequest {
            model: model.clone(),
            messages: ollama_messages,
            stream: false,
            options,
        };
        
        let url = format!("{}/api/chat", self.base_url());
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| BridgeError::Provider(format!("Failed to send request: {e}")))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(BridgeError::Provider(format!("Ollama API error: {error_text}")));
        }
        
        let ollama_response: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| BridgeError::Provider(format!("Failed to parse response: {e}")))?;
        
        // Log timing information if available
        if let Some(duration) = ollama_response.total_duration {
            tracing::debug!("Ollama query took {}ms", duration / 1_000_000);
        }
        
        let content = ollama_response.message
            .map(|m| {
                // Log the role for debugging
                tracing::trace!("Ollama response role: {}", m.role);
                m.content
            })
            .unwrap_or_default();
        
        let usage = if let (Some(prompt_tokens), Some(completion_tokens)) = 
            (ollama_response.prompt_eval_count, ollama_response.eval_count) {
            Some(Usage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            })
        } else {
            None
        };
        
        Ok(ModelResponse {
            query_id,
            model,
            content,
            usage,
            finish_reason: Some("stop".to_string()),
            metadata: Default::default(),
        })
    }
    
    async fn stream_query(
        &self,
        model: String,
        messages: Vec<Message>,
        parameters: ModelParameters,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, BridgeError>> + Send>>, BridgeError> {
        let _query_id = Uuid::new_v4();
        
        // Convert messages to Ollama format
        let ollama_messages: Vec<OllamaMessage> = messages
            .into_iter()
            .map(|msg| OllamaMessage {
                role: match msg.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::Function => "user",
                }.to_string(),
                content: msg.content,
            })
            .collect();
        
        // Build options
        let options = Some(OllamaOptions {
            temperature: parameters.temperature,
            top_p: parameters.top_p,
            seed: parameters.seed.map(|s| s as i64),
            num_predict: parameters.max_tokens.map(|t| t as i32),
        });
        
        // Make streaming request
        let request = OllamaChatRequest {
            model,
            messages: ollama_messages,
            stream: true,
            options,
        };
        
        let url = format!("{}/api/chat", self.base_url());
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| BridgeError::Provider(format!("Failed to send request: {e}")))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(BridgeError::Provider(format!("Ollama API error: {error_text}")));
        }
        
        // Create channel for streaming
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Spawn task to process stream
        tokio::spawn(async move {
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut total_prompt_tokens = 0u32;
            let mut total_completion_tokens = 0u32;
            
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        
                        // Process complete JSON objects from buffer
                        while let Some(line_end) = buffer.find('\n') {
                            let line = buffer[..line_end].trim().to_string();
                            buffer = buffer[line_end + 1..].to_string();
                            
                            if line.is_empty() {
                                continue;
                            }
                            
                            match serde_json::from_str::<OllamaChatResponse>(&line) {
                                Ok(response) => {
                                    if let Some(tokens) = response.prompt_eval_count {
                                        total_prompt_tokens = tokens;
                                    }
                                    if let Some(tokens) = response.eval_count {
                                        total_completion_tokens = tokens;
                                    }
                                    
                                    // Log timing for debugging
                                    if let Some(duration) = response.total_duration {
                                        tracing::trace!("Stream chunk took {}ms", duration / 1_000_000);
                                    }
                                    
                                    let content = response.message
                                        .map(|m| {
                                            // Log the role for debugging
                                            tracing::trace!("Ollama stream response role: {}", m.role);
                                            m.content
                                        })
                                        .unwrap_or_default();
                                    
                                    let chunk = StreamChunk {
                                        content,
                                        is_final: response.done,
                                        usage: if response.done {
                                            Some(Usage {
                                                prompt_tokens: total_prompt_tokens,
                                                completion_tokens: total_completion_tokens,
                                                total_tokens: total_prompt_tokens + total_completion_tokens,
                                            })
                                        } else {
                                            None
                                        },
                                    };
                                    
                                    if tx.send(Ok(chunk)).is_err() {
                                        break;
                                    }
                                    
                                    if response.done {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to parse Ollama response: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(BridgeError::Stream(format!("Stream error: {e}"))));
                        break;
                    }
                }
            }
        });
        
        let stream = UnboundedReceiverStream::new(rx);
        Ok(Box::pin(stream))
    }
    
    async fn list_models(&self) -> Result<Vec<ModelInfo>, BridgeError> {
        let url = format!("{}/api/tags", self.base_url());
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| BridgeError::Provider(format!("Failed to list models: {e}")))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(BridgeError::Provider(format!("Failed to list models: {error_text}")));
        }
        
        let models_response: OllamaModelsResponse = response
            .json()
            .await
            .map_err(|e| BridgeError::Provider(format!("Failed to parse models: {e}")))?;
        
        let models = models_response.models
            .into_iter()
            .map(|m| ModelInfo {
                name: m.name,
                description: m.parameter_size,
                context_length: None, // Ollama doesn't provide this in the list
                capabilities: vec!["chat".to_string(), "completion".to_string()],
            })
            .collect();
        
        Ok(models)
    }
    
    async fn health_check(&self) -> Result<HealthStatus, BridgeError> {
        let url = format!("{}/api/tags", self.base_url());
        
        match self.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                Ok(HealthStatus::Healthy)
            }
            Ok(response) => {
                Ok(HealthStatus::Degraded(format!(
                    "Ollama returned status: {}",
                    response.status()
                )))
            }
            Err(e) => {
                Ok(HealthStatus::Unhealthy(format!(
                    "Cannot connect to Ollama: {e}"
                )))
            }
        }
    }
} 