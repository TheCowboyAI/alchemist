//! Alchemist UI Demo - Demonstrates working UI functionality
//!
//! This is a concrete demonstration of the Alchemist UI system working.

use iced::{Element, Task, window, Length, Theme};
use iced::widget::{button, column, text, container, row, scrollable, Space};
use std::collections::VecDeque;
use chrono::{DateTime, Local};

fn main() -> iced::Result {
    println!("═══════════════════════════════════════════");
    println!("    🧪 ALCHEMIST UI - Working Demo");
    println!("═══════════════════════════════════════════");
    println!();
    
    iced::application("Alchemist - CIM Control System", App::update, App::view)
        .window(window::Settings {
            size: iced::Size::new(1200.0, 800.0),
            position: window::Position::Centered,
            ..Default::default()
        })
        .theme(|_| Theme::Dark)
        .run()
}

#[derive(Default)]
struct App {
    current_view: View,
    event_log: VecDeque<Event>,
    dialog_messages: Vec<DialogMessage>,
    system_stats: SystemStats,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum View {
    #[default]
    Dashboard,
    Dialog,
    Events,
}

#[derive(Debug, Clone)]
struct Event {
    timestamp: DateTime<Local>,
    domain: String,
    message: String,
}

#[derive(Debug, Clone)]
struct DialogMessage {
    sender: String,
    content: String,
    timestamp: DateTime<Local>,
}

#[derive(Default)]
struct SystemStats {
    events_processed: u64,
    active_domains: u32,
    uptime_seconds: u64,
}

#[derive(Debug, Clone)]
enum Message {
    SwitchView(View),
    AddEvent(String, String),
    SendDialogMessage(String),
    ClearDialog,
    RefreshStats,
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchView(view) => {
                self.current_view = view;
                self.log_event("UI", format!("Switched to {:?} view", view));
            }
            Message::AddEvent(domain, msg) => {
                self.log_event(&domain, msg);
            }
            Message::SendDialogMessage(msg) => {
                if !msg.is_empty() {
                    self.dialog_messages.push(DialogMessage {
                        sender: "User".to_string(),
                        content: msg.clone(),
                        timestamp: Local::now(),
                    });
                    
                    // Simulate AI response
                    self.dialog_messages.push(DialogMessage {
                        sender: "AI Assistant".to_string(),
                        content: format!("Processing: {}", msg),
                        timestamp: Local::now(),
                    });
                    
                    self.log_event("Dialog", format!("User: {}", msg));
                }
            }
            Message::ClearDialog => {
                self.dialog_messages.clear();
                self.log_event("Dialog", "Conversation cleared".to_string());
            }
            Message::RefreshStats => {
                self.system_stats.events_processed = self.event_log.len() as u64;
                self.system_stats.active_domains = 14; // From progress.json
                self.system_stats.uptime_seconds += 1;
            }
        }
        Task::none()
    }
    
    fn log_event(&mut self, domain: &str, message: String) {
        self.event_log.push_front(Event {
            timestamp: Local::now(),
            domain: domain.to_string(),
            message,
        });
        
        // Keep only last 100 events
        if self.event_log.len() > 100 {
            self.event_log.pop_back();
        }
        
        self.system_stats.events_processed += 1;
    }
    
    fn view(&self) -> Element<'_, Message> {
        let header = container(
            row![
                text("🧪 Alchemist Control System").size(28),
                Space::with_width(Length::Fill),
                button("Dashboard").on_press(Message::SwitchView(View::Dashboard)),
                button("Dialog").on_press(Message::SwitchView(View::Dialog)),
                button("Events").on_press(Message::SwitchView(View::Events)),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center)
        )
        .padding(20)
        .style(|_theme| container::background(iced::Color::from_rgb(0.1, 0.1, 0.1)));
        
        let content = match self.current_view {
            View::Dashboard => self.view_dashboard(),
            View::Dialog => self.view_dialog(),
            View::Events => self.view_events(),
        };
        
        column![header, content].into()
    }
    
    fn view_dashboard(&self) -> Element<'_, Message> {
        let stats = column![
            text("System Overview").size(24),
            Space::with_height(20),
            text(format!("Events Processed: {}", self.system_stats.events_processed)),
            text(format!("Active Domains: {}", self.system_stats.active_domains)),
            text(format!("Uptime: {} seconds", self.system_stats.uptime_seconds)),
            Space::with_height(20),
            button("Simulate Graph Event").on_press(Message::AddEvent(
                "Graph".to_string(),
                "Node added to graph".to_string()
            )),
            button("Simulate Dialog Event").on_press(Message::AddEvent(
                "Dialog".to_string(),
                "New conversation started".to_string()
            )),
            button("Simulate Workflow Event").on_press(Message::AddEvent(
                "Workflow".to_string(),
                "Workflow state transition".to_string()
            )),
        ]
        .spacing(10);
        
        container(stats)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
    
    fn view_dialog(&self) -> Element<'_, Message> {
        let messages = self.dialog_messages.iter().map(|msg| {
            let is_user = msg.sender == "User";
            
            container(
                column![
                    text(&msg.sender).size(14),
                    text(&msg.content).size(16),
                    text(msg.timestamp.format("%H:%M:%S").to_string()).size(12),
                ]
                .spacing(5)
            )
            .padding(10)
            .width(Length::Fill)
            .style(move |_theme| {
                if is_user {
                    container::background(iced::Color::from_rgb(0.2, 0.2, 0.4))
                } else {
                    container::background(iced::Color::from_rgb(0.2, 0.4, 0.2))
                }
            })
            .into()
        });
        
        let chat_view = scrollable(
            column(messages)
                .spacing(10)
                .padding(10)
        )
        .height(Length::Fill);
        
        let controls = row![
            button("Send Test Message").on_press(Message::SendDialogMessage(
                "Hello, AI Assistant!".to_string()
            )),
            button("Clear").on_press(Message::ClearDialog),
        ]
        .spacing(10);
        
        column![
            text("Dialog Interface").size(24),
            Space::with_height(10),
            chat_view,
            Space::with_height(10),
            controls,
        ]
        .padding(20)
        .into()
    }
    
    fn view_events(&self) -> Element<'_, Message> {
        let events = self.event_log.iter().take(50).map(|event| {
            container(
                row![
                    text(event.timestamp.format("%H:%M:%S").to_string()).size(14),
                    text(&event.domain).size(14),
                    text(&event.message).size(14),
                ]
                .spacing(20)
            )
            .padding(5)
            .into()
        });
        
        column![
            text("Event Log").size(24),
            Space::with_height(10),
            scrollable(
                column(events)
                    .spacing(5)
                    .padding(10)
            )
            .height(Length::Fill),
        ]
        .padding(20)
        .into()
    }
}

impl Default for DialogMessage {
    fn default() -> Self {
        Self {
            sender: String::new(),
            content: String::new(),
            timestamp: Local::now(),
        }
    }
}