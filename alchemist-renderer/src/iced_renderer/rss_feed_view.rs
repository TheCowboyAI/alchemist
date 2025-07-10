//! RSS feed view for Iced renderer

use iced::{
    widget::{button, column, container, text},
    Element,
};

pub struct RssFeedView {
    feed_title: String,
}

#[derive(Debug, Clone)]
pub enum RssFeedMessage {
    Refresh,
}

impl RssFeedView {
    pub fn new() -> Self {
        Self {
            feed_title: "RSS Feed".to_string(),
        }
    }

    pub fn update(&mut self, message: RssFeedMessage) {
        match message {
            RssFeedMessage::Refresh => {
                // Refresh feed
            }
        }
    }

    pub fn view(&self) -> Element<RssFeedMessage> {
        container(
            column![
                text(&self.feed_title).size(20),
                button("Refresh").on_press(RssFeedMessage::Refresh),
            ]
            .spacing(10)
        )
        .padding(10)
        .into()
    }
}