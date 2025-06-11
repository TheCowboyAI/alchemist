use crate::domain::conceptual_graph::{
    BusinessRule, ComplianceResult, ConceptId, FactValue, InferredFacts, RuleContextId,
    RuleEvaluation, RuleId, RuleViolation,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Events related to rule context operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleContextEvent {
    /// A new rule context was created
    RuleContextCreated {
        context_id: RuleContextId,
        name: String,
        domain_context: ConceptId,
    },

    /// A rule was added to the context
    RuleAdded {
        context_id: RuleContextId,
        rule: BusinessRule,
    },

    /// A rule was removed from the context
    RuleRemoved {
        context_id: RuleContextId,
        rule_id: RuleId,
    },

    /// A rule was enabled or disabled
    RuleEnabledChanged {
        context_id: RuleContextId,
        rule_id: RuleId,
        enabled: bool,
    },

    /// Rules were evaluated for a concept
    RulesEvaluated {
        context_id: RuleContextId,
        evaluation: RuleEvaluation,
    },

    /// Compliance was checked for a concept
    ComplianceChecked {
        context_id: RuleContextId,
        result: ComplianceResult,
    },

    /// New facts were inferred
    FactsInferred {
        context_id: RuleContextId,
        inferred_facts: InferredFacts,
    },

    /// Impact analysis was performed
    ImpactAnalyzed {
        context_id: RuleContextId,
        affected_rules: Vec<RuleId>,
        fact_type: String,
    },

    /// Rule priority was updated
    RulePriorityUpdated {
        context_id: RuleContextId,
        rule_id: RuleId,
        old_priority: u32,
        new_priority: u32,
    },

    /// A fact was added
    FactAdded {
        context_id: RuleContextId,
        concept_id: ConceptId,
        fact_type: String,
        value: FactValue,
    },

    /// A fact was removed
    FactRemoved {
        context_id: RuleContextId,
        concept_id: ConceptId,
        fact_type: String,
    },

    /// Rule actions were executed
    RuleActionsExecuted {
        context_id: RuleContextId,
        rule_id: RuleId,
        concept_id: ConceptId,
        actions_performed: Vec<String>,
    },

    /// Rules were validated
    RulesValidated {
        context_id: RuleContextId,
        validation_results: HashMap<RuleId, ValidationResult>,
    },

    /// Rules were exported
    RulesExported {
        context_id: RuleContextId,
        format: String,
        rule_count: usize,
    },

    /// Rules were imported
    RulesImported {
        context_id: RuleContextId,
        format: String,
        imported_count: usize,
        failed_count: usize,
    },

    /// A rule violation occurred
    RuleViolated {
        context_id: RuleContextId,
        violation: RuleViolation,
    },

    /// Rule execution failed
    RuleExecutionFailed {
        context_id: RuleContextId,
        rule_id: RuleId,
        error: String,
    },

    /// Circular dependency detected
    CircularDependencyDetected {
        context_id: RuleContextId,
        rule_chain: Vec<RuleId>,
    },
}

/// Result of rule validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub issues: Vec<String>,
}
