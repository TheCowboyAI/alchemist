//! Real-time dashboard updates via NATS subscriptions
//!
//! This module connects NATS event streams to the dashboard renderer,
//! providing live updates as domain events occur.

use anyhow::{Result, Context};
use async_nats::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{info, error};
use std::sync::Arc;
use futures::StreamExt;
use dashmap::DashMap;
use chrono::{DateTime, Utc};

use crate::{
    dashboard::{DashboardData, DomainInfo, DialogInfo, EventInfo, PolicyInfo},
    dashboard_events::{DashboardEvent, DashboardProjection},
    renderer_api::{RendererCommand, RendererApi},
};

/// Dashboard update message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardUpdate {
    /// Full dashboard refresh
    FullUpdate(DashboardData),
    
    /// Domain status changed
    DomainUpdate {
        domain: String,
        info: DomainInfo,
    },
    
    /// New event added
    EventAdded(EventInfo),
    
    /// Dialog status changed
    DialogUpdate {
        dialog_id: String,
        info: Option<DialogInfo>,
    },
    
    /// System metrics updated
    MetricsUpdate {
        total_events: u64,
        domains: Vec<(String, u64)>,
    },
    
    /// Policy status changed
    PolicyUpdate {
        policy_id: String,
        info: Option<PolicyInfo>,
    },
}

/// Manages real-time updates to dashboard renderers
pub struct DashboardRealtimeManager {
    /// NATS client for subscriptions
    nats_client: Client,
    
    /// Active dashboard renderer IDs
    active_dashboards: Arc<DashMap<String, DashboardState>>,
    
    /// Renderer API for sending updates
    renderer_api: Arc<RendererApi>,
    
    /// Dashboard projection for current state
    projection: Arc<tokio::sync::RwLock<DashboardProjection>>,
    
    /// Update channel
    update_tx: mpsc::Sender<(String, DashboardUpdate)>,
    update_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<(String, DashboardUpdate)>>>,
}

#[derive(Debug)]
struct DashboardState {
    renderer_id: String,
    last_update: DateTime<Utc>,
    update_frequency: std::time::Duration,
}

impl DashboardRealtimeManager {
    /// Create a new real-time dashboard manager
    pub fn new(
        nats_client: Client,
        renderer_api: Arc<RendererApi>,
    ) -> Self {
        let (update_tx, update_rx) = mpsc::channel(1000);
        
        Self {
            nats_client,
            active_dashboards: Arc::new(DashMap::new()),
            renderer_api,
            projection: Arc::new(tokio::sync::RwLock::new(DashboardProjection::new())),
            update_tx,
            update_rx: Arc::new(tokio::sync::Mutex::new(update_rx)),
        }
    }
    
    /// Register a dashboard renderer for updates
    pub fn register_dashboard(&self, renderer_id: String) {
        let state = DashboardState {
            renderer_id: renderer_id.clone(),
            last_update: Utc::now(),
            update_frequency: std::time::Duration::from_millis(100), // 10 updates/sec max
        };
        
        self.active_dashboards.insert(renderer_id, state);
        info!("Registered dashboard for real-time updates");
    }
    
    /// Unregister a dashboard renderer
    pub fn unregister_dashboard(&self, renderer_id: &str) {
        self.active_dashboards.remove(renderer_id);
        info!("Unregistered dashboard from real-time updates");
    }
    
    /// Start the real-time update system
    pub async fn start(self: Arc<Self>) -> Result<()> {
        // Start NATS event subscription
        let manager = self.clone();
        tokio::spawn(async move {
            if let Err(e) = manager.subscribe_to_events().await {
                error!("NATS subscription error: {}", e);
            }
        });
        
        // Start update dispatcher
        let manager = self.clone();
        tokio::spawn(async move {
            manager.dispatch_updates().await;
        });
        
        // Start periodic full updates
        let manager = self.clone();
        tokio::spawn(async move {
            manager.periodic_full_updates().await;
        });
        
        Ok(())
    }
    
