//! NATS bridge for renderer communication
//!
//! This module provides event-based communication between UI components
//! and renderers using NATS subjects.

use anyhow::Result;
use async_nats::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{info, error};
use std::sync::Arc;
use dashmap::DashMap;
use futures::StreamExt;

/// NATS subjects for renderer communication
pub mod subjects {
    /// Commands sent to renderers
    pub const RENDERER_COMMAND: &str = "cim.renderer.command";
    /// Events from renderers
    pub const RENDERER_EVENT: &str = "cim.renderer.event";
    /// Dialog-specific commands
    pub const DIALOG_COMMAND: &str = "cim.dialog.command";
    /// Dialog-specific events
    pub const DIALOG_EVENT: &str = "cim.dialog.event";
    /// Dashboard updates
    pub const DASHBOARD_UPDATE: &str = "cim.dashboard.update";
    /// System status updates
    pub const SYSTEM_STATUS: &str = "cim.system.status";
}

/// Event that can be published to NATS
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NatsRendererEvent {
    /// UI component registered
    ComponentRegistered {
        component_id: String,
        component_type: ComponentType,
    },
    /// UI component unregistered
    ComponentUnregistered {
        component_id: String,
    },
    /// Dialog message sent
    DialogMessage {
        conversation_id: String,
        role: String,
        content: String,
    },
    /// Dialog response streaming
    DialogStreaming {
        conversation_id: String,
        token: String,
    },
    /// Dialog response complete
    DialogComplete {
        conversation_id: String,
    },
    /// Dashboard data updated
    DashboardUpdate {
        data: crate::dashboard::DashboardData,
    },
    /// System status changed
    SystemStatusChanged {
        connected: bool,
        nats_status: String,
        active_components: Vec<String>,
    },
    /// Error occurred
    Error {
        component_id: String,
        error: String,
    },
}

/// Type of UI component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    Dashboard,
    Dialog,
    Launcher,
    Monitor,
    Editor,
}

/// Bridge between NATS and renderer API
pub struct RendererNatsBridge {
    client: Client,
    renderer_api: Arc<crate::renderer_api::RendererApi>,
    event_handlers: DashMap<String, mpsc::Sender<NatsRendererEvent>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl RendererNatsBridge {
    /// Create new bridge
    pub async fn new(client: Client) -> Result<Self> {
        let renderer_api = Arc::new(crate::renderer_api::RendererApi::new());
        
        Ok(Self {
            client,
            renderer_api,
            event_handlers: DashMap::new(),
            shutdown_tx: None,
        })
    }
    
    /// Start the bridge
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting Renderer NATS Bridge");
        
        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);
        
        // Subscribe to renderer events
        let mut renderer_sub = self.client.subscribe(subjects::RENDERER_EVENT).await?;
        let mut dialog_sub = self.client.subscribe(subjects::DIALOG_EVENT).await?;
        
        // Clone for async tasks
        let client = self.client.clone();
        let renderer_api = self.renderer_api.clone();
        let event_handlers = self.event_handlers.clone();
        
        // Spawn event processing task
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle renderer events
                    Some(msg) = renderer_sub.next() => {
                        if let Ok(event) = serde_json::from_slice::<NatsRendererEvent>(&msg.payload) {
                            Self::handle_nats_event(&client, &renderer_api, &event_handlers, event).await;
                        }
                    }
                    
                    // Handle dialog events
                    Some(msg) = dialog_sub.next() => {
                        if let Ok(event) = serde_json::from_slice::<NatsRendererEvent>(&msg.payload) {
                            Self::handle_nats_event(&client, &renderer_api, &event_handlers, event).await;
                        }
                    }
                    
