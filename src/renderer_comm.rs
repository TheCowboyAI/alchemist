//! NATS-based communication for renderer processes
//!
//! This module provides communication between the main Alchemist process
//! and renderer processes using NATS messaging.

use anyhow::{Result, Context};
use async_nats::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{info, debug, warn, error};
use std::sync::Arc;
use dashmap::DashMap;
use futures::StreamExt;

use crate::renderer_api::{RendererCommand, RendererEvent};
use crate::renderer_events::{EventBuilder, ShellToRendererEvent, RendererToShellEvent};

/// NATS subjects for renderer communication
pub mod subjects {
    /// Commands sent to renderers
    pub const RENDERER_COMMAND_PREFIX: &str = "alchemist.renderer.cmd";
    /// Events from renderers
    pub const RENDERER_EVENT_PREFIX: &str = "alchemist.renderer.event";
    /// Renderer registration
    pub const RENDERER_REGISTER: &str = "alchemist.renderer.register";
    /// Renderer heartbeat
    pub const RENDERER_HEARTBEAT: &str = "alchemist.renderer.heartbeat";
}

/// Message wrapper for NATS communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsMessage<T> {
    /// Unique message ID
    pub id: String,
    /// Renderer ID
    pub renderer_id: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The actual payload
    pub payload: T,
}

/// Renderer registration message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererRegistration {
    pub renderer_id: String,
    pub renderer_type: crate::renderer::RendererType,
    pub title: String,
    pub pid: u32,
}

/// NATS-based renderer communication manager
pub struct RendererComm {
    /// NATS client
    client: Client,
    /// Active renderer subscriptions
    subscriptions: Arc<DashMap<String, async_nats::Subscriber>>,
    /// Event sender for forwarding renderer events
    event_sender: mpsc::Sender<RendererEvent>,
}

impl RendererComm {
    /// Create a new renderer communication manager
    pub async fn new(client: Client, event_sender: mpsc::Sender<RendererEvent>) -> Result<Self> {
        Ok(Self {
            client,
            subscriptions: Arc::new(DashMap::new()),
            event_sender,
        })
    }
    
    /// Start listening for renderer events
    pub async fn start(self: Arc<Self>) -> Result<()> {
        // Subscribe to renderer registration
        let mut register_sub = self.client
            .subscribe(subjects::RENDERER_REGISTER)
            .await?;
        
        // Subscribe to all renderer events
        let event_subject = format!("{}.*", subjects::RENDERER_EVENT_PREFIX);
        let mut event_sub = self.client
            .subscribe(event_subject)
            .await?;
        
        // Handle registration messages
        let comm = self.clone();
        tokio::spawn(async move {
            while let Some(msg) = register_sub.next().await {
                if let Ok(registration) = serde_json::from_slice::<RendererRegistration>(&msg.payload) {
                    info!("Renderer registered: {} ({})", registration.renderer_id, registration.title);
                    
                    // Create a subscription for this specific renderer's events
                    let renderer_subject = format!("{}.{}", subjects::RENDERER_EVENT_PREFIX, registration.renderer_id);
                    if let Ok(sub) = comm.client.subscribe(renderer_subject).await {
                        comm.subscriptions.insert(registration.renderer_id.clone(), sub);
                        debug!("Created subscription for renderer {}", registration.renderer_id);
                    }
                    
                    // Send acknowledgment
                    if let Some(reply) = msg.reply {
                        let _ = comm.client.publish(reply, "ok".into()).await;
                    }
                }
            }
        });
        
        // Handle renderer events
        let comm = self.clone();
        tokio::spawn(async move {
            while let Some(msg) = event_sub.next().await {
                if let Ok(nats_msg) = serde_json::from_slice::<NatsMessage<RendererEvent>>(&msg.payload) {
                    debug!("Received event from renderer {}: {:?}", nats_msg.renderer_id, nats_msg.payload);
                    
                    // Forward to event channel
                    if let Err(e) = comm.event_sender.send(nats_msg.payload).await {
                        error!("Failed to forward renderer event: {}", e);
                    }
                }
            }
        });
        
        info!("Renderer communication started");
        Ok(())
    }
    
    /// Send a command to a specific renderer
    pub async fn send_command(&self, renderer_id: &str, command: RendererCommand) -> Result<()> {
        let subject = format!("{}.{}", subjects::RENDERER_COMMAND_PREFIX, renderer_id);
        
        let nats_msg = NatsMessage {
            id: uuid::Uuid::new_v4().to_string(),
            renderer_id: renderer_id.to_string(),
            timestamp: chrono::Utc::now(),
            payload: command,
        };
        
        let payload = serde_json::to_vec(&nats_msg)?;
        
        self.client
            .publish(subject, payload.into())
            .await
            .context("Failed to send command to renderer")?;
        
        debug!("Sent command to renderer {}", renderer_id);
        Ok(())
    }
    
