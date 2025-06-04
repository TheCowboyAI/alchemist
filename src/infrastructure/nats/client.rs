//! NATS client wrapper for CIM

use super::{NatsConfig, NatsError};
use async_nats::{Client, jetstream};
use async_nats::jetstream::{Context as JetStreamContext, stream};
use std::sync::Arc;
use tracing::{debug, info};
use futures::StreamExt;

/// Wrapper around NATS client with JetStream support
#[derive(Clone)]
pub struct NatsClient {
    /// Core NATS client
    client: Client,

    /// JetStream context
    jetstream: Option<Arc<JetStreamContext>>,

    /// Configuration
    config: NatsConfig,
}

impl NatsClient {
    /// Create a new NATS client from configuration
    pub async fn new(config: NatsConfig) -> Result<Self, NatsError> {
        let client = super::connect_with_config(&config).await?;

        let jetstream = if config.jetstream.enabled {
            info!("Initializing JetStream context");
            let js = jetstream::new(client.clone());

            // Create default event stream if it doesn't exist
            Self::ensure_event_stream(&js, &config.jetstream.default_stream).await?;

            Some(Arc::new(js))
        } else {
            None
        };

        Ok(Self {
            client,
            jetstream,
            config,
        })
    }

    /// Get the underlying NATS client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get JetStream context
    pub fn jetstream(&self) -> Result<&JetStreamContext, NatsError> {
        self.jetstream
            .as_ref()
            .map(|js| js.as_ref())
            .ok_or_else(|| NatsError::JetStreamError("JetStream not enabled".to_string()))
    }

    /// Publish a message to a subject
    pub async fn publish(&self, subject: &str, payload: Vec<u8>) -> Result<(), NatsError> {
        debug!("Publishing to subject: {}", subject);
        self.client
            .publish(subject.to_string(), payload.into())
            .await?;
        Ok(())
    }

    /// Publish with headers (for deduplication)
    pub async fn publish_with_headers(
        &self,
        subject: &str,
        headers: async_nats::HeaderMap,
        payload: Vec<u8>,
    ) -> Result<(), NatsError> {
        debug!("Publishing to subject {} with headers", subject);
        self.client
            .publish_with_headers(subject.to_string(), headers, payload.into())
            .await?;
        Ok(())
    }

    /// Subscribe to a subject
    pub async fn subscribe(&self, subject: &str) -> Result<async_nats::Subscriber, NatsError> {
        debug!("Subscribing to subject: {}", subject);
        Ok(self.client.subscribe(subject.to_string()).await?)
    }

    /// Create or get a JetStream stream
    pub async fn get_or_create_stream(
        &self,
        name: &str,
        subjects: Vec<String>,
    ) -> Result<stream::Stream, NatsError> {
        let js = self.jetstream()?;

        let config = stream::Config {
            name: name.to_string(),
            subjects,
            retention: match self.config.jetstream.default_stream.retention {
                super::config::RetentionPolicy::Limits => stream::RetentionPolicy::Limits,
                super::config::RetentionPolicy::Interest => stream::RetentionPolicy::Interest,
                super::config::RetentionPolicy::WorkQueue => stream::RetentionPolicy::WorkQueue,
            },
            max_age: self.config.jetstream.default_stream.max_age,
            max_messages: self.config.jetstream.default_stream.max_messages.unwrap_or(0),
            max_bytes: self.config.jetstream.default_stream.max_bytes.unwrap_or(0),
            duplicate_window: self.config.jetstream.default_stream.duplicate_window,
            ..Default::default()
        };

        match js.get_stream(&config.name).await {
            Ok(stream) => {
                info!("Using existing stream: {}", name);
                Ok(stream)
            }
            Err(_) => {
                info!("Creating new stream: {}", name);
                Ok(js.create_stream(config).await?)
            }
        }
    }

    /// Ensure the default event stream exists
    async fn ensure_event_stream(
        js: &JetStreamContext,
        config: &super::config::StreamConfig,
    ) -> Result<(), NatsError> {
        let stream_name = &config.name_prefix;
        let subjects = vec![
            format!("events.>"),
            format!("commands.>"),
            format!("queries.>"),
        ];

        let stream_config = stream::Config {
            name: stream_name.clone(),
            subjects,
            retention: match config.retention {
                super::config::RetentionPolicy::Limits => stream::RetentionPolicy::Limits,
                super::config::RetentionPolicy::Interest => stream::RetentionPolicy::Interest,
                super::config::RetentionPolicy::WorkQueue => stream::RetentionPolicy::WorkQueue,
            },
            max_age: config.max_age,
            max_messages: config.max_messages.unwrap_or(0),
            max_bytes: config.max_bytes.unwrap_or(0),
            duplicate_window: config.duplicate_window,
            ..Default::default()
        };

        match js.get_stream(&stream_name).await {
            Ok(_) => {
                info!("Event stream '{}' already exists", stream_name);
            }
            Err(_) => {
                info!("Creating event stream '{}'", stream_name);
                js.create_stream(stream_config).await?;
            }
        }

        Ok(())
    }

    /// Check if the client is connected
    pub async fn health_check(&self) -> Result<(), NatsError> {
        // Try to flush to verify connection
        self.client.flush().await?;

        // If JetStream is enabled, check it too
        if let Some(js) = &self.jetstream {
            let mut stream_names = js.stream_names();
            let _ = stream_names.next().await;
        }

        Ok(())
    }
}
