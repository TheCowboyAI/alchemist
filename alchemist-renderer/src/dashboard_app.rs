//! Dashboard application for Iced

use iced::{
    widget::{button, column, container, row, scrollable, text, Column, Container, Row, Space},
    window, Alignment, Application, Background, BorderRadius, Color, Command, Element, Length, Settings, Theme,
};
use serde_json::Value;
use alchemist::dashboard::{DashboardData, DomainInfo, DomainHealth};

pub struct DashboardApp {
    data: DashboardData,
    selected_domain: Option<String>,
    active_tab: Tab,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Domains,
    Dialogs,
    Events,
    Policies,
    RssFeeds,
}

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(Tab),
    DomainSelected(String),
    LaunchWorkflow(String),
    LaunchGraph(String),
    OpenDialog(String),
    RefreshData,
}

impl DashboardApp {
    pub fn new(data: DashboardData) -> Self {
        Self {
            data,
            selected_domain: None,
            active_tab: Tab::Domains,
        }
    }
}

impl Application for DashboardApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = DashboardData;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::new(flags), Command::none())
    }

    fn title(&self) -> String {
        "Alchemist - Domain Dashboard".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::TabSelected(tab) => {
                self.active_tab = tab;
            }
            Message::DomainSelected(domain) => {
                self.selected_domain = Some(domain);
            }
            Message::LaunchWorkflow(domain) => {
                // In real implementation, would send IPC to launch workflow visualizer
                println!("Launching workflow for domain: {}", domain);
            }
            Message::LaunchGraph(domain) => {
                // In real implementation, would send IPC to launch 3D graph
                println!("Launching graph for domain: {}", domain);
            }
            Message::OpenDialog(id) => {
                println!("Opening dialog: {}", id);
            }
            Message::RefreshData => {
                // In real implementation, would fetch fresh data
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let header = self.view_header();
        let tabs = self.view_tabs();
        let content = match self.active_tab {
            Tab::Domains => self.view_domains(),
            Tab::Dialogs => self.view_dialogs(),
            Tab::Events => self.view_events(),
            Tab::Policies => self.view_policies(),
            Tab::RssFeeds => self.view_rss_feeds(),
        };
        let status_bar = self.view_status_bar();

        column![
            header,
            tabs,
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(20),
            status_bar,
        ]
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

impl DashboardApp {
    fn view_header(&self) -> Element<Message> {
        container(
            row![
                text("🚀 Alchemist").size(28),
                Space::with_width(Length::Fill),
                text("Domain Control System").size(16),
                Space::with_width(20),
                button("Refresh").on_press(Message::RefreshData),
            ]
            .align_y(Alignment::Center)
        )
        .padding(20)
        .style(|theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
            ..Default::default()
        })
        .into()
    }

    fn view_tabs(&self) -> Element<Message> {
        container(
            row![
                Self::tab_button("Domains", Tab::Domains, self.active_tab),
                Self::tab_button("Dialogs", Tab::Dialogs, self.active_tab),
                Self::tab_button("Events", Tab::Events, self.active_tab),
                Self::tab_button("Policies", Tab::Policies, self.active_tab),
                Self::tab_button("RSS Feeds", Tab::RssFeeds, self.active_tab),
            ]
            .spacing(10)
        )
        .padding(10)
        .style(|theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
            ..Default::default()
        })
        .into()
    }

    fn tab_button(label: &str, tab: Tab, active: Tab) -> Element<Message> {
        let is_active = tab == active;
        button(text(label).size(16))
            .on_press(Message::TabSelected(tab))
            .style(move |theme: &Theme, status| {
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

    fn view_domains(&self) -> Element<Message> {
        let domains: Vec<Element<Message>> = self.data.domains
            .iter()
            .map(|domain| self.domain_card(domain))
            .collect();

        scrollable(
            Column::with_children(domains)
                .spacing(15)
                .width(Length::Fill)
        )
        .into()
    }

    fn domain_card(&self, domain: &DomainInfo) -> Element<Message> {
        let health_color = match &domain.health {
            DomainHealth::Healthy => Color::from_rgb(0.2, 0.8, 0.2),
            DomainHealth::Warning(_) => Color::from_rgb(0.8, 0.8, 0.2),
            DomainHealth::Error(_) => Color::from_rgb(0.8, 0.2, 0.2),
            DomainHealth::Unknown => Color::from_rgb(0.5, 0.5, 0.5),
        };

        let health_indicator = container(Space::new(10, 10))
            .style(move |theme: &Theme| container::Style {
                background: Some(Background::Color(health_color)),
                border: iced::Border {
                    radius: BorderRadius::from(5.0),
                    ..Default::default()
                },
                ..Default::default()
            });

        container(
            column![
                row![
                    health_indicator,
                    text(&domain.name).size(20),
                    Space::with_width(Length::Fill),
                    text(format!("{} events", domain.event_count)).size(14),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                
                text(&domain.description).size(14),
                
                row![
                    button("View Workflow")
                        .on_press(Message::LaunchWorkflow(domain.name.clone())),
                    button("View Graph")
                        .on_press(Message::LaunchGraph(domain.name.clone())),
                    Space::with_width(Length::Fill),
                    if domain.enabled {
                        text("✅ Enabled").size(14)
                    } else {
                        text("⏸️ Disabled").size(14)
                    },
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            ]
            .spacing(10)
        )
        .padding(15)
        .width(Length::Fill)
        .style(|theme: &Theme| container::Style {
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

    fn view_dialogs(&self) -> Element<Message> {
        let dialogs: Vec<Element<Message>> = self.data.active_dialogs
            .iter()
            .map(|dialog| {
                container(
                    row![
                        column![
                            text(&dialog.title).size(18),
                            text(format!("Model: {} | {} messages", dialog.model, dialog.message_count)).size(14),
                        ],
                        Space::with_width(Length::Fill),
                        text(&dialog.last_active).size(14),
                        button("Open").on_press(Message::OpenDialog(dialog.id.clone())),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center)
                )
                .padding(15)
                .width(Length::Fill)
                .style(|theme: &Theme| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
                    border: iced::Border {
                        color: Color::from_rgb(0.3, 0.3, 0.3),
                        width: 1.0,
                        radius: BorderRadius::from(8.0),
                    },
                    ..Default::default()
                })
                .into()
            })
            .collect();

        scrollable(
            Column::with_children(dialogs)
                .spacing(10)
                .width(Length::Fill)
        )
        .into()
    }

    fn view_events(&self) -> Element<Message> {
        let events: Vec<Element<Message>> = self.data.recent_events
            .iter()
            .map(|event| {
                row![
                    text(&event.timestamp).size(14),
                    text(&event.domain).size(14),
                    text(&event.event_type).size(14),
                    text(&event.summary).size(14),
                ]
                .spacing(20)
                .into()
            })
            .collect();

        scrollable(
            column![
                row![
                    text("Time").size(16),
                    text("Domain").size(16),
                    text("Event").size(16),
                    text("Summary").size(16),
                ]
                .spacing(20),
                
                Column::with_children(events)
                    .spacing(5),
            ]
            .spacing(10)
        )
        .into()
    }

    fn view_policies(&self) -> Element<Message> {
        let policies: Vec<Element<Message>> = self.data.active_policies
            .iter()
            .map(|policy| {
                row![
                    if policy.enabled { "✅" } else { "⏸️" },
                    text(&policy.name).size(16),
                    text(&policy.domain).size(14),
                    text(format!("{} rules", policy.rules_count)).size(14),
                ]
                .spacing(20)
                .into()
            })
            .collect();

        scrollable(
            Column::with_children(policies)
                .spacing(10)
        )
        .into()
    }

    fn view_rss_feeds(&self) -> Element<Message> {
        // Create sample RSS feed data for demonstration
        use alchemist::rss_feed_manager::{ProcessedRssItem, RssFeedConfig, FeedState, Sentiment, Entity};
        
        let feeds = vec![
            FeedState {
                config: RssFeedConfig {
                    id: "tech-news".to_string(),
                    name: "Tech News Aggregator".to_string(),
                    url: "https://news.ycombinator.com/rss".to_string(),
                    category: "technology".to_string(),
                    update_interval: 300,
                    filters: vec![],
                    transformations: vec![],
                    enabled: true,
                },
                last_update: Some(chrono::Utc::now()),
                item_count: 42,
                error_count: 0,
                last_error: None,
            },
            FeedState {
                config: RssFeedConfig {
                    id: "arxiv-cs".to_string(),
                    name: "ArXiv Computer Science".to_string(),
                    url: "http://arxiv.org/rss/cs".to_string(),
                    category: "research".to_string(),
                    update_interval: 3600,
                    filters: vec![],
                    transformations: vec![],
                    enabled: true,
                },
                last_update: Some(chrono::Utc::now()),
                item_count: 15,
                error_count: 0,
                last_error: None,
            },
        ];
        
        let items = vec![
            ProcessedRssItem {
                id: "1".to_string(),
                feed_id: "tech-news".to_string(),
                feed_name: "Tech News Aggregator".to_string(),
                title: "New Breakthrough in Graph Neural Networks".to_string(),
                description: "Researchers demonstrate significant improvements in GNN architectures...".to_string(),
                link: "https://example.com/gnn".to_string(),
                pub_date: chrono::Utc::now() - chrono::Duration::hours(2),
                author: Some("AI Research Team".to_string()),
                categories: vec!["AI".to_string(), "Machine Learning".to_string()],
                guid: None,
                sentiment: Some(Sentiment {
                    score: 0.8,
                    label: "positive".to_string(),
                    confidence: 0.95,
                }),
                entities: vec![
                    Entity {
                        text: "GNN".to_string(),
                        entity_type: "TECHNOLOGY".to_string(),
                        confidence: 0.98,
                    },
                ],
                keywords: vec!["graph".to_string(), "neural networks".to_string(), "AI".to_string()],
                summary: Some("Major advancement in graph neural network efficiency and scalability.".to_string()),
                relevance_score: 0.92,
                processed_at: chrono::Utc::now(),
                event_id: uuid::Uuid::new_v4(),
            },
            ProcessedRssItem {
                id: "2".to_string(),
                feed_id: "arxiv-cs".to_string(),
                feed_name: "ArXiv Computer Science".to_string(),
                title: "Event Sourcing Patterns in Distributed Systems".to_string(),
                description: "A comprehensive survey of event sourcing implementations...".to_string(),
                link: "https://arxiv.org/abs/2401.12345".to_string(),
                pub_date: chrono::Utc::now() - chrono::Duration::hours(5),
                author: Some("Smith et al.".to_string()),
                categories: vec!["Distributed Systems".to_string()],
                guid: None,
                sentiment: Some(Sentiment {
                    score: 0.1,
                    label: "neutral".to_string(),
                    confidence: 0.88,
                }),
                entities: vec![
                    Entity {
                        text: "Event Sourcing".to_string(),
                        entity_type: "CONCEPT".to_string(),
                        confidence: 0.99,
                    },
                ],
                keywords: vec!["event sourcing".to_string(), "CQRS".to_string(), "distributed".to_string()],
                summary: Some("Survey of modern event sourcing patterns with performance analysis.".to_string()),
                relevance_score: 0.88,
                processed_at: chrono::Utc::now(),
                event_id: uuid::Uuid::new_v4(),
            },
        ];
        
        // Create RSS feed view
        use crate::rss_feed_view::{RssFeedView, RssFeedMessage};
        let rss_view = RssFeedView::new(feeds, items);
        
        // For now, just show a placeholder since we can't integrate the view directly
        // In a real implementation, we'd need to properly integrate the RSS view messages
        container(
            column![
                text("📰 RSS Feed Streams").size(20),
                text("Real-time RSS feeds processed through CIM event streams").size(14),
                Space::with_height(20),
                
                // Show some stats
                row![
                    container(
                        column![
                            text("Active Feeds").size(16),
                            text("2").size(24),
                        ]
                        .spacing(5)
                        .align_x(Alignment::Center)
                    )
                    .padding(20)
                    .width(Length::Fixed(150.0))
                    .style(|theme: &Theme| container::Style {
                        background: Some(Background::Color(Color::from_rgb(0.2, 0.3, 0.4))),
                        border: iced::Border {
                            radius: BorderRadius::from(8.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    
                    container(
                        column![
                            text("Total Items").size(16),
                            text("57").size(24),
                        ]
                        .spacing(5)
                        .align_x(Alignment::Center)
                    )
                    .padding(20)
                    .width(Length::Fixed(150.0))
                    .style(|theme: &Theme| container::Style {
                        background: Some(Background::Color(Color::from_rgb(0.3, 0.4, 0.2))),
                        border: iced::Border {
                            radius: BorderRadius::from(8.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    
                    container(
                        column![
                            text("Items/Hour").size(16),
                            text("12.5").size(24),
                        ]
                        .spacing(5)
                        .align_x(Alignment::Center)
                    )
                    .padding(20)
                    .width(Length::Fixed(150.0))
                    .style(|theme: &Theme| container::Style {
                        background: Some(Background::Color(Color::from_rgb(0.4, 0.3, 0.2))),
                        border: iced::Border {
                            radius: BorderRadius::from(8.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ]
                .spacing(20),
                
                Space::with_height(20),
                
                text("Note: RSS feeds are consumed from NATS event streams").size(12),
                text("CIM processes feeds with NLP enrichment and filtering").size(12),
            ]
            .spacing(10)
        )
        .padding(20)
        .into()
    }
    
    fn view_status_bar(&self) -> Element<Message> {
        container(
            row![
                if self.data.system_status.nats_connected {
                    text("🟢 NATS Connected")
                } else {
                    text("🔴 NATS Disconnected")
                },
                text(" | "),
                text(format!("Events: {}", self.data.system_status.total_events)),
                text(" | "),
                text(format!("Uptime: {}", self.data.system_status.uptime)),
                text(" | "),
                text(format!("Memory: {:.1}%", self.data.system_status.memory_usage)),
            ]
            .spacing(5)
        )
        .padding(10)
        .width(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
            ..Default::default()
        })
        .into()
    }
}