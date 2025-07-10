//! Dialog UI for AI conversations in Iced

use alchemist::renderer::DialogMessage;
use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Column, Space},
    window, executor, alignment, Application, Command, Element, Length, Settings, Theme,
    Color, Background, Border,
};
use chrono::{DateTime, Utc, Local};

pub struct DialogUI {
    dialog_id: String,
    ai_model: String,
    messages: Vec<DialogMessage>,
    system_prompt: Option<String>,
    input_value: String,
    is_loading: bool,
    scroll_to_bottom: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    SendMessage,
    ReceivedMessage(DialogMessage),
    ClearDialog,
    CopyMessage(usize),
    ExportDialog,
    ToggleSystemPrompt,
    Close,
}

pub struct DialogConfig {
    pub dialog_id: String,
    pub ai_model: String,
    pub messages: Vec<DialogMessage>,
    pub system_prompt: Option<String>,
}

impl Application for DialogUI {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = DialogConfig;

    fn new(flags: DialogConfig) -> (Self, Command<Self::Message>) {
        (
            Self {
                dialog_id: flags.dialog_id,
                ai_model: flags.ai_model,
                messages: flags.messages,
                system_prompt: flags.system_prompt,
                input_value: String::new(),
                is_loading: false,
                scroll_to_bottom: true,
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        format!("AI Dialog - {} ({})", self.dialog_id, self.ai_model)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::InputChanged(value) => {
                self.input_value = value;
                Command::none()
            }
            Message::SendMessage => {
                if !self.input_value.trim().is_empty() && !self.is_loading {
                    // Add user message
                    let user_msg = DialogMessage {
                        role: "user".to_string(),
                        content: self.input_value.clone(),
                        timestamp: Utc::now(),
                    };
                    self.messages.push(user_msg);
                    self.input_value.clear();
                    self.is_loading = true;
                    self.scroll_to_bottom = true;
                    
                    // TODO: Send message to AI through IPC/WebSocket
                    // For now, simulate a response after a delay
                    Command::none()
                } else {
                    Command::none()
                }
            }
            Message::ReceivedMessage(msg) => {
                self.messages.push(msg);
                self.is_loading = false;
                self.scroll_to_bottom = true;
                Command::none()
            }
            Message::ClearDialog => {
                self.messages.clear();
                Command::none()
            }
            Message::CopyMessage(index) => {
                if let Some(msg) = self.messages.get(index) {
                    // TODO: Copy to clipboard
                    println!("Copy message: {}", msg.content);
                }
                Command::none()
            }
            Message::ExportDialog => {
                // TODO: Export dialog to file
                println!("Export dialog");
                Command::none()
            }
            Message::ToggleSystemPrompt => {
                // TODO: Show/hide system prompt editor
                Command::none()
            }
            Message::Close => {
                window::close(window::Id::MAIN)
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        // Header with model info and actions
        let header = container(
            row![
                text(format!("Model: {}", self.ai_model))
                    .size(14),
                Space::with_width(Length::Fill),
                button("Clear").on_press(Message::ClearDialog),
                button("Export").on_press(Message::ExportDialog),
                button("Close").on_press(Message::Close),
            ]
            .spacing(10)
            .align_y(alignment::Vertical::Center)
        )
        .padding(10)
        .style(|theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
            border: Border {
                color: Color::from_rgb(0.8, 0.8, 0.8),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

        // Messages area
        let mut messages_column = Column::new().spacing(10).padding(10);
        
        for (i, msg) in self.messages.iter().enumerate() {
            messages_column = messages_column.push(self.render_message(msg, i));
        }
        
        if self.is_loading {
            messages_column = messages_column.push(
                container(
                    text("AI is thinking...").size(14).style(|theme: &Theme| text::Style {
                        color: Some(Color::from_rgb(0.5, 0.5, 0.5)),
                    })
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
                    )
                    .padding(10),
            ]
            .spacing(10)
            .align_y(alignment::Vertical::Center)
        )
        .padding(10)
        .style(|theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
            border: Border {
                color: Color::from_rgb(0.8, 0.8, 0.8),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

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
        let is_system = msg.role == "system";
        
        let (bg_color, text_color, alignment) = if is_user {
            (
                Color::from_rgb(0.2, 0.4, 0.9),
                Color::WHITE,
                alignment::Horizontal::End,
            )
        } else if is_system {
            (
                Color::from_rgb(0.8, 0.8, 0.8),
                Color::from_rgb(0.3, 0.3, 0.3),
                alignment::Horizontal::Start,
            )
        } else {
            (
                Color::from_rgb(0.95, 0.95, 0.95),
                Color::BLACK,
                alignment::Horizontal::Start,
            )
        };
        
        let time_str = msg.timestamp.with_timezone(&Local).format("%H:%M").to_string();
        
        let message_content = container(
            column![
                row![
                    text(&msg.role).size(12).style(move |theme: &Theme| text::Style {
                        color: Some(text_color),
                    }),
                    Space::with_width(Length::Fill),
                    text(&time_str).size(10).style(move |theme: &Theme| text::Style {
                        color: Some(Color::from_rgba(text_color.r, text_color.g, text_color.b, 0.7)),
                    }),
                ]
                .align_y(alignment::Vertical::Center),
                text(&msg.content).size(14).style(move |theme: &Theme| text::Style {
                    color: Some(text_color),
                }),
            ]
            .spacing(5)
        )
        .padding(10)
        .max_width(600)
        .style(move |theme: &Theme| container::Style {
            background: Some(Background::Color(bg_color)),
            border: Border {
                color: bg_color,
                width: 0.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        });
        
        let message_row = if is_user {
            row![
                Space::with_width(Length::Fill),
                message_content,
                Space::with_width(Length::Shrink),
            ]
        } else {
            row![
                Space::with_width(Length::Shrink),
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