    /// Broadcast a command to all renderers
    pub async fn broadcast_command(&self, command: RendererCommand) -> Result<()> {
        let subject = format!("{}.broadcast", subjects::RENDERER_COMMAND_PREFIX);
        
        let nats_msg = NatsMessage {
            id: uuid::Uuid::new_v4().to_string(),
            renderer_id: "broadcast".to_string(),
            timestamp: chrono::Utc::now(),
            payload: command,
        };
        
        let payload = serde_json::to_vec(&nats_msg)?;
        
        self.client
            .publish(subject, payload.into())
            .await
            .context("Failed to broadcast command")?;
        
        debug!("Broadcast command to all renderers");
        Ok(())
    }
    
    /// Check if a renderer is alive
    pub async fn ping_renderer(&self, renderer_id: &str) -> Result<bool> {
        let subject = format!("{}.{}", subjects::RENDERER_HEARTBEAT, renderer_id);
        
        match self.client
            .request(subject, "ping".into())
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    /// Send a dialog message added event to a renderer
    pub async fn send_dialog_message_added(
        &self,
        renderer_id: String,
        dialog_id: String,
        role: String,
        content: String,
    ) -> Result<()> {
        let event = EventBuilder::dialog_message_added(renderer_id.clone(), dialog_id, role, content);
        let subject = format!("{}.{}", subjects::RENDERER_COMMAND_PREFIX, renderer_id);
        
        let nats_msg = NatsMessage {
            id: EventBuilder::new_id(),
            renderer_id,
            timestamp: EventBuilder::now(),
            payload: event,
        };
        
        let payload = serde_json::to_vec(&nats_msg)?;
        
        self.client
            .publish(subject, payload.into())
            .await
            .context("Failed to send dialog message added event")?;
        
        Ok(())
    }
    
    /// Remove a subscription for a disconnected renderer
    pub async fn remove_subscription(&self, renderer_id: &str) -> Result<()> {
        if let Some((_, mut sub)) = self.subscriptions.remove(renderer_id) {
            sub.unsubscribe().await?;
            info!("Removed subscription for renderer {}", renderer_id);
        }
        Ok(())
    }
    
    /// Get list of active renderer IDs
    pub fn get_active_renderers(&self) -> Vec<String> {
        self.subscriptions.iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
    
    /// Cleanup disconnected renderers
    pub async fn cleanup_disconnected(&self) -> Result<()> {
        let renderer_ids = self.get_active_renderers();
        let mut disconnected = Vec::new();
        
        for id in renderer_ids {
            if !self.ping_renderer(&id).await? {
                disconnected.push(id);
            }
        }
        
        for id in disconnected {
            self.remove_subscription(&id).await?;
        }
        
        if !disconnected.is_empty() {
            info!("Cleaned up {} disconnected renderers", disconnected.len());
        }
        
        Ok(())
    }
}

/// Renderer-side NATS communication
pub struct RendererClient {
    /// NATS client
    client: Client,
    /// Renderer ID
    renderer_id: String,
    /// Command receiver
    command_receiver: mpsc::Receiver<RendererCommand>,
    /// Event sender for sending events to main process
    event_sender: mpsc::Sender<RendererEvent>,
    /// Event receiver for internal use
    event_receiver: Option<mpsc::Receiver<RendererEvent>>,
}

impl RendererClient {
    /// Create a new renderer client
    pub async fn new(
        client: Client,
        renderer_id: String,
        renderer_type: crate::renderer::RendererType,
        title: String,
    ) -> Result<Self> {
        // Register with main process
        let registration = RendererRegistration {
            renderer_id: renderer_id.clone(),
            renderer_type,
            title,
            pid: std::process::id(),
        };
        
        let payload = serde_json::to_vec(&registration)?;
        
        // Send registration and wait for acknowledgment
        let response = client
            .request(subjects::RENDERER_REGISTER, payload.into())
            .await
            .context("Failed to register renderer")?;
        
        if response.payload != "ok" {
            return Err(anyhow::anyhow!("Registration failed"));
        }
        
        info!("Renderer {} registered successfully", renderer_id);
        
        let (cmd_tx, cmd_rx) = mpsc::channel(100);
        let (event_tx, event_rx) = mpsc::channel(100);
        
        let client_obj = Self {
            client,
            renderer_id,
            command_receiver: cmd_rx,
            event_sender: event_tx,
            event_receiver: Some(event_rx),
        };
        
        Ok(client_obj)
    }
    
    /// Start the renderer client
    pub async fn start(mut self) -> Result<(mpsc::Receiver<RendererCommand>, mpsc::Sender<RendererEvent>)> {
        // Create channels for external use
        let (cmd_tx, cmd_rx) = mpsc::channel(100);
        let (event_tx, event_rx) = mpsc::channel(100);
        
        // Subscribe to commands for this renderer
        let cmd_subject = format!("{}.{}", subjects::RENDERER_COMMAND_PREFIX, self.renderer_id);
        let mut cmd_sub = self.client.subscribe(cmd_subject).await?;
        
        // Also subscribe to broadcast commands
        let broadcast_subject = format!("{}.broadcast", subjects::RENDERER_COMMAND_PREFIX);
        let mut broadcast_sub = self.client.subscribe(broadcast_subject).await?;
        
        // Subscribe to heartbeat requests
        let heartbeat_subject = format!("{}.{}", subjects::RENDERER_HEARTBEAT, self.renderer_id);
        let mut heartbeat_sub = self.client.subscribe(heartbeat_subject).await?;
        
        // Handle commands
        let client = self.client.clone();
        let renderer_id = self.renderer_id.clone();
        let cmd_tx_for_handler = cmd_tx.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(msg) = cmd_sub.next() => {
                        if let Ok(nats_msg) = serde_json::from_slice::<NatsMessage<RendererCommand>>(&msg.payload) {
                            debug!("Received command: {:?}", nats_msg.payload);
                            // Forward to local handler
                            if let Err(e) = cmd_tx_for_handler.send(nats_msg.payload).await {
                                error!("Failed to forward command to handler: {}", e);
                            }
                        }
                    }
                    Some(msg) = broadcast_sub.next() => {
                        if let Ok(nats_msg) = serde_json::from_slice::<NatsMessage<RendererCommand>>(&msg.payload) {
                            debug!("Received broadcast command: {:?}", nats_msg.payload);
                            // Forward to local handler
                            if let Err(e) = cmd_tx_for_handler.send(nats_msg.payload).await {
                                error!("Failed to forward broadcast command to handler: {}", e);
                            }
                        }
                    }
                    Some(msg) = heartbeat_sub.next() => {
                        if let Some(reply) = msg.reply {
                            let _ = client.publish(reply, "pong".into()).await;
                        }
                    }
                }
            }
        });
        
        // Handle outgoing events - forward from event_tx to NATS
        let client_for_events = self.client.clone();
        let renderer_id_for_events = self.renderer_id.clone();
        let event_tx_clone = event_tx.clone();
        tokio::spawn(async move {
            let mut event_rx = event_rx;
            while let Some(event) = event_rx.recv().await {
                let subject = format!("{}.{}", subjects::RENDERER_EVENT_PREFIX, renderer_id_for_events);
                
                let nats_msg = NatsMessage {
                    id: EventBuilder::new_id(),
                    renderer_id: renderer_id_for_events.clone(),
                    timestamp: EventBuilder::now(),
                    payload: event,
                };
                
                if let Ok(payload) = serde_json::to_vec(&nats_msg) {
                    if let Err(e) = client_for_events.publish(subject.clone(), payload.into()).await {
                        error!("Failed to publish event to NATS: {}", e);
                    }
                }
            }
        });
        
        Ok((cmd_rx, event_tx))
    }
    
    /// Send a dialog message submitted event
    pub async fn send_dialog_message_submitted(
        &self,
        dialog_id: String,
        content: String,
    ) -> Result<()> {
        let event = EventBuilder::dialog_message_submitted(
            self.renderer_id.clone(),
            dialog_id,
            content,
        );
        
        let subject = format!("{}.{}", subjects::RENDERER_EVENT_PREFIX, self.renderer_id);
        
        let nats_msg = NatsMessage {
            id: EventBuilder::new_id(),
            renderer_id: self.renderer_id.clone(),
            timestamp: EventBuilder::now(),
            payload: event,
        };
        
        if let Ok(payload) = serde_json::to_vec(&nats_msg) {
            self.client
                .publish(subject, payload.into())
                .await
                .context("Failed to send dialog message submitted event")?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_nats_message_serialization() {
        use crate::renderer_api::RendererCommand;
        
        let msg = NatsMessage {
            id: "test-123".to_string(),
            renderer_id: "renderer-456".to_string(),
            timestamp: chrono::Utc::now(),
            payload: RendererCommand::Close,
        };
        
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: NatsMessage<RendererCommand> = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.id, msg.id);
        assert_eq!(parsed.renderer_id, msg.renderer_id);
    }
}