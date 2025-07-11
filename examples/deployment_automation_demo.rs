//! Deployment automation demonstration
//! 
//! This example demonstrates the deployment automation features including:
//! - Deployment pipelines
//! - Multi-environment promotion
//! - Canary deployments
//! - Approval workflows
//! - GitOps integration

use alchemist::{
    deployment::{DeploymentManager, DeploymentTarget, DeploymentStrategy},
    deployment_automation::{
        AutomationConfig, DeploymentWindow, PromotionPolicy, SuccessCriteria,
        PipelineTrigger,
    },
    config::AlchemistConfig,
    shell_commands::DeployCommands,
};
use anyhow::Result;
use chrono::Weekday;
use std::collections::HashMap;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🚀 Alchemist Deployment Automation Demo");
    
    // Create configuration
    let config = AlchemistConfig::default();
    
    // Initialize deployment manager
    let mut deployment_manager = DeploymentManager::new(&config).await?;
    
    // Initialize Nix deployer (required for automation)
    deployment_manager.init_nix_deployer("nats://localhost:4222").await?;
    
    // Create automation configuration
    let automation_config = AutomationConfig {
        gitops_enabled: true,
        git_repo: Some("https://github.com/example/deployments.git".to_string()),
        git_branch: "main".to_string(),
        config_path: "deployments/".to_string(),
        auto_rollback: true,
        rollback_threshold: 0.2, // 20% failure threshold
        canary_enabled: true,
        canary_percentage: 10,
        canary_duration: 30, // 30 minutes
        deployment_windows: vec![
            // Weekday deployment window
            DeploymentWindow {
                name: "weekday".to_string(),
                days: vec![
                    Weekday::Mon,
                    Weekday::Tue,
                    Weekday::Wed,
                    Weekday::Thu,
                    Weekday::Fri,
                ],
                start_hour: 9,  // 9 AM UTC
                end_hour: 17,   // 5 PM UTC
                environments: vec!["development".to_string(), "staging".to_string()],
            },
            // Weekend deployment window for production
            DeploymentWindow {
                name: "weekend".to_string(),
                days: vec![Weekday::Sat, Weekday::Sun],
                start_hour: 0,
                end_hour: 24,
                environments: vec!["production".to_string()],
            },
        ],
        promotion_policies: vec![
            // Auto-promote from dev to staging
            PromotionPolicy {
                name: "dev-to-staging".to_string(),
                from_environment: "development".to_string(),
                to_environment: "staging".to_string(),
                automatic: true,
                required_approvals: 0,
                health_check_minutes: 15,
                success_criteria: SuccessCriteria {
                    min_health_percentage: 95.0,
                    max_error_rate: 0.01,
                    min_uptime_minutes: 10,
                    required_metrics: vec![
                        "cpu_usage".to_string(),
                        "memory_usage".to_string(),
                        "response_time".to_string(),
                    ],
                },
            },
            // Manual promotion from staging to production
            PromotionPolicy {
                name: "staging-to-production".to_string(),
                from_environment: "staging".to_string(),
                to_environment: "production".to_string(),
                automatic: false,
                required_approvals: 2,
                health_check_minutes: 60,
                success_criteria: SuccessCriteria {
                    min_health_percentage: 99.0,
                    max_error_rate: 0.001,
                    min_uptime_minutes: 30,
                    required_metrics: vec![
                        "cpu_usage".to_string(),
                        "memory_usage".to_string(),
                        "response_time".to_string(),
                        "error_rate".to_string(),
                        "request_count".to_string(),
                    ],
                },
            },
        ],
    };
    
    // Initialize automation
    deployment_manager.init_automation(automation_config).await?;
    
    info!("✅ Deployment automation initialized");
    
    // Demo 1: Create a deployment pipeline
    info!("\n📋 Demo 1: Creating deployment pipeline");
    deployment_manager.handle_command(DeployCommands::Pipeline {
        name: "CIM v2.0 Release".to_string(),
        environments: vec!["development".to_string(), "staging".to_string(), "production".to_string()],
        canary: true,
    }).await?;
    
    // Demo 2: List pipelines
    info!("\n📋 Demo 2: Listing pipelines");
    deployment_manager.handle_command(DeployCommands::Pipelines).await?;
    
    // Demo 3: Check pipeline status
    info!("\n📋 Demo 3: Checking pipeline status");
    // Note: In a real scenario, you would use the actual pipeline ID
    deployment_manager.handle_command(DeployCommands::PipelineStatus {
        id: "example-pipeline-id".to_string(),
    }).await?;
    
    // Demo 4: List pending approvals
    info!("\n📋 Demo 4: Listing pending approvals");
    deployment_manager.handle_command(DeployCommands::Approvals).await?;
    
    // Demo 5: Process an approval
    info!("\n📋 Demo 5: Processing approval");
    // Note: In a real scenario, you would use the actual approval ID
    deployment_manager.handle_command(DeployCommands::Approve {
        id: "example-approval-id".to_string(),
        approve: true,
        comments: Some("Approved after review".to_string()),
    }).await?;
    
    // Demo 6: Deploy with regular deployment command
    info!("\n📋 Demo 6: Regular deployment (will be tracked by automation)");
    deployment_manager.handle_command(DeployCommands::Deploy {
        target: "local".to_string(),
        domains: vec!["graph".to_string(), "workflow".to_string()],
    }).await?;
    
    // Demo 7: Generate Nix configurations
    info!("\n📋 Demo 7: Generating Nix configurations");
    deployment_manager.handle_command(DeployCommands::Generate {
        target: "local".to_string(),
    }).await?;
    
    // Demo 8: Validate deployment
    info!("\n📋 Demo 8: Validating deployment");
    deployment_manager.handle_command(DeployCommands::Validate {
        target: "local".to_string(),
    }).await?;
    
    info!("\n🎉 Deployment automation demo completed!");
    info!("\n📚 Key Features Demonstrated:");
    info!("   - Deployment pipelines with multiple stages");
    info!("   - Multi-environment progression (dev → staging → production)");
    info!("   - Canary deployments with gradual traffic shifting");
    info!("   - Approval workflows with required approvers");
    info!("   - Deployment windows for controlled rollouts");
    info!("   - Automatic promotion based on success criteria");
    info!("   - GitOps integration for configuration management");
    info!("   - Automatic rollback on failure thresholds");
    
    info!("\n💡 Next Steps:");
    info!("   1. Monitor pipeline progress with 'alchemist deploy pipeline-status <id>'");
    info!("   2. Approve deployments with 'alchemist deploy approve <id> --approve'");
    info!("   3. Create custom pipelines with 'alchemist deploy pipeline <name> -e dev,staging,prod'");
    info!("   4. Enable canary deployments with '--canary' flag");
    info!("   5. Configure GitOps by updating the automation config");
    
    Ok(())
}