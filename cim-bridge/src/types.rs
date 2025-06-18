//! Core types for CIM Bridge NATS communication

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

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
    ProvidersListed {
        providers: Vec<ProviderInfo>,
    },
    ModelsListed {
        provider: String,
        models: Vec<ModelInfo>,
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

/// Time range for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub is_configured: bool,
    pub capabilities: Vec<String>,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub description: Option<String>,
    pub context_length: Option<u32>,
    pub capabilities: Vec<String>,
} 