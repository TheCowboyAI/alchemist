# ConceptualSpaces Domain

## Overview

The ConceptualSpaces Domain implements geometric representations of knowledge and concepts, enabling semantic reasoning and similarity-based operations. Based on cognitive science principles, it represents concepts as regions in multi-dimensional spaces where distance corresponds to semantic similarity.

## Key Concepts

### Conceptual Space
- **Definition**: A geometric structure with quality dimensions representing properties
- **Components**: Dimensions, regions, points, metrics
- **Properties**: Dimensionality, metric type, boundaries
- **Applications**: Knowledge representation, similarity search, categorization

### Quality Dimension
- **Definition**: A measurable aspect of concepts (color, size, temperature)
- **Types**: Continuous, categorical, ordinal, circular
- **Properties**: Name, range, metric, weight
- **Examples**: Hue (circular), size (continuous), category (categorical)

### Conceptual Point
- **Definition**: A specific location in conceptual space representing an instance
- **Properties**: Coordinates, concept reference, confidence
- **Operations**: Distance calculation, projection, transformation

### Convex Region
- **Definition**: A bounded area representing a natural category
- **Properties**: Prototype (central point), boundaries, member points
- **Principle**: Points between any two members are also members

## Domain Events

### Commands
- `cmd.conceptualspaces.create_space` - Define new conceptual space
- `cmd.conceptualspaces.add_concept` - Place concept in space
- `cmd.conceptualspaces.form_region` - Create category region
- `cmd.conceptualspaces.calculate_similarity` - Compute distance
- `cmd.conceptualspaces.project_concept` - Transform to subspace

### Events
- `event.conceptualspaces.space_created` - New space defined
- `event.conceptualspaces.concept_added` - Concept positioned
- `event.conceptualspaces.region_formed` - Category created
- `event.conceptualspaces.similarity_calculated` - Distance computed
- `event.conceptualspaces.concept_moved` - Position updated

### Queries
- `query.conceptualspaces.find_similar` - Nearest neighbors
- `query.conceptualspaces.categorize` - Determine category
- `query.conceptualspaces.get_prototype` - Category center
- `query.conceptualspaces.calculate_path` - Semantic navigation

## API Reference

### ConceptualSpaceAggregate
```rust
pub struct ConceptualSpaceAggregate {
    pub id: SpaceId,
    pub name: String,
    pub dimensions: Vec<QualityDimension>,
    pub regions: HashMap<RegionId, ConvexRegion>,
    pub points: HashMap<ConceptId, ConceptualPoint>,
    pub metric: DistanceMetric,
}
```

### Key Methods
- `create_space()` - Initialize conceptual space
- `add_dimension()` - Define quality dimension
- `place_concept()` - Position in space
- `form_region()` - Create category from examples
- `find_nearest()` - Similarity search

## Conceptual Space Examples

### Color Space
```rust
// Define RGB color space
let color_space = ConceptualSpaceBuilder::new("Colors")
    .add_dimension(QualityDimension {
        name: "red".to_string(),
        dimension_type: DimensionType::Continuous,
        range: 0.0..1.0,
    })
    .add_dimension(QualityDimension {
        name: "green".to_string(),
        dimension_type: DimensionType::Continuous,
        range: 0.0..1.0,
    })
    .add_dimension(QualityDimension {
        name: "blue".to_string(),
        dimension_type: DimensionType::Continuous,
        range: 0.0..1.0,
    })
    .with_metric(DistanceMetric::Euclidean)
    .build();

// Add color concepts
let red = space.add_concept("red", ConceptualPoint::new(vec![1.0, 0.0, 0.0]));
let yellow = space.add_concept("yellow", ConceptualPoint::new(vec![1.0, 1.0, 0.0]));
let orange = space.add_concept("orange", ConceptualPoint::new(vec![1.0, 0.5, 0.0]));
```

### Semantic Space
```rust
// Product similarity space
let product_space = ConceptualSpaceBuilder::new("Products")
    .add_dimension(QualityDimension {
        name: "price".to_string(),
        dimension_type: DimensionType::Continuous,
        range: 0.0..10000.0,
        weight: 0.3,
    })
    .add_dimension(QualityDimension {
        name: "quality".to_string(),
        dimension_type: DimensionType::Ordinal,
        levels: vec!["low", "medium", "high", "premium"],
        weight: 0.5,
    })
    .add_dimension(QualityDimension {
        name: "category".to_string(),
        dimension_type: DimensionType::Categorical,
        categories: vec!["electronics", "clothing", "food"],
        weight: 0.2,
    })
    .build();
```

## Category Formation

