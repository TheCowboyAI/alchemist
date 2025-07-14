//! Event visualization component for domain events
//!
//! This module provides real-time visualization of CIM domain events
//! flowing through the system via NATS.

use anyhow::Result;
use iced::{Task, Element, Theme, window, Length, Alignment, Color};
use iced::widget::{column, container, row, text, button, scrollable, Space};
use tokio::sync::mpsc;
use std::collections::{VecDeque, HashMap};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use async_nats::Client;
use uuid::Uuid;
use futures::StreamExt;

/// Maximum events to display
const MAX_EVENTS: usize = 100;
const MAX_GRAPH_POINTS: usize = 60; // 60 seconds of data

#[derive(Debug, Clone)]
pub enum Message {
    EventReceived(DomainEvent),
    RefreshStats,
    ClearEvents,
    TogglePause,
    FilterDomain(Option<String>),
    Close,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub timestamp: DateTime<Utc>,
    pub domain: String,
    pub event_type: String,
    pub aggregate_id: String,
    pub payload: serde_json::Value,
    pub correlation_id: Option<String>,
}

#[derive(Debug, Default)]
pub struct EventStats {
    pub total_events: u64,
    pub events_per_domain: HashMap<String, u64>,
    pub events_per_type: HashMap<String, u64>,
    pub events_per_second: VecDeque<(DateTime<Utc>, u32)>,
}

pub struct EventVisualizer {
    events: VecDeque<DomainEvent>,
    stats: EventStats,
    event_receiver: Option<mpsc::Receiver<DomainEvent>>,
    is_paused: bool,
    domain_filter: Option<String>,
    available_domains: Vec<String>,
    nats_connected: bool,
}