    /// Subscribe to NATS events and transform to dashboard updates
    async fn subscribe_to_events(&self) -> Result<()> {
        // Subscribe to dashboard events
        let mut subscriber = self.nats_client
            .subscribe("alchemist.dashboard.events")
            .await
            .context("Failed to subscribe to dashboard events")?;
        
        // Also subscribe to direct domain events for lower latency
        let mut domain_subscriber = self.nats_client
            .subscribe("events.domain.>")
            .await
            .context("Failed to subscribe to domain events")?;
        
        info!("Started NATS subscriptions for dashboard updates");
        
        loop {
            tokio::select! {
                Some(msg) = subscriber.next() => {
                    if let Ok(event) = serde_json::from_slice::<DashboardEvent>(&msg.payload) {
                        self.handle_dashboard_event(event).await;
                    }
                }
                Some(msg) = domain_subscriber.next() => {
                    // Extract domain from subject (e.g., events.domain.workflow.created)
                    let parts: Vec<&str> = msg.subject.split('.').collect();
                    if parts.len() >= 3 {
                        let domain = parts[2];
                        self.handle_domain_event(domain, &msg.payload).await;
                    }
                }
            }
        }
    }
    
    /// Handle a dashboard event
    async fn handle_dashboard_event(&self, event: DashboardEvent) {
        let update = match event {
            DashboardEvent::DomainHealthChanged { domain, health, reason, timestamp } => {
                // Update projection
                let mut projection = self.projection.write().await;
                if let Some(status) = projection.domains.get_mut(&domain) {
                    status.health = health.clone();
                    status.last_event = timestamp;
                }
                
                // Create update
                DashboardUpdate::DomainUpdate {
                    domain: domain.clone(),
                    info: DomainInfo {
                        name: domain.clone(),
                        description: self.get_domain_description(&domain),
                        enabled: true,
                        health: health.clone().into(),
                        healthy: matches!(health, crate::dashboard_events::DomainHealth::Healthy),
                        event_count: projection.domains.get(&domain)
                            .map(|s| s.event_count)
                            .unwrap_or(0),
                        dependencies: self.get_domain_dependencies(&domain),
                    },
                }
            }
            
            DashboardEvent::DialogStarted { dialog_id, title, model, .. } => {
                let info = DialogInfo {
                    id: dialog_id.clone(),
                    title,
                    model,
                    message_count: 0,
                    last_active: "just now".to_string(),
                };
                
                DashboardUpdate::DialogUpdate {
                    dialog_id,
                    info: Some(info),
                }
            }
            
            DashboardEvent::DialogMessageAdded { dialog_id, message_count, .. } => {
                // Update projection
                let projection = self.projection.read().await;
                if let Some(dialog) = projection.active_dialogs.iter()
                    .find(|d| d.id == dialog_id) {
                    
                    let info = DialogInfo {
                        id: dialog_id.clone(),
                        title: dialog.title.clone(),
                        model: dialog.model.clone(),
                        message_count,
                        last_active: "just now".to_string(),
                    };
                    
                    DashboardUpdate::DialogUpdate {
                        dialog_id,
                        info: Some(info),
                    }
                } else {
                    return;
                }
            }
            
            DashboardEvent::MetricsUpdated { domain, events_processed, .. } => {
                // Update projection
                let mut projection = self.projection.write().await;
                if let Some(status) = projection.domains.get_mut(&domain) {
                    status.event_count = events_processed;
                }
                projection.metrics.total_events += 1;
                
                // Create metrics update
                let domains: Vec<(String, u64)> = projection.domains.iter()
                    .map(|(name, status)| (name.clone(), status.event_count))
                    .collect();
                
                DashboardUpdate::MetricsUpdate {
                    total_events: projection.metrics.total_events,
                    domains,
                }
            }
            
            _ => return, // Handle other events as needed
        };
        
        // Send update to all dashboards
        self.broadcast_update(update).await;
    }
    
