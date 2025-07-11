//! Deployment automation features for Alchemist
//! 
//! Provides automated deployment pipelines, GitOps integration, canary deployments,
//! and multi-environment promotion workflows.

use anyhow::{Result, Context};
use async_trait::async_trait;
use chrono::{DateTime, Utc, Timelike, Weekday, Datelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::{
    deployment::{Deployment, DeploymentStatus, DeploymentTarget, DeploymentStrategy, DeploymentManager},
    nix_deployment::{NixDeployer, NixDeploymentSpec, DeploymentEvent, HealthEvent},
    nats_client::NatsClient,
    error::AlchemistError,
};

/// Deployment automation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationConfig {
    /// Enable GitOps mode
    pub gitops_enabled: bool,
    /// Git repository URL
    pub git_repo: Option<String>,
    /// Branch to watch
    pub git_branch: String,
    /// Path to deployment configs in repo
    pub config_path: String,
    /// Enable automated rollbacks
    pub auto_rollback: bool,
    /// Rollback threshold (% of failing health checks)
    pub rollback_threshold: f32,
    /// Enable canary deployments
    pub canary_enabled: bool,
    /// Canary traffic percentage
    pub canary_percentage: u8,
    /// Canary duration in minutes
    pub canary_duration: u32,
    /// Deployment windows
    pub deployment_windows: Vec<DeploymentWindow>,
    /// Promotion policies
    pub promotion_policies: Vec<PromotionPolicy>,
}

/// Deployment window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentWindow {
    /// Window name
    pub name: String,
    /// Allowed days
    pub days: Vec<Weekday>,
    /// Start hour (UTC)
    pub start_hour: u32,
    /// End hour (UTC)
    pub end_hour: u32,
    /// Environments allowed in this window
    pub environments: Vec<String>,
}

/// Promotion policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionPolicy {
    /// Policy name
    pub name: String,
    /// Source environment
    pub from_environment: String,
    /// Target environment
    pub to_environment: String,
    /// Automatic promotion
    pub automatic: bool,
    /// Required approvals
    pub required_approvals: u32,
    /// Health check duration before promotion
    pub health_check_minutes: u32,
    /// Success criteria
    pub success_criteria: SuccessCriteria,
}

/// Success criteria for promotions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriteria {
    /// Minimum health percentage
    pub min_health_percentage: f32,
    /// Maximum error rate
    pub max_error_rate: f32,
    /// Minimum uptime minutes
    pub min_uptime_minutes: u32,
    /// Required metrics
    pub required_metrics: Vec<String>,
}

/// Deployment pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentPipeline {
    /// Pipeline ID
    pub id: String,
    /// Pipeline name
    pub name: String,
    /// Pipeline stages
    pub stages: Vec<PipelineStage>,
    /// Current stage
    pub current_stage: usize,
    /// Pipeline status
    pub status: PipelineStatus,
    /// Started at
    pub started_at: DateTime<Utc>,
    /// Completed at
    pub completed_at: Option<DateTime<Utc>>,
    /// Trigger
    pub trigger: PipelineTrigger,
}

/// Pipeline stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    /// Stage name
    pub name: String,
    /// Stage type
    pub stage_type: StageType,
    /// Target environment
    pub environment: String,
    /// Deployment configuration
    pub deployment_config: HashMap<String, serde_json::Value>,
    /// Stage status
    pub status: StageStatus,
    /// Started at
    pub started_at: Option<DateTime<Utc>>,
    /// Completed at
    pub completed_at: Option<DateTime<Utc>>,
    /// Stage results
    pub results: HashMap<String, String>,
}

/// Stage type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StageType {
    /// Build stage
    Build,
    /// Test stage
    Test,
    /// Security scan
    SecurityScan,
    /// Deploy stage
    Deploy,
    /// Health check
    HealthCheck,
    /// Smoke test
    SmokeTest,
    /// Load test
    LoadTest,
    /// Manual approval
    Approval,
    /// Rollback
    Rollback,
}

/// Stage status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StageStatus {
    Pending,
    Running,
    Success,
    Failed,
    Skipped,
    Cancelled,
}

/// Pipeline status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineStatus {
    Running,
    Success,
    Failed,
    Cancelled,
    RolledBack,
}

