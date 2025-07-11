//! Minimal Iced renderer to test window creation

use anyhow::Result;
use iced::{Task, Element, Theme, window, Length};
use iced::widget::{column, text, button};

#[derive(Debug, Clone)]
enum Message {
    Exit,
}

struct MinimalApp;

impl MinimalApp {
    fn new() -> (Self, Task<Message>) {
        (MinimalApp, Task::none())
    }

    fn title(&self) -> String {
        "Alchemist - Test Window".to_string()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Exit => {
                std::process::exit(0);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        column![
            text("Alchemist UI is running!").size(24),
            text("This is a minimal window to verify Iced is working.").size(16),
            button("Exit").on_press(Message::Exit),
        ]
        .padding(20)
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

pub fn run_minimal() -> Result<()> {
    iced::application(
        MinimalApp::title,
        MinimalApp::update,
        MinimalApp::view
    )
    .window(window::Settings {
        size: iced::Size::new(400.0, 300.0),
        position: window::Position::Centered,
        resizable: true,
        ..Default::default()
    })
    .theme(|_| Theme::Dark)
    .run_with(|| MinimalApp::new())
    .map_err(|e| anyhow::anyhow!("Iced error: {}", e))
}