//! Domain Dashboard - Main visual interface for Alchemist
//! 
//! This dashboard consumes events from JetStream and displays real-time
//! domain status using Event Sourcing and DDD patterns.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::renderer::{RenderRequest, RenderData, RenderConfig, RendererType};
use uuid::Uuid;
use tokio::sync::mpsc;
use std::sync::Arc;
use async_nats::Client;

use crate::dashboard_events::{DashboardEventProcessor, DashboardProjection, DomainHealth as EventDomainHealth};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub domains: Vec<DomainInfo>,
    pub active_dialogs: Vec<DialogInfo>,
    pub recent_events: Vec<EventInfo>,
    pub system_status: SystemStatus,
    pub active_policies: Vec<PolicyInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInfo {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub health: DomainHealth,
    pub healthy: bool,
    pub event_count: u64,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainHealth {
    Healthy,
    Warning(String),
    Error(String),
    Unknown,
}

impl From<EventDomainHealth> for DomainHealth {
    fn from(health: EventDomainHealth) -> Self {
        match health {
            EventDomainHealth::Healthy => DomainHealth::Healthy,
            EventDomainHealth::Warning(msg) => DomainHealth::Warning(msg),
            EventDomainHealth::Error(msg) => DomainHealth::Error(msg),
            EventDomainHealth::Unknown => DomainHealth::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogInfo {
    pub id: String,
    pub title: String,
    pub model: String,
    pub message_count: usize,
    pub last_active: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInfo {
    pub timestamp: String,
    pub domain: String,
    pub event_type: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyInfo {
    pub name: String,
    pub domain: String,
    pub rules_count: usize,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub nats_connected: bool,
    pub total_events: u64,
    pub uptime_seconds: f64,
    pub memory_usage_mb: f32,
}

/// Launches dashboard with event sourcing connection
pub async fn launch_dashboard_with_events(
    renderer_manager: &crate::renderer::RendererManager,
    nats_client: Option<Client>,
) -> Result<String> {
    // If NATS client provided, start event processor
    if let Some(client) = nats_client {
        let (event_tx, _event_rx) = mpsc::channel(1000);
        let processor = DashboardEventProcessor::new(client, event_tx).await?;
        
        // Start processing events in background
        tokio::spawn(async move {
            if let Err(e) = processor.start().await {
                tracing::error!("Dashboard event processor error: {}", e);
            }
        });
        
        // TODO: Connect event_rx to dashboard updates via IPC
    }
    
    let dashboard_data = DashboardData::from_projection(&DashboardProjection::new());
    
    let request = RenderRequest {
        id: Uuid::new_v4().to_string(),
        renderer: RendererType::Iced,
        title: "Alchemist - Domain Dashboard".to_string(),
        data: RenderData::Dashboard(dashboard_data),
        config: RenderConfig {
            width: 1400,
            height: 900,
            position: Some((100, 100)),
            fullscreen: false,
            resizable: true,
            always_on_top: false,
        },
    };
    
    renderer_manager.spawn(request).await
}

impl DashboardData {
    /// Create dashboard data from event projection
    pub fn from_projection(projection: &DashboardProjection) -> Self {
        let memory_usage_mb = crate::system_monitor::get_memory_usage_mb();
        let domains: Vec<DomainInfo> = projection.domains.values()
            .map(|status| {
                let health: DomainHealth = status.health.clone().into();
                let healthy = matches!(health, DomainHealth::Healthy);
                DomainInfo {
                    name: status.name.clone(),
                    description: Self::get_domain_description(&status.name),
                    enabled: status.event_count > 0,
                    health,
                    healthy,
                    event_count: status.event_count,
                    dependencies: Self::get_domain_dependencies(&status.name),
                }
            })
            .collect();
        
        let active_dialogs: Vec<DialogInfo> = projection.active_dialogs.iter()
            .map(|dialog| DialogInfo {
                id: dialog.id.clone(),
                title: dialog.title.clone(),
                model: dialog.model.clone(),
                message_count: dialog.message_count,
                last_active: format!("{} ago", Self::format_time_ago(dialog.last_activity)),
            })
            .collect();
        
        let recent_events: Vec<EventInfo> = projection.recent_events.iter()
            .rev()
            .take(20)
            .map(|event| EventInfo {
                timestamp: event.timestamp.format("%H:%M:%S").to_string(),
                domain: event.domain.clone(),
                event_type: event.event_type.clone(),
                summary: event.description.clone(),
            })
            .collect();
        
        Self {
            domains,
            active_dialogs,
            recent_events,
            system_status: SystemStatus {
                nats_connected: true, // TODO: Get from actual connection
                total_events: projection.metrics.total_events,
                uptime_seconds: 0.0, // TODO: Track actual uptime
                memory_usage_mb,
            },
            active_policies: Vec::new(), // TODO: Get from policy domain
        }
    }
    
    fn get_domain_description(name: &str) -> String {
        match name {
            "workflow" => "Business process execution and state machines",
            "agent" => "AI provider integration and semantic search",
            "document" => "Document lifecycle and version control",
            "policy" => "Business rule enforcement",
            "graph" => "Core graph operations and spatial positioning",
            "dialog" => "AI conversation management",
            "identity" => "Identity and access management",
            "organization" => "Organizational structure and roles",
            "person" => "Person profiles and relationships",
            "location" => "Spatial and geographic data",
            _ => "Domain module",
        }.to_string()
    }
    
    fn get_domain_dependencies(name: &str) -> Vec<String> {
        match name {
            "workflow" => vec!["graph".to_string()],
            "agent" => vec!["graph".to_string()],
            "dialog" => vec!["agent".to_string()],
            _ => vec![],
        }
    }
    
    fn format_time_ago(time: chrono::DateTime<chrono::Utc>) -> String {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(time);
        
        if duration.num_seconds() < 60 {
            format!("{} seconds", duration.num_seconds())
        } else if duration.num_minutes() < 60 {
            format!("{} minutes", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{} hours", duration.num_hours())
        } else {
            format!("{} days", duration.num_days())
        }
    }
    
    pub fn example() -> Self {
        Self {
            domains: vec![
                DomainInfo {
                    name: "workflow".to_string(),
                    description: "Business process execution".to_string(),
                    enabled: true,
                    health: DomainHealth::Healthy,
                    healthy: true,
                    event_count: 1234,
                    dependencies: vec!["graph".to_string()],
                },
                DomainInfo {
                    name: "agent".to_string(),
                    description: "AI provider integration".to_string(),
                    enabled: true,
                    health: DomainHealth::Warning("Rate limit approaching".to_string()),
                    healthy: false,
                    event_count: 567,
                    dependencies: vec!["graph".to_string()],
                },
                DomainInfo {
                    name: "document".to_string(),
                    description: "Document lifecycle management".to_string(),
                    enabled: true,
                    health: DomainHealth::Healthy,
                    healthy: true,
                    event_count: 890,
                    dependencies: vec![],
                },
                DomainInfo {
                    name: "policy".to_string(),
                    description: "Business rule enforcement".to_string(),
                    enabled: true,
                    health: DomainHealth::Healthy,
                    healthy: true,
                    event_count: 234,
                    dependencies: vec![],
                },
                DomainInfo {
                    name: "graph".to_string(),
                    description: "Core graph operations".to_string(),
                    enabled: true,
                    health: DomainHealth::Healthy,
                    healthy: true,
                    event_count: 3456,
                    dependencies: vec![],
                },
            ],
            active_dialogs: vec![
                DialogInfo {
                    id: "d1".to_string(),
                    title: "System Architecture Discussion".to_string(),
                    model: "claude-3".to_string(),
                    message_count: 15,
                    last_active: "2 minutes ago".to_string(),
                },
                DialogInfo {
                    id: "d2".to_string(),
                    title: "Workflow Design".to_string(),
                    model: "gpt-4".to_string(),
                    message_count: 8,
                    last_active: "1 hour ago".to_string(),
                },
            ],
            recent_events: vec![
                EventInfo {
                    timestamp: "10:45:23".to_string(),
                    domain: "workflow".to_string(),
                    event_type: "WorkflowCompleted".to_string(),
                    summary: "Order processing workflow completed successfully".to_string(),
                },
                EventInfo {
                    timestamp: "10:44:15".to_string(),
                    domain: "agent".to_string(),
                    event_type: "QueryExecuted".to_string(),
                    summary: "AI query: 'Analyze customer sentiment'".to_string(),
                },
                EventInfo {
                    timestamp: "10:43:02".to_string(),
                    domain: "document".to_string(),
                    event_type: "DocumentCreated".to_string(),
                    summary: "New report: Q4 Analysis.pdf".to_string(),
                },
            ],
            system_status: SystemStatus {
                nats_connected: true,
                total_events: 12345,
                uptime_seconds: 314400.0,  // 3 days, 14 hours
                memory_usage_mb: 45.2,
            },
            active_policies: vec![
                PolicyInfo {
                    name: "api-rate-limit".to_string(),
                    domain: "agent".to_string(),
                    rules_count: 3,
                    enabled: true,
                },
                PolicyInfo {
                    name: "document-approval".to_string(),
                    domain: "document".to_string(),
                    rules_count: 5,
                    enabled: true,
                },
            ],
        }
    }
}

pub async fn launch_dashboard(
    renderer_manager: &crate::renderer::RendererManager,
) -> Result<String> {
    // Create dashboard data from current projection state
    let dashboard_data = DashboardData::example(); // TODO: Connect to real projection
    
    let request = RenderRequest {
        id: Uuid::new_v4().to_string(),
        renderer: RendererType::Iced,
        title: "Alchemist - Domain Dashboard".to_string(),
        data: RenderData::Dashboard(dashboard_data.clone()),
        config: RenderConfig {
            width: 1400,
            height: 900,
            position: Some((100, 100)),
            fullscreen: false,
            resizable: true,
            always_on_top: false,
        },
    };
    
    renderer_manager.spawn(request).await
}

/// Launch dashboard in the same process (for development)
pub async fn launch_dashboard_inprocess(
    initial_data: DashboardData,
    data_receiver: mpsc::Receiver<DashboardData>,
) -> Result<()> {
    crate::dashboard_window::run_dashboard_window(initial_data, data_receiver).await
}