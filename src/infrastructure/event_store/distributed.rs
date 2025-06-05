//! Distributed event store using NATS JetStream

use crate::domain::events::DomainEvent;
use crate::infrastructure::nats::{NatsClient, NatsError};
use async_nats::jetstream::{self, stream::Config as StreamConfig, Context};
use futures::StreamExt;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use time::OffsetDateTime;

/// Configuration for the distributed event store
#[derive(Debug, Clone)]
pub struct DistributedEventStoreConfig {
    /// Stream name for events
    pub stream_name: String,

    /// Subject pattern for events
    pub subject_pattern: String,

    /// Maximum age of events (in seconds)
    pub max_age_secs: u64,

    /// Cache size for recent events
    pub cache_size: usize,

    /// Enable deduplication
    pub enable_deduplication: bool,

    /// Deduplication window (in seconds)
    pub deduplication_window_secs: u64,
}

impl Default for DistributedEventStoreConfig {
    fn default() -> Self {
        Self {
            stream_name: "CIM-EVENTS".to_string(),
            subject_pattern: "events.>".to_string(),
            max_age_secs: 365 * 24 * 60 * 60, // 1 year
            cache_size: 10000,
            enable_deduplication: true,
            deduplication_window_secs: 120, // 2 minutes
        }
    }
}

/// Event wrapper for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub event_type: String,
    pub event: DomainEvent,
    pub timestamp: DateTime<Utc>,
}

/// Distributed event store using NATS JetStream
pub struct DistributedEventStore {
    /// JetStream context
    context: Context,

    /// Configuration
    config: DistributedEventStoreConfig,

    /// LRU cache for recent events
    cache: Arc<Mutex<LruCache<Uuid, StoredEvent>>>,

    /// Stream name
    stream_name: String,
}

impl DistributedEventStore {
    /// Create a new distributed event store
    pub async fn new(
        nats_client: &NatsClient,
        config: DistributedEventStoreConfig,
    ) -> Result<Self, NatsError> {
        // Get JetStream context
        let context = jetstream::new(nats_client.client().clone());

        // Create or update the stream
        let stream_config = StreamConfig {
            name: config.stream_name.clone(),
            subjects: vec![config.subject_pattern.clone()],
            max_age: std::time::Duration::from_secs(config.max_age_secs),
            duplicate_window: if config.enable_deduplication {
                std::time::Duration::from_secs(config.deduplication_window_secs)
            } else {
                std::time::Duration::from_secs(0)
            },
            ..Default::default()
        };

        match context.create_stream(stream_config.clone()).await {
            Ok(_) => info!("Created JetStream stream: {}", config.stream_name),
            Err(e) if e.to_string().contains("already exists") => {
                // Stream already exists, that's fine
                info!("JetStream stream already exists: {}", config.stream_name);
            }
            Err(e) => return Err(NatsError::JetStreamError(e.to_string())),
        }

        // Create LRU cache
        let cache_size = NonZeroUsize::new(config.cache_size)
            .unwrap_or(NonZeroUsize::new(10000).unwrap());
        let cache = Arc::new(Mutex::new(LruCache::new(cache_size)));

        Ok(Self {
            context,
            stream_name: config.stream_name.clone(),
            config,
            cache,
        })
    }

    /// Store an event with aggregate information
    pub async fn store_event(
        &self,
        event: &DomainEvent,
        aggregate_id: Uuid,
        aggregate_type: &str,
    ) -> Result<(), NatsError> {
        // Create stored event wrapper
        let stored_event = StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            aggregate_type: aggregate_type.to_string(),
            event_type: event.event_type().to_string(),
            event: event.clone(),
            timestamp: Utc::now(),
        };

        // Determine subject based on event type
        let subject = format!("events.{}.{}", aggregate_type, event.event_type());

        // Serialize event
        let payload = serde_json::to_vec(&stored_event)
            .map_err(|e| NatsError::SerializationError(e.to_string()))?;

        // Publish to JetStream
        let ack = self.context
            .publish(subject.clone(), payload.into())
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        info!("Stored event {} to JetStream, sequence: {}",
            stored_event.event_id, ack.sequence);

        // Update cache
        let mut cache = self.cache.lock().await;
        cache.put(stored_event.event_id, stored_event);

