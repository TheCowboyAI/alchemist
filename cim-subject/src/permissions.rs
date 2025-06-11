//! Subject-based permissions and access control

use crate::error::Result;
use crate::pattern::Pattern;
use crate::subject::Subject;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Permissions for subject-based operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    /// Rules for this permission set
    rules: Vec<PermissionRule>,
    /// Default policy when no rules match
    default_policy: Policy,
}

impl Default for Permissions {
    fn default() -> Self {
        Self::new(Policy::Deny)
    }
}

impl Permissions {
    /// Create new permissions with a default policy
    pub fn new(default_policy: Policy) -> Self {
        Self {
            rules: Vec::new(),
            default_policy,
        }
    }

    /// Add a permission rule
    pub fn add_rule(&mut self, rule: PermissionRule) {
        self.rules.push(rule);
    }

    /// Check if an operation is allowed on a subject
    pub fn is_allowed(&self, subject: &Subject, operation: Operation) -> bool {
        // Collect all matching rules
        let mut matching_rules: Vec<&PermissionRule> = self.rules
            .iter()
            .filter(|rule| rule.matches(subject, operation))
            .collect();

        // Sort by specificity (most specific first)
        matching_rules.sort_by(|a, b| {
            if a.pattern.is_more_specific_than(&b.pattern) {
                std::cmp::Ordering::Less
            } else if b.pattern.is_more_specific_than(&a.pattern) {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });

        // Apply the most specific rule
        if let Some(rule) = matching_rules.first() {
            return rule.policy == Policy::Allow;
        }

        // No rule matched, use default policy
        self.default_policy == Policy::Allow
    }

    /// Check if publishing to a subject is allowed
    pub fn can_publish(&self, subject: &Subject) -> bool {
        self.is_allowed(subject, Operation::Publish)
    }

    /// Check if subscribing to a subject is allowed
    pub fn can_subscribe(&self, subject: &Subject) -> bool {
        self.is_allowed(subject, Operation::Subscribe)
    }

    /// Check if requesting on a subject is allowed
    pub fn can_request(&self, subject: &Subject) -> bool {
        self.is_allowed(subject, Operation::Request)
    }

    /// Get all allowed subjects for an operation from a list
    pub fn filter_allowed(&self, subjects: &[Subject], operation: Operation) -> Vec<Subject> {
        subjects
            .iter()
            .filter(|s| self.is_allowed(s, operation))
            .cloned()
            .collect()
    }

    /// Merge another permission set into this one
    pub fn merge(&mut self, other: Permissions) {
        self.rules.extend(other.rules);
    }

    /// Create a more restrictive permission set (intersection)
    pub fn intersect(&self, other: &Permissions) -> Permissions {
        let mut result = Permissions::new(Policy::Deny);

        // For each rule in self that allows
        for self_rule in &self.rules {
            if self_rule.policy == Policy::Allow {
                // Check if there's an overlapping allow rule in other
                for other_rule in &other.rules {
                    if other_rule.policy == Policy::Allow {
                        // Check if patterns could overlap and operations intersect
                        let ops_intersection: HashSet<_> = self_rule.operations
                            .intersection(&other_rule.operations)
                            .copied()
                            .collect();

                        if !ops_intersection.is_empty() {
                            // Check if one pattern is more specific than the other
                            // We need to determine if the patterns actually overlap
                            // For simplicity, we'll check if they match the same prefix

                            // Only add the more specific pattern
                            if self_rule.pattern.is_more_specific_than(&other_rule.pattern) {
                                result.add_rule(PermissionRule::new(
                                    self_rule.pattern.clone(),
                                    ops_intersection,
                                    Policy::Allow,
                                ));
                            } else {
                                result.add_rule(PermissionRule::new(
                                    other_rule.pattern.clone(),
                                    ops_intersection,
                                    Policy::Allow,
                                ));
                            }
                        }
                    }
                }
            }
        }

        result
    }
}

/// A permission rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    /// Pattern to match subjects
    pub pattern: Pattern,
    /// Operations this rule applies to
    pub operations: HashSet<Operation>,
    /// Policy (allow or deny)
    pub policy: Policy,
    /// Optional description
    pub description: Option<String>,
}

impl PermissionRule {
    /// Create a new permission rule
    pub fn new(pattern: Pattern, operations: HashSet<Operation>, policy: Policy) -> Self {
        Self {
            pattern,
            operations,
            policy,
            description: None,
        }
    }

    /// Create an allow rule
    pub fn allow(pattern: Pattern, operations: HashSet<Operation>) -> Self {
        Self::new(pattern, operations, Policy::Allow)
    }

    /// Create a deny rule
    pub fn deny(pattern: Pattern, operations: HashSet<Operation>) -> Self {
        Self::new(pattern, operations, Policy::Deny)
    }

    /// Add a description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Check if this rule matches a subject and operation
    pub fn matches(&self, subject: &Subject, operation: Operation) -> bool {
        self.pattern.matches(subject) && self.operations.contains(&operation)
    }
}

/// Operations that can be performed on subjects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Operation {
    /// Publish messages to a subject
    Publish,
    /// Subscribe to receive messages from a subject
    Subscribe,
    /// Make request-reply calls on a subject
    Request,
    /// All operations
    All,
}

