//! Event-driven Dialog UI for AI conversations in Iced
//!
//! This implementation uses pure events for all communication,
//! integrating naturally with Iced's TEA architecture.

use alchemist::renderer::DialogMessage;
use alchemist::renderer_events::{RendererToShellEvent, ShellToRendererEvent, EventBuilder};
use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Column, Space},
    window, executor, alignment, Application, Command, Element, Length, Settings, Theme,
    subscription, Subscription,
};
use chrono::{DateTime, Utc, Local};
use tokio::sync::mpsc;
use async_nats::Client;

pub struct DialogUI {
    dialog_id: String,
    renderer_id: String,
    ai_model: String,
    messages: Vec<DialogMessage>,
    system_prompt: Option<String>,
    input_value: String,
    is_loading: bool,
    current_stream: Option<String>,
    // Event channels
    event_sender: Option<mpsc::Sender<RendererToShellEvent>>,
    event_receiver: Option<mpsc::Receiver<ShellToRendererEvent>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    // UI interactions
    InputChanged(String),
    SendMessage,
    ClearDialog,
    CopyMessage(usize),
    ModelChanged(String),
    Close,
    
    // Events from shell
    EventReceived(ShellToRendererEvent),
    Connected(mpsc::Sender<RendererToShellEvent>),
}

pub struct DialogConfig {
    pub dialog_id: String,
    pub renderer_id: String,
    pub ai_model: String,
    pub messages: Vec<DialogMessage>,
    pub system_prompt: Option<String>,
    pub nats_client: Client,
}

impl Application for DialogUI {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = DialogConfig;

    fn new(flags: DialogConfig) -> (Self, Command<Self::Message>) {
        let dialog = Self {
            dialog_id: flags.dialog_id,
            renderer_id: flags.renderer_id.clone(),
            ai_model: flags.ai_model,
            messages: flags.messages,
            system_prompt: flags.system_prompt,
            input_value: String::new(),
            is_loading: false,
            current_stream: None,
            event_sender: None,
            event_receiver: None,
        };
        
        // Initialize NATS connection
        let renderer_id = flags.renderer_id;
        let nats_client = flags.nats_client;
        
        let init_command = Command::perform(
            async move {
                // Set up event channels
                let (tx, rx) = mpsc::channel(100);
                
                // Register with shell via NATS
                let event = RendererToShellEvent::Initialized {
                    event_id: EventBuilder::new_id(),
                    timestamp: EventBuilder::now(),
                    renderer_id: renderer_id.clone(),
                    renderer_type: alchemist::renderer::RendererType::Iced,
                    pid: std::process::id(),
                };
                
                // Send initialization event
                let subject = format!("alchemist.renderer.event.{}", renderer_id);
                let payload = serde_json::to_vec(&event).unwrap();
                let _ = nats_client.publish(subject, payload.into()).await;
                
                tx
            },
            Message::Connected
        );
        
        (dialog, init_command)
    }

