# Phase 2: ACT Structures Implementation Plan

## Overview

This phase implements the remaining Applied Category Theory structures using Domain-Driven Design terminology. We'll build upon the ConceptGraph foundation to add advanced composition and reasoning capabilities.

## Timeline: 2 Weeks (January 8-22, 2025)

## Week 1: ContextBridge and MetricContext

### Day 1-2: ContextBridge Implementation

**ContextBridge** represents relationships between different bounded contexts, enabling cross-domain collaboration and translation.

#### Domain Model

```rust
// src/domain/conceptual_graph/context_bridge.rs

/// Represents a relationship between two bounded contexts
pub struct ContextBridge {
    pub id: ContextBridgeId,
    pub source_context: ConceptId,
    pub target_context: ConceptId,
    pub mapping_type: ContextMappingType,
    pub translation_rules: Vec<TranslationRule>,
}

pub enum ContextMappingType {
    /// Shared kernel - common subset of concepts
    SharedKernel {
        shared_concepts: Vec<ConceptId>,
    },

    /// Customer-supplier relationship
    CustomerSupplier {
        contract: InterfaceContract,
    },

    /// Conformist - downstream conforms to upstream
    Conformist,

    /// Anti-corruption layer
    AntiCorruptionLayer {
        translator: TranslationGraph,
    },

    /// Open host service
    OpenHostService {
        published_language: PublishedLanguage,
    },

    /// Partnership - mutual influence
    Partnership,
}

pub struct TranslationRule {
    pub source_concept: ConceptId,
    pub target_concept: ConceptId,
    pub transformation: ConceptTransformation,
}
```

#### Commands and Events

```rust
// Commands
pub enum ContextBridgeCommand {
    CreateBridge {
        source_context: ConceptId,
        target_context: ConceptId,
        mapping_type: ContextMappingType,
    },
    AddTranslationRule {
        bridge_id: ContextBridgeId,
        rule: TranslationRule,
    },
    TranslateConcept {
        bridge_id: ContextBridgeId,
        concept: ConceptGraph,
        direction: TranslationDirection,
    },
}

// Events
pub enum ContextBridgeEvent {
    BridgeCreated {
        bridge_id: ContextBridgeId,
        source_context: ConceptId,
        target_context: ConceptId,
        mapping_type: ContextMappingType,
    },
    TranslationRuleAdded {
        bridge_id: ContextBridgeId,
        rule: TranslationRule,
    },
    ConceptTranslated {
        bridge_id: ContextBridgeId,
        source_concept: ConceptGraph,
        target_concept: ConceptGraph,
    },
}
```

### Day 3-4: MetricContext Implementation

**MetricContext** represents domains with measurable relationships (distance, cost, time, probability).

#### Domain Model

```rust
// src/domain/conceptual_graph/metric_context.rs

/// A context enriched with metric structure
pub struct MetricContext {
    pub id: MetricContextId,
    pub base_context: ConceptId,
    pub metric_type: MetricType,
    pub metric_space: MetricSpace,
}

pub enum MetricType {
    /// Semantic distance between concepts
    SemanticDistance {
        distance_function: DistanceFunction,
    },

    /// Cost of transformations
    TransformationCost {
        cost_function: CostFunction,
    },

    /// Time delays in processes
    TemporalDelay {
        delay_function: DelayFunction,
    },

    /// Probability of relationships
    Probabilistic {
        probability_function: ProbabilityFunction,
    },

    /// Resource consumption
    ResourceMetric {
        resource_type: ResourceType,
        consumption_function: ConsumptionFunction,
    },
}

pub struct MetricSpace {
    /// Distance/cost matrix between concepts
    pub distances: HashMap<(ConceptId, ConceptId), f64>,

    /// Metric properties
    pub is_symmetric: bool,
    pub satisfies_triangle_inequality: bool,
    pub has_zero_self_distance: bool,
}
```

#### Operations

```rust
impl MetricContext {
    /// Find shortest path between concepts
    pub fn shortest_path(&self, from: ConceptId, to: ConceptId) -> Result<Path> {
        // Dijkstra's algorithm using metric distances
    }

    /// Find k-nearest neighbors
    pub fn nearest_neighbors(&self, concept: ConceptId, k: usize) -> Vec<(ConceptId, f64)> {
        // Return k closest concepts by metric
    }

    /// Cluster concepts by metric similarity
    pub fn cluster_by_distance(&self, threshold: f64) -> Vec<ConceptCluster> {
        // Hierarchical clustering using metric
    }

    /// Calculate metric ball (all concepts within radius)
    pub fn metric_ball(&self, center: ConceptId, radius: f64) -> Vec<ConceptId> {
        // All concepts within metric distance
    }
}
```

### Day 5: Integration and Testing