/// Pipeline trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineTrigger {
    /// Manual trigger
    Manual { user: String },
    /// Git commit trigger
    GitCommit { commit: String, author: String },
    /// Schedule trigger
    Schedule { schedule: String },
    /// API trigger
    Api { source: String },
    /// Event trigger
    Event { event_type: String, event_id: String },
}

/// Canary deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryDeployment {
    /// Canary ID
    pub id: String,
    /// Target deployment
    pub deployment_id: String,
    /// Canary version
    pub canary_version: String,
    /// Stable version
    pub stable_version: String,
    /// Traffic percentage
    pub traffic_percentage: u8,
    /// Started at
    pub started_at: DateTime<Utc>,
    /// Promotion time
    pub promotion_time: DateTime<Utc>,
    /// Metrics
    pub metrics: CanaryMetrics,
    /// Status
    pub status: CanaryStatus,
}

/// Canary metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryMetrics {
    /// Request count
    pub request_count: u64,
    /// Error count
    pub error_count: u64,
    /// Average response time
    pub avg_response_time_ms: f64,
    /// Success rate
    pub success_rate: f32,
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
}

/// Canary status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CanaryStatus {
    Running,
    Promoting,
    Promoted,
    RollingBack,
    RolledBack,
    Failed,
}

/// Deployment approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentApproval {
    /// Approval ID
    pub id: String,
    /// Pipeline ID
    pub pipeline_id: String,
    /// Stage name
    pub stage_name: String,
    /// Required approvers
    pub required_approvers: Vec<String>,
    /// Current approvals
    pub approvals: Vec<Approval>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Expires at
    pub expires_at: DateTime<Utc>,
    /// Status
    pub status: ApprovalStatus,
}

/// Individual approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approval {
    /// Approver
    pub approver: String,
    /// Approved
    pub approved: bool,
    /// Comments
    pub comments: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Approval status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

/// Deployment automation manager
pub struct DeploymentAutomation {
    /// Automation configuration
    config: AutomationConfig,
    /// Deployment manager
    deployment_manager: Arc<RwLock<DeploymentManager>>,
    /// Nix deployer
    nix_deployer: Arc<NixDeployer>,
    /// NATS client
    nats_client: NatsClient,
    /// Active pipelines
    pipelines: Arc<RwLock<HashMap<String, DeploymentPipeline>>>,
    /// Active canaries
    canaries: Arc<RwLock<HashMap<String, CanaryDeployment>>>,
    /// Pending approvals
    approvals: Arc<RwLock<HashMap<String, DeploymentApproval>>>,
}

impl DeploymentAutomation {
    /// Create new deployment automation
    pub async fn new(
        config: AutomationConfig,
        deployment_manager: Arc<RwLock<DeploymentManager>>,
        nix_deployer: Arc<NixDeployer>,
        nats_client: NatsClient,
    ) -> Result<Self> {
        let automation = Self {
            config,
            deployment_manager,
            nix_deployer,
            nats_client,
            pipelines: Arc::new(RwLock::new(HashMap::new())),
            canaries: Arc::new(RwLock::new(HashMap::new())),
            approvals: Arc::new(RwLock::new(HashMap::new())),
        };

        // Start background tasks
        automation.start_background_tasks().await?;

        Ok(automation)
    }

    /// Start background tasks
    async fn start_background_tasks(&self) -> Result<()> {
        // Start GitOps watcher if enabled
        if self.config.gitops_enabled {
            self.start_gitops_watcher().await?;
        }

        // Start deployment window enforcer
        self.start_window_enforcer().await?;

        // Start canary monitor
        if self.config.canary_enabled {
            self.start_canary_monitor().await?;
        }

        // Start health monitor for auto-rollback
        if self.config.auto_rollback {
            self.start_health_monitor().await?;
        }

        // Start promotion monitor
        self.start_promotion_monitor().await?;

        Ok(())
    }

