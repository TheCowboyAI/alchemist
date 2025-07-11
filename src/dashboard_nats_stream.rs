//! Real-time NATS event streaming for the dashboard

use anyhow::Result;
use async_nats::Client;
use futures::stream::StreamExt;
use tokio::sync::mpsc;
use tracing::{info, warn, error};

use crate::dashboard::{DashboardData, DomainInfo, DialogInfo, EventInfo, DomainHealth};
use crate::dashboard_events::DashboardEvent;

/// Streams real-time NATS events to the dashboard
pub struct DashboardNatsStream {
    nats_client: Client,
    data_sender: mpsc::Sender<DashboardData>,
    current_data: DashboardData,
}

impl DashboardNatsStream {
    pub fn new(
        nats_client: Client,
        data_sender: mpsc::Sender<DashboardData>,
        initial_data: DashboardData,
    ) -> Self {
        Self {
            nats_client,
            data_sender,
            current_data: initial_data,
        }
    }

    /// Start streaming events from NATS
    pub async fn start_streaming(mut self) -> Result<()> {
        info!("Starting NATS event streaming for dashboard");

        // Subscribe to all CIM events
        let mut subscriber = self.nats_client
            .subscribe("cim.>")
            .await?;

        // Also subscribe to dashboard-specific events
        let mut dashboard_sub = self.nats_client
            .subscribe("dashboard.>")
            .await?;

        // Process events
        loop {
            tokio::select! {
                Some(msg) = subscriber.next() => {
                    self.handle_cim_event(msg).await;
                }
                Some(msg) = dashboard_sub.next() => {
                    self.handle_dashboard_event(msg).await;
                }
            }
        }
    }

    async fn handle_cim_event(&mut self, msg: async_nats::Message) {
        let subject = msg.subject.to_string();
        let parts: Vec<&str> = subject.split('.').collect();

        if parts.len() >= 3 {
            let domain = parts[1];
            let event_type = parts[2..].join(".");

            // Update domain event count
            if let Some(domain_info) = self.current_data.domains.iter_mut()
                .find(|d| d.name == domain) {
                domain_info.event_count += 1;
            }

            // Add to recent events
            let event = EventInfo {
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                domain: domain.to_string(),
                event_type: event_type.clone(),
                summary: format!("Event on {}", subject),
            };

            self.current_data.recent_events.insert(0, event);
            if self.current_data.recent_events.len() > 50 {
                self.current_data.recent_events.truncate(50);
            }

            // Update total event count
            self.current_data.system_status.total_events += 1;

            // Handle specific event types
            match (domain, event_type.as_str()) {
                ("dialog", "created") => {
                    self.handle_dialog_created(&msg.payload).await;
                }
                ("dialog", "message.added") => {
                    self.handle_dialog_message(&msg.payload).await;
                }
                ("workflow", "started") | ("workflow", "completed") => {
                    self.update_domain_health(domain, DomainHealth::Healthy).await;
                }
                ("agent", "error") => {
                    self.update_domain_health(
                        domain, 
                        DomainHealth::Error("Agent error occurred".to_string())
                    ).await;
                }
                _ => {}
            }

            // Send updated data
            let _ = self.data_sender.send(self.current_data.clone()).await;
        }
    }

    async fn handle_dashboard_event(&mut self, msg: async_nats::Message) {
        // Handle dashboard-specific control events
        let subject = msg.subject.to_string();
        
        match subject.as_str() {
            "dashboard.refresh" => {
                // Force refresh all data
                info!("Dashboard refresh requested");
                let _ = self.data_sender.send(self.current_data.clone()).await;
            }
            "dashboard.domain.health" => {
                // Update domain health status
                if let Ok(event) = serde_json::from_slice::<DashboardEvent>(&msg.payload) {
                    match event {
                        DashboardEvent::DomainHealthChanged { domain, health, .. } => {
                            self.update_domain_health(&domain, health.into()).await;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    async fn handle_dialog_created(&mut self, payload: &[u8]) {
        // Parse dialog creation event and add to active dialogs
        if let Ok(dialog_data) = serde_json::from_slice::<serde_json::Value>(payload) {
            if let (Some(id), Some(title)) = (
                dialog_data["id"].as_str(),
                dialog_data["title"].as_str()
            ) {
                let dialog = DialogInfo {
                    id: id.to_string(),
                    title: title.to_string(),
                    model: dialog_data["model"].as_str().unwrap_or("unknown").to_string(),
                    message_count: 0,
                    last_active: "just now".to_string(),
                };
                
                self.current_data.active_dialogs.push(dialog);
            }
        }
    }

    async fn handle_dialog_message(&mut self, payload: &[u8]) {
        // Update dialog message count and last active time
        if let Ok(msg_data) = serde_json::from_slice::<serde_json::Value>(payload) {
            if let Some(dialog_id) = msg_data["dialog_id"].as_str() {
                if let Some(dialog) = self.current_data.active_dialogs.iter_mut()
                    .find(|d| d.id == dialog_id) {
                    dialog.message_count += 1;
                    dialog.last_active = "just now".to_string();
                }
            }
        }
    }

    async fn update_domain_health(&mut self, domain: &str, health: DomainHealth) {
        if let Some(domain_info) = self.current_data.domains.iter_mut()
            .find(|d| d.name == domain) {
            domain_info.health = health.clone();
            domain_info.healthy = matches!(health, DomainHealth::Healthy);
        }
    }
}

/// Create and start a NATS event stream for the dashboard
pub async fn create_dashboard_stream(
    nats_client: Client,
    initial_data: DashboardData,
) -> (mpsc::Receiver<DashboardData>, tokio::task::JoinHandle<()>) {
    let (tx, rx) = mpsc::channel(100);
    
    let stream = DashboardNatsStream::new(nats_client, tx, initial_data);
    
    let handle = tokio::spawn(async move {
        if let Err(e) = stream.start_streaming().await {
            error!("Dashboard NATS stream error: {}", e);
        }
    });
    
    (rx, handle)
}