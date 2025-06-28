# ConceptGraph Design

## Core Insight

A **ConceptGraph** is a DDD Aggregate that:
- Has a **root ContextGraph** (the aggregate root)
- **Composes multiple ContextGraphs** (aggregate members)
- **Maintains invariants** across the composed graphs
- Has **conceptual components** attached for positioning in conceptual space

## Design

```rust
// A ConceptGraph is an aggregate of ContextGraphs with these components:

/// Component that makes a ContextGraph into a ConceptGraph
#[derive(Debug, Clone)]
pub struct ConceptualSpace {
    pub quality_dimensions: Vec<QualityDimension>,
    pub position: ConceptualPoint,
    pub category: CategoryType,
}

/// Quality dimensions define the conceptual space
#[derive(Debug, Clone)]
pub struct QualityDimension {
    pub name: String,
    pub dimension_type: DimensionType,
    pub range: Range<f64>,
    pub metric: DistanceMetric,
}

/// Applied Category Theory classification
#[derive(Debug, Clone)]
pub enum CategoryType {
    Order(OrderType),
    Database,
    Monoidal,
    Profunctor,
    Enriched(EnrichmentType),
    Topos,
    Operad,
}

/// Component for graph morphisms
#[derive(Debug, Clone)]
pub struct Morphisms {
    pub morphisms: Vec<GraphMorphism>,
}

impl Component for ConceptualSpace { /* ... */ }
impl Component for Morphisms { /* ... */ }
```

## ConceptGraph as DDD Aggregate

```rust
/// ConceptGraph follows the DDD Aggregate pattern
pub struct ConceptGraphAggregate {
    /// The root ContextGraph (aggregate root)
    pub root: ContextGraph<String, String>,

    /// Composed ContextGraphs (aggregate members)
    pub members: HashMap<ContextGraphId, Box<dyn Any>>, // ContextGraph<?, ?>

    /// Aggregate invariants
    pub invariants: Vec<ConceptGraphInvariant>,
}

impl ConceptGraphAggregate {
    /// All modifications go through the aggregate root
    pub fn add_member<N, E>(&mut self, graph: ContextGraph<N, E>) -> Result<()> {
        // Validate invariants
        self.validate_addition(&graph)?;

        // Add to members
        let id = graph.id;
        self.members.insert(id, Box::new(graph));

        // Update root with reference
        let ref_node = self.root.add_node(format!("ref:{}", id));
        self.root.get_node_mut(ref_node)?
            .components.add(GraphReference(id))?;

        Ok(())
    }

    /// Maintain aggregate consistency
    fn validate_addition<N, E>(&self, graph: &ContextGraph<N, E>) -> Result<()> {
        // Check aggregate invariants
        for invariant in &self.invariants {
            invariant.check(self, graph)?;
        }
        Ok(())
    }
}
```

## Creating a ConceptGraph

```rust
// Start with any ContextGraph
let mut graph = ContextGraph::<String, String>::new("UserConcept");

// Add conceptual space component to make it a ConceptGraph
graph.metadata.properties.insert("concept_graph".to_string(), json!(true));

// Add the conceptual space component
let conceptual_space = ConceptualSpace {
    quality_dimensions: vec![
        QualityDimension {
            name: "authority".to_string(),
            dimension_type: DimensionType::Continuous,
            range: 0.0..1.0,
            metric: DistanceMetric::Euclidean,
        },
        QualityDimension {
            name: "activity".to_string(),
            dimension_type: DimensionType::Continuous,
            range: 0.0..100.0,
            metric: DistanceMetric::Euclidean,
        },
    ],
    position: ConceptualPoint::new(vec![0.5, 50.0]),
    category: CategoryType::Database,
};

// This is the key - we attach it as a component to the graph's metadata
// (Since metadata is already a component storage)
// Or we could create a special "root node" that holds graph-level components

// Add morphisms component
let morphisms = Morphisms {
    morphisms: vec![
        GraphMorphism::Embedding { /* ... */ },
        GraphMorphism::Homomorphism { /* ... */ },
    ],
};
```

