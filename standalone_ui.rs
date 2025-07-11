//! Completely standalone UI test - no library dependencies

use iced::{Task, Element, Theme, window, Length, Alignment};
use iced::widget::{column, container, row, text, button, Space};

fn main() -> iced::Result {
    println!("Starting Alchemist UI...");
    iced::application("Alchemist", App::update, App::view)
        .window(window::Settings {
            size: iced::Size::new(1000.0, 700.0),
            position: window::Position::Centered,
            resizable: true,
            ..Default::default()
        })
        .theme(|_| Theme::Dark)
        .run()
}

#[derive(Default)]
struct App {
    event_count: u32,
    nats_connected: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Refresh,
    ToggleNats,
    Exit,
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Refresh => {
                self.event_count += 1;
                Task::none()
            }
            Message::ToggleNats => {
                self.nats_connected = !self.nats_connected;
                Task::none()
            }
            Message::Exit => {
                std::process::exit(0);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let header = row![
            text("🧪 Alchemist Dashboard").size(32),
            Space::with_width(Length::Fill),
            button("Refresh").on_press(Message::Refresh),
            button("Exit").on_press(Message::Exit),
        ]
        .spacing(20)
        .padding(20)
        .align_y(Alignment::Center);

        let status = container(
            column![
                text("System Status").size(24),
                Space::with_height(Length::Fixed(10.0)),
                row![
                    text("NATS: "),
                    button(if self.nats_connected { "Connected ✓" } else { "Disconnected ✗" })
                        .on_press(Message::ToggleNats)
                        .style(if self.nats_connected { button::success } else { button::danger }),
                ].spacing(10),
                text(format!("Events: {}", self.event_count)),
                text("Memory: 45.2 MB"),
                text("Uptime: 3d 14h"),
            ]
            .spacing(10)
            .padding(20)
        )
        .style(container::rounded_box);

        let domains = container(
            column![
                text("Active Domains").size(24),
                Space::with_height(Length::Fixed(10.0)),
                domain_row("workflow", "Business process execution", 1234, true),
                domain_row("agent", "AI provider integration", 567, true),
                domain_row("document", "Document lifecycle", 890, true),
                domain_row("policy", "Business rules", 234, false),
                domain_row("graph", "Core graph operations", 3456, true),
            ]
            .spacing(5)
            .padding(20)
        )
        .style(container::rounded_box);

        let events = container(
            column![
                text("Recent Events").size(24),
                Space::with_height(Length::Fixed(10.0)),
                event_row("10:45:23", "workflow", "WorkflowCompleted"),
                event_row("10:44:15", "agent", "QueryExecuted"),
                event_row("10:43:02", "document", "DocumentCreated"),
                event_row("10:42:11", "policy", "RuleEvaluated"),
                event_row("10:41:33", "graph", "NodeAdded"),
            ]
            .spacing(5)
            .padding(20)
        )
        .style(container::rounded_box);

        let content = column![
            header,
            row![
                column![status].width(Length::FillPortion(1)),
                column![domains].width(Length::FillPortion(2)),
            ]
            .spacing(20)
            .height(Length::FillPortion(1)),
            events.height(Length::FillPortion(1)),
        ]
        .spacing(20);

        container(content)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn domain_row(name: &str, desc: &str, events: u32, healthy: bool) -> Element<'static, Message> {
    row![
        text(name).width(Length::Fixed(100.0)),
        text(desc).width(Length::Fill),
        text(format!("{} events", events)).width(Length::Fixed(100.0)),
        text(if healthy { "✓" } else { "✗" })
            .size(20)
            .color(if healthy { 
                iced::Color::from_rgb(0.0, 1.0, 0.0) 
            } else { 
                iced::Color::from_rgb(1.0, 0.0, 0.0) 
            }),
    ]
    .spacing(10)
    .padding(5)
    .into()
}

fn event_row(time: &str, domain: &str, event: &str) -> Element<'static, Message> {
    row![
        text(time).width(Length::Fixed(80.0)),
        text(domain).width(Length::Fixed(100.0)),
        text(event).width(Length::Fill),
    ]
    .spacing(10)
    .padding(2)
    .into()
}