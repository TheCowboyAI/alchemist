# Conceptual Spaces

## Semantic Operations and AI Integration

CIM's conceptual spaces provide semantic intelligence by representing knowledge as geometric structures. All conceptual operations use NATS messaging for distributed semantic processing and AI agent collaboration.

## Conceptual Space Architecture

### Space Definition

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptualSpace {
    pub space_id: ConceptualSpaceId,
    pub name: String,
    pub dimensions: Vec<QualityDimension>,
    pub regions: HashMap<ConceptId, ConvexRegion>,
    pub embedding_model: EmbeddingModel,
    pub distance_metric: DistanceMetric,
    pub created_by: ActorId,
    pub access_policy: AccessPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityDimension {
    pub name: String,
    pub dimension_type: DimensionType,
    pub range: Range<f32>,
    pub weight: f32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DimensionType {
    Continuous,      // e.g., temperature, size
    Categorical,     // e.g., color categories  
    Ordinal,        // e.g., small < medium < large
    Circular,       // e.g., hue, direction
}
```

## Space Management Operations

### Creating Conceptual Spaces

#### Create New Space
```rust
// Command: cmd.conceptual.create_space
#[derive(Serialize, Deserialize)]
pub struct CreateConceptualSpaceCommand {
    pub name: String,
    pub description: String,
    pub dimensions: Vec<DimensionSpec>,
    pub embedding_config: EmbeddingConfig,
    pub distance_metric: DistanceMetric,
    pub initial_concepts: Vec<ConceptSpec>,
}

// NATS Example
let create_space = CreateConceptualSpaceCommand {
    name: "AI/ML Knowledge Space".to_string(),
    description: "Conceptual space for artificial intelligence and machine learning concepts".to_string(),
    dimensions: vec![
        DimensionSpec {
            name: "complexity".to_string(),
            dimension_type: DimensionType::Continuous,
            range: 0.0..1.0,
            weight: 1.0,
            description: "Algorithmic and conceptual complexity".to_string(),
        },
        DimensionSpec {
            name: "application_domain".to_string(),
            dimension_type: DimensionType::Categorical,
            range: 0.0..10.0, // Encoded categories
            weight: 0.8,
            description: "Primary application domain".to_string(),
        },
        DimensionSpec {
            name: "maturity".to_string(),
            dimension_type: DimensionType::Ordinal,
            range: 0.0..5.0, // Research -> Production
            weight: 0.6,
            description: "Technology maturity level".to_string(),
        },
    ],
    embedding_config: EmbeddingConfig {
        model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
        dimension_mapping: DimensionMapping::Learned,
        preprocessing: TextPreprocessing::Standard,
    },
    distance_metric: DistanceMetric::WeightedEuclidean,
    initial_concepts: vec![],
};

let response = client.request(
    "cmd.conceptual.create_space",
    serde_json::to_vec(&create_space)?.into()
).timeout(Duration::from_secs(10)).await?;

let result: CommandResult = serde_json::from_slice(&response.payload)?;
match result {
    CommandResult::Success { event_id, .. } => {
        println!("Conceptual space created: {}", event_id);
    }
    _ => eprintln!("Failed to create space: {:?}", result),
}
```

### Space Configuration

#### Update Space Dimensions
```rust
// Command: cmd.conceptual.reconfigure_space
#[derive(Serialize, Deserialize)]
pub struct ReconfigureSpaceCommand {
    pub space_id: ConceptualSpaceId,
    pub dimension_changes: Vec<DimensionChange>,
    pub recalculate_embeddings: bool,
    pub migration_strategy: MigrationStrategy,
}

#[derive(Serialize, Deserialize)]
pub enum DimensionChange {
    Add(DimensionSpec),
    Remove(String), // dimension name
    ModifyWeight { name: String, new_weight: f32 },
    ModifyRange { name: String, new_range: Range<f32> },
}

// Example: Add new dimension and reweight existing ones
let reconfig = ReconfigureSpaceCommand {
    space_id: "space-ai-ml".into(),
    dimension_changes: vec![
        DimensionChange::Add(DimensionSpec {
            name: "energy_efficiency".to_string(),
            dimension_type: DimensionType::Continuous,
            range: 0.0..1.0,
            weight: 0.7,
            description: "Computational energy efficiency".to_string(),
        }),
        DimensionChange::ModifyWeight {
            name: "complexity".to_string(),
            new_weight: 1.2,
        },
    ],
    recalculate_embeddings: true,
    migration_strategy: MigrationStrategy::Gradual {
        batch_size: 100,
        delay_between_batches: Duration::from_millis(100),
    },
};

client.publish("cmd.conceptual.reconfigure_space", serde_json::to_vec(&reconfig)?.into()).await?;
```

## Concept Operations

### Adding Concepts

#### Add Single Concept
```rust
// Command: cmd.conceptual.add_concept
#[derive(Serialize, Deserialize)]
pub struct AddConceptCommand {
    pub space_id: ConceptualSpaceId,
    pub concept: ConceptDefinition,
    pub auto_position: bool,
    pub explicit_position: Option<ConceptualPoint>,
    pub similarity_threshold: f32,
}

#[derive(Serialize, Deserialize)]
pub struct ConceptDefinition {
    pub name: String,
    pub description: String,
    pub properties: HashMap<String, Value>,
    pub content: Option<String>, // For embedding generation
    pub category: Option<String>,
    pub metadata: HashMap<String, Value>,
}

// NATS Example
let add_concept = AddConceptCommand {
    space_id: "space-ai-ml".into(),
    concept: ConceptDefinition {
        name: "Transformer Architecture".to_string(),
        description: "Attention-based neural network architecture".to_string(),
        properties: HashMap::from([
            ("complexity".into(), Value::Number(0.8.into())),
            ("application_domain".into(), Value::String("NLP".into())),
            ("maturity".into(), Value::Number(4.0.into())),
        ]),
        content: Some("The Transformer is a deep learning model that uses self-attention mechanisms to process sequential data. It has revolutionized natural language processing and forms the basis for large language models like GPT and BERT.".to_string()),
        category: Some("Neural Architecture".to_string()),
        metadata: HashMap::new(),
    },
    auto_position: true, // Let the system calculate optimal position
    explicit_position: None,
    similarity_threshold: 0.7,
};

let response = client.request(
    "cmd.conceptual.add_concept",
    serde_json::to_vec(&add_concept)?.into()
).timeout(Duration::from_secs(5)).await?;
```

#### Batch Add Concepts
```rust
// Command: cmd.conceptual.batch_add_concepts
#[derive(Serialize, Deserialize)]
pub struct BatchAddConceptsCommand {
    pub space_id: ConceptualSpaceId,
    pub concepts: Vec<ConceptDefinition>,
    pub auto_position_all: bool,
    pub calculate_similarities: bool,
    pub form_initial_categories: bool,
}

// Load concepts from external source
let concepts = load_concepts_from_wikipedia_categories(vec![
    "Machine Learning",
    "Deep Learning", 
    "Natural Language Processing",
    "Computer Vision",
]);

let batch_add = BatchAddConceptsCommand {
    space_id: "space-ai-ml".into(),
    concepts,
    auto_position_all: true,
    calculate_similarities: true,
    form_initial_categories: true,
};

let response = client.request(
    "cmd.conceptual.batch_add_concepts",
    serde_json::to_vec(&batch_add)?.into()
).timeout(Duration::from_secs(30)).await?;

// Monitor batch processing progress
let mut progress_subscriber = client.subscribe("event.conceptual.batch_progress").await?;
while let Some(message) = progress_subscriber.next().await {
    let progress: BatchProgressEvent = serde_json::from_slice(&message.payload)?;
    println!("Batch progress: {}/{} concepts processed", 
        progress.processed_count, progress.total_count);
    
    if progress.completed {
        println!("Batch processing completed!");
        break;
    }
}
```

## Embedding Operations

### Embedding Calculation

#### Generate Embeddings
```rust
// Command: cmd.conceptual.calculate_embeddings
#[derive(Serialize, Deserialize)]
pub struct CalculateEmbeddingsCommand {
    pub space_id: ConceptualSpaceId,
    pub concept_ids: Option<Vec<ConceptId>>, // None = all concepts
    pub embedding_model: Option<String>,
    pub recalculate_existing: bool,
    pub batch_size: u32,
}

// Calculate embeddings for new concepts
let calc_embeddings = CalculateEmbeddingsCommand {
    space_id: "space-ai-ml".into(),
    concept_ids: None, // Process all concepts
    embedding_model: Some("sentence-transformers/all-mpnet-base-v2".to_string()),
    recalculate_existing: false,
    batch_size: 50,
};

client.publish("cmd.conceptual.calculate_embeddings", serde_json::to_vec(&calc_embeddings)?.into()).await?;

// Subscribe to embedding calculation events
let mut embedding_subscriber = client.subscribe("event.conceptual.embedding_calculated").await?;
while let Some(message) = embedding_subscriber.next().await {
    let event: EmbeddingCalculatedEvent = serde_json::from_slice(&message.payload)?;
    println!("Embedding calculated for concept: {} (confidence: {:.2})", 
        event.concept_id, event.confidence_score);
}
```

#### Custom Embedding Integration
```rust
// Command: cmd.conceptual.import_embeddings
#[derive(Serialize, Deserialize)]
pub struct ImportEmbeddingsCommand {
    pub space_id: ConceptualSpaceId,
    pub embeddings: Vec<ConceptEmbedding>,
    pub embedding_model: String,
    pub dimension_mapping: DimensionMapping,
    pub validation_strategy: ValidationStrategy,
}

#[derive(Serialize, Deserialize)]
pub struct ConceptEmbedding {
    pub concept_id: ConceptId,
    pub vector: Vec<f32>,
    pub metadata: EmbeddingMetadata,
}

// Import pre-calculated embeddings from external system
let import_embeddings = ImportEmbeddingsCommand {
    space_id: "space-ai-ml".into(),
    embeddings: load_embeddings_from_file("ai_concepts.embeddings")?,
    embedding_model: "custom-ai-model-v2".to_string(),
    dimension_mapping: DimensionMapping::Explicit(vec![
        ("complexity".to_string(), 0..64),
        ("application_domain".to_string(), 64..128),
        ("maturity".to_string(), 128..192),
    ]),
    validation_strategy: ValidationStrategy::CrossValidation { folds: 5 },
};

client.publish("cmd.conceptual.import_embeddings", serde_json::to_vec(&import_embeddings)?.into()).await?;
```

## Similarity Operations

### Similarity Calculations

#### Find Similar Concepts
```rust
// Query: query.conceptual.find_similar
#[derive(Serialize, Deserialize)]
pub struct FindSimilarQuery {
    pub space_id: ConceptualSpaceId,
    pub target_concept: ConceptId,
    pub similarity_metric: SimilarityMetric,
    pub threshold: f32,
    pub max_results: u32,
    pub include_scores: bool,
    pub filter_criteria: Option<Vec<ConceptFilter>>,
}

// NATS Example
let similarity_query = FindSimilarQuery {
    space_id: "space-ai-ml".into(),
    target_concept: "concept-transformer".into(),
    similarity_metric: SimilarityMetric::Cosine,
    threshold: 0.7,
    max_results: 10,
    include_scores: true,
    filter_criteria: Some(vec![
        ConceptFilter::ByCategory("Neural Architecture".to_string()),
        ConceptFilter::ByProperty {
            key: "maturity".to_string(),
            min_value: Some(3.0),
            max_value: None,
        },
    ]),
};

let response = client.request(
    "query.conceptual.find_similar",
    serde_json::to_vec(&similarity_query)?.into()
).timeout(Duration::from_secs(5)).await?;

let result: QueryResult<Vec<SimilarityMatch>> = serde_json::from_slice(&response.payload)?;
for match_item in result.data {
    println!("Similar concept: {} (score: {:.3})", 
        match_item.concept_id, match_item.similarity_score);
}
```

#### Compute Similarity Matrix
```rust
// Query: query.conceptual.similarity_matrix
#[derive(Serialize, Deserialize)]
pub struct SimilarityMatrixQuery {
    pub space_id: ConceptualSpaceId,
    pub concept_ids: Vec<ConceptId>,
    pub similarity_metric: SimilarityMetric,
    pub symmetric: bool,
    pub sparse_threshold: Option<f32>, // Only include values above threshold
}

// Calculate similarity matrix for concept clustering
let matrix_query = SimilarityMatrixQuery {
    space_id: "space-ai-ml".into(),
    concept_ids: get_all_concept_ids(), // Get from previous query
    similarity_metric: SimilarityMetric::WeightedEuclidean,
    symmetric: true,
    sparse_threshold: Some(0.5), // Only similarities > 0.5
};

let response = client.request(
    "query.conceptual.similarity_matrix",
    serde_json::to_vec(&matrix_query)?.into()
).timeout(Duration::from_secs(15)).await?;

let matrix: QueryResult<SimilarityMatrix> = serde_json::from_slice(&response.payload)?;
```

## Category Formation

### Automatic Categorization

#### Discover Categories
```rust
// Command: cmd.conceptual.discover_categories
#[derive(Serialize, Deserialize)]
pub struct DiscoverCategoriesCommand {
    pub space_id: ConceptualSpaceId,
    pub clustering_algorithm: ClusteringAlgorithm,
    pub parameters: ClusteringParameters,
    pub min_category_size: u32,
    pub max_categories: Option<u32>,
    pub validation_criteria: CategoryValidationCriteria,
}

#[derive(Serialize, Deserialize)]
pub enum ClusteringAlgorithm {
    KMeans { k: u32 },
    DBSCAN { epsilon: f32, min_points: u32 },
    HierarchicalClustering { linkage: LinkageType },
    GaussianMixture { components: u32 },
}

// Discover natural categories in the conceptual space
let discover_categories = DiscoverCategoriesCommand {
    space_id: "space-ai-ml".into(),
    clustering_algorithm: ClusteringAlgorithm::DBSCAN {
        epsilon: 0.3,
        min_points: 3,
    },
    parameters: ClusteringParameters {
        use_all_dimensions: true,
        normalize_dimensions: true,
        random_seed: Some(42),
    },
    min_category_size: 3,
    max_categories: Some(20),
    validation_criteria: CategoryValidationCriteria {
        min_coherence: 0.6,
        max_overlap: 0.2,
        require_convexity: true,
    },
};

let response = client.request(
    "cmd.conceptual.discover_categories",
    serde_json::to_vec(&discover_categories)?.into()
).timeout(Duration::from_secs(30)).await?;

// Monitor category discovery progress
let mut discovery_subscriber = client.subscribe("event.conceptual.category_discovery_progress").await?;
while let Some(message) = discovery_subscriber.next().await {
    let progress: CategoryDiscoveryProgressEvent = serde_json::from_slice(&message.payload)?;
    println!("Discovery progress: {} categories found, coherence: {:.2}", 
        progress.categories_found, progress.average_coherence);
}
```

#### Manual Category Creation
```rust
// Command: cmd.conceptual.create_category
#[derive(Serialize, Deserialize)]
pub struct CreateCategoryCommand {
    pub space_id: ConceptualSpaceId,
    pub category_name: String,
    pub description: String,
    pub member_concepts: Vec<ConceptId>,
    pub prototype_concept: Option<ConceptId>,
    pub boundary_method: BoundaryMethod,
}

#[derive(Serialize, Deserialize)]
pub enum BoundaryMethod {
    ConvexHull,
    MinimumBoundingSphere,
    ConceptualRegion { margin: f32 },
    ExplicitBoundary { boundaries: Vec<Hyperplane> },
}

// Create a category for deep learning concepts
let create_category = CreateCategoryCommand {
    space_id: "space-ai-ml".into(),
    category_name: "Deep Learning Architectures".to_string(),
    description: "Neural network architectures with multiple layers".to_string(),
    member_concepts: vec![
        "concept-transformer".into(),
        "concept-cnn".into(),
        "concept-rnn".into(),
        "concept-gan".into(),
    ],
    prototype_concept: Some("concept-transformer".into()),
    boundary_method: BoundaryMethod::ConvexHull,
};

client.publish("cmd.conceptual.create_category", serde_json::to_vec(&create_category)?.into()).await?;
```

## Conceptual Navigation

### Path Finding in Conceptual Space

#### Find Conceptual Paths
```rust
// Query: query.conceptual.find_path
#[derive(Serialize, Deserialize)]
pub struct FindConceptualPathQuery {
    pub space_id: ConceptualSpaceId,
    pub start_concept: ConceptId,
    pub end_concept: ConceptId,
    pub path_type: PathType,
    pub constraints: Vec<PathConstraint>,
    pub max_path_length: u32,
}

#[derive(Serialize, Deserialize)]
pub enum PathType {
    DirectDistance,     // Straight line in conceptual space
    SemanticPath,       // Through semantically related concepts
    CategoryPath,       // Through category boundaries
    MinimalTransition,  // Minimize dimension changes
}

// Find semantic path between concepts
let path_query = FindConceptualPathQuery {
    space_id: "space-ai-ml".into(),
    start_concept: "concept-linear-regression".into(),
    end_concept: "concept-transformer".into(),
    path_type: PathType::SemanticPath,
    constraints: vec![
        PathConstraint::AvoidCategory("Deprecated Methods".to_string()),
        PathConstraint::PreferDimension("complexity".to_string()),
    ],
    max_path_length: 5,
};

let response = client.request(
    "query.conceptual.find_path",
    serde_json::to_vec(&path_query)?.into()
).timeout(Duration::from_secs(10)).await?;

let path: QueryResult<ConceptualPath> = serde_json::from_slice(&response.payload)?;
for step in path.data.steps {
    println!("Path step: {} -> {} (distance: {:.3})", 
        step.from_concept, step.to_concept, step.distance);
}
```

#### Navigate Conceptual Neighborhoods
```rust
// Query: query.conceptual.explore_neighborhood
#[derive(Serialize, Deserialize)]
pub struct ExploreNeighborhoodQuery {
    pub space_id: ConceptualSpaceId,
    pub center_concept: ConceptId,
    pub exploration_radius: f32,
    pub exploration_strategy: ExplorationStrategy,
    pub include_categories: bool,
    pub max_concepts: u32,
}

#[derive(Serialize, Deserialize)]
pub enum ExplorationStrategy {
    RadialExpansion,         // Expand outward uniformly
    DimensionGuided,         // Follow specific dimensions
    SimilarityBased,         // Follow similarity gradients
    RandomWalk { steps: u32 }, // Random exploration
}

// Explore the neighborhood around a concept
let explore_query = ExploreNeighborhoodQuery {
    space_id: "space-ai-ml".into(),
    center_concept: "concept-neural-network".into(),
    exploration_radius: 2.0,
    exploration_strategy: ExplorationStrategy::SimilarityBased,
    include_categories: true,
    max_concepts: 20,
};

let response = client.request(
    "query.conceptual.explore_neighborhood",
    serde_json::to_vec(&explore_query)?.into()
).timeout(Duration::from_secs(8)).await?;

let neighborhood: QueryResult<ConceptualNeighborhood> = serde_json::from_slice(&response.payload)?;
```

## AI Agent Integration

### Agent-Driven Conceptual Operations

#### Agent Concept Learning
```rust
// Command: cmd.conceptual.agent_learn_concept
#[derive(Serialize, Deserialize)]
pub struct AgentLearnConceptCommand {
    pub space_id: ConceptualSpaceId,
    pub agent_id: AgentId,
    pub learning_source: LearningSource,
    pub confidence_threshold: f32,
    pub integration_strategy: IntegrationStrategy,
}

#[derive(Serialize, Deserialize)]
pub enum LearningSource {
    Text { content: String },
    Examples { positive: Vec<String>, negative: Vec<String> },
    Interaction { user_feedback: Vec<FeedbackItem> },
    Knowledge { structured_data: serde_json::Value },
}

// Agent learns new concept from interaction
let agent_learning = AgentLearnConceptCommand {
    space_id: "space-ai-ml".into(),
    agent_id: "agent-knowledge-curator".into(),
    learning_source: LearningSource::Interaction {
        user_feedback: vec![
            FeedbackItem {
                concept_pair: ("concept-a".into(), "concept-b".into()),
                similarity_adjustment: 0.2,
                reason: "User indicated these are more similar than computed".to_string(),
            },
        ],
    },
    confidence_threshold: 0.8,
    integration_strategy: IntegrationStrategy::GradualUpdate {
        learning_rate: 0.1,
        validation_required: true,
    },
};

client.publish("cmd.conceptual.agent_learn_concept", serde_json::to_vec(&agent_learning)?.into()).await?;
```

#### Agent-Suggested Categories
```rust
// Command: cmd.conceptual.agent_suggest_categories
#[derive(Serialize, Deserialize)]
pub struct AgentSuggestCategoriesCommand {
    pub space_id: ConceptualSpaceId,
    pub agent_id: AgentId,
    pub analysis_scope: AnalysisScope,
    pub suggestion_criteria: SuggestionCriteria,
    pub max_suggestions: u32,
}

#[derive(Serialize, Deserialize)]
pub struct SuggestionCriteria {
    pub min_coherence: f32,
    pub prefer_interpretable: bool,
    pub align_with_existing: bool,
    pub optimize_for: OptimizationTarget,
}

// Agent suggests new category structure
let agent_suggestion = AgentSuggestCategoriesCommand {
    space_id: "space-ai-ml".into(),
    agent_id: "agent-category-optimizer".into(),
    analysis_scope: AnalysisScope::EntireSpace,
    suggestion_criteria: SuggestionCriteria {
        min_coherence: 0.7,
        prefer_interpretable: true,
        align_with_existing: false,
        optimize_for: OptimizationTarget::MaximizeCoherence,
    },
    max_suggestions: 5,
};

let response = client.request(
    "cmd.conceptual.agent_suggest_categories",
    serde_json::to_vec(&agent_suggestion)?.into()
).timeout(Duration::from_secs(20)).await?;

// Process agent suggestions
let suggestions: CommandResult = serde_json::from_slice(&response.payload)?;
if let CommandResult::Success { .. } = suggestions {
    // Subscribe to suggestion events
    let mut suggestion_subscriber = client.subscribe("event.conceptual.category_suggested").await?;
    while let Some(message) = suggestion_subscriber.next().await {
        let suggestion: CategorySuggestionEvent = serde_json::from_slice(&message.payload)?;
        println!("Agent suggests category: {} (coherence: {:.2})", 
            suggestion.category_name, suggestion.coherence_score);
    }
}
```

## Real-Time Conceptual Updates

### Live Space Monitoring

#### Subscribe to Conceptual Changes
```rust
// Subscribe to all conceptual space events
let mut conceptual_subscriber = client.subscribe("event.conceptual.>").await?;

while let Some(message) = conceptual_subscriber.next().await {
    match message.subject.as_str() {
        "event.conceptual.concept_added" => {
            let event: ConceptAddedEvent = serde_json::from_slice(&message.payload)?;
            println!("New concept added: {} to space {}", 
                event.concept_id, event.space_id);
            update_visualization_with_new_concept(event).await?;
        }
        "event.conceptual.similarity_updated" => {
            let event: SimilarityUpdatedEvent = serde_json::from_slice(&message.payload)?;
            println!("Similarity updated: {} <-> {} = {:.3}", 
                event.concept_a, event.concept_b, event.new_similarity);
            update_edge_weights(event).await?;
        }
        "event.conceptual.category_formed" => {
            let event: CategoryFormedEvent = serde_json::from_slice(&message.payload)?;
            println!("New category formed: {} with {} members", 
                event.category_name, event.member_count);
            create_category_visualization(event).await?;
        }
        "event.conceptual.space_recalibrated" => {
            let event: SpaceRecalibratedEvent = serde_json::from_slice(&message.payload)?;
            println!("Space recalibrated: {} dimensions affected", 
                event.affected_dimensions.len());
            refresh_entire_space_visualization(event).await?;
        }
        _ => {}
    }
}
```

### Performance Monitoring

#### Monitor Conceptual Operations
```rust
// Subscribe to performance metrics
let mut perf_subscriber = client.subscribe("event.conceptual.performance.*").await?;

while let Some(message) = perf_subscriber.next().await {
    match message.subject.as_str() {
        "event.conceptual.performance.embedding_calculation" => {
            let event: EmbeddingPerformanceEvent = serde_json::from_slice(&message.payload)?;
            if event.calculation_time_ms > 1000 {
                log::warn!("Slow embedding calculation: {}ms for {} concepts", 
                    event.calculation_time_ms, event.concept_count);
            }
        }
        "event.conceptual.performance.similarity_search" => {
            let event: SimilaritySearchPerformanceEvent = serde_json::from_slice(&message.payload)?;
            log::info!("Similarity search: {}ms for {} comparisons", 
                event.search_time_ms, event.comparisons_made);
        }
        _ => {}
    }
}
```

---

**All conceptual space operations in CIM use NATS messaging to enable distributed semantic processing, real-time AI agent collaboration, and scalable knowledge representation across the entire system.** 