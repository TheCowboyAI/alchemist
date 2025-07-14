//! Simple launcher that works with current iced version
//! 
//! A minimal launcher that provides access to the working UI components.

use anyhow::Result;
use iced::{Task, Element, Theme, window, Length, Alignment};
use iced::widget::{column, container, text, button, Space};

#[derive(Debug, Clone)]
pub enum Message {
    LaunchDashboard,
    LaunchDialogWindow,
    Exit,
}

pub struct SimpleLauncher {
    nats_connected: bool,
}

impl SimpleLauncher {
    pub fn new() -> (Self, Task<Message>) {
        // Check NATS connection
        let nats_url = std::env::var("NATS_URL")
            .unwrap_or_else(|_| "nats://localhost:4222".to_string());
        
        let nats_connected = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match async_nats::connect(&nats_url).await {
                    Ok(_) => true,
                    Err(e) => {
                        eprintln!("NATS not available: {}", e);
                        false
                    }
                }
            })
        });
        
        (
            SimpleLauncher {
                nats_connected,
            },
            Task::none()
        )
    }
    
    pub fn title(&self) -> String {
        "Alchemist Launcher".to_string()
    }
    
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LaunchDashboard => {
                println!("Launching dashboard...");
                tokio::spawn(async {
                    match crate::dashboard_minimal::run_minimal_dashboard().await {
                        Ok(_) => println!("Dashboard closed"),
                        Err(e) => eprintln!("Dashboard error: {}", e),
                    }
                });
                Task::none()
            }
            
            Message::LaunchDialogWindow => {
                println!("Launching dialog window...");
                tokio::spawn(async {
                    match crate::dialog_window_minimal::run_dialog_window(
                        "AI Assistant".to_string()
                    ).await {
                        Ok(_) => println!("Dialog window closed"),
                        Err(e) => eprintln!("Dialog window error: {}", e),
                    }
                });
                Task::none()
            }
            
            Message::Exit => {
                std::process::exit(0);
            }
        }
    }
    
    pub fn view(&self) -> Element<Message> {
        let header = container(
            column![
                text("🧪 Alchemist Control System").size(32),
                Space::with_height(Length::Fixed(10.0)),
                text("Simple launcher for core UI components").size(16),
                Space::with_height(Length::Fixed(20.0)),
                text(if self.nats_connected {
                    "✅ NATS Connected"
                } else {
                    "⚠️  NATS Not Available (Demo Mode)"
                })
                .size(14)
                .color(if self.nats_connected {
                    iced::Color::from_rgb(0.0, 0.8, 0.0)
                } else {
                    iced::Color::from_rgb(0.8, 0.8, 0.0)
                }),
            ]
            .align_x(Alignment::Center)
        )
        .padding(20)
        .width(Length::Fill);
        
        let buttons = container(
            column![
                button(
                    text("📊 Launch Dashboard").size(18)
                )
                .on_press(Message::LaunchDashboard)
                .padding(15)
                .width(Length::Fixed(300.0)),
                
                Space::with_height(Length::Fixed(10.0)),
                
                button(
                    text("💬 Launch Dialog Window").size(18)
                )
                .on_press(Message::LaunchDialogWindow)
                .padding(15)
                .width(Length::Fixed(300.0)),
                
                Space::with_height(Length::Fixed(30.0)),
                
                button(
                    text("Exit").size(16)
                )
                .on_press(Message::Exit)
                .padding(10)
                .width(Length::Fixed(100.0)),
            ]
            .align_x(Alignment::Center)
        )
        .width(Length::Fill)
        .center_x(Length::Fill);
        
        container(
            column![
                header,
                buttons,
            ]
        )
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}

pub async fn run_simple_launcher() -> Result<()> {
    println!("Starting Alchemist Simple Launcher...");
    
    iced::application(
        SimpleLauncher::title,
        SimpleLauncher::update,
        SimpleLauncher::view
    )
    .window(window::Settings {
        size: iced::Size::new(600.0, 500.0),
        position: window::Position::Centered,
        resizable: false,
        ..Default::default()
    })
    .theme(|_| Theme::Dark)
    .run_with(SimpleLauncher::new)
    .map_err(|e| anyhow::anyhow!("Iced error: {}", e))
}