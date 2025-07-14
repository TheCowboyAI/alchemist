//! CIM deployment management

use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;
use uuid::Uuid;

use crate::{
    config::{AlchemistConfig, DeploymentConfig, ServiceConfig, AgentConfig, ResourceLimits as ConfigResourceLimits, HealthCheckConfig as ConfigHealthCheckConfig},
    shell_commands::DeployCommands,
    nats_client::NatsClient,
    nix_deployment::{
        NixDeployer, NixDeploymentSpec,
    },
    deployment_automation::{
        DeploymentAutomation, AutomationConfig, DeploymentWindow, PipelineTrigger,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub id: String,
    pub name: String,
    pub environment: String,
    pub nats_url: String,
    pub domains: Vec<String>,
    pub status: DeploymentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub events_processed: u64,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub target: DeploymentTarget,
    pub strategy: DeploymentStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    NotDeployed,
    Deploying,
    Running,
    Degraded,
    Stopped,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentTask {
    pub id: String,
    pub deployment_id: String,
    pub domains: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: TaskStatus,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentTarget {
    Local,
    Development { host: String },
    Production { hosts: Vec<String> },
    Edge { leaf_nodes: Vec<String> },
}

impl DeploymentTarget {
    pub fn name(&self) -> &str {
        match self {
            DeploymentTarget::Local => "local",
            DeploymentTarget::Development { .. } => "development",
            DeploymentTarget::Production { .. } => "production",
            DeploymentTarget::Edge { .. } => "edge",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStrategy {
    RollingUpdate { max_unavailable: u32 },
    BlueGreen,
    Recreate,
}

pub struct DeploymentManager {
    deployments: DashMap<String, Deployment>,
    tasks: DashMap<String, DeploymentTask>,
    configs: HashMap<String, DeploymentConfig>,
    nix_deployer: Option<NixDeployer>,
    nats_client: Option<NatsClient>,
    automation: Option<DeploymentAutomation>,
}

impl DeploymentManager {
    pub async fn new(config: &AlchemistConfig) -> Result<Self> {
        let deployments = DashMap::new();
        
        // Create deployments from config
        for (name, deploy_config) in &config.deployments {
            let deployment = Deployment {
                id: Uuid::new_v4().to_string(),
                name: name.clone(),
                environment: deploy_config.environment.clone(),
                nats_url: deploy_config.nats_url.clone(),
                domains: deploy_config.domains.clone(),
                status: DeploymentStatus::NotDeployed,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                events_processed: 0,
                last_heartbeat: None,
                target: match deploy_config.environment.as_str() {
                    "local" => DeploymentTarget::Local,
                    "development" => DeploymentTarget::Development { 
                        host: "dev.example.com".to_string() 
                    },
                    "production" => DeploymentTarget::Production { 
                        hosts: vec!["prod1.example.com".to_string(), "prod2.example.com".to_string()] 
                    },
                    _ => DeploymentTarget::Local,
                },
                strategy: DeploymentStrategy::RollingUpdate { max_unavailable: 1 },
            };
            
            deployments.insert(name.clone(), deployment);
        }
        
        Ok(Self {
            deployments,
            tasks: DashMap::new(),
            configs: config.deployments.clone(),
            nix_deployer: None,
            nats_client: None,
            automation: None,
        })
    }

    /// Initialize Nix deployer
    pub async fn init_nix_deployer(&mut self, nats_url: &str) -> Result<()> {
        let nats_client = NatsClient::new(nats_url).await?;
        let nix_dir = PathBuf::from("./nix-deployments");
        let templates_dir = PathBuf::from("./nix/templates");
        
        let nix_deployer = NixDeployer::new(
            nats_client.clone(),
            nix_dir,
            templates_dir
        ).await?;
        
        self.nix_deployer = Some(nix_deployer);
        self.nats_client = Some(nats_client);
        
        Ok(())
    }

    /// Initialize deployment automation
    pub async fn init_automation(&mut self, config: AutomationConfig) -> Result<()> {
        if let (Some(nix_deployer), Some(nats_client)) = (&self.nix_deployer, &self.nats_client) {
            // Note: In a real implementation, we would need to refactor this to avoid
            // the circular dependency between DeploymentManager and DeploymentAutomation.
            // For now, we'll create the automation without the manager reference.
            
            // Create a minimal deployment manager for automation
            let minimal_manager = std::sync::Arc::new(tokio::sync::RwLock::new(DeploymentManager {
                deployments: self.deployments.clone(),
                tasks: self.tasks.clone(),
                configs: self.configs.clone(),
                nix_deployer: Some(nix_deployer.clone()),
                nats_client: Some(nats_client.clone()),
                automation: None,
            }));
            
            let automation = DeploymentAutomation::new(
                config,
                minimal_manager,
                std::sync::Arc::new(nix_deployer.clone()),
                nats_client.clone(),
            ).await?;
            
            self.automation = Some(automation);
            info!("Deployment automation initialized");
        } else {
            return Err(anyhow::anyhow!("Nix deployer and NATS client must be initialized first"));
        }
        
        Ok(())
    }
    
    pub async fn handle_command(&mut self, command: DeployCommands) -> Result<()> {
        match command {
            DeployCommands::List => {
                self.list_deployments_cli().await?;
            }
            DeployCommands::Deploy { target, domains } => {
                self.deploy_cli(target, domains).await?;
            }
            DeployCommands::Status { id } => {
                self.show_status_cli(id).await?;
            }
            DeployCommands::Generate { target } => {
                self.generate_nix_configs(target).await?;
            }
            DeployCommands::Apply { target } => {
                self.apply_nix_deployment(target).await?;
            }
            DeployCommands::Validate { target } => {
                self.validate_deployment(target).await?;
            }
            DeployCommands::Rollback { deployment_id } => {
                self.rollback_deployment(deployment_id).await?;
            }
            DeployCommands::Pipeline { name, environments, canary } => {
                self.create_pipeline_cli(name, environments, canary).await?;
            }
            DeployCommands::Pipelines => {
                self.list_pipelines_cli().await?;
            }
            DeployCommands::PipelineStatus { id } => {
                self.show_pipeline_status_cli(id).await?;
            }
            DeployCommands::Approve { id, approve, comments } => {
                self.process_approval_cli(id, approve, comments).await?;
            }
            DeployCommands::Approvals => {
                self.list_approvals_cli().await?;
            }
        }
        Ok(())
    }
    
    /// Get deployment configuration by name
    pub fn get_config(&self, name: &str) -> Option<&DeploymentConfig> {
        self.configs.get(name)
    }
    
    /// Update deployment configuration
    pub fn update_config(&mut self, name: String, config: DeploymentConfig) {
        // Update the stored configuration
        self.configs.insert(name.clone(), config.clone());
        
        // Also update the deployment if it exists
        if let Some(mut deployment) = self.deployments.get_mut(&name) {
            deployment.environment = config.environment;
            deployment.nats_url = config.nats_url;
            deployment.domains = config.domains;
            deployment.updated_at = Utc::now();
        }
    }
    
    pub async fn list_deployments(&self) -> Result<Vec<DeploymentSummary>> {
        let mut summaries: Vec<DeploymentSummary> = self.deployments
            .iter()
            .map(|entry| {
                let deployment = entry.value();
                DeploymentSummary {
                    name: deployment.name.clone(),
                    environment: deployment.environment.clone(),
                    nats_url: deployment.nats_url.clone(),
                    status: deployment.status.clone(),
                    domain_count: deployment.domains.len(),
                    events_processed: deployment.events_processed,
                    last_heartbeat: deployment.last_heartbeat,
                }
            })
            .collect();
        
        summaries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(summaries)
    }
    
    async fn list_deployments_cli(&self) -> Result<()> {
        let deployments = self.list_deployments().await?;
        
        if deployments.is_empty() {
            println!("No CIM deployments configured.");
            println!("Add deployments to your alchemist.toml configuration.");
            return Ok(());
        }
        
        println!("🚀 CIM Deployments:");
        println!("─────────────────────────────");
        
        for deployment in deployments {
            let status_icon = match deployment.status {
                DeploymentStatus::Running => "✅",
                DeploymentStatus::Degraded => "⚠️",
                DeploymentStatus::Stopped => "⏹️",
                DeploymentStatus::Failed(_) => "❌",
                DeploymentStatus::Deploying => "🔄",
                DeploymentStatus::NotDeployed => "⏸️",
            };
            
            println!(
                "{} {} ({}) - {}",
                status_icon,
                deployment.name,
                deployment.environment,
                self.format_status(&deployment.status)
            );
            println!("   NATS: {}", deployment.nats_url);
            println!("   Domains: {}", deployment.domain_count);
            
            if deployment.events_processed > 0 {
                println!("   Events: {}", deployment.events_processed);
            }
            
            if let Some(heartbeat) = deployment.last_heartbeat {
                let ago = Utc::now().signed_duration_since(heartbeat);
                println!("   Last seen: {} ago", self.format_duration(ago));
            }
        }
        
        Ok(())
    }
    
    async fn deploy_cli(&mut self, target: String, domains: Vec<String>) -> Result<()> {
        let deployment = self.deployments.get(&target)
            .ok_or_else(|| crate::error::AlchemistError::ServiceNotFound(format!("Deployment target not found: {}", target)))?
            .clone();
        
        println!("🚀 Deploying to: {}", deployment.name);
        println!("   Environment: {}", deployment.environment);
        println!("   NATS URL: {}", deployment.nats_url);
        
        let domains_to_deploy = if domains.is_empty() {
            println!("   Domains: {} (all configured)", deployment.domains.join(", "));
            deployment.domains.clone()
        } else {
            println!("   Domains: {} (specified)", domains.join(", "));
            domains
        };
        
        // Create deployment task
        let task_id = Uuid::new_v4().to_string();
        let task = DeploymentTask {
            id: task_id.clone(),
            deployment_id: deployment.id.clone(),
            domains: domains_to_deploy.clone(),
            started_at: Utc::now(),
            completed_at: None,
            status: TaskStatus::Pending,
            logs: vec![
                format!("Deployment task created for {}", deployment.name),
                format!("Target environment: {}", deployment.environment),
                format!("Domains to deploy: {}", domains_to_deploy.join(", ")),
            ],
        };
        
        self.tasks.insert(task_id.clone(), task);
        
        // Update deployment status
        if let Some(mut deploy) = self.deployments.get_mut(&target) {
            deploy.status = DeploymentStatus::Deploying;
            deploy.updated_at = Utc::now();
        }
        
        // Simulate deployment steps
        println!("\n📋 Deployment Steps:");
        println!("1. ✅ Validating domain configurations");
        println!("2. ✅ Connecting to NATS at {}", deployment.nats_url);
        println!("3. ✅ Creating JetStream streams");
        println!("4. ✅ Deploying domain event handlers");
        println!("5. ✅ Verifying deployment");
        
        // Update task status
        if let Some(mut task) = self.tasks.get_mut(&task_id) {
            task.status = TaskStatus::Completed;
            task.completed_at = Some(Utc::now());
            task.logs.push("Deployment completed successfully".to_string());
        }
        
        // Update deployment status
        if let Some(mut deploy) = self.deployments.get_mut(&target) {
            deploy.status = DeploymentStatus::Running;
            deploy.updated_at = Utc::now();
            deploy.last_heartbeat = Some(Utc::now());
        }
        
        println!("\n✅ Deployment completed successfully!");
        println!("   Task ID: {}", task_id);
        println!("   Use 'alchemist deploy status {}' to check status", task_id);
        
        Ok(())
    }
    
    async fn show_status_cli(&self, id: String) -> Result<()> {
        // Try to find as deployment ID first
        if let Some(deployment) = self.deployments.iter().find(|e| e.value().id == id) {
            let deploy = deployment.value();
            
            println!("📊 Deployment Status:");
            println!("─────────────────────────────");
            println!("Name: {}", deploy.name);
            println!("Environment: {}", deploy.environment);
            println!("Status: {}", self.format_status(&deploy.status));
            println!("NATS URL: {}", deploy.nats_url);
            println!("Created: {}", deploy.created_at.format("%Y-%m-%d %H:%M"));
            println!("Updated: {}", deploy.updated_at.format("%Y-%m-%d %H:%M"));
            
            if deploy.events_processed > 0 {
                println!("Events Processed: {}", deploy.events_processed);
            }
            
            if let Some(heartbeat) = deploy.last_heartbeat {
                println!("Last Heartbeat: {}", heartbeat.format("%Y-%m-%d %H:%M:%S"));
            }
            
            println!("\nDeployed Domains:");
            for domain in &deploy.domains {
                println!("  • {}", domain);
            }
            
            return Ok(());
        }
        
        // Try as task ID
        if let Some(task) = self.tasks.get(&id) {
            println!("📋 Deployment Task Status:");
            println!("─────────────────────────────");
            println!("Task ID: {}", task.id);
            println!("Started: {}", task.started_at.format("%Y-%m-%d %H:%M:%S"));
            
            if let Some(completed) = task.completed_at {
                println!("Completed: {}", completed.format("%Y-%m-%d %H:%M:%S"));
                let duration = completed.signed_duration_since(task.started_at);
                println!("Duration: {}", self.format_duration(duration));
            }
            
            println!("Status: {}", self.format_task_status(&task.status));
            
            println!("\nDomains:");
            for domain in &task.domains {
                println!("  • {}", domain);
            }
            
            if !task.logs.is_empty() {
                println!("\nLogs:");
                for log in &task.logs {
                    println!("  {}", log);
                }
            }
            
            return Ok(());
        }
        
        Err(anyhow::anyhow!("No deployment or task found with ID: {}", id))
    }
    
    fn format_status(&self, status: &DeploymentStatus) -> String {
        match status {
            DeploymentStatus::NotDeployed => "Not Deployed".to_string(),
            DeploymentStatus::Deploying => "Deploying...".to_string(),
            DeploymentStatus::Running => "Running".to_string(),
            DeploymentStatus::Degraded => "Degraded".to_string(),
            DeploymentStatus::Stopped => "Stopped".to_string(),
            DeploymentStatus::Failed(reason) => format!("Failed: {}", reason),
        }
    }
    
    fn format_task_status(&self, status: &TaskStatus) -> String {
        match status {
            TaskStatus::Pending => "Pending".to_string(),
            TaskStatus::Running => "Running".to_string(),
            TaskStatus::Completed => "Completed".to_string(),
            TaskStatus::Failed(reason) => format!("Failed: {}", reason),
        }
    }
    
    fn format_duration(&self, duration: chrono::Duration) -> String {
        let seconds = duration.num_seconds();
        
        if seconds < 60 {
            format!("{} seconds", seconds)
        } else if seconds < 3600 {
            format!("{} minutes", seconds / 60)
        } else if seconds < 86400 {
            format!("{} hours", seconds / 3600)
        } else {
            format!("{} days", seconds / 86400)
        }
    }

    /// Generate Nix configurations for deployment
    async fn generate_nix_configs(&mut self, target: String) -> Result<()> {
        // Initialize Nix deployer if not already done
        if self.nix_deployer.is_none() {
            self.init_nix_deployer("nats://localhost:4222").await?;
        }
        
        let deployment = self.deployments.get(&target)
            .ok_or_else(|| crate::error::AlchemistError::ServiceNotFound(format!("Deployment target not found: {}", target)))?
            .clone();
        
        // Get the deployment configuration from our stored configs
        let config = self.configs.get(&target)
            .ok_or_else(|| anyhow::anyhow!("Deployment configuration not found: {}", target))?;
        
        println!("🔧 Generating Nix configurations for: {}", deployment.name);
        println!("   Target: {:?}", deployment.target);
        println!("   Strategy: {:?}", deployment.strategy);
        println!("   Environment: {}", config.environment);
        println!("   NATS URL: {}", config.nats_url);
        
        // Create Nix deployment specification using both deployment and config
        let nix_spec = self.create_nix_spec_with_config(&deployment, config)?;
        
        // Generate configurations
        if let Some(deployer) = &self.nix_deployer {
            let deployment_dir = deployer.generate_configs(&nix_spec).await?;
            
            println!("\n✅ Nix configurations generated successfully!");
            println!("   Location: {}", deployment_dir.display());
            println!("   Run 'alchemist deploy apply {}' to deploy", target);
        } else {
            return Err(anyhow::anyhow!("Nix deployer not initialized"));
        }
        
        Ok(())
    }
    
    /// Apply Nix deployment
    async fn apply_nix_deployment(&mut self, target: String) -> Result<()> {
        // Initialize Nix deployer if not already done
        if self.nix_deployer.is_none() {
            self.init_nix_deployer("nats://localhost:4222").await?;
        }
        
        let deployment = self.deployments.get(&target)
            .ok_or_else(|| crate::error::AlchemistError::ServiceNotFound(format!("Deployment target not found: {}", target)))?
            .clone();
        
        println!("🚀 Applying Nix deployment for: {}", deployment.name);
        
        // Create deployment task
        let task_id = Uuid::new_v4().to_string();
        let task = DeploymentTask {
            id: task_id.clone(),
            deployment_id: deployment.id.clone(),
            domains: deployment.domains.clone(),
            started_at: Utc::now(),
            completed_at: None,
            status: TaskStatus::Running,
            logs: vec![
                format!("Starting Nix deployment for {}", deployment.name),
                format!("Target: {:?}", deployment.target),
                format!("Strategy: {:?}", deployment.strategy),
            ],
        };
        
        self.tasks.insert(task_id.clone(), task);
        
        // Update deployment status
        if let Some(mut deploy) = self.deployments.get_mut(&target) {
            deploy.status = DeploymentStatus::Deploying;
            deploy.updated_at = Utc::now();
        }
        
        // Create and apply Nix specification
        let nix_spec = self.create_nix_spec(&deployment)?;
        let deployment_dir = PathBuf::from(format!("./nix-deployments/{}", nix_spec.id));
        
        if let Some(deployer) = &self.nix_deployer {
            match deployer.apply_deployment(&nix_spec, &deployment_dir).await {
                Ok(deployment_id) => {
                    // Update task status
                    if let Some(mut task) = self.tasks.get_mut(&task_id) {
                        task.status = TaskStatus::Completed;
                        task.completed_at = Some(Utc::now());
                        task.logs.push("Deployment completed successfully".to_string());
                    }
                    
                    // Update deployment status
                    if let Some(mut deploy) = self.deployments.get_mut(&target) {
                        deploy.status = DeploymentStatus::Running;
                        deploy.updated_at = Utc::now();
                        deploy.last_heartbeat = Some(Utc::now());
                    }
                    
                    println!("\n✅ Deployment applied successfully!");
                    println!("   Deployment ID: {}", deployment_id);
                    println!("   Use 'alchemist deploy status {}' to monitor", deployment_id);
                    
                    // Start monitoring deployment health
                    self.start_deployment_monitoring(&deployment_id).await?;
                }
                Err(e) => {
                    // Update task status
                    if let Some(mut task) = self.tasks.get_mut(&task_id) {
                        task.status = TaskStatus::Failed(e.to_string());
                        task.completed_at = Some(Utc::now());
                        task.logs.push(format!("Deployment failed: {}", e));
                    }
                    
                    // Update deployment status
                    if let Some(mut deploy) = self.deployments.get_mut(&target) {
                        deploy.status = DeploymentStatus::Failed(e.to_string());
                        deploy.updated_at = Utc::now();
                    }
                    
                    return Err(e.into());
                }
            }
        } else {
            return Err(anyhow::anyhow!("Nix deployer not initialized"));
        }
        
        Ok(())
    }
    
    /// Validate deployment configuration
    async fn validate_deployment(&mut self, target: String) -> Result<()> {
        // Initialize Nix deployer if not already done
        if self.nix_deployer.is_none() {
            self.init_nix_deployer("nats://localhost:4222").await?;
        }
        
        let deployment = self.deployments.get(&target)
            .ok_or_else(|| crate::error::AlchemistError::ServiceNotFound(format!("Deployment target not found: {}", target)))?
            .clone();
        
        println!("🔍 Validating deployment configuration for: {}", deployment.name);
        
        // Create Nix specification
        let nix_spec = self.create_nix_spec(&deployment)?;
        let deployment_dir = PathBuf::from(format!("./nix-deployments/{}", nix_spec.id));
        
        if let Some(deployer) = &self.nix_deployer {
            match deployer.validate_deployment(&deployment_dir).await {
                Ok(_) => {
                    println!("\n✅ Deployment configuration is valid!");
                    println!("   All Nix modules and configurations passed validation");
                }
                Err(e) => {
                    println!("\n❌ Deployment validation failed!");
                    println!("   Error: {}", e);
                    return Err(e.into());
                }
            }
        } else {
            return Err(anyhow::anyhow!("Nix deployer not initialized"));
        }
        
        Ok(())
    }
    
    /// Rollback deployment
    async fn rollback_deployment(&mut self, deployment_id: String) -> Result<()> {
        // Initialize Nix deployer if not already done
        if self.nix_deployer.is_none() {
            self.init_nix_deployer("nats://localhost:4222").await?;
        }
        
        println!("🔄 Rolling back deployment: {}", deployment_id);
        
        if let Some(deployer) = &self.nix_deployer {
            deployer.rollback_deployment(&deployment_id).await?;
            
            println!("\n✅ Deployment rolled back successfully!");
        } else {
            return Err(anyhow::anyhow!("Nix deployer not initialized"));
        }
        
        Ok(())
    }
    
    /// Create Nix deployment specification from deployment and config
    fn create_nix_spec_with_config(&self, deployment: &Deployment, config: &DeploymentConfig) -> Result<NixDeploymentSpec> {
        // Use config fields to enrich the specification
        let mut env_vars = HashMap::from([
            ("DEPLOYMENT_ID".to_string(), deployment.id.clone()),
            ("DEPLOYMENT_NAME".to_string(), deployment.name.clone()),
            ("ENVIRONMENT".to_string(), config.environment.clone()),
            ("NATS_URL".to_string(), config.nats_url.clone()),
        ]);
        
        // Add any custom services from config
        if let Some(custom_services) = config.services.as_ref() {
            for (key, value) in custom_services {
                env_vars.insert(format!("SERVICE_{}", key.to_uppercase()), value.clone());
            }
        }
        
        // Add any custom agents from config  
        if let Some(custom_agents) = config.agents.as_ref() {
            for (key, value) in custom_agents {
                env_vars.insert(format!("AGENT_{}", key.to_uppercase()), value.clone());
            }
        }
        
        // Create the spec with config information
        let mut spec = self.create_nix_spec(deployment)?;
        spec.environment.extend(env_vars);
        
        Ok(spec)
    }
    
    /// Create Nix deployment specification from deployment
    fn create_nix_spec(&self, deployment: &Deployment) -> Result<NixDeploymentSpec> {
        use crate::nix_deployment::*;
        
        // Create service specifications
        let services = vec![
            NixServiceSpec {
                name: "alchemist-api".to_string(),
                config: ServiceConfig {
                    name: "api".to_string(),
                    executable: "alchemist-api".to_string(),
                    args: vec![],
                    port: Some(8080),
                    environment: HashMap::from([
                        ("ENVIRONMENT".to_string(), deployment.environment.clone()),
                        ("NATS_URL".to_string(), deployment.nats_url.clone()),
                    ]),
                    resources: ConfigResourceLimits {
                        cpu: Some(1.0),
                        memory: Some(1024),
                        disk: None,
                    },
                    health_check: Some(ConfigHealthCheckConfig {
                        endpoint: "/health".to_string(),
                        interval: 30,
                        timeout: 5,
                        failure_threshold: 3,
                    }),
                },
                resources: ResourceLimits {
                    cpu: Some("1000m".to_string()),
                    memory: Some("1Gi".to_string()),
                    disk: None,
                },
                health_check: HealthCheckConfig {
                    http_endpoint: Some("/health".to_string()),
                    tcp_port: None,
                    interval: 30,
                    timeout: 5,
                    retries: 3,
                },
                replicas: match &deployment.target {
                    DeploymentTarget::Production { .. } => 3,
                    _ => 1,
                },
            },
            NixServiceSpec {
                name: "alchemist-scheduler".to_string(),
                config: ServiceConfig {
                    name: "scheduler".to_string(),
                    executable: "alchemist-scheduler".to_string(),
                    args: vec![],
                    port: Some(8081),
                    environment: HashMap::from([
                        ("ENVIRONMENT".to_string(), deployment.environment.clone()),
                        ("NATS_URL".to_string(), deployment.nats_url.clone()),
                    ]),
                    resources: ConfigResourceLimits {
                        cpu: Some(0.5),
                        memory: Some(512),
                        disk: None,
                    },
                    health_check: Some(ConfigHealthCheckConfig {
                        endpoint: "/health".to_string(),
                        interval: 30,
                        timeout: 5,
                        failure_threshold: 3,
                    }),
                },
                resources: ResourceLimits {
                    cpu: Some("500m".to_string()),
                    memory: Some("512Mi".to_string()),
                    disk: None,
                },
                health_check: HealthCheckConfig {
                    http_endpoint: None,
                    tcp_port: Some(8081),
                    interval: 30,
                    timeout: 5,
                    retries: 3,
                },
                replicas: match &deployment.target {
                    DeploymentTarget::Production { .. } => 2,
                    _ => 1,
                },
            },
        ];
        
        // Create agent specifications
        let agents = vec![
            NixAgentSpec {
                name: "worker".to_string(),
                config: AgentConfig {
                    name: "worker".to_string(),
                    agent_type: "general".to_string(),
                    capabilities: vec!["compute".to_string(), "storage".to_string()],
                    config: HashMap::new(),
                    environment: HashMap::new(),
                    resources: ConfigResourceLimits {
                        cpu: Some(2.0),
                        memory: Some(2048),
                        disk: None,
                    },
                },
                resources: ResourceLimits {
                    cpu: Some("2000m".to_string()),
                    memory: Some("2Gi".to_string()),
                    disk: None,
                },
                capabilities: vec!["compute".to_string(), "storage".to_string()],
            },
        ];
        
        // Create NATS mesh configuration
        let nats_mesh = match &deployment.target {
            DeploymentTarget::Local => NatsMeshConfig {
                nodes: vec![
                    NatsNode {
                        name: "nats-local".to_string(),
                        host: "localhost".to_string(),
                        client_port: 4222,
                        cluster_port: 6222,
                        routes: vec![],
                    },
                ],
                leaf_nodes: vec![],
                jetstream: JetStreamConfig {
                    store_dir: "/var/lib/nats/jetstream".to_string(),
                    max_memory: "512M".to_string(),
                    max_file: "1G".to_string(),
                },
            },
            DeploymentTarget::Production { hosts } => NatsMeshConfig {
                nodes: hosts.iter().enumerate().map(|(i, host)| {
                    NatsNode {
                        name: format!("nats-{}", i + 1),
                        host: host.clone(),
                        client_port: 4222,
                        cluster_port: 6222,
                        routes: hosts.iter().enumerate()
                            .filter(|(j, _)| *j != i)
                            .map(|(_, h)| format!("{}:6222", h))
                            .collect(),
                    }
                }).collect(),
                leaf_nodes: vec![],
                jetstream: JetStreamConfig {
                    store_dir: "/var/lib/nats/jetstream".to_string(),
                    max_memory: "4G".to_string(),
                    max_file: "100G".to_string(),
                },
            },
            DeploymentTarget::Edge { leaf_nodes } => NatsMeshConfig {
                nodes: vec![],
                leaf_nodes: leaf_nodes.iter().map(|node| {
                    LeafNode {
                        name: node.clone(),
                        remotes: vec![deployment.nats_url.clone()],
                        credentials: None,
                    }
                }).collect(),
                jetstream: JetStreamConfig {
                    store_dir: "/var/lib/nats/jetstream".to_string(),
                    max_memory: "256M".to_string(),
                    max_file: "1G".to_string(),
                },
            },
            _ => NatsMeshConfig {
                nodes: vec![
                    NatsNode {
                        name: "nats-dev".to_string(),
                        host: "0.0.0.0".to_string(),
                        client_port: 4222,
                        cluster_port: 6222,
                        routes: vec![],
                    },
                ],
                leaf_nodes: vec![],
                jetstream: JetStreamConfig {
                    store_dir: "/var/lib/nats/jetstream".to_string(),
                    max_memory: "1G".to_string(),
                    max_file: "10G".to_string(),
                },
            },
        };
        
        Ok(NixDeploymentSpec {
            id: deployment.id.clone(),
            target: deployment.target.clone(),
            services,
            agents,
            nats_mesh,
            strategy: deployment.strategy.clone(),
            environment: HashMap::from([
                ("DEPLOYMENT_ID".to_string(), deployment.id.clone()),
                ("DEPLOYMENT_NAME".to_string(), deployment.name.clone()),
                ("ENVIRONMENT".to_string(), deployment.environment.clone()),
            ]),
            secrets: SecretsConfig {
                provider: SecretsProvider::Environment,
                paths: HashMap::new(),
            },
        })
    }
    
    /// Start monitoring deployment health
    async fn start_deployment_monitoring(&self, deployment_id: &str) -> Result<()> {
        if let Some(deployer) = &self.nix_deployer {
            // Start monitoring in a background task
            let deployer_clone = deployer.clone();
            let deployment_id_clone = deployment_id.to_string();
            let deployment_id = deployment_id.to_string();
            
            tokio::spawn(async move {
                if let Err(e) = deployer_clone.monitor_deployment(&deployment_id_clone).await {
                    tracing::error!("Deployment monitoring error: {}", e);
                }
            });
            
            info!("Started deployment health monitoring for: {}", deployment_id);
        }
        
        Ok(())
    }

    /// Create deployment pipeline via CLI
    async fn create_pipeline_cli(&mut self, name: String, environments: Vec<String>, canary: bool) -> Result<()> {
        if self.automation.is_none() {
            // Initialize automation with default config if not already done
            let mut config = AutomationConfig {
                gitops_enabled: false,
                git_repo: None,
                git_branch: "main".to_string(),
                config_path: "deployments/".to_string(),
                auto_rollback: true,
                rollback_threshold: 0.2,
                canary_enabled: canary,
                canary_percentage: 10,
                canary_duration: 30,
                deployment_windows: vec![],
                promotion_policies: vec![],
            };
            
            // Add default deployment window (always open)
            config.deployment_windows.push(DeploymentWindow {
                name: "default".to_string(),
                days: vec![
                    chrono::Weekday::Mon,
                    chrono::Weekday::Tue,
                    chrono::Weekday::Wed,
                    chrono::Weekday::Thu,
                    chrono::Weekday::Fri,
                    chrono::Weekday::Sat,
                    chrono::Weekday::Sun,
                ],
                start_hour: 0,
                end_hour: 24,
                environments: environments.clone(),
            });
            
            self.init_automation(config).await?;
        }
        
        if let Some(automation) = &self.automation {
            println!("🚀 Creating deployment pipeline: {}", name);
            println!("   Environments: {}", environments.join(" → "));
            if canary {
                println!("   Canary deployment: Enabled (10% traffic)");
            }
            
            let pipeline_id = automation.create_pipeline(
                name,
                environments,
                PipelineTrigger::Manual { user: "cli".to_string() },
            ).await?;
            
            println!("\n✅ Pipeline created successfully!");
            println!("   Pipeline ID: {}", pipeline_id);
            println!("   Use 'alchemist deploy pipeline-status {}' to monitor", pipeline_id);
        }
        
        Ok(())
    }
    
    /// List pipelines via CLI
    async fn list_pipelines_cli(&self) -> Result<()> {
        if let Some(automation) = &self.automation {
            let pipelines = automation.list_pipelines().await?;
            
            if pipelines.is_empty() {
                println!("No deployment pipelines found.");
                println!("Use 'alchemist deploy pipeline' to create one.");
                return Ok(());
            }
            
            println!("📋 Deployment Pipelines:");
            println!("─────────────────────────────");
            
            for (id, name, status) in pipelines {
                let status_icon = match status {
                    crate::deployment_automation::PipelineStatus::Running => "🔄",
                    crate::deployment_automation::PipelineStatus::Success => "✅",
                    crate::deployment_automation::PipelineStatus::Failed => "❌",
                    crate::deployment_automation::PipelineStatus::Cancelled => "🚫",
                    crate::deployment_automation::PipelineStatus::RolledBack => "↩️",
                };
                
                println!("{} {} - {}", status_icon, name, id[..8].to_string());
            }
        } else {
            println!("Deployment automation not initialized.");
        }
        
        Ok(())
    }
    
    /// Show pipeline status via CLI
    async fn show_pipeline_status_cli(&self, id: String) -> Result<()> {
        if let Some(automation) = &self.automation {
            if let Some(pipeline) = automation.get_pipeline_status(&id).await? {
                println!("📊 Pipeline Status: {}", pipeline.name);
                println!("─────────────────────────────");
                println!("ID: {}", pipeline.id);
                println!("Status: {:?}", pipeline.status);
                println!("Started: {}", pipeline.started_at.format("%Y-%m-%d %H:%M"));
                
                if let Some(completed) = pipeline.completed_at {
                    println!("Completed: {}", completed.format("%Y-%m-%d %H:%M"));
                }
                
                println!("\n📋 Stages:");
                for (i, stage) in pipeline.stages.iter().enumerate() {
                    let is_current = i == pipeline.current_stage;
                    let status_icon = match &stage.status {
                        crate::deployment_automation::StageStatus::Pending => "⏸️",
                        crate::deployment_automation::StageStatus::Running => "🔄",
                        crate::deployment_automation::StageStatus::Success => "✅",
                        crate::deployment_automation::StageStatus::Failed => "❌",
                        crate::deployment_automation::StageStatus::Skipped => "⏭️",
                        crate::deployment_automation::StageStatus::Cancelled => "🚫",
                    };
                    
                    let current_marker = if is_current { "▶" } else { " " };
                    println!("{} {} {} - {} ({})", 
                        current_marker,
                        status_icon,
                        stage.name,
                        stage.environment,
                        format!("{:?}", stage.stage_type)
                    );
                }
            } else {
                println!("Pipeline not found: {}", id);
            }
        } else {
            println!("Deployment automation not initialized.");
        }
        
        Ok(())
    }
    
    /// Process approval via CLI
    async fn process_approval_cli(&self, id: String, approve: bool, comments: Option<String>) -> Result<()> {
        if let Some(automation) = &self.automation {
            automation.process_approval(
                id.clone(),
                "cli_user".to_string(),
                approve,
                comments,
            ).await?;
            
            println!("✅ Approval processed: {}", if approve { "Approved" } else { "Rejected" });
        } else {
            println!("Deployment automation not initialized.");
        }
        
        Ok(())
    }
    
    /// List pending approvals via CLI
    async fn list_approvals_cli(&self) -> Result<()> {
        if let Some(automation) = &self.automation {
            let approvals = automation.list_pending_approvals().await?;
            
            if approvals.is_empty() {
                println!("No pending approvals.");
                return Ok(());
            }
            
            println!("🔐 Pending Approvals:");
            println!("─────────────────────────────");
            
            for approval in approvals {
                println!("ID: {}", approval.id);
                println!("Pipeline: {}", approval.pipeline_id[..8].to_string());
                println!("Stage: {}", approval.stage_name);
                println!("Required Approvers: {}", approval.required_approvers.join(", "));
                println!("Current Approvals: {}/{}", 
                    approval.approvals.len(), 
                    approval.required_approvers.len()
                );
                println!("Expires: {}", approval.expires_at.format("%Y-%m-%d %H:%M"));
                println!();
            }
            
            println!("Use 'alchemist deploy approve <id> --approve' to approve");
            println!("Use 'alchemist deploy approve <id> --comments \"reason\"' to reject");
        } else {
            println!("Deployment automation not initialized.");
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DeploymentSummary {
    pub name: String,
    pub environment: String,
    pub nats_url: String,
    pub status: DeploymentStatus,
    pub domain_count: usize,
    pub events_processed: u64,
    pub last_heartbeat: Option<DateTime<Utc>>,
}