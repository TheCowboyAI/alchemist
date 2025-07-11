//! NATS message flow visualizer
//!
//! Real-time visualization of NATS message flow between subjects,
//! showing publishers, subscribers, and message routing.

use anyhow::Result;
use iced::{Task, Element, Theme, window, Length, Alignment, Color, Point, Vector, Size, Rectangle};
use iced::widget::{column, container, row, text, button, scrollable, Space, Canvas, pick_list};
use iced::widget::canvas::{self, Cache, Frame, Geometry, Path, Stroke, Fill};
use tokio::sync::mpsc;
use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use async_nats::Client;
use futures::StreamExt;

const NODE_RADIUS: f32 = 40.0;
const MESSAGE_SPEED: f32 = 200.0; // pixels per second
const MAX_MESSAGES: usize = 50;

#[derive(Debug, Clone)]
pub enum Message {
    MessageReceived(NatsMessage),
    UpdateAnimation,
    TogglePause,
    ClearMessages,
    FilterSubject(Option<String>),
    ShowDetails(String),
    Close,
    NatsConnected(Option<async_nats::Client>, bool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsMessage {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub subject: String,
    pub payload_size: usize,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SubjectNode {
    pub subject: String,
    pub position: Point,
    pub publishers: u32,
    pub subscribers: u32,
    pub message_count: u64,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MessageAnimation {
    pub message: NatsMessage,
    pub from_pos: Point,
    pub to_pos: Point,
    pub progress: f32,
    pub start_time: std::time::Instant,
}

pub struct NatsFlowVisualizer {
    client: Option<Client>,
    subjects: HashMap<String, SubjectNode>,
    messages: VecDeque<NatsMessage>,
    animations: Vec<MessageAnimation>,
    is_paused: bool,
    subject_filter: Option<String>,
    selected_message: Option<String>,
    cache: Cache,
    nats_connected: bool,
    layout_radius: f32,
}

impl NatsFlowVisualizer {
    pub fn new() -> (Self, Task<Message>) {
        let nats_url = std::env::var("NATS_URL")
            .unwrap_or_else(|_| "nats://localhost:4222".to_string());
        
        // We'll connect to NATS in a task
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        
        // Spawn connection task
        let nats_url_clone = nats_url.clone();
        tokio::spawn(async move {
            match async_nats::connect(&nats_url_clone).await {
                Ok(client) => {
                    println!("Connected to NATS at {}", nats_url_clone);
                    let _ = tx.send((Some(client), true)).await;
                }
                Err(e) => {
                    eprintln!("Could not connect to NATS: {} - using demo mode", e);
                    let _ = tx.send((None, false)).await;
                }
            }
        });
        
        // For now, start disconnected
        let (client, nats_connected) = (None, false);
        
        let mut visualizer = NatsFlowVisualizer {
            client,
            subjects: HashMap::new(),
            messages: VecDeque::with_capacity(MAX_MESSAGES),
            animations: Vec::new(),
            is_paused: false,
            subject_filter: None,
            selected_message: None,
            cache: Cache::default(),
            nats_connected,
            layout_radius: 200.0,
        };
        
        // Initialize with known CIM subjects
        let cim_subjects = vec![
            "cim.workflow.>",
            "cim.document.>",
            "cim.location.>",
            "cim.nix.>",
            "cim.renderer.>",
            "cim.dialog.>",
            "cim.dashboard.>",
            "cim.system.>",
        ];
        
        for (i, subject) in cim_subjects.iter().enumerate() {
            let angle = (i as f32 / cim_subjects.len() as f32) * 2.0 * std::f32::consts::PI;
            let x = 400.0 + visualizer.layout_radius * angle.cos();
            let y = 300.0 + visualizer.layout_radius * angle.sin();
            
            visualizer.subjects.insert(
                subject.to_string(),
                SubjectNode {
                    subject: subject.to_string(),
                    position: Point::new(x, y),
                    publishers: 0,
                    subscribers: 0,
                    message_count: 0,
                    last_activity: Utc::now(),
                },
            );
        }
        
        // We'll start the message listener once we get the client from the connection task
        let task = Task::perform(
            async move {
                // Wait for connection result
                if let Some((client, connected)) = rx.recv().await {
                    Message::NatsConnected(client, connected)
                } else {
                    Message::NatsConnected(None, false)
                }
            },
            |msg| msg
        );
        
        // Start demo messages if not connected
        if !visualizer.nats_connected {
            visualizer.start_demo_messages();
        }
        
        (visualizer, task)
    }
    
    async fn nats_listener_task(client: async_nats::Client, tx: mpsc::Sender<NatsMessage>) {
            let mut sub = match client.subscribe(">").await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to subscribe to all subjects: {}", e);
                    return;
                }
            };
            
            tokio::spawn(async move {
                while let Some(msg) = sub.next().await {
                    let nats_msg = NatsMessage {
                        id: uuid::Uuid::new_v4().to_string(),
                        timestamp: Utc::now(),
                        subject: msg.subject.to_string(),
                        payload_size: msg.payload.len(),
                        headers: msg.headers.map(|h| {
                            h.iter()
                                .map(|(k, v)| (k.to_string(), format!("{:?}", v)))
                                .collect()
                        }).unwrap_or_default(),
                    };
                    
                    let _ = tx.send(nats_msg).await;
                }
            });
    }
    
    fn start_demo_messages(&self) {
        // Demo messages will be generated in the update loop
    }
    
    pub fn title(&self) -> String {
        "NATS Message Flow Visualizer".to_string()
    }
    
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MessageReceived(msg) => {
                if !self.is_paused {
                    self.process_message(msg);
                    self.cache.clear();
                }
                Task::none()
            }
            
            Message::UpdateAnimation => {
                if !self.is_paused {
                    self.update_animations();
                    
                    // Generate demo messages if not connected
                    if !self.nats_connected && self.animations.len() < 3 {
                        self.generate_demo_message();
                    }
                    
                    self.cache.clear();
                }
                Task::none()
            }
            
            Message::TogglePause => {
                self.is_paused = !self.is_paused;
                Task::none()
            }
            
            Message::ClearMessages => {
                self.messages.clear();
                self.animations.clear();
                for subject in self.subjects.values_mut() {
                    subject.message_count = 0;
                }
                self.cache.clear();
                Task::none()
            }
            
            Message::FilterSubject(filter) => {
                self.subject_filter = filter;
                Task::none()
            }
            
            Message::ShowDetails(msg_id) => {
                self.selected_message = Some(msg_id);
                Task::none()
            }
            
            Message::Close => {
                std::process::exit(0);
            }
            
            Message::NatsConnected(client, connected) => {
                self.client = client;
                self.nats_connected = connected;
                
                // If connected, start the listener
                if let Some(ref client) = self.client {
                    let (msg_tx, mut msg_rx) = mpsc::channel(100);
                    let client_clone = client.clone();
                    
                    // Start listener in background
                    tokio::spawn(async move {
                        Self::nats_listener_task(client_clone, msg_tx).await;
                    });
                    
                    // Return task to receive messages
                    Task::perform(
                        async move {
                            if let Some(msg) = msg_rx.recv().await {
                                Message::MessageReceived(msg)
                            } else {
                                Message::UpdateAnimation
                            }
                        },
                        |msg| msg
                    )
                } else {
                    Task::none()
                }
            }
        }
    }
    