    /// Create deployment pipeline
    pub async fn create_pipeline(
        &self,
        name: String,
        environments: Vec<String>,
        trigger: PipelineTrigger,
    ) -> Result<String> {
        let pipeline_id = Uuid::new_v4().to_string();
        
        // Build pipeline stages based on environments
        let mut stages = vec![];
        
        // Add build stage
        stages.push(PipelineStage {
            name: "Build".to_string(),
            stage_type: StageType::Build,
            environment: "build".to_string(),
            deployment_config: HashMap::new(),
            status: StageStatus::Pending,
            started_at: None,
            completed_at: None,
            results: HashMap::new(),
        });

        // Add test stage
        stages.push(PipelineStage {
            name: "Test".to_string(),
            stage_type: StageType::Test,
            environment: "test".to_string(),
            deployment_config: HashMap::new(),
            status: StageStatus::Pending,
            started_at: None,
            completed_at: None,
            results: HashMap::new(),
        });

        // Add security scan
        stages.push(PipelineStage {
            name: "Security Scan".to_string(),
            stage_type: StageType::SecurityScan,
            environment: "scan".to_string(),
            deployment_config: HashMap::new(),
            status: StageStatus::Pending,
            started_at: None,
            completed_at: None,
            results: HashMap::new(),
        });

        // Add deployment stages for each environment
        for (i, env) in environments.iter().enumerate() {
            // Deploy stage
            stages.push(PipelineStage {
                name: format!("Deploy to {}", env),
                stage_type: StageType::Deploy,
                environment: env.clone(),
                deployment_config: HashMap::new(),
                status: StageStatus::Pending,
                started_at: None,
                completed_at: None,
                results: HashMap::new(),
            });

            // Health check stage
            stages.push(PipelineStage {
                name: format!("Health Check {}", env),
                stage_type: StageType::HealthCheck,
                environment: env.clone(),
                deployment_config: HashMap::new(),
                status: StageStatus::Pending,
                started_at: None,
                completed_at: None,
                results: HashMap::new(),
            });

            // Add approval stage for production
            if env == "production" || i < environments.len() - 1 {
                stages.push(PipelineStage {
                    name: format!("Approve {}", env),
                    stage_type: StageType::Approval,
                    environment: env.clone(),
                    deployment_config: HashMap::new(),
                    status: StageStatus::Pending,
                    started_at: None,
                    completed_at: None,
                    results: HashMap::new(),
                });
            }
        }

        let pipeline = DeploymentPipeline {
            id: pipeline_id.clone(),
            name,
            stages,
            current_stage: 0,
            status: PipelineStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            trigger,
        };

        // Store pipeline
        self.pipelines.write().await.insert(pipeline_id.clone(), pipeline.clone());

        // Start pipeline execution in background
        // In production, this would be handled by a task queue
        info!("Pipeline {} created and ready for execution", pipeline_id);

        // Publish pipeline created event
        self.publish_automation_event(
            AutomationEvent::PipelineCreated {
                pipeline_id: pipeline_id.clone(),
                name: pipeline.name,
                stages: pipeline.stages.len(),
            }
        ).await?;

        Ok(pipeline_id)
    }

    /// Execute pipeline
    async fn execute_pipeline(&self, pipeline_id: String) -> Result<()> {
        // For now, run directly without spawning
        // In production, we'd ensure all types are Send + Sync
        if let Err(e) = self.run_pipeline(pipeline_id).await {
            error!("Pipeline execution failed: {}", e);
        }

        Ok(())
    }

