//! Main Alchemist shell implementation

use anyhow::{Result, bail};
use console::Style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::collections::HashMap;
use tracing::{info, warn, error};

// Import command types
use crate::shell_commands::{AiCommands, DialogCommands, PolicyCommands, DomainCommands, DeployCommands, WorkflowCommands, EventCommands, AlertCommands, GraphCommands};
use crate::render_commands::RenderCommands;
use std::sync::Arc;

use crate::{
    config::AlchemistConfig,
    progress::{Progress, ProgressFormat},
    ai::AiManager,
    dialog::DialogManager,
    policy::PolicyManager,
    domain::DomainManager,
    deployment::DeploymentManager,
    renderer::RendererManager,
    workflow::WorkflowManager,
    event_monitor::{EventMonitor, EventFilter, EventSeverity, ExportFormat, parse_filter_dsl},
};

pub struct AlchemistShell {
    pub config: AlchemistConfig,
    pub ai_manager: AiManager,
    pub dialog_manager: DialogManager,
    pub policy_manager: PolicyManager,
    pub domain_manager: DomainManager,
    pub deployment_manager: DeploymentManager,
    pub renderer_manager: RendererManager,
    pub workflow_manager: WorkflowManager,
    pub nats_client: Option<async_nats::Client>,
    pub event_monitor: Option<Arc<EventMonitor>>,
}

