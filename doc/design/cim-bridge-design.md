# CIM Bridge Architecture Design

## Overview

The CIM Bridge provides a generic abstraction layer for integrating various AI model providers (Ollama, OpenAI, Anthropic) with CIM's Agent domain. It handles command/query translation, correlation tracking, and NATS-based communication while maintaining provider-specific implementations.

## Core Architecture

### Generic Bridge Interface

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Generic AI model bridge trait
#[async_trait]
pub trait AIModelBridge: Send + Sync {
    /// The provider-specific configuration type
    type Config: Send + Sync;
    
    /// The provider-specific error type
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Initialize the bridge with configuration
    async fn initialize(config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized;
    
    /// Submit a query to the AI model
    async fn query(&self, request: ModelRequest) -> Result<ModelResponse, Self::Error>;
    
    /// Stream a response from the AI model
    async fn stream_query(
        &self,
        request: ModelRequest,
    ) -> Result<Box<dyn Stream<Item = Result<StreamChunk, Self::Error>> + Send>, Self::Error>;
    
    /// Get available models from the provider
    async fn list_models(&self) -> Result<Vec<ModelInfo>, Self::Error>;
    
    /// Health check for the provider
    async fn health_check(&self) -> Result<HealthStatus, Self::Error>;
}

/// Generic model request with correlation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequest {
    pub id: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub model: String,
    pub messages: Vec<Message>,
    pub parameters: ModelParameters,
    pub metadata: HashMap<String, Value>,
}

/// Generic model response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub request_id: Uuid,
    pub correlation_id: Uuid,
    pub model: String,
    pub content: String,
    pub usage: Option<Usage>,
    pub metadata: HashMap<String, Value>,
    pub timestamp: DateTime<Utc>,
}

/// Message format for conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub name: Option<String>,
    pub metadata: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Function,
}

/// Model parameters (temperature, max_tokens, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParameters {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub seed: Option<u64>,
}
```

### NATS Integration Layer

```rust
/// Bridge service that connects NATS to AI providers
pub struct BridgeService<B: AIModelBridge> {
    bridge: B,
    nats_client: NatsClient,
    subject_prefix: String,
}

impl<B: AIModelBridge> BridgeService<B> {
    pub async fn new(
        bridge: B,
        nats_client: NatsClient,
        subject_prefix: String,
    ) -> Result<Self, Error> {
        Ok(Self {
            bridge,
            nats_client,
            subject_prefix,
        })
    }
    
    /// Start listening for commands and queries
    pub async fn start(&self) -> Result<(), Error> {
        // Subscribe to query requests
        let query_subject = format!("{}.query.request", self.subject_prefix);
        let mut query_sub = self.nats_client.subscribe(&query_subject).await?;
        
        // Subscribe to model list requests
        let list_subject = format!("{}.models.list", self.subject_prefix);
        let mut list_sub = self.nats_client.subscribe(&list_subject).await?;
        
        // Process messages
        tokio::select! {
            _ = self.handle_queries(query_sub) => {},
            _ = self.handle_model_lists(list_sub) => {},
        }
        
        Ok(())
    }
    
    async fn handle_queries(&self, mut sub: Subscription) -> Result<(), Error> {
        while let Some(msg) = sub.next().await {
            let request: ModelRequest = serde_json::from_slice(&msg.payload)?;
            
            // Track correlation
            let span = tracing::span!(
                tracing::Level::INFO,
                "ai_query",
                correlation_id = %request.correlation_id,
                causation_id = ?request.causation_id,
            );
            
            let response = self.bridge.query(request.clone()).await?;
            
            // Publish response
            let response_subject = format!("{}.query.response", self.subject_prefix);
            self.nats_client
                .publish(&response_subject, &serde_json::to_vec(&response)?)
                .await?;
            
            // Publish event
            let event = AgentQueryCompleted {
                agent_id: request.metadata.get("agent_id").cloned(),
                correlation_id: request.correlation_id,
                model: response.model,
                tokens_used: response.usage.map(|u| u.total_tokens),
            };
            
            let event_subject = "agent.query.completed";
            self.nats_client
                .publish(event_subject, &serde_json::to_vec(&event)?)
                .await?;
        }
        
        Ok(())
    }
}
```

## Ollama Bridge Implementation

```rust
use ollama_rs::{Ollama, GenerateRequest};

pub struct OllamaBridge {
    client: Ollama,
    default_model: String,
}

#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub host: String,
    pub port: u16,
    pub default_model: String,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 11434,
            default_model: "llama2".to_string(),
        }
    }
}

#[async_trait]
impl AIModelBridge for OllamaBridge {
    type Config = OllamaConfig;
    type Error = OllamaError;
    
    async fn initialize(config: Self::Config) -> Result<Self, Self::Error> {
        let client = Ollama::new(
            format!("http://{}:{}", config.host, config.port),
            None,
        );
        
        Ok(Self {
            client,
            default_model: config.default_model,
        })
    }
    
