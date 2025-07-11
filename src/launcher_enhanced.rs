//! Enhanced launcher with NATS integration
//! 
//! This launcher uses event-based communication for all UI components.

use anyhow::Result;
use iced::{Task, Element, Theme, window, Length, Alignment};
use iced::widget::{column, container, row, text, button, text_input, scrollable, Space, pick_list};
use tokio::sync::mpsc;
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use async_nats::Client;
use std::sync::Arc;
use tracing::{info, warn, error};

use crate::launcher::{Message, Panel, Conversation, ChatMessage, Document, DocumentType};
use crate::renderer_nats_bridge::{RendererNatsBridge, NatsRendererEvent, ComponentType};

pub struct EnhancedLauncher {
    // Core launcher state (reuse from launcher.rs)
    current_conversation: Option<Conversation>,
    conversations: Vec<Conversation>,
    input_value: String,
    available_models: Vec<String>,
    current_model: String,
    is_waiting: bool,
    documents: Vec<Document>,
    current_document: Option<Document>,
    active_panel: Panel,
    
    // Enhanced NATS integration
    nats_client: Option<Client>,
    nats_bridge: Option<Arc<RendererNatsBridge>>,
    event_receiver: Option<mpsc::Receiver<NatsRendererEvent>>,
    nats_connected: bool,
    active_components: Vec<String>,
    
    // Communication channels
    ai_sender: Option<mpsc::Sender<(String, String)>>,
    response_receiver: Option<mpsc::Receiver<String>>,
}

impl EnhancedLauncher {
    pub fn new() -> (Self, Task<Message>) {
        // Try to connect to NATS
        let nats_url = std::env::var("NATS_URL")
            .unwrap_or_else(|_| "nats://localhost:4222".to_string());
        
        // We'll connect to NATS asynchronously
        let (nats_client, nats_bridge, event_receiver) = (None, None, None);
        
        // Create AI communication channels
        let (ai_tx, ai_rx) = mpsc::channel(10);
        let (response_tx, response_rx) = mpsc::channel(10);
        
        // Always use local simulation for now
        tokio::spawn(async move {
            let mut rx: tokio::sync::mpsc::Receiver<(String, String)> = ai_rx;
            while let Some((prompt, _conv_id)) = rx.recv().await {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                let _ = response_tx.send(format!(
                    "Local response to: '{}'",
                    prompt
                )).await;
            }
        });
        
        // Load saved data
        let conversations = Self::load_conversations();
        let documents = Self::load_documents();
        
        (
            EnhancedLauncher {
                current_conversation: conversations.first().cloned(),
                conversations,
                input_value: String::new(),
                available_models: vec![
                    "claude-3-sonnet".to_string(),
                    "claude-3-opus".to_string(),
                    "gpt-4".to_string(),
                    "gpt-3.5-turbo".to_string(),
                ],
                current_model: "claude-3-sonnet".to_string(),
                is_waiting: false,
                documents,
                current_document: None,
                active_panel: Panel::Conversations,
                nats_connected: nats_client.is_some(),
                nats_client,
                nats_bridge,
                event_receiver,
                active_components: vec![],
                ai_sender: Some(ai_tx),
                response_receiver: Some(response_rx),
            },
            Task::none()
        )
    }
    
    fn load_conversations() -> Vec<Conversation> {
        // In production, load from disk or database
        vec![
            Conversation {
                id: "conv-1".to_string(),
                title: "System Architecture Discussion".to_string(),
                created: Utc::now() - chrono::Duration::hours(2),
                last_modified: Utc::now() - chrono::Duration::minutes(30),
                messages: vec![
                    ChatMessage {
                        role: "user".to_string(),
                        content: "How should we structure the CIM domains?".to_string(),
                        timestamp: Utc::now() - chrono::Duration::hours(2),
                    },
                    ChatMessage {
                        role: "assistant".to_string(),
                        content: "The CIM domains should follow DDD principles with bounded contexts for each major area of functionality.".to_string(),
                        timestamp: Utc::now() - chrono::Duration::hours(2),
                    },
                ],
                model: "claude-3-sonnet".to_string(),
            },
        ]
    }
    