    fn process_message(&mut self, msg: NatsMessage) {
        // Find matching subject pattern
        let subject_pattern = self.subjects.keys()
            .find(|pattern| self.matches_pattern(&msg.subject, pattern))
            .cloned();
        
        if let Some(pattern) = subject_pattern {
            if let Some(node) = self.subjects.get_mut(&pattern) {
                node.message_count += 1;
                node.last_activity = msg.timestamp;
                
                // Create animation from center to subject
                let from_pos = Point::new(400.0, 300.0);
                let to_pos = node.position;
                
                self.animations.push(MessageAnimation {
                    message: msg.clone(),
                    from_pos,
                    to_pos,
                    progress: 0.0,
                    start_time: std::time::Instant::now(),
                });
            }
        }
        
        self.messages.push_front(msg);
        if self.messages.len() > MAX_MESSAGES {
            self.messages.pop_back();
        }
    }
    
    fn matches_pattern(&self, subject: &str, pattern: &str) -> bool {
        if pattern.ends_with(">") {
            let prefix = &pattern[..pattern.len() - 1];
            subject.starts_with(prefix)
        } else {
            subject == pattern
        }
    }
    
    fn update_animations(&mut self) {
        let now = std::time::Instant::now();
        
        self.animations.retain_mut(|anim| {
            let elapsed = now.duration_since(anim.start_time).as_secs_f32();
            let distance = ((anim.to_pos.x - anim.from_pos.x).powi(2) + 
                           (anim.to_pos.y - anim.from_pos.y).powi(2)).sqrt();
            let duration = distance / MESSAGE_SPEED;
            
            anim.progress = (elapsed / duration).min(1.0);
            
            anim.progress < 1.0
        });
    }
    
