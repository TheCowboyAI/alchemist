//! Iced UI Showcase - Demonstrates Alchemist's 2D interface capabilities

use iced::{
    application, executor, widget as w, Application, Command, Element, Length, Settings, Theme,
    Subscription, window,
};
use iced::widget::{Column, Container, Row, Scrollable};
use plotters::prelude::*;
use plotters_iced::{Chart, ChartWidget};
use std::sync::Arc;

#[derive(Debug, Clone)]
enum Message {
    TabSelected(Tab),
    DashboardUpdate,
    EventReceived(String),
    WorkflowNodeAdded,
    WorkflowNodeRemoved(usize),
    AIMessageSent(String),
    AIResponseReceived(String),
    SettingChanged(SettingType),
    ChartUpdate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Dashboard,
    EventMonitor,
    WorkflowEditor,
    AIDialog,
    Performance,
    Settings,
}

#[derive(Debug, Clone)]
enum SettingType {
    ThemeChanged(bool), // true for dark theme
    AutoSaveToggled(bool),
    NotificationsToggled(bool),
}

struct AlchemistUI {
    selected_tab: Tab,
    events: Vec<(String, String)>, // (timestamp, event)
    workflow_nodes: Vec<String>,
    ai_messages: Vec<(bool, String)>, // (is_user, message)
    ai_input: String,
    dark_theme: bool,
    auto_save: bool,
    notifications_enabled: bool,
    performance_data: Vec<f32>,
}

