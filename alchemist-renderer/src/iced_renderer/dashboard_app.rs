//! Dashboard application for Iced renderer

use iced::{
    widget::{button, column, container, row, scrollable, text, Rule, Space},
    Application, Command, Element, Length, Settings, Theme, Alignment, Color,
};
use alchemist::dashboard::{DashboardData, DomainHealth};

pub struct DashboardApp {
    data: DashboardData,
    selected_tab: Tab,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Overview,
    Domains,
    Events,
    Dialogs,
}

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(Tab),
    RefreshData,
}

impl Application for DashboardApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = DashboardData;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                data: flags,
                selected_tab: Tab::Overview,
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        "Alchemist - Domain Dashboard".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::TabSelected(tab) => {
                self.selected_tab = tab;
            }
            Message::RefreshData => {
                // TODO: Implement data refresh
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let header = self.view_header();
        let content = match self.selected_tab {
            Tab::Overview => self.view_overview(),
            Tab::Domains => self.view_domains(),
            Tab::Events => self.view_events(),
            Tab::Dialogs => self.view_dialogs(),
        };

        container(
            column![
                header,
                Rule::horizontal(2),
                content,
            ]
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

impl DashboardApp {
    fn view_header(&self) -> Element<Message> {
        let tabs = row![
            self.tab_button("Overview", Tab::Overview),
            self.tab_button("Domains", Tab::Domains),
            self.tab_button("Events", Tab::Events),
            self.tab_button("Dialogs", Tab::Dialogs),
            Space::with_width(Length::Fill),
            button("Refresh").on_press(Message::RefreshData),
        ]
        .spacing(10)
        .padding(10)
        .align_items(Alignment::Center);

        container(tabs)
            .width(Length::Fill)
            .style(iced::theme::Container::Box)
            .into()
    }

    fn tab_button(&self, label: &str, tab: Tab) -> Element<Message> {
        let is_selected = self.selected_tab == tab;
        let button = button(text(label).size(16))
            .padding([8, 16])
            .on_press(Message::TabSelected(tab));
        
        if is_selected {
            button.style(iced::theme::Button::Primary)
        } else {
            button.style(iced::theme::Button::Secondary)
        }
        .into()
    }

    fn view_overview(&self) -> Element<Message> {
        let system_info = container(
            column![
                text("System Status").size(20),
                Rule::horizontal(1),
                row![
                    text("NATS Connected:"),
                    text(if self.data.system_status.nats_connected { "✓" } else { "✗" })
                        .color(if self.data.system_status.nats_connected { 
                            Color::from_rgb(0.0, 0.8, 0.0) 
                        } else { 
                            Color::from_rgb(0.8, 0.0, 0.0) 
                        })
                ].spacing(10),
                row![
                    text("Total Events:"),
                    text(format!("{}", self.data.system_status.total_events))
                ].spacing(10),
                row![
                    text("Memory Usage:"),
                    text(format!("{:.1}%", self.data.system_status.memory_usage))
                ].spacing(10),
            ]
            .spacing(10)
            .padding(20)
        )
        .style(iced::theme::Container::Box)
        .width(Length::Fill);

        let domain_summary = container(
            column![
                text("Domain Summary").size(20),
                Rule::horizontal(1),
                text(format!("Active Domains: {}", 
                    self.data.domains.iter().filter(|d| d.enabled).count()
                )),
                text(format!("Total Events: {}", 
                    self.data.domains.iter().map(|d| d.event_count).sum::<u64>()
                )),
                text(format!("Healthy: {} | Warning: {} | Error: {}", 
                    self.data.domains.iter().filter(|d| matches!(d.health, DomainHealth::Healthy)).count(),
                    self.data.domains.iter().filter(|d| matches!(d.health, DomainHealth::Warning(_))).count(),
                    self.data.domains.iter().filter(|d| matches!(d.health, DomainHealth::Error(_))).count()
                )),
            ]
            .spacing(10)
            .padding(20)
        )
        .style(iced::theme::Container::Box)
        .width(Length::Fill);

        container(
            scrollable(
                column![
                    system_info,
                    domain_summary,
                ]
                .spacing(20)
                .padding(20)
            )
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_domains(&self) -> Element<Message> {
        let domain_cards: Vec<Element<Message>> = self.data.domains.iter()
            .map(|domain| {
                let health_color = match &domain.health {
                    DomainHealth::Healthy => Color::from_rgb(0.0, 0.8, 0.0),
                    DomainHealth::Warning(_) => Color::from_rgb(0.8, 0.6, 0.0),
                    DomainHealth::Error(_) => Color::from_rgb(0.8, 0.0, 0.0),
                    DomainHealth::Unknown => Color::from_rgb(0.5, 0.5, 0.5),
                };

                let health_status = match &domain.health {
                    DomainHealth::Healthy => "● Healthy".to_string(),
                    DomainHealth::Warning(msg) => format!("⚠ Warning: {}", msg),
                    DomainHealth::Error(msg) => format!("✗ Error: {}", msg),
                    DomainHealth::Unknown => "? Unknown".to_string(),
                };

                container(
                    column![
                        row![
                            text(&domain.name).size(18),
                            Space::with_width(Length::Fill),
                            text(&health_status).color(health_color).size(14),
                        ],
                        text(&domain.description).size(14).style(iced::theme::Text::Color(Color::from_rgb(0.6, 0.6, 0.6))),
                        Rule::horizontal(1),
                        row![
                            text(format!("Events: {}", domain.event_count)).size(12),
                            Space::with_width(Length::Fill),
                            text(if domain.enabled { "Enabled" } else { "Disabled" })
                                .size(12)
                                .color(if domain.enabled { 
                                    Color::from_rgb(0.0, 0.8, 0.0) 
                                } else { 
                                    Color::from_rgb(0.5, 0.5, 0.5) 
                                }),
                        ],
                        if !domain.dependencies.is_empty() {
                            text(format!("Dependencies: {}", domain.dependencies.join(", ")))
                                .size(12)
                                .style(iced::theme::Text::Color(Color::from_rgb(0.6, 0.6, 0.6)))
                        } else {
                            text("").size(12)
                        },
                    ]
                    .spacing(8)
                    .padding(15)
                )
                .style(iced::theme::Container::Box)
                .width(Length::Fill)
                .into()
            })
            .collect();

        container(
            scrollable(
                column(domain_cards)
                    .spacing(15)
                    .padding(20)
            )
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_events(&self) -> Element<Message> {
        let event_list: Vec<Element<Message>> = self.data.recent_events.iter()
            .map(|event| {
                container(
                    row![
                        text(&event.timestamp).size(14).style(iced::theme::Text::Color(Color::from_rgb(0.6, 0.6, 0.6))),
                        text(&event.domain).size(14),
                        text(&event.event_type).size(14),
                        text(&event.summary).size(14),
                    ]
                    .spacing(20)
                    .padding(10)
                )
                .width(Length::Fill)
                .into()
            })
            .collect();

        container(
            scrollable(
                column![
                    text("Recent Events").size(20),
                    Rule::horizontal(2),
                    column(event_list).spacing(5)
                ]
                .spacing(10)
                .padding(20)
            )
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_dialogs(&self) -> Element<Message> {
        let dialog_cards: Vec<Element<Message>> = self.data.active_dialogs.iter()
            .map(|dialog| {
                container(
                    column![
                        row![
                            text(&dialog.title).size(16),
                            Space::with_width(Length::Fill),
                            text(&dialog.last_active).size(12).style(iced::theme::Text::Color(Color::from_rgb(0.6, 0.6, 0.6))),
                        ],
                        row![
                            text(format!("Model: {}", dialog.model)).size(14),
                            Space::with_width(Length::Fill),
                            text(format!("{} messages", dialog.message_count)).size(14),
                        ],
                    ]
                    .spacing(8)
                    .padding(15)
                )
                .style(iced::theme::Container::Box)
                .width(Length::Fill)
                .into()
            })
            .collect();

        container(
            scrollable(
                column![
                    text("Active Dialogs").size(20),
                    Rule::horizontal(2),
                    column(dialog_cards).spacing(10)
                ]
                .spacing(10)
                .padding(20)
            )
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}