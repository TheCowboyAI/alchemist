# IPLD Relationships

> Part of the [CID/IPLD Architecture](./cid-ipld-architecture.md)

## Overview

CIM uses IPLD relationships to create a rich, interconnected graph of content. These relationships enable semantic navigation, provenance tracking, and intelligent content discovery. All relationships are bidirectional and indexed for efficient traversal.

## Relationship Model

### Core Relationship Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpldRelationship {
    pub relationship_id: Cid,
    pub source: Cid,
    pub target: Cid,
    pub predicate: RelationshipPredicate,
    pub metadata: RelationshipMetadata,
    pub created_at: SystemTime,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipMetadata {
    pub provenance: Provenance,
    pub attributes: HashMap<String, Value>,
    pub bidirectional: bool,
    pub strength: f32,
    pub context: Option<Cid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Provenance {
    UserDefined {
        user_id: String,
        timestamp: SystemTime,
    },
    SystemInferred {
        algorithm: String,
        confidence: f32,
    },
    Imported {
        source: String,
        import_time: SystemTime,
    },
    Transformation {
        process: String,
        input_cids: Vec<Cid>,
    },
}
```

### Relationship Predicates

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RelationshipPredicate {
    // Structural relationships
    Contains,           // Parent contains child
    PartOf,            // Child is part of parent
    References,        // Document references another
    Links,             // Hyperlink or cross-reference
    Includes,          // Includes as dependency

    // Semantic relationships
    SimilarTo(f32),    // Semantic similarity with score
    RelatedTo,         // General relationship
    DescribedBy,       // Metadata or documentation
    AnnotatedBy,       // Annotations or comments
    TaggedWith,        // Tags or categories

    // Temporal relationships
    Precedes,          // Temporal ordering
    Follows,           // Reverse of precedes
    ConcurrentWith,    // Happened at same time
    DerivedFrom,       // Created from source
    UpdateOf,          // New version of content

    // Transformation relationships
    TransformedFrom,   // Result of transformation
    TransformedTo,     // Source of transformation
    GeneratedBy,       // Created by process
    ProcessedBy,       // Modified by process
    ExtractedFrom,     // Extracted content

    // Version relationships
    PreviousVersion,   // Version chain
    NextVersion,       // Forward version
    BranchOf,          // Branched from version
    MergeOf,           // Merged from versions
    ConflictsWith,     // Version conflict

    // Custom relationships
    Custom(String),    // Domain-specific predicates
}

impl RelationshipPredicate {
    pub fn inverse(&self) -> Option<RelationshipPredicate> {
        match self {
            RelationshipPredicate::Contains => Some(RelationshipPredicate::PartOf),
            RelationshipPredicate::PartOf => Some(RelationshipPredicate::Contains),
            RelationshipPredicate::Precedes => Some(RelationshipPredicate::Follows),
            RelationshipPredicate::Follows => Some(RelationshipPredicate::Precedes),
            RelationshipPredicate::TransformedFrom => Some(RelationshipPredicate::TransformedTo),
            RelationshipPredicate::TransformedTo => Some(RelationshipPredicate::TransformedFrom),
            RelationshipPredicate::PreviousVersion => Some(RelationshipPredicate::NextVersion),
            RelationshipPredicate::NextVersion => Some(RelationshipPredicate::PreviousVersion),
            _ => None,
        }
    }

    pub fn is_transitive(&self) -> bool {
        matches!(self,
            RelationshipPredicate::Contains |
            RelationshipPredicate::PartOf |
            RelationshipPredicate::Precedes |
            RelationshipPredicate::Follows |
            RelationshipPredicate::DerivedFrom
        )
    }
}
```

## Relationship Index

### Index Structure

```rust
pub struct RelationshipIndex {
    // Forward index: source -> [(predicate, target)]
    forward_index: HashMap<Cid, Vec<(RelationshipPredicate, Cid)>>,

    // Reverse index: target -> [(predicate, source)]
    reverse_index: HashMap<Cid, Vec<(RelationshipPredicate, Cid)>>,

    // Predicate index: predicate -> [(source, target)]
    predicate_index: HashMap<RelationshipPredicate, Vec<(Cid, Cid)>>,

    // Metadata storage
    relationship_metadata: HashMap<(Cid, RelationshipPredicate, Cid), RelationshipMetadata>,
}

impl RelationshipIndex {
    pub fn add_relationship(
        &mut self,
        source: Cid,
        predicate: RelationshipPredicate,
        target: Cid,
        metadata: RelationshipMetadata,
    ) -> Result<()> {
        // Add to forward index
        self.forward_index
            .entry(source)
            .or_insert_with(Vec::new)
            .push((predicate.clone(), target));

        // Add to reverse index
        self.reverse_index
            .entry(target)
            .or_insert_with(Vec::new)
            .push((predicate.clone(), source));

        // Add to predicate index
        self.predicate_index
            .entry(predicate.clone())
            .or_insert_with(Vec::new)
            .push((source, target));

        // Store metadata
        self.relationship_metadata
            .insert((source, predicate.clone(), target), metadata.clone());

        // If bidirectional, add inverse relationship
        if metadata.bidirectional {
            if let Some(inverse) = predicate.inverse() {
                self.add_relationship(target, inverse, source, metadata)?;
            }
        }

        Ok(())
    }

    pub fn find_related(
        &self,
        cid: Cid,
        predicate: Option<RelationshipPredicate>,
        direction: Direction,
    ) -> Vec<(Cid, RelationshipPredicate, RelationshipMetadata)> {
        match direction {
            Direction::Outgoing => {
                self.find_outgoing_relationships(cid, predicate)
            }
            Direction::Incoming => {
                self.find_incoming_relationships(cid, predicate)
            }
            Direction::Both => {
                let mut results = self.find_outgoing_relationships(cid, predicate.clone());
                results.extend(self.find_incoming_relationships(cid, predicate));
                results
            }
        }
    }

    pub fn find_path(
        &self,
        from: Cid,
        to: Cid,
        max_depth: usize,
        allowed_predicates: Option<Vec<RelationshipPredicate>>,
    ) -> Option<Vec<(Cid, RelationshipPredicate, Cid)>> {
        // BFS to find shortest path
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent_map = HashMap::new();

        queue.push_back((from, 0));
        visited.insert(from);

        while let Some((current, depth)) = queue.pop_front() {
            if current == to {
                return Some(self.reconstruct_path(&parent_map, from, to));
            }

            if depth >= max_depth {
                continue;
            }

            // Explore neighbors
            if let Some(relationships) = self.forward_index.get(&current) {
                for (predicate, target) in relationships {
                    if let Some(ref allowed) = allowed_predicates {
                        if !allowed.contains(predicate) {
                            continue;
                        }
                    }

                    if !visited.contains(target) {
                        visited.insert(*target);
                        parent_map.insert(*target, (current, predicate.clone()));
                        queue.push_back((*target, depth + 1));
                    }
                }
            }
        }

        None
    }
}
```

### Relationship Queries

```rust
pub struct RelationshipQuery {
    pub source: Option<Cid>,
    pub target: Option<Cid>,
    pub predicates: Option<Vec<RelationshipPredicate>>,
    pub min_confidence: Option<f32>,
    pub max_depth: Option<usize>,
    pub include_transitive: bool,
}

impl RelationshipIndex {
    pub fn query(&self, query: RelationshipQuery) -> Vec<IpldRelationship> {
        let mut results = Vec::new();

        // Direct relationships
        if let Some(source) = query.source {
            results.extend(self.query_from_source(source, &query));
        } else if let Some(target) = query.target {
            results.extend(self.query_from_target(target, &query));
        } else if let Some(predicates) = &query.predicates {
            results.extend(self.query_by_predicates(predicates, &query));
        }

        // Transitive closure if requested
        if query.include_transitive {
            results.extend(self.compute_transitive_closure(&results, &query));
        }

        // Filter by confidence
        if let Some(min_confidence) = query.min_confidence {
            results.retain(|r| r.confidence >= min_confidence);
        }

        results
    }

    fn compute_transitive_closure(
        &self,
        direct_relationships: &[IpldRelationship],
        query: &RelationshipQuery,
    ) -> Vec<IpldRelationship> {
        let mut transitive_results = Vec::new();
        let max_depth = query.max_depth.unwrap_or(5);

        for relationship in direct_relationships {
            if relationship.predicate.is_transitive() {
                let mut current = vec![relationship.target];
                let mut depth = 1;

                while depth < max_depth && !current.is_empty() {
                    let mut next_level = Vec::new();

                    for cid in current {
                        if let Some(relationships) = self.forward_index.get(&cid) {
                            for (pred, target) in relationships {
                                if pred == &relationship.predicate {
                                    next_level.push(*target);

                                    // Create transitive relationship
                                    transitive_results.push(IpldRelationship {
                                        relationship_id: Cid::default(), // Generated
                                        source: relationship.source,
                                        target: *target,
                                        predicate: relationship.predicate.clone(),
                                        metadata: RelationshipMetadata {
                                            provenance: Provenance::SystemInferred {
                                                algorithm: "transitive_closure".to_string(),
                                                confidence: relationship.confidence * 0.9_f32.powi(depth),
                                            },
                                            attributes: HashMap::new(),
                                            bidirectional: false,
                                            strength: relationship.metadata.strength * 0.9_f32.powi(depth),
                                            context: Some(relationship.relationship_id),
                                        },
                                        created_at: SystemTime::now(),
                                        confidence: relationship.confidence * 0.9_f32.powi(depth),
                                    });
                                }
                            }
                        }
                    }

                    current = next_level;
                    depth += 1;
                }
            }
        }

        transitive_results
    }
}
```

## Semantic Relationships

### Similarity Detection

```rust
pub struct SemanticRelationshipDetector {
    embedding_index: Arc<EmbeddingIndex>,
    similarity_threshold: f32,
}

impl SemanticRelationshipDetector {
    pub async fn detect_similar_content(
        &self,
        cid: Cid,
        limit: usize,
    ) -> Result<Vec<(Cid, f32)>> {
        // Get embedding for source content
        let source_embedding = self.embedding_index.get_embedding(cid).await?;

        // Find nearest neighbors
        let neighbors = self.embedding_index
            .find_nearest(&source_embedding, limit + 1, self.similarity_threshold)
            .await?;

        // Filter out self and convert to relationships
        Ok(neighbors
            .into_iter()
            .filter(|(neighbor_cid, _)| *neighbor_cid != cid)
            .collect())
    }

    pub async fn create_similarity_relationships(
        &self,
        cid: Cid,
        index: &mut RelationshipIndex,
    ) -> Result<()> {
        let similar_content = self.detect_similar_content(cid, 10).await?;

        for (similar_cid, similarity_score) in similar_content {
            let metadata = RelationshipMetadata {
                provenance: Provenance::SystemInferred {
                    algorithm: "embedding_similarity".to_string(),
                    confidence: similarity_score,
                },
                attributes: hashmap! {
                    "similarity_score".to_string() => json!(similarity_score),
                    "embedding_model".to_string() => json!("text-embedding-ada-002"),
                },
                bidirectional: true,
                strength: similarity_score,
                context: None,
            };

            index.add_relationship(
                cid,
                RelationshipPredicate::SimilarTo(similarity_score),
                similar_cid,
                metadata,
            )?;
        }

        Ok(())
    }
}
```

### Topic Clustering

```rust
pub struct TopicClusterDetector {
    clustering_algorithm: Box<dyn ClusteringAlgorithm>,
    min_cluster_size: usize,
}

impl TopicClusterDetector {
    pub async fn detect_topic_clusters(
        &self,
        cids: &[Cid],
        embedding_index: &EmbeddingIndex,
    ) -> Result<Vec<TopicCluster>> {
        // Get embeddings for all content
        let mut embeddings = Vec::new();
        for cid in cids {
            let embedding = embedding_index.get_embedding(*cid).await?;
            embeddings.push((*cid, embedding));
        }

        // Run clustering algorithm
        let clusters = self.clustering_algorithm.cluster(&embeddings)?;

        // Convert to topic clusters
        let mut topic_clusters = Vec::new();
        for (cluster_id, cluster_cids) in clusters {
            if cluster_cids.len() >= self.min_cluster_size {
                let cluster = TopicCluster {
                    cluster_id: Cid::default(), // Generated
                    topic_name: self.generate_topic_name(&cluster_cids).await?,
                    members: cluster_cids,
                    centroid: self.calculate_centroid(&embeddings, &cluster_cids),
                    coherence_score: self.calculate_coherence(&embeddings, &cluster_cids),
                };
                topic_clusters.push(cluster);
            }
        }

        Ok(topic_clusters)
    }

    pub async fn create_cluster_relationships(
        &self,
        clusters: &[TopicCluster],
        index: &mut RelationshipIndex,
    ) -> Result<()> {
        for cluster in clusters {
            // Create relationships between cluster members
            for i in 0..cluster.members.len() {
                for j in i + 1..cluster.members.len() {
                    let metadata = RelationshipMetadata {
                        provenance: Provenance::SystemInferred {
                            algorithm: "topic_clustering".to_string(),
                            confidence: cluster.coherence_score,
                        },
                        attributes: hashmap! {
                            "cluster_id".to_string() => json!(cluster.cluster_id),
                            "topic".to_string() => json!(cluster.topic_name),
                        },
                        bidirectional: true,
                        strength: cluster.coherence_score,
                        context: Some(cluster.cluster_id),
                    };

                    index.add_relationship(
                        cluster.members[i],
                        RelationshipPredicate::RelatedTo,
                        cluster.members[j],
                        metadata,
                    )?;
                }
            }
        }

        Ok(())
    }
}
```

## Provenance Tracking

### Transformation Chain

```rust
pub struct ProvenanceTracker {
    relationship_index: Arc<RwLock<RelationshipIndex>>,
}

impl ProvenanceTracker {
    pub async fn track_transformation(
        &self,
        input_cids: Vec<Cid>,
        output_cid: Cid,
        transformation: TransformationMetadata,
    ) -> Result<()> {
        let mut index = self.relationship_index.write().await;

        for input_cid in input_cids {
            let metadata = RelationshipMetadata {
                provenance: Provenance::Transformation {
                    process: transformation.process_name.clone(),
                    input_cids: vec![input_cid],
                },
                attributes: hashmap! {
                    "transformation_id".to_string() => json!(transformation.id),
                    "timestamp".to_string() => json!(transformation.timestamp),
                    "parameters".to_string() => json!(transformation.parameters),
                },
                bidirectional: false,
                strength: 1.0,
                context: None,
            };

            index.add_relationship(
                output_cid,
                RelationshipPredicate::TransformedFrom,
                input_cid,
                metadata,
            )?;
        }

        Ok(())
    }

    pub async fn get_provenance_chain(
        &self,
        cid: Cid,
        max_depth: usize,
    ) -> Result<ProvenanceChain> {
        let index = self.relationship_index.read().await;
        let mut chain = ProvenanceChain::new(cid);
        let mut queue = VecDeque::new();
        queue.push_back((cid, 0));

        while let Some((current_cid, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            // Find all transformation sources
            let relationships = index.find_related(
                current_cid,
                Some(RelationshipPredicate::TransformedFrom),
                Direction::Outgoing,
            );

            for (source_cid, predicate, metadata) in relationships {
                chain.add_step(ProvenanceStep {
                    from: source_cid,
                    to: current_cid,
                    transformation: extract_transformation(&metadata),
                    timestamp: extract_timestamp(&metadata),
                });

                queue.push_back((source_cid, depth + 1));
            }
        }

        Ok(chain)
    }
}
```

## Relationship Visualization

### Graph Generation

```rust
pub struct RelationshipGraphGenerator {
    layout_engine: Box<dyn GraphLayoutEngine>,
    style_config: GraphStyleConfig,
}

impl RelationshipGraphGenerator {
    pub fn generate_graph(
        &self,
        root_cid: Cid,
        index: &RelationshipIndex,
        depth: usize,
        filter: Option<RelationshipFilter>,
    ) -> RelationshipGraph {
        let mut graph = RelationshipGraph::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((root_cid, 0));
        visited.insert(root_cid);

        while let Some((current_cid, current_depth)) = queue.pop_front() {
            if current_depth >= depth {
                continue;
            }

            // Add node
            graph.add_node(GraphNode {
                cid: current_cid,
                depth: current_depth,
                attributes: self.get_node_attributes(current_cid),
            });

            // Get relationships
            let relationships = index.find_related(
                current_cid,
                None,
                Direction::Both,
            );

            for (related_cid, predicate, metadata) in relationships {
                // Apply filter if provided
                if let Some(ref filter) = filter {
                    if !filter.should_include(&predicate, &metadata) {
                        continue;
                    }
                }

                // Add edge
                graph.add_edge(GraphEdge {
                    source: current_cid,
                    target: related_cid,
                    predicate: predicate.clone(),
                    metadata: metadata.clone(),
                    style: self.get_edge_style(&predicate),
                });

                // Queue for exploration
                if !visited.contains(&related_cid) {
                    visited.insert(related_cid);
                    queue.push_back((related_cid, current_depth + 1));
                }
            }
        }

        // Apply layout
        self.layout_engine.layout(&mut graph);

        graph
    }
}
```

## Related Documents

- [Core Architecture](./cid-ipld-core.md) - CID and IPLD basics
- [Content Types](./cid-ipld-content-types.md) - Type definitions
- [Business Intelligence](./cid-ipld-business-intelligence.md) - Relationship analytics
- [Content Transformations](./cid-ipld-transformations.md) - Transformation tracking

## Next Steps

1. Define custom relationship predicates for your domain
2. Configure semantic similarity detection
3. Set up provenance tracking for transformations
4. Implement relationship visualization