- Integrate ContextBridge with existing ConceptGraph
- Integrate MetricContext with quality dimensions
- Create comprehensive test suite
- Add visualization support for metric spaces

## Week 2: RuleContext and Advanced Features

### Day 6-7: RuleContext Implementation

**RuleContext** represents domains with internal business logic and reasoning capabilities.

#### Domain Model

```rust
// src/domain/conceptual_graph/rule_context.rs

/// A context with logical structure and rules
pub struct RuleContext {
    pub id: RuleContextId,
    pub base_context: ConceptId,
    pub logic_system: LogicSystem,
    pub rules: Vec<BusinessRule>,
    pub truth_values: TruthValueSystem,
}

pub struct LogicSystem {
    /// Logical operators
    pub operators: LogicalOperators,

    /// Inference rules
    pub inference_rules: Vec<InferenceRule>,

    /// Axioms (always true statements)
    pub axioms: Vec<LogicalStatement>,
}

pub struct BusinessRule {
    pub id: RuleId,
    pub name: String,
    pub condition: LogicalExpression,
    pub consequence: RuleConsequence,
    pub priority: u32,
}

pub enum LogicalExpression {
    /// Atomic predicate
    Predicate {
        concept: ConceptId,
        property: PropertyName,
        value: PropertyValue,
    },

    /// Logical AND
    And(Box<LogicalExpression>, Box<LogicalExpression>),

    /// Logical OR
    Or(Box<LogicalExpression>, Box<LogicalExpression>),

    /// Logical NOT
    Not(Box<LogicalExpression>),

    /// Implication
    Implies(Box<LogicalExpression>, Box<LogicalExpression>),

    /// Universal quantification
    ForAll {
        variable: Variable,
        domain: ConceptId,
        expression: Box<LogicalExpression>,
    },

    /// Existential quantification
    Exists {
        variable: Variable,
        domain: ConceptId,
        expression: Box<LogicalExpression>,
    },
}
```

#### Rule Engine

```rust
impl RuleContext {
    /// Evaluate a logical expression
    pub fn evaluate(&self, expr: &LogicalExpression, context: &EvaluationContext) -> TruthValue {
        // Recursive evaluation with context
    }

    /// Apply all applicable rules
    pub fn apply_rules(&self, facts: &FactSet) -> Vec<RuleApplication> {
        // Forward chaining inference
    }

    /// Check consistency of rules
    pub fn check_consistency(&self) -> ConsistencyReport {
        // Verify no contradictions
    }

    /// Derive new facts from existing ones
    pub fn derive_facts(&self, facts: &FactSet) -> FactSet {
        // Inference engine
    }
}
```

### Day 8-9: Advanced Composition Features

#### Cross-Context Operations

```rust
/// Compose contexts using bridges
pub struct ContextComposer {
    pub contexts: HashMap<ConceptId, ConceptGraph>,
    pub bridges: HashMap<ContextBridgeId, ContextBridge>,
    pub metrics: HashMap<MetricContextId, MetricContext>,
    pub rules: HashMap<RuleContextId, RuleContext>,
}

impl ContextComposer {
    /// Federated query across contexts
    pub fn federated_query(&self, query: FederatedQuery) -> QueryResult {
        // Query multiple contexts and combine results
    }

    /// Translate concept through bridge chain
    pub fn translate_through_chain(&self, concept: ConceptGraph, path: Vec<ContextBridgeId>) -> Result<ConceptGraph> {
        // Sequential translation
    }

    /// Find optimal path considering metrics
    pub fn optimal_translation_path(&self, from: ConceptId, to: ConceptId) -> Result<Vec<ContextBridgeId>> {
        // Use metric distances to find best translation path
    }
}
```

### Day 10: Polish and Documentation

- Complete integration tests
- Performance optimization
- Documentation and examples
- Prepare for Phase 3

## Deliverables

1. **ContextBridge Module**
   - Full implementation of context mapping patterns
   - Translation rules and transformations
   - Anti-corruption layer support

2. **MetricContext Module**
   - Distance/cost/time metrics
   - Shortest path algorithms
   - Clustering and similarity search

3. **RuleContext Module**
   - Business rule engine
   - Logical inference system
   - Consistency checking

4. **Integration Features**
   - Cross-context queries
   - Metric-aware translation
   - Rule-based composition

5. **Tests and Documentation**
   - Unit tests for each module
   - Integration tests for composition
   - User guide and examples

## Success Criteria

- [ ] All three context types implemented and tested
- [ ] Integration with existing ConceptGraph system
- [ ] Cross-context operations working
- [ ] Performance benchmarks met (< 100ms for typical operations)
- [ ] Documentation complete
- [ ] Examples demonstrating real-world usage

## Next Phase Preview

Phase 3 will focus on:
- Domain model importers (DDD, UML, etc.)
- Workflow engine using conceptual graphs
- Visual composition interface
- AI agent integration
