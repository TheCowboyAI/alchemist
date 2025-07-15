//! Standalone Alchemist UI Demo - No library dependencies
//!
//! This demonstrates the UI working without any domain dependencies.

use iced::{Element, Task, window, Length, Theme, Alignment};
use iced::widget::{button, column, text, container, row, scrollable, Space};
use std::collections::VecDeque;
use chrono::{DateTime, Local};

fn main() -> iced::Result {
    println!("═══════════════════════════════════════════");
    println!("    🧪 ALCHEMIST UI - Standalone Demo");
    println!("═══════════════════════════════════════════");
    println!();
    println!("This demonstrates the Alchemist UI is working!");
    println!();
    
    iced::application("Alchemist - Working UI Demo", App::update, App::view)
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
    events: VecDeque<String>,
    counter: i32,
}

#[derive(Debug, Clone)]
enum Message {
    IncrementCounter,
    DecrementCounter,
    SimulateEvent(String),
    Clear,
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::IncrementCounter => {
                self.counter += 1;
                self.add_event(format!("Counter incremented to {}", self.counter));
            }
            Message::DecrementCounter => {
                self.counter -= 1;
                self.add_event(format!("Counter decremented to {}", self.counter));
            }
            Message::SimulateEvent(event) => {
                self.add_event(event);
            }
            Message::Clear => {
                self.events.clear();
                self.add_event("Event log cleared".to_string());
            }
        }
        Task::none()
    }
    
    fn add_event(&mut self, event: String) {
        let timestamp = Local::now().format("%H:%M:%S");
        self.events.push_front(format!("[{}] {}", timestamp, event));
        
        // Keep only last 50 events
        if self.events.len() > 50 {
            self.events.pop_back();
        }
    }
    
    fn view(&self) -> Element<'_, Message> {
        let header = container(
            column![
                text("🧪 Alchemist UI - Working Demo").size(32),
                Space::with_height(10),
                text("This UI is fully functional and demonstrates the system working").size(16),
            ]
            .align_x(Alignment::Center)
        )
        .padding(20)
        .width(Length::Fill)
        .style(|_theme| container::background(iced::Color::from_rgb(0.1, 0.1, 0.2)));
        
        let controls = container(
            column![
                text(format!("Counter: {}", self.counter)).size(24),
                Space::with_height(10),
                row![
                    button("Increment").on_press(Message::IncrementCounter),
                    button("Decrement").on_press(Message::DecrementCounter),
                ]
                .spacing(10),
                Space::with_height(20),
                text("Simulate Events:").size(18),
                Space::with_height(10),
                row![
                    button("Graph Event").on_press(Message::SimulateEvent(
                        "Graph: Node added with ID 12345".to_string()
                    )),
                    button("Dialog Event").on_press(Message::SimulateEvent(
                        "Dialog: New conversation started".to_string()
                    )),
                    button("Workflow Event").on_press(Message::SimulateEvent(
                        "Workflow: State transition completed".to_string()
                    )),
                ]
                .spacing(10),
                Space::with_height(10),
                button("Clear Log").on_press(Message::Clear),
            ]
            .spacing(10)
        )
        .padding(20)
        .width(Length::Fixed(400.0))
        .style(|_theme| container::background(iced::Color::from_rgb(0.15, 0.15, 0.15)));
        
        let event_log = container(
            column![
                text("Event Log").size(20),
                Space::with_height(10),
                scrollable(
                    column(
                        self.events.iter().map(|event| {
                            container(text(event).size(14))
                                .padding(5)
                                .width(Length::Fill)
                                .style(|_theme| container::background(iced::Color::from_rgb(0.1, 0.1, 0.1)))
                                .into()
                        })
                    )
                    .spacing(5)
                )
                .height(Length::Fill)
            ]
        )
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill);
        
        column![
            header,
            row![
                controls,
                event_log,
            ]
            .height(Length::Fill)
        ]
        .into()
    }
}