    /// Run pipeline stages
    async fn run_pipeline(&self, pipeline_id: String) -> Result<()> {
        loop {
            // Get current pipeline state
            let (current_stage, stage_type, environment) = {
                let pipelines = self.pipelines.read().await;
                let pipeline = pipelines.get(&pipeline_id)
                    .ok_or_else(|| anyhow::anyhow!("Pipeline not found"))?;
                
                if pipeline.current_stage >= pipeline.stages.len() {
                    // Pipeline completed
                    break;
                }

                let stage = &pipeline.stages[pipeline.current_stage];
                (pipeline.current_stage, stage.stage_type.clone(), stage.environment.clone())
            };

            // Check deployment window
            if !self.is_deployment_allowed(&environment).await? {
                info!("Deployment window closed for environment: {}", environment);
                
                // Wait for window to open
                tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
                continue;
            }

            // Execute stage
            let stage_result = match stage_type {
                StageType::Build => self.execute_build_stage(&pipeline_id).await,
                StageType::Test => self.execute_test_stage(&pipeline_id).await,
                StageType::SecurityScan => self.execute_security_scan(&pipeline_id).await,
                StageType::Deploy => self.execute_deploy_stage(&pipeline_id, &environment).await,
                StageType::HealthCheck => self.execute_health_check(&pipeline_id, &environment).await,
                StageType::SmokeTest => self.execute_smoke_test(&pipeline_id, &environment).await,
                StageType::LoadTest => self.execute_load_test(&pipeline_id, &environment).await,
                StageType::Approval => self.execute_approval_stage(&pipeline_id, current_stage).await,
                StageType::Rollback => self.execute_rollback_stage(&pipeline_id, &environment).await,
            };

            // Update stage status
            match stage_result {
                Ok(_) => {
                    self.update_stage_status(&pipeline_id, current_stage, StageStatus::Success).await?;
                    
                    // Move to next stage
                    self.advance_pipeline(&pipeline_id).await?;
                }
                Err(e) => {
                    error!("Stage failed: {}", e);
                    self.update_stage_status(&pipeline_id, current_stage, StageStatus::Failed).await?;
                    
                    // Update pipeline status
                    self.update_pipeline_status(&pipeline_id, PipelineStatus::Failed).await?;
                    
                    // Trigger rollback if configured
                    if self.config.auto_rollback {
                        self.trigger_pipeline_rollback(&pipeline_id).await?;
                    }
                    
                    break;
                }
            }
        }

        // Check if pipeline completed successfully
        let completed = {
            let pipelines = self.pipelines.read().await;
            let pipeline = pipelines.get(&pipeline_id).unwrap();
            pipeline.current_stage >= pipeline.stages.len()
        };

        if completed {
            self.update_pipeline_status(&pipeline_id, PipelineStatus::Success).await?;
            
            // Trigger any promotion policies
            self.check_promotion_policies(&pipeline_id).await?;
        }

        Ok(())
    }

    /// Execute build stage
    async fn execute_build_stage(&self, pipeline_id: &str) -> Result<()> {
        info!("Executing build stage for pipeline: {}", pipeline_id);
        
        // Simulate build process
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        
        // In real implementation, this would:
        // 1. Checkout code
        // 2. Run build commands
        // 3. Create artifacts
        // 4. Push to registry
        
        Ok(())
    }

    /// Execute test stage
    async fn execute_test_stage(&self, pipeline_id: &str) -> Result<()> {
        info!("Executing test stage for pipeline: {}", pipeline_id);
        
        // Run tests
        tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
        
        // In real implementation, this would:
        // 1. Run unit tests
        // 2. Run integration tests
        // 3. Generate coverage reports
        // 4. Check quality gates
        
        Ok(())
    }

    /// Execute security scan stage
    async fn execute_security_scan(&self, pipeline_id: &str) -> Result<()> {
        info!("Executing security scan for pipeline: {}", pipeline_id);
        
        // Run security scans
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
        
        // In real implementation, this would:
        // 1. Run SAST (Static Application Security Testing)
        // 2. Run dependency vulnerability scan
        // 3. Run container scan
        // 4. Check security policies
        
        Ok(())
    }

