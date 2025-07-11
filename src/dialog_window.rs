//! Dialog window implementation using Iced
//! Provides an interactive UI for AI conversations

use anyhow::Result;
use iced::{Task, Element, Theme, window, Length, Alignment};
use iced::widget::{column, container, row, text, scrollable, button, text_input, Space, Column};
use tokio::sync::mpsc;
use std::collections::VecDeque;

use crate::dialog::{DialogMessage, MessageRole};

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    SendMessage,
    MessageReceived(DialogMessage),
    TokenReceived(String),
    StreamComplete,
    SelectModel(String),
    ClearHistory,
    ExportDialog,
    CloseWindow,
    Scroll(scrollable::Viewport),
    RefreshData,
}

pub struct DialogWindow {
    dialog_id: String,
    title: String,
    messages: VecDeque<DialogMessage>,
    input_value: String,
    current_model: String,
    available_models: Vec<String>,
    is_waiting: bool,
    current_response: Option<String>,
    command_sender: mpsc::Sender<DialogCommand>,
    event_receiver: Option<mpsc::Receiver<DialogEvent>>,
    scroll_to_bottom: bool,
}

#[derive(Debug, Clone)]
pub enum DialogCommand {
    SendMessage { content: String, model: String },
    ChangeModel { model: String },
    ExportDialog { format: ExportFormat },
}

#[derive(Debug, Clone)]
pub enum DialogEvent {
    MessageAdded(DialogMessage),
    TokenStreamed(String),
    ResponseComplete,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Markdown,
    Json,
    Text,
}

impl DialogWindow {
    pub fn new(
        dialog_id: String,
        title: String,
        command_sender: mpsc::Sender<DialogCommand>,
        event_receiver: mpsc::Receiver<DialogEvent>,
    ) -> (Self, Task<Message>) {
        let available_models = vec![
            "claude-3-sonnet".to_string(),
            "claude-3-opus".to_string(),
            "gpt-4".to_string(),
            "gpt-3.5-turbo".to_string(),
        ];

        (
            DialogWindow {
                dialog_id,
                title,
                messages: VecDeque::new(),
                input_value: String::new(),
                current_model: available_models[0].clone(),
                available_models,
                is_waiting: false,
                current_response: None,
                command_sender,
                event_receiver: Some(event_receiver),
                scroll_to_bottom: false,
            },
            Task::none()
        )
    }