## Helper Functions

```rust
/// Check if a ContextGraph is a ConceptGraph
pub fn is_concept_graph<N, E>(graph: &ContextGraph<N, E>) -> bool {
    graph.metadata.properties.get("concept_graph")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Get conceptual space from a graph (if it has one)
pub fn get_conceptual_space<N, E>(graph: &ContextGraph<N, E>) -> Option<&ConceptualSpace> {
    // Would need a way to attach components to the graph itself
    // Options:
    // 1. Special root node
    // 2. Extended metadata
    // 3. Graph-level component storage
    None // Placeholder
}

/// Builder pattern for creating ConceptGraphs
pub struct ConceptGraphBuilder<N, E> {
    graph: ContextGraph<N, E>,
    conceptual_space: ConceptualSpace,
}

impl<N, E> ConceptGraphBuilder<N, E> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            graph: ContextGraph::new(name),
            conceptual_space: ConceptualSpace::default(),
        }
    }

    pub fn with_dimension(mut self, dim: QualityDimension) -> Self {
        self.conceptual_space.quality_dimensions.push(dim);
        self
    }

    pub fn with_category(mut self, category: CategoryType) -> Self {
        self.conceptual_space.category = category;
        self
    }

    pub fn build(mut self) -> ContextGraph<N, E> {
        // Mark as concept graph
        self.graph.metadata.properties.insert(
            "concept_graph".to_string(),
            json!(true)
        );

        // TODO: Attach conceptual_space component
        // This requires deciding where graph-level components live

        self.graph
    }
}
```

## Benefits of This Approach

1. **No New Types**: ConceptGraph is just a pattern, not a new struct
2. **Composable**: Any ContextGraph can become a ConceptGraph by adding components
3. **Flexible**: Different graphs can have different conceptual components
4. **Backwards Compatible**: Existing ContextGraphs work unchanged
5. **Progressive Enhancement**: Add conceptual features as needed

## Examples

### Domain Entity as ConceptGraph

```rust
// User entity with conceptual positioning
let user = ConceptGraphBuilder::<String, String>::new("User")
    .with_dimension(QualityDimension {
        name: "authority".to_string(),
        dimension_type: DimensionType::Continuous,
        range: 0.0..1.0,
        metric: DistanceMetric::Euclidean,
    })
    .with_dimension(QualityDimension {
        name: "engagement".to_string(),
        dimension_type: DimensionType::Continuous,
        range: 0.0..100.0,
        metric: DistanceMetric::Euclidean,
    })
    .with_category(CategoryType::Database)
    .build();

// Add nodes as normal
let id_node = user.add_node("id".to_string());
let email_node = user.add_node("email".to_string());
let role_node = user.add_node("role".to_string());
```

### Workflow as ConceptGraph

```rust
// Workflow with monoidal category structure
let workflow = ConceptGraphBuilder::<String, String>::new("OrderProcessing")
    .with_category(CategoryType::Monoidal)
    .with_dimension(QualityDimension {
        name: "complexity".to_string(),
        dimension_type: DimensionType::Discrete,
        range: 1.0..10.0,
        metric: DistanceMetric::Manhattan,
    })
    .build();

// Add workflow steps
let validate = workflow.add_node("Validate".to_string());
let process = workflow.add_node("Process".to_string());
let fulfill = workflow.add_node("Fulfill".to_string());
```

## Implementation Note

To fully realize this design, we need to decide where to store graph-level components. Options:

1. **Special Root Node**: Create a hidden root node that holds graph components
2. **Extended Metadata**: Enhance the metadata system to hold components
3. **Graph Components Field**: Add a `components: ComponentStorage` field to ContextGraph
4. **External Registry**: Store graph components in a separate registry

Each approach has trade-offs in terms of API design and implementation complexity.