    async fn query(&self, request: ModelRequest) -> Result<ModelResponse, Self::Error> {
        // Convert messages to Ollama format
        let prompt = self.messages_to_prompt(&request.messages);
        
        let ollama_request = GenerateRequest {
            model: request.model.clone(),
            prompt,
            options: self.parameters_to_options(&request.parameters),
            ..Default::default()
        };
        
        let response = self.client.generate(ollama_request).await?;
        
        Ok(ModelResponse {
            request_id: request.id,
            correlation_id: request.correlation_id,
            model: request.model,
            content: response.response,
            usage: Some(Usage {
                prompt_tokens: response.prompt_eval_count.unwrap_or(0) as u32,
                completion_tokens: response.eval_count.unwrap_or(0) as u32,
                total_tokens: (response.prompt_eval_count.unwrap_or(0) + 
                              response.eval_count.unwrap_or(0)) as u32,
            }),
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        })
    }
    
    async fn list_models(&self) -> Result<Vec<ModelInfo>, Self::Error> {
        let models = self.client.list_models().await?;
        
        Ok(models
            .models
            .into_iter()
            .map(|m| ModelInfo {
                id: m.name,
                name: m.name,
                description: m.description,
                capabilities: vec!["text-generation".to_string()],
            })
            .collect())
    }
    
    async fn health_check(&self) -> Result<HealthStatus, Self::Error> {
        match self.client.list_models().await {
            Ok(_) => Ok(HealthStatus::Healthy),
            Err(_) => Ok(HealthStatus::Unhealthy("Cannot connect to Ollama".to_string())),
        }
    }
}
```

## OpenAI Bridge Implementation

```rust
use async_openai::{Client, types::{CreateChatCompletionRequest, ChatCompletionRequestMessage}};

pub struct OpenAIBridge {
    client: Client,
    organization_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub organization_id: Option<String>,
    pub base_url: Option<String>,
}

#[async_trait]
impl AIModelBridge for OpenAIBridge {
    type Config = OpenAIConfig;
    type Error = OpenAIError;
    
    async fn initialize(config: Self::Config) -> Result<Self, Self::Error> {
        let mut client_config = async_openai::config::OpenAIConfig::new()
            .with_api_key(config.api_key);
        
        if let Some(org_id) = &config.organization_id {
            client_config = client_config.with_org_id(org_id);
        }
        
        if let Some(base_url) = config.base_url {
            client_config = client_config.with_api_base(base_url);
        }
        
        let client = Client::with_config(client_config);
        
        Ok(Self {
            client,
            organization_id: config.organization_id,
        })
    }
    
    async fn query(&self, request: ModelRequest) -> Result<ModelResponse, Self::Error> {
        let messages: Vec<ChatCompletionRequestMessage> = request
            .messages
            .into_iter()
            .map(|m| self.convert_message(m))
            .collect();
        
        let openai_request = CreateChatCompletionRequest {
            model: request.model.clone(),
            messages,
            temperature: request.parameters.temperature,
            max_tokens: request.parameters.max_tokens.map(|t| t as i32),
            top_p: request.parameters.top_p,
            frequency_penalty: request.parameters.frequency_penalty,
            presence_penalty: request.parameters.presence_penalty,
            stop: request.parameters.stop_sequences,
            ..Default::default()
        };
        
        let response = self.client.chat().create(openai_request).await?;
        
        let choice = response.choices.first()
            .ok_or_else(|| OpenAIError::NoResponse)?;
        
        Ok(ModelResponse {
            request_id: request.id,
            correlation_id: request.correlation_id,
            model: response.model,
            content: choice.message.content.clone().unwrap_or_default(),
            usage: response.usage.map(|u| Usage {
                prompt_tokens: u.prompt_tokens as u32,
                completion_tokens: u.completion_tokens as u32,
                total_tokens: u.total_tokens as u32,
            }),
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        })
    }
}
```

## Anthropic Bridge Implementation

```rust
use anthropic_sdk::{Client, MessagesRequest};

pub struct AnthropicBridge {
    client: Client,
    default_model: String,
}

#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    pub api_key: String,
    pub default_model: String,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            default_model: "claude-3-opus-20240229".to_string(),
        }
    }
}

#[async_trait]
impl AIModelBridge for AnthropicBridge {
    type Config = AnthropicConfig;
    type Error = AnthropicError;
    
    async fn initialize(config: Self::Config) -> Result<Self, Self::Error> {
        let client = Client::new(&config.api_key);
        
        Ok(Self {
            client,
            default_model: config.default_model,
        })
    }
    
