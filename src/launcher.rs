//! Unified launcher for Alchemist with conversation and document management

use anyhow::Result;
use iced::{Task, Element, Theme, window, Length, Alignment};
use iced::widget::{column, container, row, text, button, text_input, scrollable, Space, pick_list};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Message {
    // Launcher actions
    LaunchDashboard,
    LaunchDialogWindow,
    LaunchNatsMonitor,
    LaunchWorkflowEditor,
    LaunchEventVisualizer,
    LaunchPerformanceMonitor,
    LaunchDeploymentManager,
    
    // Dialog actions
    InputChanged(String),
    SendMessage,
    ResponseReceived(String),
    SelectModel(String),
    
    // Conversation management
    NewConversation,
    SelectConversation(String),
    DeleteConversation,
    ExportConversation,
    
    // Document management
    OpenDocument(String),
    NewDocument,
    SaveDocument,
    CloseDocument,
    
    // UI actions
    TogglePanel(Panel),
    Refresh,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    Conversations,
    Documents,
    Settings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub messages: Vec<ChatMessage>,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub doc_type: DocumentType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    Markdown,
    Code,
    Config,
    Workflow,
}

pub struct AlchemistLauncher {
    // Dialog state
    current_conversation: Option<Conversation>,
    conversations: Vec<Conversation>,
    input_value: String,
    available_models: Vec<String>,
    current_model: String,
    is_waiting: bool,
    
    // Document state
    documents: Vec<Document>,
    current_document: Option<Document>,
    
    // UI state
    active_panel: Panel,
    
    // Communication
    ai_sender: Option<mpsc::Sender<(String, String)>>, // (prompt, conversation_id)
    response_receiver: Option<mpsc::Receiver<String>>,
    nats_connected: bool,
}

