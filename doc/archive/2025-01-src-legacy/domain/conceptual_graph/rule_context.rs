use crate::domain::conceptual_graph::ConceptId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for a rule context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuleContextId(pub Uuid);

impl RuleContextId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for RuleContextId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a domain context with business rules and reasoning capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleContext {
    pub id: RuleContextId,
    pub name: String,
    pub domain_context: ConceptId,
    pub rules: HashMap<RuleId, BusinessRule>,
    pub inference_engine: InferenceEngine,
    pub rule_dependencies: RuleDependencyGraph,
}

impl RuleContext {
    pub fn new(name: String, domain_context: ConceptId) -> Self {
        Self {
            id: RuleContextId::new(),
            name,
            domain_context,
            rules: HashMap::new(),
            inference_engine: InferenceEngine::default(),
            rule_dependencies: RuleDependencyGraph::new(),
        }
    }

    /// Add a business rule to the context
    pub fn add_rule(&mut self, rule: BusinessRule) -> Result<(), String> {
        // Validate rule doesn't create circular dependencies
        if self.would_create_cycle(&rule) {
            return Err("Rule would create circular dependency".to_string());
        }

        let rule_id = rule.id;
        self.rules.insert(rule_id, rule);
        self.update_dependencies(rule_id)?;

        Ok(())
    }

    /// Remove a rule and update dependencies
    pub fn remove_rule(&mut self, rule_id: RuleId) -> Result<(), String> {
        self.rules
            .remove(&rule_id)
            .ok_or_else(|| "Rule not found".to_string())?;

        self.rule_dependencies.remove_rule(rule_id);
        Ok(())
    }

    /// Apply rules to evaluate a concept
    pub fn evaluate(&self, concept: ConceptId, facts: &FactSet) -> Result<RuleEvaluation, String> {
        self.inference_engine.evaluate(concept, &self.rules, facts)
    }

    /// Check if all rules are satisfied for a concept
    pub fn check_compliance(&self, concept: ConceptId, facts: &FactSet) -> ComplianceResult {
        let mut violations = Vec::new();
        let mut satisfied = Vec::new();

        for (rule_id, rule) in &self.rules {
            match rule.evaluate(concept, facts) {
                Ok(true) => satisfied.push(*rule_id),
                Ok(false) => violations.push(RuleViolation {
                    rule_id: *rule_id,
                    rule_name: rule.name.clone(),
                    concept,
                    message: rule.get_violation_reason(concept, facts),
                    severity: rule.notification_severity.clone(),
                }),
                Err(e) => violations.push(RuleViolation {
                    rule_id: *rule_id,
                    rule_name: rule.name.clone(),
                    concept,
                    message: format!("Evaluation error: {}", e),
                    severity: NotificationSeverity::Error,
                }),
            }
        }

        ComplianceResult {
            compliant: violations.is_empty(),
            violations,
            satisfied_rules: satisfied,
        }
    }

    /// Infer new facts based on existing facts and rules
    pub fn infer_facts(&self, facts: &FactSet) -> Result<InferredFacts, String> {
        self.inference_engine.forward_chain(&self.rules, facts)
    }

    /// Find rules that would be triggered by a fact change
    pub fn analyze_impact(&self, fact_change: &FactChange) -> Vec<RuleId> {
        self.rules
            .iter()
            .filter(|(_, rule)| rule.depends_on_fact(&fact_change.fact_type))
            .map(|(id, _)| *id)
            .collect()
    }

    fn would_create_cycle(&self, new_rule: &BusinessRule) -> bool {
        // Check if adding this rule would create a dependency cycle
        self.rule_dependencies.would_create_cycle(&new_rule)
    }

    fn update_dependencies(&mut self, rule_id: RuleId) -> Result<(), String> {
        if let Some(rule) = self.rules.get(&rule_id) {
            self.rule_dependencies.update_for_rule(rule_id, rule)?;
        }
        Ok(())
    }
}

/// Unique identifier for a rule
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuleId(pub Uuid);

impl RuleId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Business rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRule {
    pub id: RuleId,
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
    pub priority: u32,
    pub enabled: bool,
    pub notification_severity: NotificationSeverity,
}