impl Operation {
    /// Get all basic operations (not including All)
    pub fn all_operations() -> HashSet<Operation> {
        let mut ops = HashSet::new();
        ops.insert(Operation::Publish);
        ops.insert(Operation::Subscribe);
        ops.insert(Operation::Request);
        ops
    }
}

/// Permission policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Policy {
    /// Allow the operation
    Allow,
    /// Deny the operation
    Deny,
}

/// Builder for permissions
#[derive(Debug, Default)]
pub struct PermissionsBuilder {
    rules: Vec<PermissionRule>,
    default_policy: Option<Policy>,
}

impl PermissionsBuilder {
    /// Create a new permissions builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the default policy
    pub fn default_policy(mut self, policy: Policy) -> Self {
        self.default_policy = Some(policy);
        self
    }

    /// Allow a pattern for specific operations
    pub fn allow(mut self, pattern: &str, operations: &[Operation]) -> Result<Self> {
        let pattern = Pattern::new(pattern)?;
        let ops: HashSet<_> = operations.iter().copied().collect();
        self.rules.push(PermissionRule::allow(pattern, ops));
        Ok(self)
    }

    /// Deny a pattern for specific operations
    pub fn deny(mut self, pattern: &str, operations: &[Operation]) -> Result<Self> {
        let pattern = Pattern::new(pattern)?;
        let ops: HashSet<_> = operations.iter().copied().collect();
        self.rules.push(PermissionRule::deny(pattern, ops));
        Ok(self)
    }

    /// Allow all operations on a pattern
    pub fn allow_all(self, pattern: &str) -> Result<Self> {
        self.allow(pattern, &[Operation::Publish, Operation::Subscribe, Operation::Request])
    }

    /// Deny all operations on a pattern
    pub fn deny_all(self, pattern: &str) -> Result<Self> {
        self.deny(pattern, &[Operation::Publish, Operation::Subscribe, Operation::Request])
    }

    /// Build the permissions
    pub fn build(self) -> Permissions {
        let default_policy = self.default_policy.unwrap_or(Policy::Deny);
        let mut perms = Permissions::new(default_policy);
        perms.rules = self.rules;
        perms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_permissions() {
        let perms = PermissionsBuilder::new()
            .default_policy(Policy::Deny)
            .allow("users.*.created.>", &[Operation::Publish])
            .unwrap()
            .allow("users.>", &[Operation::Subscribe])
            .unwrap()
            .build();

        let subject = Subject::new("users.person.created.v1").unwrap();

        assert!(perms.can_publish(&subject));
        assert!(perms.can_subscribe(&subject));
        assert!(!perms.can_request(&subject)); // Not allowed
    }

    #[test]
    fn test_deny_overrides() {
        let perms = PermissionsBuilder::new()
            .default_policy(Policy::Allow)
            .deny("*.*.deleted.>", &[Operation::Publish])
            .unwrap()
            .build();

        let subject = Subject::new("users.person.deleted.v1").unwrap();

        assert!(!perms.can_publish(&subject)); // Explicitly denied
        assert!(perms.can_subscribe(&subject)); // Default allow
    }

    #[test]
    fn test_permission_ordering() {
        let perms = PermissionsBuilder::new()
            .allow("users.>", &[Operation::Subscribe])
            .unwrap()
            .deny("users.admin.>", &[Operation::Subscribe])
            .unwrap()
            .build();

        let user_subject = Subject::new("users.person.created.v1").unwrap();
        let admin_subject = Subject::new("users.admin.created.v1").unwrap();

        assert!(perms.can_subscribe(&user_subject));
        assert!(!perms.can_subscribe(&admin_subject)); // More specific deny
    }

    #[test]
    fn test_filter_allowed() {
        let perms = PermissionsBuilder::new()
            .allow("events.public.>", &[Operation::Subscribe])
            .unwrap()
            .build();

        let subjects = vec![
            Subject::new("events.public.news.v1").unwrap(),
            Subject::new("events.private.data.v1").unwrap(),
            Subject::new("events.public.alert.v1").unwrap(),
        ];

        let allowed = perms.filter_allowed(&subjects, Operation::Subscribe);
        assert_eq!(allowed.len(), 2);
        assert!(allowed.iter().all(|s| s.context() == "events" && s.aggregate() == "public"));
    }

    #[test]
    fn test_permission_intersection() {
        let perms1 = PermissionsBuilder::new()
            .allow("users.>", &[Operation::Subscribe])
            .unwrap()
            .allow("orders.>", &[Operation::Subscribe])
            .unwrap()
            .build();

        let perms2 = PermissionsBuilder::new()
            .allow("users.person.>", &[Operation::Subscribe])
            .unwrap()
            .allow("inventory.>", &[Operation::Subscribe])
            .unwrap()
            .build();

        let intersection = perms1.intersect(&perms2);

        // Only users.person.> should be in the intersection
        let user_person = Subject::new("users.person.created.v1").unwrap();
        let user_admin = Subject::new("users.admin.created.v1").unwrap();
        let order = Subject::new("orders.order.placed.v1").unwrap();

        assert!(intersection.can_subscribe(&user_person)); // In both
        assert!(!intersection.can_subscribe(&user_admin)); // Only in perms1
        assert!(!intersection.can_subscribe(&order)); // Only in perms1
    }
}
