//! Simplified Iced renderer for basic UI windows

use anyhow::Result;
use iced::{Task, Element, Theme, window, Length};
use iced::widget::{column, container, text, scrollable, button, row, Space};
use alchemist::renderer::{RenderRequest, RenderData};

#[derive(Debug, Clone)]
enum Message {
    CloseWindow,
}

struct AlchemistWindow {
    title: String,
    data: RenderData,
}

impl AlchemistWindow {
    fn new(request: RenderRequest) -> (Self, Task<Message>) {
        (
            AlchemistWindow {
                title: request.title,
                data: request.data,
            },
            Task::none()
        )
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CloseWindow => {
                std::process::exit(0);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let header = row![
            text(&self.title).size(24),
            Space::with_width(Length::Fill),
            button("Close").on_press(Message::CloseWindow),
        ]
        .padding(10);

        let content = match &self.data {
            RenderData::Dashboard(dashboard) => {
                column![
                    text("System Status").size(20),
                    text(format!("NATS Connected: {}", if dashboard.system_status.nats_connected { "Yes" } else { "No" })),
                    text(format!("Total Events: {}", dashboard.system_status.total_events)),
                    text("").size(16),
                    text("Domains").size(20),
                    text(format!("{} domains registered", dashboard.domains.len())),
                    text("").size(16),
                    text("Recent Events").size(20),
                    text(format!("{} recent events", dashboard.recent_events.len())),
                ]
                .spacing(10)
            }
            RenderData::Dialog { dialog_id, ai_model, messages, .. } => {
                let mut msg_column = column![
                    text(format!("Dialog ID: {}", dialog_id)).size(16),
                    text(format!("AI Model: {}", ai_model)).size(16),
                    text("").size(10),
                    text("Messages:").size(18),
                ];

                for (_i, msg) in messages.iter().enumerate() {
                    msg_column = msg_column.push(
                        container(
                            text(format!("{}: {}", msg.role, msg.content))
                                .size(14)
                        )
                        .padding(5)
                        .style(container::rounded_box)
                    );
                }

                msg_column.spacing(5)
            }
            RenderData::Graph3D { nodes, edges } => {
                column![
                    text("3D Graph Visualization").size(20),
                    text(format!("Nodes: {}", nodes.len())),
                    text(format!("Edges: {}", edges.len())),
                    text("").size(10),
                    text("(3D rendering requires Bevy renderer)").size(14),
                ]
                .spacing(10)
            }
            RenderData::Markdown { .. } => {
                // Markdown is handled specially - this shouldn't be reached
                column![
                    text("Markdown Viewer").size(20),
                    text("This should be using the dedicated markdown viewer").size(14),
                ]
                .spacing(10)
            }
            _ => {
                // For now, show something for all data types
                column![
                    text("Alchemist Dashboard").size(24),
                    text("Renderer is working!").size(16),
                    text("").size(10),
                    text(format!("Data type: {:?}", std::mem::discriminant(&self.data))).size(14),
                ]
                .spacing(10)
            }
        };

        container(
            column![
                container(header)
                    .style(container::rounded_box)
                    .width(Length::Fill),
                scrollable(
                    container(content)
                        .padding(20)
                        .width(Length::Fill)
                )
                .height(Length::Fill),
            ]
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

pub fn run(request: RenderRequest) -> Result<()> {
    iced::application(
        AlchemistWindow::title,
        AlchemistWindow::update,
        AlchemistWindow::view
    )
    .window(window::Settings {
        size: iced::Size::new(
            request.config.width as f32,
            request.config.height as f32
        ),
        position: window::Position::Centered,
        resizable: request.config.resizable,
        ..Default::default()
    })
    .theme(|_| Theme::Dark)
    .run_with(|| AlchemistWindow::new(request))
    .map_err(|e| anyhow::anyhow!("Iced error: {}", e))
}