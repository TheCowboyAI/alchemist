//! Example: Creating a custom Iced UI window in Alchemist
//!
//! This example shows how to create your own UI component that integrates
//! with Alchemist's event-based architecture.

use anyhow::Result;
use iced::{Task, Element, Theme, window, Length, Alignment};
use iced::widget::{column, container, row, text, button, text_input, Space};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

// Define your window's messages
#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    SubmitPressed,
    ResponseReceived(String),
    ClearPressed,
    CloseWindow,
}

// Define commands your window can send
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomCommand {
    ProcessText { text: String },
    Clear,
}

// Define events your window receives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomEvent {
    ProcessingComplete { result: String },
    Error { message: String },
}

// Your window struct
pub struct CustomWindow {
    title: String,
    input_value: String,
    output_value: String,
    is_processing: bool,
    command_sender: mpsc::Sender<CustomCommand>,
    event_receiver: Option<mpsc::Receiver<CustomEvent>>,
}

impl CustomWindow {
    pub fn new(
        title: String,
        command_sender: mpsc::Sender<CustomCommand>,
        event_receiver: mpsc::Receiver<CustomEvent>,
    ) -> (Self, Task<Message>) {
        (
            Self {
                title,
                input_value: String::new(),
                output_value: String::new(),
                is_processing: false,
                command_sender,
                event_receiver: Some(event_receiver),
            },
            Task::none()
        )
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InputChanged(value) => {
                self.input_value = value;
                Task::none()
            }
            Message::SubmitPressed => {
                if !self.input_value.is_empty() && !self.is_processing {
                    self.is_processing = true;
                    let text = self.input_value.clone();
                    let tx = self.command_sender.clone();
                    
                    tokio::spawn(async move {
                        let _ = tx.send(CustomCommand::ProcessText { text }).await;
                    });
                    
                    // Start polling for events
                    Task::run(
                        async {
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            Message::ResponseReceived(String::new())
                        },
                        |msg| msg
                    )
                } else {
                    Task::none()
                }
            }
            Message::ResponseReceived(_) => {
                // Check for events
                if let Some(receiver) = &mut self.event_receiver {
                    if let Ok(event) = receiver.try_recv() {
                        match event {
                            CustomEvent::ProcessingComplete { result } => {
                                self.output_value = result;
                                self.is_processing = false;
                                self.input_value.clear();
                            }
                            CustomEvent::Error { message } => {
                                self.output_value = format!("Error: {}", message);
                                self.is_processing = false;
                            }
                        }
                    }
                }
                
                // Continue polling if still processing
                if self.is_processing {
                    Task::run(
                        async {
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            Message::ResponseReceived(String::new())
                        },
                        |msg| msg
                    )
                } else {
                    Task::none()
                }
            }
            Message::ClearPressed => {
                self.input_value.clear();
                self.output_value.clear();
                let tx = self.command_sender.clone();
                tokio::spawn(async move {
                    let _ = tx.send(CustomCommand::Clear).await;
                });
                Task::none()
            }
            Message::CloseWindow => {
                std::process::exit(0);
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let header = row![
            text(&self.title).size(24),
            Space::with_width(Length::Fill),
            button("Clear").on_press(Message::ClearPressed),
            button("Close").on_press(Message::CloseWindow),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let input_section = column![
            text("Input:").size(16),
            text_input("Enter text to process...", &self.input_value)
                .on_input(Message::InputChanged)
                .on_submit(Message::SubmitPressed)
                .padding(10),
            button(if self.is_processing { "Processing..." } else { "Submit" })
                .on_press_maybe(if self.is_processing { None } else { Some(Message::SubmitPressed) })
                .width(Length::Fill),
        ]
        .spacing(10);

        let output_section = column![
            text("Output:").size(16),
            container(
                text(&self.output_value)
                    .size(14)
            )
            .width(Length::Fill)
            .height(Length::Fixed(200.0))
            .padding(10)
            .style(container::bordered_box),
        ]
        .spacing(10);

        let content = column![
            container(header)
                .width(Length::Fill)
                .padding(10)
                .style(container::rounded_box),
            container(input_section)
                .width(Length::Fill)
                .padding(20)
                .style(container::rounded_box),
            container(output_section)
                .width(Length::Fill)
                .padding(20)
                .style(container::rounded_box),
        ]
        .spacing(10)
        .padding(10);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

// Handler that processes commands
pub struct CustomHandler {
    event_sender: mpsc::Sender<CustomEvent>,
}

impl CustomHandler {
    pub fn new(event_sender: mpsc::Sender<CustomEvent>) -> Self {
        Self { event_sender }
    }

    pub async fn start(self, mut command_receiver: mpsc::Receiver<CustomCommand>) -> Result<()> {
        while let Some(command) = command_receiver.recv().await {
            match command {
                CustomCommand::ProcessText { text } => {
                    // Simulate processing
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    
                    // Transform the text (example: reverse it)
                    let result = text.chars().rev().collect::<String>();
                    
                    let _ = self.event_sender.send(
                        CustomEvent::ProcessingComplete { result }
                    ).await;
                }
                CustomCommand::Clear => {
                    // Nothing to do for clear
                }
            }
        }
        Ok(())
    }
}

// Function to launch the custom window
pub async fn launch_custom_window() -> Result<()> {
    let (cmd_tx, cmd_rx) = mpsc::channel(100);
    let (event_tx, event_rx) = mpsc::channel(100);

    // Start the handler
    let handler = CustomHandler::new(event_tx);
    tokio::spawn(async move {
        if let Err(e) = handler.start(cmd_rx).await {
            eprintln!("Handler error: {}", e);
        }
    });

    // Run the window
    iced::application(
        CustomWindow::title,
        CustomWindow::update,
        CustomWindow::view
    )
    .window(window::Settings {
        size: iced::Size::new(600.0, 500.0),
        position: window::Position::Centered,
        resizable: true,
        ..Default::default()
    })
    .theme(|_| Theme::Dark)
    .run_with(|| CustomWindow::new(
        "Custom UI Example".to_string(),
        cmd_tx,
        event_rx
    ))
    .map_err(|e| anyhow::anyhow!("Window error: {}", e))
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Launching custom UI window example...");
    launch_custom_window().await
}