    pub fn title(&self) -> String {
        format!("{} - Alchemist Dialog", self.title)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InputChanged(value) => {
                self.input_value = value;
                Task::none()
            }
            Message::SendMessage => {
                if !self.input_value.is_empty() && !self.is_waiting {
                    let content = self.input_value.clone();
                    self.input_value.clear();
                    self.is_waiting = true;
                    self.current_response = Some(String::new());
                    
                    // Add user message
                    let user_msg = DialogMessage {
                        role: MessageRole::User,
                        content: content.clone(),
                        timestamp: chrono::Utc::now(),
                        tokens: None,
                    };
                    self.messages.push_back(user_msg);
                    self.scroll_to_bottom = true;
                    
                    // Send to backend
                    let tx = self.command_sender.clone();
                    let model = self.current_model.clone();
                    tokio::spawn(async move {
                        let _ = tx.send(DialogCommand::SendMessage { content, model }).await;
                    });
                }
                Task::none()
            }
            Message::MessageReceived(msg) => {
                self.messages.push_back(msg);
                self.scroll_to_bottom = true;
                Task::none()
            }
            Message::TokenReceived(token) => {
                if let Some(response) = &mut self.current_response {
                    response.push_str(&token);
                }
                Task::none()
            }
            Message::StreamComplete => {
                // Complete the streaming response
                if let Some(response) = self.current_response.take() {
                    self.messages.push_back(DialogMessage {
                        role: MessageRole::Assistant,
                        content: response,
                        timestamp: chrono::Utc::now(),
                        tokens: None,
                    });
                    self.scroll_to_bottom = true;
                    self.is_waiting = false;
                }
                Task::none()
            }
            Message::SelectModel(model) => {
                self.current_model = model.clone();
                let tx = self.command_sender.clone();
                tokio::spawn(async move {
                    let _ = tx.send(DialogCommand::ChangeModel { model }).await;
                });
                Task::none()
            }
            Message::ClearHistory => {
                self.messages.clear();
                self.current_response = None;
                self.is_waiting = false;
                Task::none()
            }
            Message::ExportDialog => {
                let tx = self.command_sender.clone();
                tokio::spawn(async move {
                    let _ = tx.send(DialogCommand::ExportDialog { 
                        format: ExportFormat::Markdown 
                    }).await;
                });
                Task::none()
            }
            Message::CloseWindow => {
                std::process::exit(0);
            }
            Message::Scroll(_viewport) => {
                self.scroll_to_bottom = false;
                Task::none()
            }
            Message::RefreshData => {
                // Check for new events
                let mut tasks = vec![];
                
                if let Some(receiver) = &mut self.event_receiver {
                    match receiver.try_recv() {
                        Ok(event) => {
                            match event {
                                DialogEvent::TokenStreamed(token) => {
                                    tasks.push(Task::done(Message::TokenReceived(token)));
                                }
                                DialogEvent::MessageAdded(msg) => {
                                    tasks.push(Task::done(Message::MessageReceived(msg)));
                                }
                                DialogEvent::ResponseComplete => {
                                    self.is_waiting = false;
                                    self.current_response = None;
                                }
                                DialogEvent::Error(err) => {
                                    eprintln!("Dialog error: {}", err);
                                    self.is_waiting = false;
                                }
                            }
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                            // No new events
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                            // Channel disconnected
                            self.event_receiver = None;
                        }
                    }
                }
                
                // Continue polling if we're waiting
                if self.is_waiting {
                    // Schedule another refresh
                    tasks.push(Task::done(Message::RefreshData));
                }
                
                Task::batch(tasks)
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let header = row![
            text(&self.title).size(24),
            Space::with_width(Length::Fill),
            // Model selector
            row(
                self.available_models.iter()
                    .map(|model| {
                        button(text(model).size(12))
                            .on_press(Message::SelectModel(model.clone()))
                            .style(if &self.current_model == model {
                                button::primary
                            } else {
                                button::secondary
                            })
                            .into()
                    })
                    .collect::<Vec<Element<Message>>>()
            ).spacing(5),
            Space::with_width(Length::Fixed(20.0)),
            button("Export").on_press(Message::ExportDialog),
            button("Clear").on_press(Message::ClearHistory),
            button("Close").on_press(Message::CloseWindow),
        ]
        .spacing(10)
        .padding(10)
        .align_y(Alignment::Center);

        // Messages area
        let mut messages_column = Column::new();
        
        for msg in &self.messages {
            let is_user = matches!(msg.role, MessageRole::User);
            let msg_row = row![
                if is_user {
                    container(text(""))
                        .width(Length::FillPortion(1))
                } else {
                    container(
                        text("AI")
                            .size(12)
                            .color(iced::Color::from_rgb(0.5, 0.5, 1.0))
                    )
                    .width(Length::Fixed(30.0))
                    .center_x(Length::Fill)
                },
                container(
                    column![
                        text(&msg.content)
                            .size(14),
                        if let Some(tokens) = msg.tokens {
                            text(format!("Tokens: {}", tokens))
                            .size(10)
                            .color(iced::Color::from_rgb(0.6, 0.6, 0.6))
                        } else {
                            text("")
                        }
                    ]
                    .spacing(2)
                    .padding(10)
                )
                .style(if is_user {
                    container::rounded_box
                } else {
                    container::bordered_box
                })
                .width(Length::FillPortion(3)),
                if is_user {
                    container(
                        text("You")
                            .size(12)
                            .color(iced::Color::from_rgb(0.5, 1.0, 0.5))
                    )
                    .width(Length::Fixed(30.0))
                    .center_x(Length::Fill)
                } else {
                    container(text(""))
                        .width(Length::FillPortion(1))
                },
            ]
            .spacing(10)
            .align_y(Alignment::Start);
            
            messages_column = messages_column.push(msg_row);
        }
        
        // Show current streaming response
        if let Some(response) = &self.current_response {
            if !response.is_empty() {
                let streaming_row = row![
                    container(
                        text("AI")
                            .size(12)
                            .color(iced::Color::from_rgb(0.5, 0.5, 1.0))
                    )
                    .width(Length::Fixed(30.0))
                    .center_x(Length::Fill),
                    container(
                        text(response)
                            .size(14)
                    )
                    .style(container::bordered_box)
                    .padding(10)
                    .width(Length::FillPortion(3)),
                    container(text(""))
                        .width(Length::FillPortion(1)),
                ]
                .spacing(10)
                .align_y(Alignment::Start);
                
                messages_column = messages_column.push(streaming_row);
            }
        }
        
        let messages_scroll = scrollable(
            container(messages_column.spacing(10))
                .padding(10)
                .width(Length::Fill)
        )
        .height(Length::Fill)
        .on_scroll(Message::Scroll);

        // Input area
        let input_row = row![
            text_input("Type your message...", &self.input_value)
                .on_input(Message::InputChanged)
                .on_submit(Message::SendMessage)
                .size(16)
                .padding(10)
                .width(Length::Fill),
            button(
                text(if self.is_waiting { "..." } else { "Send" })
                    .size(16)
            )
            .on_press_maybe(if self.is_waiting { None } else { Some(Message::SendMessage) })
            .padding(10),
        ]
        .spacing(10)
        .padding(10);

        // Main layout
        let content = column![
            container(header)
                .width(Length::Fill)
                .style(container::rounded_box),
            container(messages_scroll)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(container::rounded_box),
            container(input_row)
                .width(Length::Fill)
                .style(container::rounded_box),
        ]
        .spacing(10)
        .padding(10);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        // Poll for refresh every 100ms when waiting for response
        if self.is_waiting {
            iced::time::every(std::time::Duration::from_millis(100))
                .map(|_| Message::RefreshData)
        } else {
            iced::Subscription::none()
        }
    }
}

/// Run the dialog window
pub async fn run_dialog_window(
    dialog_id: String,
    title: String,
    command_sender: mpsc::Sender<DialogCommand>,
    event_receiver: mpsc::Receiver<DialogEvent>,
) -> Result<()> {
    iced::application(
        DialogWindow::title,
        DialogWindow::update,
        DialogWindow::view
    )
    .subscription(DialogWindow::subscription)
    .window(window::Settings {
        size: iced::Size::new(900.0, 700.0),
        position: window::Position::Centered,
        resizable: true,
        ..Default::default()
    })
    .theme(|_| Theme::Dark)
    .run_with(|| DialogWindow::new(dialog_id, title, command_sender, event_receiver))
    .map_err(|e| anyhow::anyhow!("Iced error: {}", e))
}