//! Minimal UI test to verify iced is working

use iced::{Element, Task, Theme, window, Length};
use iced::widget::{button, column, container, text};

#[derive(Debug, Clone)]
enum Message {
    ButtonPressed,
}

struct MinimalApp {
    counter: i32,
}

impl MinimalApp {
    fn new() -> (Self, Task<Message>) {
        (
            MinimalApp { counter: 0 },
            Task::none()
        )
    }
    
    fn title(&self) -> String {
        "Minimal Iced Test".to_string()
    }
    
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ButtonPressed => {
                self.counter += 1;
                println!("Button pressed! Counter: {}", self.counter);
                Task::none()
            }
        }
    }
    
    fn view(&self) -> Element<Message> {
        container(
            column![
                text("Minimal Iced Test").size(24),
                text(format!("Counter: {}", self.counter)).size(18),
                button("Click Me!")
                    .on_press(Message::ButtonPressed)
                    .padding(10),
            ]
            .spacing(20)
        )
        .padding(20)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting minimal UI test...");
    
    iced::application(
        MinimalApp::title,
        MinimalApp::update,
        MinimalApp::view
    )
    .window(window::Settings {
        size: iced::Size::new(400.0, 300.0),
        position: window::Position::Centered,
        ..Default::default()
    })
    .theme(|_| Theme::Dark)
    .run_with(MinimalApp::new)
    .map_err(|e| anyhow::anyhow!("Iced error: {}", e))
}