impl EventVisualizer {
    pub fn new() -> (Self, Task<Message>) {
        // Try to connect to NATS
        let (event_tx, event_rx) = mpsc::channel(100);
        
        let nats_url = std::env::var("NATS_URL")
            .unwrap_or_else(|_| "nats://localhost:4222".to_string());
        
        let nats_connected = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match async_nats::connect(&nats_url).await {
                    Ok(client) => {
                        spawn_event_listener(client, event_tx);
                        true
                    }
                    Err(e) => {
                        eprintln!("Could not connect to NATS: {} - using demo mode", e);
                        spawn_demo_events(event_tx);
                        false
                    }
                }
            })
        });
        
        (
            EventVisualizer {
                events: VecDeque::with_capacity(MAX_EVENTS),
                stats: EventStats::default(),
                event_receiver: Some(event_rx),
                is_paused: false,
                domain_filter: None,
                available_domains: vec![
                    "workflow".to_string(),
                    "document".to_string(),
                    "location".to_string(),
                    "nix".to_string(),
                ],
                nats_connected,
            },
            Task::none()
        )
    }
    
    pub fn title(&self) -> String {
        "CIM Event Visualizer".to_string()
    }
    
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::EventReceived(_) => {
                if !self.is_paused {
                    self.process_incoming_events();
                }
                Task::none()
            }
            
            Message::RefreshStats => {
                self.update_stats();
                Task::none()
            }
            
            Message::ClearEvents => {
                self.events.clear();
                self.stats = EventStats::default();
                Task::none()
            }
            
            Message::TogglePause => {
                self.is_paused = !self.is_paused;
                Task::none()
            }
            
            Message::FilterDomain(domain) => {
                self.domain_filter = domain;
                Task::none()
            }
            
            Message::Close => {
                std::process::exit(0);
            }
        }
    }
    
    fn process_incoming_events(&mut self) {
        if let Some(receiver) = &mut self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                // Update stats
                self.stats.total_events += 1;
                *self.stats.events_per_domain
                    .entry(event.domain.clone())
                    .or_insert(0) += 1;
                *self.stats.events_per_type
                    .entry(event.event_type.clone())
                    .or_insert(0) += 1;
                
                // Add to available domains if new
                if !self.available_domains.contains(&event.domain) {
                    self.available_domains.push(event.domain.clone());
                }
                
                // Apply filter
                if let Some(filter) = &self.domain_filter {
                    if &event.domain != filter {
                        continue;
                    }
                }
                
                // Add event
                self.events.push_front(event);
                if self.events.len() > MAX_EVENTS {
                    self.events.pop_back();
                }
            }
        }
    }
    
    fn update_stats(&mut self) {
        let now = Utc::now();
        
        // Count events in the last second
        let one_second_ago = now - chrono::Duration::seconds(1);
        let events_last_second = self.events.iter()
            .filter(|e| e.timestamp > one_second_ago)
            .count() as u32;
        
        self.stats.events_per_second.push_back((now, events_last_second));
        
        // Keep only last 60 seconds
        while self.stats.events_per_second.len() > MAX_GRAPH_POINTS {
            self.stats.events_per_second.pop_front();
        }
    }
    
    pub fn view(&self) -> Element<Message> {
        let header = self.view_header();
        let stats_panel = self.view_stats();
        let events_list = self.view_events();
        
        let content = column![
            header,
            row![
                container(stats_panel)
                    .width(Length::Fixed(300.0))
                    .height(Length::Fill)
                    .style(container::rounded_box),
                container(events_list)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(container::rounded_box),
            ]
            .spacing(10),
        ]
        .spacing(10);
        
        container(content)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
    
    fn view_header(&self) -> Element<Message> {
        container(
            row![
                text("📊 CIM Event Visualizer").size(24),
                Space::with_width(Length::Fill),
                text(if self.nats_connected { "NATS ✓" } else { "Demo Mode" })
                    .color(if self.nats_connected { 
                        Color::from_rgb(0.0, 1.0, 0.0) 
                    } else { 
                        Color::from_rgb(1.0, 0.5, 0.0) 
                    }),
                button(if self.is_paused { "▶ Resume" } else { "⏸ Pause" })
                    .on_press(Message::TogglePause)
                    .style(button::secondary),
                button("Clear").on_press(Message::ClearEvents),
                button("Close").on_press(Message::Close),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        )
        .padding(10)
        .style(container::rounded_box)
        .into()
    }
    
    fn view_stats(&self) -> Element<Message> {
        let mut stats_col = column![
            text("Statistics").size(20),
            Space::with_height(Length::Fixed(10.0)),
            text(format!("Total Events: {}", self.stats.total_events)),
            Space::with_height(Length::Fixed(20.0)),
        ]
        .spacing(5);
        
        // Domain filter
        stats_col = stats_col.push(text("Filter by Domain:").size(14));
        stats_col = stats_col.push(
            button("All Domains")
                .on_press(Message::FilterDomain(None))
                .style(if self.domain_filter.is_none() {
                    button::primary
                } else {
                    button::secondary
                })
                .width(Length::Fill)
        );
        
        for domain in &self.available_domains {
            let count = self.stats.events_per_domain.get(domain).unwrap_or(&0);
            let is_selected = self.domain_filter.as_ref() == Some(domain);
            
            stats_col = stats_col.push(
                button(text(format!("{}: {}", domain, count)).size(14))
                    .on_press(Message::FilterDomain(Some(domain.clone())))
                    .style(if is_selected { button::primary } else { button::secondary })
                    .width(Length::Fill)
            );
        }
        
        stats_col = stats_col.push(Space::with_height(Length::Fixed(20.0)));
        stats_col = stats_col.push(text("Event Types:").size(14));
        
        // Show top event types
        let mut event_types: Vec<_> = self.stats.events_per_type.iter().collect();
        event_types.sort_by(|a, b| b.1.cmp(a.1));
        
        for (event_type, count) in event_types.iter().take(5) {
            stats_col = stats_col.push(
                text(format!("{}: {}", event_type, count))
                    .size(12)
                    .color(Color::from_rgb(0.7, 0.7, 0.7))
            );
        }
        
        scrollable(stats_col.padding(10))
            .height(Length::Fill)
            .into()
    }
    
    fn view_events(&self) -> Element<Message> {
        let mut events_col = column![
            text("Recent Events").size(20),
            Space::with_height(Length::Fixed(10.0)),
        ]
        .spacing(5);
        
        for event in &self.events {
            let event_box = container(
                column![
                    row![
                        text(&event.domain)
                            .size(14)
                            .color(Color::from_rgb(0.5, 0.8, 1.0)),
                        text(" | "),
                        text(&event.event_type)
                            .size(14)
                            .color(Color::from_rgb(0.8, 0.8, 0.5)),
                        Space::with_width(Length::Fill),
                        text(event.timestamp.format("%H:%M:%S").to_string())
                            .size(12)
                            .color(Color::from_rgb(0.6, 0.6, 0.6)),
                    ],
                    text(format!("ID: {}", &event.aggregate_id))
                        .size(11)
                        .color(Color::from_rgb(0.5, 0.5, 0.5)),
                    if let Some(corr_id) = &event.correlation_id {
                        text(format!("Correlation: {}", corr_id))
                            .size(10)
                            .color(Color::from_rgb(0.4, 0.4, 0.4))
                    } else {
                        text("")
                    },
                ]
                .spacing(2)
            )
            .padding(8)
            .style(container::bordered_box);
            
            events_col = events_col.push(event_box);
        }
        
        if self.events.is_empty() {
            events_col = events_col.push(
                container(
                    text(if self.is_paused {
                        "Paused - No events"
                    } else {
                        "Waiting for events..."
                    })
                    .size(16)
                    .color(Color::from_rgb(0.6, 0.6, 0.6))
                )
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .height(Length::Fixed(200.0))
            );
        }
        
        scrollable(events_col.padding(10))
            .height(Length::Fill)
            .into()
    }
    
    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch([
            // Poll for events
            iced::time::every(std::time::Duration::from_millis(100))
                .map(|_| Message::EventReceived(DomainEvent {
                    timestamp: Utc::now(),
                    domain: String::new(),
                    event_type: String::new(),
                    aggregate_id: String::new(),
                    payload: serde_json::Value::Null,
                    correlation_id: None,
                })),
            // Update stats
            iced::time::every(std::time::Duration::from_secs(1))
                .map(|_| Message::RefreshStats),
        ])
    }
}

