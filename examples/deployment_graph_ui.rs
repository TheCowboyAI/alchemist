//! Deployment Graph UI Example
//!
//! This example demonstrates the deployment graph visualization in a simple UI.
//! Run with: cargo run --example deployment_graph_ui

use alchemist::prelude::*;
use iced::{Element, Task, Theme};
use iced::widget::{button, column, container, row, text, Space};
use cim_domain_graph::{
    Graph, GraphId, NodeId, EdgeId,
    deployment::{
        DeploymentNodeType, DeploymentEdgeType, ResourceRequirements,
        DatabaseEngine, MessageBusType,
        graph_adapter::create_deployment_node_metadata,
    },
};
use std::collections::HashMap;

pub fn main() -> iced::Result {
    iced::application("Deployment Graph Visualization", DeploymentGraphApp::update, DeploymentGraphApp::view)
        .theme(DeploymentGraphApp::theme)
        .run_with(DeploymentGraphApp::new)
}

#[derive(Debug)]
struct DeploymentGraphApp {
    graph: Graph,
    selected_node: Option<NodeId>,
    show_details: bool,
}

#[derive(Debug, Clone)]
enum Message {
    AddService,
    AddDatabase,
    AddAgent,
    AddLoadBalancer,
    SelectNode(NodeId),
    ToggleDetails,
    GenerateNix,
}

impl DeploymentGraphApp {
    fn new() -> (Self, Task<Message>) {
        let mut graph = Graph::new(
            GraphId::new(),
            "CIM Leaf Deployment".to_string(),
            "Example deployment configuration".to_string(),
        );
        
        // Create initial deployment graph
        Self::create_example_deployment(&mut graph);
        
        (
            Self {
                graph,
                selected_node: None,
                show_details: true,
            },
            Task::none()
        )
    }
    