### Prototype Theory
```rust
// Form category from examples
let examples = vec![
    ConceptualPoint::new(vec![0.9, 0.1, 0.1]), // Dark red
    ConceptualPoint::new(vec![1.0, 0.0, 0.0]), // Pure red
    ConceptualPoint::new(vec![0.8, 0.2, 0.2]), // Light red
];

let red_region = space.form_region_from_examples(
    "red_colors",
    &examples,
    RegionFormation::ConvexHull,
);

// Prototype is centroid of examples
let prototype = red_region.prototype(); // ~[0.9, 0.1, 0.1]
```

### Region Operations
```rust
// Test category membership
let test_color = ConceptualPoint::new(vec![0.85, 0.15, 0.15]);
if red_region.contains(&test_color) {
    println!("Color belongs to red category");
}

// Find degree of membership
let membership = red_region.membership_degree(&test_color); // 0.0-1.0

// Merge regions
let pink_region = space.get_region("pink_colors");
let expanded_red = red_region.merge_with(&pink_region);
```

## Similarity and Distance

### Distance Metrics
```rust
pub enum DistanceMetric {
    Euclidean,           // Standard geometric distance
    Manhattan,           // City-block distance
    WeightedEuclidean,   // Dimension importance weights
    Contextual {         // Context-dependent weights
        base: Box<DistanceMetric>,
        context_weights: HashMap<ContextId, Vec<f32>>,
    },
}

// Calculate similarity
let similarity = space.calculate_similarity(&concept_a, &concept_b);

// Find k nearest neighbors
let neighbors = space.find_k_nearest(&query_point, k: 5);
```

### Semantic Navigation
```rust
// Path between concepts
let path = space.find_semantic_path(
    &start_concept,
    &end_concept,
    PathConstraints {
        max_steps: 10,
        avoid_regions: vec![forbidden_region],
        prefer_regions: vec![preferred_region],
    },
);

// Concept interpolation
let intermediate = space.interpolate(&concept_a, &concept_b, t: 0.5);
```

## AI Integration

### Embedding Bridge
```rust
// Convert AI embeddings to conceptual space
let embedding_bridge = EmbeddingBridge::new(
    embedding_model: "text-embedding-3",
    target_space: conceptual_space,
    dimension_mapping: DimensionMapping::PCA(n_components: 10),
);

// Place text in conceptual space
let text = "A bright red sports car";
let embedding = ai_model.embed(text).await?;
let point = embedding_bridge.map_to_space(embedding)?;
space.add_concept(text, point);
```

### Concept Learning
```rust
// Learn from user feedback
let feedback = UserFeedback::SimilarityCorrection {
    concept_a: "apple",
    concept_b: "orange",
    should_be_similar: true,
};

space.adapt_from_feedback(feedback, learning_rate: 0.1)?;

// Discover new categories
let clusters = space.discover_categories(
    min_points: 5,
    max_distance: 0.3,
);
```

## Visualization

### 3D Projection
```rust
// Project high-dimensional space to 3D for visualization
let projection = space.project_to_3d(
    ProjectionMethod::PCA,
    preserve: PreservationGoal::LocalStructure,
);

// Render in Bevy
for (concept_id, point_3d) in projection {
    commands.spawn((
        ConceptNode { id: concept_id },
        Transform::from_translation(point_3d.into()),
        // Visual components...
    ));
}
```

### Interactive Exploration
- Navigate through concept space
- Visualize category boundaries
- Show similarity gradients
- Animate concept movements

## Use Cases

### Knowledge Management
- Document categorization
- Content recommendation
- Semantic search
- Expertise mapping

### Product Design
- Feature space analysis
- Market positioning
- Similarity assessment
- Gap identification

### AI Reasoning
- Concept combination
- Analogy making
- Category learning
- Semantic inference

## Performance Characteristics

- **Dimensionality**: Up to 100 dimensions efficiently
- **Points**: 1M+ concepts per space
- **Query Speed**: <1ms for k-NN with indices
- **Region Operations**: <10ms for membership tests

## Best Practices

1. **Dimension Design**: Choose orthogonal, interpretable dimensions
2. **Metric Selection**: Match metric to domain semantics
3. **Region Convexity**: Maintain convex regions for natural categories
4. **Sparse Spaces**: Use dimensionality reduction for efficiency
5. **Incremental Learning**: Adapt spaces based on usage

## Related Domains

- **Graph Domain**: Conceptual graphs with semantic edges
- **AI Agent Domain**: Reasoning in conceptual spaces
- **Workflow Domain**: Semantic process similarity
- **Document Domain**: Content categorization 