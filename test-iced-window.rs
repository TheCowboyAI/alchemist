use iced::{Element, Task, Theme};
use iced::widget::{column, text};

#[derive(Debug, Clone)]
enum Message {}

struct App;

impl App {
    fn new() -> (Self, Task<Message>) {
        (App, Task::none())
    }
    
    fn title(&self) -> String {
        "Test Window".to_string()
    }
    
    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }
    
    fn view(&self) -> Element<Message> {
        column![
            text("Hello from Iced!").size(24),
        ]
        .padding(20)
        .into()
    }
}

fn main() -> iced::Result {
    println!("Starting Iced test window...");
    
    iced::application(
        App::title,
        App::update,
        App::view
    )
    .theme(|_| Theme::Dark)
    .run_with(|| App::new())
}