impl BusinessRule {
    pub fn evaluate(&self, concept: ConceptId, facts: &FactSet) -> Result<bool, String> {
        if !self.enabled {
            return Ok(true); // Disabled rules are considered satisfied
        }

        // Evaluate all conditions
        for condition in &self.conditions {
            if !condition.evaluate(concept, facts)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn get_violation_reason(&self, concept: ConceptId, facts: &FactSet) -> String {
        for condition in &self.conditions {
            if let Ok(false) = condition.evaluate(concept, facts) {
                return condition.describe_failure(concept, facts);
            }
        }
        "Unknown violation".to_string()
    }

    pub fn depends_on_fact(&self, fact_type: &str) -> bool {
        self.conditions.iter().any(|c| c.references_fact(fact_type))
    }
}

/// Types of business rules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuleType {
    /// Validation rule - must always be true
    Validation,
    /// Derivation rule - derives new facts from existing ones
    Derivation,
    /// Constraint rule - limits allowed values or relationships
    Constraint,
    /// Policy rule - business policy enforcement
    Policy,
    /// Calculation rule - computes values
    Calculation,
}

/// Rule condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    /// Simple fact check
    FactExists {
        fact_type: String,
        expected_value: Option<FactValue>,
    },
    /// Comparison between facts
    Comparison {
        left: FactReference,
        operator: ComparisonOperator,
        right: FactReference,
    },
    /// Logical combination of conditions
    Logical {
        operator: LogicalOperator,
        conditions: Vec<Condition>,
    },
    /// Pattern matching on graph structure
    GraphPattern { pattern: GraphPattern },
    /// Custom predicate function
    Custom {
        predicate_name: String,
        parameters: HashMap<String, FactValue>,
    },
}

impl Condition {
    pub fn evaluate(&self, concept: ConceptId, facts: &FactSet) -> Result<bool, String> {
        match self {
            Condition::FactExists {
                fact_type,
                expected_value,
            } => {
                let fact = facts.get_fact(concept, fact_type);
                match (fact, expected_value) {
                    (Some(actual), Some(expected)) => Ok(&actual == expected),
                    (Some(_), None) => Ok(true),
                    (None, _) => Ok(false),
                }
            }
            Condition::Comparison {
                left,
                operator,
                right,
            } => {
                let left_val = left.resolve(concept, facts)?;
                let right_val = right.resolve(concept, facts)?;
                operator.evaluate(&left_val, &right_val)
            }
            Condition::Logical {
                operator,
                conditions,
            } => match operator {
                LogicalOperator::And => {
                    for cond in conditions {
                        if !cond.evaluate(concept, facts)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                LogicalOperator::Or => {
                    for cond in conditions {
                        if cond.evaluate(concept, facts)? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                }
                LogicalOperator::Not => {
                    if conditions.len() != 1 {
                        return Err("NOT operator requires exactly one condition".to_string());
                    }
                    Ok(!conditions[0].evaluate(concept, facts)?)
                }
            },
            Condition::GraphPattern { pattern } => pattern.matches(concept, facts),
            Condition::Custom {
                predicate_name,
                parameters,
            } => {
                // In a real implementation, this would call registered predicates
                Err(format!(
                    "Custom predicate '{}' not implemented",
                    predicate_name
                ))
            }
        }
    }

    pub fn describe_failure(&self, concept: ConceptId, facts: &FactSet) -> String {
        match self {
            Condition::FactExists {
                fact_type,
                expected_value,
            } => match expected_value {
                Some(expected) => format!("Expected {} to be {:?}", fact_type, expected),
                None => format!("Expected {} to exist", fact_type),
            },
            Condition::Comparison {
                left,
                operator,
                right,
            } => {
                format!("Comparison failed: {:?} {:?} {:?}", left, operator, right)
            }
            Condition::Logical { operator, .. } => {
                format!("Logical condition {:?} not satisfied", operator)
            }
            Condition::GraphPattern { pattern } => {
                format!("Graph pattern {:?} not matched", pattern)
            }
            Condition::Custom { predicate_name, .. } => {
                format!("Custom predicate '{}' failed", predicate_name)
            }
        }
    }

    pub fn references_fact(&self, fact_type: &str) -> bool {
        match self {
            Condition::FactExists { fact_type: ft, .. } => ft == fact_type,
            Condition::Comparison { left, right, .. } => {
                left.references_fact(fact_type) || right.references_fact(fact_type)
            }
            Condition::Logical { conditions, .. } => {
                conditions.iter().any(|c| c.references_fact(fact_type))
            }
            Condition::GraphPattern { .. } => false, // Simplified for now
            Condition::Custom { .. } => true,        // Conservative assumption
        }
    }
}

/// Actions to take when a rule is satisfied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    /// Assert a new fact
    AssertFact { fact_type: String, value: FactValue },
    /// Retract an existing fact
    RetractFact { fact_type: String },
    /// Trigger another rule
    TriggerRule { rule_id: RuleId },
    /// Send a notification
    Notify {
        message: String,
        severity: NotificationSeverity,
    },
    /// Execute a custom action
    Custom {
        action_name: String,
        parameters: HashMap<String, FactValue>,
    },
}

/// Fact reference for comparisons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactReference {
    /// Direct fact value
    Fact { fact_type: String },
    /// Literal value
    Literal { value: FactValue },
    /// Computed value
    Computed { expression: String },
}

impl FactReference {
    pub fn resolve(&self, concept: ConceptId, facts: &FactSet) -> Result<FactValue, String> {
        match self {
            FactReference::Fact { fact_type } => facts
                .get_fact(concept, fact_type)
                .ok_or_else(|| format!("Fact '{}' not found", fact_type)),
            FactReference::Literal { value } => Ok(value.clone()),
            FactReference::Computed { expression } => {
                // In a real implementation, this would evaluate the expression
                Err(format!(
                    "Expression evaluation not implemented: {}",
                    expression
                ))
            }
        }
    }

