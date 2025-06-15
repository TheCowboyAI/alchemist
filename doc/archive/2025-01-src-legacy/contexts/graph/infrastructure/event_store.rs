//! Graph event store implementation
//!
//! This module provides the event store implementation for the graph context,
//! integrating with NATS JetStream for persistence.

use crate::shared::events::{DomainEvent, EventStore, EventSubscription};
use crate::shared::types::Result;
use async_trait::async_trait;

/// Graph-specific event store implementation
pub struct GraphEventStore {
    // Will be implemented with NATS client
}

#[async_trait]
impl EventStore for GraphEventStore {
    async fn append_events(&self, events: Vec<Box<dyn DomainEvent>>) -> Result<()> {
        // TODO: Implement with NATS
        Ok(())
    }

    async fn load_events(
        &self,
        aggregate_id: &str,
        from_version: u64,
    ) -> Result<Vec<serde_json::Value>> {
        // TODO: Implement with NATS
        Ok(vec![])
    }

    async fn subscribe(&self, event_type: &str) -> Result<Box<dyn EventSubscription>> {
        // TODO: Implement with NATS
        unimplemented!()
    }
}
