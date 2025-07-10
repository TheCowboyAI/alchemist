//! Main Alchemist shell implementation

use anyhow::Result;
use console::Style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use tracing::{info, warn, error};

// Import command types
use crate::shell_commands::{AiCommands, DialogCommands, PolicyCommands, ClaimsCommands, DomainCommands, DeployCommands};
use crate::render_commands::RenderCommands;

use crate::{
    config::AlchemistConfig,
    progress::{Progress, ProgressFormat},
    ai::AiManager,
    dialog::{DialogManager, MessageRole},
    policy::PolicyManager,
    domain::DomainManager,
    deployment::DeploymentManager,
    renderer::RendererManager,
};

pub struct AlchemistShell {
    pub config: AlchemistConfig,
    pub ai_manager: AiManager,
    pub dialog_manager: DialogManager,
    pub policy_manager: PolicyManager,
    pub domain_manager: DomainManager,
    pub deployment_manager: DeploymentManager,
    pub renderer_manager: RendererManager,
}

impl AlchemistShell {
    pub async fn new(config: AlchemistConfig) -> Result<Self> {
        info!("Initializing Alchemist shell...");
        
        let ai_manager = AiManager::new(&config).await?;
        let dialog_manager = DialogManager::new(&config).await?;
        let policy_manager = PolicyManager::new(&config).await?;
        let domain_manager = DomainManager::new(&config).await?;
        let deployment_manager = DeploymentManager::new(&config).await?;
        let renderer_manager = RendererManager::new()?;
        
        Ok(Self {
            config,
            ai_manager,
            dialog_manager,
            policy_manager,
            domain_manager,
            deployment_manager,
            renderer_manager,
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
                "dashboard" => {
                    println!("Launching dashboard...");
                    let id = crate::dashboard::launch_dashboard(&self.renderer_manager).await?;
                    println!("Dashboard launched with ID: {}", id);
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
        self.dialog_manager.handle_command(command).await
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
    
    /// Handle render commands
    pub async fn handle_render_command(&mut self, command: RenderCommands) -> Result<()> {
        use crate::render_commands::{RenderCommands, DemoType};
        use crate::renderer::{RenderData, GraphNode, GraphEdge};
        
        match command {
            RenderCommands::Graph { title, file, iced } => {
                // Load graph data from file or create demo data
                let (nodes, edges) = if let Some(file_path) = file {
                    // Load from file
                    let content = std::fs::read_to_string(file_path)?;
                    let data: serde_json::Value = serde_json::from_str(&content)?;
                    // Parse nodes and edges from JSON
                    (vec![], vec![]) // TODO: Implement proper parsing
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
            RenderCommands::Edit { file, language } => {
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
                        println!("  {} - {} ({})", &id[..8], title, 
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
                    _ => {
                        println!("Demo type not yet implemented");
                    }
                }
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
                // Launch dialog UI window
                let dialog_id = if args.len() > 1 { 
                    args[1].to_string() 
                } else { 
                    uuid::Uuid::new_v4().to_string() 
                };
                
                let ai_model = self.config.general.default_ai_model.as_deref()
                    .unwrap_or("gpt-4");
                
                println!("Launching dialog UI for {} with model {}...", dialog_id, ai_model);
                
                let messages = vec![
                    crate::renderer::DialogMessage {
                        role: "system".to_string(),
                        content: "You are a helpful AI assistant.".to_string(),
                        timestamp: chrono::Utc::now(),
                    },
                ];
                
                let window_id = self.renderer_manager.spawn_dialog(
                    &format!("AI Dialog - {}", dialog_id),
                    dialog_id.clone(),
                    ai_model.to_string(),
                    messages,
                    Some("You are a helpful AI assistant.".to_string()),
                ).await?;
                
                println!("Dialog UI launched with window ID: {}", window_id);
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
                        println!("  [{}] {} - {}", &id[..8], title, renderer_name);
                    }
                }
            }
            "demo" => {
                println!("Available demos: graph3d, document");
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
        println!("  {} - Launch domain dashboard", help_style.apply_to("dashboard"));
        println!("  {} - Clear screen", help_style.apply_to("clear"));
        println!("  {} - Exit shell", help_style.apply_to("exit, quit"));
    }
}