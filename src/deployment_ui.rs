//! Deployment management UI
//!
//! Visual interface for managing Nix deployments, flake configurations,
//! and deployment status across different environments.

use anyhow::Result;
use iced::{Task, Element, Theme, window, Length, Alignment, Color};
use iced::widget::{column, container, row, text, button, text_input, scrollable, Space, pick_list};
use tokio::sync::mpsc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Message {
    // Deployment actions
    DeploymentSelected(String),
    StartDeployment,
    StopDeployment,
    RollbackDeployment,
    RefreshStatus,
    
    // Environment actions
    EnvironmentSelected(String),
    AddEnvironment,
    RemoveEnvironment,
    
    // Flake actions
    FlakeInputChanged(String),
    UpdateFlake,
    LockFlake,
    
    // Config actions
    ConfigChanged(String, String),
    SaveConfig,
    
    // UI actions
    TabSelected(Tab),
    ShowLogs(String),
    ClearLogs,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Deployments,
    Environments,
    Configuration,
    Logs,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DeploymentStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub id: String,
    pub name: String,
    pub environment: String,
    pub flake_url: String,
    pub status: DeploymentStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub packages: Vec<String>,
    pub services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub name: String,
    pub description: String,
    pub variables: HashMap<String, String>,
    pub active_deployments: usize,
}

#[derive(Debug, Clone)]
pub struct DeploymentLog {
    pub deployment_id: String,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Success,
}

pub struct DeploymentManager {
    // State
    deployments: Vec<Deployment>,
    environments: Vec<Environment>,
    logs: Vec<DeploymentLog>,
    
    // UI state
    selected_deployment: Option<String>,
    selected_environment: Option<String>,
    current_tab: Tab,
    flake_input: String,
    config_values: HashMap<String, String>,
    
    // Communication
    command_sender: Option<mpsc::Sender<DeploymentCommand>>,
    event_receiver: Option<mpsc::Receiver<DeploymentEvent>>,
}

#[derive(Debug, Clone)]
pub enum DeploymentCommand {
    Deploy { deployment_id: String },
    Stop { deployment_id: String },
    Rollback { deployment_id: String },
    UpdateFlake { url: String },
}

#[derive(Debug, Clone)]
pub enum DeploymentEvent {
    StatusChanged { deployment_id: String, status: DeploymentStatus },
    LogMessage { log: DeploymentLog },
    DeploymentCompleted { deployment_id: String, success: bool },
}

impl DeploymentManager {
    pub fn new() -> (Self, Task<Message>) {
        // Create communication channels
        let (cmd_tx, cmd_rx) = mpsc::channel(10);
        let (event_tx, event_rx) = mpsc::channel(100);
        
        // Spawn deployment handler
        tokio::spawn(deployment_handler(cmd_rx, event_tx));
        
        // Create demo data
        let environments = vec![
            Environment {
                name: "development".to_string(),
                description: "Development environment".to_string(),
                variables: HashMap::from([
                    ("DEBUG".to_string(), "true".to_string()),
                    ("LOG_LEVEL".to_string(), "debug".to_string()),
                ]),
                active_deployments: 2,
            },
            Environment {
                name: "staging".to_string(),
                description: "Staging environment".to_string(),
                variables: HashMap::from([
                    ("DEBUG".to_string(), "false".to_string()),
                    ("LOG_LEVEL".to_string(), "info".to_string()),
                ]),
                active_deployments: 1,
            },
            Environment {
                name: "production".to_string(),
                description: "Production environment".to_string(),
                variables: HashMap::from([
                    ("DEBUG".to_string(), "false".to_string()),
                    ("LOG_LEVEL".to_string(), "warning".to_string()),
                ]),
                active_deployments: 3,
            },
        ];
        
        let deployments = vec![
            Deployment {
                id: Uuid::new_v4().to_string(),
                name: "web-app".to_string(),
                environment: "production".to_string(),
                flake_url: "github:myorg/web-app#nixosConfigurations.prod".to_string(),
                status: DeploymentStatus::Completed,
                started_at: Some(Utc::now() - chrono::Duration::hours(2)),
                completed_at: Some(Utc::now() - chrono::Duration::hours(1)),
                error: None,
                packages: vec!["nginx".to_string(), "nodejs".to_string()],
                services: vec!["web-app.service".to_string()],
            },
            Deployment {
                id: Uuid::new_v4().to_string(),
                name: "api-service".to_string(),
                environment: "staging".to_string(),
                flake_url: "github:myorg/api#nixosConfigurations.staging".to_string(),
                status: DeploymentStatus::InProgress,
                started_at: Some(Utc::now() - chrono::Duration::minutes(5)),
                completed_at: None,
                error: None,
                packages: vec!["postgresql".to_string(), "redis".to_string()],
                services: vec!["api.service".to_string(), "worker.service".to_string()],
            },
        ];
        
        (
            DeploymentManager {
                deployments,
                environments,
                logs: Vec::new(),
                selected_deployment: None,
                selected_environment: Some("production".to_string()),
                current_tab: Tab::Deployments,
                flake_input: String::new(),
                config_values: HashMap::new(),
                command_sender: Some(cmd_tx),
                event_receiver: Some(event_rx),
            },
            Task::none()
        )
    }
    
