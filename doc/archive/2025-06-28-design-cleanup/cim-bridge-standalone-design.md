# CIM Bridge Standalone Architecture Design

## Overview

The CIM Bridge provides a completely standalone, UI-agnostic abstraction layer for integrating various AI model providers (Ollama, OpenAI, Anthropic) with CIM's event-driven architecture. It communicates exclusively through NATS, making it compatible with any UI framework or interface.

## Core Principles

1. **Zero UI Dependencies**: No Bevy, egui, or any UI framework dependencies
2. **NATS-Only Communication**: All interaction through NATS subjects
3. **Pure Domain Service**: Focused solely on AI model bridging
4. **Framework Agnostic**: Can be used with any UI or no UI at all

## Architecture

```
┌─────────────────┐     NATS Subjects      ┌──────────────────┐
│   Any UI/CLI    │ ◄──────────────────► │   CIM Bridge     │
│ (Bevy/egui/Web) │                        │  (Standalone)    │
└─────────────────┘                        └──────────────────┘
                                                    │
                                           ┌────────┴────────┐
                                           │                 │
                                      ┌────▼───┐  ┌─────────▼────┐
                                      │ Ollama │  │   OpenAI     │
                                      └────────┘  └──────────────┘
```

## NATS Subject Hierarchy

```yaml
# Commands (UI → Bridge)
bridge.command.query                    # Submit a query to current provider
bridge.command.stream_query             # Stream a response
bridge.command.switch_provider          # Change active provider
bridge.command.list_providers           # Get available providers
bridge.command.list_models              # List models for a provider
bridge.command.health_check             # Check provider health

# Queries (UI → Bridge)
bridge.query.status                     # Get bridge status
bridge.query.active_provider            # Get current provider
bridge.query.metrics                    # Get performance metrics
bridge.query.capabilities               # Get provider capabilities

# Events (Bridge → UI)
bridge.event.query.started              # Query processing started
bridge.event.query.completed            # Query completed
bridge.event.query.failed               # Query failed
bridge.event.stream.chunk               # Streaming response chunk
bridge.event.provider.switched          # Provider changed
bridge.event.provider.health            # Health status update
bridge.event.metrics.updated            # Metrics update

# Provider-specific subjects
bridge.ollama.status                    # Ollama-specific status
bridge.openai.status                    # OpenAI-specific status
bridge.anthropic.status                 # Anthropic-specific status
```

## Core Types (No UI Dependencies)

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Command envelope for NATS communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEnvelope {
    pub id: Uuid,
    pub command: BridgeCommand,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Bridge commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeCommand {
    Query {
        model: String,
        messages: Vec<Message>,
        parameters: ModelParameters,
    },
    StreamQuery {
        model: String,
        messages: Vec<Message>,
        parameters: ModelParameters,
    },
    SwitchProvider {
        provider: String,
        config: Option<serde_json::Value>,
    },
    ListProviders,
    ListModels {
        provider: Option<String>,
    },
    HealthCheck {
        provider: Option<String>,
    },
}

/// Query envelope for NATS communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryEnvelope {
    pub id: Uuid,
    pub query: BridgeQuery,
    pub correlation_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// Bridge queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeQuery {
    Status,
    ActiveProvider,
    Metrics {
        provider: Option<String>,
        time_range: Option<TimeRange>,
    },
    Capabilities {
        provider: Option<String>,
    },
}

/// Event envelope for NATS communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub event: BridgeEvent,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Bridge events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeEvent {
    QueryStarted {
        query_id: Uuid,
        provider: String,
        model: String,
    },
    QueryCompleted {
        query_id: Uuid,
        response: ModelResponse,
    },
    QueryFailed {
        query_id: Uuid,
        error: String,
        provider: String,
    },
    StreamChunk {
        query_id: Uuid,
        chunk: StreamChunk,
        sequence: u32,
    },
    ProviderSwitched {
        old_provider: Option<String>,
        new_provider: String,
    },
    ProviderHealth {
        provider: String,
        status: HealthStatus,
        details: Option<String>,
    },
    MetricsUpdated {
        provider: String,
        metrics: ProviderMetrics,
    },
}

/// Message format for AI conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub name: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Function,
}

/// Model parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelParameters {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub seed: Option<u64>,
}

/// Model response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub query_id: Uuid,
    pub model: String,
    pub content: String,
    pub usage: Option<Usage>,
    pub finish_reason: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Stream chunk for streaming responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub content: String,
    pub is_final: bool,
    pub usage: Option<Usage>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

/// Provider metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_latency_ms: f64,
    pub tokens_processed: u64,
    pub last_request: Option<DateTime<Utc>>,
}
```

## Standalone Bridge Service

```rust
use async_nats::{Client, Subscriber};
use tokio::sync::RwLock;
use std::sync::Arc;

