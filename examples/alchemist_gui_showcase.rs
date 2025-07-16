//! Alchemist GUI Showcase - Demonstrates both Iced and Bevy visual interfaces

use iced::{
    application, executor, Application, Command, Element, Settings, Theme,
    widget::{button, column, container, row, text, Column},
    window, Length, Subscription,
};
use std::process::Command as ProcessCommand;

#[derive(Debug, Clone)]
enum Message {
    LaunchBevyDemo(BevyDemo),
    LaunchIcedDemo(IcedDemo),
    OpenDashboard,
    OpenWorkflowEditor,
    OpenEventVisualizer,
    OpenDialogWindow,
    OpenSettings,
    ShowAbout,
}

#[derive(Debug, Clone)]
enum BevyDemo {
    GraphVisualization,
    WorkflowAnimation,
    NatsEventStream,
    DeploymentGraph,
    InteractiveAI,
}

#[derive(Debug, Clone)]
enum IcedDemo {
    Dashboard,
    WorkflowEditor,
    EventVisualizer,
    DialogWindow,
    PerformanceMonitor,
}

struct AlchemistShowcase {
    status_message: String,
}

impl Application for AlchemistShowcase {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            AlchemistShowcase {
                status_message: "Welcome to Alchemist - The Composable Information Machine".to_string(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Alchemist GUI Showcase")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LaunchBevyDemo(demo) => {
                self.status_message = format!("Launching Bevy 3D Demo: {:?}", demo);
                // Launch appropriate Bevy demo based on selection
                match demo {
                    BevyDemo::GraphVisualization => {
                        let _ = ProcessCommand::new("cargo")
                            .args(&["run", "--example", "graph_visualization_demo"])
                            .spawn();
                    }
                    BevyDemo::WorkflowAnimation => {
                        let _ = ProcessCommand::new("cargo")
                            .args(&["run", "--example", "workflow_demo_visual"])
                            .spawn();
                    }
                    BevyDemo::NatsEventStream => {
                        let _ = ProcessCommand::new("cargo")
                            .args(&["run", "--example", "nats_event_visualization_demo"])
                            .spawn();
                    }
                    BevyDemo::DeploymentGraph => {
                        let _ = ProcessCommand::new("cargo")
                            .args(&["run", "--example", "deployment_graph_visual"])
                            .spawn();
                    }
                    BevyDemo::InteractiveAI => {
                        let _ = ProcessCommand::new("cargo")
                            .args(&["run", "--example", "interactive_ai_graph_demo"])
                            .spawn();
                    }
                }
            }
            Message::LaunchIcedDemo(demo) => {
                self.status_message = format!("Launching Iced 2D Demo: {:?}", demo);
                // Launch appropriate Iced demo based on selection
                match demo {
                    IcedDemo::Dashboard => {
                        let _ = ProcessCommand::new("cargo")
                            .args(&["run", "--example", "dashboard_demo"])
                            .spawn();
                    }
                    IcedDemo::WorkflowEditor => {
                        let _ = ProcessCommand::new("cargo")
                            .args(&["run", "--example", "workflow_editor_demo"])
                            .spawn();
                    }
                    IcedDemo::EventVisualizer => {
                        let _ = ProcessCommand::new("cargo")
                            .args(&["run", "--example", "event_visualizer_demo"])
                            .spawn();
                    }
                    IcedDemo::DialogWindow => {
                        let _ = ProcessCommand::new("cargo")
                            .args(&["run", "--example", "dialog_window_demo"])
                            .spawn();
                    }
                    IcedDemo::PerformanceMonitor => {
                        let _ = ProcessCommand::new("cargo")
                            .args(&["run", "--example", "performance_monitor_demo"])
                            .spawn();
                    }
                }
            }
            Message::OpenDashboard => {
                self.status_message = "Opening main dashboard...".to_string();
                let _ = ProcessCommand::new("cargo")
                    .args(&["run", "--bin", "alchemist", "--", "--dashboard"])
                    .spawn();
            }
            Message::OpenWorkflowEditor => {
                self.status_message = "Opening workflow editor...".to_string();
                let _ = ProcessCommand::new("cargo")
                    .args(&["run", "--bin", "alchemist", "--", "--workflow-editor"])
                    .spawn();
            }
            Message::OpenEventVisualizer => {
                self.status_message = "Opening event visualizer...".to_string();
                let _ = ProcessCommand::new("cargo")
                    .args(&["run", "--bin", "alchemist", "--", "--event-visualizer"])
                    .spawn();
            }
            Message::OpenDialogWindow => {
                self.status_message = "Opening AI dialog interface...".to_string();
                let _ = ProcessCommand::new("cargo")
                    .args(&["run", "--bin", "alchemist", "--", "--dialog"])
                    .spawn();
            }
            Message::OpenSettings => {
                self.status_message = "Opening settings...".to_string();
                let _ = ProcessCommand::new("cargo")
                    .args(&["run", "--bin", "alchemist", "--", "--settings"])
                    .spawn();
            }
            Message::ShowAbout => {
                self.status_message = "Alchemist - A revolutionary 3D graph visualization and editing system for the Composable Information Machine (CIM)".to_string();
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let title = text("Alchemist GUI Showcase")
            .size(32)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.6, 0.8)));

        let subtitle = text("Experience the power of visual computing with Iced 2D and Bevy 3D")
            .size(16)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6)));

        // Main application launchers
        let main_apps = column![
            text("Main Applications").size(20),
            row![
                button("Dashboard").on_press(Message::OpenDashboard).padding(10),
                button("Workflow Editor").on_press(Message::OpenWorkflowEditor).padding(10),
                button("Event Visualizer").on_press(Message::OpenEventVisualizer).padding(10),
                button("AI Dialog").on_press(Message::OpenDialogWindow).padding(10),
                button("Settings").on_press(Message::OpenSettings).padding(10),
            ].spacing(10),
        ].spacing(10);

        // Bevy 3D demos
        let bevy_demos = column![
            text("Bevy 3D Visualization Demos").size(20),
            row![
                button("Graph Visualization")
                    .on_press(Message::LaunchBevyDemo(BevyDemo::GraphVisualization))
                    .padding(10),
                button("Workflow Animation")
                    .on_press(Message::LaunchBevyDemo(BevyDemo::WorkflowAnimation))
                    .padding(10),
                button("NATS Event Stream")
                    .on_press(Message::LaunchBevyDemo(BevyDemo::NatsEventStream))
                    .padding(10),
            ].spacing(10),
            row![
                button("Deployment Graph")
                    .on_press(Message::LaunchBevyDemo(BevyDemo::DeploymentGraph))
                    .padding(10),
                button("Interactive AI Graph")
                    .on_press(Message::LaunchBevyDemo(BevyDemo::InteractiveAI))
                    .padding(10),
            ].spacing(10),
        ].spacing(10);

        // Iced 2D demos
        let iced_demos = column![
            text("Iced 2D Interface Demos").size(20),
            row![
                button("Dashboard Demo")
                    .on_press(Message::LaunchIcedDemo(IcedDemo::Dashboard))
                    .padding(10),
                button("Workflow Editor Demo")
                    .on_press(Message::LaunchIcedDemo(IcedDemo::WorkflowEditor))
                    .padding(10),
                button("Event Visualizer Demo")
                    .on_press(Message::LaunchIcedDemo(IcedDemo::EventVisualizer))
                    .padding(10),
            ].spacing(10),
            row![
                button("Dialog Window Demo")
                    .on_press(Message::LaunchIcedDemo(IcedDemo::DialogWindow))
                    .padding(10),
                button("Performance Monitor Demo")
                    .on_press(Message::LaunchIcedDemo(IcedDemo::PerformanceMonitor))
                    .padding(10),
            ].spacing(10),
        ].spacing(10);

        // Status bar
        let status = container(text(&self.status_message).size(14))
            .padding(10)
            .style(|theme: &Theme| {
                container::Appearance {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(0.1, 0.1, 0.1))),
                    text_color: Some(iced::Color::from_rgb(0.8, 0.8, 0.8)),
                    ..container::Appearance::default()
                }
            });

        // About button
        let about_button = button("About Alchemist")
            .on_press(Message::ShowAbout)
            .padding(10);

        // Main layout
        let content = column![
            title,
            subtitle,
            container(main_apps).padding(20),
            container(bevy_demos).padding(20),
            container(iced_demos).padding(20),
            about_button,
        ]
        .spacing(20)
        .align_items(iced::Alignment::Center);

        container(
            column![
                container(content).width(Length::Fill).height(Length::FillPortion(9)),
                status.width(Length::Fill).height(Length::FillPortion(1)),
            ]
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn main() -> iced::Result {
    AlchemistShowcase::run(Settings {
        window: window::Settings {
            size: iced::Size::new(900.0, 700.0),
            position: window::Position::Centered,
            min_size: Some(iced::Size::new(800.0, 600.0)),
            ..Default::default()
        },
        ..Default::default()
    })
}