fn spawn_event_listener(client: Client, tx: mpsc::Sender<DomainEvent>) {
    tokio::spawn(async move {
        // Subscribe to all CIM event subjects
        let subjects = [
            "cim.workflow.>",
            "cim.document.>",
            "cim.location.>",
            "cim.nix.>",
        ];
        
        for subject in &subjects {
            let mut sub = match client.subscribe(subject.to_string()).await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to subscribe to {}: {}", subject, e);
                    continue;
                }
            };
            
            let tx = tx.clone();
            tokio::spawn(async move {
                while let Some(msg) = sub.next().await {
                    if let Ok(event) = serde_json::from_slice::<DomainEvent>(&msg.payload) {
                        let _ = tx.send(event).await;
                    }
                }
            });
        }
    });
}

fn spawn_demo_events(tx: mpsc::Sender<DomainEvent>) {
    tokio::spawn(async move {
        let domains = ["workflow", "document", "location", "nix"];
        let event_types = [
            ("created", "aggregate.created"),
            ("updated", "aggregate.updated"),
            ("deployed", "deployment.started"),
            ("completed", "task.completed"),
        ];
        
        loop {
            // Generate random event using simple hash-based randomness
            let now = Utc::now();
            let random_val = now.timestamp_millis() as usize;
            
            let domain = domains[random_val % domains.len()];
            let (_, event_type) = event_types[(random_val / 10) % event_types.len()];
            
            let event = DomainEvent {
                timestamp: now,
                domain: domain.to_string(),
                event_type: event_type.to_string(),
                aggregate_id: format!("{}-{}", domain, Uuid::new_v4()),
                payload: serde_json::json!({
                    "demo": true,
                    "value": random_val % 100,
                }),
                correlation_id: if random_val % 2 == 0 {
                    Some(Uuid::new_v4().to_string())
                } else {
                    None
                },
            };
            
            let _ = tx.send(event).await;
            
            // Variable delay between events
            let delay_ms = 100 + (random_val % 2000);
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms as u64)).await;
        }
    });
}

pub async fn run_event_visualizer() -> Result<()> {
    println!("Starting CIM Event Visualizer...");
    
    iced::application(
        EventVisualizer::title,
        EventVisualizer::update,
        EventVisualizer::view
    )
    .subscription(EventVisualizer::subscription)
    .window(window::Settings {
        size: iced::Size::new(1000.0, 700.0),
        position: window::Position::Centered,
        resizable: true,
        ..Default::default()
    })
    .theme(|_| Theme::Dark)
    .run_with(EventVisualizer::new)
    .map_err(|e| anyhow::anyhow!("Iced error: {}", e))
}