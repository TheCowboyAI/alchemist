use crate::domain::conceptual_graph::{
    BusinessRule, ConceptId, FactChange, FactSet, FactValue, RuleContextId, RuleId,
};
use serde::{Deserialize, Serialize};

/// Commands for rule context operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleContextCommand {
    /// Create a new rule context
    CreateRuleContext {
        name: String,
        domain_context: ConceptId,
    },

    /// Add a business rule to the context
    AddRule {
        context_id: RuleContextId,
        rule: BusinessRule,
    },

    /// Remove a rule from the context
    RemoveRule {
        context_id: RuleContextId,
        rule_id: RuleId,
    },

    /// Enable or disable a rule
    SetRuleEnabled {
        context_id: RuleContextId,
        rule_id: RuleId,
        enabled: bool,
    },

    /// Evaluate rules for a concept
    EvaluateRules {
        context_id: RuleContextId,
        concept_id: ConceptId,
        facts: FactSet,
    },

    /// Check compliance for a concept
    CheckCompliance {
        context_id: RuleContextId,
        concept_id: ConceptId,
        facts: FactSet,
    },

    /// Infer new facts based on existing facts
    InferFacts {
        context_id: RuleContextId,
        facts: FactSet,
    },

    /// Analyze impact of a fact change
    AnalyzeImpact {
        context_id: RuleContextId,
        fact_change: FactChange,
    },

    /// Update rule priority
    UpdateRulePriority {
        context_id: RuleContextId,
        rule_id: RuleId,
        new_priority: u32,
    },

    /// Add fact to the working memory
    AddFact {
        context_id: RuleContextId,
        concept_id: ConceptId,
        fact_type: String,
        value: FactValue,
    },

    /// Remove fact from working memory
    RemoveFact {
        context_id: RuleContextId,
        concept_id: ConceptId,
        fact_type: String,
    },

    /// Execute rule actions
    ExecuteRuleActions {
        context_id: RuleContextId,
        rule_id: RuleId,
        concept_id: ConceptId,
    },

    /// Validate rule consistency
    ValidateRules { context_id: RuleContextId },

    /// Export rules to a format
    ExportRules {
        context_id: RuleContextId,
        format: ExportFormat,
    },

    /// Import rules from a format
    ImportRules {
        context_id: RuleContextId,
        rules_data: String,
        format: ExportFormat,
    },
}

/// Export format for rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Yaml,
    DecisionTable,
    RuleML,
}
