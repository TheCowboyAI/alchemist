//! RSS Feed viewer for Iced dashboard

use iced::{
    widget::{button, column, container, row, scrollable, text, Column, Container, Row, Space},
    Alignment, Background, BorderRadius, Color, Element, Length,
};
use alchemist::rss_feed_manager::{ProcessedRssItem, RssFeedConfig, FeedState, Sentiment};

#[derive(Debug, Clone)]
pub enum RssFeedMessage {
    RefreshFeeds,
    ToggleFeed(String),
    ViewItem(String),
    FilterByFeed(Option<String>),
    FilterBySentiment(Option<String>),
}

pub struct RssFeedView {
    feeds: Vec<FeedState>,
    items: Vec<ProcessedRssItem>,
    selected_feed: Option<String>,
    sentiment_filter: Option<String>,
}

impl RssFeedView {
    pub fn new(feeds: Vec<FeedState>, items: Vec<ProcessedRssItem>) -> Self {
        Self {
            feeds,
            items,
            selected_feed: None,
            sentiment_filter: None,
        }
    }
    
    pub fn update(&mut self, message: RssFeedMessage) {
        match message {
            RssFeedMessage::FilterByFeed(feed_id) => {
                self.selected_feed = feed_id;
            }
            RssFeedMessage::FilterBySentiment(sentiment) => {
                self.sentiment_filter = sentiment;
            }
            _ => {}
        }
    }
    
