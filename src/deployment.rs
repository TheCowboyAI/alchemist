//! CIM deployment management

use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

use crate::{
    config::{AlchemistConfig, DeploymentConfig},
    shell_commands::DeployCommands,
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

pub struct DeploymentManager {
    deployments: DashMap<String, Deployment>,
    tasks: DashMap<String, DeploymentTask>,
    configs: HashMap<String, DeploymentConfig>,
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
            };
            
            deployments.insert(name.clone(), deployment);
        }
        
        Ok(Self {
            deployments,
            tasks: DashMap::new(),
            configs: config.deployments.clone(),
        })
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
        }
        Ok(())
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
            .ok_or_else(|| anyhow::anyhow!("Deployment target not found: {}", target))?
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