    pub fn title(&self) -> String {
        "Deployment Manager".to_string()
    }
    
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DeploymentSelected(id) => {
                self.selected_deployment = Some(id);
                Task::none()
            }
            
            Message::StartDeployment => {
                if let Some(env) = &self.selected_environment {
                    let deployment = Deployment {
                        id: Uuid::new_v4().to_string(),
                        name: format!("deployment-{}", Utc::now().format("%Y%m%d-%H%M%S")),
                        environment: env.clone(),
                        flake_url: self.flake_input.clone(),
                        status: DeploymentStatus::InProgress,
                        started_at: Some(Utc::now()),
                        completed_at: None,
                        error: None,
                        packages: vec!["package1".to_string(), "package2".to_string()],
                        services: vec!["service1".to_string()],
                    };
                    
                    if let Some(sender) = &self.command_sender {
                        let cmd = DeploymentCommand::Deploy {
                            deployment_id: deployment.id.clone(),
                        };
                        let sender = sender.clone();
                        tokio::spawn(async move {
                            let _ = sender.send(cmd).await;
                        });
                    }
                    
                    self.deployments.push(deployment);
                }
                Task::none()
            }
            
            Message::StopDeployment => {
                if let Some(id) = &self.selected_deployment {
                    if let Some(deployment) = self.deployments.iter_mut().find(|d| &d.id == id) {
                        deployment.status = DeploymentStatus::Failed;
                        deployment.completed_at = Some(Utc::now());
                        deployment.error = Some("Stopped by user".to_string());
                    }
                }
                Task::none()
            }
            
            Message::RollbackDeployment => {
                if let Some(id) = &self.selected_deployment {
                    if let Some(deployment) = self.deployments.iter_mut().find(|d| &d.id == id) {
                        deployment.status = DeploymentStatus::RolledBack;
                        
                        self.logs.push(DeploymentLog {
                            deployment_id: id.clone(),
                            timestamp: Utc::now(),
                            level: LogLevel::Warning,
                            message: "Deployment rolled back".to_string(),
                        });
                    }
                }
                Task::none()
            }
            
            Message::RefreshStatus => {
                // Check for events
                let mut events = Vec::new();
                if let Some(receiver) = &mut self.event_receiver {
                    while let Ok(event) = receiver.try_recv() {
                        events.push(event);
                    }
                }
                for event in events {
                    self.handle_deployment_event(event);
                }
                Task::none()
            }
            
            Message::EnvironmentSelected(env) => {
                self.selected_environment = Some(env);
                Task::none()
            }
            
            Message::FlakeInputChanged(value) => {
                self.flake_input = value;
                Task::none()
            }
            
            Message::TabSelected(tab) => {
                self.current_tab = tab;
                Task::none()
            }
            