    /// Handle a raw domain event
    async fn handle_domain_event(&self, domain: &str, payload: &[u8]) {
        // Update event count
        let mut projection = self.projection.write().await;
        
        let status = projection.domains.entry(domain.to_string())
            .or_insert_with(|| crate::dashboard_events::DomainStatus {
                name: domain.to_string(),
                event_count: 0,
                health: crate::dashboard_events::DomainHealth::Unknown,
                last_event: Utc::now(),
                error_count: 0,
            });
        
        status.event_count += 1;
        status.last_event = Utc::now();
        
        // Create event info
        let event_info = EventInfo {
            timestamp: Utc::now().format("%H:%M:%S").to_string(),
            domain: domain.to_string(),
            event_type: "DomainEvent".to_string(),
            summary: format!("Event in {} domain", domain),
        };
        
        // Add to recent events
        projection.recent_events.push(crate::dashboard_events::EventSummary {
            timestamp: Utc::now(),
            domain: domain.to_string(),
            event_type: "DomainEvent".to_string(),
            description: event_info.summary.clone(),
        });
        
        // Keep only last 100 events
        if projection.recent_events.len() > 100 {
            projection.recent_events.remove(0);
        }
        
        drop(projection);
        
        // Send update
        self.broadcast_update(DashboardUpdate::EventAdded(event_info)).await;
    }
    
    /// Broadcast an update to all active dashboards
    async fn broadcast_update(&self, update: DashboardUpdate) {
        for entry in self.active_dashboards.iter() {
            let renderer_id = entry.key().clone();
            let update_clone = update.clone();
            
            if let Err(e) = self.update_tx.send((renderer_id, update_clone)).await {
                error!("Failed to queue dashboard update: {}", e);
            }
        }
    }
    
    /// Dispatch queued updates to renderers
    async fn dispatch_updates(&self) {
        let mut rx = self.update_rx.lock().await;
        
        while let Some((renderer_id, update)) = rx.recv().await {
            // Check rate limiting
            if let Some(mut state) = self.active_dashboards.get_mut(&renderer_id) {
                let now = Utc::now();
                let time_since_last = now.signed_duration_since(state.last_update);
                
                if time_since_last.to_std().unwrap() < state.update_frequency {
                    // Skip this update due to rate limiting
                    continue;
                }
                
                state.last_update = now;
            }
            
            // Send update to renderer
            let data = serde_json::json!({
                "type": "dashboard_update",
                "update": update,
            });
            
            if let Err(e) = self.renderer_api.send_command(
                &renderer_id,
                RendererCommand::UpdateData { data }
            ).await {
                error!("Failed to send update to dashboard {}: {}", renderer_id, e);
                // Remove failed dashboard
                self.active_dashboards.remove(&renderer_id);
            }
        }
    }
    
    /// Send periodic full updates
    async fn periodic_full_updates(&self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // Generate full dashboard data
            let projection = self.projection.read().await;
            let dashboard_data = DashboardData::from_projection(&*projection);
            drop(projection);
            
            // Send to all dashboards
            self.broadcast_update(DashboardUpdate::FullUpdate(dashboard_data)).await;
        }
    }
    
    /// Get domain description
    fn get_domain_description(&self, name: &str) -> String {
        match name {
            "workflow" => "Business process execution and state machines",
            "agent" => "AI provider integration and semantic search",
            "document" => "Document lifecycle and version control",
            "policy" => "Business rule enforcement",
            "graph" => "Core graph operations and spatial positioning",
            "dialog" => "AI conversation management",
            _ => "Domain module",
        }.to_string()
    }
    
    /// Get domain dependencies
    fn get_domain_dependencies(&self, name: &str) -> Vec<String> {
        match name {
            "workflow" => vec!["graph".to_string()],
            "agent" => vec!["graph".to_string()],
            "dialog" => vec!["agent".to_string()],
            _ => vec![],
        }
    }
}

/// Create and start a dashboard with real-time updates
pub async fn launch_realtime_dashboard(
    renderer_manager: &crate::renderer::RendererManager,
    nats_client: Client,
    realtime_manager: Arc<DashboardRealtimeManager>,
) -> Result<String> {
    // Launch the dashboard renderer
    let dashboard_id = crate::dashboard::launch_dashboard(renderer_manager).await?;
    
    // Register for real-time updates
    realtime_manager.register_dashboard(dashboard_id.clone());
    
    info!("Launched dashboard {} with real-time updates", dashboard_id);
    
    Ok(dashboard_id)
}