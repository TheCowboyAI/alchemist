//! Command definitions for the Alchemist shell

use clap::Subcommand;
use crate::progress::ProgressFormat;
use crate::render_commands::RenderCommands;

#[derive(Subcommand)]
pub enum Commands {
    /// Manage AI model configurations
    Ai {
        #[command(subcommand)]
        command: AiCommands,
    },
    
    /// Manage dialogs with AI
    Dialog {
        #[command(subcommand)]
        command: DialogCommands,
    },
    
    /// Manage policies and claims
    Policy {
        #[command(subcommand)]
        command: PolicyCommands,
    },
    
    /// Manage domain hierarchy
    Domain {
        #[command(subcommand)]
        command: DomainCommands,
    },
    
    /// Deploy to CIM instances
    Deploy {
        #[command(subcommand)]
        command: DeployCommands,
    },
    
    /// Show project progress
    Progress {
        /// Path to progress.json
        #[arg(short, long, default_value = "doc/progress/progress.json")]
        file: String,
        
        /// Output format
        #[arg(short, long, value_enum, default_value = "tree")]
        format: ProgressFormat,
    },
    
    /// Launch renderer windows (Bevy/Iced)
    Render {
        #[command(subcommand)]
        command: RenderCommands,
    },
}

#[derive(Subcommand)]
pub enum AiCommands {
    /// List configured AI models
    List,
    
    /// Add a new AI model
    Add {
        /// Model name
        name: String,
        
        /// Provider (openai, anthropic, ollama)
        #[arg(short, long)]
        provider: String,
        
        /// API endpoint
        #[arg(short, long)]
        endpoint: Option<String>,
    },
    
    /// Remove an AI model
    Remove {
        /// Model name
        name: String,
    },
    
    /// Test AI model connection
    Test {
        /// Model name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum DialogCommands {
    /// Start a new dialog
    New {
        /// Dialog title
        #[arg(short, long)]
        title: Option<String>,
        
        /// AI model to use
        #[arg(short, long)]
        model: Option<String>,
    },
    
    /// List recent dialogs
    List {
        /// Number of dialogs to show
        #[arg(short, long, default_value = "10")]
        count: usize,
    },
    
    /// Continue a dialog
    Continue {
        /// Dialog ID
        id: String,
    },
    
    /// Export dialog history
    Export {
        /// Dialog ID
        id: String,
        
        /// Output format (json, markdown)
        #[arg(short, long, default_value = "markdown")]
        format: String,
    },
}

#[derive(Subcommand)]
pub enum PolicyCommands {
    /// List all policies
    List {
        /// Filter by domain
        #[arg(short, long)]
        domain: Option<String>,
    },
    
    /// Show policy details
    Show {
        /// Policy ID
        id: String,
    },
    
    /// Edit a policy
    Edit {
        /// Policy ID
        id: String,
    },
    
    /// Create a new policy
    New {
        /// Policy name
        name: String,
        
        /// Domain
        #[arg(short, long)]
        domain: String,
    },
    
    /// Manage claims
    Claims {
        #[command(subcommand)]
        command: ClaimsCommands,
    },
}

#[derive(Subcommand)]
pub enum ClaimsCommands {
    /// List claims
    List,
    
    /// Add a claim
    Add {
        /// Claim name
        name: String,
        
        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },
    
    /// Remove a claim
    Remove {
        /// Claim name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum DomainCommands {
    /// List all domains
    List,
    
    /// Show domain hierarchy
    Tree {
        /// Root domain
        #[arg(short, long)]
        root: Option<String>,
    },
    
    /// Show domain details
    Show {
        /// Domain name
        name: String,
    },
    
    /// Visualize domain relationships
    Graph {
        /// Output format (dot, json, mermaid)
        #[arg(short, long, default_value = "mermaid")]
        format: String,
    },
}

#[derive(Subcommand)]
pub enum DeployCommands {
    /// List CIM deployments
    List,
    
    /// Deploy to a CIM
    Deploy {
        /// Target CIM name
        target: String,
        
        /// Domains to deploy
        #[arg(short, long)]
        domains: Vec<String>,
    },
    
    /// Show deployment status
    Status {
        /// Deployment ID
        id: String,
    },
}