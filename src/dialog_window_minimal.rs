//! Minimal dialog window for AI conversations

use anyhow::Result;
use iced::{Task, Element, Theme, window, Length, Alignment};
use iced::widget::{column, container, row, text, button, text_input, scrollable, Space};
use tokio::sync::mpsc;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    SendMessage,
    ResponseReceived(String),
    ToggleModel,
    Close,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    role: String,
    content: String,
}

pub struct DialogWindow {
    title: String,
    messages: VecDeque<ChatMessage>,
    input_value: String,
    current_model: String,
    models: Vec<String>,
    is_waiting: bool,
    ai_sender: Option<mpsc::Sender<String>>,
    response_receiver: Option<mpsc::Receiver<String>>,
}

impl DialogWindow {
    pub fn new(title: String) -> (Self, Task<Message>) {
        // Create channels for AI communication
        let (ai_tx, ai_rx) = mpsc::channel(10);
        let (response_tx, response_rx) = mpsc::channel(10);
        
        // Spawn mock AI responder
        tokio::spawn(async move {
            let mut rx = ai_rx;
            while let Some(prompt) = rx.recv().await {
                println!("AI received prompt: {}", prompt);
                
                // Simulate AI thinking
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                
                // Send mock response
                let response = format!("This is a mock AI response to: '{}'", prompt);
                let _ = response_tx.send(response).await;
            }
        });
        
        (
            DialogWindow {
                title,
                messages: VecDeque::new(),
                input_value: String::new(),
                current_model: "claude-3-sonnet".to_string(),
                models: vec![
                    "claude-3-sonnet".to_string(),
                    "claude-3-opus".to_string(),
                    "gpt-4".to_string(),
                ],
                is_waiting: false,
                ai_sender: Some(ai_tx),
                response_receiver: Some(response_rx),
            },
            Task::none()
        )
    }

    pub fn title(&self) -> String {
        format!("Dialog: {}", self.title)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InputChanged(value) => {
                self.input_value = value;
                Task::none()
            }
            Message::SendMessage => {
                if !self.input_value.trim().is_empty() && !self.is_waiting {
                    // Add user message
                    self.messages.push_back(ChatMessage {
                        role: "user".to_string(),
                        content: self.input_value.clone(),
                    });
                    
                    // Send to AI
                    if let Some(sender) = &self.ai_sender {
                        let prompt = self.input_value.clone();
                        let sender = sender.clone();
                        tokio::spawn(async move {
                            let _ = sender.send(prompt).await;
                        });
                    }
                    
                    self.input_value.clear();
                    self.is_waiting = true;
                }
                Task::none()
            }
            Message::ResponseReceived(_) => {
                // Check for actual response
                if let Some(receiver) = &mut self.response_receiver {
                    if let Ok(response) = receiver.try_recv() {
                        // Add AI response
                        self.messages.push_back(ChatMessage {
                            role: "assistant".to_string(),
                            content: response,
                        });
                        self.is_waiting = false;
                    }
                }
                Task::none()
            }
            Message::ToggleModel => {
                // Cycle through models
                let current_idx = self.models.iter()
                    .position(|m| m == &self.current_model)
                    .unwrap_or(0);
                let next_idx = (current_idx + 1) % self.models.len();
                self.current_model = self.models[next_idx].clone();
                Task::none()
            }
            Message::Close => {
                std::process::exit(0);
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let header = row![
            text(&self.title).size(24),
            Space::with_width(Length::Fill),
            button(text(&self.current_model).size(14))
                .on_press(Message::ToggleModel)
                .style(button::secondary),
            button("Close").on_press(Message::Close),
        ]
        .spacing(10)
        .padding(10)
        .align_y(Alignment::Center);

        // Messages area
        let mut messages_col = column![].spacing(10);
        
        for msg in &self.messages {
            let is_user = msg.role == "user";
            let msg_container = container(
                text(&msg.content).size(14)
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
                    container(text("AI is thinking...").size(14))
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
        .spacing(10)
        .padding(10);

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
        .spacing(10);

        container(content)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        if self.is_waiting {
            // Poll for AI responses
            iced::time::every(std::time::Duration::from_millis(100))
                .map(|_| Message::ResponseReceived("Check for response".to_string()))
        } else {
            iced::Subscription::none()
        }
    }
}

pub async fn run_dialog_window(title: String) -> Result<()> {
    println!("Starting dialog window: {}", title);
    
    iced::application(
        DialogWindow::title,
        DialogWindow::update,
        DialogWindow::view
    )
    .subscription(DialogWindow::subscription)
    .window(window::Settings {
        size: iced::Size::new(800.0, 600.0),
        position: window::Position::Centered,
        resizable: true,
        ..Default::default()
    })
    .theme(|_| Theme::Dark)
    .run_with(|| DialogWindow::new(title))
    .map_err(|e| anyhow::anyhow!("Iced error: {}", e))
}