    fn create_example_deployment(graph: &mut Graph) {
        // Add a simple web service deployment
        let web_id = NodeId::new();
        let web_node = DeploymentNodeType::Service {
            name: "web-service".to_string(),
            command: "python -m http.server".to_string(),
            args: vec!["8080".to_string()],
            environment: HashMap::new(),
            port: Some(8080),
            health_check: None,
            resources: ResourceRequirements {
                cpu_cores: Some(0.5),
                memory_mb: Some(256),
                disk_gb: None,
            },
        };
        graph.add_node(
            web_id,
            "Service".to_string(),
            create_deployment_node_metadata(web_node),
        ).ok();
        
        // Add database
        let db_id = NodeId::new();
        let db_node = DeploymentNodeType::Database {
            name: "postgres".to_string(),
            engine: DatabaseEngine::PostgreSQL,
            version: "15".to_string(),
            persistent: true,
            backup_schedule: Some("0 2 * * *".to_string()),
            resources: ResourceRequirements {
                cpu_cores: Some(1.0),
                memory_mb: Some(1024),
                disk_gb: Some(20),
            },
        };
        graph.add_node(
            db_id,
            "Database".to_string(),
            create_deployment_node_metadata(db_node),
        ).ok();
        
        // Add dependency
        graph.add_edge(
            EdgeId::new(),
            web_id,
            db_id,
            "DependsOn".to_string(),
            HashMap::new(),
        ).ok();
    }
    
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AddService => {
                let node_id = NodeId::new();
                let service = DeploymentNodeType::Service {
                    name: format!("service-{}", self.graph.nodes().len() + 1),
                    command: "echo 'Hello World'".to_string(),
                    args: vec![],
                    environment: HashMap::new(),
                    port: Some(8000 + self.graph.nodes().len() as u16),
                    health_check: None,
                    resources: ResourceRequirements::default(),
                };
                self.graph.add_node(
                    node_id,
                    "Service".to_string(),
                    create_deployment_node_metadata(service),
                ).ok();
            }
            Message::AddDatabase => {
                let node_id = NodeId::new();
                let db = DeploymentNodeType::Database {
                    name: format!("database-{}", self.graph.nodes().len() + 1),
                    engine: DatabaseEngine::PostgreSQL,
                    version: "15".to_string(),
                    persistent: true,
                    backup_schedule: None,
                    resources: ResourceRequirements::default(),
                };
                self.graph.add_node(
                    node_id,
                    "Database".to_string(),
                    create_deployment_node_metadata(db),
                ).ok();
            }
            Message::AddAgent => {
                let node_id = NodeId::new();
                let agent = DeploymentNodeType::Agent {
                    name: format!("agent-{}", self.graph.nodes().len() + 1),
                    capabilities: vec!["monitor".to_string()],
                    subscriptions: vec!["events.*".to_string()],
                    rate_limit: None,
                    resources: ResourceRequirements::default(),
                };
                self.graph.add_node(
                    node_id,
                    "Agent".to_string(),
                    create_deployment_node_metadata(agent),
                ).ok();
            }
            Message::AddLoadBalancer => {
                let node_id = NodeId::new();
                let lb = DeploymentNodeType::LoadBalancer {
                    name: "nginx-lb".to_string(),
                    strategy: cim_domain_graph::deployment::LoadBalancingStrategy::RoundRobin,
                    health_check_interval: std::time::Duration::from_secs(5),
                    backends: vec![],
                };
                self.graph.add_node(
                    node_id,
                    "LoadBalancer".to_string(),
                    create_deployment_node_metadata(lb),
                ).ok();
            }
            Message::SelectNode(node_id) => {
                self.selected_node = Some(node_id);
            }
            Message::ToggleDetails => {
                self.show_details = !self.show_details;
            }
            Message::GenerateNix => {
                println!("Generating Nix configuration from graph...");
                // This would use the GraphToNixTranslator
                println!("Graph has {} nodes and {} edges", 
                    self.graph.nodes().len(), 
                    self.graph.edges().len()
                );
            }
        }
        Task::none()
    }
    
    fn view(&self) -> Element<Message> {
        let title = text("Deployment Graph Visualization")
            .size(24);
        
        let controls = row![
            button("Add Service").on_press(Message::AddService),
            button("Add Database").on_press(Message::AddDatabase),
            button("Add Agent").on_press(Message::AddAgent),
            button("Add Load Balancer").on_press(Message::AddLoadBalancer),
            Space::with_width(20),
            button("Generate Nix").on_press(Message::GenerateNix),
            Space::with_width(20),
            button(if self.show_details { "Hide Details" } else { "Show Details" })
                .on_press(Message::ToggleDetails),
        ]
        .spacing(10);
        
        let graph_info = column![
            text(format!("Nodes: {}", self.graph.nodes().len())),
            text(format!("Edges: {}", self.graph.edges().len())),
        ]
        .spacing(5);
        
        let node_list = column(
            self.graph.nodes()
                .iter()
                .map(|(id, node)| {
                    let is_selected = self.selected_node.as_ref() == Some(id);
                    let node_button = button(
                        text(format!("{} ({})", id, node.node_type))
                            .size(14)
                    )
                    .on_press(Message::SelectNode(*id))
                    .style(if is_selected {
                        button::primary
                    } else {
                        button::secondary
                    });
                    
                    Element::from(node_button)
                })
                .collect()
        )
        .spacing(5);
        
        let mut main_content = column![
            title,
            Space::with_height(10),
            controls,
            Space::with_height(20),
            row![
                column![
                    text("Graph Overview").size(18),
                    Space::with_height(10),
                    graph_info,
                    Space::with_height(20),
                    text("Nodes:").size(16),
                    node_list,
                ]
                .width(300),
                Space::with_width(20),
                self.view_details(),
            ]
        ]
        .spacing(10)
        .padding(20);
        
        container(main_content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
    
    fn view_details(&self) -> Element<Message> {
        if !self.show_details {
            return column![].into();
        }
        
        let content = if let Some(node_id) = &self.selected_node {
            if let Some(node) = self.graph.nodes().get(node_id) {
                let mut details = vec![
                    text(format!("Selected Node: {}", node_id)).size(18).into(),
                    Space::with_height(10).into(),
                    text(format!("Type: {}", node.node_type)).into(),
                ];
                
                if let Some(deployment_data) = node.metadata.get("deployment") {
                    if let Ok(node_type) = serde_json::from_value::<DeploymentNodeType>(deployment_data.clone()) {
                        details.push(Space::with_height(10).into());
                        details.push(text("Deployment Configuration:").size(16).into());
                        
                        match node_type {
                            DeploymentNodeType::Service { name, port, command, resources, .. } => {
                                details.push(text(format!("  Name: {}", name)).into());
                                details.push(text(format!("  Command: {}", command)).into());
                                if let Some(p) = port {
                                    details.push(text(format!("  Port: {}", p)).into());
                                }
                                if let Some(cpu) = resources.cpu_cores {
                                    details.push(text(format!("  CPU: {} cores", cpu)).into());
                                }
                                if let Some(mem) = resources.memory_mb {
                                    details.push(text(format!("  Memory: {} MB", mem)).into());
                                }
                            }
                            DeploymentNodeType::Database { name, engine, version, persistent, .. } => {
                                details.push(text(format!("  Name: {}", name)).into());
                                details.push(text(format!("  Engine: {:?}", engine)).into());
                                details.push(text(format!("  Version: {}", version)).into());
                                details.push(text(format!("  Persistent: {}", persistent)).into());
                            }
                            DeploymentNodeType::Agent { name, capabilities, .. } => {
                                details.push(text(format!("  Name: {}", name)).into());
                                details.push(text(format!("  Capabilities: {}", capabilities.join(", "))).into());
                            }
                            DeploymentNodeType::LoadBalancer { name, strategy, backends, .. } => {
                                details.push(text(format!("  Name: {}", name)).into());
                                details.push(text(format!("  Strategy: {:?}", strategy)).into());
                                details.push(text(format!("  Backends: {}", backends.len())).into());
                            }
                            _ => {}
                        }
                    }
                }
                
                // Show edges
                let outgoing = self.graph.edges()
                    .values()
                    .filter(|e| e.source_id == *node_id)
                    .count();
                let incoming = self.graph.edges()
                    .values()
                    .filter(|e| e.target_id == *node_id)
                    .count();
                
                details.push(Space::with_height(10).into());
                details.push(text(format!("Connections: {} out, {} in", outgoing, incoming)).into());
                
                column(details)
            } else {
                column![text("Node not found")]
            }
        } else {
            column![
                text("Select a node to view details").size(16),
                Space::with_height(20),
                text("This deployment graph represents:").size(14),
                text("• Services that will run on the CIM Leaf"),
                text("• Databases and storage systems"),
                text("• AI agents for automation"),
                text("• Load balancers and networking"),
                Space::with_height(10),
                text("The graph will be translated to a NixOS").size(14),
                text("configuration (flake.nix) for deployment."),
            ]
        };
        
        container(content)
            .width(400)
            .padding(10)
            .style(container::bordered_box)
            .into()
    }
    
    fn theme(&self) -> Theme {
        Theme::Dark
    }
}