            Message::ClearLogs => {
                self.logs.clear();
                Task::none()
            }
            
            Message::Close => {
                std::process::exit(0);
            }
            
            _ => Task::none(),
        }
    }
    
    fn handle_deployment_event(&mut self, event: DeploymentEvent) {
        match event {
            DeploymentEvent::StatusChanged { deployment_id, status } => {
                if let Some(deployment) = self.deployments.iter_mut()
                    .find(|d| d.id == deployment_id) {
                    deployment.status = status;
                    if matches!(status, DeploymentStatus::Completed | DeploymentStatus::Failed) {
                        deployment.completed_at = Some(Utc::now());
                    }
                }
            }
            
            DeploymentEvent::LogMessage { log } => {
                self.logs.push(log);
            }
            
            DeploymentEvent::DeploymentCompleted { deployment_id, success } => {
                if let Some(deployment) = self.deployments.iter_mut()
                    .find(|d| d.id == deployment_id) {
                    deployment.status = if success {
                        DeploymentStatus::Completed
                    } else {
                        DeploymentStatus::Failed
                    };
                    deployment.completed_at = Some(Utc::now());
                }
            }
        }
    }
    
    pub fn view(&self) -> Element<Message> {
        let header = self.view_header();
        let tabs = self.view_tabs();
        
        let content = match self.current_tab {
            Tab::Deployments => self.view_deployments(),
            Tab::Environments => self.view_environments(),
            Tab::Configuration => self.view_configuration(),
            Tab::Logs => self.view_logs(),
        };
        
        container(
            column![
                header,
                tabs,
                content,
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
                text("🚀 Deployment Manager").size(24),
                Space::with_width(Length::Fill),
                text(format!("{} deployments", self.deployments.len())),
                button("Refresh").on_press(Message::RefreshStatus),
                button("Close").on_press(Message::Close),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        )
        .padding(10)
        .style(container::rounded_box)
        .into()
    }
    
    fn view_tabs(&self) -> Element<Message> {
        container(
            row![
                button("Deployments")
                    .on_press(Message::TabSelected(Tab::Deployments))
                    .style(if self.current_tab == Tab::Deployments {
                        button::primary
                    } else {
                        button::secondary
                    }),
                button("Environments")
                    .on_press(Message::TabSelected(Tab::Environments))
                    .style(if self.current_tab == Tab::Environments {
                        button::primary
                    } else {
                        button::secondary
                    }),
                button("Configuration")
                    .on_press(Message::TabSelected(Tab::Configuration))
                    .style(if self.current_tab == Tab::Configuration {
                        button::primary
                    } else {
                        button::secondary
                    }),
                button("Logs")
                    .on_press(Message::TabSelected(Tab::Logs))
                    .style(if self.current_tab == Tab::Logs {
                        button::primary
                    } else {
                        button::secondary
                    }),
            ]
            .spacing(10)
        )
        .padding(10)
        .style(container::rounded_box)
        .into()
    }
    
    fn view_deployments(&self) -> Element<Message> {
        let mut deployments_list = column![].spacing(10);
        
        for deployment in &self.deployments {
            let status_color = match deployment.status {
                DeploymentStatus::Completed => Color::from_rgb(0.2, 0.8, 0.2),
                DeploymentStatus::InProgress => Color::from_rgb(1.0, 0.8, 0.0),
                DeploymentStatus::Failed => Color::from_rgb(1.0, 0.2, 0.2),
                DeploymentStatus::RolledBack => Color::from_rgb(0.6, 0.6, 0.6),
                DeploymentStatus::NotStarted => Color::from_rgb(0.4, 0.4, 0.4),
            };
            
            let is_selected = self.selected_deployment.as_ref() == Some(&deployment.id);
            
            let deployment_card = button(
                container(
                    column![
                        row![
                            text(&deployment.name).size(18),
                            Space::with_width(Length::Fill),
                            text(format!("{:?}", deployment.status))
                                .color(status_color),
                        ],
                        text(format!("Environment: {}", deployment.environment))
                            .size(14)
                            .color(Color::from_rgb(0.7, 0.7, 0.7)),
                        text(&deployment.flake_url)
                            .size(12)
                            .color(Color::from_rgb(0.6, 0.6, 0.6)),
                        if let Some(started) = deployment.started_at {
                            text(format!("Started: {}", started.format("%Y-%m-%d %H:%M:%S")))
                                .size(12)
                                .color(Color::from_rgb(0.5, 0.5, 0.5))
                        } else {
                            text("")
                        },
                    ]
                    .spacing(5)
                )
                .padding(10)
            )
            .on_press(Message::DeploymentSelected(deployment.id.clone()))
            .style(if is_selected { button::primary } else { button::secondary })
            .width(Length::Fill);
            
            deployments_list = deployments_list.push(deployment_card);
        }
        
        let actions = if self.selected_deployment.is_some() {
            row![
                button("Stop").on_press(Message::StopDeployment),
                button("Rollback").on_press(Message::RollbackDeployment),
            ]
            .spacing(10)
        } else {
            row![]
        };
        
        container(
            column![
                scrollable(deployments_list)
                    .height(Length::FillPortion(4)),
                container(actions)
                    .padding(10),
            ]
            .spacing(10)
        )
        .height(Length::Fill)
        .style(container::rounded_box)
        .into()
    }
    
    fn view_environments(&self) -> Element<Message> {
        let env_list = pick_list(
            self.environments.iter().map(|e| e.name.clone()).collect::<Vec<_>>(),
            self.selected_environment.clone(),
            Message::EnvironmentSelected,
        );
        
        let new_deployment = container(
            column![
                text("New Deployment").size(20),
                Space::with_height(Length::Fixed(10.0)),
                row![
                    text("Environment:"),
                    env_list,
                ]
                .spacing(10),
                Space::with_height(Length::Fixed(10.0)),
                text_input("Flake URL (e.g., github:owner/repo#config)", &self.flake_input)
                    .on_input(Message::FlakeInputChanged),
                Space::with_height(Length::Fixed(10.0)),
                button("Start Deployment")
                    .on_press(Message::StartDeployment)
                    .style(button::primary),
            ]
            .spacing(10)
            .padding(20)
        )
        .style(container::bordered_box);
        
        let mut env_details = column![].spacing(10);
        
        for env in &self.environments {
            let env_card = container(
                column![
                    text(&env.name).size(18),
                    text(&env.description)
                        .size(14)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text(format!("Active deployments: {}", env.active_deployments))
                        .size(12),
                    text("Variables:")
                        .size(12),
                    container(
                        column(
                            env.variables.iter().map(|(k, v)| {
                                text(format!("  {} = {}", k, v))
                                    .size(11)
                                    .color(Color::from_rgb(0.6, 0.6, 0.6))
                                    .into()
                            }).collect::<Vec<_>>()
                        )
                        .spacing(2)
                    )
                    .padding(iced::Padding { top: 0.0, right: 0.0, bottom: 0.0, left: 10.0 }),
                ]
                .spacing(5)
            )
            .padding(15)
            .style(container::bordered_box);
            
            env_details = env_details.push(env_card);
        }
        
        container(
            column![
                new_deployment,
                scrollable(env_details),
            ]
            .spacing(20)
        )
        .height(Length::Fill)
        .style(container::rounded_box)
        .into()
    }
    
    fn view_configuration(&self) -> Element<Message> {
        container(
            column![
                text("Deployment Configuration").size(20),
                Space::with_height(Length::Fixed(20.0)),
                text("Global Settings").size(16),
                container(
                    text("Configuration options will be displayed here")
                        .color(Color::from_rgb(0.6, 0.6, 0.6))
                )
                .padding(20)
                .style(container::bordered_box),
                Space::with_height(Length::Fixed(20.0)),
                text("Flake Lock").size(16),
                button("Update Flake Lock").on_press(Message::UpdateFlake),
            ]
            .spacing(10)
            .padding(20)
        )
        .height(Length::Fill)
        .style(container::rounded_box)
        .into()
    }
    
    fn view_logs(&self) -> Element<Message> {
        let mut logs_col = column![
            row![
                text("Deployment Logs").size(20),
                Space::with_width(Length::Fill),
                button("Clear").on_press(Message::ClearLogs),
            ]
            .align_y(Alignment::Center),
            Space::with_height(Length::Fixed(10.0)),
        ]
        .spacing(5);
        
        for log in self.logs.iter().rev().take(100) {
            let (icon, color) = match log.level {
                LogLevel::Info => ("ℹ️", Color::from_rgb(0.6, 0.6, 0.6)),
                LogLevel::Warning => ("⚠️", Color::from_rgb(1.0, 0.8, 0.0)),
                LogLevel::Error => ("❌", Color::from_rgb(1.0, 0.2, 0.2)),
                LogLevel::Success => ("✅", Color::from_rgb(0.2, 0.8, 0.2)),
            };
            
            logs_col = logs_col.push(
                row![
                    text(log.timestamp.format("%H:%M:%S").to_string())
                        .size(12)
                        .color(Color::from_rgb(0.5, 0.5, 0.5)),
                    text(icon),
                    text(&log.message)
                        .size(14)
                        .color(color),
                ]
                .spacing(10)
            );
        }
        
        container(
            scrollable(logs_col.padding(10))
                .height(Length::Fill)
        )
        .height(Length::Fill)
        .style(container::rounded_box)
        .into()
    }
    
    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(500))
            .map(|_| Message::RefreshStatus)
    }
}

