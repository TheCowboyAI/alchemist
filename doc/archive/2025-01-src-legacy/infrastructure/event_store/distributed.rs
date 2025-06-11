//! Distributed event store using NATS JetStream

use crate::domain::events::DomainEvent;
use crate::infrastructure::event_store::EventStoreError;
use crate::infrastructure::nats::{NatsClient, NatsError};
use async_nats::jetstream::{self, consumer::pull::Config as ConsumerConfig, stream::Config as StreamConfig};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use time::OffsetDateTime;
use lru::LruCache;
use std::sync::Arc;
use tokio::sync::Mutex;
use futures::StreamExt;
use std::num::NonZeroUsize;

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

    /// Bucket name for object store
    pub bucket_name: String,

    /// Maximum messages per subject
    pub max_messages_per_subject: i64,

    /// Maximum age of events in seconds
    pub max_age_seconds: u64,
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
            bucket_name: "events-bucket".to_string(),
            max_messages_per_subject: 1000,
            max_age_seconds: 365 * 24 * 60 * 60, // 1 year
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
    /// NATS client
    client: NatsClient,

    /// Configuration
    #[allow(dead_code)]
    config: DistributedEventStoreConfig,

    /// LRU cache for recent events
    cache: Arc<Mutex<LruCache<Uuid, StoredEvent>>>,

    /// Stream name
    stream_name: String,
}

impl DistributedEventStore {
    /// Create a new distributed event store
    pub async fn new(
        client: NatsClient,
        config: DistributedEventStoreConfig,
    ) -> Result<Self, EventStoreError> {
        let stream_name = config.stream_name.clone();

        // Get JetStream context
        let jetstream = client.jetstream()
            .map_err(|e| EventStoreError::Storage(e.to_string()))?;

        // Create stream configuration
        let stream_config = StreamConfig {
            name: stream_name.clone(),
            subjects: vec!["events.>".to_string()],
            retention: jetstream::stream::RetentionPolicy::Limits,
            max_messages_per_subject: config.max_messages_per_subject,
            max_age: Duration::from_secs(config.max_age_seconds),
            storage: jetstream::stream::StorageType::File,
            ..Default::default()
        };

        // Create or update the stream
        match jetstream.create_stream(stream_config).await {
            Ok(_) => info!("Created JetStream stream: {}", stream_name),
            Err(e) => {
                if e.to_string().contains("already exists") {
                    info!("JetStream stream already exists: {}", stream_name);
                } else {
                    return Err(EventStoreError::Storage(e.to_string()));
                }
            }
        }

        // Create LRU cache
        let cache_size = NonZeroUsize::new(config.cache_size)
            .unwrap_or(NonZeroUsize::new(10000).unwrap());
        let cache = Arc::new(Mutex::new(LruCache::new(cache_size)));

        Ok(Self {
            client,
            config,
            cache,
            stream_name,
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

        // Get JetStream context
        let jetstream = self.client.jetstream()
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        // Publish to JetStream
        let ack = jetstream
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
        let subject = format!("events.{aggregate_type}.>");

        // Get JetStream context
        let jetstream = self.client.jetstream()
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        // Get stream handle
        let stream = jetstream
            .get_stream(&self.stream_name)
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        // Create consumer for reading events
        let consumer = stream
            .create_consumer(ConsumerConfig {
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
        // Get JetStream context
        let jetstream = self.client.jetstream()
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        // Get stream handle
        let stream = jetstream
            .get_stream(&self.stream_name)
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        // Convert chrono DateTime to time OffsetDateTime
        let timestamp = start_time.timestamp();
        let start_time_offset = OffsetDateTime::from_unix_timestamp(timestamp)
            .map_err(|e| NatsError::SerializationError(format!("Invalid timestamp: {e}")))?;

        // Create consumer for replay
        let consumer_config = ConsumerConfig {
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
        // Get JetStream context
        let jetstream = self.client.jetstream()
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        // Get stream info
        let mut stream = jetstream
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

    /// Load events by aggregate type
    pub async fn load_events_by_type(
        &self,
        aggregate_type: &str,
    ) -> Result<Vec<DomainEvent>, NatsError> {
        let subject = format!("events.{aggregate_type}.>");

        // Get JetStream context
        let jetstream = self.client.jetstream()
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        // Get stream handle
        let stream = jetstream
            .get_stream(&self.stream_name)
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        // Create consumer for reading events
        let consumer = stream.create_consumer(ConsumerConfig {
            filter_subject: subject,
            ..Default::default()
        }).await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        // Fetch messages
        let mut messages = consumer.fetch().max_messages(1000).messages().await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        let mut events = Vec::new();

        while let Some(Ok(message)) = messages.next().await {
            // Deserialize event
            let stored_event = serde_json::from_slice::<StoredEvent>(&message.payload)
                .map_err(|e| NatsError::SerializationError(e.to_string()))?;

            events.push(stored_event.event);

            // Acknowledge message
            message.ack().await
                .map_err(|e| NatsError::JetStreamError(e.to_string()))?;
        }

        info!("Loaded {} events from JetStream for aggregate type {}",
            events.len(), aggregate_type);

        // Events are already in order from JetStream
        Ok(events)
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
    // Tests will be added in the next step
}
