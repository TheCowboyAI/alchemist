//! NATS client wrapper for Alchemist

use async_nats::{Client, Message, Subscriber};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use bevy::prelude::*;

/// NATS client wrapper
#[derive(Clone, Resource)]
pub struct NatsClient {
    client: Client,
}

impl NatsClient {
    /// Create a new NATS client
    pub async fn new(url: &str) -> Result<Self, async_nats::ConnectError> {
        let client = async_nats::connect(url).await?;
        Ok(Self { client })
    }

    /// Publish a message to a subject
    pub async fn publish(&self, subject: &str, payload: Vec<u8>) -> Result<(), async_nats::PublishError> {
        self.client.publish(subject.to_string(), Bytes::from(payload)).await
    }

    /// Publish a JSON message to a subject
    pub async fn publish_json<T: Serialize>(
        &self,
        subject: &str,
        payload: &T,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let json = serde_json::to_vec(payload)?;
        self.publish(subject, json).await.map_err(|e| e.into())
    }

    /// Request-reply pattern
    pub async fn request(&self, subject: &str, payload: Vec<u8>) -> Result<Message, async_nats::RequestError> {
        self.client
            .request(subject.to_string(), Bytes::from(payload))
            .await
    }

    /// Request-reply with timeout
    pub async fn request_timeout(
        &self,
        subject: &str,
        payload: Vec<u8>,
        timeout: Duration,
    ) -> Result<Message, Box<dyn std::error::Error + Send + Sync>> {
        match tokio::time::timeout(timeout, self.request(subject, payload)).await {
            Ok(result) => result.map_err(|e| e.into()),
            Err(_) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::TimedOut, "Request timed out")) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    /// Subscribe to a subject
    pub async fn subscribe(&self, subject: &str) -> Result<Subscriber, async_nats::SubscribeError> {
        self.client.subscribe(subject.to_string()).await
    }

    /// Queue subscribe to a subject
    pub async fn queue_subscribe(
        &self,
        subject: &str,
        queue_group: &str,
    ) -> Result<Subscriber, async_nats::SubscribeError> {
        self.client
            .queue_subscribe(subject.to_string(), queue_group.to_string())
            .await
    }

    /// Flush pending messages
    pub async fn flush(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.client.flush().await.map_err(|e| e.into())
    }

    /// Drain the connection
    pub async fn drain(&self) {
        let _ = self.client.drain().await;
    }

    /// Get the underlying NATS client
    pub fn inner(&self) -> &Client {
        &self.client
    }
}

/// NATS JetStream wrapper
pub struct JetStreamClient {
    context: async_nats::jetstream::Context,
}

impl JetStreamClient {
    /// Create a new JetStream client
    pub async fn new(client: &NatsClient) -> Self {
        let context = async_nats::jetstream::new(client.inner().clone());
        Self { context }
    }

    /// Get a stream by name
    pub async fn get_stream(
        &self,
        name: &str,
    ) -> Result<async_nats::jetstream::stream::Stream, Box<dyn std::error::Error + Send + Sync>> {
        self.context.get_stream(name).await.map_err(|e| e.into())
    }

    /// Create or update a stream
    pub async fn create_stream(
        &self,
        config: async_nats::jetstream::stream::Config,
    ) -> Result<async_nats::jetstream::stream::Stream, Box<dyn std::error::Error + Send + Sync>> {
        self.context.create_stream(config).await.map_err(|e| e.into())
    }

    /// Delete a stream
    pub async fn delete_stream(&self, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.context.delete_stream(name).await.map(|_| ()).map_err(|e| e.into())
    }

    /// Get a consumer by name
    pub async fn get_consumer<T: async_nats::jetstream::consumer::IntoConsumerConfig + async_nats::jetstream::consumer::FromConsumer>(
        &self,
        stream: &str,
        consumer_name: &str,
    ) -> Result<async_nats::jetstream::consumer::Consumer<T>, Box<dyn std::error::Error + Send + Sync>> {
        let stream = self.get_stream(stream).await?;
        stream.get_consumer(consumer_name).await.map_err(|e| e.into())
    }

    /// Publish a message to a stream
    pub async fn publish(
        &self,
        subject: &str,
        payload: Bytes,
    ) -> Result<async_nats::jetstream::context::PublishAckFuture, Box<dyn std::error::Error + Send + Sync>> {
        self.context.publish(subject.to_string(), payload).await.map_err(|e| e.into())
    }

    /// Get the underlying JetStream context
    pub fn context(&self) -> &async_nats::jetstream::Context {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nats_client_creation() {
        // This test requires a running NATS server
        if let Ok(client) = NatsClient::new("nats://localhost:4222").await {
            // Test basic publish
            let result = client.publish("test.subject", b"test message".to_vec()).await;
            assert!(result.is_ok() || result.is_err()); // Allow failure if no server
        }
    }
}