    fn title(&self) -> String {
        format!("AI Dialog - {} ({})", self.dialog_id, self.ai_model)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Connected(sender) => {
                self.event_sender = Some(sender);
                Command::none()
            }
            
            Message::InputChanged(value) => {
                self.input_value = value;
                Command::none()
            }
            
            Message::SendMessage => {
                if !self.input_value.trim().is_empty() && !self.is_loading {
                    // Add user message locally
                    let user_msg = DialogMessage {
                        role: "user".to_string(),
                        content: self.input_value.clone(),
                        timestamp: Utc::now(),
                    };
                    self.messages.push(user_msg);
                    
                    // Send event to shell
                    if let Some(sender) = &self.event_sender {
                        let event = EventBuilder::dialog_message_submitted(
                            self.renderer_id.clone(),
                            self.dialog_id.clone(),
                            self.input_value.clone(),
                        );
                        
                        let sender = sender.clone();
                        let cmd = Command::perform(
                            async move {
                                let _ = sender.send(event).await;
                            },
                            |_| Message::InputChanged(String::new())
                        );
                        
                        self.input_value.clear();
                        self.is_loading = true;
                        return cmd;
                    }
                }
                Command::none()
            }
            
            Message::EventReceived(event) => {
                match event {
                    ShellToRendererEvent::DialogMessageAdded { content, role, .. } => {
                        let msg = DialogMessage {
                            role,
                            content,
                            timestamp: Utc::now(),
                        };
                        self.messages.push(msg);
                        self.is_loading = false;
                    }
                    
                    ShellToRendererEvent::DialogTokenStreamed { token, .. } => {
                        if self.current_stream.is_none() {
                            // Start new streaming message
                            let msg = DialogMessage {
                                role: "assistant".to_string(),
                                content: token,
                                timestamp: Utc::now(),
                            };
                            self.messages.push(msg);
                            self.current_stream = Some(token);
                        } else if let Some(last_msg) = self.messages.last_mut() {
                            // Append to existing stream
                            last_msg.content.push_str(&token);
                        }
                    }
                    
                    ShellToRendererEvent::DialogStreamCompleted { .. } => {
                        self.current_stream = None;
                        self.is_loading = false;
                    }
                    
                    ShellToRendererEvent::DialogLoadingStateChanged { loading, .. } => {
                        self.is_loading = loading;
                    }
                    
                    ShellToRendererEvent::DialogCleared { .. } => {
                        self.messages.clear();
                    }
                    
                    _ => {}
                }
                Command::none()
            }
            
            Message::ClearDialog => {
                if let Some(sender) = &self.event_sender {
                    let event = RendererToShellEvent::DialogClearRequested {
                        event_id: EventBuilder::new_id(),
                        timestamp: EventBuilder::now(),
                        renderer_id: self.renderer_id.clone(),
                        dialog_id: self.dialog_id.clone(),
                    };
                    
                    let sender = sender.clone();
                    Command::perform(
                        async move {
                            let _ = sender.send(event).await;
                        },
                        |_| Message::ClearDialog
                    )
                } else {
                    Command::none()
                }
            }
            
            Message::ModelChanged(model) => {
                self.ai_model = model.clone();
                
                if let Some(sender) = &self.event_sender {
                    let event = RendererToShellEvent::DialogModelChanged {
                        event_id: EventBuilder::new_id(),
                        timestamp: EventBuilder::now(),
                        renderer_id: self.renderer_id.clone(),
                        dialog_id: self.dialog_id.clone(),
                        model,
                    };
                    
                    let sender = sender.clone();
                    Command::perform(
                        async move {
                            let _ = sender.send(event).await;
                        },
                        |_| Message::InputChanged(String::new())
                    )
                } else {
                    Command::none()
                }
            }
            
            Message::CopyMessage(index) => {
                if let Some(msg) = self.messages.get(index) {
                    // TODO: Implement clipboard support
                    println!("Copy message: {}", msg.content);
                }
                Command::none()
            }
            
            Message::Close => {
                if let Some(sender) = &self.event_sender {
                    let event = RendererToShellEvent::WindowClosing {
                        event_id: EventBuilder::new_id(),
                        timestamp: EventBuilder::now(),
                        renderer_id: self.renderer_id.clone(),
                    };
                    
                    let sender = sender.clone();
                    let _ = Command::perform(
                        async move {
                            let _ = sender.send(event).await;
                        },
                        |_| Message::Close
                    );
                }
                
                window::close(window::Id::MAIN)
            }
        }
    }
    
    fn subscription(&self) -> Subscription<Self::Message> {
        // Subscribe to events from shell via NATS
        struct EventSubscription;
        
        subscription::channel(
            std::any::TypeId::of::<EventSubscription>(),
            100,
            |mut output| async move {
                // This would connect to NATS and listen for events
                // For now, we'll use a placeholder
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        )
    }

    fn view(&self) -> Element<Self::Message> {
        // Header with model info and actions
        let header = container(
            row![
                text(format!("Model: {}", self.ai_model)).size(14),
                Space::with_width(Length::Fill),
                button("Clear").on_press(Message::ClearDialog),
                button("Close").on_press(Message::Close),
            ]
            .spacing(10)
            .align_y(alignment::Vertical::Center)
        )
        .padding(10);

        // Messages area
        let mut messages_column = Column::new().spacing(10).padding(10);
        
        for (i, msg) in self.messages.iter().enumerate() {
            messages_column = messages_column.push(self.render_message(msg, i));
        }
        
        if self.is_loading {
            messages_column = messages_column.push(
                container(
                    text("AI is thinking...").size(14)
                )
                .padding(10)
            );
        }
        
        let messages_scroll = scrollable(messages_column)
            .height(Length::Fill)
            .width(Length::Fill);

        // Input area
        let input_area = container(
            row![
                text_input("Type your message...", &self.input_value)
                    .on_input(Message::InputChanged)
                    .on_submit(Message::SendMessage)
                    .padding(10)
                    .width(Length::Fill),
                button("Send")
                    .on_press_maybe(
                        if !self.input_value.trim().is_empty() && !self.is_loading {
                            Some(Message::SendMessage)
                        } else {
                            None
                        }
                    ),
            ]
            .spacing(10)
            .align_y(alignment::Vertical::Center)
        )
        .padding(10);

        // Main layout
        column![
            header,
            messages_scroll,
            input_area,
        ]
        .into()
    }
}

impl DialogUI {
    fn render_message(&self, msg: &DialogMessage, index: usize) -> Element<Message> {
        let is_user = msg.role == "user";
        let time_str = msg.timestamp.with_timezone(&Local).format("%H:%M").to_string();
        
        let message_content = container(
            column![
                row![
                    text(&msg.role).size(12),
                    Space::with_width(Length::Fill),
                    text(&time_str).size(10),
                ]
                .align_y(alignment::Vertical::Center),
                text(&msg.content).size(14),
            ]
            .spacing(5)
        )
        .padding(10)
        .max_width(600);
        
        let message_row = if is_user {
            row![
                Space::with_width(Length::Fill),
                message_content,
            ]
        } else {
            row![
                message_content,
                button("Copy")
                    .on_press(Message::CopyMessage(index))
                    .padding(5),
                Space::with_width(Length::Fill),
            ]
        };
        
        container(message_row)
            .width(Length::Fill)
            .into()
    }
}