    pub fn references_fact(&self, fact_type: &str) -> bool {
        match self {
            FactReference::Fact { fact_type: ft } => ft == fact_type,
            FactReference::Literal { .. } => false,
            FactReference::Computed { .. } => true, // Conservative assumption
        }
    }
}

/// Comparison operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Contains,
    StartsWith,
    EndsWith,
}

impl ComparisonOperator {
    pub fn evaluate(&self, left: &FactValue, right: &FactValue) -> Result<bool, String> {
        match (self, left, right) {
            (ComparisonOperator::Equal, l, r) => Ok(l == r),
            (ComparisonOperator::NotEqual, l, r) => Ok(l != r),
            (ComparisonOperator::GreaterThan, FactValue::Number(l), FactValue::Number(r)) => {
                Ok(l > r)
            }
            (
                ComparisonOperator::GreaterThanOrEqual,
                FactValue::Number(l),
                FactValue::Number(r),
            ) => Ok(l >= r),
            (ComparisonOperator::LessThan, FactValue::Number(l), FactValue::Number(r)) => Ok(l < r),
            (ComparisonOperator::LessThanOrEqual, FactValue::Number(l), FactValue::Number(r)) => {
                Ok(l <= r)
            }
            (ComparisonOperator::Contains, FactValue::Text(l), FactValue::Text(r)) => {
                Ok(l.contains(r))
            }
            (ComparisonOperator::StartsWith, FactValue::Text(l), FactValue::Text(r)) => {
                Ok(l.starts_with(r))
            }
            (ComparisonOperator::EndsWith, FactValue::Text(l), FactValue::Text(r)) => {
                Ok(l.ends_with(r))
            }
            _ => Err(format!(
                "Invalid comparison: {:?} {:?} {:?}",
                left, self, right
            )),
        }
    }
}

/// Logical operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// Graph pattern for structural matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPattern {
    pub pattern_type: PatternType,
    pub constraints: Vec<PatternConstraint>,
}

impl GraphPattern {
    pub fn matches(&self, concept: ConceptId, facts: &FactSet) -> Result<bool, String> {
        // Simplified implementation
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    /// Node with specific properties
    Node {
        properties: HashMap<String, FactValue>,
    },
    /// Path between nodes
    Path {
        min_length: usize,
        max_length: usize,
    },
    /// Subgraph structure
    Subgraph { nodes: usize, edges: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternConstraint {
    pub constraint_type: String,
    pub parameters: HashMap<String, FactValue>,
}

/// Fact value types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FactValue {
    Boolean(bool),
    Number(f64),
    Text(String),
    Date(String), // ISO 8601
    List(Vec<FactValue>),
    Object(HashMap<String, FactValue>),
    Null,
}

/// Set of facts for reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactSet {
    facts: HashMap<(ConceptId, String), FactValue>,
    metadata: HashMap<(ConceptId, String), FactMetadata>,
}

impl FactSet {
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_fact(&mut self, concept: ConceptId, fact_type: String, value: FactValue) {
        let key = (concept, fact_type);
        self.facts.insert(key.clone(), value);
        self.metadata.insert(
            key,
            FactMetadata {
                timestamp: std::time::SystemTime::now(),
                source: FactSource::Direct,
                confidence: 1.0,
            },
        );
    }

    pub fn get_fact(&self, concept: ConceptId, fact_type: &str) -> Option<FactValue> {
        self.facts.get(&(concept, fact_type.to_string())).cloned()
    }