impl AlchemistShell {
    pub async fn new(config: AlchemistConfig) -> Result<Self> {
        info!("Initializing Alchemist shell...");
        
        // Try to connect to NATS if configured
        let nats_client = if let Some(nats_url) = &config.general.nats_url {
            match async_nats::connect(nats_url).await {
                Ok(client) => {
                    info!("Connected to NATS at {}", nats_url);
                    Some(client)
                }
                Err(e) => {
                    warn!("Could not connect to NATS: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        let ai_manager = AiManager::new(&config).await?;
        let dialog_manager = DialogManager::new(&config).await?;
        let policy_manager = PolicyManager::new(&config).await?;
        let domain_manager = DomainManager::new(&config).await?;
        let deployment_manager = DeploymentManager::new(&config).await?;
        let renderer_manager = RendererManager::new()?;
        let workflow_manager = WorkflowManager::new(nats_client.clone()).await?;
        
        // Create event monitor if NATS is available
        let event_monitor = if let Some(ref client) = nats_client {
            // Create events database in the same directory as dialog history
            let data_dir = std::path::Path::new(&config.general.dialog_history_path)
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf();
            std::fs::create_dir_all(&data_dir).ok();
            let db_path = data_dir.join("events.db");
            
            match EventMonitor::new(
                Arc::new(client.clone()),
                db_path.to_str().unwrap_or("events.db"),
                10000,
            ).await {
                Ok(monitor) => {
                    let monitor = Arc::new(monitor);
                    // Start monitoring in background
                    let monitor_clone = monitor.clone();
                    tokio::spawn(async move {
                        if let Err(e) = monitor_clone.start_monitoring().await {
                            error!("Event monitoring error: {}", e);
                        }
                    });
                    Some(monitor)
                }
                Err(e) => {
                    warn!("Could not create event monitor: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        Ok(Self {
            config,
            ai_manager,
            dialog_manager,
            policy_manager,
            domain_manager,
            deployment_manager,
            renderer_manager,
            workflow_manager,
            nats_client,
            event_monitor,
        })
    }
    
    /// Run interactive shell mode
    pub async fn run_interactive(&self) -> Result<()> {
        let title_style = Style::new().bold().cyan();
        
        println!("{}", title_style.apply_to("🚀 Alchemist Interactive Shell"));
        println!("Type 'help' for commands, 'exit' to quit");
        println!("Press TAB for command completion, UP/DOWN for history\n");
        
        let mut enhanced_shell = crate::shell_enhanced::EnhancedShell::new();
        
        loop {
            let input = enhanced_shell.read_line()?;
            
            let parts: Vec<&str> = input.trim().split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            
            match parts[0] {
                "help" | "?" => self.show_help(),
                "exit" | "quit" => {
                    println!("Goodbye! 👋");
                    break;
                }
                "status" => self.show_status().await?,
                "progress" => {
                    self.show_progress(
                        &self.config.general.progress_file_path,
                        ProgressFormat::Tree
                    ).await?;
                }
                "ai" => self.handle_interactive_ai(&parts[1..]).await?,
                "dialog" => self.handle_interactive_dialog(&parts[1..]).await?,
                "policy" => self.handle_interactive_policy(&parts[1..]).await?,
                "domain" => self.handle_interactive_domain(&parts[1..]).await?,
                "deploy" => self.handle_interactive_deploy(&parts[1..]).await?,
                "render" => self.handle_interactive_render(&parts[1..]).await?,
                "workflow" => self.handle_interactive_workflow(&parts[1..]).await?,
                "event" => self.handle_interactive_event(&parts[1..]).await?,
                "graph" => self.handle_interactive_graph(&parts[1..]).await?,
                "dashboard" => {
                    println!("Launching real-time dashboard...");
                    
                    // Create real-time manager if NATS is available
                    if let Some(nats_client) = &self.nats_client {
                        let realtime_manager = Arc::new(
                            crate::dashboard_realtime::DashboardRealtimeManager::new(
                                nats_client.clone(),
                                Arc::new(crate::renderer_api::RendererApi::new()),
                            )
                        );
                        
                        // Start the real-time update system
                        realtime_manager.clone().start().await?;
                        
                        // Launch dashboard with real-time updates
                        let id = crate::dashboard_realtime::launch_realtime_dashboard(
                            &self.renderer_manager,
                            nats_client.clone(),
                            realtime_manager,
                        ).await?;
                        
                        println!("Real-time dashboard launched with ID: {}", id);
                    } else {
                        // Fallback to static dashboard
                        let id = crate::dashboard::launch_dashboard(&self.renderer_manager).await?;
                        println!("Dashboard launched with ID: {} (no real-time updates - NATS not connected)", id);
                    }
                }
                "clear" => {
                    print!("\x1B[2J\x1B[1;1H");
                }
                _ => {
                    println!("Unknown command: {}. Type 'help' for available commands.", parts[0]);
                }
            }
        }
        
        Ok(())
    }
    
    /// Show current status
    pub async fn show_status(&self) -> Result<()> {
        let style = Style::new().bold();
        
        println!("{}", style.apply_to("📊 Alchemist Status"));
        println!("─────────────────────────────");
        
        // AI Models
        let models = self.ai_manager.list_models().await?;
        println!("🤖 AI Models: {} configured", models.len());
        if let Some(default) = &self.config.general.default_ai_model {
            println!("   Default: {}", default);
        }
        
        // Dialogs
        let dialog_count = self.dialog_manager.count_dialogs().await?;
        println!("💬 Dialogs: {} stored", dialog_count);
        
        // Policies
        let policy_count = self.policy_manager.count_policies().await?;
        println!("📜 Policies: {} active", policy_count);
        
        // Domains
        let domains = self.domain_manager.list_domains().await?;
        let enabled = domains.iter().filter(|d| d.enabled).count();
        println!("🔧 Domains: {}/{} enabled", enabled, domains.len());
        
        // Deployments
        let deployments = self.deployment_manager.list_deployments().await?;
        println!("🚀 Deployments: {} configured", deployments.len());
        
        // NATS Connection
        if let Some(nats_url) = &self.config.general.nats_url {
            println!("🔌 NATS: {}", nats_url);
        }
        
        println!();
        
        Ok(())
    }
    
    /// Show progress
    pub async fn show_progress(&self, file: &str, format: ProgressFormat) -> Result<()> {
        let path = Path::new(file);
        
        if !path.exists() {
            error!("Progress file not found: {}", file);
            return Ok(());
        }
        
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
        );
        pb.set_message("Loading progress data...");
        
        let progress = Progress::load(path).await?;
        pb.finish_and_clear();
        
        match format {
            ProgressFormat::Tree => {
                println!("{}", progress.format_tree());
            }
            ProgressFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&progress)?);
            }
            ProgressFormat::Summary => {
                println!("{}", progress.format_summary());
            }
            ProgressFormat::Timeline => {
                println!("{}", progress.format_timeline());
            }
        }
        
        Ok(())
    }
    
    /// Handle AI commands
    pub async fn handle_ai_command(&mut self, command: AiCommands) -> Result<()> {
        self.ai_manager.handle_command(command).await
    }
    
    /// Handle dialog commands
    pub async fn handle_dialog_command(&mut self, command: DialogCommands) -> Result<()> {
        // Check if this is a new dialog command
        let _is_new = matches!(&command, DialogCommands::New { .. });
        
        // Execute the command
        self.dialog_manager.handle_command(command).await?;
        
        Ok(())
    }
    
    /// Launch a dialog window with AI backend
    pub async fn launch_dialog_window(&self, dialog_id: &str, title: &str) -> Result<()> {
        // For now, just print a message since we can't clone AiManager
        println!("🚀 Dialog window launch requested for: {}", title);
        println!("   ID: {}", dialog_id);
        println!("   (UI launch disabled - AiManager doesn't implement Clone)");
        
        Ok(())
    }
    
    /// Handle policy commands
    pub async fn handle_policy_command(&mut self, command: PolicyCommands) -> Result<()> {
        self.policy_manager.handle_command(command).await
    }
    
    /// Handle domain commands
    pub async fn handle_domain_command(&mut self, command: DomainCommands) -> Result<()> {
        self.domain_manager.handle_command(command).await
    }
    
    /// Handle deployment commands
    pub async fn handle_deploy_command(&mut self, command: DeployCommands) -> Result<()> {
        self.deployment_manager.handle_command(command).await
    }
    
    /// Handle workflow commands
    pub async fn handle_workflow_command(&mut self, command: WorkflowCommands) -> Result<()> {
        use crate::workflow::{Workflow, WorkflowAction, load_workflow_from_yaml, load_workflow_from_json};
        use std::collections::HashMap;
        use console::style;
        
        match command {
            WorkflowCommands::New { name, description, file } => {
                if let Some(file_path) = file {
                    // Load from file
                    let workflow = if file_path.ends_with(".yaml") || file_path.ends_with(".yml") {
                        load_workflow_from_yaml(&file_path).await?
                    } else {
                        load_workflow_from_json(&file_path).await?
                    };
                    
                    let id = self.workflow_manager.create_workflow(workflow).await?;
                    println!("✅ Created workflow from file: {}", id);
                } else {
                    // Create empty workflow
                    let workflow = Workflow {
                        id: String::new(),
                        name,
                        description,
                        steps: Vec::new(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        metadata: HashMap::new(),
                    };
                    
                    let id = self.workflow_manager.create_workflow(workflow).await?;
                    println!("✅ Created new workflow: {}", id);
                    println!("   Add steps using the workflow editor or import from a file");
                }
            }
            WorkflowCommands::List => {
                let workflows = self.workflow_manager.list_workflows().await?;
                
                if workflows.is_empty() {
                    println!("No workflows found");
                } else {
                    println!("📋 Workflows:");
                    for workflow in workflows {
                        let id_display = if workflow.id.len() >= 8 {
                            &workflow.id[..8]
                        } else {
                            &workflow.id
                        };
                        println!("  {} - {} ({} steps)", 
                            style(id_display).dim(),
                            style(&workflow.name).bold(),
                            workflow.steps.len()
                        );
                        if let Some(desc) = &workflow.description {
                            println!("      {}", style(desc).italic());
                        }
                    }
                }
            }
            WorkflowCommands::Show { id } => {
                if let Some(workflow) = self.workflow_manager.get_workflow(&id).await? {
                    println!("📄 Workflow: {}", style(&workflow.name).bold());
                    println!("   ID: {}", workflow.id);
                    if let Some(desc) = &workflow.description {
                        println!("   Description: {}", desc);
                    }
                    println!("   Created: {}", workflow.created_at.format("%Y-%m-%d %H:%M:%S"));
                    println!("   Steps:");
                    
                    for step in &workflow.steps {
                        println!("   - {} ({})", style(&step.name).cyan(), step.id);
                        if !step.dependencies.is_empty() {
                            println!("     Dependencies: {}", step.dependencies.join(", "));
                        }
                        match &step.action {
                            WorkflowAction::Command { command, .. } => {
                                println!("     Action: Command - {}", command);
                            }
                            WorkflowAction::HttpRequest { url, method, .. } => {
                                println!("     Action: HTTP {} {}", method, url);
                            }
                            WorkflowAction::NatsPublish { subject, .. } => {
                                println!("     Action: NATS Publish to {}", subject);
                            }
                            _ => {
                                println!("     Action: {:?}", step.action);
                            }
                        }
                    }
                } else {
                    println!("Workflow not found: {}", id);
                }
            }
            WorkflowCommands::Run { id, inputs, file } => {
                // Parse inputs
                let input_vars = if let Some(input_file) = file {
                    let content = tokio::fs::read_to_string(&input_file).await?;
                    if input_file.ends_with(".yaml") || input_file.ends_with(".yml") {
                        serde_yaml::from_str(&content)?
                    } else {
                        serde_json::from_str(&content)?
                    }
                } else if let Some(input_json) = inputs {
                    serde_json::from_str(&input_json)?
                } else {
                    HashMap::new()
                };
                
                println!("🚀 Starting workflow execution...");
                let execution_id = self.workflow_manager.execute_workflow(&id, input_vars).await?;
                println!("✅ Workflow execution started: {}", execution_id);
                println!("   Use 'workflow status {}' to check progress", execution_id);
            }
            WorkflowCommands::Status { execution_id } => {
                if let Some(execution) = self.workflow_manager.get_execution(&execution_id).await? {
                    println!("📊 Execution Status: {:?}", execution.state);
                    println!("   Started: {}", execution.started_at.format("%Y-%m-%d %H:%M:%S"));
                    if let Some(completed) = execution.completed_at {
                        println!("   Completed: {}", completed.format("%Y-%m-%d %H:%M:%S"));
                    }
                    
                    println!("   Steps:");
                    for (step_id, state) in &execution.step_states {
                        let status_icon = match state.state {
                            crate::workflow::StepState::Pending => "⏳",
                            crate::workflow::StepState::Running => "🔄",
                            crate::workflow::StepState::Completed => "✅",
                            crate::workflow::StepState::Failed => "❌",
                            crate::workflow::StepState::Skipped => "⏭️",
                            crate::workflow::StepState::Retrying { .. } => "🔁",
                        };
                        
                        println!("   {} {} - {:?}", status_icon, step_id, state.state);
                        if let Some(error) = &state.error {
                            println!("      Error: {}", style(error).red());
                        }
                    }
                    
                    if !execution.errors.is_empty() {
                        println!("   Errors:");
                        for error in &execution.errors {
                            println!("   - {}", style(&error.error).red());
                        }
                    }
                } else {
                    println!("Execution not found: {}", execution_id);
                }
            }
            WorkflowCommands::Stop { execution_id } => {
                self.workflow_manager.stop_execution(&execution_id).await?;
                println!("⏹️  Workflow execution stopped: {}", execution_id);
            }
            WorkflowCommands::Import { path } => {
                let workflow = if path.ends_with(".yaml") || path.ends_with(".yml") {
                    load_workflow_from_yaml(&path).await?
                } else {
                    load_workflow_from_json(&path).await?
                };
                
                let id = self.workflow_manager.create_workflow(workflow.clone()).await?;
                println!("✅ Imported workflow: {} ({})", workflow.name, id);
            }
            WorkflowCommands::Export { id, output, format } => {
                if let Some(workflow) = self.workflow_manager.get_workflow(&id).await? {
                    let content = match format.as_str() {
                        "yaml" => serde_yaml::to_string(&workflow)?,
                        "json" => serde_json::to_string_pretty(&workflow)?,
                        _ => bail!("Unsupported format: {}", format),
                    };
                    
                    tokio::fs::write(&output, content).await?;
                    println!("✅ Exported workflow to: {}", output);
                } else {
                    println!("Workflow not found: {}", id);
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle render commands
    pub async fn handle_render_command(&mut self, command: RenderCommands) -> Result<()> {
        use crate::render_commands::{RenderCommands, DemoType};
        use crate::renderer::{GraphNode, GraphEdge};
        
        match command {
            RenderCommands::Graph { title, file, iced } => {
                // Load graph data from file or create demo data
                let (nodes, edges) = if let Some(file_path) = file {
                    // Load from file using the graph parser
                    let content = std::fs::read_to_string(&file_path)?;
                    
                    // Parse based on file extension
                    let (parsed_nodes, parsed_edges) = if file_path.ends_with(".json") {
                        crate::graph_parser::parse_json_graph(&content)?
                    } else if file_path.ends_with(".nix") {
                        crate::graph_parser::parse_nix_graph(&content)?
                    } else if file_path.ends_with(".md") {
                        crate::graph_parser::parse_markdown_graph(&content)?
                    } else {
                        // Try JSON by default
                        crate::graph_parser::parse_json_graph(&content)?
                    };
                    
                    // Convert to renderer format
                    let nodes = parsed_nodes.into_iter().map(|n| GraphNode {
                        id: n.id,
                        label: n.label,
                        position: Some(n.position),
                        color: n.color,
                        size: n.size,
                        metadata: n.metadata,
                    }).collect();
                    
                    let edges = parsed_edges.into_iter().map(|e| GraphEdge {
                        source: e.source_id,
                        target: e.target_id,
                        label: e.label,
                        weight: Some(e.weight),
                        color: e.color,
                    }).collect();
                    
                    (nodes, edges)
                } else {
                    // Demo data
                    (vec![
                        GraphNode {
                            id: "1".to_string(),
                            label: "Node 1".to_string(),
                            position: Some([0.0, 0.0, 0.0]),
                            color: Some([1.0, 0.0, 0.0, 1.0]),
                            size: Some(1.0),
                            metadata: serde_json::Value::Null,
                        },
                        GraphNode {
                            id: "2".to_string(),
                            label: "Node 2".to_string(),
                            position: Some([2.0, 1.0, 0.0]),
                            color: Some([0.0, 1.0, 0.0, 1.0]),
                            size: Some(1.0),
                            metadata: serde_json::Value::Null,
                        },
                    ],
                    vec![
                        GraphEdge {
                            source: "1".to_string(),
                            target: "2".to_string(),
                            label: Some("connects".to_string()),
                            weight: Some(1.0),
                            color: None,
                        },
                    ])
                };
                
                if iced {
                    println!("Note: 2D graph rendering in Iced not yet implemented");
                } else {
                    let id = self.renderer_manager.spawn_graph_3d(&title, nodes, edges).await?;
                    println!("Launched 3D graph visualization: {}", id);
                }
            }
            RenderCommands::Document { file, format } => {
                let content = std::fs::read_to_string(&file)?;
                let format = format.unwrap_or_else(|| {
                    if file.ends_with(".md") { "markdown".to_string() }
                    else if file.ends_with(".html") { "html".to_string() }
                    else { "text".to_string() }
                });
                
                let id = self.renderer_manager.spawn_document(&file, content, &format).await?;
                println!("Launched document viewer: {}", id);
            }
            RenderCommands::Edit { file, language: _ } => {
                let (content, path) = if let Some(file_path) = file {
                    let content = std::fs::read_to_string(&file_path).ok();
                    (content, Some(file_path))
                } else {
                    (None, None)
                };
                
                let id = self.renderer_manager.spawn_text_editor("Text Editor", path, content).await?;
                println!("Launched text editor: {}", id);
            }
            RenderCommands::List => {
                let active = self.renderer_manager.list_active();
                if active.is_empty() {
                    println!("No active renderer windows");
                } else {
                    println!("Active renderer windows:");
                    for (id, renderer_type, title) in active {
                        let id_display = if id.len() >= 8 {
                            &id[..8]
                        } else {
                            &id
                        };
                        println!("  {} - {} ({})", id_display, title, 
                            match renderer_type {
                                crate::renderer::RendererType::Bevy => "Bevy 3D",
                                crate::renderer::RendererType::Iced => "Iced 2D",
                            }
                        );
                    }
                }
            }
            RenderCommands::Close { id } => {
                self.renderer_manager.close(&id).await?;
                println!("Closed renderer: {}", id);
            }
            RenderCommands::Demo { demo_type } => {
                match demo_type {
                    DemoType::Graph3d => {
                        println!("Launching 3D graph demo...");
                        // Create demo graph data
                        let mut nodes = vec![];
                        let mut edges = vec![];
                        
                        // Create a simple network
                        for i in 0..10 {
                            nodes.push(GraphNode {
                                id: i.to_string(),
                                label: format!("Node {}", i),
                                position: None, // Let renderer position them
                                color: Some([
                                    (i as f32 * 0.1) % 1.0,
                                    0.5,
                                    1.0 - (i as f32 * 0.1) % 1.0,
                                    1.0
                                ]),
                                size: Some(1.0 + (i as f32 * 0.1)),
                                metadata: serde_json::json!({ "type": "demo", "index": i }),
                            });
                        }
                        
                        // Connect nodes
                        for i in 0..9 {
                            edges.push(GraphEdge {
                                source: i.to_string(),
                                target: (i + 1).to_string(),
                                label: None,
                                weight: Some(1.0),
                                color: None,
                            });
                        }
                        
                        let id = self.renderer_manager.spawn_graph_3d("3D Graph Demo", nodes, edges).await?;
                        println!("Launched 3D graph demo: {}", id);
                    }
                    DemoType::Document => {
                        let content = r#"# Alchemist Demo Document

This is a demonstration of the Iced document viewer.

## Features

- **Markdown rendering**
- Syntax highlighting
- Scrollable content
- Responsive layout

## Code Example

```rust
fn main() {
    println!("Hello from Alchemist!");
}
```

## Lists

1. First item
2. Second item
3. Third item

- Bullet point
- Another point
  - Nested point
"#;
                        let id = self.renderer_manager.spawn_document("Document Demo", content.to_string(), "markdown").await?;
                        println!("Launched document viewer demo: {}", id);
                    }
                    DemoType::Markdown => {
                        let content = std::fs::read_to_string("alchemist-renderer/examples/markdown_example.md")
                            .unwrap_or_else(|_| r#"# Markdown Demo

This is a fallback markdown demo when the example file is not found.

## Features
- **Bold text** and *italic text*
- `Inline code` blocks
- Lists and tables
- Blockquotes

> This is a blockquote example.

### Code Example
```rust
fn main() {
    println!("Hello from Alchemist!");
}
```
"#.to_string());
                        
                        let id = self.renderer_manager.spawn_markdown("Markdown Demo", content, Some("light")).await?;
                        println!("Launched markdown viewer demo: {}", id);
                    }
                    DemoType::Chart => {
                        let chart_data = std::fs::read_to_string("alchemist-renderer/examples/chart_example.json")
                            .unwrap_or_else(|_| r#"{
  "data": [
    {
      "name": "Demo Series",
      "data": [
        {"x": 0, "y": 10, "label": "Point 1"},
        {"x": 1, "y": 25, "label": "Point 2"},
        {"x": 2, "y": 20, "label": "Point 3"},
        {"x": 3, "y": 30, "label": "Point 4"},
        {"x": 4, "y": 35, "label": "Point 5"}
      ],
      "color": [0.12, 0.47, 0.71, 1.0]
    }
  ],
  "chart_type": "line",
  "options": {
    "title": "Chart Demo",
    "x_label": "X Axis",
    "y_label": "Y Axis",
    "show_grid": true,
    "show_legend": true
  }
}"#.to_string());
                        
                        let data: serde_json::Value = serde_json::from_str(&chart_data)?;
                        let id = self.renderer_manager.spawn_chart(
                            "Chart Demo",
                            data["data"].clone(),
                            "line",
                            data["options"].clone()
                        ).await?;
                        println!("Launched chart viewer demo: {}", id);
                    }
                    _ => {
                        println!("Demo type not yet implemented");
                    }
                }
            }
            RenderCommands::Markdown { file, theme } => {
                let content = std::fs::read_to_string(&file)?;
                let id = self.renderer_manager.spawn_markdown(&file, content, Some(&theme)).await?;
                println!("Launched markdown viewer: {}", id);
            }
            RenderCommands::Chart { file, chart_type, title } => {
                let content = std::fs::read_to_string(&file)?;
                let data: serde_json::Value = serde_json::from_str(&content)?;
                
                let chart_title = title.unwrap_or_else(|| "Chart Viewer".to_string());
                let id = self.renderer_manager.spawn_chart(
                    &chart_title,
                    data["data"].clone(),
                    &chart_type,
                    data.get("options").cloned().unwrap_or(serde_json::json!({}))
                ).await?;
                println!("Launched chart viewer: {}", id);
            }
            _ => {
                println!("Render command not yet implemented");
            }
        }
        Ok(())
    }
    
    // Interactive command handlers
    async fn handle_interactive_ai(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!("AI commands: list, add, remove, test");
            return Ok(());
        }
        
        match args[0] {
            "list" => {
                let models = self.ai_manager.list_models().await?;
                for (name, config) in models {
                    println!("  {} - {} ({})", name, config.model_name, config.provider);
                }
            }
            _ => println!("Unknown AI command: {}", args[0]),
        }
        
        Ok(())
    }
    
    async fn handle_interactive_dialog(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!("Dialog commands: new, list, continue, ui");
            return Ok(());
        }
        
        match args[0] {
            "new" => {
                println!("Starting new dialog...");
                // Would launch dialog interface
            }
            "list" => {
                let dialogs = self.dialog_manager.list_recent(5).await?;
                for dialog in dialogs {
                    println!("  {} - {}", dialog.id, dialog.title);
                }
            }
            "ui" => {
                // Create a new dialog if ID not provided
                let (dialog_id, title) = if args.len() > 1 { 
                    (args[1].to_string(), format!("Dialog - {}", args[1]))
                } else {
                    let id = uuid::Uuid::new_v4().to_string();
                    let title = format!("New Dialog - {}", chrono::Local::now().format("%Y-%m-%d %H:%M"));
                    
                    // Create a new dialog through the dialog manager
                    // We'll use the new_dialog_cli method indirectly by saving the dialog
                    let dialog = crate::dialog::Dialog {
                        id: id.clone(),
                        title: title.clone(),
                        model: self.config.general.default_ai_model.clone().unwrap_or_else(|| "claude-3-sonnet".to_string()),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        messages: Vec::new(),
                        metadata: crate::dialog::DialogMetadata {
                            domain: None,
                            context: None,
                            tags: Vec::new(),
                            total_tokens: 0,
                        },
                    };
                    
                    // Save the dialog to the manager's storage
                    let dialog_json = serde_json::to_string_pretty(&dialog)?;
                    let dialog_path = self.config.general.dialog_history_path.trim_end_matches('/');
                    let dialog_file = format!("{}/dialog_{}.json", dialog_path, id);
                    std::fs::write(&dialog_file, dialog_json)?;
                    
                    (id, title)
                };
                
                println!("Launching AI dialog window...");
                
                // Launch the dialog window with AI integration
                let dialog_manager = self.dialog_manager.clone();
                let ai_manager = self.ai_manager.clone();
                
                // Spawn the dialog handler and window in a separate task
                tokio::spawn(async move {
                    if let Err(e) = crate::dialog_handler::launch_dialog_window_with_ai(
                        dialog_id.clone(),
                        title,
                        dialog_manager,
                        ai_manager,
                    ).await {
                        eprintln!("Failed to launch dialog window: {}", e);
                    }
                });
                
                println!("Dialog window launched in background");
            }
            _ => println!("Unknown dialog command: {}", args[0]),
        }
        
        Ok(())
    }
    
    async fn handle_interactive_policy(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!("Policy commands: list, show, edit, new");
            return Ok(());
        }
        
        match args[0] {
            "list" => {
                let policies = self.policy_manager.list_policies(None).await?;
                for policy in policies {
                    println!("  {} - {}", policy.id, policy.name);
                }
            }
            _ => println!("Unknown policy command: {}", args[0]),
        }
        
        Ok(())
    }
    
    async fn handle_interactive_domain(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!("Domain commands: list, tree, show, graph");
            return Ok(());
        }
        
        match args[0] {
            "list" => {
                let domains = self.domain_manager.list_domains().await?;
                for domain in domains {
                    let status = if domain.enabled { "✓" } else { "✗" };
                    println!("  [{}] {} - {}", status, domain.name, domain.description);
                }
            }
            "tree" => {
                let tree = self.domain_manager.show_hierarchy(None).await?;
                println!("{}", tree);
            }
            _ => println!("Unknown domain command: {}", args[0]),
        }
        
        Ok(())
    }
    
    async fn handle_interactive_deploy(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!("Deploy commands: list, deploy, status");
            return Ok(());
        }
        
        match args[0] {
            "list" => {
                let deployments = self.deployment_manager.list_deployments().await?;
                for deployment in deployments {
                    println!("  {} - {} ({})", deployment.name, deployment.environment, deployment.nats_url);
                }
            }
            _ => println!("Unknown deploy command: {}", args[0]),
        }
        
        Ok(())
    }
    
    async fn handle_interactive_workflow(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!("Workflow commands: new, list, show, run, status, stop, import, export");
            return Ok(());
        }
        
        match args[0] {
            "list" => {
                let workflows = self.workflow_manager.list_workflows().await?;
                if workflows.is_empty() {
                    println!("No workflows found");
                } else {
                    for workflow in workflows {
                        let id_display = if workflow.id.len() >= 8 {
                            &workflow.id[..8]
                        } else {
                            &workflow.id
                        };
                        println!("  {} - {} ({} steps)", 
                            id_display, 
                            workflow.name,
                            workflow.steps.len()
                        );
                    }
                }
            }
            "new" => {
                println!("Use 'alchemist workflow new <name>' to create a new workflow");
            }
            _ => println!("Unknown workflow command: {}", args[0]),
        }
        
        Ok(())
    }
    
    async fn handle_interactive_render(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!("Render commands: graph, document, edit, list, close, demo");
            return Ok(());
        }
        
        match args[0] {
            "graph" => {
                println!("Launching 3D graph visualization...");
                let nodes = vec![
                    crate::renderer::GraphNode {
                        id: "demo1".to_string(),
                        label: "Demo Node".to_string(),
                        position: Some([0.0, 0.0, 0.0]),
                        color: Some([0.5, 0.5, 1.0, 1.0]),
                        size: Some(1.5),
                        metadata: serde_json::Value::Null,
                    }
                ];
                let id = self.renderer_manager.spawn_graph_3d("Interactive Graph", nodes, vec![]).await?;
                println!("Launched graph: {}", id);
            }
            "list" => {
                let active = self.renderer_manager.list_active();
                if active.is_empty() {
                    println!("No active renderer windows");
                } else {
                    for (id, renderer_type, title) in active {
                        let renderer_name = match renderer_type {
                            crate::renderer::RendererType::Bevy => "Bevy",
                            crate::renderer::RendererType::Iced => "Iced",
                        };
                        let id_display = if id.len() >= 8 {
                            &id[..8]
                        } else {
                            &id
                        };
                        println!("  [{}] {} - {}", id_display, title, renderer_name);
                    }
                }
            }
            "demo" => {
                println!("Available demos: graph3d, document, markdown, chart");
            }
            "chart" => {
                // Launch demo chart
                println!("Launching chart demo...");
                let chart_data = serde_json::json!({
                    "series": [
                        {
                            "name": "Sample Data",
                            "color": [0.2, 0.6, 1.0],
                            "data": [
                                {"x": 1.0, "y": 10.0},
                                {"x": 2.0, "y": 20.0},
                                {"x": 3.0, "y": 15.0},
                                {"x": 4.0, "y": 25.0},
                                {"x": 5.0, "y": 30.0},
                            ]
                        }
                    ],
                    "title": "Demo Chart",
                    "x_label": "X Axis",
                    "y_label": "Y Axis"
                });
                
                let id = self.renderer_manager.spawn_chart(
                    "Chart Demo",
                    chart_data,
                    "line",
                    serde_json::json!({}),
                ).await?;
                println!("Launched chart: {}", id);
            }
            "markdown" => {
                if args.len() > 1 {
                    // Load markdown file
                    match std::fs::read_to_string(args[1]) {
                        Ok(content) => {
                            println!("Opening markdown file: {}", args[1]);
                            let theme = if args.len() > 2 { Some(args[2].to_string()) } else { None };
                            let id = self.renderer_manager.spawn_markdown(
                                args[1],
                                content,
                                theme.as_deref(),
                            ).await?;
                            println!("Launched markdown viewer: {}", id);
                        }
                        Err(e) => {
                            println!("Error reading file: {}", e);
                        }
                    }
                } else {
                    println!("Usage: render markdown <file> [theme]");
                    println!("Theme: 'light' or 'dark' (default: dark)");
                }
            }
            _ => println!("Unknown render command: {}", args[0]),
        }
        
        Ok(())
    }
    
    fn show_help(&self) {
        let help_style = Style::new().dim();
        println!("Available commands:");
        println!("  {} - Show this help", help_style.apply_to("help, ?"));
        println!("  {} - Show system status", help_style.apply_to("status"));
        println!("  {} - Show project progress", help_style.apply_to("progress"));
        println!("  {} - Manage AI models", help_style.apply_to("ai"));
        println!("  {} - Manage dialogs", help_style.apply_to("dialog"));
        println!("  {} - Manage policies", help_style.apply_to("policy"));
        println!("  {} - Manage domains", help_style.apply_to("domain"));
        println!("  {} - Manage deployments", help_style.apply_to("deploy"));
        println!("  {} - Launch renderer windows", help_style.apply_to("render"));
        println!("  {} - Manage workflows", help_style.apply_to("workflow"));
        println!("  {} - Launch domain dashboard", help_style.apply_to("dashboard"));
        println!("  {} - Monitor system events", help_style.apply_to("event"));
        println!("  {} - Manage graph operations", help_style.apply_to("graph"));
        println!("  {} - Clear screen", help_style.apply_to("clear"));
        println!("  {} - Exit shell", help_style.apply_to("exit, quit"));
    }
    
    /// Handle event command
    pub async fn handle_event_command(&self, command: EventCommands) -> Result<()> {
        if let Some(monitor) = &self.event_monitor {
            match command {
                EventCommands::List { count, domain, event_type, severity } => {
                    // Build filter
                    let filter = EventFilter {
                        domains: domain.map(|d| vec![d]),
                        event_types: event_type.map(|t| vec![t]),
                        min_severity: severity.and_then(|s| parse_severity(&s)),
                        time_range: None,
                        correlation_id: None,
                        subject_pattern: None,
                        metadata_filters: HashMap::new(),
                    };
                    
                    // Query events
                    let events = monitor.query_events(&filter).await?;
                    
                    // Display events
                    println!("Recent Events (showing up to {}):", count);
                    println!("─────────────────────────────────────────────────────");
                    
                    for (i, event) in events.iter().take(count).enumerate() {
                        println!("{:>4}. [{:^8}] {} - {} ({})",
                            i + 1,
                            event.severity,
                            event.timestamp.format("%H:%M:%S"),
                            event.event_type,
                            event.domain
                        );
                        if let Some(corr_id) = &event.correlation_id {
                            let id_display = if corr_id.len() >= 8 {
                                &corr_id[..8]
                            } else {
                                corr_id
                            };
                            println!("      Correlation: {}", id_display);
                        }
                    }
                }
                
                EventCommands::Watch { filter, interval: _ } => {
                    println!("Starting real-time event monitoring (Ctrl+C to stop)...");
                    
                    // Parse filter if provided
                    let _filter_expr = filter.map(|f| parse_filter_dsl(&f)).transpose()?;
                    
                    // TODO: Implement real-time watching
                    println!("Real-time monitoring not yet implemented");
                }
                
                EventCommands::Query { criteria, limit, format } => {
                    // Parse filter DSL
                    let _filter_expr = parse_filter_dsl(&criteria)?;
                    
                    // TODO: Convert filter expression to EventFilter
                    println!("Querying events with: {}", criteria);
                    
                    // For now, just query all
                    let events = monitor.query_events(&EventFilter {
                        domains: None,
                        event_types: None,
                        min_severity: None,
                        time_range: None,
                        correlation_id: None,
                        subject_pattern: None,
                        metadata_filters: HashMap::new(),
                    }).await?;
                    
                    match format.as_str() {
                        "json" => {
                            println!("{}", serde_json::to_string_pretty(&events)?);
                        }
                        "yaml" => {
                            println!("{}", serde_yaml::to_string(&events)?);
                        }
                        _ => {
                            // Table format
                            println!("Timestamp             | Domain    | Type         | Severity | Subject");
                            println!("──────────────────────┼───────────┼──────────────┼──────────┼─────────");
                            for event in events.iter().take(limit) {
                                println!("{} | {:^9} | {:^12} | {:^8} | {}",
                                    event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                                    truncate(&event.domain, 9),
                                    truncate(&event.event_type, 12),
                                    event.severity,
                                    truncate(&event.subject, 30)
                                );
                            }
                        }
                    }
                }
                
                EventCommands::Stats { window: _, group_by } => {
                    let stats = monitor.get_statistics().await;
                    
                    println!("Event Statistics");
                    println!("═══════════════════════════════════════");
                    println!("Total Events: {}", stats.total_count);
                    println!("Avg Processing Time: {:.2}ms", stats.avg_processing_time_ms);
                    
                    if let Some(group) = group_by {
                        match group.as_str() {
                            "domain" => {
                                println!("\nBy Domain:");
                                for (domain, count) in stats.by_domain.iter() {
                                    println!("  {}: {}", domain, count);
                                }
                            }
                            "type" => {
                                println!("\nBy Type:");
                                for (event_type, count) in stats.by_type.iter() {
                                    println!("  {}: {}", event_type, count);
                                }
                            }
                            "severity" => {
                                println!("\nBy Severity:");
                                for (severity, count) in stats.by_severity.iter() {
                                    println!("  {}: {}", severity, count);
                                }
                            }
                            _ => println!("Unknown grouping field: {}", group),
                        }
                    }
                    
                    // Show events per minute graph
                    println!("\nEvents per minute (last 60 minutes):");
                    if !stats.events_per_minute.is_empty() {
                        let max = *stats.events_per_minute.iter().max().unwrap_or(&0);
                        if max > 0 {
                            for (i, &count) in stats.events_per_minute.iter().enumerate() {
                                let bar_len = (count as f64 / max as f64 * 40.0) as usize;
                                let bar = "█".repeat(bar_len);
                                println!("{:>2}: {} {}", 60 - i, bar, count);
                            }
                        }
                    }
                }
                
                EventCommands::Export { format, output, filter, start, end } => {
                    // Build filter
                    let mut event_filter = EventFilter {
                        domains: None,
                        event_types: None,
                        min_severity: None,
                        time_range: None,
                        correlation_id: None,
                        subject_pattern: None,
                        metadata_filters: HashMap::new(),
                    };
                    
                    // Parse time range if provided
                    if let (Some(start_str), Some(end_str)) = (start, end) {
                        use chrono::DateTime;
                        let start_time = DateTime::parse_from_rfc3339(&start_str)?.with_timezone(&chrono::Utc);
                        let end_time = DateTime::parse_from_rfc3339(&end_str)?.with_timezone(&chrono::Utc);
                        event_filter.time_range = Some(crate::event_monitor::TimeRange {
                            start: start_time,
                            end: end_time,
                        });
                    }
                    
                    // Parse filter DSL if provided
                    if let Some(filter_str) = filter {
                        // TODO: Convert filter DSL to EventFilter
                        println!("Filter: {}", filter_str);
                    }
                    
                    // Export events
                    let export_format = format.parse::<ExportFormat>()?;
                    monitor.export_events(&event_filter, export_format, &output).await?;
                    
                    println!("Events exported to: {}", output);
                }
                
                EventCommands::Alert { command } => {
                    self.handle_alert_command(command).await?;
                }
            }
        } else {
            println!("Event monitoring not available (NATS not connected)");
        }
        
        Ok(())
    }
    
    /// Handle alert subcommands
    async fn handle_alert_command(&self, command: AlertCommands) -> Result<()> {
        if let Some(_monitor) = &self.event_monitor {
            match command {
                AlertCommands::List => {
                    println!("Alert rules not yet implemented");
                }
                AlertCommands::Add { name, filter, action, target, throttle } => {
                    println!("Adding alert rule: {}", name);
                    println!("  Filter: {}", filter);
                    println!("  Action: {}", action);
                    if let Some(t) = target {
                        println!("  Target: {}", t);
                    }
                    if let Some(th) = throttle {
                        println!("  Throttle: {}s", th);
                    }
                    println!("Alert rules not yet implemented");
                }
                AlertCommands::Remove { id } => {
                    println!("Removing alert rule: {}", id);
                    println!("Alert rules not yet implemented");
                }
                AlertCommands::Test { id } => {
                    println!("Testing alert rule: {}", id);
                    println!("Alert rules not yet implemented");
                }
            }
        } else {
            println!("Event monitoring not available (NATS not connected)");
        }
        
        Ok(())
    }
    
    /// Handle interactive event commands
    async fn handle_interactive_event(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!("Event commands: list, watch, query, stats, export, alert");
            return Ok(());
        }
        
        match args[0] {
            "list" => {
                self.handle_event_command(EventCommands::List {
                    count: 20,
                    domain: None,
                    event_type: None,
                    severity: None,
                }).await?;
            }
            "stats" => {
                self.handle_event_command(EventCommands::Stats {
                    window: "1h".to_string(),
                    group_by: Some("domain".to_string()),
                }).await?;
            }
            "watch" => {
                println!("Use 'alchemist event watch' to start real-time monitoring");
            }
            _ => println!("Unknown event command: {}", args[0]),
        }
        
        Ok(())
    }
    