impl AlchemistLauncher {
    pub fn new() -> (Self, Task<Message>) {
        // Create channels for AI communication
        let (ai_tx, ai_rx) = mpsc::channel(10);
        let (response_tx, response_rx) = mpsc::channel(10);
        
        // Spawn AI handler
        tokio::spawn(async move {
            let mut rx = ai_rx;
            while let Some((prompt, _conv_id)) = rx.recv().await {
                // Simulate AI processing
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                
                // Send mock response
                let response = format!(
                    "I understand you're asking about: '{}'. \
                    This is a demo response from the Alchemist AI system. \
                    In a full implementation, this would connect to real AI models.",
                    prompt
                );
                let _ = response_tx.send(response).await;
            }
        });
        
        // Load saved conversations (mock data for demo)
        let conversations = vec![
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
                        content: "The CIM domains should follow DDD principles...".to_string(),
                        timestamp: Utc::now() - chrono::Duration::hours(2),
                    },
                ],
                model: "claude-3-sonnet".to_string(),
            },
            Conversation {
                id: "conv-2".to_string(),
                title: "Workflow Design Patterns".to_string(),
                created: Utc::now() - chrono::Duration::days(1),
                last_modified: Utc::now() - chrono::Duration::hours(5),
                messages: vec![],
                model: "gpt-4".to_string(),
            },
        ];
        
        // Load documents (mock data)
        let documents = vec![
            Document {
                id: "doc-1".to_string(),
                title: "readme.md".to_string(),
                content: "# Alchemist\n\nCIM Control System".to_string(),
                created: Utc::now() - chrono::Duration::days(7),
                last_modified: Utc::now() - chrono::Duration::days(1),
                doc_type: DocumentType::Markdown,
            },
            Document {
                id: "doc-2".to_string(),
                title: "workflow.yaml".to_string(),
                content: "name: example\nsteps:\n  - action: deploy".to_string(),
                created: Utc::now() - chrono::Duration::days(3),
                last_modified: Utc::now() - chrono::Duration::hours(12),
                doc_type: DocumentType::Workflow,
            },
        ];
        
        (
            AlchemistLauncher {
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
                ai_sender: Some(ai_tx),
                response_receiver: Some(response_rx),
                nats_connected: false,
            },
            Task::none()
        )
    }

    pub fn title(&self) -> String {
        "Alchemist Launcher".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Launcher actions
            Message::LaunchDashboard => {
                println!("Launching dashboard...");
                tokio::spawn(async {
                    match crate::dashboard_minimal::run_minimal_dashboard().await {
                        Ok(_) => println!("Dashboard closed"),
                        Err(e) => eprintln!("Dashboard error: {}", e),
                    }
                });
                Task::none()
            }
            Message::LaunchDialogWindow => {
                println!("Launching dialog window...");
                tokio::spawn(async {
                    match crate::dialog_window_minimal::run_dialog_window(
                        "Standalone Dialog".to_string()
                    ).await {
                        Ok(_) => println!("Dialog window closed"),
                        Err(e) => eprintln!("Dialog window error: {}", e),
                    }
                });
                Task::none()
            }
            Message::LaunchNatsMonitor => {
                println!("Launching NATS flow visualizer...");
                tokio::spawn(async {
                    match crate::nats_flow_visualizer::run_nats_flow_visualizer().await {
                        Ok(_) => println!("NATS flow visualizer closed"),
                        Err(e) => eprintln!("NATS flow visualizer error: {}", e),
                    }
                });
                Task::none()
            }
            Message::LaunchWorkflowEditor => {
                println!("Launching workflow editor...");
                tokio::spawn(async {
                    match crate::workflow_editor::run_workflow_editor().await {
                        Ok(_) => println!("Workflow editor closed"),
                        Err(e) => eprintln!("Workflow editor error: {}", e),
                    }
                });
                Task::none()
            }
            Message::LaunchEventVisualizer => {
                println!("Launching event visualizer...");
                tokio::spawn(async {
                    match crate::event_visualizer::run_event_visualizer().await {
                        Ok(_) => println!("Event visualizer closed"),
                        Err(e) => eprintln!("Event visualizer error: {}", e),
                    }
                });
                Task::none()
            }
            Message::LaunchPerformanceMonitor => {
                println!("Launching performance monitor...");
                tokio::spawn(async {
                    match crate::performance_monitor_ui::run_performance_monitor().await {
                        Ok(_) => println!("Performance monitor closed"),
                        Err(e) => eprintln!("Performance monitor error: {}", e),
                    }
                });
                Task::none()
            }
            Message::LaunchDeploymentManager => {
                println!("Launching deployment manager...");
                tokio::spawn(async {
                    match crate::deployment_ui::run_deployment_manager().await {
                        Ok(_) => println!("Deployment manager closed"),
                        Err(e) => eprintln!("Deployment manager error: {}", e),
                    }
                });
                Task::none()
            }
            
            // Dialog actions
            Message::InputChanged(value) => {
                self.input_value = value;
                Task::none()
            }
            Message::SendMessage => {
                if !self.input_value.trim().is_empty() && !self.is_waiting {
                    if let Some(conv) = &mut self.current_conversation {
                        // Add user message
                        conv.messages.push(ChatMessage {
                            role: "user".to_string(),
                            content: self.input_value.clone(),
                            timestamp: Utc::now(),
                        });
                        conv.last_modified = Utc::now();
                        
                        // Send to AI
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
            Message::ResponseReceived(_) => {
                // Check for actual response
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
                Task::none()
            }
            Message::SelectModel(model) => {
                self.current_model = model.clone();
                if let Some(conv) = &mut self.current_conversation {
                    conv.model = model;
                }
                Task::none()
            }
            
            // Conversation management
            Message::NewConversation => {
                let new_conv = Conversation {
                    id: format!("conv-{}", Uuid::new_v4()),
                    title: "New Conversation".to_string(),
                    created: Utc::now(),
                    last_modified: Utc::now(),
                    messages: vec![],
                    model: self.current_model.clone(),
                };
                self.conversations.push(new_conv.clone());
                self.current_conversation = Some(new_conv);
                Task::none()
            }
            Message::SelectConversation(id) => {
                self.current_conversation = self.conversations.iter()
                    .find(|c| c.id == id)
                    .cloned();
                if let Some(conv) = &self.current_conversation {
                    self.current_model = conv.model.clone();
                }
                Task::none()
            }
            Message::DeleteConversation => {
                if let Some(current) = &self.current_conversation {
                    self.conversations.retain(|c| c.id != current.id);
                    self.current_conversation = self.conversations.first().cloned();
                }
                Task::none()
            }
            Message::ExportConversation => {
                if let Some(conv) = &self.current_conversation {
                    println!("Exporting conversation: {}", conv.title);
                    // In real implementation, save to file
                }
                Task::none()
            }
            
            // Document management
            Message::OpenDocument(id) => {
                self.current_document = self.documents.iter()
                    .find(|d| d.id == id)
                    .cloned();
                Task::none()
            }
            Message::NewDocument => {
                let new_doc = Document {
                    id: format!("doc-{}", Uuid::new_v4()),
                    title: "Untitled".to_string(),
                    content: String::new(),
                    created: Utc::now(),
                    last_modified: Utc::now(),
                    doc_type: DocumentType::Markdown,
                };
                self.documents.push(new_doc.clone());
                self.current_document = Some(new_doc);
                Task::none()
            }
            Message::SaveDocument => {
                if let Some(doc) = &mut self.current_document {
                    doc.last_modified = Utc::now();
                    println!("Saving document: {}", doc.title);
                }
                Task::none()
            }
            Message::CloseDocument => {
                self.current_document = None;
                Task::none()
            }
            
            // UI actions
            Message::TogglePanel(panel) => {
                self.active_panel = panel;
                Task::none()
            }
            Message::Refresh => {
                // Check for AI responses
                if self.is_waiting {
                    return Task::done(Message::ResponseReceived("check".to_string()));
                }
                Task::none()
            }
            Message::Exit => {
                std::process::exit(0);
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let header = self.view_header();
        let launcher_buttons = self.view_launcher_buttons();
        let main_content = row![
            container(self.view_side_panel()).width(Length::Fixed(300.0)),
            container(self.view_dialog_area()).width(Length::Fill),
        ]
        .spacing(10);
        
        let content = column![
            header,
            launcher_buttons,
            container(main_content)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(container::rounded_box),
        ]
        .spacing(10);

        container(content)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_header(&self) -> Element<Message> {
        container(
            row![
                text("🧪 Alchemist Launcher").size(32),
                Space::with_width(Length::Fill),
                text(if self.nats_connected { "NATS ✓" } else { "NATS ✗" })
                    .color(if self.nats_connected { 
                        iced::Color::from_rgb(0.0, 1.0, 0.0) 
                    } else { 
                        iced::Color::from_rgb(1.0, 0.0, 0.0) 
                    }),
                button("Exit").on_press(Message::Exit),
            ]
            .spacing(20)
            .align_y(Alignment::Center)
        )
        .padding(15)
        .style(container::rounded_box)
        .into()
    }

    fn view_launcher_buttons(&self) -> Element<Message> {
        container(
            row![
                button("📊 Dashboard")
                    .on_press(Message::LaunchDashboard)
                    .style(button::primary),
                button("💬 New Dialog")
                    .on_press(Message::LaunchDialogWindow)
                    .style(button::secondary),
                button("📡 NATS Flow")
                    .on_press(Message::LaunchNatsMonitor)
                    .style(button::secondary),
                button("🔄 Workflow Editor")
                    .on_press(Message::LaunchWorkflowEditor)
                    .style(button::secondary),
                button("📊 Event Visualizer")
                    .on_press(Message::LaunchEventVisualizer)
                    .style(button::secondary),
                button("📈 Performance")
                    .on_press(Message::LaunchPerformanceMonitor)
                    .style(button::secondary),
                button("🚀 Deployments")
                    .on_press(Message::LaunchDeploymentManager)
                    .style(button::secondary),
            ]
            .spacing(10)
        )
        .padding(10)
        .style(container::rounded_box)
        .into()
    }

    fn view_side_panel(&self) -> Element<Message> {
        let panel_tabs = row![
            button(text("Conversations").size(14))
                .on_press(Message::TogglePanel(Panel::Conversations))
                .style(if self.active_panel == Panel::Conversations {
                    button::primary
                } else {
                    button::secondary
                }),
            button(text("Documents").size(14))
                .on_press(Message::TogglePanel(Panel::Documents))
                .style(if self.active_panel == Panel::Documents {
                    button::primary
                } else {
                    button::secondary
                }),
            button(text("Settings").size(14))
                .on_press(Message::TogglePanel(Panel::Settings))
                .style(if self.active_panel == Panel::Settings {
                    button::primary
                } else {
                    button::secondary
                }),
        ]
        .spacing(5);

        let panel_content = match self.active_panel {
            Panel::Conversations => self.view_conversations_panel(),
            Panel::Documents => self.view_documents_panel(),
            Panel::Settings => self.view_settings_panel(),
        };

        container(
            column![
                panel_tabs,
                container(panel_content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(container::bordered_box),
            ]
            .spacing(10)
        )
        .padding(10)
        .style(container::rounded_box)
        .into()
    }

    fn view_conversations_panel(&self) -> Element<Message> {
        let mut content = column![
            button("➕ New Conversation")
                .on_press(Message::NewConversation)
                .width(Length::Fill),
        ]
        .spacing(5);

        for conv in &self.conversations {
            let is_selected = self.current_conversation.as_ref()
                .map(|c| c.id == conv.id)
                .unwrap_or(false);
            
            content = content.push(
                button(
                    column![
                        text(&conv.title).size(14),
                        text(format!("{} messages", conv.messages.len()))
                            .size(11)
                            .color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                    ]
                    .spacing(2)
                )
                .on_press(Message::SelectConversation(conv.id.clone()))
                .style(if is_selected { button::primary } else { button::secondary })
                .width(Length::Fill)
            );
        }

        scrollable(content.padding(10))
            .height(Length::Fill)
            .into()
    }

    fn view_documents_panel(&self) -> Element<Message> {
        let mut content = column![
            button("➕ New Document")
                .on_press(Message::NewDocument)
                .width(Length::Fill),
        ]
        .spacing(5);

        for doc in &self.documents {
            let is_selected = self.current_document.as_ref()
                .map(|d| d.id == doc.id)
                .unwrap_or(false);
            
            let icon = match doc.doc_type {
                DocumentType::Markdown => "📝",
                DocumentType::Code => "💻",
                DocumentType::Config => "⚙️",
                DocumentType::Workflow => "🔄",
            };
            
            content = content.push(
                button(
                    row![
                        text(icon),
                        text(&doc.title).size(14),
                    ]
                    .spacing(5)
                )
                .on_press(Message::OpenDocument(doc.id.clone()))
                .style(if is_selected { button::primary } else { button::secondary })
                .width(Length::Fill)
            );
        }

        scrollable(content.padding(10))
            .height(Length::Fill)
            .into()
    }

    fn view_settings_panel(&self) -> Element<Message> {
        container(
            column![
                text("Settings").size(18),
                Space::with_height(Length::Fixed(20.0)),
                text("Default Model:").size(14),
                pick_list(
                    self.available_models.clone(),
                    Some(self.current_model.clone()),
                    Message::SelectModel,
                ),
                Space::with_height(Length::Fixed(20.0)),
                text("NATS URL:").size(14),
                text_input("nats://localhost:4222", "")
                    .on_input(|_| Message::Refresh),
            ]
            .spacing(10)
            .padding(10)
        )
        .into()
    }

    fn view_dialog_area(&self) -> Element<Message> {
        if let Some(conv) = &self.current_conversation {
            let header = row![
                text(&conv.title).size(20),
                Space::with_width(Length::Fill),
                pick_list(
                    self.available_models.clone(),
                    Some(conv.model.clone()),
                    Message::SelectModel,
                ).width(Length::Fixed(150.0)),
                button("📤 Export").on_press(Message::ExportConversation),
                button("🗑️ Delete").on_press(Message::DeleteConversation),
            ]
            .spacing(10)
            .align_y(Alignment::Center);

            // Messages area
            let mut messages_col = column![].spacing(10);
            
            for msg in &conv.messages {
                let is_user = msg.role == "user";
                let msg_container = container(
                    column![
                        text(if is_user { "You" } else { &conv.model })
                            .size(12)
                            .color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                        text(&msg.content).size(14),
                        text(msg.timestamp.format("%H:%M").to_string())
                            .size(10)
                            .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                    ]
                    .spacing(3)
                )
                .padding(10)
                .style(if is_user {
                    container::rounded_box
                } else {
                    container::bordered_box
                });
                
                let msg_row = if is_user {
                    row![
                        Space::with_width(Length::FillPortion(1)),
                        container(msg_container).width(Length::FillPortion(3)),
                    ]
                } else {
                    row![
                        container(msg_container).width(Length::FillPortion(3)),
                        Space::with_width(Length::FillPortion(1)),
                    ]
                };
                
                messages_col = messages_col.push(msg_row);
            }
            
            if self.is_waiting {
                messages_col = messages_col.push(
                    row![
                        container(
                            text("AI is thinking...").size(14)
                        )
                        .padding(10)
                        .style(container::bordered_box)
                        .width(Length::FillPortion(3)),
                        Space::with_width(Length::FillPortion(1)),
                    ]
                );
            }
            
            let messages_scroll = scrollable(messages_col.padding(10))
                .height(Length::Fill);

            // Input area
            let input_row = row![
                text_input("Type your message...", &self.input_value)
                    .on_input(Message::InputChanged)
                    .on_submit(Message::SendMessage)
                    .size(16),
                button("Send")
                    .on_press(Message::SendMessage)
                    .style(if self.is_waiting || self.input_value.trim().is_empty() {
                        button::secondary
                    } else {
                        button::primary
                    }),
            ]
            .spacing(10);

            container(
                column![
                    header,
                    container(messages_scroll)
                        .height(Length::Fill)
                        .style(container::bordered_box),
                    input_row,
                ]
                .spacing(10)
                .padding(10)
            )
            .into()
        } else {
            container(
                column![
                    text("No conversation selected").size(24),
                    Space::with_height(Length::Fixed(20.0)),
                    button("Create New Conversation")
                        .on_press(Message::NewConversation)
                        .style(button::primary),
                ]
                .spacing(20)
                .align_x(Alignment::Center)
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        // Poll for updates every 100ms when waiting for response
        if self.is_waiting {
            iced::time::every(std::time::Duration::from_millis(100))
                .map(|_| Message::Refresh)
        } else {
            iced::Subscription::none()
        }
    }
}

pub async fn run_launcher() -> Result<()> {
    println!("🚀 Starting Alchemist Launcher...");
    
    iced::application(
        AlchemistLauncher::title,
        AlchemistLauncher::update,
        AlchemistLauncher::view
    )
    .subscription(AlchemistLauncher::subscription)
    .window(window::Settings {
        size: iced::Size::new(1200.0, 800.0),
        position: window::Position::Centered,
        resizable: true,
        ..Default::default()
    })
    .theme(|_| Theme::Dark)
    .run_with(AlchemistLauncher::new)
    .map_err(|e| anyhow::anyhow!("Iced error: {}", e))
}