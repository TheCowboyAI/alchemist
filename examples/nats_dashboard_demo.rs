//! NATS-connected dashboard demo

use iced::{Element, Task, window, Length, Theme, Alignment};
use iced::widget::{button, column, text, container, row, Space, scrollable};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct DashboardData {
    nats_connected: bool,
    total_events: u64,
    domains: Vec<DomainInfo>,
    recent_events: Vec<EventInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DomainInfo {
    name: String,
    event_count: u64,
    healthy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EventInfo {
    timestamp: String,
    domain: String,
    event_type: String,
}

impl DashboardData {
    fn example() -> Self {
        Self {
            nats_connected: false,
            total_events: 0,
            domains: vec![
                DomainInfo { name: "workflow".to_string(), event_count: 0, healthy: true },
                DomainInfo { name: "agent".to_string(), event_count: 0, healthy: true },
                DomainInfo { name: "document".to_string(), event_count: 0, healthy: true },
            ],
            recent_events: vec![],
        }
    }
}

fn main() -> iced::Result {
    println!("🚀 Starting NATS Dashboard Demo...");
    
    // Create a channel for dashboard updates
    let (tx, rx) = mpsc::channel(100);
    
    // Check for NATS connection
    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    
    // Spawn NATS listener
    let tx_clone = tx.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            println!("Attempting NATS connection to: {}", nats_url);
            
            match async_nats::connect(&nats_url).await {
                Ok(client) => {
                    println!("✅ Connected to NATS!");
                    
                    // Update connection status
                    let mut data = DashboardData::example();
                    data.nats_connected = true;
                    let _ = tx_clone.send(data.clone()).await;
                    
                    // Subscribe to events
                    let mut sub = client.subscribe("cim.>.events.>").await.unwrap();
                    
                    use futures::StreamExt;
                    while let Some(msg) = sub.next().await {
                        // Update data based on message
                        data.total_events += 1;
                        
                        // Parse subject for domain
                        let parts: Vec<&str> = msg.subject.split('.').collect();
                        if parts.len() >= 2 {
                            let domain = parts[1];
                            
                            // Update domain count
                            if let Some(d) = data.domains.iter_mut().find(|d| d.name == domain) {
                                d.event_count += 1;
                            }
                            
                            // Add to recent events
                            data.recent_events.push(EventInfo {
                                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                                domain: domain.to_string(),
                                event_type: parts.get(3).unwrap_or(&"unknown").to_string(),
                            });
                            
                            // Keep only last 20 events
                            if data.recent_events.len() > 20 {
                                data.recent_events.remove(0);
                            }
                        }
                        
                        // Send update
                        let _ = tx_clone.send(data.clone()).await;
                    }
                }
                Err(e) => {
                    println!("❌ NATS connection failed: {}", e);
                    println!("Running in demo mode");
                    
                    // Demo mode - generate fake events
                    let mut data = DashboardData::example();
                    let domains = vec!["workflow", "agent", "document", "policy"];
                    let events = vec!["created", "updated", "deleted", "executed"];
                    
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        
                        data.total_events += 1;
                        let domain = domains[data.total_events as usize % domains.len()];
                        let event = events[data.total_events as usize % events.len()];
                        
                        if let Some(d) = data.domains.iter_mut().find(|d| d.name == domain) {
                            d.event_count += 1;
                        }
                        
                        data.recent_events.push(EventInfo {
                            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                            domain: domain.to_string(),
                            event_type: event.to_string(),
                        });
                        
                        if data.recent_events.len() > 20 {
                            data.recent_events.remove(0);
                        }
                        
                        let _ = tx_clone.send(data.clone()).await;
                    }
                }
            }
        });
    });
    
    // Run the UI
    let initial_state = State {
        data: DashboardData::example(),
        receiver: Some(rx),
    };
    
    iced::application("NATS Dashboard Demo", update, view)
        .subscription(|state| subscription(state))
        .window(window::Settings {
            size: iced::Size::new(1000.0, 700.0),
            position: window::Position::Centered,
            ..Default::default()
        })
        .theme(|_| Theme::Dark)
        .run_with(|| (initial_state, Task::none()))
}

#[derive(Default)]
struct State {
    data: DashboardData,
    receiver: Option<mpsc::Receiver<DashboardData>>,
}

#[derive(Debug, Clone)]
enum Message {
    DataReceived(DashboardData),
    Refresh,
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::DataReceived(data) => {
            state.data = data;
            Task::none()
        }
        Message::Refresh => {
            // Check for new data
            if let Some(rx) = &mut state.receiver {
                if let Ok(data) = rx.try_recv() {
                    state.data = data;
                }
            }
            Task::none()
        }
    }
}

fn view(state: &State) -> Element<'_, Message> {
    let data = &state.data;
    
    let header = container(
        row![
            text("🧪 NATS Dashboard Demo").size(32),
            Space::with_width(Length::Fill),
            text(if data.nats_connected { "NATS ✓" } else { "NATS ✗" })
                .color(if data.nats_connected { 
                    iced::Color::from_rgb(0.0, 1.0, 0.0) 
                } else { 
                    iced::Color::from_rgb(1.0, 0.0, 0.0) 
                }),
        ]
        .align_y(Alignment::Center)
    )
    .padding(20)
    .style(container::rounded_box);

    let status = container(
        column![
            text("System Status").size(24),
            text(format!("Total Events: {}", data.total_events)).size(18),
            text(format!("Connected: {}", data.nats_connected)).size(16),
        ]
        .spacing(10)
    )
    .padding(20)
    .style(container::rounded_box);

    let mut domains_col = column![text("Active Domains").size(24)].spacing(5);
    for domain in &data.domains {
        domains_col = domains_col.push(
            row![
                text(&domain.name).width(Length::Fixed(100.0)),
                text(format!("{} events", domain.event_count)),
                text(if domain.healthy { "✓" } else { "✗" })
                    .color(if domain.healthy { 
                        iced::Color::from_rgb(0.0, 1.0, 0.0) 
                    } else { 
                        iced::Color::from_rgb(1.0, 0.0, 0.0) 
                    }),
            ]
            .spacing(20)
        );
    }
    let domains = container(domains_col)
        .padding(20)
        .style(container::rounded_box);

    let mut events_col = column![text("Recent Events").size(24)].spacing(3);
    for event in &data.recent_events {
        events_col = events_col.push(
            row![
                text(&event.timestamp).size(12).width(Length::Fixed(80.0)),
                text(&event.domain).size(12).width(Length::Fixed(100.0)),
                text(&event.event_type).size(12),
            ]
            .spacing(10)
        );
    }
    let events = container(scrollable(events_col).height(Length::Fixed(300.0)))
        .padding(20)
        .style(container::rounded_box);

    let content = column![
        header,
        row![
            status.width(Length::FillPortion(1)),
            domains.width(Length::FillPortion(2)),
        ]
        .spacing(20),
        events,
    ]
    .spacing(20);

    container(content)
        .padding(30)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn subscription(_state: &State) -> iced::Subscription<Message> {
    // Poll for updates every 100ms
    iced::time::every(std::time::Duration::from_millis(100))
        .map(|_| Message::Refresh)
}