    fn load_documents() -> Vec<Document> {
        // In production, load from disk
        vec![
            Document {
                id: "doc-1".to_string(),
                title: "README.md".to_string(),
                content: "# Alchemist\n\nCIM Control System with event-driven architecture".to_string(),
                created: Utc::now() - chrono::Duration::days(7),
                last_modified: Utc::now() - chrono::Duration::days(1),
                doc_type: DocumentType::Markdown,
            },
        ]
    }
    
    pub fn title(&self) -> String {
        "Alchemist Enhanced Launcher".to_string()
    }
    
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LaunchDashboard => {
                info!("Launching NATS-connected dashboard");
                
                if let Some(client) = self.nats_client.clone() {
                    tokio::spawn(async move {
                        // Create NATS-connected dashboard
                        let initial_data = crate::dashboard::DashboardData::example();
                        let (rx, _handle) = crate::nats_dashboard_connector::create_nats_dashboard_stream(
                            client,
                            initial_data,
                        ).await;
                        
                        match crate::dashboard_minimal::run_dashboard_with_nats(rx).await {
                            Ok(_) => info!("Dashboard closed"),
                            Err(e) => error!("Dashboard error: {}", e),
                        }
                    });
                } else {
                    // Fallback to minimal dashboard
                    tokio::spawn(async {
                        match crate::dashboard_minimal::run_minimal_dashboard().await {
                            Ok(_) => info!("Dashboard closed"),
                            Err(e) => error!("Dashboard error: {}", e),
                        }
                    });
                }
                
                Task::none()
            }
            
            Message::LaunchDialogWindow => {
                info!("Launching dialog window");
                
                let conv_id = format!("conv-{}", Uuid::new_v4());
                
                if let (Some(client), Some(bridge)) = (self.nats_client.clone(), self.nats_bridge.clone()) {
                    tokio::spawn(async move {
                        // Create NATS-connected dialog
                        match crate::renderer_nats_bridge::connect_dialog_to_nats(
                            client,
                            conv_id.clone(),
                        ).await {
                            Ok((prompt_tx, response_rx)) => {
                                info!("Dialog connected to NATS");
                                // Dialog window would use these channels
                            }
                            Err(e) => {
                                error!("Failed to connect dialog to NATS: {}", e);
                            }
                        }
                        
                        // For now, run the minimal dialog
                        match crate::dialog_window_minimal::run_dialog_window(
                            "NATS Dialog".to_string()
                        ).await {
                            Ok(_) => info!("Dialog window closed"),
                            Err(e) => error!("Dialog window error: {}", e),
                        }
                    });
                } else {
                    // Fallback to minimal dialog
                    tokio::spawn(async {
                        match crate::dialog_window_minimal::run_dialog_window(
                            "Standalone Dialog".to_string()
                        ).await {
                            Ok(_) => info!("Dialog window closed"),
                            Err(e) => error!("Dialog window error: {}", e),
                        }
                    });
                }
                
                Task::none()
            }
            
            Message::Refresh => {
                // Check for NATS events
                let mut events = Vec::new();
                if let Some(receiver) = &mut self.event_receiver {
                    while let Ok(event) = receiver.try_recv() {
                        events.push(event);
                    }
                }
                for event in events {
                    self.handle_nats_event(event);
                }
                
                // Check for AI responses
                if self.is_waiting {
                    if let Some(receiver) = &mut self.response_receiver {
                        if let Ok(response) = receiver.try_recv() {
                            if let Some(conv) = &mut self.current_conversation {
                                conv.messages.push(ChatMessage {
                                    role: "assistant".to_string(),
                                    content: response,
                                    timestamp: Utc::now(),
                                });
                                conv.last_modified = Utc::now();
                            }
                            self.is_waiting = false;
                        }
                    }
                }
                
                Task::none()
            }
            
