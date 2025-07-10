//! Dashboard application for Iced renderer

use iced::{
    widget::{button, column, container, row, text},
    Application, Command, Element, Length, Settings, Theme,
};

pub struct DashboardApp {
    title: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Close,
}

impl Application for DashboardApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = alchemist::dashboard::DashboardData;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                title: flags.title.clone(),
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Close => {
                // Close window
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        container(
            column![
                text("Dashboard").size(24),
                button("Close").on_press(Message::Close),
            ]
            .spacing(20)
        )
        .padding(20)
        .center_x()
        .center_y()
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}