    pub fn remove_fact(&mut self, concept: ConceptId, fact_type: &str) {
        let key = (concept, fact_type.to_string());
        self.facts.remove(&key);
        self.metadata.remove(&key);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactMetadata {
    pub timestamp: std::time::SystemTime,
    pub source: FactSource,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactSource {
    Direct,
    Inferred { rule_id: RuleId },
    External { source: String },
}

/// Inference engine for rule evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceEngine {
    pub strategy: InferenceStrategy,
    pub max_iterations: usize,
}

impl Default for InferenceEngine {
    fn default() -> Self {
        Self {
            strategy: InferenceStrategy::ForwardChaining,
            max_iterations: 100,
        }
    }
}

impl InferenceEngine {
    pub fn evaluate(
        &self,
        concept: ConceptId,
        rules: &HashMap<RuleId, BusinessRule>,
        facts: &FactSet,
    ) -> Result<RuleEvaluation, String> {
        let mut triggered_rules = Vec::new();
        let mut failed_rules = Vec::new();

        for (rule_id, rule) in rules {
            match rule.evaluate(concept, facts) {
                Ok(true) => triggered_rules.push(*rule_id),
                Ok(false) => failed_rules.push(*rule_id),
                Err(e) => return Err(format!("Rule evaluation error: {}", e)),
            }
        }

        Ok(RuleEvaluation {
            concept_id: concept,
            triggered_rules,
            failed_rules,
            execution_time: std::time::Duration::from_millis(0), // Placeholder
        })
    }

    pub fn forward_chain(
        &self,
        rules: &HashMap<RuleId, BusinessRule>,
        facts: &FactSet,
    ) -> Result<InferredFacts, String> {
        let inferred = InferredFacts::new();
        let working_facts = facts.clone();
        let mut iteration = 0;

        loop {
            iteration += 1;
            if iteration > self.max_iterations {
                return Err("Maximum inference iterations exceeded".to_string());
            }

            let new_facts_added = false;

            for (rule_id, rule) in rules {
                if rule.rule_type != RuleType::Derivation {
                    continue;
                }

                // Check if rule conditions are satisfied
                // If so, execute actions to derive new facts
                // This is a simplified implementation
            }

            if !new_facts_added {
                break;
            }
        }

        Ok(inferred)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InferenceStrategy {
    ForwardChaining,
    BackwardChaining,
    Mixed,
}

/// Result of rule evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEvaluation {
    pub concept_id: ConceptId,
    pub triggered_rules: Vec<RuleId>,
    pub failed_rules: Vec<RuleId>,
    pub execution_time: std::time::Duration,
}

/// Compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    pub compliant: bool,
    pub violations: Vec<RuleViolation>,
    pub satisfied_rules: Vec<RuleId>,
}

/// Represents a change to a fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactChange {
    pub concept_id: ConceptId,
    pub fact_type: String,
    pub old_value: Option<FactValue>,
    pub new_value: Option<FactValue>,
}

/// Represents a rule violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleViolation {
    pub rule_id: RuleId,
    pub rule_name: String,
    pub concept: ConceptId,
    pub message: String,
    pub severity: NotificationSeverity,
}

/// Inferred facts from forward chaining
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredFacts {
    pub facts: Vec<(ConceptId, String, FactValue)>,
    pub derivation_chain: Vec<RuleId>,
}

impl InferredFacts {
    pub fn new() -> Self {
        Self {
            facts: Vec::new(),
            derivation_chain: Vec::new(),
        }
    }
}

/// Notification severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Rule dependency graph for cycle detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDependencyGraph {
    dependencies: HashMap<RuleId, HashSet<RuleId>>,
}

impl RuleDependencyGraph {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }

    pub fn would_create_cycle(&self, new_rule: &BusinessRule) -> bool {
        // Simplified cycle detection
        false
    }

    pub fn update_for_rule(&mut self, rule_id: RuleId, rule: &BusinessRule) -> Result<(), String> {
        // Update dependency graph based on rule actions
        let mut deps = HashSet::new();

        for action in &rule.actions {
            if let Action::TriggerRule { rule_id: target } = action {
                deps.insert(*target);
            }
        }

        self.dependencies.insert(rule_id, deps);
        Ok(())
    }

    pub fn remove_rule(&mut self, rule_id: RuleId) {
        self.dependencies.remove(&rule_id);

        // Remove references to this rule from other rules
        for deps in self.dependencies.values_mut() {
            deps.remove(&rule_id);
        }
    }
}

/// Impact analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub affected_concepts: Vec<ConceptId>,
    pub affected_rules: Vec<RuleId>,
    pub potential_violations: Vec<RuleViolation>,
}

/// Export format for rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Yaml,
    Xml,
    Custom(String),
}

/// Validation result for rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
