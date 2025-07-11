//! Integration tests for policy evaluation engine

#[cfg(test)]
mod tests {
    use alchemist::policy::{Policy, PolicyManager, Rule, RuleCondition, RuleAction, Claim};
    use alchemist::policy_engine::{PolicyEngine, EvaluationContext, Subject, Resource, Action, Decision};
    use alchemist::config::{AlchemistConfig, GeneralConfig, PolicyConfig};
    use chrono::Utc;
    use std::collections::{HashMap, HashSet};
    use tempfile::TempDir;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_policy_evaluation_basic() {
        // Create test config
        let temp_dir = TempDir::new().unwrap();
        let config = AlchemistConfig {
            general: GeneralConfig {
                default_ai_model: None,
                dialog_history_path: temp_dir.path().join("dialogs").to_string_lossy().to_string(),
                progress_file_path: temp_dir.path().join("progress.json").to_string_lossy().to_string(),
                nats_url: None,
                log_level: "info".to_string(),
            },
            ai_models: HashMap::new(),
            policy: PolicyConfig {
                storage_path: temp_dir.path().to_string_lossy().to_string(),
                validation_enabled: true,
                evaluation_timeout: 1000,
                cache_ttl: Some(60),
            },
            deployments: HashMap::new(),
            domains: Default::default(),
        };
        
        let mut manager = PolicyManager::new(&config).await.unwrap();
        
        // Create a test policy
        let policy = Policy {
            id: Uuid::new_v4().to_string(),
            name: "Test Policy".to_string(),
            domain: "test-domain".to_string(),
            description: "Policy for testing".to_string(),
            rules: vec![
                Rule {
                    id: "rule1".to_string(),
                    condition: RuleCondition::HasClaim("admin".to_string()),
                    action: RuleAction::Allow,
                    priority: 100,
                },
                Rule {
                    id: "rule2".to_string(),
                    condition: RuleCondition::HasClaim("user".to_string()),
                    action: RuleAction::Allow,
                    priority: 50,
                },
                Rule {
                    id: "rule3".to_string(),
                    condition: RuleCondition::Always,
                    action: RuleAction::Deny,
                    priority: 1,
                },
            ],
            claims: vec!["admin".to_string(), "user".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            enabled: true,
        };
        
        // Save and load policy
        manager.create_policy(policy).await.unwrap();
        
        // Test evaluation with admin claim
        let decision = manager.evaluate(
            "user1".to_string(),
            "user".to_string(),
            vec!["admin".to_string()],
            "resource1".to_string(),
            "document".to_string(),
            "test-domain".to_string(),
            "read".to_string(),
        ).await.unwrap();
        
        assert_eq!(decision, Decision::Allow);
        
        // Test evaluation with user claim
        let decision = manager.evaluate(
            "user2".to_string(),
            "user".to_string(),
            vec!["user".to_string()],
            "resource1".to_string(),
            "document".to_string(),
            "test-domain".to_string(),
            "read".to_string(),
        ).await.unwrap();
        
        assert_eq!(decision, Decision::Allow);
        
        // Test evaluation with no claims
        let decision = manager.evaluate(
            "user3".to_string(),
            "user".to_string(),
            vec![],
            "resource1".to_string(),
            "document".to_string(),
            "test-domain".to_string(),
            "read".to_string(),
        ).await.unwrap();
        
        assert_eq!(decision, Decision::Deny);
    }
}