    /// Execute deploy stage
    async fn execute_deploy_stage(&self, pipeline_id: &str, environment: &str) -> Result<()> {
        info!("Executing deploy stage for pipeline: {} to {}", pipeline_id, environment);
        
        // Get deployment configuration
        let deployment_manager = self.deployment_manager.read().await;
        let deployment_config = deployment_manager.get_config(environment)
            .ok_or_else(|| anyhow::anyhow!("Deployment config not found for: {}", environment))?;
        
        // Check if canary deployment is enabled
        if self.config.canary_enabled && environment == "production" {
            self.start_canary_deployment(pipeline_id, environment).await?;
        } else {
            // Regular deployment
            // In real implementation, this would use the deployment manager
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
        
        Ok(())
    }

    /// Execute health check stage
    async fn execute_health_check(&self, pipeline_id: &str, environment: &str) -> Result<()> {
        info!("Executing health check for pipeline: {} in {}", pipeline_id, environment);
        
        // Run health checks
        let start_time = Utc::now();
        let check_duration = tokio::time::Duration::from_secs(120);
        
        while Utc::now().signed_duration_since(start_time).to_std()? < check_duration {
            // Check service health
            // In real implementation, this would query actual health endpoints
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
        
        Ok(())
    }

    /// Execute smoke test stage
    async fn execute_smoke_test(&self, pipeline_id: &str, environment: &str) -> Result<()> {
        info!("Executing smoke tests for pipeline: {} in {}", pipeline_id, environment);
        
        // Run smoke tests
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        
        // In real implementation, this would:
        // 1. Run critical path tests
        // 2. Verify core functionality
        // 3. Check integrations
        
        Ok(())
    }

    /// Execute load test stage
    async fn execute_load_test(&self, pipeline_id: &str, environment: &str) -> Result<()> {
        info!("Executing load tests for pipeline: {} in {}", pipeline_id, environment);
        
        // Run load tests
        tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
        
        // In real implementation, this would:
        // 1. Generate load
        // 2. Monitor performance metrics
        // 3. Check SLAs
        // 4. Generate reports
        
        Ok(())
    }

    /// Execute approval stage
    async fn execute_approval_stage(&self, pipeline_id: &str, stage_index: usize) -> Result<()> {
        info!("Waiting for approval for pipeline: {} stage: {}", pipeline_id, stage_index);
        
        // Create approval request
        let approval_id = Uuid::new_v4().to_string();
        let approval = DeploymentApproval {
            id: approval_id.clone(),
            pipeline_id: pipeline_id.to_string(),
            stage_name: format!("Stage {}", stage_index),
            required_approvers: vec!["admin".to_string(), "lead".to_string()],
            approvals: vec![],
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(24),
            status: ApprovalStatus::Pending,
        };
        
        self.approvals.write().await.insert(approval_id.clone(), approval);
        
        // Publish approval request
        self.publish_automation_event(
            AutomationEvent::ApprovalRequested {
                approval_id: approval_id.clone(),
                pipeline_id: pipeline_id.to_string(),
                stage_name: format!("Stage {}", stage_index),
                approvers: vec!["admin".to_string(), "lead".to_string()],
            }
        ).await?;
        
        // Wait for approval
        let timeout = tokio::time::Duration::from_secs(3600); // 1 hour timeout
        let start_time = tokio::time::Instant::now();
        
        loop {
            if start_time.elapsed() > timeout {
                return Err(anyhow::anyhow!("Approval timeout"));
            }
            
            let status = {
                let approvals = self.approvals.read().await;
                approvals.get(&approval_id).map(|a| a.status.clone())
            };
            
            match status {
                Some(ApprovalStatus::Approved) => return Ok(()),
                Some(ApprovalStatus::Rejected) => return Err(anyhow::anyhow!("Approval rejected")),
                Some(ApprovalStatus::Expired) => return Err(anyhow::anyhow!("Approval expired")),
                _ => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
        }
    }

    /// Execute rollback stage
    async fn execute_rollback_stage(&self, pipeline_id: &str, environment: &str) -> Result<()> {
        info!("Executing rollback for pipeline: {} in {}", pipeline_id, environment);
        
        // Perform rollback
        // In real implementation, this would use the deployment manager
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        
        Ok(())
    }

    /// Start canary deployment
    async fn start_canary_deployment(&self, pipeline_id: &str, environment: &str) -> Result<()> {
        let canary_id = Uuid::new_v4().to_string();
        
        let canary = CanaryDeployment {
            id: canary_id.clone(),
            deployment_id: pipeline_id.to_string(),
            canary_version: "v2.0.0".to_string(), // Would come from build
            stable_version: "v1.0.0".to_string(), // Current version
            traffic_percentage: self.config.canary_percentage,
            started_at: Utc::now(),
            promotion_time: Utc::now() + chrono::Duration::minutes(self.config.canary_duration as i64),
            metrics: CanaryMetrics {
                request_count: 0,
                error_count: 0,
                avg_response_time_ms: 0.0,
                success_rate: 100.0,
                custom_metrics: HashMap::new(),
            },
            status: CanaryStatus::Running,
        };
        
        self.canaries.write().await.insert(canary_id.clone(), canary);
        
        // Publish canary started event
        self.publish_automation_event(
            AutomationEvent::CanaryStarted {
                canary_id,
                deployment_id: pipeline_id.to_string(),
                traffic_percentage: self.config.canary_percentage,
            }
        ).await?;
        
        Ok(())
    }

    /// Check if deployment is allowed in current window
    async fn is_deployment_allowed(&self, environment: &str) -> Result<bool> {
        let now = Utc::now();
        let current_day = now.weekday();
        let current_hour = now.hour();
        
        for window in &self.config.deployment_windows {
            if window.environments.contains(&environment.to_string()) &&
               window.days.contains(&current_day) &&
               current_hour >= window.start_hour &&
               current_hour < window.end_hour {
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Update stage status
    async fn update_stage_status(
        &self,
        pipeline_id: &str,
        stage_index: usize,
        status: StageStatus,
    ) -> Result<()> {
        let mut pipelines = self.pipelines.write().await;
        if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
            if let Some(stage) = pipeline.stages.get_mut(stage_index) {
                stage.status = status;
                stage.completed_at = Some(Utc::now());
            }
        }
        Ok(())
    }

    /// Update pipeline status
    async fn update_pipeline_status(
        &self,
        pipeline_id: &str,
        status: PipelineStatus,
    ) -> Result<()> {
        let mut pipelines = self.pipelines.write().await;
        if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
            pipeline.status = status;
            pipeline.completed_at = Some(Utc::now());
        }
        Ok(())
    }

    /// Advance pipeline to next stage
    async fn advance_pipeline(&self, pipeline_id: &str) -> Result<()> {
        let mut pipelines = self.pipelines.write().await;
        if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
            pipeline.current_stage += 1;
        }
        Ok(())
    }

    /// Trigger pipeline rollback
    async fn trigger_pipeline_rollback(&self, pipeline_id: &str) -> Result<()> {
        info!("Triggering rollback for pipeline: {}", pipeline_id);
        
        // Add rollback stage
        let mut pipelines = self.pipelines.write().await;
        if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
            pipeline.stages.push(PipelineStage {
                name: "Rollback".to_string(),
                stage_type: StageType::Rollback,
                environment: "all".to_string(),
                deployment_config: HashMap::new(),
                status: StageStatus::Pending,
                started_at: None,
                completed_at: None,
                results: HashMap::new(),
            });
            
            pipeline.status = PipelineStatus::RolledBack;
        }
        
        Ok(())
    }

    /// Check promotion policies
    async fn check_promotion_policies(&self, pipeline_id: &str) -> Result<()> {
        let pipeline = {
            let pipelines = self.pipelines.read().await;
            pipelines.get(pipeline_id).cloned()
        };
        
        if let Some(pipeline) = pipeline {
            // Find the last deployed environment
            let last_env = pipeline.stages.iter()
                .filter(|s| matches!(s.stage_type, StageType::Deploy))
                .filter(|s| matches!(s.status, StageStatus::Success))
                .last()
                .map(|s| &s.environment);
            
            if let Some(from_env) = last_env {
                // Check promotion policies
                for policy in &self.config.promotion_policies {
                    if &policy.from_environment == from_env && policy.automatic {
                        // Check success criteria
                        if self.check_success_criteria(&policy.success_criteria, from_env).await? {
                            // Trigger promotion
                            self.trigger_promotion(
                                pipeline_id,
                                &policy.from_environment,
                                &policy.to_environment,
                            ).await?;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Check success criteria
    async fn check_success_criteria(
        &self,
        criteria: &SuccessCriteria,
        environment: &str,
    ) -> Result<bool> {
        // In real implementation, this would check actual metrics
        // For now, return true
        Ok(true)
    }

    /// Trigger promotion
    async fn trigger_promotion(
        &self,
        pipeline_id: &str,
        from_env: &str,
        to_env: &str,
    ) -> Result<()> {
        info!("Triggering promotion from {} to {}", from_env, to_env);
        
        // Create new pipeline for promotion
        self.create_pipeline(
            format!("Promote {} to {}", from_env, to_env),
            vec![to_env.to_string()],
            PipelineTrigger::Event {
                event_type: "promotion".to_string(),
                event_id: pipeline_id.to_string(),
            },
        ).await?;
        
        Ok(())
    }

    /// Start GitOps watcher
    async fn start_gitops_watcher(&self) -> Result<()> {
        if let Some(repo_url) = &self.config.git_repo {
            info!("Starting GitOps watcher for: {}", repo_url);
            
            // In real implementation, this would:
            // 1. Clone/pull the repository
            // 2. Watch for changes
            // 3. Trigger deployments on changes
            
            let automation = self.clone();
            tokio::spawn(async move {
                loop {
                    // Check for git changes
                    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                    
                    // If changes detected, trigger pipeline
                    // automation.create_pipeline(...).await
                }
            });
        }
        
        Ok(())
    }

    /// Start deployment window enforcer
    async fn start_window_enforcer(&self) -> Result<()> {
        let automation = self.clone();
        
        tokio::spawn(async move {
            loop {
                // Check active deployments against windows
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                
                // Pause deployments outside of windows
                let pipelines = automation.pipelines.read().await;
                for (id, pipeline) in pipelines.iter() {
                    if matches!(pipeline.status, PipelineStatus::Running) {
                        // Check if current stage is allowed
                        // If not, pause the pipeline
                    }
                }
            }
        });
        
        Ok(())
    }

    /// Start canary monitor
    async fn start_canary_monitor(&self) -> Result<()> {
        let automation = self.clone();
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                
                // Check canary deployments
                let mut canaries_to_update = vec![];
                {
                    let canaries = automation.canaries.read().await;
                    for (id, canary) in canaries.iter() {
                        if matches!(canary.status, CanaryStatus::Running) {
                            if Utc::now() > canary.promotion_time {
                                canaries_to_update.push(id.clone());
                            }
                        }
                    }
                }
                
                // Update canaries
                for canary_id in canaries_to_update {
                    if let Err(e) = automation.evaluate_canary(&canary_id).await {
                        error!("Failed to evaluate canary {}: {}", canary_id, e);
                    }
                }
            }
        });
        
        Ok(())
    }

    /// Evaluate canary deployment
    async fn evaluate_canary(&self, canary_id: &str) -> Result<()> {
        let canary = {
            let canaries = self.canaries.read().await;
            canaries.get(canary_id).cloned()
        };
        
        if let Some(canary) = canary {
            // Check canary metrics
            if canary.metrics.success_rate > 95.0 && canary.metrics.error_count < 10 {
                // Promote canary
                self.promote_canary(canary_id).await?;
            } else {
                // Rollback canary
                self.rollback_canary(canary_id).await?;
            }
        }
        
        Ok(())
    }

    /// Promote canary deployment
    async fn promote_canary(&self, canary_id: &str) -> Result<()> {
        let mut canaries = self.canaries.write().await;
        if let Some(canary) = canaries.get_mut(canary_id) {
            canary.status = CanaryStatus::Promoted;
            canary.traffic_percentage = 100;
            
            info!("Promoted canary deployment: {}", canary_id);
            
            // Publish event
            self.publish_automation_event(
                AutomationEvent::CanaryPromoted {
                    canary_id: canary_id.to_string(),
                    version: canary.canary_version.clone(),
                }
            ).await?;
        }
        
        Ok(())
    }

    /// Rollback canary deployment
    async fn rollback_canary(&self, canary_id: &str) -> Result<()> {
        let mut canaries = self.canaries.write().await;
        if let Some(canary) = canaries.get_mut(canary_id) {
            canary.status = CanaryStatus::RolledBack;
            canary.traffic_percentage = 0;
            
            warn!("Rolled back canary deployment: {}", canary_id);
            
            // Publish event
            self.publish_automation_event(
                AutomationEvent::CanaryRolledBack {
                    canary_id: canary_id.to_string(),
                    reason: "Failed success criteria".to_string(),
                }
            ).await?;
        }
        
        Ok(())
    }

    /// Start health monitor for auto-rollback
    async fn start_health_monitor(&self) -> Result<()> {
        let automation = self.clone();
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                
                // Monitor health of active deployments
                // In real implementation, this would check actual health metrics
                // and trigger rollbacks if thresholds are exceeded
            }
        });
        
        Ok(())
    }

    /// Start promotion monitor
    async fn start_promotion_monitor(&self) -> Result<()> {
        let automation = self.clone();
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
                
                // Check for deployments ready for promotion
                // This would evaluate success criteria and trigger promotions
            }
        });
        