    /// Handle interactive graph commands
    async fn handle_interactive_graph(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!("Graph commands: load, save, list, show");
            return Ok(());
        }
        
        match args[0] {
            "load" => {
                if args.len() < 2 {
                    println!("Usage: graph load <file> [id]");
                } else {
                    let file_path = args[1];
                    let graph_id = if args.len() > 2 { Some(args[2].to_string()) } else { None };
                    self.handle_graph_command(GraphCommands::Load {
                        path: file_path.to_string(),
                        id: graph_id,
                    }).await?;
                }
            }
            "save" => {
                if args.len() < 3 {
                    println!("Usage: graph save <id> <output-file> [format]");
                } else {
                    let graph_id = args[1];
                    let output = args[2];
                    let format = if args.len() > 3 { args[3].to_string() } else { "json".to_string() };
                    self.handle_graph_command(GraphCommands::Save {
                        id: graph_id.to_string(),
                        output: output.to_string(),
                        format,
                    }).await?;
                }
            }
            "list" => {
                self.handle_graph_command(GraphCommands::List).await?;
            }
            "show" => {
                if args.len() < 2 {
                    println!("Usage: graph show <id>");
                } else {
                    self.handle_graph_command(GraphCommands::Show {
                        id: args[1].to_string(),
                    }).await?;
                }
            }
            _ => println!("Unknown graph command: {}", args[0]),
        }
        
