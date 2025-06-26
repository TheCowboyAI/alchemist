//! Security policy abstractions

use crate::{auth::AuthContext, error::Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Security policy definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Policy rules
    pub rules: Vec<PolicyRule>,
    /// Priority (higher values take precedence)
    pub priority: i32,
    /// Whether the policy is enabled
    pub enabled: bool,
    /// Policy metadata
    pub metadata: HashMap<String, String>,
}

impl SecurityPolicy {
    /// Create a new security policy
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            rules: Vec::new(),
            priority: 0,
            enabled: true,
            metadata: HashMap::new(),
        }
    }

    /// Add a rule to the policy
    pub fn add_rule(mut self, rule: PolicyRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// A single policy rule
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule identifier
    pub id: String,
    /// Condition to evaluate
    pub condition: PolicyCondition,
    /// Effect if condition matches
    pub effect: PolicyEffect,
    /// Resources this rule applies to
    pub resources: Vec<String>,
    /// Actions this rule applies to
    pub actions: Vec<String>,
}

impl PolicyRule {
    /// Create a new policy rule
    pub fn new(id: impl Into<String>, condition: PolicyCondition, effect: PolicyEffect) -> Self {
        Self {
            id: id.into(),
            condition,
            effect,
            resources: Vec::new(),
            actions: Vec::new(),
        }
    }

    /// Add a resource pattern
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resources.push(resource.into());
        self
    }

    /// Add an action pattern
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.actions.push(action.into());
        self
    }

    /// Check if the rule applies to a resource
    pub fn matches_resource(&self, resource: &str) -> bool {
        if self.resources.is_empty() {
            return true; // Empty means all resources
        }

        self.resources
            .iter()
            .any(|pattern| pattern_matches(pattern, resource))
    }

    /// Check if the rule applies to an action
    pub fn matches_action(&self, action: &str) -> bool {
        if self.actions.is_empty() {
            return true; // Empty means all actions
        }

        self.actions
            .iter()
            .any(|pattern| pattern_matches(pattern, action))
    }
}

/// Policy condition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PolicyCondition {
    /// Always matches
    Always,
    /// Never matches
    Never,
    /// Matches if the identity has a specific claim
    HasClaim(String),
    /// Matches if the identity has a specific role
    InRole(String),
    /// Matches if the identity has all specified claims
    HasAllClaims(Vec<String>),
    /// Matches if the identity has any of the specified claims
    HasAnyClaim(Vec<String>),
    /// Matches if a property equals a value
    PropertyEquals { key: String, value: String },
    /// Matches if all sub-conditions match
    And(Vec<PolicyCondition>),
    /// Matches if any sub-condition matches
    Or(Vec<PolicyCondition>),
    /// Inverts a condition
    Not(Box<PolicyCondition>),
    /// Custom condition (evaluated externally)
    Custom(String),
}

impl PolicyCondition {
    /// Evaluate the condition against an auth context
    pub fn evaluate(&self, context: &AuthContext) -> bool {
        match self {
            PolicyCondition::Always => true,
            PolicyCondition::Never => false,
            PolicyCondition::HasClaim(claim_type) => {
                context.identity.get_claim(claim_type).is_some()
            }
            PolicyCondition::InRole(role) => context.identity.has_role(role),
            PolicyCondition::HasAllClaims(claims) => claims
                .iter()
                .all(|claim| context.identity.get_claim(claim).is_some()),
            PolicyCondition::HasAnyClaim(claims) => claims
                .iter()
                .any(|claim| context.identity.get_claim(claim).is_some()),
            PolicyCondition::PropertyEquals { key, value } => {
                context.properties.get(key) == Some(value)
            }
            PolicyCondition::And(conditions) => {
                conditions.iter().all(|cond| cond.evaluate(context))
            }
            PolicyCondition::Or(conditions) => conditions.iter().any(|cond| cond.evaluate(context)),
            PolicyCondition::Not(condition) => !condition.evaluate(context),
            PolicyCondition::Custom(_) => {
                // Custom conditions need external evaluation
                false
            }
        }
    }
}

/// Policy effect
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyEffect {
    /// Allow the action
    Allow,
    /// Deny the action
    Deny,
    /// Audit the action
    Audit,
}

/// Trait for policy evaluation
#[async_trait]
pub trait PolicyEvaluator: Send + Sync {
    /// Evaluate policies for a context
    async fn evaluate(
        &self,
        context: &AuthContext,
        policies: &[SecurityPolicy],
        resource: &str,
        action: &str,
    ) -> Result<PolicyDecision>;

    /// Evaluate custom conditions
    async fn evaluate_custom(&self, context: &AuthContext, condition: &str) -> Result<bool> {
        // Default implementation returns false
        Ok(false)
    }
}

/// Policy decision result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyDecision {
    /// Final effect
    pub effect: PolicyEffect,
    /// Rules that matched
    pub matched_rules: Vec<String>,
    /// Whether audit is required
    pub audit_required: bool,
    /// Evaluation details
    pub details: HashMap<String, String>,
}

impl PolicyDecision {
    /// Create an allow decision
    pub fn allow() -> Self {
        Self {
            effect: PolicyEffect::Allow,
            matched_rules: Vec::new(),
            audit_required: false,
            details: HashMap::new(),
        }
    }

    /// Create a deny decision
    pub fn deny() -> Self {
        Self {
            effect: PolicyEffect::Deny,
            matched_rules: Vec::new(),
            audit_required: false,
            details: HashMap::new(),
        }
    }

    /// Add a matched rule
    pub fn with_matched_rule(mut self, rule_id: impl Into<String>) -> Self {
        self.matched_rules.push(rule_id.into());
        self
    }