async fn deployment_handler(
    mut cmd_rx: mpsc::Receiver<DeploymentCommand>,
    event_tx: mpsc::Sender<DeploymentEvent>,
) {
    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            DeploymentCommand::Deploy { deployment_id } => {
                // Simulate deployment process
                let tx = event_tx.clone();
                let id = deployment_id.clone();
                
                tokio::spawn(async move {
                    // Starting
                    let _ = tx.send(DeploymentEvent::LogMessage {
                        log: DeploymentLog {
                            deployment_id: id.clone(),
                            timestamp: Utc::now(),
                            level: LogLevel::Info,
                            message: "Starting deployment...".to_string(),
                        }
                    }).await;
                    
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    
                    // Building
                    let _ = tx.send(DeploymentEvent::LogMessage {
                        log: DeploymentLog {
                            deployment_id: id.clone(),
                            timestamp: Utc::now(),
                            level: LogLevel::Info,
                            message: "Building Nix packages...".to_string(),
                        }
                    }).await;
                    
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    
                    // Deploying
                    let _ = tx.send(DeploymentEvent::LogMessage {
                        log: DeploymentLog {
                            deployment_id: id.clone(),
                            timestamp: Utc::now(),
                            level: LogLevel::Info,
                            message: "Deploying to target...".to_string(),
                        }
                    }).await;
                    
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    
                    // Complete
                    let _ = tx.send(DeploymentEvent::LogMessage {
                        log: DeploymentLog {
                            deployment_id: id.clone(),
                            timestamp: Utc::now(),
                            level: LogLevel::Success,
                            message: "Deployment completed successfully!".to_string(),
                        }
                    }).await;
                    
                    let _ = tx.send(DeploymentEvent::DeploymentCompleted {
                        deployment_id: id,
                        success: true,
                    }).await;
                });
            }
            _ => {}
        }
    }
}

pub async fn run_deployment_manager() -> Result<()> {
    println!("Starting Deployment Manager...");
    
    iced::application(
        DeploymentManager::title,
        DeploymentManager::update,
        DeploymentManager::view
    )
    .subscription(DeploymentManager::subscription)
    .window(window::Settings {
        size: iced::Size::new(1200.0, 800.0),
        position: window::Position::Centered,
        resizable: true,
        ..Default::default()
    })
    .theme(|_| Theme::Dark)
    .run_with(DeploymentManager::new)
    .map_err(|e| anyhow::anyhow!("Iced error: {}", e))
}