        Ok(())
    }
    
    /// Handle graph commands
    pub async fn handle_graph_command(&self, command: GraphCommands) -> Result<()> {
        match command {
            GraphCommands::Load { path, id: _ } => {
                println!("Loading graph from: {}", path);
                
                #[cfg(feature = "bevy")]
                {
                    // Check if we have a renderer manager and can send events
                    // For now, we'll just print a message
                    println!("Graph file I/O requires a Bevy renderer window to be active.");
                    println!("Launch a graph renderer with: render graph");
                    
                    // In a real implementation, we would send a GraphLoadRequest event
                    // to the active Bevy window
                }
                
                #[cfg(not(feature = "bevy"))]
                {
                    println!("Graph functionality requires the 'bevy' feature to be enabled.");
                    println!("Rebuild with: cargo build --features bevy");
                }
            }
            GraphCommands::Save { id, output, format } => {
                println!("Saving graph {} to: {}", id, output);
                
                #[cfg(feature = "bevy")]
                {
                    use crate::graph_plugin::GraphExportFormat;
                    
                    let export_format = match format.as_str() {
                        "json" => GraphExportFormat::Json,
                        "cytoscape" => GraphExportFormat::Cytoscape,
                        "graphviz" | "dot" => GraphExportFormat::Graphviz,
                        "gexf" => GraphExportFormat::Gexf,
                        _ => {
                            println!("Unknown format: {}. Using JSON.", format);
                            GraphExportFormat::Json
                        }
                    };
                    
                    println!("Export format: {:?}", export_format);
                    println!("Graph file I/O requires a Bevy renderer window to be active.");
                    println!("Launch a graph renderer with: render graph");
                    
                    // In a real implementation, we would send a GraphSaveRequest event
                    // to the active Bevy window with the graph
                }
                
                #[cfg(not(feature = "bevy"))]
                {
                    println!("Graph functionality requires the 'bevy' feature to be enabled.");
                    println!("Rebuild with: cargo build --features bevy");
                }
            }
            GraphCommands::List => {
                #[cfg(feature = "bevy")]
                {
                    println!("Graph listing requires a Bevy renderer window to be active.");
                    println!("Launch a graph renderer with: render graph");
                }
                
                #[cfg(not(feature = "bevy"))]
                {
                    println!("Graph functionality requires the 'bevy' feature to be enabled.");
                    println!("Rebuild with: cargo build --features bevy");
                }
            }
            GraphCommands::Show { id } => {
                println!("Showing graph details for: {}", id);
                
                #[cfg(feature = "bevy")]
                {
                    println!("Graph details require a Bevy renderer window to be active.");
                    println!("Launch a graph renderer with: render graph");
                }
                
                #[cfg(not(feature = "bevy"))]
                {
                    println!("Graph functionality requires the 'bevy' feature to be enabled.");
                    println!("Rebuild with: cargo build --features bevy");
                }
            }
        }
        
        Ok(())
    }
}