    fn generate_demo_message(&mut self) {
        let subjects = vec![
            "cim.workflow.created",
            "cim.workflow.updated",
            "cim.document.saved",
            "cim.document.deleted",
            "cim.location.changed",
            "cim.nix.deployed",
            "cim.dialog.message",
            "cim.dashboard.update",
            "cim.renderer.event",
            "cim.system.status",
        ];
        
        let now = Utc::now();
        let subject_idx = (now.timestamp_millis() as usize) % subjects.len();
        
        let msg = NatsMessage {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: now,
            subject: subjects[subject_idx].to_string(),
            payload_size: 100 + (now.timestamp_millis() as usize % 900),
            headers: HashMap::new(),
        };
        
        self.process_message(msg);
    }
    
    pub fn view(&self) -> Element<Message> {
        let header = self.view_header();
        let main_content = row![
            container(self.view_canvas())
                .width(Length::FillPortion(3))
                .height(Length::Fill)
                .style(container::rounded_box),
            container(self.view_sidebar())
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .style(container::rounded_box),
        ]
        .spacing(10);
        
        container(
            column![
                header,
                main_content,
            ]
            .spacing(10)
        )
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    fn view_header(&self) -> Element<Message> {
        container(
            row![
                text("📡 NATS Message Flow").size(24),
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
                button("Clear").on_press(Message::ClearMessages),
                button("Close").on_press(Message::Close),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        )
        .padding(10)
        .style(container::rounded_box)
        .into()
    }
    
    fn view_sidebar(&self) -> Element<Message> {
        let mut sidebar = column![
            text("Recent Messages").size(20),
            Space::with_height(Length::Fixed(10.0)),
        ]
        .spacing(5);
        
        // Message stats
        let total_messages: u64 = self.subjects.values()
            .map(|s| s.message_count)
            .sum();
        
        sidebar = sidebar.push(
            container(
                column![
                    text(format!("Total Messages: {}", total_messages)),
                    text(format!("Active Subjects: {}", self.subjects.len())),
                    text(format!("Animations: {}", self.animations.len())),
                ]
                .spacing(2)
            )
            .padding(10)
            .style(container::bordered_box)
        );
        
        sidebar = sidebar.push(Space::with_height(Length::Fixed(20.0)));
        
        // Subject filter
        let subjects: Vec<String> = self.subjects.keys().cloned().collect();
        sidebar = sidebar.push(
            pick_list(
                subjects,
                self.subject_filter.clone(),
                |s| Message::FilterSubject(Some(s)),
            )
            .placeholder("Filter by subject...")
        );
        
        sidebar = sidebar.push(Space::with_height(Length::Fixed(10.0)));
        
        // Recent messages
        for msg in self.messages.iter().take(20) {
            if let Some(filter) = &self.subject_filter {
                if !self.matches_pattern(&msg.subject, filter) {
                    continue;
                }
            }
            
            let is_selected = self.selected_message.as_ref() == Some(&msg.id);
            
            let msg_button = button(
                column![
                    text(&msg.subject)
                        .size(12)
                        .color(Color::from_rgb(0.7, 0.9, 1.0)),
                    text(format!("{} bytes", msg.payload_size))
                        .size(10)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                    text(msg.timestamp.format("%H:%M:%S").to_string())
                        .size(10)
                        .color(Color::from_rgb(0.5, 0.5, 0.5)),
                ]
                .spacing(2)
            )
            .on_press(Message::ShowDetails(msg.id.clone()))
            .style(if is_selected { button::primary } else { button::text })
            .width(Length::Fill);
            
            sidebar = sidebar.push(msg_button);
        }
        
        scrollable(sidebar.padding(10))
            .height(Length::Fill)
            .into()
    }
    
    fn view_canvas(&self) -> Element<Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
    
    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(50))
            .map(|_| Message::UpdateAnimation)
    }
}

impl<Message> canvas::Program<Message> for NatsFlowVisualizer {
    type State = ();
    
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        
        // Draw center hub
        frame.fill(
            &Path::circle(center, 30.0),
            Color::from_rgb(0.2, 0.2, 0.2),
        );
        frame.stroke(
            &Path::circle(center, 30.0),
            Stroke::default()
                .with_color(Color::from_rgb(0.4, 0.4, 0.4))
                .with_width(2.0),
        );
        
