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
        #[arg(short = 'p', long, default_value = "doc/progress/progress.json")]
        file: String,
        
        /// Output format
        #[arg(short = 'f', long, value_enum, default_value = "tree")]
        format: ProgressFormat,
    },
    
    /// Launch renderer windows (Bevy/Iced)
    Render {
        #[command(subcommand)]
        command: RenderCommands,
    },
    
    /// Launch the real-time domain dashboard
    Dashboard,
    
    /// Launch dashboard in-process (for development)
    DashboardLocal,
    
    /// Manage workflows
    Workflow {
        #[command(subcommand)]
        command: WorkflowCommands,
    },
    
    /// Monitor and analyze system events
    Event {
        #[command(subcommand)]
        command: EventCommands,
    },
    
    /// Manage graph operations
    Graph {
        #[command(subcommand)]
        command: GraphCommands,
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
    
    /// Generate Nix deployment configurations
    Generate {
        /// Target deployment name
        target: String,
    },
    
    /// Apply Nix deployment
    Apply {
        /// Target deployment name
        target: String,
    },
    
    /// Validate deployment configuration
    Validate {
        /// Target deployment name
        target: String,
    },
    
    /// Rollback a deployment
    Rollback {
        /// Deployment ID to rollback
        deployment_id: String,
    },
    
    /// Create deployment pipeline
    Pipeline {
        /// Pipeline name
        name: String,
        
        /// Target environments
        #[arg(short, long)]
        environments: Vec<String>,
        
        /// Enable canary deployment
        #[arg(long)]
        canary: bool,
    },
    
    /// List deployment pipelines
    Pipelines,
    
    /// Show pipeline status
    PipelineStatus {
        /// Pipeline ID
        id: String,
    },
    
    /// Process deployment approval
    Approve {
        /// Approval ID
        id: String,
        
        /// Approve or reject
        #[arg(long)]
        approve: bool,
        
        /// Comments
        #[arg(short, long)]
        comments: Option<String>,
    },
    
    /// List pending approvals
    Approvals,
}

#[derive(Subcommand)]
pub enum WorkflowCommands {
    /// Create a new workflow
    New {
        /// Workflow name
        name: String,
        
        /// Description
        #[arg(short, long)]
        description: Option<String>,
        
        /// Load from YAML file
        #[arg(short, long)]
        file: Option<String>,
    },
    
    /// List all workflows
    List,
    
    /// Show workflow details
    Show {
        /// Workflow ID
        id: String,
    },
    
    /// Run a workflow
    Run {
        /// Workflow ID
        id: String,
        
        /// Input variables as JSON
        #[arg(short, long)]
        inputs: Option<String>,
        
        /// Input file (JSON or YAML)
        #[arg(short, long)]
        file: Option<String>,
    },
    
    /// Check workflow execution status
    Status {
        /// Execution ID
        execution_id: String,
    },
    
    /// Stop a running workflow
    Stop {
        /// Execution ID
        execution_id: String,
    },
    
    /// Import workflow from file
    Import {
        /// File path (YAML or JSON)
        path: String,
    },
    
    /// Export workflow to file
    Export {
        /// Workflow ID
        id: String,
        
        /// Output file path
        #[arg(short, long)]
        output: String,
        
        /// Format (yaml or json)
        #[arg(short, long, default_value = "yaml")]
        format: String,
    },
}

#[derive(Subcommand)]
pub enum EventCommands {
    /// List recent events
    List {
        /// Number of events to show
        #[arg(short, long, default_value = "20")]
        count: usize,
        
        /// Filter by domain
        #[arg(short, long)]
        domain: Option<String>,
        
        /// Filter by event type
        #[arg(short = 't', long)]
        event_type: Option<String>,
        
        /// Minimum severity level
        #[arg(short, long)]
        severity: Option<String>,
    },
    
    /// Watch events in real-time
    Watch {
        /// Filter expression (e.g., "domain:workflow AND severity:error")
        #[arg(short, long)]
        filter: Option<String>,
        
        /// Update interval in seconds
        #[arg(short, long, default_value = "1")]
        interval: u64,
    },
    
    /// Query historical events
    Query {
        /// Query criteria as filter DSL
        criteria: String,
        
        /// Maximum number of results
        #[arg(short, long, default_value = "100")]
        limit: usize,
        
        /// Output format (table, json, yaml)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Show event statistics
    Stats {
        /// Time window (e.g., "1h", "24h", "7d")
        #[arg(short, long, default_value = "1h")]
        window: String,
        
        /// Group by field (domain, type, severity)
        #[arg(short, long)]
        group_by: Option<String>,
    },
    
    /// Export events to file
    Export {
        /// Export format (json, csv, yaml)
        format: String,
        
        /// Output file path
        #[arg(short, long)]
        output: String,
        
        /// Filter expression
        #[arg(short, long)]
        filter: Option<String>,
        
        /// Time range start (ISO 8601)
        #[arg(long)]
        start: Option<String>,
        
        /// Time range end (ISO 8601)
        #[arg(long)]
        end: Option<String>,
    },
    
    /// Manage alert rules
    Alert {
        #[command(subcommand)]
        command: AlertCommands,
    },
}

#[derive(Subcommand)]
pub enum AlertCommands {
    /// List all alert rules
    List,
    
    /// Add a new alert rule
    Add {
        /// Rule name
        name: String,
        
        /// Filter expression
        filter: String,
        
        /// Action type (log, email, webhook, command)
        #[arg(short, long)]
        action: String,
        
        /// Action target (email address, webhook URL, etc.)
        #[arg(short = 't', long)]
        target: Option<String>,
        
        /// Throttle duration in seconds
        #[arg(long)]
        throttle: Option<u64>,
    },
    
    /// Remove an alert rule
    Remove {
        /// Rule ID
        id: String,
    },
    
    /// Test an alert rule
    Test {
        /// Rule ID
        id: String,
    },
}

#[derive(Subcommand)]
pub enum GraphCommands {
    /// Load a graph from file
    Load {
        /// File path to load from
        path: String,
        
        /// Graph ID (optional, auto-generated if not provided)
        #[arg(short, long)]
        id: Option<String>,
    },
    
    /// Save a graph to file
    Save {
        /// Graph ID to save
        id: String,
        
        /// Output file path
        #[arg(short, long)]
        output: String,
        
        /// Export format (json, cytoscape, graphviz, gexf)
        #[arg(short, long, default_value = "json")]
        format: String,
    },
    
    /// List loaded graphs
    List,
    
    /// Show graph details
    Show {
        /// Graph ID
        id: String,
    },
}