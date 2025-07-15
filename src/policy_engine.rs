//! Policy evaluation engine
//!
//! This module provides the core policy evaluation functionality,
//! supporting event-based policy decisions with claims-based authorization.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc, Timelike, Datelike};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{info, warn};

use crate::policy::{Policy, RuleCondition, RuleAction};

/// Context for policy evaluation
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Subject making the request (user, service, etc.)
    pub subject: Subject,
    /// Resource being accessed
    pub resource: Resource,
    /// Action being performed
    pub action: Action,
    /// Additional context data
    pub metadata: HashMap<String, serde_json::Value>,
    /// Event that triggered the evaluation (if any)
    pub event: Option<Event>,
}

/// Subject of a policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    /// Unique identifier
    pub id: String,
    /// Type of subject (user, service, agent, etc.)
    pub subject_type: String,
    /// Claims held by the subject
    pub claims: HashSet<String>,
    /// Domains the subject belongs to
    pub domains: HashSet<String>,
    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

/// Resource being accessed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource identifier
    pub id: String,
    /// Resource type
    pub resource_type: String,
    /// Domain the resource belongs to
    pub domain: String,
    /// Resource attributes
    pub attributes: HashMap<String, String>,
}

/// Action being performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Action name
    pub name: String,
    /// Action type (read, write, execute, etc.)
    pub action_type: String,
    /// Additional parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Event that may trigger policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event ID
    pub id: String,
    /// Event type
    pub event_type: String,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event data
    pub data: serde_json::Value,
}

/// Result of policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// Overall decision
    pub decision: Decision,
    /// Policies that were evaluated
    pub evaluated_policies: Vec<String>,
    /// Rules that matched
    pub matched_rules: Vec<MatchedRule>,
    /// Evaluation timestamp
    pub timestamp: DateTime<Utc>,
    /// Evaluation duration in milliseconds
    pub duration_ms: u64,
    /// Additional obligations to fulfill
    pub obligations: Vec<Obligation>,
}

/// Policy decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    /// Action is allowed
    Allow,
    /// Action is denied
    Deny,
    /// Action requires additional approval
    RequireApproval,
    /// No applicable policy found
    NotApplicable,
}

/// A rule that matched during evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedRule {
    /// Policy ID
    pub policy_id: String,
    /// Rule ID
    pub rule_id: String,
    /// Rule priority
    pub priority: u32,
    /// Action taken
    pub action: RuleAction,
}

/// Obligation that must be fulfilled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obligation {
    /// Obligation type
    pub obligation_type: String,
    /// Required parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Trait for custom condition evaluators
#[async_trait]
pub trait ConditionEvaluator: Send + Sync {
    /// Evaluate a custom condition
    async fn evaluate(
        &self,
        condition: &str,
        context: &EvaluationContext,
    ) -> Result<bool>;
}

/// Policy evaluation engine
pub struct PolicyEngine {
    /// Loaded policies by ID
    policies: Arc<DashMap<String, Policy>>,
    /// Policies indexed by domain
    domain_policies: Arc<DashMap<String, Vec<String>>>,
    /// Custom condition evaluators
    condition_evaluators: Arc<DashMap<String, Box<dyn ConditionEvaluator>>>,
    /// Evaluation cache
    cache: Arc<DashMap<String, CachedResult>>,
    /// Cache TTL in seconds
    cache_ttl: u64,
}

/// Cached evaluation result
#[derive(Clone)]
struct CachedResult {
    result: EvaluationResult,
    expires_at: DateTime<Utc>,
}

impl PolicyEngine {
    /// Create a new policy engine
    pub fn new(cache_ttl: u64) -> Self {
        Self {
            policies: Arc::new(DashMap::new()),
            domain_policies: Arc::new(DashMap::new()),
            condition_evaluators: Arc::new(DashMap::new()),
            cache: Arc::new(DashMap::new()),
            cache_ttl,
        }
    }
    
    /// Load a policy into the engine
    pub fn load_policy(&self, policy: Policy) -> Result<()> {
        let policy_id = policy.id.clone();
        let domain = policy.domain.clone();
        
        // Add to main storage
        self.policies.insert(policy_id.clone(), policy);
        
        // Update domain index
        self.domain_policies
            .entry(domain.clone())
            .or_insert_with(Vec::new)
            .push(policy_id);
        
        // Clear cache for this domain
        self.clear_domain_cache(&domain);
        
        Ok(())
    }
    
