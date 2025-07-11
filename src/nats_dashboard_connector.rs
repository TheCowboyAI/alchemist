//! NATS to Dashboard connector for real-time event streaming

use anyhow::Result;
use async_nats::Client;
use tokio::sync::mpsc;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::{info, error, debug};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::dashboard::{DashboardData, DomainInfo, DialogInfo, EventInfo, DomainHealth};

/// Events that can be received from NATS
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NatsEvent {
    DomainEvent {
        domain: String,
        event_type: String,
        timestamp: DateTime<Utc>,
        data: serde_json::Value,
    },
    DialogEvent {
        dialog_id: String,
        event_type: String,
        timestamp: DateTime<Utc>,
        data: serde_json::Value,
    },
    SystemEvent {
        event_type: String,
        timestamp: DateTime<Utc>,
        data: serde_json::Value,
    },
    HealthCheck {
        domain: String,
        healthy: bool,
        message: Option<String>,
        timestamp: DateTime<Utc>,
    },
}

/// Connects NATS events to dashboard updates
pub struct NatsDashboardConnector {
    client: Client,
    dashboard_tx: mpsc::Sender<DashboardData>,
    current_data: DashboardData,
}

impl NatsDashboardConnector {
    pub fn new(
        client: Client,
        dashboard_tx: mpsc::Sender<DashboardData>,
        initial_data: DashboardData,
    ) -> Self {
        Self {
            client,
            dashboard_tx,
            current_data: initial_data,
        }
    }

    /// Start listening to NATS events and updating the dashboard
    pub async fn start(mut self) -> Result<()> {
        info!("Starting NATS dashboard connector");

        // Subscribe to all CIM events
        let mut domain_events = self.client.subscribe("cim.*.events.>").await?;
        let mut dialog_events = self.client.subscribe("dialog.*.events.>").await?;
        let mut system_events = self.client.subscribe("system.events.>").await?;
        let mut health_events = self.client.subscribe("cim.*.health").await?;

        info!("Subscribed to NATS event streams");

        loop {
            tokio::select! {
                Some(msg) = domain_events.next() => {
                    self.handle_domain_event(msg).await;
                }
                Some(msg) = dialog_events.next() => {
                    self.handle_dialog_event(msg).await;
                }
                Some(msg) = system_events.next() => {
                    self.handle_system_event(msg).await;
                }
                Some(msg) = health_events.next() => {
                    self.handle_health_event(msg).await;
                }
            }
        }
    }

    async fn handle_domain_event(&mut self, msg: async_nats::Message) {
        debug!("Received domain event on subject: {}", msg.subject);
        
        // Extract domain name from subject (cim.DOMAIN.events.EVENT_TYPE)
        let parts: Vec<&str> = msg.subject.split('.').collect();
        if parts.len() >= 4 {
            let domain = parts[1];
            let event_type = parts[3];
            
            // Update domain event count
            if let Some(domain_info) = self.current_data.domains.iter_mut()
                .find(|d| d.name == domain) 
            {
                domain_info.event_count += 1;
            } else {
                // New domain discovered
                self.current_data.domains.push(DomainInfo {
                    name: domain.to_string(),
                    description: format!("Domain: {}", domain),
                    enabled: true,
                    health: DomainHealth::Unknown,
                    healthy: true,
                    event_count: 1,
                    dependencies: vec![],
                });
            }

            // Add to recent events
            self.current_data.recent_events.push(EventInfo {
                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                domain: domain.to_string(),
                event_type: event_type.to_string(),
                summary: format!("Event: {} in {}", event_type, domain),
            });

            // Keep only last 50 events
            if self.current_data.recent_events.len() > 50 {
                self.current_data.recent_events.remove(0);
            }

            // Update total event count
            self.current_data.system_status.total_events += 1;

            // Send update
            self.send_update().await;
        }
    }

    async fn handle_dialog_event(&mut self, msg: async_nats::Message) {
        debug!("Received dialog event on subject: {}", msg.subject);
        
        // Parse dialog event
        if let Ok(event_data) = serde_json::from_slice::<serde_json::Value>(&msg.payload) {
            if let Some(dialog_id) = event_data.get("dialog_id").and_then(|v| v.as_str()) {
                if let Some(dialog) = self.current_data.active_dialogs.iter_mut()
                    .find(|d| d.id == dialog_id)
                {
                    dialog.message_count += 1;
                    dialog.last_active = "just now".to_string();
                } else {
                    // New dialog
                    self.current_data.active_dialogs.push(DialogInfo {
                        id: dialog_id.to_string(),
                        title: event_data.get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Untitled")
                            .to_string(),
                        model: event_data.get("model")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        message_count: 1,
                        last_active: "just now".to_string(),
                    });
                }
            }
        }

        self.send_update().await;
    }

    async fn handle_system_event(&mut self, msg: async_nats::Message) {
        debug!("Received system event on subject: {}", msg.subject);
        
        // Update system status based on event
        if msg.subject.contains("nats.connected") {
            self.current_data.system_status.nats_connected = true;
        } else if msg.subject.contains("nats.disconnected") {
            self.current_data.system_status.nats_connected = false;
        }

        self.send_update().await;
    }

    async fn handle_health_event(&mut self, msg: async_nats::Message) {
        debug!("Received health event on subject: {}", msg.subject);
        
        // Extract domain from subject
        let parts: Vec<&str> = msg.subject.split('.').collect();
        if parts.len() >= 3 {
            let domain = parts[1];
            
            // Parse health status
            if let Ok(health_data) = serde_json::from_slice::<serde_json::Value>(&msg.payload) {
                let healthy = health_data.get("healthy")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                
                let health_status = if healthy {
                    DomainHealth::Healthy
                } else {
                    let message = health_data.get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unhealthy");
                    DomainHealth::Error(message.to_string())
                };
                
                // Update domain health
                if let Some(domain_info) = self.current_data.domains.iter_mut()
                    .find(|d| d.name == domain)
                {
                    domain_info.health = health_status;
                    domain_info.healthy = healthy;
                }
            }
        }

        self.send_update().await;
    }

    async fn send_update(&self) {
        if let Err(e) = self.dashboard_tx.send(self.current_data.clone()).await {
            error!("Failed to send dashboard update: {}", e);
        }
    }
}

/// Create a NATS-connected dashboard data stream
pub async fn create_nats_dashboard_stream(
    nats_client: Client,
    initial_data: DashboardData,
) -> (mpsc::Receiver<DashboardData>, tokio::task::JoinHandle<()>) {
    let (tx, rx) = mpsc::channel(100);
    
    let connector = NatsDashboardConnector::new(nats_client, tx, initial_data);
    
    let handle = tokio::spawn(async move {
        if let Err(e) = connector.start().await {
            error!("NATS dashboard connector error: {}", e);
        }
    });
    
    (rx, handle)
}