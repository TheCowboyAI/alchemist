//! Dashboard window implementation using Iced
//! This runs in-process instead of spawning a separate renderer

use anyhow::Result;
use iced::{Task, Element, Theme, window, Length, Alignment};
use iced::widget::{column, container, row, text, scrollable, button, Space};
use tokio::sync::mpsc;

use crate::dashboard::DashboardData;

#[derive(Debug, Clone)]
pub enum Message {
    RefreshData,
    DataUpdated(DashboardData),
    CloseWindow,
    DomainClicked(String),
    DialogClicked(String),
    EventClicked(String),
    ToggleNatsConnection,
}

pub struct DashboardWindow {
    data: DashboardData,
    data_receiver: Option<mpsc::Receiver<DashboardData>>,
    selected_domain: Option<String>,
    selected_dialog: Option<String>,
    show_domain_details: bool,
}

impl DashboardWindow {
    pub fn new(data: DashboardData, data_receiver: mpsc::Receiver<DashboardData>) -> (Self, Task<Message>) {
        (
            DashboardWindow {
                data,
                data_receiver: Some(data_receiver),
                selected_domain: None,
                selected_dialog: None,
                show_domain_details: false,
            },
            Task::none()
        )
    }

    pub fn title(&self) -> String {
        "Alchemist - Domain Dashboard".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::RefreshData => {
                // Check for new data
                if let Some(receiver) = &mut self.data_receiver {
                    match receiver.try_recv() {
                        Ok(new_data) => {
                            self.data = new_data;
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                            // No new data, that's fine
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                            // Channel disconnected, remove receiver
                            self.data_receiver = None;
                        }
                    }
                }
                Task::none()
            }
            Message::DataUpdated(data) => {
                self.data = data;
                Task::none()
            }
            Message::CloseWindow => {
                // Graceful shutdown
                std::process::exit(0);
            }
            Message::DomainClicked(domain_name) => {
                if domain_name.is_empty() {
                    // Close button was clicked
                    self.selected_domain = None;
                    self.show_domain_details = false;
                } else if self.selected_domain.as_ref() == Some(&domain_name) {
                    // Same domain clicked again - toggle visibility
                    self.show_domain_details = !self.show_domain_details;
                } else {
                    // Different domain clicked
                    self.selected_domain = Some(domain_name);
                    self.show_domain_details = true;
                }
                Task::none()
            }
            Message::DialogClicked(dialog_id) => {
                self.selected_dialog = Some(dialog_id);
                Task::none()
            }
            Message::EventClicked(_event_id) => {
                // TODO: Show event details
                Task::none()
            }
            Message::ToggleNatsConnection => {
                // TODO: Actually toggle NATS connection
                self.data.system_status.nats_connected = !self.data.system_status.nats_connected;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let header = row![
            text("Alchemist Dashboard").size(28),
            Space::with_width(Length::Fill),
            button("Refresh").on_press(Message::RefreshData),
            button("Close").on_press(Message::CloseWindow),
        ]
        .spacing(10)
        .padding(10)
        .align_y(Alignment::Center);

        // System status
        let status_section = column![
            text("System Status").size(20),
            row![
                text("NATS: "),
                button(
                    text(if self.data.system_status.nats_connected { "Connected" } else { "Disconnected" })
                        .color(if self.data.system_status.nats_connected { 
                            iced::Color::from_rgb(0.0, 1.0, 0.0) 
                        } else { 
                            iced::Color::from_rgb(1.0, 0.0, 0.0) 
                        })
                )
                .on_press(Message::ToggleNatsConnection)
                .style(button::text)
            ].spacing(5),
            text(format!("Total Events: {}", self.data.system_status.total_events)),
            text(format!("Memory Usage: {:.2} MB", self.data.system_status.memory_usage_mb)),
            text(format!("Uptime: {:.0} seconds", self.data.system_status.uptime_seconds)),
        ]
        .spacing(5)
        .padding(10);

        // Domains section
        let domains_section = column![
            text("Active Domains").size(20),
            text(format!("{} domains registered", self.data.domains.len())),
            scrollable(
                column(
                    self.data.domains.iter()
                        .map(|domain| {
                            let domain_name = domain.name.clone();
                            let is_selected = self.selected_domain.as_ref() == Some(&domain_name);
                            
                            button(
                                row![
                                    text(&domain.name).width(Length::Fixed(150.0)),
                                    text(&domain.description).width(Length::Fill),
                                    text(format!("{} events", domain.event_count)).width(Length::Fixed(100.0)),
                                    text(if domain.healthy { "✓" } else { "✗" })
                                        .color(if domain.healthy { 
                                            iced::Color::from_rgb(0.0, 1.0, 0.0) 
                                        } else { 
                                            iced::Color::from_rgb(1.0, 0.0, 0.0) 
                                        })
                                        .width(Length::Fixed(20.0))
                                ]
                                .spacing(10)
                                .padding(5)
                            )
                            .on_press(Message::DomainClicked(domain_name))
                            .style(if is_selected { button::primary } else { button::secondary })
                            .width(Length::Fill)
                            .into()
                        })
                        .collect::<Vec<Element<Message>>>()
                )
                .spacing(5)
            )
            .height(Length::Fixed(200.0))
        ]
        .spacing(5)
        .padding(10);

        // Dialogs section
        let dialogs_section = column![
            text("Active Dialogs").size(20),
            text(format!("{} dialogs", self.data.active_dialogs.len())),
            scrollable(
                column(
                    self.data.active_dialogs.iter()
                        .map(|dialog| {
                            let dialog_id = dialog.id.clone();
                            let is_selected = self.selected_dialog.as_ref() == Some(&dialog_id);
                            
                            button(
                                column![
                                    text(&dialog.title).size(16),
                                    row![
                                        text(format!("Model: {}", dialog.model)).size(12),
                                        Space::with_width(Length::Fill),
                                        text(format!("{} msgs", dialog.message_count)).size(12),
                                    ],
                                    text(&dialog.last_active).size(11)
                                        .color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                                ]
                                .spacing(3)
                                .padding(5)
                                .width(Length::Fill)
                            )
                            .on_press(Message::DialogClicked(dialog_id))
                            .style(if is_selected { button::primary } else { button::secondary })
                            .width(Length::Fill)
                            .into()
                        })
                        .collect::<Vec<Element<Message>>>()
                )
                .spacing(5)
            )
            .height(Length::Fixed(150.0))
        ]
        .spacing(5)
        .padding(10);

        // Recent events section
        let events_section = column![
            text("Recent Events").size(20),
            scrollable(
                column(
                    self.data.recent_events.iter()
                        .map(|event| {
                            row![
                                text(&event.timestamp)
                                    .width(Length::Fixed(80.0)),
                                text(&event.domain)
                                    .width(Length::Fixed(200.0)),
                                text(&event.event_type)
                                    .width(Length::Fixed(150.0)),
                            ]
                            .spacing(10)
                            .into()
                        })
                        .collect::<Vec<Element<Message>>>()
                )
                .spacing(2)
            )
            .height(Length::Fixed(200.0))
        ]
        .spacing(5)
        .padding(10);

        // Domain details panel (shown when a domain is selected)
        let details_panel = if self.show_domain_details && self.selected_domain.is_some() {
            if let Some(selected) = self.selected_domain.as_ref() {
                if let Some(domain) = self.data.domains.iter().find(|d| &d.name == selected) {
                    Some(container(
                        column![
                            row![
                                text(format!("Domain: {}", domain.name)).size(24),
                                Space::with_width(Length::Fill),
                                button("Close").on_press(Message::DomainClicked(String::new())),
                            ],
                            text(&domain.description).size(14),
                            Space::with_height(Length::Fixed(10.0)),
                            text(format!("Total Events: {}", domain.event_count)),
                            text(format!("Health: {:?}", domain.health)),
                            text(format!("Dependencies: {}", domain.dependencies.join(", "))),
                            Space::with_height(Length::Fixed(20.0)),
                            text("Recent Events:").size(16),
                            scrollable(
                                column(
                                    self.data.recent_events.iter()
                                        .filter(|e| e.domain == domain.name)
                                        .take(10)
                                        .map(|event| {
                                            column![
                                                row![
                                                    text(&event.timestamp).size(12),
                                                    text(&event.event_type).size(12),
                                                ],
                                                text(&event.summary).size(11)
                                                    .color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                                            ]
                                            .spacing(2)
                                            .into()
                                        })
                                        .collect::<Vec<Element<Message>>>()
                                )
                                .spacing(5)
                            )
                            .height(Length::Fill),
                        ]
                        .spacing(10)
                        .padding(15)
                    )
                    .width(Length::Fixed(400.0))
                    .height(Length::Fill)
                    .style(container::rounded_box))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Main layout
        let mut main_content = row![
            column![
                container(header)
                    .width(Length::Fill)
                    .style(container::rounded_box),
                row![
                    container(status_section)
                        .width(Length::FillPortion(1))
                        .height(Length::Fill)
                        .style(container::rounded_box),
                    container(domains_section)
                        .width(Length::FillPortion(2))
                        .height(Length::Fill)
                        .style(container::rounded_box),
                ]
                .spacing(10)
                .height(Length::FillPortion(2)),
                row![
                    container(dialogs_section)
                        .width(Length::FillPortion(1))
                        .height(Length::Fill)
                        .style(container::rounded_box),
                    container(events_section)
                        .width(Length::FillPortion(2))
                        .height(Length::Fill)
                        .style(container::rounded_box),
                ]
                .spacing(10)
                .height(Length::FillPortion(3)),
            ]
            .spacing(10)
            .width(Length::Fill)
        ]
        .spacing(10);

        // Add details panel if visible
        if let Some(panel) = details_panel {
            main_content = main_content.push(panel);
        }

        let content = container(main_content)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        // Poll for updates every 100ms
        iced::time::every(std::time::Duration::from_millis(100))
            .map(|_| Message::RefreshData)
    }
}

/// Run the dashboard window
pub async fn run_dashboard_window(
    initial_data: DashboardData,
    data_receiver: mpsc::Receiver<DashboardData>,
) -> Result<()> {
    iced::application(
        DashboardWindow::title,
        DashboardWindow::update,
        DashboardWindow::view
    )
    .subscription(DashboardWindow::subscription)
    .window(window::Settings {
        size: iced::Size::new(1200.0, 800.0),
        position: window::Position::Centered,
        resizable: true,
        ..Default::default()
    })
    .theme(|_| Theme::Dark)
    .run_with(|| DashboardWindow::new(initial_data, data_receiver))
    .map_err(|e| anyhow::anyhow!("Iced error: {}", e))
}