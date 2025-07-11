//! Real-time dashboard application for Iced renderer
//!
//! This dashboard displays domain status and updates in real-time
//! through NATS event subscriptions.

use iced::{
    widget::{button, column, container, row, text, scrollable, Column, Row, Space},
    alignment, Application, Command, Element, Length, Settings, Theme,
    Subscription, subscription, Color, window,
};
use alchemist::dashboard::{DashboardData, DomainInfo, DialogInfo, EventInfo, DomainHealth};
use alchemist::dashboard_realtime::DashboardUpdate;
use alchemist::renderer_api::RendererCommand;
use std::collections::VecDeque;
use chrono::{DateTime, Utc, Local};

pub struct DashboardRealtimeApp {
    /// Current dashboard data
    data: DashboardData,
    
    /// Event log (limited to last 100)
    event_log: VecDeque<EventInfo>,
    
    /// Connection status
    connected: bool,
    
    /// Last update time
    last_update: DateTime<Utc>,
    
    /// Update channel from NATS
    update_receiver: Option<tokio::sync::mpsc::Receiver<DashboardUpdate>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Received dashboard update
    UpdateReceived(DashboardUpdate),
    
    /// Refresh requested
    Refresh,
    
    /// Close window
    Close,
    
    /// Toggle domain details
    ToggleDomain(String),
    
    /// View dialog
    ViewDialog(String),
    
    /// Clear event log
    ClearEvents,
}