    async fn query(&self, request: ModelRequest) -> Result<ModelResponse, Self::Error> {
        let anthropic_request = MessagesRequest {
            model: request.model.clone(),
            messages: self.convert_messages(&request.messages),
            max_tokens: request.parameters.max_tokens.unwrap_or(1024),
            temperature: request.parameters.temperature,
            top_p: request.parameters.top_p,
            stop_sequences: request.parameters.stop_sequences,
            ..Default::default()
        };
        
        let response = self.client.messages(anthropic_request).await?;
        
        Ok(ModelResponse {
            request_id: request.id,
            correlation_id: request.correlation_id,
            model: request.model,
            content: response.content.first()
                .map(|c| c.text.clone())
                .unwrap_or_default(),
            usage: Some(Usage {
                prompt_tokens: response.usage.input_tokens as u32,
                completion_tokens: response.usage.output_tokens as u32,
                total_tokens: (response.usage.input_tokens + 
                              response.usage.output_tokens) as u32,
            }),
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        })
    }
}
```

## Domain Composition

### cim-compose Configuration

```rust
// cim-compose/src/compositions/ollama.rs

use cim_compose::{Compose, CompositionBuilder};
use cim_domain_agent::AgentDomain;
use cim_bridge_ollama::OllamaBridge;

pub fn compose_ollama_domain() -> Result<ComposedDomain, Error> {
    CompositionBuilder::new()
        .with_domain(AgentDomain::new())
        .with_bridge(OllamaBridge::new(OllamaConfig::default()))
        .with_event_mapping(|event| {
            match event {
                AgentEvent::QueryRequested { .. } => Some(BridgeCommand::Query),
                AgentEvent::ModelSwitched { .. } => Some(BridgeCommand::SwitchModel),
                _ => None,
            }
        })
        .with_projection(OllamaModelProjection::new())
        .build()
}
```

### Usage Example

```rust
// Example: Using the composed Ollama domain

use cim_domain_ollama::OllamaDomain;
use cim_domain_agent::commands::QueryAgent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize NATS
    let nats_client = NatsClient::connect("nats://localhost:4222").await?;
    
    // Create composed domain
    let ollama_domain = OllamaDomain::new(
        nats_client.clone(),
        OllamaConfig {
            host: "localhost".to_string(),
            port: 11434,
            default_model: "llama2".to_string(),
        },
    ).await?;
    
    // Start the domain
    ollama_domain.start().await?;
    
    // Send a query
    let query = QueryAgent {
        agent_id: AgentId::new(),
        prompt: "Explain event sourcing in simple terms".to_string(),
        model: Some("llama2".to_string()),
        parameters: Default::default(),
    };
    
    let correlation_id = Uuid::new_v4();
    
    nats_client
        .publish(
            "agent.command.query",
            &CommandEnvelope {
                command: query,
                correlation_id,
                causation_id: None,
                metadata: HashMap::new(),
            },
        )
        .await?;
    
    // Listen for response
    let mut sub = nats_client.subscribe("agent.query.response").await?;
    
    if let Some(msg) = sub.next().await {
        let response: ModelResponse = serde_json::from_slice(&msg.payload)?;
        println!("Response: {}", response.content);
    }
    
    Ok(())
}
```

## NATS Subject Hierarchy

```
# Commands
agent.command.query
agent.command.switch_model
agent.command.list_models

# Queries
agent.query.get_status
agent.query.get_history
agent.query.get_capabilities

# Events
agent.query.started
agent.query.completed
agent.query.failed
agent.model.switched
agent.models.listed

# Bridge-specific subjects
bridge.ollama.health
bridge.ollama.metrics
bridge.openai.health
bridge.openai.metrics
bridge.anthropic.health
bridge.anthropic.metrics
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Provider error: {0}")]
    ProviderError(String),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}
```

## Monitoring and Observability

```rust
/// Metrics collected by bridges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeMetrics {
    pub provider: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_latency_ms: f64,
    pub tokens_processed: u64,
    pub last_health_check: DateTime<Utc>,
    pub health_status: HealthStatus,
}

/// Health status for bridges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}
```

## Configuration Management

```yaml
# Example configuration file
bridges:
  ollama:
    enabled: true
    host: localhost
    port: 11434
    default_model: llama2
    health_check_interval: 30s
    
  openai:
    enabled: true
    api_key: ${OPENAI_API_KEY}
    organization_id: ${OPENAI_ORG_ID}
    default_model: gpt-4
    rate_limit:
      requests_per_minute: 60
      tokens_per_minute: 90000
      
  anthropic:
    enabled: true
    api_key: ${ANTHROPIC_API_KEY}
    default_model: claude-3-opus-20240229
    rate_limit:
      requests_per_minute: 50
```

## Testing Strategy

1. **Unit Tests**: Test each bridge implementation independently
2. **Integration Tests**: Test NATS communication and event flow
3. **Mock Providers**: Create mock implementations for testing
4. **Performance Tests**: Measure latency and throughput
5. **Resilience Tests**: Test error handling and recovery

This architecture provides a clean, extensible way to integrate multiple AI providers with CIM's event-driven architecture while maintaining proper separation of concerns and enabling easy composition of domains. 