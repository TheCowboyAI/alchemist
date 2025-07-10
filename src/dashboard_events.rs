//! Dashboard event sourcing integration with JetStream

use anyhow::Result;
use async_nats::jetstream::{self, consumer::PullConsumer, stream::Stream};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, error};
use futures::StreamExt;

use cim_domain::DomainEventEnum;

/// Dashboard-specific events that are derived from domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardEvent {
    /// Domain health changed
    DomainHealthChanged {
        domain: String,
        health: DomainHealth,
        reason: Option<String>,
        timestamp: DateTime<Utc>,
    },
    
    /// New dialog started
    DialogStarted {
        dialog_id: String,
        title: String,
        model: String,
        user_id: String,
        timestamp: DateTime<Utc>,
    },
    
    /// Dialog message added
    DialogMessageAdded {
        dialog_id: String,
        message_count: usize,
        tokens_used: u32,
        timestamp: DateTime<Utc>,
    },
    
    /// Workflow state changed
    WorkflowStateChanged {
        workflow_id: String,
        domain: String,
        from_state: String,
        to_state: String,
        timestamp: DateTime<Utc>,
    },
    
    /// Policy evaluated
    PolicyEvaluated {
        policy_id: String,
        domain: String,
        decision: PolicyDecision,
        timestamp: DateTime<Utc>,
    },
    
    /// Event processing metrics
    MetricsUpdated {
        domain: String,
        events_processed: u64,
        processing_rate: f64,
        error_rate: f64,
        timestamp: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainHealth {
    Healthy,
    Warning(String),
    Error(String),
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyDecision {
    Allow,
    Deny,
    RequireApproval,
}

/// Subscribes to JetStream subjects and transforms domain events into dashboard events
pub struct DashboardEventProcessor {
    jetstream: jetstream::Context,
    event_tx: mpsc::Sender<DashboardEvent>,
    subjects: Vec<String>,
}

impl DashboardEventProcessor {
    pub async fn new(
        nats_client: async_nats::Client,
        event_tx: mpsc::Sender<DashboardEvent>,
    ) -> Result<Self> {
        let jetstream = jetstream::new(nats_client);
        
        // Subscribe to all domain event subjects
        let subjects = vec![
            "events.domain.workflow.*".to_string(),
            "events.domain.agent.*".to_string(),
            "events.domain.document.*".to_string(),
            "events.domain.policy.*".to_string(),
            "events.domain.dialog.*".to_string(),
            "events.domain.graph.*".to_string(),
            "events.system.metrics.*".to_string(),
        ];
        
        Ok(Self {
            jetstream,
            event_tx,
            subjects,
        })
    }
    
    /// Start processing events from JetStream
    pub async fn start(&self) -> Result<()> {
        // Get or create the main event stream
        let stream = self.get_or_create_stream().await?;
        
        // Create durable consumer for dashboard
        let consumer = self.get_or_create_consumer(&stream).await?;
        
        // Start processing messages
        let mut messages = consumer.messages().await?;
        
        while let Some(Ok(msg)) = messages.next().await {
            // Parse the domain event
            if let Ok(event) = self.parse_domain_event(&msg.payload) {
                // Transform to dashboard event
                if let Some(dashboard_event) = self.transform_event(event).await {
                    // Send to dashboard
                    if let Err(e) = self.event_tx.send(dashboard_event).await {
                        error!("Failed to send dashboard event: {}", e);
                    }
                }
            }
            
            // Acknowledge message
            if let Err(e) = msg.ack().await {
                error!("Failed to ack message: {}", e);
            }
        }
        
        Ok(())
    }
    
    async fn get_or_create_stream(&self) -> Result<Stream> {
        let stream_name = "CIM-EVENTS";
        
        match self.jetstream.get_stream(stream_name).await {
            Ok(stream) => Ok(stream),
            Err(_) => {
                info!("Creating JetStream stream: {}", stream_name);
                
                let stream = self.jetstream
                    .create_stream(jetstream::stream::Config {
                        name: stream_name.to_string(),
                        subjects: self.subjects.clone(),
                        retention: jetstream::stream::RetentionPolicy::Limits,
                        max_messages: 1_000_000,
                        max_age: std::time::Duration::from_secs(86400 * 30), // 30 days
                        storage: jetstream::stream::StorageType::File,
                        num_replicas: 1,
                        ..Default::default()
                    })
                    .await?;
                
                Ok(stream)
            }
        }
    }
    
    async fn get_or_create_consumer(&self, stream: &Stream) -> Result<PullConsumer> {
        let consumer_name = "dashboard-consumer";
        
        match stream.get_consumer(consumer_name).await {
            Ok(consumer) => Ok(consumer),
            Err(_) => {
                info!("Creating JetStream consumer: {}", consumer_name);
                
                let consumer = stream
                    .create_consumer(jetstream::consumer::pull::Config {
                        name: Some(consumer_name.to_string()),
                        durable_name: Some(consumer_name.to_string()),
                        filter_subjects: self.subjects.clone(),
                        ..Default::default()
                    })
                    .await?;
                
                Ok(consumer)
            }
        }
    }
    
    fn parse_domain_event(&self, payload: &[u8]) -> Result<DomainEventEnum> {
        let event: DomainEventEnum = serde_json::from_slice(payload)?;
        Ok(event)
    }
    
    async fn transform_event(&self, event: DomainEventEnum) -> Option<DashboardEvent> {
        match event {
            // Transform workflow events
            DomainEventEnum::WorkflowTransitioned(event) => {
                Some(DashboardEvent::WorkflowStateChanged {
                    workflow_id: event.workflow_id.to_string(),
                    domain: "workflow".to_string(),
                    from_state: event.from_state,
                    to_state: event.to_state,
                    timestamp: Utc::now(),
                })
            }
            
            // For now, only handle workflow events
            _ => None,
        }
    }
}

/// Projection that maintains dashboard view state from events
pub struct DashboardProjection {
    pub domains: HashMap<String, DomainStatus>,
    pub active_dialogs: Vec<DialogSummary>,
    pub recent_events: Vec<EventSummary>,
    pub metrics: SystemMetrics,
}

#[derive(Debug, Clone)]
pub struct DomainStatus {
    pub name: String,
    pub health: DomainHealth,
    pub event_count: u64,
    pub last_event: DateTime<Utc>,
    pub error_count: u64,
}

#[derive(Debug, Clone)]
pub struct DialogSummary {
    pub id: String,
    pub title: String,
    pub model: String,
    pub message_count: usize,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct EventSummary {
    pub timestamp: DateTime<Utc>,
    pub domain: String,
    pub event_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Default)]
pub struct SystemMetrics {
    pub total_events: u64,
    pub events_per_second: f64,
    pub error_rate: f64,
    pub domains_active: usize,
}

impl DashboardProjection {
    pub fn new() -> Self {
        Self {
            domains: Self::initialize_domains(),
            active_dialogs: Vec::new(),
            recent_events: Vec::new(),
            metrics: SystemMetrics::default(),
        }
    }
    
    fn initialize_domains() -> HashMap<String, DomainStatus> {
        let domain_names = vec![
            "workflow", "agent", "document", "policy", "graph", "dialog",
            "identity", "organization", "person", "location",
        ];
        
        domain_names.into_iter()
            .map(|name| {
                (name.to_string(), DomainStatus {
                    name: name.to_string(),
                    health: DomainHealth::Unknown,
                    event_count: 0,
                    last_event: Utc::now(),
                    error_count: 0,
                })
            })
            .collect()
    }
    
    /// Apply dashboard event to update projection state
    pub fn apply_event(&mut self, event: DashboardEvent) {
        // Create summary before moving event
        let summary = self.event_to_summary(&event);
        
        match event {
            DashboardEvent::DomainHealthChanged { domain, health, .. } => {
                if let Some(status) = self.domains.get_mut(&domain) {
                    status.health = health;
                    status.last_event = Utc::now();
                }
            }
            
            DashboardEvent::DialogStarted { dialog_id, title, model, .. } => {
                self.active_dialogs.push(DialogSummary {
                    id: dialog_id,
                    title,
                    model,
                    message_count: 0,
                    last_activity: Utc::now(),
                });
                
                // Keep only last 20 dialogs
                if self.active_dialogs.len() > 20 {
                    self.active_dialogs.remove(0);
                }
            }
            
            DashboardEvent::DialogMessageAdded { dialog_id, message_count, .. } => {
                if let Some(dialog) = self.active_dialogs.iter_mut().find(|d| d.id == dialog_id) {
                    dialog.message_count = message_count;
                    dialog.last_activity = Utc::now();
                }
            }
            
            DashboardEvent::MetricsUpdated { domain, events_processed, .. } => {
                if let Some(status) = self.domains.get_mut(&domain) {
                    status.event_count = events_processed;
                    status.last_event = Utc::now();
                }
                
                self.metrics.total_events += 1;
                self.metrics.domains_active = self.domains.values()
                    .filter(|d| d.event_count > 0)
                    .count();
            }
            
            _ => {}
        }
        
        // Add to recent events
        self.recent_events.push(summary);
        
        // Keep only last 100 events
        if self.recent_events.len() > 100 {
            self.recent_events.remove(0);
        }
    }
    
    fn event_to_summary(&self, event: &DashboardEvent) -> EventSummary {
        match event {
            DashboardEvent::WorkflowStateChanged { workflow_id, from_state, to_state, timestamp, .. } => {
                EventSummary {
                    timestamp: *timestamp,
                    domain: "workflow".to_string(),
                    event_type: "StateChanged".to_string(),
                    description: format!("Workflow {} transitioned from {} to {}", workflow_id, from_state, to_state),
                }
            }
            DashboardEvent::PolicyEvaluated { policy_id, decision, timestamp, .. } => {
                EventSummary {
                    timestamp: *timestamp,
                    domain: "policy".to_string(),
                    event_type: "Evaluated".to_string(),
                    description: format!("Policy {} decision: {:?}", policy_id, decision),
                }
            }
            _ => EventSummary {
                timestamp: Utc::now(),
                domain: "system".to_string(),
                event_type: "Event".to_string(),
                description: "System event".to_string(),
            }
        }
    }
}

use std::collections::HashMap;