impl Application for DashboardRealtimeApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = (DashboardData, tokio::sync::mpsc::Receiver<DashboardUpdate>);

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let (data, update_receiver) = flags;
        
        (
            Self {
                data,
                event_log: VecDeque::with_capacity(100),
                connected: true,
                last_update: Utc::now(),
                update_receiver: Some(update_receiver),
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        format!("Alchemist Dashboard - {} domains active", 
            self.data.domains.iter().filter(|d| d.enabled).count())
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::UpdateReceived(update) => {
                self.last_update = Utc::now();
                self.apply_update(update);
                Command::none()
            }
            
            Message::Refresh => {
                // Request full refresh from main process
                Command::none()
            }
            
            Message::Close => {
                window::close(window::Id::MAIN)
            }
            
            Message::ToggleDomain(_domain) => {
                // TODO: Implement domain detail view
                Command::none()
            }
            
            Message::ViewDialog(_dialog_id) => {
                // TODO: Open dialog viewer
                Command::none()
            }
            
            Message::ClearEvents => {
                self.event_log.clear();
                Command::none()
            }
        }
    }
    
    fn subscription(&self) -> Subscription<Self::Message> {
        struct UpdateSubscription;
        
        subscription::channel(
            std::any::TypeId::of::<UpdateSubscription>(),
            100,
            |mut output| async move {
                // This would receive updates from NATS
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    // In real implementation, receive from update channel
                }
            }
        )
    }

    fn view(&self) -> Element<Self::Message> {
        let header = self.view_header();
        let content = row![
            self.view_domains(),
            self.view_center_panel(),
            self.view_side_panel(),
        ]
        .spacing(20);
        
        container(
            column![
                header,
                content,
                self.view_footer(),
            ]
            .spacing(10)
        )
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

impl DashboardRealtimeApp {
    /// Apply a dashboard update
    fn apply_update(&mut self, update: DashboardUpdate) {
        match update {
            DashboardUpdate::FullUpdate(data) => {
                self.data = data;
            }
            
            DashboardUpdate::DomainUpdate { domain, info } => {
                if let Some(d) = self.data.domains.iter_mut().find(|d| d.name == domain) {
                    *d = info;
                }
            }
            
            DashboardUpdate::EventAdded(event) => {
                self.event_log.push_front(event.clone());
                if self.event_log.len() > 100 {
                    self.event_log.pop_back();
                }
                
                // Also add to recent events
                self.data.recent_events.insert(0, event);
                if self.data.recent_events.len() > 20 {
                    self.data.recent_events.pop();
                }
            }
            
            DashboardUpdate::DialogUpdate { dialog_id, info } => {
                if let Some(info) = info {
                    if let Some(d) = self.data.active_dialogs.iter_mut().find(|d| d.id == dialog_id) {
                        *d = info;
                    } else {
                        self.data.active_dialogs.push(info);
                    }
                } else {
                    // Remove dialog
                    self.data.active_dialogs.retain(|d| d.id != dialog_id);
                }
            }
            
            DashboardUpdate::MetricsUpdate { total_events, domains } => {
                self.data.system_status.total_events = total_events;
                
                // Update domain event counts
                for (domain_name, count) in domains {
                    if let Some(d) = self.data.domains.iter_mut().find(|d| d.name == domain_name) {
                        d.event_count = count;
                    }
                }
            }
            
            DashboardUpdate::PolicyUpdate { policy_id, info } => {
                if let Some(info) = info {
                    if let Some(p) = self.data.active_policies.iter_mut().find(|p| p.name == policy_id) {
                        *p = info;
                    } else {
                        self.data.active_policies.push(info);
                    }
                } else {
                    // Remove policy
                    self.data.active_policies.retain(|p| p.name != policy_id);
                }
            }
        }
    }
    
    /// View header section
    fn view_header(&self) -> Element<Message> {
        let status_icon = if self.connected { "🟢" } else { "🔴" };
        let last_update = self.last_update.with_timezone(&Local).format("%H:%M:%S");
        
        container(
            row![
                text("Alchemist Domain Dashboard").size(28),
                Space::with_width(Length::Fill),
                text(format!("{} Connected", status_icon)).size(14),
                text("|").size(14),
                text(format!("Last update: {}", last_update)).size(14),
                button("Refresh").on_press(Message::Refresh),
                button("Close").on_press(Message::Close),
            ]
            .spacing(10)
            .align_y(alignment::Vertical::Center)
        )
        .padding(10)
        .style(|theme: &Theme| {
            container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
                border: iced::Border {
                    color: Color::from_rgb(0.3, 0.3, 0.3),
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
    }
    
    /// View domains panel
    fn view_domains(&self) -> Element<Message> {
        let mut domain_list = Column::new()
            .spacing(10)
            .push(text("Domains").size(20));
        
        for domain in &self.data.domains {
            let health_icon = match &domain.health {
                DomainHealth::Healthy => "✅",
                DomainHealth::Warning(_) => "⚠️",
                DomainHealth::Error(_) => "❌",
                DomainHealth::Unknown => "❓",
            };
            
            let domain_card = container(
                column![
                    row![
                        text(health_icon).size(16),
                        text(&domain.name).size(16),
                        Space::with_width(Length::Fill),
                        text(format!("{}", domain.event_count)).size(14),
                    ]
                    .spacing(5),
                    text(&domain.description).size(12),
                ]
                .spacing(5)
            )
            .padding(10)
            .width(Length::Fill)
            .style(|theme: &Theme| {
                container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
                    border: iced::Border {
                        color: Color::from_rgb(0.3, 0.3, 0.3),
                        width: 1.0,
                        radius: 5.0.into(),
                    },
                    ..Default::default()
                }
            });
            
            domain_list = domain_list.push(domain_card);
        }
        
        container(scrollable(domain_list))
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .into()
    }
    
    /// View center panel (events)
    fn view_center_panel(&self) -> Element<Message> {
        let mut event_list = Column::new()
            .spacing(5)
            .push(
                row![
                    text("Recent Events").size(20),
                    Space::with_width(Length::Fill),
                    button("Clear").on_press(Message::ClearEvents),
                ]
            );
        
        for event in &self.data.recent_events {
            let event_row = row![
                text(&event.timestamp).size(12),
                text(&event.domain).size(12),
                text(&event.event_type).size(12),
                text(&event.summary).size(12),
            ]
            .spacing(10);
            
            event_list = event_list.push(event_row);
        }
        
        container(scrollable(event_list))
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .padding(10)
            .style(|theme: &Theme| {
                container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.05, 0.05, 0.05))),
                    ..Default::default()
                }
            })
            .into()
    }
    
    /// View side panel (dialogs and policies)
    fn view_side_panel(&self) -> Element<Message> {
        let mut panel = Column::new().spacing(20);
        
        // Active dialogs
        let mut dialogs = Column::new()
            .spacing(5)
            .push(text("Active Dialogs").size(18));
        
        for dialog in &self.data.active_dialogs {
            let dialog_card = container(
                column![
                    text(&dialog.title).size(14),
                    row![
                        text(&dialog.model).size(12),
                        text("|").size(12),
                        text(format!("{} msgs", dialog.message_count)).size(12),
                    ]
                    .spacing(5),
                    text(&dialog.last_active).size(10),
                ]
                .spacing(3)
            )
            .padding(8)
            .style(|theme: &Theme| {
                container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.15))),
                    border: iced::Border {
                        color: Color::from_rgb(0.3, 0.3, 0.4),
                        width: 1.0,
                        radius: 3.0.into(),
                    },
                    ..Default::default()
                }
            });
            
            dialogs = dialogs.push(dialog_card);
        }
        
        panel = panel.push(dialogs);
        
        // Active policies
        let mut policies = Column::new()
            .spacing(5)
            .push(text("Active Policies").size(18));
        
        for policy in &self.data.active_policies {
            let status = if policy.enabled { "✅" } else { "⏸️" };
            
            policies = policies.push(
                row![
                    text(status).size(14),
                    text(&policy.name).size(14),
                    text(format!("({} rules)", policy.rules_count)).size(12),
                ]
                .spacing(5)
            );
        }
        
        panel = panel.push(policies);
        
        container(scrollable(panel))
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .into()
    }
    
    /// View footer with system status
    fn view_footer(&self) -> Element<Message> {
        let status = &self.data.system_status;
        
        container(
            row![
                text(format!("Total Events: {}", status.total_events)).size(12),
                text("|").size(12),
                text(format!("Memory: {:.1}%", status.memory_usage)).size(12),
                text("|").size(12),
                text(format!("Uptime: {}", status.uptime)).size(12),
            ]
            .spacing(10)
        )
        .padding(5)
        .style(|theme: &Theme| {
            container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.05, 0.05, 0.05))),
                border: iced::Border {
                    color: Color::from_rgb(0.2, 0.2, 0.2),
                    width: 1.0,
                    radius: 3.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
    }
}