    pub fn view(&self) -> Element<RssFeedMessage> {
        let header = self.view_header();
        let feed_list = self.view_feed_list();
        let item_list = self.view_item_list();
        
        row![
            // Left panel - Feed list
            container(feed_list)
                .width(Length::Fixed(300.0))
                .height(Length::Fill)
                .style(|theme: &iced::Theme| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
                    border: iced::Border {
                        color: Color::from_rgb(0.3, 0.3, 0.3),
                        width: 1.0,
                        radius: BorderRadius::from(0.0),
                    },
                    ..Default::default()
                }),
            
            // Right panel - Items
            column![
                header,
                container(item_list)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(20),
            ]
            .width(Length::Fill),
        ]
        .into()
    }
    
    fn view_header(&self) -> Element<RssFeedMessage> {
        container(
            row![
                text("📰 RSS Feeds").size(24),
                Space::with_width(Length::Fill),
                self.sentiment_filter_buttons(),
                Space::with_width(20),
                button("Refresh").on_press(RssFeedMessage::RefreshFeeds),
            ]
            .align_y(Alignment::Center)
        )
        .padding(20)
        .style(|theme: &iced::Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
            ..Default::default()
        })
        .into()
    }
    
    fn sentiment_filter_buttons(&self) -> Element<RssFeedMessage> {
        row![
            self.filter_button("All", None),
            self.filter_button("Positive", Some("positive".to_string())),
            self.filter_button("Neutral", Some("neutral".to_string())),
            self.filter_button("Negative", Some("negative".to_string())),
        ]
        .spacing(5)
        .into()
    }
    
    fn filter_button(&self, label: &str, value: Option<String>) -> Element<RssFeedMessage> {
        let is_active = self.sentiment_filter == value;
        
        button(text(label).size(14))
            .on_press(RssFeedMessage::FilterBySentiment(value))
            .style(move |theme: &iced::Theme, status| {
                let base_color = if is_active {
                    Color::from_rgb(0.3, 0.5, 0.8)
                } else {
                    Color::from_rgb(0.2, 0.2, 0.2)
                };
                
                button::Style {
                    background: Some(Background::Color(base_color)),
                    text_color: Color::WHITE,
                    border: iced::Border {
                        color: Color::from_rgb(0.4, 0.4, 0.4),
                        width: 1.0,
                        radius: BorderRadius::from(4.0),
                    },
                    ..Default::default()
                }
            })
            .into()
    }
    
    fn view_feed_list(&self) -> Element<RssFeedMessage> {
        let mut feed_column = Column::new()
            .spacing(10)
            .padding(10);
        
        feed_column = feed_column.push(
            text("Feeds").size(18)
        );
        
        // "All Feeds" option
        let all_selected = self.selected_feed.is_none();
        feed_column = feed_column.push(
            button(
                row![
                    text("All Feeds").size(14),
                    Space::with_width(Length::Fill),
                    text(format!("{}", self.items.len())).size(12),
                ]
                .width(Length::Fill)
            )
            .on_press(RssFeedMessage::FilterByFeed(None))
            .width(Length::Fill)
            .style(move |theme: &iced::Theme, status| {
                let base_color = if all_selected {
                    Color::from_rgb(0.3, 0.5, 0.8)
                } else {
                    Color::from_rgb(0.2, 0.2, 0.2)
                };
                
                button::Style {
                    background: Some(Background::Color(base_color)),
                    text_color: Color::WHITE,
                    border: iced::Border::default(),
                    ..Default::default()
                }
            })
        );
        
        // Individual feeds
        for feed in &self.feeds {
            let is_selected = Some(&feed.config.id) == self.selected_feed.as_ref();
            let status_color = if feed.error_count > 0 {
                Color::from_rgb(0.8, 0.2, 0.2)
            } else {
                Color::from_rgb(0.2, 0.8, 0.2)
            };
            
            feed_column = feed_column.push(
                button(
                    column![
                        row![
                            container(Space::new(8, 8))
                                .style(move |theme: &iced::Theme| container::Style {
                                    background: Some(Background::Color(status_color)),
                                    border: iced::Border {
                                        radius: BorderRadius::from(4.0),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                            text(&feed.config.name).size(14),
                        ]
                        .spacing(5)
                        .align_y(Alignment::Center),
                        
                        row![
                            Space::with_width(13),
                            text(format!("{} items", feed.item_count)).size(12),
                        ],
                    ]
                    .spacing(2)
                )
                .on_press(RssFeedMessage::FilterByFeed(Some(feed.config.id.clone())))
                .width(Length::Fill)
                .style(move |theme: &iced::Theme, status| {
                    let base_color = if is_selected {
                        Color::from_rgb(0.3, 0.5, 0.8)
                    } else {
                        Color::from_rgb(0.2, 0.2, 0.2)
                    };
                    
                    button::Style {
                        background: Some(Background::Color(base_color)),
                        text_color: Color::WHITE,
                        border: iced::Border::default(),
                        ..Default::default()
                    }
                })
            );
        }
        
        scrollable(feed_column).into()
    }
    
    fn view_item_list(&self) -> Element<RssFeedMessage> {
        let filtered_items: Vec<&ProcessedRssItem> = self.items
            .iter()
            .filter(|item| {
                // Filter by feed
                if let Some(ref feed_id) = self.selected_feed {
                    if item.feed_id != *feed_id {
                        return false;
                    }
                }
                
                // Filter by sentiment
                if let Some(ref sentiment_filter) = self.sentiment_filter {
                    if let Some(ref sentiment) = item.sentiment {
                        if sentiment.label != *sentiment_filter {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                
                true
            })
            .collect();
        
        if filtered_items.is_empty() {
            return container(
                text("No items to display").size(16)
            )
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }
        
        let items: Vec<Element<RssFeedMessage>> = filtered_items
            .into_iter()
            .map(|item| self.render_item(item))
            .collect();
        
        scrollable(
            Column::with_children(items)
                .spacing(15)
                .width(Length::Fill)
        )
        .into()
    }
    
    fn render_item(&self, item: &ProcessedRssItem) -> Element<RssFeedMessage> {
        let sentiment_indicator = if let Some(ref sentiment) = item.sentiment {
            let color = match sentiment.label.as_str() {
                "positive" => Color::from_rgb(0.2, 0.8, 0.2),
                "negative" => Color::from_rgb(0.8, 0.2, 0.2),
                _ => Color::from_rgb(0.6, 0.6, 0.6),
            };
            
            container(
                text(&sentiment.label).size(12)
            )
            .padding(5)
            .style(move |theme: &iced::Theme| container::Style {
                background: Some(Background::Color(color.scale_alpha(0.2))),
                border: iced::Border {
                    color,
                    width: 1.0,
                    radius: BorderRadius::from(4.0),
                },
                ..Default::default()
            })
        } else {
            container(Space::new(0, 0))
        };
        
        container(
            column![
                row![
                    text(&item.title).size(18),
                    Space::with_width(Length::Fill),
                    sentiment_indicator,
                ]
                .align_y(Alignment::Center),
                
                text(&item.feed_name).size(12),
                
                text(&item.description)
                    .size(14)
                    .style(|theme: &iced::Theme| text::Style {
                        color: Some(Color::from_rgb(0.8, 0.8, 0.8)),
                    }),
                
                if let Some(ref summary) = item.summary {
                    text(format!("Summary: {}", summary))
                        .size(14)
                        .style(|theme: &iced::Theme| text::Style {
                            color: Some(Color::from_rgb(0.6, 0.8, 1.0)),
                        })
                } else {
                    text("")
                },
                
                row![
                    text(item.pub_date.format("%Y-%m-%d %H:%M").to_string()).size(12),
                    Space::with_width(20),
                    if !item.keywords.is_empty() {
                        text(format!("Keywords: {}", item.keywords.join(", "))).size(12)
                    } else {
                        text("")
                    },
                    Space::with_width(Length::Fill),
                    button("View")
                        .on_press(RssFeedMessage::ViewItem(item.id.clone()))
                        .style(|theme: &iced::Theme, status| button::Style {
                            background: Some(Background::Color(Color::from_rgb(0.2, 0.4, 0.6))),
                            text_color: Color::WHITE,
                            border: iced::Border::default(),
                            ..Default::default()
                        }),
                ]
                .align_y(Alignment::Center),
                
                if !item.entities.is_empty() {
                    row(
                        item.entities
                            .iter()
                            .take(5)
                            .map(|entity| {
                                container(
                                    text(format!("{} ({})", entity.text, entity.entity_type))
                                        .size(11)
                                )
                                .padding(3)
                                .style(|theme: &iced::Theme| container::Style {
                                    background: Some(Background::Color(Color::from_rgb(0.3, 0.3, 0.3))),
                                    border: iced::Border {
                                        radius: BorderRadius::from(3.0),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })
                                .into()
                            })
                            .collect()
                    )
                    .spacing(5)
                } else {
                    row![]
                },
            ]
            .spacing(8)
        )
        .padding(15)
        .width(Length::Fill)
        .style(|theme: &iced::Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
            border: iced::Border {
                color: Color::from_rgb(0.3, 0.3, 0.3),
                width: 1.0,
                radius: BorderRadius::from(8.0),
            },
            ..Default::default()
        })
        .into()
    }
}