        frame.fill_text(canvas::Text {
            content: "NATS".to_string(),
            position: center,
            color: Color::WHITE,
            size: 14.0.into(),
            font: iced::Font::default(),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            vertical_alignment: iced::alignment::Vertical::Center,
            line_height: iced::widget::text::LineHeight::default(),
            shaping: iced::widget::text::Shaping::Basic,
        });
        
        // Draw subject nodes
        for (subject, node) in &self.subjects {
            // Connection line
            frame.stroke(
                &Path::line(center, node.position),
                Stroke::default()
                    .with_color(Color::from_rgba(0.3, 0.3, 0.3, 0.5))
                    .with_width(1.0),
            );
            
            // Node circle
            let activity_age = (Utc::now() - node.last_activity).num_seconds() as f32;
            let activity_alpha = (1.0 - activity_age / 10.0).max(0.3);
            
            frame.fill(
                &Path::circle(node.position, NODE_RADIUS),
                Color::from_rgba(0.2, 0.6, 1.0, activity_alpha),
            );
            
            frame.stroke(
                &Path::circle(node.position, NODE_RADIUS),
                Stroke::default()
                    .with_color(Color::from_rgb(0.4, 0.8, 1.0))
                    .with_width(2.0),
            );
            
            // Subject name
            let short_subject = subject.trim_end_matches('>');
            frame.fill_text(canvas::Text {
                content: short_subject.to_string(),
                position: node.position + Vector::new(0.0, -NODE_RADIUS - 10.0),
                color: Color::WHITE,
                size: 12.0.into(),
                font: iced::Font::default(),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                vertical_alignment: iced::alignment::Vertical::Bottom,
                line_height: iced::widget::text::LineHeight::default(),
                shaping: iced::widget::text::Shaping::Basic,
            });
            
            // Message count
            if node.message_count > 0 {
                frame.fill_text(canvas::Text {
                    content: node.message_count.to_string(),
                    position: node.position,
                    color: Color::WHITE,
                    size: 16.0.into(),
                    font: iced::Font::default(),
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    vertical_alignment: iced::alignment::Vertical::Center,
                    line_height: iced::widget::text::LineHeight::default(),
                    shaping: iced::widget::text::Shaping::Basic,
                });
            }
        }
        
        // Draw message animations
        for anim in &self.animations {
            let current_pos = Point::new(
                anim.from_pos.x + (anim.to_pos.x - anim.from_pos.x) * anim.progress,
                anim.from_pos.y + (anim.to_pos.y - anim.from_pos.y) * anim.progress,
            );
            
            // Message trail
            let trail_length = 30.0;
            let trail_dir = Vector::new(
                anim.from_pos.x - anim.to_pos.x,
                anim.from_pos.y - anim.to_pos.y,
            );
            let trail_norm = (trail_dir.x.powi(2) + trail_dir.y.powi(2)).sqrt();
            if trail_norm > 0.0 {
                let trail_unit = Vector::new(
                    trail_dir.x / trail_norm,
                    trail_dir.y / trail_norm,
                );
                
                let mut path = canvas::path::Builder::new();
                path.move_to(current_pos);
                path.line_to(current_pos + trail_unit * trail_length);
                
                frame.stroke(
                    &path.build(),
                    Stroke::default()
                        .with_color(Color::from_rgba(0.2, 1.0, 0.8, 0.5))
                        .with_width(3.0),
                );
            }
            
            // Message dot
            frame.fill(
                &Path::circle(current_pos, 5.0),
                Color::from_rgb(0.2, 1.0, 0.8),
            );
        }
        
        vec![frame.into_geometry()]
    }
}

pub async fn run_nats_flow_visualizer() -> Result<()> {
    println!("Starting NATS Flow Visualizer...");
    
    iced::application(
        NatsFlowVisualizer::title,
        NatsFlowVisualizer::update,
        NatsFlowVisualizer::view
    )
    .subscription(NatsFlowVisualizer::subscription)
    .window(window::Settings {
        size: iced::Size::new(1200.0, 800.0),
        position: window::Position::Centered,
        resizable: true,
        ..Default::default()
    })
    .theme(|_| Theme::Dark)
    .run_with(NatsFlowVisualizer::new)
    .map_err(|e| anyhow::anyhow!("Iced error: {}", e))
}