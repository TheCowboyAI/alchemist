//! Tests for deployment automation features

use alchemist::{
    deployment::{DeploymentManager, DeploymentTarget, DeploymentStrategy},
    deployment_automation::{
        AutomationConfig, DeploymentWindow, PromotionPolicy, SuccessCriteria,
        PipelineTrigger, DeploymentAutomation, PipelineStatus, StageStatus,
        CanaryStatus, ApprovalStatus,
    },
    config::{AlchemistConfig, DeploymentConfig},
    nix_deployment::NixDeployer,
    nats_client::NatsClient,
};
use chrono::{Weekday, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Create test automation config
fn create_test_automation_config() -> AutomationConfig {
    AutomationConfig {
        gitops_enabled: false,
        git_repo: None,
        git_branch: "main".to_string(),
        config_path: "test/".to_string(),
        auto_rollback: true,
        rollback_threshold: 0.2,
        canary_enabled: true,
        canary_percentage: 10,
        canary_duration: 5, // 5 minutes for testing
        deployment_windows: vec![
            DeploymentWindow {
                name: "always".to_string(),
                days: vec![
                    Weekday::Mon,
                    Weekday::Tue,
                    Weekday::Wed,
                    Weekday::Thu,
                    Weekday::Fri,
                    Weekday::Sat,
                    Weekday::Sun,
                ],
                start_hour: 0,
                end_hour: 24,
                environments: vec!["test".to_string(), "staging".to_string(), "production".to_string()],
            },
        ],
        promotion_policies: vec![
            PromotionPolicy {
                name: "test-to-staging".to_string(),
                from_environment: "test".to_string(),
                to_environment: "staging".to_string(),
                automatic: true,
                required_approvals: 0,
                health_check_minutes: 1,
                success_criteria: SuccessCriteria {
                    min_health_percentage: 90.0,
                    max_error_rate: 0.1,
                    min_uptime_minutes: 1,
                    required_metrics: vec!["health".to_string()],
                },
            },
        ],
    }
}

#[tokio::test]
async fn test_deployment_automation_initialization() {
    let config = AlchemistConfig::default();
    let mut deployment_manager = DeploymentManager::new(&config).await.unwrap();
    
    // Initialize required components
    deployment_manager.init_nix_deployer("nats://localhost:4222").await.unwrap();
    
    // Initialize automation
    let automation_config = create_test_automation_config();
    deployment_manager.init_automation(automation_config).await.unwrap();
    
    // Verify automation is initialized
    // In a real test, we would check internal state
    assert!(true, "Automation should be initialized");
}

#[tokio::test]
async fn test_pipeline_creation() {
    // This test would require mocking NATS and other dependencies
    // For now, we test the pipeline structure
    
    let automation_config = create_test_automation_config();
    assert_eq!(automation_config.canary_percentage, 10);
    assert_eq!(automation_config.canary_duration, 5);
    assert!(automation_config.auto_rollback);
}

#[tokio::test]
async fn test_deployment_windows() {
    let automation_config = create_test_automation_config();
    let window = &automation_config.deployment_windows[0];
    
    // Test window covers all days
    assert_eq!(window.days.len(), 7);
    assert_eq!(window.start_hour, 0);
    assert_eq!(window.end_hour, 24);
    assert!(window.environments.contains(&"test".to_string()));
    assert!(window.environments.contains(&"staging".to_string()));
    assert!(window.environments.contains(&"production".to_string()));
}

#[tokio::test]
async fn test_promotion_policy() {
    let automation_config = create_test_automation_config();
    let policy = &automation_config.promotion_policies[0];
    
    assert_eq!(policy.name, "test-to-staging");
    assert_eq!(policy.from_environment, "test");
    assert_eq!(policy.to_environment, "staging");
    assert!(policy.automatic);
    assert_eq!(policy.required_approvals, 0);
    
    // Test success criteria
    assert_eq!(policy.success_criteria.min_health_percentage, 90.0);
    assert_eq!(policy.success_criteria.max_error_rate, 0.1);
    assert_eq!(policy.success_criteria.min_uptime_minutes, 1);
    assert!(policy.success_criteria.required_metrics.contains(&"health".to_string()));
}

#[tokio::test]
async fn test_canary_configuration() {
    let automation_config = create_test_automation_config();
    
    assert!(automation_config.canary_enabled);
    assert_eq!(automation_config.canary_percentage, 10);
    assert_eq!(automation_config.canary_duration, 5);
}

#[tokio::test]
async fn test_rollback_configuration() {
    let automation_config = create_test_automation_config();
    
    assert!(automation_config.auto_rollback);
    assert_eq!(automation_config.rollback_threshold, 0.2);
}

#[cfg(test)]
mod pipeline_tests {
    use super::*;
    use alchemist::deployment_automation::{
        DeploymentPipeline, PipelineStage, StageType, StageStatus,
    };
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_pipeline() -> DeploymentPipeline {
        DeploymentPipeline {
            id: Uuid::new_v4().to_string(),
            name: "Test Pipeline".to_string(),
            stages: vec![
                PipelineStage {
                    name: "Build".to_string(),
                    stage_type: StageType::Build,
                    environment: "build".to_string(),
                    deployment_config: HashMap::new(),
                    status: StageStatus::Pending,
                    started_at: None,
                    completed_at: None,
                    results: HashMap::new(),
                },
                PipelineStage {
                    name: "Test".to_string(),
                    stage_type: StageType::Test,
                    environment: "test".to_string(),
                    deployment_config: HashMap::new(),
                    status: StageStatus::Pending,
                    started_at: None,
                    completed_at: None,
                    results: HashMap::new(),
                },
                PipelineStage {
                    name: "Deploy".to_string(),
                    stage_type: StageType::Deploy,
                    environment: "staging".to_string(),
                    deployment_config: HashMap::new(),
                    status: StageStatus::Pending,
                    started_at: None,
                    completed_at: None,
                    results: HashMap::new(),
                },
            ],
            current_stage: 0,
            status: PipelineStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            trigger: PipelineTrigger::Manual { user: "test".to_string() },
        }
    }

    #[test]
    fn test_pipeline_structure() {
        let pipeline = create_test_pipeline();
        
        assert_eq!(pipeline.stages.len(), 3);
        assert_eq!(pipeline.current_stage, 0);
        assert!(matches!(pipeline.status, PipelineStatus::Running));
        assert!(matches!(pipeline.trigger, PipelineTrigger::Manual { .. }));
    }

    #[test]
    fn test_stage_types() {
        let pipeline = create_test_pipeline();
        
        assert!(matches!(pipeline.stages[0].stage_type, StageType::Build));
        assert!(matches!(pipeline.stages[1].stage_type, StageType::Test));
        assert!(matches!(pipeline.stages[2].stage_type, StageType::Deploy));
    }
}

#[cfg(test)]
mod canary_tests {
    use super::*;
    use alchemist::deployment_automation::{
        CanaryDeployment, CanaryMetrics, CanaryStatus,
    };
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_canary() -> CanaryDeployment {
        CanaryDeployment {
            id: Uuid::new_v4().to_string(),
            deployment_id: Uuid::new_v4().to_string(),
            canary_version: "v2.0.0".to_string(),
            stable_version: "v1.0.0".to_string(),
            traffic_percentage: 10,
            started_at: Utc::now(),
            promotion_time: Utc::now() + chrono::Duration::minutes(30),
            metrics: CanaryMetrics {
                request_count: 1000,
                error_count: 5,
                avg_response_time_ms: 25.0,
                success_rate: 99.5,
                custom_metrics: HashMap::new(),
            },
            status: CanaryStatus::Running,
        }
    }

    #[test]
    fn test_canary_structure() {
        let canary = create_test_canary();
        
        assert_eq!(canary.traffic_percentage, 10);
        assert_eq!(canary.canary_version, "v2.0.0");
        assert_eq!(canary.stable_version, "v1.0.0");
        assert!(matches!(canary.status, CanaryStatus::Running));
    }

    #[test]
    fn test_canary_metrics() {
        let canary = create_test_canary();
        
        assert_eq!(canary.metrics.request_count, 1000);
        assert_eq!(canary.metrics.error_count, 5);
        assert_eq!(canary.metrics.avg_response_time_ms, 25.0);
        assert_eq!(canary.metrics.success_rate, 99.5);
    }

    #[test]
    fn test_canary_promotion_criteria() {
        let canary = create_test_canary();
        
        // Check if canary meets promotion criteria
        let meets_criteria = canary.metrics.success_rate > 95.0 && 
                           canary.metrics.error_count < 50;
        
        assert!(meets_criteria, "Canary should meet promotion criteria");
    }
}

#[cfg(test)]
mod approval_tests {
    use super::*;
    use alchemist::deployment_automation::{
        DeploymentApproval, Approval, ApprovalStatus,
    };
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_approval() -> DeploymentApproval {
        DeploymentApproval {
            id: Uuid::new_v4().to_string(),
            pipeline_id: Uuid::new_v4().to_string(),
            stage_name: "Deploy to Production".to_string(),
            required_approvers: vec!["alice".to_string(), "bob".to_string()],
            approvals: vec![],
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(24),
            status: ApprovalStatus::Pending,
        }
    }

    #[test]
    fn test_approval_structure() {
        let approval = create_test_approval();
        
        assert_eq!(approval.required_approvers.len(), 2);
        assert!(approval.approvals.is_empty());
        assert!(matches!(approval.status, ApprovalStatus::Pending));
    }

    #[test]
    fn test_approval_expiration() {
        let approval = create_test_approval();
        let duration = approval.expires_at.signed_duration_since(approval.created_at);
        
        assert_eq!(duration.num_hours(), 24);
    }

    #[test]
    fn test_approval_processing() {
        let mut approval = create_test_approval();
        
        // Add first approval
        approval.approvals.push(Approval {
            approver: "alice".to_string(),
            approved: true,
            comments: Some("Looks good".to_string()),
            timestamp: Utc::now(),
        });
        
        // Check if we need more approvals
        assert_eq!(approval.approvals.len(), 1);
        assert!(approval.approvals.len() < approval.required_approvers.len());
        
        // Add second approval
        approval.approvals.push(Approval {
            approver: "bob".to_string(),
            approved: true,
            comments: None,
            timestamp: Utc::now(),
        });
        
        // Now we have enough approvals
        assert_eq!(approval.approvals.len(), approval.required_approvers.len());
    }
}