//! Alchemist - CIM Control System
//!
//! This application provides:
//! - CLI shell for CIM control and AI dialog management
//! - Domain-driven workflow management
//! - Policy and deployment management

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

// Import shell modules
mod config;
mod ai;
mod dialog;
mod policy;
mod domain;
mod deployment;
mod progress;
mod shell;
mod shell_commands;
mod renderer;
mod render_commands;
mod dashboard;
mod dashboard_events;
mod rss_feed_manager;

use crate::{
    shell::AlchemistShell,
    shell_commands::Commands,
    progress::ProgressFormat,
};


#[derive(Parser)]
#[command(name = "alchemist")]
#[command(about = "Alchemist - The CIM Control System", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Run in interactive shell mode
    #[arg(short, long)]
    interactive: bool,
    
    /// Run without dashboard (CLI only)
    #[arg(long)]
    no_dashboard: bool,
    
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,
}


fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Run in CLI/shell mode
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run_cli_mode(cli))
}

async fn run_cli_mode(cli: Cli) -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("alchemist=debug".parse()?)
        )
        .init();

    // Load configuration
    let config_path = cli.config.as_deref().unwrap_or("alchemist.toml");
    let config = config::load_or_create(config_path).await?;
    
    // Create shell
    let mut shell = AlchemistShell::new(config).await?;
    
    // Handle commands
    match cli.command {
        Some(command) => {
            handle_command(&mut shell, command).await?;
        }
        None => {
            if cli.interactive {
                info!("Starting Alchemist interactive shell...");
                shell.run_interactive().await?;
            } else if !cli.no_dashboard {
                // Launch dashboard by default
                println!("🚀 Launching Alchemist Dashboard...");
                println!("Use --no-dashboard to run in CLI mode only");
                println!("Use --interactive for shell mode\n");
                
                // Launch dashboard with event sourcing
                use crate::dashboard::launch_dashboard_with_events;
                
                // Try to connect to NATS for real-time events
                let nats_client = if let Some(nats_url) = &shell.config.general.nats_url {
                    match async_nats::connect(nats_url).await {
                        Ok(client) => {
                            println!("✅ Connected to NATS at {}", nats_url);
                            println!("   Dashboard will show real-time domain events");
                            Some(client)
                        }
                        Err(e) => {
                            println!("⚠️  Could not connect to NATS: {}", e);
                            println!("   Dashboard will run in demo mode");
                            None
                        }
                    }
                } else {
                    println!("ℹ️  No NATS URL configured");
                    println!("   Dashboard will run in demo mode");
                    None
                };
                
                let dashboard_id = launch_dashboard_with_events(&shell.renderer_manager, nats_client).await?;
                println!("Dashboard launched with ID: {}", dashboard_id);
                
                // Keep the main process running
                println!("\nPress Ctrl+C to exit");
                tokio::signal::ctrl_c().await?;
                
                // Clean up
                shell.renderer_manager.close(&dashboard_id).await?;
            } else {
                // Show status/help
                println!("Alchemist - The CIM Control System");
                println!("\nUse --help for usage information");
                println!("Use --interactive or -i to start interactive shell");
                
                // Show quick status
                shell.show_status().await?;
            }
        }
    }
    
    Ok(())
}

async fn handle_command(shell: &mut AlchemistShell, command: Commands) -> Result<()> {
    match command {
        Commands::Ai { command } => {
            shell.handle_ai_command(command).await?;
        }
        Commands::Dialog { command } => {
            shell.handle_dialog_command(command).await?;
        }
        Commands::Policy { command } => {
            shell.handle_policy_command(command).await?;
        }
        Commands::Domain { command } => {
            shell.handle_domain_command(command).await?;
        }
        Commands::Deploy { command } => {
            shell.handle_deploy_command(command).await?;
        }
        Commands::Progress { file, format } => {
            shell.show_progress(&file, format).await?;
        }
        Commands::Render { command } => {
            shell.handle_render_command(command).await?;
        }
    }
    
    Ok(())
}