                    // Shutdown signal
                    _ = shutdown_rx.recv() => {
                        info!("Renderer NATS Bridge shutting down");
                        break;
                    }
                }
            }
        });
        
        // Publish initial status
        self.publish_system_status().await?;
        
        Ok(())
    }
    
    /// Register a UI component
    pub async fn register_component(
        &self,
        component_id: String,
        component_type: ComponentType,
    ) -> mpsc::Receiver<NatsRendererEvent> {
        let (tx, rx) = mpsc::channel(100);
        self.event_handlers.insert(component_id.clone(), tx);
        
        // Publish registration event
        let event = NatsRendererEvent::ComponentRegistered {
            component_id,
            component_type,
        };
        
        if let Err(e) = self.publish_event(subjects::RENDERER_EVENT, &event).await {
            error!("Failed to publish component registration: {}", e);
        }
        
        rx
    }
    
    /// Unregister a UI component
    pub async fn unregister_component(&self, component_id: &str) {
        self.event_handlers.remove(component_id);
        
        // Publish unregistration event
        let event = NatsRendererEvent::ComponentUnregistered {
            component_id: component_id.to_string(),
        };
        
        if let Err(e) = self.publish_event(subjects::RENDERER_EVENT, &event).await {
            error!("Failed to publish component unregistration: {}", e);
        }
    }
    
    /// Send dialog message
    pub async fn send_dialog_message(
        &self,
        conversation_id: String,
        role: String,
        content: String,
    ) -> Result<()> {
        let event = NatsRendererEvent::DialogMessage {
            conversation_id,
            role,
            content,
        };
        
        self.publish_event(subjects::DIALOG_EVENT, &event).await
    }
    
    /// Update dashboard data
    pub async fn update_dashboard(&self, data: crate::dashboard::DashboardData) -> Result<()> {
        let event = NatsRendererEvent::DashboardUpdate { data };
        self.publish_event(subjects::DASHBOARD_UPDATE, &event).await
    }
    
    /// Publish event to NATS
    async fn publish_event(&self, subject: &str, event: &NatsRendererEvent) -> Result<()> {
        let payload = serde_json::to_vec(event)?;
        self.client.publish(subject.to_string(), payload.into()).await?;
        Ok(())
    }
    
    /// Handle incoming NATS event
    async fn handle_nats_event(
        client: &Client,
        renderer_api: &Arc<crate::renderer_api::RendererApi>,
        event_handlers: &DashMap<String, mpsc::Sender<NatsRendererEvent>>,
        event: NatsRendererEvent,
    ) {
        match &event {
            NatsRendererEvent::DialogMessage { conversation_id, role, content } => {
                // Convert to renderer command
                let cmd = crate::renderer_api::DialogCommand::AddMessage {
                    role: role.clone(),
                    content: content.clone(),
                };
                
                // Send to relevant dialog windows
                for handler in event_handlers.iter() {
                    if handler.key().starts_with("dialog-") {
                        let _ = handler.value().send(event.clone()).await;
                    }
                }
            }
            
            NatsRendererEvent::DashboardUpdate { data } => {
                // Send to all dashboard components
                for handler in event_handlers.iter() {
                    if handler.key().starts_with("dashboard-") {
                        let _ = handler.value().send(event.clone()).await;
                    }
                }
            }
            
            NatsRendererEvent::SystemStatusChanged { .. } => {
                // Broadcast to all components
                for handler in event_handlers.iter() {
                    let _ = handler.value().send(event.clone()).await;
                }
            }
            
            _ => {
                // Forward to specific handlers
                for handler in event_handlers.iter() {
                    let _ = handler.value().send(event.clone()).await;
                }
            }
        }
    }
    
    /// Publish current system status
    async fn publish_system_status(&self) -> Result<()> {
        let active_components: Vec<String> = self.event_handlers
            .iter()
            .map(|entry| entry.key().clone())
            .collect();
        
        let event = NatsRendererEvent::SystemStatusChanged {
            connected: true,
            nats_status: "Connected".to_string(),
            active_components,
        };
        
        self.publish_event(subjects::SYSTEM_STATUS, &event).await
    }
    
    /// Get renderer API reference
    pub fn renderer_api(&self) -> Arc<crate::renderer_api::RendererApi> {
        self.renderer_api.clone()
    }
}

/// Create NATS-connected UI component
pub async fn create_nats_ui_component(
    client: Client,
    component_id: String,
    component_type: ComponentType,
) -> Result<(mpsc::Receiver<NatsRendererEvent>, Arc<RendererNatsBridge>)> {
    let mut bridge = RendererNatsBridge::new(client).await?;
    bridge.start().await?;
    
    let bridge = Arc::new(bridge);
    let rx = bridge.register_component(component_id, component_type).await;
    
    Ok((rx, bridge))
}

/// Helper to connect dialog window to NATS
pub async fn connect_dialog_to_nats(
    client: Client,
    conversation_id: String,
) -> Result<(mpsc::Sender<(String, String)>, mpsc::Receiver<String>)> {
    let (bridge_rx, bridge) = create_nats_ui_component(
        client,
        format!("dialog-{}", conversation_id),
        ComponentType::Dialog,
    ).await?;
    
    // Create channels for dialog communication
    let (prompt_tx, mut prompt_rx) = mpsc::channel::<(String, String)>(10);
    let (response_tx, response_rx) = mpsc::channel::<String>(10);
    
    // Spawn handler for dialog events
    let bridge_clone = bridge.clone();
    let conversation_id_clone = conversation_id.clone();
    tokio::spawn(async move {
        let mut rx = bridge_rx;
        let mut current_response = String::new();
        
        while let Some(event) = rx.recv().await {
            match event {
                NatsRendererEvent::DialogStreaming { conversation_id: conv_id, token } => {
                    if conv_id == conversation_id_clone {
                        current_response.push_str(&token);
                    }
                }
                
                NatsRendererEvent::DialogComplete { conversation_id: conv_id } => {
                    if conv_id == conversation_id_clone && !current_response.is_empty() {
                        let _ = response_tx.send(current_response.clone()).await;
                        current_response.clear();
                    }
                }
                
                _ => {}
            }
        }
    });
    
    // Spawn handler for outgoing prompts
    tokio::spawn(async move {
        while let Some((prompt, conv_id)) = prompt_rx.recv().await {
            let _ = bridge.send_dialog_message(
                conv_id,
                "user".to_string(),
                prompt,
            ).await;
        }
    });
    
    Ok((prompt_tx, response_rx))
}