//! Audit trail abstractions

use crate::error::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Audit event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub id: String,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// Event type/category
    pub event_type: String,
    /// Who performed the action
    pub actor: Actor,
    /// What action was performed
    pub action: String,
    /// What resource was affected
    pub resource: Resource,
    /// Outcome of the action
    pub outcome: AuditOutcome,
    /// Additional context
    pub context: HashMap<String, serde_json::Value>,
    /// Correlation ID for tracing
    pub correlation_id: Option<String>,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(
        event_type: impl Into<String>,
        actor: Actor,
        action: impl Into<String>,
        resource: Resource,
        outcome: AuditOutcome,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: event_type.into(),
            actor,
            action: action.into(),
            resource,
            outcome,
            context: HashMap::new(),
            correlation_id: None,
        }
    }

    /// Add context to the event
    pub fn with_context(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.context.insert(key.into(), value);
        self
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }
}

/// Actor in an audit event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Actor {
    /// Actor type (user, service, system)
    pub actor_type: ActorType,
    /// Actor identifier
    pub id: String,
    /// Actor name/label
    pub name: Option<String>,
    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

impl Actor {
    /// Create a user actor
    pub fn user(id: impl Into<String>) -> Self {
        Self {
            actor_type: ActorType::User,
            id: id.into(),
            name: None,
            attributes: HashMap::new(),
        }
    }

    /// Create a service actor
    pub fn service(id: impl Into<String>) -> Self {
        Self {
            actor_type: ActorType::Service,
            id: id.into(),
            name: None,
            attributes: HashMap::new(),
        }
    }

    /// Create a system actor
    pub fn system() -> Self {
        Self {
            actor_type: ActorType::System,
            id: "system".to_string(),
            name: Some("System".to_string()),
            attributes: HashMap::new(),
        }
    }

    /// Set the actor name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Add an attribute
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

/// Actor type
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorType {
    /// Human user
    User,
    /// Service account
    Service,
    /// System/automated
    System,
    /// External system
    External,
}

/// Resource in an audit event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Resource {
    /// Resource type
    pub resource_type: String,
    /// Resource identifier
    pub id: String,
    /// Resource name/label
    pub name: Option<String>,
    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

impl Resource {
    /// Create a new resource
    pub fn new(resource_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            id: id.into(),
            name: None,
            attributes: HashMap::new(),
        }
    }

    /// Set the resource name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Add an attribute
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

/// Outcome of an audited action
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditOutcome {
    /// Action succeeded
    Success,
    /// Action failed
    Failure,
    /// Action denied by policy
    Denied,
    /// Unknown outcome
    Unknown,
}

/// Trait for audit logging
#[async_trait]
pub trait AuditLogger: Send + Sync {
    /// Log an audit event
    async fn log(&self, event: AuditEvent) -> Result<()>;

    /// Log multiple events
    async fn log_batch(&self, events: Vec<AuditEvent>) -> Result<()> {
        for event in events {
            self.log(event).await?;
        }
        Ok(())
    }
}

/// Trait for audit queries
#[async_trait]
pub trait AuditQuerier: Send + Sync {
    /// Query audit events
    async fn query(&self, filter: AuditFilter) -> Result<Vec<AuditEvent>>;

    /// Count events matching filter
    async fn count(&self, filter: AuditFilter) -> Result<u64>;
}

/// Filter for audit queries
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AuditFilter {
    /// Start time (inclusive)
    pub start_time: Option<DateTime<Utc>>,
    /// End time (exclusive)
    pub end_time: Option<DateTime<Utc>>,
    /// Filter by event type
    pub event_types: Vec<String>,
    /// Filter by actor ID
    pub actor_ids: Vec<String>,
    /// Filter by resource type
    pub resource_types: Vec<String>,
    /// Filter by resource ID
    pub resource_ids: Vec<String>,
    /// Filter by action
    pub actions: Vec<String>,
    /// Filter by outcome
    pub outcomes: Vec<AuditOutcome>,
    /// Filter by correlation ID
    pub correlation_id: Option<String>,
    /// Maximum results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

impl AuditFilter {
    /// Create a new filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Set time range
    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    /// Filter by event type
    pub fn with_event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_types.push(event_type.into());
        self
    }

    /// Filter by actor
    pub fn with_actor(mut self, actor_id: impl Into<String>) -> Self {
        self.actor_ids.push(actor_id.into());
        self
    }

    /// Filter by resource
    pub fn with_resource(
        mut self,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Self {
        self.resource_types.push(resource_type.into());
        self.resource_ids.push(resource_id.into());
        self
    }

    /// Set pagination
    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

/// Trait for audit retention
#[async_trait]
pub trait AuditRetention: Send + Sync {
    /// Apply retention policy
    async fn apply_retention(&self, policy: RetentionPolicy) -> Result<u64>;

    /// Archive old events
    async fn archive(&self, before: DateTime<Utc>) -> Result<u64>;
}

/// Retention policy
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// How long to keep events
    pub retention_period: chrono::Duration,
    /// Whether to archive before deletion
    pub archive_before_delete: bool,
    /// Event types to exclude from retention
    pub excluded_types: Vec<String>,
}

// Re-export uuid for convenience
pub use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_audit_event_creation() {
        let actor = Actor::user("user123")
            .with_name("John Doe")
            .with_attribute("ip", "192.168.1.1");

        let resource = Resource::new("document", "doc456")
            .with_name("Important Document")
            .with_attribute("classification", "confidential");

        let event = AuditEvent::new(
            "document.access",
            actor,
            "read",
            resource,
            AuditOutcome::Success,
        )
        .with_context("method", json!("api"))
        .with_correlation_id("req-789");

        assert_eq!(event.event_type, "document.access");
        assert_eq!(event.action, "read");
        assert_eq!(event.outcome, AuditOutcome::Success);
        assert_eq!(event.correlation_id, Some("req-789".to_string()));
        assert_eq!(event.context.get("method"), Some(&json!("api")));
    }

    #[test]
    fn test_audit_filter() {
        let filter = AuditFilter::new()
            .with_event_type("security.login")
            .with_actor("user123")
            .with_pagination(50, 0);

        assert_eq!(filter.event_types, vec!["security.login"]);
        assert_eq!(filter.actor_ids, vec!["user123"]);
        assert_eq!(filter.limit, Some(50));
        assert_eq!(filter.offset, Some(0));
    }

    #[test]
    fn test_actor_types() {
        let user = Actor::user("u123");
        assert_eq!(user.actor_type, ActorType::User);

        let service = Actor::service("svc-api");
        assert_eq!(service.actor_type, ActorType::Service);

        let system = Actor::system();
        assert_eq!(system.actor_type, ActorType::System);
        assert_eq!(system.name, Some("System".to_string()));
    }
}
