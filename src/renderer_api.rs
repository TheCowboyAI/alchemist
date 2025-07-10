//! Renderer API for communication between main process and renderer windows

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use std::sync::Arc;
use dashmap::DashMap;
use uuid::Uuid;

/// Messages that can be sent from the main process to a renderer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RendererCommand {
    /// Update data in the renderer
    UpdateData {
        data: serde_json::Value,
    },
    /// Dialog-specific commands
    DialogCommand(DialogCommand),
    /// Close the renderer window
    Close,
}

/// Dialog-specific commands
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum DialogCommand {
    /// Add a new message to the dialog
    AddMessage {
        role: String,
        content: String,
    },
    /// Update streaming response
    StreamToken {
        token: String,
    },
    /// Complete streaming response
    CompleteStream,
    /// Set loading state
    SetLoading {
        loading: bool,
    },
    /// Update system prompt
    UpdateSystemPrompt {
        prompt: String,
    },
    /// Clear all messages
    ClearMessages,
}

/// Messages that can be sent from a renderer to the main process
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RendererEvent {
    /// Window was closed
    WindowClosed {
        renderer_id: String,
    },
    /// Dialog-specific events
    DialogEvent {
        renderer_id: String,
        event: DialogEvent,
    },
    /// Request data update
    RequestData {
        renderer_id: String,
        data_type: String,
    },
    /// Error occurred
    Error {
        renderer_id: String,
        error: String,
    },
}

/// Dialog-specific events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum DialogEvent {
    /// User sent a message
    UserMessage {
        content: String,
    },
    /// User requested to clear dialog
    ClearRequested,
    /// User requested to export dialog
    ExportRequested {
        format: String,
    },
    /// User changed AI model
    ModelChanged {
        model: String,
    },
    /// User updated system prompt
    SystemPromptChanged {
        prompt: String,
    },
}

/// API endpoint for renderer communication
pub struct RendererApi {
    /// Command senders for each renderer
    command_senders: DashMap<String, mpsc::Sender<RendererCommand>>,
    /// Event receiver from all renderers
    event_receiver: Arc<tokio::sync::Mutex<mpsc::Receiver<RendererEvent>>>,
    /// Event sender (cloned for each renderer)
    event_sender: mpsc::Sender<RendererEvent>,
}

impl RendererApi {
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::channel(100);
        
        Self {
            command_senders: DashMap::new(),
            event_receiver: Arc::new(tokio::sync::Mutex::new(event_rx)),
            event_sender: event_tx,
        }
    }
    
    /// Register a new renderer
    pub fn register_renderer(&self, renderer_id: String) -> mpsc::Receiver<RendererCommand> {
        let (cmd_tx, cmd_rx) = mpsc::channel(100);
        self.command_senders.insert(renderer_id, cmd_tx);
        cmd_rx
    }
    
    /// Unregister a renderer
    pub fn unregister_renderer(&self, renderer_id: &str) {
        self.command_senders.remove(renderer_id);
    }
    
    /// Send command to a specific renderer
    pub async fn send_command(&self, renderer_id: &str, command: RendererCommand) -> Result<()> {
        if let Some(sender) = self.command_senders.get(renderer_id) {
            sender.send(command).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Renderer not found: {}", renderer_id))
        }
    }
    
    /// Send dialog command to a renderer
    pub async fn send_dialog_command(&self, renderer_id: &str, command: DialogCommand) -> Result<()> {
        self.send_command(renderer_id, RendererCommand::DialogCommand(command)).await
    }
    
    /// Broadcast command to all renderers
    pub async fn broadcast_command(&self, command: RendererCommand) -> Result<()> {
        let futures: Vec<_> = self.command_senders.iter()
            .map(|entry| {
                let sender = entry.value().clone();
                let cmd = command.clone();
                async move { sender.send(cmd).await }
            })
            .collect();
        
        futures::future::join_all(futures).await;
        Ok(())
    }
    
    /// Get event sender for renderer to send events
    pub fn get_event_sender(&self) -> mpsc::Sender<RendererEvent> {
        self.event_sender.clone()
    }
}

/// Helper to create dialog messages
pub fn create_dialog_message(role: &str, content: &str) -> crate::renderer::DialogMessage {
    crate::renderer::DialogMessage {
        role: role.to_string(),
        content: content.to_string(),
        timestamp: chrono::Utc::now(),
    }
}

/// Convert dialog manager messages to renderer messages
pub fn convert_dialog_messages(messages: &[crate::dialog::Message]) -> Vec<crate::renderer::DialogMessage> {
    messages.iter().map(|msg| {
        crate::renderer::DialogMessage {
            role: match msg.role {
                crate::dialog::MessageRole::System => "system",
                crate::dialog::MessageRole::User => "user",
                crate::dialog::MessageRole::Assistant => "assistant",
            }.to_string(),
            content: msg.content.clone(),
            timestamp: msg.timestamp,
        }
    }).collect()
}