// Helper functions
fn parse_severity(s: &str) -> Option<EventSeverity> {
    match s.to_lowercase().as_str() {
        "debug" => Some(EventSeverity::Debug),
        "info" => Some(EventSeverity::Info),
        "warning" => Some(EventSeverity::Warning),
        "error" => Some(EventSeverity::Error),
        "critical" => Some(EventSeverity::Critical),
        _ => None,
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GeneralConfig, DomainConfig, DomainRelationship};
    use crate::render_commands::DemoType;
    use std::collections::HashMap;
    use tempfile::TempDir;
    use tokio;
    
    // Mock implementations for testing
    struct MockNatsClient;
    
    #[derive(Clone)]
    struct MockEventMonitor;
    
    impl MockEventMonitor {
        fn new() -> Self {
            MockEventMonitor
        }
    }
    
    // Helper to create test configuration
    fn create_test_config() -> AlchemistConfig {
        AlchemistConfig {
            general: GeneralConfig {
                default_ai_model: Some("test-model".to_string()),
                progress_file_path: "test_progress.json".to_string(),
                nats_url: None,
                dialog_history_path: "test_dialogs".to_string(),
                log_level: "info".to_string(),
            },
            ai_models: HashMap::new(),
            policy: crate::config::PolicyConfig {
                storage_path: "test_policies".to_string(),
                validation_enabled: true,
                evaluation_timeout: 5000,
                cache_ttl: Some(300),
            },
            deployments: HashMap::new(),
            domains: crate::config::DomainRegistryConfig {
                available: vec![],
                relationships: vec![],
            },
            cache: None,
        }
    }
    
    // Helper to create test config with temp directories
    fn create_test_config_with_temp_dir(temp_dir: &TempDir) -> AlchemistConfig {
        AlchemistConfig {
            general: GeneralConfig {
                default_ai_model: Some("test-model".to_string()),
                progress_file_path: temp_dir.path().join("progress.json").to_string_lossy().to_string(),
                nats_url: None,
                dialog_history_path: temp_dir.path().join("dialogs").to_string_lossy().to_string(),
                log_level: "info".to_string(),
            },
            ai_models: HashMap::new(),
            policy: crate::config::PolicyConfig {
                storage_path: temp_dir.path().join("policies").to_string_lossy().to_string(),
                validation_enabled: true,
                evaluation_timeout: 5000,
                cache_ttl: Some(300),
            },
            deployments: HashMap::new(),
            domains: crate::config::DomainRegistryConfig {
                available: vec![],
                relationships: vec![],
            },
            cache: None,
        }
    }
    
    // Test shell initialization
    #[tokio::test]
    async fn test_shell_initialization() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create required directories
        std::fs::create_dir_all(temp_dir.path().join("dialogs")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("policies")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("workflows")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("domains")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("deployments")).unwrap();
        
        let config = create_test_config_with_temp_dir(&temp_dir);
        
        // Test shell creation without NATS
        let shell = AlchemistShell::new(config.clone()).await;
        assert!(shell.is_ok(), "Shell should initialize successfully without NATS");
        
        let shell = shell.unwrap();
        assert!(shell.nats_client.is_none(), "NATS client should be None when not configured");
        assert!(shell.event_monitor.is_none(), "Event monitor should be None without NATS");
    }
    
    #[tokio::test]
    async fn test_shell_initialization_with_nats_url() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create required directories
        std::fs::create_dir_all(temp_dir.path().join("dialogs")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("policies")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("workflows")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("domains")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("deployments")).unwrap();
        
        let mut config = create_test_config_with_temp_dir(&temp_dir);
        // Set invalid NATS URL - connection will fail but shell should still initialize
        config.general.nats_url = Some("nats://invalid:4222".to_string());
        
        let shell = AlchemistShell::new(config).await;
        assert!(shell.is_ok(), "Shell should initialize even with invalid NATS URL");
        
        let shell = shell.unwrap();
        assert!(shell.nats_client.is_none(), "NATS client should be None when connection fails");
    }
    
    // Test status display
    #[tokio::test]
    async fn test_show_status() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create required directories
        std::fs::create_dir_all(temp_dir.path().join("dialogs")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("policies")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("workflows")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("domains")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("deployments")).unwrap();
        
        let config = create_test_config_with_temp_dir(&temp_dir);
        let shell = AlchemistShell::new(config).await.unwrap();
        
        // Test status display - should not panic
        let result = shell.show_status().await;
        assert!(result.is_ok(), "show_status should not fail");
    }
    
    // Test progress viewing
    #[tokio::test]
    #[ignore = "Requires proper Progress module setup"]
    async fn test_show_progress() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a test progress file
        let progress_file = temp_dir.path().join("progress.json");
        let progress_data = r#"{
            "project": "Test Project",
            "version": "1.0.0",
            "status": "in_progress",
            "completion_percentage": 50,
            "current_date": "2024-01-01",
            "current_focus": "Testing",
            "summary": "Test summary",
            "overall_completion": 50,
            "last_updated": "2024-01-01T00:00:00Z",
            "domains": {},
            "metrics": {
                "total_tests": 100,
                "tests_passing": 90,
                "code_coverage": 85.5,
                "cyclomatic_complexity": 5.2
            },
            "recent_changes": [],
            "milestones_achieved": [],
            "architecture_health": {
                "overall_score": 85,
                "modularity": 90,
                "coupling": 80,
                "cohesion": 85,
                "test_coverage": 85
            },
            "graph_structure": {
                "nodes": [],
                "edges": []
            }
        }"#;
        std::fs::write(&progress_file, progress_data).unwrap();
        
        // Create required directories
        std::fs::create_dir_all(temp_dir.path().join("dialogs")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("policies")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("workflows")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("domains")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("deployments")).unwrap();
        
        let config = create_test_config_with_temp_dir(&temp_dir);
        let shell = AlchemistShell::new(config).await.unwrap();
        
        // Test different progress formats
        let result = shell.show_progress(&progress_file.to_string_lossy(), ProgressFormat::Tree).await;
        assert!(result.is_ok(), "show_progress with Tree format should not fail: {:?}", result);
        
        let result = shell.show_progress(&progress_file.to_string_lossy(), ProgressFormat::Json).await;
        assert!(result.is_ok(), "show_progress with Json format should not fail");
        
        let result = shell.show_progress(&progress_file.to_string_lossy(), ProgressFormat::Summary).await;
        assert!(result.is_ok(), "show_progress with Summary format should not fail");
        
        let result = shell.show_progress(&progress_file.to_string_lossy(), ProgressFormat::Timeline).await;
        assert!(result.is_ok(), "show_progress with Timeline format should not fail");
        
        // Test with non-existent file
        let result = shell.show_progress("non_existent_file.json", ProgressFormat::Tree).await;
        assert!(result.is_ok(), "show_progress with non-existent file should handle gracefully");
    }
    
    // Test help display
    #[test] 
    fn test_show_help() {
        // We can't test this properly without initializing managers,
        // but we can at least verify the function exists
        // The actual integration test would need a full shell setup
    }
    
    // Helper to create shell with temp directories
    async fn create_test_shell() -> (AlchemistShell, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        
        // Create required directories
        std::fs::create_dir_all(temp_dir.path().join("dialogs")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("policies")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("workflows")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("domains")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("deployments")).unwrap();
        
        let config = create_test_config_with_temp_dir(&temp_dir);
        let shell = AlchemistShell::new(config).await.unwrap();
        
        (shell, temp_dir)
    }
    
    // Test interactive command handlers
    #[tokio::test]
    async fn test_handle_interactive_ai() {
        let (shell, _temp_dir) = create_test_shell().await;
        
        // Test with empty args
        let result = shell.handle_interactive_ai(&[]).await;
        assert!(result.is_ok(), "handle_interactive_ai with empty args should not fail");
        
        // Test list command
        let result = shell.handle_interactive_ai(&["list"]).await;
        assert!(result.is_ok(), "handle_interactive_ai list should not fail");
        
        // Test unknown command
        let result = shell.handle_interactive_ai(&["unknown"]).await;
        assert!(result.is_ok(), "handle_interactive_ai with unknown command should not fail");
    }
    
    #[tokio::test]
    async fn test_handle_interactive_dialog() {
        let (shell, _temp_dir) = create_test_shell().await;
        
        // Test with empty args
        let result = shell.handle_interactive_dialog(&[]).await;
        assert!(result.is_ok(), "handle_interactive_dialog with empty args should not fail");
        
        // Test various commands
        let result = shell.handle_interactive_dialog(&["new"]).await;
        assert!(result.is_ok(), "handle_interactive_dialog new should not fail");
        
        let result = shell.handle_interactive_dialog(&["list"]).await;
        assert!(result.is_ok(), "handle_interactive_dialog list should not fail");
        
        // UI commands may fail if renderer binary doesn't exist
        let _ = shell.handle_interactive_dialog(&["ui"]).await;
        let _ = shell.handle_interactive_dialog(&["ui", "test-id"]).await;
    }
    
    #[tokio::test]
    async fn test_handle_interactive_workflow() {
        let (shell, _temp_dir) = create_test_shell().await;
        
        // Test with empty args
        let result = shell.handle_interactive_workflow(&[]).await;
        assert!(result.is_ok(), "handle_interactive_workflow with empty args should not fail");
        
        // Test list command
        let result = shell.handle_interactive_workflow(&["list"]).await;
        assert!(result.is_ok(), "handle_interactive_workflow list should not fail");
        
        // Test new command
        let result = shell.handle_interactive_workflow(&["new"]).await;
        assert!(result.is_ok(), "handle_interactive_workflow new should not fail");
    }
    
    #[tokio::test]
    async fn test_handle_interactive_event() {
        let (shell, _temp_dir) = create_test_shell().await;
        
        // Test with empty args
        let result = shell.handle_interactive_event(&[]).await;
        assert!(result.is_ok(), "handle_interactive_event with empty args should not fail");
        
        // Test without event monitor (should handle gracefully)
        let result = shell.handle_interactive_event(&["list"]).await;
        assert!(result.is_ok(), "handle_interactive_event list without monitor should not fail");
        
        let result = shell.handle_interactive_event(&["stats"]).await;
        assert!(result.is_ok(), "handle_interactive_event stats without monitor should not fail");
    }
    
    // Test workflow command handling
    #[tokio::test]
    async fn test_handle_workflow_command() {
        let (mut shell, temp_dir) = create_test_shell().await;
        
        // Test new workflow command
        let command = WorkflowCommands::New {
            name: "test-workflow".to_string(),
            description: Some("Test workflow".to_string()),
            file: None,
        };
        let result = shell.handle_workflow_command(command).await;
        assert!(result.is_ok(), "handle_workflow_command new should not fail");
        
        // Test list workflows command
        let command = WorkflowCommands::List;
        let result = shell.handle_workflow_command(command).await;
        assert!(result.is_ok(), "handle_workflow_command list should not fail");
        
        // Test show workflow command with non-existent ID
        let command = WorkflowCommands::Show {
            id: "non-existent".to_string(),
        };
        let result = shell.handle_workflow_command(command).await;
        assert!(result.is_ok(), "handle_workflow_command show should not fail");
        
        // Test export workflow command with non-existent ID
        let export_file = temp_dir.path().join("export.yaml");
        let command = WorkflowCommands::Export {
            id: "non-existent".to_string(),
            output: export_file.to_string_lossy().to_string(),
            format: "yaml".to_string(),
        };
        let result = shell.handle_workflow_command(command).await;
        assert!(result.is_ok(), "handle_workflow_command export should not fail");
    }
    
    // Test render command handling
    #[tokio::test]
    #[ignore = "Requires renderer binary to exist"]
    async fn test_handle_render_command() {
        let (mut shell, _temp_dir) = create_test_shell().await;
        
        // Test graph command - may fail if renderer binary doesn't exist
        let command = RenderCommands::Graph {
            title: "Test Graph".to_string(),
            file: None,
            iced: false,
        };
        let result = shell.handle_render_command(command).await;
        // Don't assert success as renderer binary may not exist in test environment
        
        // Test list command - should always work
        let command = RenderCommands::List;
        let result = shell.handle_render_command(command).await;
        assert!(result.is_ok(), "handle_render_command list should not fail");
        
        // Test demo commands
        let command = RenderCommands::Demo {
            demo_type: DemoType::Graph3d,
        };
        let result = shell.handle_render_command(command).await;
        assert!(result.is_ok(), "handle_render_command demo graph3d should not fail");
        
        let command = RenderCommands::Demo {
            demo_type: DemoType::Document,
        };
        let result = shell.handle_render_command(command).await;
        assert!(result.is_ok(), "handle_render_command demo document should not fail");
    }
    
    // Test event command handling
    #[tokio::test]
    async fn test_handle_event_command_without_monitor() {
        let (shell, _temp_dir) = create_test_shell().await;
        
        // Test various event commands without event monitor
        let command = EventCommands::List {
            count: 10,
            domain: None,
            event_type: None,
            severity: None,
        };
        let result = shell.handle_event_command(command).await;
        assert!(result.is_ok(), "handle_event_command list without monitor should not fail");
        
        let command = EventCommands::Stats {
            window: "1h".to_string(),
            group_by: Some("domain".to_string()),
        };
        let result = shell.handle_event_command(command).await;
        assert!(result.is_ok(), "handle_event_command stats without monitor should not fail");
    }
    
    // Test helper functions
    #[test]
    fn test_parse_severity() {
        assert_eq!(parse_severity("debug"), Some(EventSeverity::Debug));
        assert_eq!(parse_severity("info"), Some(EventSeverity::Info));
        assert_eq!(parse_severity("warning"), Some(EventSeverity::Warning));
        assert_eq!(parse_severity("error"), Some(EventSeverity::Error));
        assert_eq!(parse_severity("critical"), Some(EventSeverity::Critical));
        assert_eq!(parse_severity("unknown"), None);
        assert_eq!(parse_severity("INFO"), Some(EventSeverity::Info)); // Case insensitive
    }
    
    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a long string", 10), "this is...");
        assert_eq!(truncate("exact", 5), "exact");
        assert_eq!(truncate("toolong", 5), "to...");
        assert_eq!(truncate("", 5), "");
    }
    
    // Test error handling in render commands
    #[tokio::test]
    async fn test_render_command_error_handling() {
        let (mut shell, _temp_dir) = create_test_shell().await;
        
        // Test document command with non-existent file
        let command = RenderCommands::Document {
            file: "non_existent_file.md".to_string(),
            format: Some("markdown".to_string()),
        };
        let result = shell.handle_render_command(command).await;
        assert!(result.is_err(), "handle_render_command document with non-existent file should fail");
        
        // Test markdown command with non-existent file
        let command = RenderCommands::Markdown {
            file: "non_existent_file.md".to_string(),
            theme: "light".to_string(),
        };
        let result = shell.handle_render_command(command).await;
        assert!(result.is_err(), "handle_render_command markdown with non-existent file should fail");
        
        // Test chart command with non-existent file
        let command = RenderCommands::Chart {
            file: "non_existent_chart.json".to_string(),
            chart_type: "line".to_string(),
            title: Some("Test Chart".to_string()),
        };
        let result = shell.handle_render_command(command).await;
        assert!(result.is_err(), "handle_render_command chart with non-existent file should fail");
    }
    
    // Test workflow command with file operations
    #[tokio::test]
    #[ignore = "Requires proper workflow YAML parsing"]
    async fn test_workflow_command_file_operations() {
        let (mut shell, temp_dir) = create_test_shell().await;
        
        // Create a test workflow YAML file
        let workflow_yaml = temp_dir.path().join("test_workflow.yaml");
        let yaml_content = r#"id: test-workflow
name: Test Workflow
description: A test workflow
steps:
  - id: step1
    name: First Step
    action:
      Command:
        command: echo
        args: ["Hello"]
        env: {}
    dependencies: []
    conditions: []
created_at: "2024-01-01T00:00:00Z"
updated_at: "2024-01-01T00:00:00Z"
metadata: {}
"#;
        std::fs::write(&workflow_yaml, yaml_content).unwrap();
        
        // Test import command
        let command = WorkflowCommands::Import {
            path: workflow_yaml.to_string_lossy().to_string(),
        };
        let result = shell.handle_workflow_command(command).await;
        assert!(result.is_ok(), "handle_workflow_command import should not fail");
        
        // Test with non-existent file
        let command = WorkflowCommands::Import {
            path: "non_existent_workflow.yaml".to_string(),
        };
        let result = shell.handle_workflow_command(command).await;
        assert!(result.is_err(), "handle_workflow_command import with non-existent file should fail");
    }
    
    // Test interactive command parsing
    #[tokio::test]
    async fn test_interactive_command_parsing() {
        let (shell, _temp_dir) = create_test_shell().await;
        
        // Test various interactive handlers
        let result = shell.handle_interactive_policy(&[]).await;
        assert!(result.is_ok());
        
        let result = shell.handle_interactive_domain(&[]).await;
        assert!(result.is_ok());
        
        let result = shell.handle_interactive_deploy(&[]).await;
        assert!(result.is_ok());
        
        let result = shell.handle_interactive_render(&[]).await;
        assert!(result.is_ok());
    }
}