        Ok(())
    }

    /// Process approval
    pub async fn process_approval(
        &self,
        approval_id: String,
        approver: String,
        approved: bool,
        comments: Option<String>,
    ) -> Result<()> {
        let mut approvals = self.approvals.write().await;
        if let Some(approval) = approvals.get_mut(&approval_id) {
            approval.approvals.push(Approval {
                approver: approver.clone(),
                approved,
                comments,
                timestamp: Utc::now(),
            });
            
            // Check if we have enough approvals
            let approved_count = approval.approvals.iter().filter(|a| a.approved).count();
            let rejected_count = approval.approvals.iter().filter(|a| !a.approved).count();
            
            if approved_count >= approval.required_approvers.len() {
                approval.status = ApprovalStatus::Approved;
                info!("Approval {} approved", approval_id);
            } else if rejected_count > 0 {
                approval.status = ApprovalStatus::Rejected;
                info!("Approval {} rejected", approval_id);
            }
            
            // Publish event
            self.publish_automation_event(
                AutomationEvent::ApprovalProcessed {
                    approval_id: approval_id.clone(),
                    approver,
                    approved,
                    status: approval.status.clone(),
                }
            ).await?;
        }
        
        Ok(())
    }

    /// Get pipeline status
    pub async fn get_pipeline_status(&self, pipeline_id: &str) -> Result<Option<DeploymentPipeline>> {
        let pipelines = self.pipelines.read().await;
        Ok(pipelines.get(pipeline_id).cloned())
    }

    /// List active pipelines
    pub async fn list_pipelines(&self) -> Result<Vec<(String, String, PipelineStatus)>> {
        let pipelines = self.pipelines.read().await;
        Ok(pipelines.iter()
            .map(|(id, p)| (id.clone(), p.name.clone(), p.status.clone()))
            .collect())
    }

    /// List pending approvals
    pub async fn list_pending_approvals(&self) -> Result<Vec<DeploymentApproval>> {
        let approvals = self.approvals.read().await;
        Ok(approvals.values()
            .filter(|a| matches!(a.status, ApprovalStatus::Pending))
            .cloned()
            .collect())
    }

    /// Publish automation event
    async fn publish_automation_event(&self, event: AutomationEvent) -> Result<()> {
        let subject = "deployment.automation.events";
        let payload = serde_json::to_vec(&event)?;
        self.nats_client.publish(subject, payload).await?;
        Ok(())
    }

    /// Clone for async tasks
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            deployment_manager: Arc::clone(&self.deployment_manager),
            nix_deployer: Arc::clone(&self.nix_deployer),
            nats_client: self.nats_client.clone(),
            pipelines: Arc::clone(&self.pipelines),
            canaries: Arc::clone(&self.canaries),
            approvals: Arc::clone(&self.approvals),
        }
    }
}