        Ok(())
    }

    /// Load events for an aggregate
    pub async fn load_events(
        &self,
        aggregate_id: Uuid,
        aggregate_type: &str,
    ) -> Result<Vec<DomainEvent>, NatsError> {
        // Check cache first
        let mut cached_events = Vec::new();
        {
            let cache = self.cache.lock().await;
            // Note: This is a simplified cache check. In production, you'd want
            // a more sophisticated cache that can query by aggregate_id
            for (_, stored_event) in cache.iter() {
                if stored_event.aggregate_id == aggregate_id {
                    cached_events.push(stored_event.clone());
                }
            }
        }

        if !cached_events.is_empty() {
            info!("Loaded {} events from cache for aggregate {}",
                cached_events.len(), aggregate_id);
            return Ok(cached_events.into_iter().map(|se| se.event).collect());
        }

        // Query JetStream for events
        let subject = format!("events.{}.>", aggregate_type);

        // Get stream handle
        let stream = self.context
            .get_stream(&self.stream_name)
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        // Create consumer for reading events
        let consumer = stream
            .create_consumer(jetstream::consumer::pull::Config {
                filter_subject: subject,
                ..Default::default()
            })
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        // Fetch messages
        let mut messages = consumer
            .fetch()
            .max_messages(1000)
            .messages()
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        let mut events = Vec::new();

        while let Some(Ok(message)) = messages.next().await {
            // Deserialize event
            match serde_json::from_slice::<StoredEvent>(&message.payload) {
                Ok(stored_event) if stored_event.aggregate_id == aggregate_id => {
                    // Update cache
                    let mut cache = self.cache.lock().await;
                    cache.put(stored_event.event_id, stored_event.clone());

                    events.push(stored_event);
                }
                Ok(_) => {
                    // Event for different aggregate, skip
                }
                Err(e) => {
                    warn!("Failed to deserialize event: {}", e);
                }
            }

            // Acknowledge message
            message.ack().await
                .map_err(|e| NatsError::JetStreamError(e.to_string()))?;
        }

        info!("Loaded {} events from JetStream for aggregate {}",
            events.len(), aggregate_id);

        // Sort events by timestamp
        events.sort_by_key(|e| e.timestamp);

        Ok(events.into_iter().map(|se| se.event).collect())
    }

    /// Replay events from a specific point in time
    pub async fn replay_from(
        &self,
        start_time: chrono::DateTime<chrono::Utc>,
        subject_filter: Option<String>,
    ) -> Result<impl futures::Stream<Item = Result<DomainEvent, NatsError>>, NatsError> {
        // Get stream handle
        let stream = self.context
            .get_stream(&self.stream_name)
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        // Convert chrono DateTime to time OffsetDateTime
        let timestamp = start_time.timestamp();
        let start_time_offset = OffsetDateTime::from_unix_timestamp(timestamp)
            .map_err(|e| NatsError::SerializationError(format!("Invalid timestamp: {}", e)))?;

        // Create consumer for replay
        let consumer_config = jetstream::consumer::pull::Config {
            deliver_policy: jetstream::consumer::DeliverPolicy::ByStartTime {
                start_time: start_time_offset,
            },
            filter_subject: subject_filter.unwrap_or_else(|| "events.>".to_string()),
            ..Default::default()
        };

        let consumer = stream
            .create_consumer(consumer_config)
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        // Create stream of events
        let messages = consumer
            .stream()
            .max_messages_per_batch(100)
            .messages()
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        // Transform to event stream
        let event_stream = messages.map(move |result| {
            match result {
                Ok(message) => {
                    // Deserialize event
                    let stored_event = serde_json::from_slice::<StoredEvent>(&message.payload)
                        .map_err(|e| NatsError::SerializationError(e.to_string()))?;

                    // Acknowledge message
                    tokio::spawn(async move {
                        if let Err(e) = message.ack().await {
                            error!("Failed to acknowledge message: {}", e);
                        }
                    });

                    Ok(stored_event.event)
                }
                Err(e) => Err(NatsError::JetStreamError(e.to_string())),
            }
        });

        Ok(event_stream)
    }

    /// Get event store statistics
    pub async fn get_stats(&self) -> Result<EventStoreStats, NatsError> {
        // Get stream info
        let mut stream = self.context
            .get_stream(&self.stream_name)
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        let info = stream.info()
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        let cache_stats = {
            let cache = self.cache.lock().await;
            CacheStats {
                size: cache.len(),
                capacity: cache.cap().get(),
            }
        };

        Ok(EventStoreStats {
            total_events: info.state.messages,
            total_bytes: info.state.bytes,
            first_sequence: info.state.first_sequence,
            last_sequence: info.state.last_sequence,
            cache_stats,
        })
    }
}

/// Event store statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStoreStats {
    pub total_events: u64,
    pub total_bytes: u64,
    pub first_sequence: u64,
    pub last_sequence: u64,
    pub cache_stats: CacheStats,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests will be added in the next step
}
