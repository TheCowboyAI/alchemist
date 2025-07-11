//! Tests for policy evaluation engine

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

#[tokio::test]
async fn test_policy_claims_permissions() {
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
    
    let manager = PolicyManager::new(&config).await.unwrap();
    
    // Add claims
    manager.add_claim(
        "admin".to_string(),
        "Administrator claim".to_string(),
        Some("test-domain".to_string()),
        vec!["read".to_string(), "write".to_string(), "delete".to_string()],
    ).await.unwrap();
    
    manager.add_claim(
        "user".to_string(),
        "Regular user claim".to_string(),
        Some("test-domain".to_string()),
        vec!["read".to_string()],
    ).await.unwrap();
    
    // Test permissions
    let admin_perms = manager.get_permissions_for_claims(&["admin".to_string()]);
    assert_eq!(admin_perms.len(), 3);
    assert!(admin_perms.contains(&"read".to_string()));
    assert!(admin_perms.contains(&"write".to_string()));
    assert!(admin_perms.contains(&"delete".to_string()));
    
    let user_perms = manager.get_permissions_for_claims(&["user".to_string()]);
    assert_eq!(user_perms.len(), 1);
    assert!(user_perms.contains(&"read".to_string()));
    
    // Test combined permissions
    let combined_perms = manager.get_permissions_for_claims(&["admin".to_string(), "user".to_string()]);
    assert_eq!(combined_perms.len(), 3); // Should be deduplicated
}

#[tokio::test]
async fn test_policy_engine_caching() {
    let engine = PolicyEngine::new(60); // 60 second cache
    
    let policy = Policy {
        id: "cache-test".to_string(),
        name: "Cache Test Policy".to_string(),
        domain: "test".to_string(),
        description: "Test caching".to_string(),
        rules: vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::HasClaim("cached".to_string()),
                action: RuleAction::Allow,
                priority: 10,
            },
        ],
        claims: vec!["cached".to_string()],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        enabled: true,
    };
    
    engine.load_policy(policy).unwrap();
    
    let context = EvaluationContext {
        subject: Subject {
            id: "user1".to_string(),
            subject_type: "user".to_string(),
            claims: HashSet::from(["cached".to_string()]),
            domains: HashSet::from(["test".to_string()]),
            attributes: HashMap::new(),
        },
        resource: Resource {
            id: "resource1".to_string(),
            resource_type: "document".to_string(),
            domain: "test".to_string(),
            attributes: HashMap::new(),
        },
        action: Action {
            name: "read".to_string(),
            action_type: "read".to_string(),
            parameters: HashMap::new(),
        },
        metadata: HashMap::new(),
        event: None,
    };
    
    // First evaluation
    let result1 = engine.evaluate(context.clone()).await.unwrap();
    assert_eq!(result1.decision, Decision::Allow);
    let duration1 = result1.duration_ms;
    
    // Second evaluation (should be cached)
    let result2 = engine.evaluate(context).await.unwrap();
    assert_eq!(result2.decision, Decision::Allow);
    let duration2 = result2.duration_ms;
    
    // Cached result should be faster
    assert!(duration2 < duration1 || duration2 == 0);
}

#[tokio::test]
async fn test_complex_rule_conditions() {
    let engine = PolicyEngine::new(60);
    
    let policy = Policy {
        id: "complex-rules".to_string(),
        name: "Complex Rules Policy".to_string(),
        domain: "test".to_string(),
        description: "Test complex conditions".to_string(),
        rules: vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::HasAllClaims(vec![
                    "role:admin".to_string(),
                    "scope:write".to_string(),
                ]),
                action: RuleAction::Allow,
                priority: 100,
            },
            Rule {
                id: "rule2".to_string(),
                condition: RuleCondition::HasAnyClaim(vec![
                    "role:user".to_string(),
                    "role:guest".to_string(),
                ]),
                action: RuleAction::RequireApproval,
                priority: 50,
            },
        ],
        claims: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        enabled: true,
    };
    
    engine.load_policy(policy).unwrap();
    
    // Test HasAllClaims - should allow
    let context1 = EvaluationContext {
        subject: Subject {
            id: "admin1".to_string(),
            subject_type: "user".to_string(),
            claims: HashSet::from([
                "role:admin".to_string(),
                "scope:write".to_string(),
                "other:claim".to_string(),
            ]),
            domains: HashSet::from(["test".to_string()]),
            attributes: HashMap::new(),
        },
        resource: Resource {
            id: "resource1".to_string(),
            resource_type: "document".to_string(),
            domain: "test".to_string(),
            attributes: HashMap::new(),
        },
        action: Action {
            name: "write".to_string(),
            action_type: "write".to_string(),
            parameters: HashMap::new(),
        },
        metadata: HashMap::new(),
        event: None,
    };
    
    let result1 = engine.evaluate(context1).await.unwrap();
    assert_eq!(result1.decision, Decision::Allow);
    
    // Test HasAnyClaim - should require approval
    let context2 = EvaluationContext {
        subject: Subject {
            id: "user1".to_string(),
            subject_type: "user".to_string(),
            claims: HashSet::from(["role:user".to_string()]),
            domains: HashSet::from(["test".to_string()]),
            attributes: HashMap::new(),
        },
        resource: Resource {
            id: "resource1".to_string(),
            resource_type: "document".to_string(),
            domain: "test".to_string(),
            attributes: HashMap::new(),
        },
        action: Action {
            name: "read".to_string(),
            action_type: "read".to_string(),
            parameters: HashMap::new(),
        },
        metadata: HashMap::new(),
        event: None,
    };
    
    let result2 = engine.evaluate(context2).await.unwrap();
    assert_eq!(result2.decision, Decision::RequireApproval);
}