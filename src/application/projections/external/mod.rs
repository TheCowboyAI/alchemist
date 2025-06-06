//! External system projections
//!
//! This module provides projections that sync graph events to external systems
//! like Neo4j, JSON files, n8n workflows, Paperless-NGx, SearXNG, and email.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc;
use futures::stream::Stream;
use std::pin::Pin;

use crate::domain::events::DomainEvent;
use crate::domain::commands::DomainCommand;

pub mod neo4j;
pub mod json;
pub mod n8n;
pub mod paperless;
pub mod searxng;
pub mod email;

/// Error types for projection operations
#[derive(Debug, Error)]
pub enum ProjectionError {
    #[error("Connection failed: {0}")]
    ConnectionError(String),

    #[error("Projection failed: {0}")]
    ProjectionFailed(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("External system error: {0}")]
    ExternalSystemError(String),

    #[error("Retry limit exceeded")]
    RetryLimitExceeded,
}

/// Recovery strategy for projection errors
#[derive(Debug, Clone)]
pub enum ErrorRecovery {
    /// Retry the operation
    Retry { delay: Duration },

    /// Skip this event and continue
    Skip,

    /// Send to dead letter queue
    DeadLetter,

    /// Stop the projection
    Stop,
}

/// Base trait for all external projections
#[async_trait]
pub trait ExternalProjection: Send + Sync {
    /// Configuration type for this projection
    type Config: Send + Sync;

    /// Connection type for this projection
    type Connection: Send + Sync;

    /// Create a new instance with configuration
    fn new(config: Self::Config) -> Self;

    /// Establish connection to the external system
    async fn connect(&self) -> Result<Self::Connection, ProjectionError>;

    /// Project a single event to the external system
    async fn project_event(
        &self,
        event: &DomainEvent,
        conn: &mut Self::Connection,
    ) -> Result<(), ProjectionError>;

    /// Handle projection errors
    async fn handle_error(&self, error: ProjectionError) -> ErrorRecovery {
        match error {
            ProjectionError::ConnectionError(_) => ErrorRecovery::Retry {
                delay: Duration::from_secs(5),
            },
            ProjectionError::RetryLimitExceeded => ErrorRecovery::DeadLetter,
            _ => ErrorRecovery::Skip,
        }
    }

    /// Health check for the projection
    async fn health_check(&self, conn: &mut Self::Connection) -> Result<(), ProjectionError> {
        Ok(())
    }
}

/// Batch projection wrapper for performance
pub struct BatchProjection<T: ExternalProjection> {
    projection: T,
    buffer: Vec<DomainEvent>,
    flush_interval: Duration,
    max_batch_size: usize,
}

impl<T: ExternalProjection> BatchProjection<T> {
    pub fn new(projection: T, flush_interval: Duration, max_batch_size: usize) -> Self {
        Self {
            projection,
            buffer: Vec::with_capacity(max_batch_size),
            flush_interval,
            max_batch_size,
        }
    }

    pub async fn add_event(&mut self, event: DomainEvent) -> Result<(), ProjectionError> {
        self.buffer.push(event);

        if self.buffer.len() >= self.max_batch_size {
            self.flush().await?;
        }

        Ok(())
    }

    pub async fn flush(&mut self) -> Result<(), ProjectionError> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let mut conn = self.projection.connect().await?;

        for event in self.buffer.drain(..) {
            self.projection.project_event(&event, &mut conn).await?;
        }

        Ok(())
    }
}

/// Resilient projection with retry and circuit breaker
pub struct ResilientProjection<T: ExternalProjection> {
    projection: T,
    retry_policy: RetryPolicy,
    circuit_breaker: CircuitBreaker,
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub exponential_base: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            exponential_base: 2.0,
        }
    }
}

#[derive(Debug)]
pub struct CircuitBreaker {
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
}

#[derive(Debug, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Configuration for all projections
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectionsConfig {
    #[serde(default)]
    pub neo4j: Option<neo4j::Neo4jConfig>,

    #[serde(default)]
    pub json: Option<json::JsonConfig>,

    #[serde(default)]
    pub n8n: Option<n8n::N8nConfig>,

    #[serde(default)]
    pub paperless: Option<paperless::PaperlessConfig>,

    #[serde(default)]
    pub searxng: Option<searxng::SearxngConfig>,

    #[serde(default)]
    pub email: Option<email::EmailConfig>,
}

/// Type alias for event streams
pub type EventStream<T> = Pin<Box<dyn Stream<Item = T> + Send>>;

/// Errors specific to ingestion
#[derive(Debug, Error)]
pub enum IngestError {
    #[error("Subscription failed: {0}")]
    SubscriptionError(String),

    #[error("Transform failed: {0}")]
    TransformError(String),

    #[error("Connection lost: {0}")]
    ConnectionLost(String),
}

/// Base trait for ingesting events from external systems
#[async_trait]
pub trait IngestHandler: Send + Sync {
    /// The type of events this handler ingests
    type Event: Send + Sync;

    /// Configuration for this handler
    type Config: Send + Sync;

    /// Create a new handler instance
    fn new(config: Self::Config) -> Self;

    /// Subscribe to the external system's event stream
    async fn subscribe(&self) -> Result<EventStream<Self::Event>, IngestError>;

    /// Transform an external event into domain commands
    async fn transform_event(
        &self,
        event: Self::Event,
    ) -> Result<Vec<DomainCommand>, IngestError>;

    /// Handle ingestion errors
    async fn handle_error(&self, error: IngestError) -> ErrorRecovery {
        match error {
            IngestError::ConnectionLost(_) => ErrorRecovery::Retry {
                delay: Duration::from_secs(5),
            },
            _ => ErrorRecovery::Skip,
        }
    }
}

/// Event correlation for transforming external events
pub struct EventCorrelator {
    rules: Vec<CorrelationRule>,
}

#[derive(Debug, Clone)]
pub struct CorrelationRule {
    pub name: String,
    pub source_pattern: String,
    pub transform: fn(serde_json::Value) -> Result<Vec<DomainCommand>, IngestError>,
}

/// Bidirectional event manager
pub struct BidirectionalEventManager {
    projections: Vec<Box<dyn ExternalProjection>>,
    ingest_handlers: Vec<Box<dyn IngestHandler<Event = serde_json::Value, Config = ()>>>,
    event_correlator: EventCorrelator,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.initial_delay, Duration::from_millis(100));
    }
}