/// Main bridge service - completely standalone
pub struct BridgeService {
    nats_client: Client,
    providers: Arc<RwLock<HashMap<String, Box<dyn Provider>>>>,
    active_provider: Arc<RwLock<Option<String>>>,
    metrics: Arc<RwLock<HashMap<String, ProviderMetrics>>>,
}

impl BridgeService {
    pub async fn new(nats_url: &str) -> Result<Self, BridgeError> {
        let nats_client = async_nats::connect(nats_url).await?;
        
        Ok(Self {
            nats_client,
            providers: Arc::new(RwLock::new(HashMap::new())),
            active_provider: Arc::new(RwLock::new(None)),
            metrics: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Start the bridge service
    pub async fn start(&self) -> Result<(), BridgeError> {
        // Subscribe to command subjects
        let command_sub = self.nats_client
            .subscribe("bridge.command.>")
            .await?;
        
        // Subscribe to query subjects
        let query_sub = self.nats_client
            .subscribe("bridge.query.>")
            .await?;
        
        // Process messages
        tokio::select! {
            _ = self.handle_commands(command_sub) => {},
            _ = self.handle_queries(query_sub) => {},
        }
        
        Ok(())
    }
    
    async fn handle_commands(&self, mut sub: Subscriber) -> Result<(), BridgeError> {
        while let Some(msg) = sub.next().await {
            let envelope: CommandEnvelope = serde_json::from_slice(&msg.payload)?;
            
            match envelope.command {
                BridgeCommand::Query { model, messages, parameters } => {
                    self.process_query(
                        envelope.id,
                        envelope.correlation_id,
                        model,
                        messages,
                        parameters,
                    ).await?;
                }
                BridgeCommand::StreamQuery { model, messages, parameters } => {
                    self.process_stream_query(
                        envelope.id,
                        envelope.correlation_id,
                        model,
                        messages,
                        parameters,
                    ).await?;
                }
                BridgeCommand::SwitchProvider { provider, config } => {
                    self.switch_provider(provider, config).await?;
                }
                BridgeCommand::ListProviders => {
                    self.list_providers(envelope.correlation_id).await?;
                }
                BridgeCommand::ListModels { provider } => {
                    self.list_models(provider, envelope.correlation_id).await?;
                }
                BridgeCommand::HealthCheck { provider } => {
                    self.health_check(provider, envelope.correlation_id).await?;
                }
            }
        }
        
        Ok(())
    }
    
    async fn process_query(
        &self,
        query_id: Uuid,
        correlation_id: Uuid,
        model: String,
        messages: Vec<Message>,
        parameters: ModelParameters,
    ) -> Result<(), BridgeError> {
        // Publish query started event
        self.publish_event(BridgeEvent::QueryStarted {
            query_id,
            provider: self.get_active_provider().await?,
            model: model.clone(),
        }, correlation_id).await?;
        
        // Get active provider
        let provider = self.get_active_provider_instance().await?;
        
        // Execute query
        match provider.query(model, messages, parameters).await {
            Ok(response) => {
                // Publish success event
                self.publish_event(BridgeEvent::QueryCompleted {
                    query_id,
                    response,
                }, correlation_id).await?;
                
                // Update metrics
                self.update_metrics(&provider.name(), true).await?;
            }
            Err(e) => {
                // Publish failure event
                self.publish_event(BridgeEvent::QueryFailed {
                    query_id,
                    error: e.to_string(),
                    provider: provider.name(),
                }, correlation_id).await?;
                
                // Update metrics
                self.update_metrics(&provider.name(), false).await?;
            }
        }
        
        Ok(())
    }
    
    async fn publish_event(
        &self,
        event: BridgeEvent,
        correlation_id: Uuid,
    ) -> Result<(), BridgeError> {
        let envelope = EventEnvelope {
            id: Uuid::new_v4(),
            event,
            correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };
        
        let subject = match &envelope.event {
            BridgeEvent::QueryStarted { .. } => "bridge.event.query.started",
            BridgeEvent::QueryCompleted { .. } => "bridge.event.query.completed",
            BridgeEvent::QueryFailed { .. } => "bridge.event.query.failed",
            BridgeEvent::StreamChunk { .. } => "bridge.event.stream.chunk",
            BridgeEvent::ProviderSwitched { .. } => "bridge.event.provider.switched",
            BridgeEvent::ProviderHealth { .. } => "bridge.event.provider.health",
            BridgeEvent::MetricsUpdated { .. } => "bridge.event.metrics.updated",
        };
        
        self.nats_client
            .publish(subject, serde_json::to_vec(&envelope)?.into())
            .await?;
        
        Ok(())
    }
}

/// Provider trait - implemented by each AI provider
#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> String;
    
    async fn query(
        &self,
        model: String,
        messages: Vec<Message>,
        parameters: ModelParameters,
    ) -> Result<ModelResponse, ProviderError>;
    
    async fn stream_query(
        &self,
        model: String,
        messages: Vec<Message>,
        parameters: ModelParameters,
    ) -> Result<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>, ProviderError>;
    
    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError>;
    
    async fn health_check(&self) -> Result<HealthStatus, ProviderError>;
}
```

## Usage Examples

### From Bevy UI

```rust
// In a Bevy system - NO direct bridge dependency
fn send_query_to_bridge(
    nats_client: Res<NatsClient>,
    mut events: EventReader<UserQueryEvent>,
) {
    for event in events.read() {
        let command = CommandEnvelope {
            id: Uuid::new_v4(),
            command: BridgeCommand::Query {
                model: "gpt-4".to_string(),
                messages: vec![
                    Message {
                        role: MessageRole::User,
                        content: event.query.clone(),
                        name: None,
                        metadata: None,
                    }
                ],
                parameters: Default::default(),
            },
            correlation_id: event.correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };
        
        // Send via NATS
        nats_client.publish(
            "bridge.command.query",
            serde_json::to_vec(&command).unwrap(),
        );
    }
}

// Listen for responses
fn handle_bridge_responses(
    nats_events: Res<NatsEventChannel>,
    mut ui_state: ResMut<ChatState>,
) {
    for event in nats_events.bridge_events() {
        match event {
            BridgeEvent::QueryCompleted { response, .. } => {
                ui_state.add_message(response.content);
            }
            _ => {}
        }
    }
}
```

### From CLI

```rust
// Pure CLI usage - no UI dependencies
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = async_nats::connect("nats://localhost:4222").await?;
    
    // Send query
    let command = CommandEnvelope {
        id: Uuid::new_v4(),
        command: BridgeCommand::Query {
            model: "llama2".to_string(),
            messages: vec![
                Message {
                    role: MessageRole::User,
                    content: "Hello, world!".to_string(),
                    name: None,
                    metadata: None,
                }
            ],
            parameters: Default::default(),
        },
        correlation_id: Uuid::new_v4(),
        causation_id: None,
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    };
    
    client.publish(
        "bridge.command.query",
        serde_json::to_vec(&command)?.into(),
    ).await?;
    
    // Listen for response
    let mut sub = client.subscribe("bridge.event.query.completed").await?;
    
    if let Some(msg) = sub.next().await {
        let event: EventEnvelope = serde_json::from_slice(&msg.payload)?;
        if let BridgeEvent::QueryCompleted { response, .. } = event.event {
            println!("Response: {}", response.content);
        }
    }
    
    Ok(())
}
```

### From Web API

```rust
// Actix-web handler - no UI dependencies
async fn query_endpoint(
    nats: web::Data<NatsClient>,
    req: web::Json<QueryRequest>,
) -> Result<impl Responder, Error> {
    let correlation_id = Uuid::new_v4();
    
    // Send command
    let command = CommandEnvelope {
        id: Uuid::new_v4(),
        command: BridgeCommand::Query {
            model: req.model.clone(),
            messages: req.messages.clone(),
            parameters: req.parameters.clone(),
        },
        correlation_id,
        causation_id: None,
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    };
    
    nats.publish(
        "bridge.command.query",
        serde_json::to_vec(&command)?.into(),
    ).await?;
    
    // Wait for response (with timeout)
    let response = wait_for_response(&nats, correlation_id).await?;
    
    Ok(HttpResponse::Ok().json(response))
}
```

## Configuration

```yaml
# bridge-config.yaml
bridge:
  nats:
    url: "nats://localhost:4222"
    
  providers:
    ollama:
      enabled: true
      host: "localhost"
      port: 11434
      default_model: "llama2"
      
    openai:
      enabled: true
      api_key: "${OPENAI_API_KEY}"
      default_model: "gpt-4"
      rate_limit:
        requests_per_minute: 60
        
    anthropic:
      enabled: true
      api_key: "${ANTHROPIC_API_KEY}"
      default_model: "claude-3-opus"
      
  defaults:
    provider: "ollama"
    timeout_ms: 30000
    max_retries: 3
```

## Benefits

1. **True Standalone**: No UI framework dependencies whatsoever
2. **Universal Compatibility**: Works with Bevy, egui, CLI, web, mobile, etc.
3. **Clean Architecture**: Pure domain service with clear boundaries
4. **Event-Driven**: Fully asynchronous, non-blocking communication
5. **Scalable**: Can run as separate service, scale independently
6. **Testable**: Easy to test without UI concerns

This architecture ensures the bridge can be used with any UI framework or no UI at all, maintaining complete separation of concerns. 