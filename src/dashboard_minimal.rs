//! Minimal dashboard implementation that actually works

use anyhow::Result;
use iced::{Task, Element, Theme, window, Length, Alignment};
use iced::widget::{column, container, row, text, button, Space, scrollable};
use tokio::sync::mpsc;
use crate::dashboard::DashboardData;

#[derive(Debug, Clone)]
pub enum Message {
    Refresh,
    Close,
    DataReceived(DashboardData),
}

pub struct MinimalDashboard {
    event_count: u32,
    data: Option<DashboardData>,
    data_receiver: Option<mpsc::Receiver<DashboardData>>,
}

impl MinimalDashboard {
    pub fn new() -> (Self, Task<Message>) {
        (
            MinimalDashboard {
                event_count: 0,
                data: None,
                data_receiver: None,
            },
            Task::none()
        )
    }

    pub fn with_receiver(data_receiver: mpsc::Receiver<DashboardData>) -> (Self, Task<Message>) {
        (
            MinimalDashboard {
                event_count: 0,
                data: None,
                data_receiver: Some(data_receiver),
            },
            Task::none()
        )
    }

    pub fn title(&self) -> String {
        "Alchemist Dashboard".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Refresh => {
                self.event_count += 1;
                println!("Refreshed! Event count: {}", self.event_count);
                
                // Check for new data
                if let Some(receiver) = &mut self.data_receiver {
                    match receiver.try_recv() {
                        Ok(new_data) => {
                            println!("Received new dashboard data!");
                            self.data = Some(new_data);
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                            // No new data
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                            println!("Data receiver disconnected");
                            self.data_receiver = None;
                        }
                    }
                }
                
                Task::none()
            }
            Message::Close => {
                std::process::exit(0);
            }
            Message::DataReceived(data) => {
                println!("Dashboard data updated!");
                self.data = Some(data);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let header = row![
            text("Alchemist Dashboard").size(28),
            Space::with_width(Length::Fill),
            button("Refresh").on_press(Message::Refresh),
            button("Close").on_press(Message::Close),
        ]
        .spacing(10)
        .padding(10)
        .align_y(Alignment::Center);

        // Show real data if available, otherwise show defaults
        let (nats_status, total_events, domains_list, recent_events) = if let Some(data) = &self.data {
            (
                data.system_status.nats_connected,
                data.system_status.total_events,
                data.domains.clone(),
                data.recent_events.clone(),
            )
        } else {
            (false, self.event_count as u64, vec![], vec![])
        };

        let status = column![
            text("System Status").size(20),
            text(format!("NATS Connected: {}", if nats_status { "Yes ✓" } else { "No ✗" })),
            text(format!("Total Events: {}", total_events)),
            text(format!("Local Events: {}", self.event_count)),
        ]
        .spacing(5)
        .padding(10);

        let mut domains_col = column![text("Domains").size(20)].spacing(5);
        
        if domains_list.is_empty() {
            domains_col = domains_col.push(text("No domains registered yet"));
        } else {
            for domain in domains_list.iter().take(10) {
                let health_icon = if domain.healthy { "✓" } else { "✗" };
                domains_col = domains_col.push(
                    text(format!("• {} - {} events {}", 
                        domain.name, 
                        domain.event_count,
                        health_icon
                    ))
                );
            }
        }
        let domains = container(domains_col.padding(10));

        // Recent events section
        let mut events_col = column![text("Recent Events").size(20)].spacing(3);
        
        if recent_events.is_empty() {
            events_col = events_col.push(text("No events yet"));
        } else {
            for event in recent_events.iter().take(10) {
                events_col = events_col.push(
                    row![
                        text(event.timestamp.clone()).size(12),
                        text(event.domain.clone()).size(12),
                        text(event.event_type.clone()).size(12),
                    ]
                    .spacing(10)
                );
            }
        }
        
        let events = scrollable(events_col.padding(10))
            .height(Length::Fixed(200.0));

        let content = column![
            container(header)
                .width(Length::Fill)
                .style(container::rounded_box),
            row![
                container(status)
                    .width(Length::FillPortion(1))
                    .height(Length::Fill)
                    .style(container::rounded_box),
                container(domains)
                    .width(Length::FillPortion(2))
                    .height(Length::Fill)
                    .style(container::rounded_box),
            ]
            .spacing(10)
            .height(Length::FillPortion(2)),
            container(events)
                .width(Length::Fill)
                .height(Length::FillPortion(1))
                .style(container::rounded_box),
        ]
        .spacing(10);

        container(content)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        // Refresh every 100ms to check for new data
        iced::time::every(std::time::Duration::from_millis(100))
            .map(|_| Message::Refresh)
    }
}

pub async fn run_minimal_dashboard() -> Result<()> {
    println!("Starting minimal dashboard...");
    
    iced::application(
        MinimalDashboard::title,
        MinimalDashboard::update,
        MinimalDashboard::view
    )
    .subscription(MinimalDashboard::subscription)
    .window(window::Settings {
        size: iced::Size::new(900.0, 700.0),
        position: window::Position::Centered,
        resizable: true,
        ..Default::default()
    })
    .theme(|_| Theme::Dark)
    .run_with(MinimalDashboard::new)
    .map_err(|e| anyhow::anyhow!("Iced error: {}", e))
}

pub async fn run_dashboard_with_nats(data_receiver: mpsc::Receiver<DashboardData>) -> Result<()> {
    println!("Starting dashboard with NATS connection...");
    
    iced::application(
        MinimalDashboard::title,
        MinimalDashboard::update,
        MinimalDashboard::view
    )
    .subscription(MinimalDashboard::subscription)
    .window(window::Settings {
        size: iced::Size::new(1000.0, 800.0),
        position: window::Position::Centered,
        resizable: true,
        ..Default::default()
    })
    .theme(|_| Theme::Dark)
    .run_with(|| MinimalDashboard::with_receiver(data_receiver))
    .map_err(|e| anyhow::anyhow!("Iced error: {}", e))
}