impl Application for AlchemistUI {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            AlchemistUI {
                selected_tab: Tab::Dashboard,
                events: vec![
                    ("12:34:01".to_string(), "System initialized".to_string()),
                    ("12:34:02".to_string(), "NATS connection established".to_string()),
                    ("12:34:03".to_string(), "Graph domain ready".to_string()),
                ],
                workflow_nodes: vec![
                    "Start".to_string(),
                    "Process Data".to_string(),
                    "AI Analysis".to_string(),
                    "Generate Report".to_string(),
                    "End".to_string(),
                ],
                ai_messages: vec![
                    (false, "Hello! I'm your AI assistant. How can I help you today?".to_string()),
                ],
                ai_input: String::new(),
                dark_theme: true,
                auto_save: true,
                notifications_enabled: true,
                performance_data: vec![50.0, 55.0, 48.0, 52.0, 60.0, 58.0, 62.0, 65.0],
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Alchemist - Iced UI Showcase")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::TabSelected(tab) => {
                self.selected_tab = tab;
            }
            Message::DashboardUpdate => {
                // Update dashboard metrics
            }
            Message::EventReceived(event) => {
                let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
                self.events.push((timestamp, event));
                if self.events.len() > 100 {
                    self.events.remove(0);
                }
            }
            Message::WorkflowNodeAdded => {
                self.workflow_nodes.push(format!("Node {}", self.workflow_nodes.len() + 1));
            }
            Message::WorkflowNodeRemoved(index) => {
                if index < self.workflow_nodes.len() {
                    self.workflow_nodes.remove(index);
                }
            }
            Message::AIMessageSent(message) => {
                if !message.is_empty() {
                    self.ai_messages.push((true, message.clone()));
                    self.ai_input.clear();
                    // Simulate AI response
                    self.ai_messages.push((false, format!("Processing: {}", message)));
                }
            }
            Message::AIResponseReceived(response) => {
                self.ai_messages.push((false, response));
            }
            Message::SettingChanged(setting) => {
                match setting {
                    SettingType::ThemeChanged(dark) => self.dark_theme = dark,
                    SettingType::AutoSaveToggled(enabled) => self.auto_save = enabled,
                    SettingType::NotificationsToggled(enabled) => self.notifications_enabled = enabled,
                }
            }
            Message::ChartUpdate => {
                // Update performance data
                self.performance_data.push(rand::random::<f32>() * 100.0);
                if self.performance_data.len() > 20 {
                    self.performance_data.remove(0);
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let tab_bar = Row::new()
            .push(self.tab_button("Dashboard", Tab::Dashboard))
            .push(self.tab_button("Events", Tab::EventMonitor))
            .push(self.tab_button("Workflow", Tab::WorkflowEditor))
            .push(self.tab_button("AI Dialog", Tab::AIDialog))
            .push(self.tab_button("Performance", Tab::Performance))
            .push(self.tab_button("Settings", Tab::Settings))
            .spacing(5)
            .padding(10);

        let content = match self.selected_tab {
            Tab::Dashboard => self.dashboard_view(),
            Tab::EventMonitor => self.event_monitor_view(),
            Tab::WorkflowEditor => self.workflow_editor_view(),
            Tab::AIDialog => self.ai_dialog_view(),
            Tab::Performance => self.performance_view(),
            Tab::Settings => self.settings_view(),
        };

        let main_content = Column::new()
            .push(
                Container::new(tab_bar)
                    .width(Length::Fill)
                    .style(|theme: &Theme| {
                        w::container::Appearance {
                            background: Some(iced::Background::Color(
                                if self.dark_theme {
                                    iced::Color::from_rgb(0.15, 0.15, 0.2)
                                } else {
                                    iced::Color::from_rgb(0.9, 0.9, 0.95)
                                }
                            )),
                            ..Default::default()
                        }
                    })
            )
            .push(
                Container::new(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(20)
            );

        Container::new(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        if self.dark_theme {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_secs(1))
            .map(|_| Message::ChartUpdate)
    }
}

impl AlchemistUI {
    fn tab_button(&self, label: &str, tab: Tab) -> Element<Message> {
        let is_selected = self.selected_tab == tab;
        
        w::button(w::text(label).size(16))
            .padding(10)
            .style(move |theme: &Theme, _| {
                w::button::Appearance {
                    background: Some(iced::Background::Color(
                        if is_selected {
                            iced::Color::from_rgb(0.2, 0.6, 0.8)
                        } else {
                            iced::Color::from_rgb(0.3, 0.3, 0.4)
                        }
                    )),
                    text_color: iced::Color::WHITE,
                    border: iced::Border::default(),
                    ..Default::default()
                }
            })
            .on_press(Message::TabSelected(tab))
            .into()
    }

    fn dashboard_view(&self) -> Element<Message> {
        let metrics = Row::new()
            .push(self.metric_card("Active Nodes", "127"))
            .push(self.metric_card("Events/sec", "1,234"))
            .push(self.metric_card("Memory Usage", "45%"))
            .push(self.metric_card("CPU Usage", "32%"))
            .spacing(20);

        let domains_status = Column::new()
            .push(w::text("Domain Status").size(20))
            .push(self.domain_status("Graph Domain", true))
            .push(self.domain_status("Agent Domain", true))
            .push(self.domain_status("Workflow Domain", true))
            .push(self.domain_status("NATS Messaging", true))
            .push(self.domain_status("Document Domain", false))
            .spacing(10);

        Column::new()
            .push(w::text("Alchemist Dashboard").size(32))
            .push(w::Space::with_height(20))
            .push(metrics)
            .push(w::Space::with_height(30))
            .push(domains_status)
            .spacing(10)
            .into()
    }

    fn event_monitor_view(&self) -> Element<Message> {
        let events_list = self.events.iter().rev().fold(
            Column::new().spacing(5),
            |column, (timestamp, event)| {
                column.push(
                    Row::new()
                        .push(w::text(timestamp).size(14).style(w::text::Style::Color(
                            iced::Color::from_rgb(0.6, 0.6, 0.7)
                        )))
                        .push(w::Space::with_width(20))
                        .push(w::text(event).size(14))
                        .spacing(10)
                )
            }
        );

        Column::new()
            .push(w::text("Event Monitor").size(24))
            .push(w::Space::with_height(20))
            .push(
                w::container(
                    Scrollable::new(events_list)
                        .height(Length::Fill)
                )
                .style(|theme: &Theme| {
                    w::container::Appearance {
                        background: Some(iced::Background::Color(
                            iced::Color::from_rgb(0.1, 0.1, 0.15)
                        )),
                        border: iced::Border {
                            color: iced::Color::from_rgb(0.3, 0.3, 0.4),
                            width: 1.0,
                            radius: 5.0.into(),
                        },
                        ..Default::default()
                    }
                })
                .padding(10)
                .height(Length::Fill)
            )
            .into()
    }

    fn workflow_editor_view(&self) -> Element<Message> {
        let nodes = self.workflow_nodes.iter().enumerate().fold(
            Column::new().spacing(10),
            |column, (i, node)| {
                column.push(
                    Row::new()
                        .push(
                            w::container(w::text(node).size(16))
                                .style(|theme: &Theme| {
                                    w::container::Appearance {
                                        background: Some(iced::Background::Color(
                                            iced::Color::from_rgb(0.2, 0.3, 0.5)
                                        )),
                                        border: iced::Border {
                                            color: iced::Color::from_rgb(0.3, 0.4, 0.6),
                                            width: 2.0,
                                            radius: 10.0.into(),
                                        },
                                        ..Default::default()
                                    }
                                })
                                .padding(15)
                                .width(Length::Fixed(200.0))
                        )
                        .push(
                            w::button("Remove")
                                .on_press(Message::WorkflowNodeRemoved(i))
                                .padding(5)
                        )
                        .spacing(10)
                        .align_items(iced::Alignment::Center)
                )
            }
        );

        Column::new()
            .push(w::text("Workflow Editor").size(24))
            .push(w::Space::with_height(20))
            .push(
                w::button("Add Node")
                    .on_press(Message::WorkflowNodeAdded)
                    .padding(10)
            )
            .push(w::Space::with_height(20))
            .push(nodes)
            .spacing(10)
            .into()
    }

    fn ai_dialog_view(&self) -> Element<Message> {
        let messages = self.ai_messages.iter().fold(
            Column::new().spacing(10),
            |column, (is_user, message)| {
                column.push(
                    w::container(
                        w::text(message).size(16)
                    )
                    .style(move |theme: &Theme| {
                        w::container::Appearance {
                            background: Some(iced::Background::Color(
                                if *is_user {
                                    iced::Color::from_rgb(0.2, 0.4, 0.6)
                                } else {
                                    iced::Color::from_rgb(0.3, 0.3, 0.4)
                                }
                            )),
                            border: iced::Border {
                                radius: 10.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    })
                    .padding(15)
                    .width(Length::FillPortion(if *is_user { 7 } else { 9 }))
                )
            }
        );

        Column::new()
            .push(w::text("AI Dialog").size(24))
            .push(w::Space::with_height(20))
            .push(
                Scrollable::new(messages)
                    .height(Length::Fixed(400.0))
            )
            .push(w::Space::with_height(20))
            .push(
                Row::new()
                    .push(
                        w::text_input("Type a message...", &self.ai_input)
                            .padding(10)
                            .width(Length::Fill)
                    )
                    .push(
                        w::button("Send")
                            .on_press(Message::AIMessageSent(self.ai_input.clone()))
                            .padding(10)
                    )
                    .spacing(10)
            )
            .into()
    }

    fn performance_view(&self) -> Element<Message> {
        Column::new()
            .push(w::text("Performance Monitor").size(24))
            .push(w::Space::with_height(20))
            .push(w::text("Real-time performance metrics").size(16))
            .push(w::Space::with_height(20))
            .push(
                Row::new()
                    .push(self.metric_card("Events Processed", "1.2M"))
                    .push(self.metric_card("Avg Response Time", "12ms"))
                    .push(self.metric_card("Error Rate", "0.01%"))
                    .spacing(20)
            )
            .into()
    }

    fn settings_view(&self) -> Element<Message> {
        Column::new()
            .push(w::text("Settings").size(24))
            .push(w::Space::with_height(30))
            .push(
                Row::new()
                    .push(w::text("Dark Theme").size(16).width(Length::Fixed(200.0)))
                    .push(
                        w::checkbox("", self.dark_theme)
                            .on_toggle(|checked| Message::SettingChanged(SettingType::ThemeChanged(checked)))
                    )
                    .align_items(iced::Alignment::Center)
            )
            .push(
                Row::new()
                    .push(w::text("Auto Save").size(16).width(Length::Fixed(200.0)))
                    .push(
                        w::checkbox("", self.auto_save)
                            .on_toggle(|checked| Message::SettingChanged(SettingType::AutoSaveToggled(checked)))
                    )
                    .align_items(iced::Alignment::Center)
            )
            .push(
                Row::new()
                    .push(w::text("Enable Notifications").size(16).width(Length::Fixed(200.0)))
                    .push(
                        w::checkbox("", self.notifications_enabled)
                            .on_toggle(|checked| Message::SettingChanged(SettingType::NotificationsToggled(checked)))
                    )
                    .align_items(iced::Alignment::Center)
            )
            .spacing(20)
            .into()
    }

    fn metric_card(&self, label: &str, value: &str) -> Element<Message> {
        w::container(
            Column::new()
                .push(w::text(label).size(14).style(w::text::Style::Color(
                    iced::Color::from_rgb(0.6, 0.6, 0.7)
                )))
                .push(w::text(value).size(28))
                .spacing(5)
                .align_items(iced::Alignment::Center)
        )
        .style(|theme: &Theme| {
            w::container::Appearance {
                background: Some(iced::Background::Color(
                    iced::Color::from_rgb(0.15, 0.15, 0.2)
                )),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.3, 0.3, 0.4),
                    width: 1.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            }
        })
        .padding(20)
        .width(Length::Fill)
        .into()
    }

    fn domain_status(&self, name: &str, active: bool) -> Element<Message> {
        Row::new()
            .push(
                w::container(w::Space::with_width(10))
                    .style(move |theme: &Theme| {
                        w::container::Appearance {
                            background: Some(iced::Background::Color(
                                if active {
                                    iced::Color::from_rgb(0.2, 0.8, 0.2)
                                } else {
                                    iced::Color::from_rgb(0.8, 0.2, 0.2)
                                }
                            )),
                            border: iced::Border {
                                radius: 5.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    })
                    .width(Length::Fixed(10.0))
                    .height(Length::Fixed(10.0))
            )
            .push(w::text(name).size(16))
            .push(w::text(if active { "Active" } else { "Inactive" }).size(14).style(
                w::text::Style::Color(
                    if active {
                        iced::Color::from_rgb(0.2, 0.8, 0.2)
                    } else {
                        iced::Color::from_rgb(0.8, 0.2, 0.2)
                    }
                )
            ))
            .spacing(10)
            .align_items(iced::Alignment::Center)
            .into()
    }
}

fn main() -> iced::Result {
    AlchemistUI::run(Settings {
        window: window::Settings {
            size: iced::Size::new(1200.0, 800.0),
            position: window::Position::Centered,
            min_size: Some(iced::Size::new(1000.0, 600.0)),
            ..Default::default()
        },
        ..Default::default()
    })
}