    /// Unload a policy from the engine
    pub fn unload_policy(&self, policy_id: &str) -> Result<()> {
        if let Some((_, policy)) = self.policies.remove(policy_id) {
            // Remove from domain index
            if let Some(mut domain_policies) = self.domain_policies.get_mut(&policy.domain) {
                domain_policies.retain(|id| id != policy_id);
            }
            
            // Clear cache for this domain
            self.clear_domain_cache(&policy.domain);
        }
        
        Ok(())
    }
    
    /// Register a custom condition evaluator
    pub fn register_evaluator(
        &self,
        name: String,
        evaluator: Box<dyn ConditionEvaluator>,
    ) {
        self.condition_evaluators.insert(name, evaluator);
    }
    
    /// Evaluate policies for a given context
    pub async fn evaluate(&self, context: EvaluationContext) -> Result<EvaluationResult> {
        let start_time = std::time::Instant::now();
        
        // Check if subject has any claims at all
        if context.subject.claims.is_empty() {
            return Err(crate::error::AlchemistError::PermissionDenied(
                format!("Subject {} has no claims to access resource", context.subject.id)
            ).into());
        }
        
        // Check cache first
        let cache_key = self.compute_cache_key(&context);
        if let Some(cached) = self.get_cached_result(&cache_key) {
            return Ok(cached);
        }
        
        // Find applicable policies
        let applicable_policies = self.find_applicable_policies(&context)?;
        
        // Evaluate each policy
        let mut matched_rules = Vec::new();
        let mut evaluated_policies = Vec::new();
        let mut obligations = Vec::new();
        
        for policy_id in &applicable_policies {
            if let Some(policy) = self.policies.get(policy_id) {
                if !policy.enabled {
                    continue;
                }
                
                evaluated_policies.push(policy_id.clone());
                
                // Evaluate rules in priority order
                let mut sorted_rules = policy.rules.clone();
                sorted_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
                
                for rule in &sorted_rules {
                    if self.evaluate_condition(&rule.condition, &context).await? {
                        matched_rules.push(MatchedRule {
                            policy_id: policy_id.clone(),
                            rule_id: rule.id.clone(),
                            priority: rule.priority,
                            action: rule.action.clone(),
                        });
                        
                        // Process rule action
                        match &rule.action {
                            RuleAction::Log => {
                                info!(
                                    "Policy {} rule {} matched for subject {}",
                                    policy_id, rule.id, context.subject.id
                                );
                            }
                            RuleAction::Transform(params) => {
                                obligations.push(Obligation {
                                    obligation_type: "transform".to_string(),
                                    parameters: serde_json::from_str(params)?,
                                });
                            }
                            RuleAction::Delegate(target) => {
                                obligations.push(Obligation {
                                    obligation_type: "delegate".to_string(),
                                    parameters: HashMap::from([
                                        ("target".to_string(), serde_json::Value::String(target.clone()))
                                    ]),
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        // Determine final decision
        let decision = self.determine_decision(&matched_rules);
        
        let result = EvaluationResult {
            decision,
            evaluated_policies,
            matched_rules,
            timestamp: Utc::now(),
            duration_ms: start_time.elapsed().as_millis() as u64,
            obligations,
        };
        
        // Cache the result
        self.cache_result(cache_key, result.clone());
        
        Ok(result)
    }
    
    /// Find policies applicable to the context
    fn find_applicable_policies(&self, context: &EvaluationContext) -> Result<Vec<String>> {
        let mut policies = HashSet::new();
        
        // Add policies for resource domain
        if let Some(domain_policies) = self.domain_policies.get(&context.resource.domain) {
            policies.extend(domain_policies.iter().cloned());
        }
        
        // Add policies for subject domains
        for domain in &context.subject.domains {
            if let Some(domain_policies) = self.domain_policies.get(domain) {
                policies.extend(domain_policies.iter().cloned());
            }
        }
        
        // Add global policies (domain = "*")
        if let Some(global_policies) = self.domain_policies.get("*") {
            policies.extend(global_policies.iter().cloned());
        }
        
        Ok(policies.into_iter().collect())
    }
    
    /// Evaluate a rule condition
    async fn evaluate_condition(
        &self,
        condition: &RuleCondition,
        context: &EvaluationContext,
    ) -> Result<bool> {
        match condition {
            RuleCondition::Always => Ok(true),
            
            RuleCondition::HasClaim(claim) => {
                Ok(context.subject.claims.contains(claim))
            }
            
            RuleCondition::HasAllClaims(claims) => {
                Ok(claims.iter().all(|c| context.subject.claims.contains(c)))
            }
            
            RuleCondition::HasAnyClaim(claims) => {
                Ok(claims.iter().any(|c| context.subject.claims.contains(c)))
            }
            
            RuleCondition::DomainIs(domain) => {
                Ok(context.resource.domain == *domain || 
                   context.subject.domains.contains(domain))
            }
            
            RuleCondition::EventType(event_type) => {
                if let Some(event) = &context.event {
                    Ok(event.event_type == *event_type)
                } else {
                    Ok(false)
                }
            }
            
            RuleCondition::Custom(expr) => {
                // Try to find a registered evaluator
                let evaluator_name = expr.split(':').next().unwrap_or("");
                
                if let Some(evaluator) = self.condition_evaluators.get(evaluator_name) {
                    evaluator.evaluate(expr, context).await
                } else {
                    warn!("No evaluator found for custom condition: {}", expr);
                    Ok(false)
                }
            }
        }
    }
    
    /// Determine final decision based on matched rules
    fn determine_decision(&self, matched_rules: &[MatchedRule]) -> Decision {
        if matched_rules.is_empty() {
            return Decision::NotApplicable;
        }
        
        // Sort by priority (highest first)
        let mut sorted_rules = matched_rules.to_vec();
        sorted_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        // Apply first matching rule with a decision action
        for rule in &sorted_rules {
            match &rule.action {
                RuleAction::Allow => return Decision::Allow,
                RuleAction::Deny => return Decision::Deny,
                RuleAction::RequireApproval => return Decision::RequireApproval,
                _ => continue,
            }
        }
        
        // Default to deny if no explicit decision
        Decision::Deny
    }
    
    /// Compute cache key for a context
    fn compute_cache_key(&self, context: &EvaluationContext) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        context.subject.id.hash(&mut hasher);
        context.resource.id.hash(&mut hasher);
        context.action.name.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }
    
    /// Get cached result if valid
    fn get_cached_result(&self, key: &str) -> Option<EvaluationResult> {
        if let Some(cached) = self.cache.get(key) {
            if cached.expires_at > Utc::now() {
                return Some(cached.result.clone());
            }
        }
        None
    }
    
    /// Cache an evaluation result
    fn cache_result(&self, key: String, result: EvaluationResult) {
        let cached = CachedResult {
            result,
            expires_at: Utc::now() + chrono::Duration::seconds(self.cache_ttl as i64),
        };
        self.cache.insert(key, cached);
    }
    
    /// Clear cache for a specific domain
    fn clear_domain_cache(&self, _domain: &str) {
        // For now, clear entire cache
        // TODO: Implement domain-specific cache clearing
        self.cache.clear();
    }
}

/// Example custom condition evaluator for time-based rules
pub struct TimeBasedEvaluator;

#[async_trait]
impl ConditionEvaluator for TimeBasedEvaluator {
    async fn evaluate(
        &self,
        condition: &str,
        _context: &EvaluationContext,
    ) -> Result<bool> {
        // Parse condition like "time:business_hours"
        if let Some(time_condition) = condition.strip_prefix("time:") {
            match time_condition {
                "business_hours" => {
                    let now = chrono::Local::now();
                    let hour = now.hour();
                    Ok(hour >= 9 && hour < 17)
                }
                "weekend" => {
                    let now = chrono::Local::now();
                    let weekday = now.weekday();
                    Ok(weekday == chrono::Weekday::Sat || 
                       weekday == chrono::Weekday::Sun)
                }
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::Rule;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tokio::sync::Mutex;
    use std::time::Duration;
    use futures::future;
    
    // Helper function to create a test policy
    fn create_test_policy(id: &str, domain: &str, rules: Vec<Rule>) -> Policy {
        Policy {
            id: id.to_string(),
            name: format!("Test Policy {}", id),
            domain: domain.to_string(),
            description: "Test policy".to_string(),
            rules,
            claims: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            enabled: true,
        }
    }
    
    // Helper function to create a test subject
    fn create_test_subject(id: &str, claims: Vec<String>, domains: Vec<String>) -> Subject {
        Subject {
            id: id.to_string(),
            subject_type: "user".to_string(),
            claims: claims.into_iter().collect(),
            domains: domains.into_iter().collect(),
            attributes: HashMap::new(),
        }
    }
    
    // Helper function to create a test resource
    fn create_test_resource(id: &str, domain: &str) -> Resource {
        Resource {
            id: id.to_string(),
            resource_type: "document".to_string(),
            domain: domain.to_string(),
            attributes: HashMap::new(),
        }
    }
    
    // Helper function to create a test action
    fn create_test_action(name: &str) -> Action {
        Action {
            name: name.to_string(),
            action_type: name.to_string(),
            parameters: HashMap::new(),
        }
    }
    
    // Helper function to create an evaluation context
    fn create_test_context(subject: Subject, resource: Resource, action: Action) -> EvaluationContext {
        EvaluationContext {
            subject,
            resource,
            action,
            metadata: HashMap::new(),
            event: None,
        }
    }

    // 1. Policy engine initialization tests
    #[test]
    fn test_policy_engine_initialization() {
        let engine = PolicyEngine::new(60);
        assert_eq!(engine.cache_ttl, 60);
        assert_eq!(engine.policies.len(), 0);
        assert_eq!(engine.domain_policies.len(), 0);
        assert_eq!(engine.condition_evaluators.len(), 0);
        assert_eq!(engine.cache.len(), 0);
    }
    
    #[test]
    fn test_policy_engine_with_different_ttl() {
        let engine = PolicyEngine::new(300);
        assert_eq!(engine.cache_ttl, 300);
    }

    // 2. Policy CRUD operations tests
    #[test]
    fn test_policy_add() {
        let engine = PolicyEngine::new(60);
        let policy = create_test_policy("policy1", "domain1", vec![]);
        
        assert!(engine.load_policy(policy.clone()).is_ok());
        assert_eq!(engine.policies.len(), 1);
        assert!(engine.policies.contains_key("policy1"));
        
        let domain_policies = engine.domain_policies.get("domain1").unwrap();
        assert_eq!(domain_policies.len(), 1);
        assert_eq!(domain_policies[0], "policy1");
    }
    
    #[test]
    fn test_policy_update() {
        let engine = PolicyEngine::new(60);
        let policy1 = create_test_policy("policy1", "domain1", vec![]);
        let policy2 = create_test_policy("policy1", "domain1", vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Allow,
                priority: 100,
            }
        ]);
        
        engine.load_policy(policy1).unwrap();
        engine.load_policy(policy2).unwrap();
        
        let stored_policy = engine.policies.get("policy1").unwrap();
        assert_eq!(stored_policy.rules.len(), 1);
    }
    
    #[test]
    fn test_policy_remove() {
        let engine = PolicyEngine::new(60);
        let policy = create_test_policy("policy1", "domain1", vec![]);
        
        engine.load_policy(policy).unwrap();
        assert_eq!(engine.policies.len(), 1);
        
        engine.unload_policy("policy1").unwrap();
        assert_eq!(engine.policies.len(), 0);
        assert!(!engine.policies.contains_key("policy1"));
        
        let domain_policies = engine.domain_policies.get("domain1");
        assert!(domain_policies.is_none() || domain_policies.unwrap().is_empty());
    }
    
    #[test]
    fn test_policy_list() {
        let engine = PolicyEngine::new(60);
        
        for i in 1..=5 {
            let policy = create_test_policy(&format!("policy{}", i), "domain1", vec![]);
            engine.load_policy(policy).unwrap();
        }
        
        assert_eq!(engine.policies.len(), 5);
        for i in 1..=5 {
            assert!(engine.policies.contains_key(&format!("policy{}", i)));
        }
    }

    // 3. Policy evaluation with different conditions
    #[tokio::test]
    async fn test_has_claim_condition() {
        let engine = PolicyEngine::new(60);
        
        let policy = create_test_policy("policy1", "test", vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::HasClaim("admin".to_string()),
                action: RuleAction::Allow,
                priority: 100,
            },
            Rule {
                id: "rule2".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Deny,
                priority: 1,
            },
        ]);
        
        engine.load_policy(policy).unwrap();
        
        // Test with admin claim
        let context = create_test_context(
            create_test_subject("user1", vec!["admin".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("read"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::Allow);
        
        // Test without admin claim
        let context = create_test_context(
            create_test_subject("user2", vec!["user".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("read"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::Deny);
    }
    
    #[tokio::test]
    async fn test_has_all_claims_condition() {
        let engine = PolicyEngine::new(60);
        
        let policy = create_test_policy("policy1", "test", vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::HasAllClaims(vec!["read".to_string(), "write".to_string()]),
                action: RuleAction::Allow,
                priority: 100,
            },
        ]);
        
        engine.load_policy(policy).unwrap();
        
        // Test with all required claims
        let context = create_test_context(
            create_test_subject("user1", vec!["read".to_string(), "write".to_string(), "delete".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("modify"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::Allow);
        
        // Test with only some claims
        let context = create_test_context(
            create_test_subject("user2", vec!["read".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("modify"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::NotApplicable);
    }
    
    #[tokio::test]
    async fn test_has_any_claim_condition() {
        let engine = PolicyEngine::new(60);
        
        let policy = create_test_policy("policy1", "test", vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::HasAnyClaim(vec!["admin".to_string(), "moderator".to_string()]),
                action: RuleAction::Allow,
                priority: 100,
            },
        ]);
        
        engine.load_policy(policy).unwrap();
        
        // Test with one of the claims
        let context = create_test_context(
            create_test_subject("user1", vec!["moderator".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("moderate"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::Allow);
        
        // Test with none of the claims
        let context = create_test_context(
            create_test_subject("user2", vec!["user".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("moderate"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::NotApplicable);
    }
    
    #[tokio::test]
    async fn test_domain_is_condition() {
        let engine = PolicyEngine::new(60);
        
        let policy = create_test_policy("policy1", "domain1", vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::DomainIs("domain1".to_string()),
                action: RuleAction::Allow,
                priority: 100,
            },
        ]);
        
        engine.load_policy(policy).unwrap();
        
        // Test with matching domain
        let context = create_test_context(
            create_test_subject("user1", vec![], vec!["domain1".to_string()]),
            create_test_resource("resource1", "domain1"),
            create_test_action("read"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::Allow);
        
        // Test with different domain
        let context = create_test_context(
            create_test_subject("user2", vec![], vec!["domain2".to_string()]),
            create_test_resource("resource2", "domain2"),
            create_test_action("read"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::NotApplicable);
    }
    
    #[tokio::test]
    async fn test_event_type_condition() {
        let engine = PolicyEngine::new(60);
        
        let policy = create_test_policy("policy1", "test", vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::EventType("user.login".to_string()),
                action: RuleAction::Log,
                priority: 100,
            },
        ]);
        
        engine.load_policy(policy).unwrap();
        
        // Test with matching event
        let mut context = create_test_context(
            create_test_subject("user1", vec![], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("audit"),
        );
        
        context.event = Some(Event {
            id: "event1".to_string(),
            event_type: "user.login".to_string(),
            timestamp: Utc::now(),
            data: serde_json::Value::Null,
        });
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.matched_rules.len(), 1);
        
        // Test without event
        let context = create_test_context(
            create_test_subject("user2", vec![], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("audit"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.matched_rules.len(), 0);
    }
    
    #[tokio::test]
    async fn test_custom_condition() {
        let engine = PolicyEngine::new(60);
        
        // Register custom evaluator
        struct TestEvaluator;
        
        #[async_trait]
        impl ConditionEvaluator for TestEvaluator {
            async fn evaluate(
                &self,
                condition: &str,
                context: &EvaluationContext,
            ) -> Result<bool> {
                Ok(condition == "test:pass" && context.subject.id == "user1")
            }
        }
        
        engine.register_evaluator("test".to_string(), Box::new(TestEvaluator));
        
        let policy = create_test_policy("policy1", "test", vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::Custom("test:pass".to_string()),
                action: RuleAction::Allow,
                priority: 100,
            },
        ]);
        
        engine.load_policy(policy).unwrap();
        
        // Test with matching custom condition
        let context = create_test_context(
            create_test_subject("user1", vec![], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("custom"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::Allow);
        
        // Test with non-matching custom condition
        let context = create_test_context(
            create_test_subject("user2", vec![], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("custom"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::NotApplicable);
    }

    // 4. Cache functionality and TTL tests
    #[tokio::test]
    async fn test_cache_hit() {
        let engine = PolicyEngine::new(60);
        
        let policy = create_test_policy("policy1", "test", vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Allow,
                priority: 100,
            },
        ]);
        
        engine.load_policy(policy).unwrap();
        
        let context = create_test_context(
            create_test_subject("user1", vec!["read".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("read"),
        );
        
        // First evaluation - cache miss
        let result1 = engine.evaluate(context.clone()).await.unwrap();
        assert_eq!(result1.decision, Decision::Allow);
        
        // Second evaluation - should be cache hit
        let result2 = engine.evaluate(context).await.unwrap();
        assert_eq!(result2.decision, Decision::Allow);
        assert_eq!(result1.timestamp, result2.timestamp); // Same timestamp indicates cache hit
    }
    
    #[tokio::test]
    async fn test_cache_ttl_expiry() {
        let engine = PolicyEngine::new(1); // 1 second TTL
        
        let policy = create_test_policy("policy1", "test", vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Allow,
                priority: 100,
            },
        ]);
        
        engine.load_policy(policy).unwrap();
        
        let context = create_test_context(
            create_test_subject("user1", vec!["read".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("read"),
        );
        
        // First evaluation
        let result1 = engine.evaluate(context.clone()).await.unwrap();
        
        // Wait for cache to expire
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Second evaluation - should be cache miss
        let result2 = engine.evaluate(context).await.unwrap();
        assert_eq!(result2.decision, Decision::Allow);
        assert_ne!(result1.timestamp, result2.timestamp); // Different timestamps indicate cache miss
    }
    
    #[tokio::test]
    async fn test_cache_invalidation_on_policy_change() {
        let engine = PolicyEngine::new(60);
        
        let policy = create_test_policy("policy1", "test", vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Allow,
                priority: 100,
            },
        ]);
        
        engine.load_policy(policy).unwrap();
        
        let context = create_test_context(
            create_test_subject("user1", vec!["read".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("read"),
        );
        
        // First evaluation
        let result1 = engine.evaluate(context.clone()).await.unwrap();
        assert_eq!(result1.decision, Decision::Allow);
        
        // Update policy - should clear cache
        let new_policy = create_test_policy("policy1", "test", vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Deny,
                priority: 100,
            },
        ]);
        
        engine.load_policy(new_policy).unwrap();
        
        // Second evaluation - should reflect new policy
        let result2 = engine.evaluate(context).await.unwrap();
        assert_eq!(result2.decision, Decision::Deny);
    }

    // 5. Domain-specific policy association tests
    #[tokio::test]
    async fn test_domain_specific_policies() {
        let engine = PolicyEngine::new(60);
        
        // Load policies for different domains
        let policy1 = create_test_policy("policy1", "domain1", vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Allow,
                priority: 100,
            },
        ]);
        
        let policy2 = create_test_policy("policy2", "domain2", vec![
            Rule {
                id: "rule2".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Deny,
                priority: 100,
            },
        ]);
        
        engine.load_policy(policy1).unwrap();
        engine.load_policy(policy2).unwrap();
        
        // Test domain1 resource
        let context = create_test_context(
            create_test_subject("user1", vec![], vec!["domain1".to_string()]),
            create_test_resource("resource1", "domain1"),
            create_test_action("read"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::Allow);
        assert!(result.evaluated_policies.contains(&"policy1".to_string()));
        assert!(!result.evaluated_policies.contains(&"policy2".to_string()));
    }
    
    #[tokio::test]
    async fn test_global_policies() {
        let engine = PolicyEngine::new(60);
        
        // Load global policy (domain = "*")
        let global_policy = create_test_policy("global", "*", vec![
            Rule {
                id: "global_rule".to_string(),
                condition: RuleCondition::HasClaim("global_admin".to_string()),
                action: RuleAction::Allow,
                priority: 200,
            },
        ]);
        
        let domain_policy = create_test_policy("domain", "test", vec![
            Rule {
                id: "domain_rule".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Deny,
                priority: 100,
            },
        ]);
        
        engine.load_policy(global_policy).unwrap();
        engine.load_policy(domain_policy).unwrap();
        
        // Test with global admin claim
        let context = create_test_context(
            create_test_subject("user1", vec!["global_admin".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("read"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::Allow); // Global policy takes precedence due to higher priority
    }

    // 6. Bulk policy operations tests
    #[test]
    fn test_bulk_policy_load() {
        let engine = PolicyEngine::new(60);
        
        let policies: Vec<Policy> = (1..=10)
            .map(|i| create_test_policy(&format!("policy{}", i), "bulk_test", vec![]))
            .collect();
        
        for policy in policies {
            assert!(engine.load_policy(policy).is_ok());
        }
        
        assert_eq!(engine.policies.len(), 10);
        let domain_policies = engine.domain_policies.get("bulk_test").unwrap();
        assert_eq!(domain_policies.len(), 10);
    }
    
    #[test]
    fn test_bulk_policy_unload() {
        let engine = PolicyEngine::new(60);
        
        // Load multiple policies
        for i in 1..=5 {
            let policy = create_test_policy(&format!("policy{}", i), "test", vec![]);
            engine.load_policy(policy).unwrap();
        }
        
        // Unload all policies
        for i in 1..=5 {
            engine.unload_policy(&format!("policy{}", i)).unwrap();
        }
        
        assert_eq!(engine.policies.len(), 0);
    }

    // 7. Policy validation tests
    #[tokio::test]
    async fn test_disabled_policy() {
        let engine = PolicyEngine::new(60);
        
        let mut policy = create_test_policy("policy1", "test", vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Allow,
                priority: 100,
            },
        ]);
        policy.enabled = false;
        
        engine.load_policy(policy).unwrap();
        
        let context = create_test_context(
            create_test_subject("user1", vec!["read".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("read"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::NotApplicable);
        assert!(result.evaluated_policies.is_empty());
    }
    
    #[tokio::test]
    async fn test_empty_rules_policy() {
        let engine = PolicyEngine::new(60);
        
        let policy = create_test_policy("policy1", "test", vec![]);
        engine.load_policy(policy).unwrap();
        
        let context = create_test_context(
            create_test_subject("user1", vec!["read".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("read"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::NotApplicable);
    }

    // 8. Concurrent policy evaluation tests
    #[tokio::test]
    async fn test_concurrent_evaluation() {
        let engine = Arc::new(PolicyEngine::new(60));
        
        let policy = create_test_policy("policy1", "test", vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Allow,
                priority: 100,
            },
        ]);
        
        engine.load_policy(policy).unwrap();
        
        let mut handles = vec![];
        
        // Spawn 10 concurrent evaluations
        for i in 0..10 {
            let engine_clone = engine.clone();
            let handle = tokio::spawn(async move {
                let context = create_test_context(
                    create_test_subject(&format!("user{}", i), vec![], vec!["test".to_string()]),
                    create_test_resource(&format!("resource{}", i), "test"),
                    create_test_action("read"),
                );
                
                engine_clone.evaluate(context).await
            });
            handles.push(handle);
        }
        
        // Wait for all evaluations to complete
        let results: Vec<Result<EvaluationResult>> = future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();
        
        // Verify all evaluations succeeded
        for result in results {
            let evaluation = result.unwrap();
            assert_eq!(evaluation.decision, Decision::Allow);
        }
    }
    
    #[tokio::test]
    async fn test_concurrent_policy_modifications() {
        let engine = Arc::new(PolicyEngine::new(60));
        
        let mut handles = vec![];
        
        // Concurrent policy additions
        for i in 0..5 {
            let engine_clone = engine.clone();
            let handle = tokio::spawn(async move {
                let policy = create_test_policy(
                    &format!("policy{}", i),
                    &format!("domain{}", i),
                    vec![],
                );
                engine_clone.load_policy(policy)
            });
            handles.push(handle);
        }
        
        // Wait for all operations
        let results: Vec<Result<()>> = future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();
        
        // Verify all operations succeeded
        for result in results {
            assert!(result.is_ok());
        }
        
        assert_eq!(engine.policies.len(), 5);
    }

    // 9. Custom condition evaluator registration tests
    #[tokio::test]
    async fn test_custom_evaluator_registration() {
        let engine = PolicyEngine::new(60);
        
        // Counter to verify evaluator is called
        let counter = Arc::new(AtomicBool::new(false));
        let counter_clone = counter.clone();
        
        struct CountingEvaluator {
            counter: Arc<AtomicBool>,
        }
        
        #[async_trait]
        impl ConditionEvaluator for CountingEvaluator {
            async fn evaluate(
                &self,
                _condition: &str,
                _context: &EvaluationContext,
            ) -> Result<bool> {
                self.counter.store(true, Ordering::SeqCst);
                Ok(true)
            }
        }
        
        engine.register_evaluator(
            "counter".to_string(),
            Box::new(CountingEvaluator { counter: counter_clone }),
        );
        
        let policy = create_test_policy("policy1", "test", vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::Custom("counter:test".to_string()),
                action: RuleAction::Allow,
                priority: 100,
            },
        ]);
        
        engine.load_policy(policy).unwrap();
        
        let context = create_test_context(
            create_test_subject("user1", vec!["read".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("read"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::Allow);
        assert!(counter.load(Ordering::SeqCst));
    }
    
    #[tokio::test]
    async fn test_missing_custom_evaluator() {
        let engine = PolicyEngine::new(60);
        
        let policy = create_test_policy("policy1", "test", vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::Custom("missing:evaluator".to_string()),
                action: RuleAction::Allow,
                priority: 100,
            },
        ]);
        
        engine.load_policy(policy).unwrap();
        
        let context = create_test_context(
            create_test_subject("user1", vec!["read".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("read"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::NotApplicable); // Rule doesn't match because evaluator is missing
    }

    // 10. Error handling scenarios tests
    #[tokio::test]
    async fn test_evaluation_with_no_policies() {
        let engine = PolicyEngine::new(60);
        
        let context = create_test_context(
            create_test_subject("user1", vec!["read".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("read"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::NotApplicable);
        assert!(result.evaluated_policies.is_empty());
        assert!(result.matched_rules.is_empty());
    }
    
    #[tokio::test]
    async fn test_custom_evaluator_error() {
        let engine = PolicyEngine::new(60);
        
        struct ErrorEvaluator;
        
        #[async_trait]
        impl ConditionEvaluator for ErrorEvaluator {
            async fn evaluate(
                &self,
                _condition: &str,
                _context: &EvaluationContext,
            ) -> Result<bool> {
                Err(anyhow::anyhow!("Evaluation error"))
            }
        }
        
        engine.register_evaluator("error".to_string(), Box::new(ErrorEvaluator));
        
        let policy = create_test_policy("policy1", "test", vec![
            Rule {
                id: "rule1".to_string(),
                condition: RuleCondition::Custom("error:test".to_string()),
                action: RuleAction::Allow,
                priority: 100,
            },
        ]);
        
        engine.load_policy(policy).unwrap();
        
        let context = create_test_context(
            create_test_subject("user1", vec!["read".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("read"),
        );
        
        // Should fail with error from evaluator
        let result = engine.evaluate(context).await;
        assert!(result.is_err());
    }

    // Additional tests for complex scenarios
    #[tokio::test]
    async fn test_priority_ordering() {
        let engine = PolicyEngine::new(60);
        
        let policy = create_test_policy("policy1", "test", vec![
            Rule {
                id: "low_priority_allow".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Allow,
                priority: 10,
            },
            Rule {
                id: "high_priority_deny".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Deny,
                priority: 100,
            },
        ]);
        
        engine.load_policy(policy).unwrap();
        
        let context = create_test_context(
            create_test_subject("user1", vec!["read".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("read"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::Deny); // Higher priority rule wins
        assert_eq!(result.matched_rules[0].rule_id, "high_priority_deny");
    }
    
    #[tokio::test]
    async fn test_multiple_effects() {
        let engine = PolicyEngine::new(60);
        
        let policy = create_test_policy("policy1", "test", vec![
            Rule {
                id: "log_rule".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Log,
                priority: 100,
            },
            Rule {
                id: "transform_rule".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Transform("{\"key\": \"value\"}".to_string()),
                priority: 90,
            },
            Rule {
                id: "delegate_rule".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Delegate("another_service".to_string()),
                priority: 80,
            },
            Rule {
                id: "allow_rule".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Allow,
                priority: 70,
            },
        ]);
        
        engine.load_policy(policy).unwrap();
        
        let context = create_test_context(
            create_test_subject("user1", vec!["read".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("read"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::Allow);
        assert_eq!(result.matched_rules.len(), 4);
        assert_eq!(result.obligations.len(), 2); // Transform and Delegate create obligations
    }
    
    #[tokio::test]
    async fn test_require_approval_effect() {
        let engine = PolicyEngine::new(60);
        
        let policy = create_test_policy("policy1", "test", vec![
            Rule {
                id: "approval_rule".to_string(),
                condition: RuleCondition::HasClaim("sensitive_operation".to_string()),
                action: RuleAction::RequireApproval,
                priority: 100,
            },
        ]);
        
        engine.load_policy(policy).unwrap();
        
        let context = create_test_context(
            create_test_subject("user1", vec!["sensitive_operation".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("delete"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        assert_eq!(result.decision, Decision::RequireApproval);
    }
    
    #[tokio::test]
    async fn test_time_based_evaluator() {
        let engine = PolicyEngine::new(60);
        engine.register_evaluator("time".to_string(), Box::new(TimeBasedEvaluator));
        
        let policy = create_test_policy("policy1", "test", vec![
            Rule {
                id: "business_hours_rule".to_string(),
                condition: RuleCondition::Custom("time:business_hours".to_string()),
                action: RuleAction::Allow,
                priority: 100,
            },
            Rule {
                id: "weekend_rule".to_string(),
                condition: RuleCondition::Custom("time:weekend".to_string()),
                action: RuleAction::Deny,
                priority: 90,
            },
        ]);
        
        engine.load_policy(policy).unwrap();
        
        let context = create_test_context(
            create_test_subject("user1", vec!["read".to_string()], vec!["test".to_string()]),
            create_test_resource("resource1", "test"),
            create_test_action("read"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        // Result depends on current time - just verify evaluation completes
        assert!(result.decision == Decision::Allow || 
                result.decision == Decision::Deny || 
                result.decision == Decision::NotApplicable);
    }
    
    #[tokio::test]
    async fn test_subject_cross_domain_access() {
        let engine = PolicyEngine::new(60);
        
        // Policies in different domains
        let policy1 = create_test_policy("policy1", "domain1", vec![
            Rule {
                id: "domain1_rule".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Allow,
                priority: 100,
            },
        ]);
        
        let policy2 = create_test_policy("policy2", "domain2", vec![
            Rule {
                id: "domain2_rule".to_string(),
                condition: RuleCondition::Always,
                action: RuleAction::Deny,
                priority: 100,
            },
        ]);
        
        engine.load_policy(policy1).unwrap();
        engine.load_policy(policy2).unwrap();
        
        // Subject belongs to both domains
        let context = create_test_context(
            create_test_subject("user1", vec![], vec!["domain1".to_string(), "domain2".to_string()]),
            create_test_resource("resource1", "domain1"),
            create_test_action("read"),
        );
        
        let result = engine.evaluate(context).await.unwrap();
        // Both policies should be evaluated
        assert!(result.evaluated_policies.contains(&"policy1".to_string()));
        assert!(result.evaluated_policies.contains(&"policy2".to_string()));
    }
}