    /// Set audit requirement
    pub fn with_audit(mut self) -> Self {
        self.audit_required = true;
        self
    }

    /// Add a detail
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }
}

/// Default policy evaluator implementation
pub struct DefaultPolicyEvaluator;

#[async_trait]
impl PolicyEvaluator for DefaultPolicyEvaluator {
    async fn evaluate(
        &self,
        context: &AuthContext,
        policies: &[SecurityPolicy],
        resource: &str,
        action: &str,
    ) -> Result<PolicyDecision> {
        let mut decision = PolicyDecision::allow();
        let mut explicit_deny = false;
        let mut explicit_allow = false;

        // Sort policies by priority (descending)
        let mut sorted_policies = policies.to_vec();
        sorted_policies.sort_by(|a, b| b.priority.cmp(&a.priority));

        for policy in &sorted_policies {
            if !policy.enabled {
                continue;
            }

            for rule in &policy.rules {
                if !rule.matches_resource(resource) || !rule.matches_action(action) {
                    continue;
                }

                let condition_met = if let PolicyCondition::Custom(name) = &rule.condition {
                    self.evaluate_custom(context, name).await?
                } else {
                    rule.condition.evaluate(context)
                };

                if condition_met {
                    decision = decision.with_matched_rule(&rule.id);

                    match rule.effect {
                        PolicyEffect::Allow => {
                            explicit_allow = true;
                        }
                        PolicyEffect::Deny => {
                            explicit_deny = true;
                            decision.effect = PolicyEffect::Deny;
                        }
                        PolicyEffect::Audit => {
                            decision = decision.with_audit();
                        }
                    }
                }
            }

            // Explicit deny takes precedence
            if explicit_deny {
                break;
            }
        }

        // Default deny if no explicit allow
        if !explicit_allow && !explicit_deny {
            decision.effect = PolicyEffect::Deny;
            decision = decision.with_detail("reason", "no matching allow rule");
        }

        Ok(decision)
    }
}

/// Helper function for pattern matching
fn pattern_matches(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        value.starts_with(prefix)
    } else {
        pattern == value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claims::{Claim, ClaimsIdentity, claim_types};
    use serde_json::json;

    #[test]
    fn test_policy_rule_matching() {
        let rule = PolicyRule::new("rule1", PolicyCondition::Always, PolicyEffect::Allow)
            .with_resource("users/*")
            .with_action("read");

        assert!(rule.matches_resource("users/123"));
        assert!(rule.matches_resource("users/alice"));
        assert!(!rule.matches_resource("groups/admin"));

        assert!(rule.matches_action("read"));
        assert!(!rule.matches_action("write"));
    }

    #[test]
    fn test_policy_conditions() {
        let mut identity = ClaimsIdentity::new();
        identity.add_claim(Claim::new(
            "test-issuer",
            "user123",
            claim_types::ROLE,
            json!("admin"),
        ));

        let context = AuthContext::new(identity, "test").with_property("ip", "192.168.1.1");

        // Test various conditions
        assert!(PolicyCondition::Always.evaluate(&context));
        assert!(!PolicyCondition::Never.evaluate(&context));
        assert!(PolicyCondition::InRole("admin".to_string()).evaluate(&context));
        assert!(!PolicyCondition::InRole("user".to_string()).evaluate(&context));

        assert!(
            PolicyCondition::PropertyEquals {
                key: "ip".to_string(),
                value: "192.168.1.1".to_string(),
            }
            .evaluate(&context)
        );

        // Test compound conditions
        let and_condition = PolicyCondition::And(vec![
            PolicyCondition::InRole("admin".to_string()),
            PolicyCondition::PropertyEquals {
                key: "ip".to_string(),
                value: "192.168.1.1".to_string(),
            },
        ]);
        assert!(and_condition.evaluate(&context));

        let not_condition =
            PolicyCondition::Not(Box::new(PolicyCondition::InRole("user".to_string())));
        assert!(not_condition.evaluate(&context));
    }

    #[tokio::test]
    async fn test_default_policy_evaluator() {
        let evaluator = DefaultPolicyEvaluator;

        let mut identity = ClaimsIdentity::new();
        identity.add_claim(Claim::new(
            "test-issuer",
            "user123",
            claim_types::ROLE,
            json!("reader"),
        ));

        let context = AuthContext::new(identity, "test");

        let policies = vec![
            SecurityPolicy::new("policy1", "Read Policy")
                .add_rule(
                    PolicyRule::new(
                        "allow-readers",
                        PolicyCondition::InRole("reader".to_string()),
                        PolicyEffect::Allow,
                    )
                    .with_resource("documents/*")
                    .with_action("read"),
                )
                .with_priority(10),
            SecurityPolicy::new("policy2", "Deny Write")
                .add_rule(
                    PolicyRule::new("deny-write", PolicyCondition::Always, PolicyEffect::Deny)
                        .with_resource("documents/*")
                        .with_action("write"),
                )
                .with_priority(20),
        ];

        // Test allowed action
        let decision = evaluator
            .evaluate(&context, &policies, "documents/123", "read")
            .await
            .unwrap();
        assert_eq!(decision.effect, PolicyEffect::Allow);
        assert!(
            decision
                .matched_rules
                .contains(&"allow-readers".to_string())
        );

        // Test denied action
        let decision = evaluator
            .evaluate(&context, &policies, "documents/123", "write")
            .await
            .unwrap();
        assert_eq!(decision.effect, PolicyEffect::Deny);
        assert!(decision.matched_rules.contains(&"deny-write".to_string()));

        // Test no matching rule (default deny)
        let decision = evaluator
            .evaluate(&context, &policies, "users/123", "read")
            .await
            .unwrap();
        assert_eq!(decision.effect, PolicyEffect::Deny);
    }
}