/// Automation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomationEvent {
    /// Pipeline created
    PipelineCreated {
        pipeline_id: String,
        name: String,
        stages: usize,
    },
    /// Stage completed
    StageCompleted {
        pipeline_id: String,
        stage_name: String,
        status: StageStatus,
    },
    /// Pipeline completed
    PipelineCompleted {
        pipeline_id: String,
        status: PipelineStatus,
    },
    /// Approval requested
    ApprovalRequested {
        approval_id: String,
        pipeline_id: String,
        stage_name: String,
        approvers: Vec<String>,
    },
    /// Approval processed
    ApprovalProcessed {
        approval_id: String,
        approver: String,
        approved: bool,
        status: ApprovalStatus,
    },
    /// Canary started
    CanaryStarted {
        canary_id: String,
        deployment_id: String,
        traffic_percentage: u8,
    },
    /// Canary promoted
    CanaryPromoted {
        canary_id: String,
        version: String,
    },
    /// Canary rolled back
    CanaryRolledBack {
        canary_id: String,
        reason: String,
    },
    /// Promotion triggered
    PromotionTriggered {
        from_environment: String,
        to_environment: String,
        pipeline_id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_pipeline() {
        // Test pipeline creation
        // This would require mocking the dependencies
    }

    #[tokio::test]
    async fn test_approval_flow() {
        // Test approval workflow
    }

    #[tokio::test]
    async fn test_canary_deployment() {
        // Test canary deployment flow
    }

    #[tokio::test]
    async fn test_deployment_windows() {
        // Test deployment window enforcement
    }
}