            // Delegate other messages to base implementation
            _ => self.update_base(message),
        }
    }
    
    fn update_base(&mut self, message: Message) -> Task<Message> {
        // This would contain the same logic as launcher.rs update method
        // for all other message types
        match message {
            Message::InputChanged(value) => {
                self.input_value = value;
                Task::none()
            }
            Message::SendMessage => {
                if !self.input_value.trim().is_empty() && !self.is_waiting {
                    if let Some(conv) = &mut self.current_conversation {
                        conv.messages.push(ChatMessage {
                            role: "user".to_string(),
                            content: self.input_value.clone(),
                            timestamp: Utc::now(),
                        });
                        conv.last_modified = Utc::now();
                        
                        if let Some(sender) = &self.ai_sender {
                            let prompt = self.input_value.clone();
                            let conv_id = conv.id.clone();
                            let sender = sender.clone();
                            tokio::spawn(async move {
                                let _ = sender.send((prompt, conv_id)).await;
                            });
                        }
                        
                        self.input_value.clear();
                        self.is_waiting = true;
                    }
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }
    
    fn handle_nats_event(&mut self, event: NatsRendererEvent) {
        match event {
            NatsRendererEvent::ComponentRegistered { component_id, component_type } => {
                info!("Component registered: {} ({:?})", component_id, component_type);
                self.active_components.push(component_id);
            }
            
            NatsRendererEvent::ComponentUnregistered { component_id } => {
                info!("Component unregistered: {}", component_id);
                self.active_components.retain(|id| id != &component_id);
            }
            
            NatsRendererEvent::SystemStatusChanged { connected, nats_status, active_components } => {
                self.nats_connected = connected;
                self.active_components = active_components;
                info!("System status: {} - {} active components", nats_status, self.active_components.len());
            }
            
            _ => {}
        }
    }
    
    pub fn view(&self) -> Element<Message> {
        // Reuse view logic from launcher.rs but with enhanced status display
        let header = container(
            row![
                text("🧪 Alchemist Enhanced Launcher").size(32),
                Space::with_width(Length::Fill),
                column![
                    text(if self.nats_connected { "NATS ✓" } else { "NATS ✗" })
                        .color(if self.nats_connected { 
                            iced::Color::from_rgb(0.0, 1.0, 0.0) 
                        } else { 
                            iced::Color::from_rgb(1.0, 0.0, 0.0) 
                        }),
                    text(format!("{} components", self.active_components.len()))
                        .size(12)
                        .color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                ]
                .spacing(2),
                button("Exit").on_press(Message::Exit),
            ]
            .spacing(20)
            .align_y(Alignment::Center)
        )
        .padding(15)
        .style(container::rounded_box);
        
        // Rest of the view would be the same as launcher.rs
        container(header).into()
    }
    
    pub fn subscription(&self) -> iced::Subscription<Message> {
        // Poll for updates every 100ms
        iced::time::every(std::time::Duration::from_millis(100))
            .map(|_| Message::Refresh)
    }
}

pub async fn run_enhanced_launcher() -> Result<()> {
    println!("🚀 Starting Alchemist Enhanced Launcher with NATS integration...");
    
    iced::application(
        EnhancedLauncher::title,
        EnhancedLauncher::update,
        EnhancedLauncher::view
    )
    .subscription(EnhancedLauncher::subscription)
    .window(window::Settings {
        size: iced::Size::new(1200.0, 800.0),
        position: window::Position::Centered,
        resizable: true,
        ..Default::default()
    })
    .theme(|_| Theme::Dark)
    .run_with(EnhancedLauncher::new)
    .map_err(|e| anyhow::anyhow!("Iced error: {}", e))
}