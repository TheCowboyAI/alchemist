# Graph Import Patterns: Technical Reference

## Core Import Patterns

### 1. NATS Message Import Pattern

The NATS integration enables real-time graph construction from message streams:

```rust
/// NATS Subject Hierarchy for Imports
/// - import.request.<format> - Request import of specific format
/// - import.stream.<source> - Continuous import stream
/// - import.cid.<store> - Import from CID reference
/// - import.transform.<morphism> - Apply transformation during import

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRequest {
    pub request_id: Uuid,
    pub source: ImportSource,
    pub format: ImportFormat,
    pub options: ImportOptions,
    pub reply_subject: Option<String>,
}

/// NATS Import Handler
pub struct NatsImportHandler {
    import_service: Arc<GraphImportService>,
    event_store: Arc<EventStore>,
    object_store: Arc<ObjectStore>,
}

impl NatsImportHandler {
    pub async fn start(&self, client: &NatsClient) -> Result<()> {
        // Subscribe to import requests
        let mut sub = client.subscribe("import.request.>").await?;

        while let Some(msg) = sub.next().await {
            let format = extract_format_from_subject(&msg.subject)?;
            let request: ImportRequest = serde_json::from_slice(&msg.data)?;

            // Process import asynchronously
            let result = self.process_import(request, format).await;

            // Send reply if requested
            if let Some(reply) = msg.reply {
                let response = ImportResponse::from(result);
                client.publish(&reply, &serde_json::to_vec(&response)?).await?;
            }
        }

        Ok(())
    }

    async fn process_import(
        &self,
        request: ImportRequest,
        format: ImportFormat,
    ) -> Result<GraphId> {
        // Load content based on source
        let content = match &request.source {
            ImportSource::InlineContent { content } => content.clone(),
            ImportSource::CidReference { cid } => {
                self.object_store.get(cid).await?
            }
            ImportSource::NatsSubject { subject } => {
                self.fetch_from_subject(subject).await?
            }
            _ => return Err(ImportError::UnsupportedSource),
        };

        // Import and transform
        let graph = self.import_service
            .import_from_content(&content, format, Some(request.options))?;

        // Publish to event store
        let graph_id = GraphId::new();
        self.event_store.publish(DomainEvent::GraphImported {
            graph_id,
            source: request.source,
            node_count: graph.nodes.len(),
            edge_count: graph.edges.len(),
        }).await?;

        Ok(graph_id)
    }
}
```

### 2. Streaming Import Pattern

For continuous data streams that need to be visualized as evolving graphs:

```rust
/// Stream processor for continuous imports
pub struct StreamingImportProcessor {
    buffer_size: usize,
    window_duration: Duration,
    aggregation_strategy: AggregationStrategy,
}

impl StreamingImportProcessor {
    pub async fn process_stream<S: Stream<Item = Vec<u8>>>(
        &self,
        stream: S,
        format: ImportFormat,
        mut graph_sink: impl Sink<ImportedGraph>,
    ) -> Result<()> {
        let windowed_stream = stream
            .chunks_timeout(self.buffer_size, self.window_duration)
            .map(|chunk| self.aggregate_chunk(chunk, format));

        pin_mut!(windowed_stream);

        while let Some(graph_result) = windowed_stream.next().await {
            match graph_result {
                Ok(graph) => {
                    // Apply incremental layout
                    let graph = self.apply_incremental_layout(graph)?;
                    graph_sink.send(graph).await?;
                }
                Err(e) => {
                    error!("Stream processing error: {}", e);
                    // Continue processing on error
                }
            }
        }

        Ok(())
    }

    fn aggregate_chunk(
        &self,
        messages: Vec<Vec<u8>>,
        format: ImportFormat,
    ) -> Result<ImportedGraph> {
        match self.aggregation_strategy {
            AggregationStrategy::Merge => {
                let mut merged = ImportedGraph::new();
                for msg in messages {
                    let partial = parse_message(&msg, format)?;
                    merged.merge(partial);
                }
                Ok(merged)
            }
            AggregationStrategy::Temporal => {
                create_temporal_graph(messages, format)
            }
            AggregationStrategy::Hierarchical => {
                create_hierarchical_graph(messages, format)
            }
        }
    }
}
```

### 3. Resource Import Pattern

Transform Bevy resources and assets into analyzable graphs:

```rust
/// Import Bevy resources as graphs
pub trait ResourceImporter: Send + Sync {
    type Resource: Resource;

    fn import_resource(&self, resource: &Self::Resource) -> ImportedGraph;
}

/// Texture resource importer
pub struct TextureImporter {
    sampling_strategy: SamplingStrategy,
    color_quantization: ColorQuantization,
}

impl ResourceImporter for TextureImporter {
    type Resource = Handle<Image>;

    fn import_resource(&self, texture_handle: &Self::Resource) -> ImportedGraph {
        let mut graph = ImportedGraph::new();

        // Sample texture data
        if let Some(image) = texture_handle.get() {
            let samples = self.sampling_strategy.sample(&image.data);

            // Create nodes for color clusters
            let clusters = self.color_quantization.cluster(&samples);
            for (cluster_id, cluster) in clusters.iter().enumerate() {
                let node = ImportedNode {
                    id: NodeId::new(),
                    label: format!("Color_{}", cluster_id),
                    position: color_to_position(&cluster.centroid),
                    properties: hashmap! {
                        "color" => json!(cluster.centroid),
                        "pixel_count" => json!(cluster.size),
                        "variance" => json!(cluster.variance),
                    },
                };
                graph.nodes.push(node);
            }

            // Connect similar colors
            for i in 0..clusters.len() {
                for j in (i + 1)..clusters.len() {
                    let distance = color_distance(&clusters[i], &clusters[j]);
                    if distance < self.color_quantization.similarity_threshold {
                        graph.edges.push(ImportedEdge {
                            id: EdgeId::new(),
                            source: graph.nodes[i].id,
                            target: graph.nodes[j].id,
                            edge_type: "similar_color".to_string(),
                            properties: hashmap! {
                                "distance" => json!(distance),
                            },
                        });
                    }
                }
            }
        }

        graph
    }
}

/// Animation resource importer
pub struct AnimationImporter {
    keyframe_extraction: KeyframeExtraction,
    motion_analysis: MotionAnalysis,
}

impl ResourceImporter for AnimationImporter {
    type Resource = Handle<AnimationClip>;

    fn import_resource(&self, animation: &Self::Resource) -> ImportedGraph {
        let mut graph = ImportedGraph::new();

        if let Some(clip) = animation.get() {
            // Extract keyframes as nodes
            let keyframes = self.keyframe_extraction.extract(clip);

            for (idx, keyframe) in keyframes.iter().enumerate() {
                let node = ImportedNode {
                    id: NodeId::new(),
                    label: format!("Frame_{}", idx),
                    position: Position3D {
                        x: keyframe.time * 100.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    properties: self.extract_keyframe_properties(keyframe),
                };
                graph.nodes.push(node);
            }

            // Analyze motion between keyframes
            for window in graph.nodes.windows(2) {
                let motion = self.motion_analysis.analyze(
                    &keyframes[window[0].id],
                    &keyframes[window[1].id],
                );

                graph.edges.push(ImportedEdge {
                    id: EdgeId::new(),
                    source: window[0].id,
                    target: window[1].id,
                    edge_type: motion.motion_type.to_string(),
                    properties: motion.properties,
                });
            }
        }

        graph
    }
}
```

### 4. Event Stream Import Pattern

Convert domain events into temporal graphs:

```rust
/// Event stream to graph converter
pub struct EventStreamImporter {
    time_window: Duration,
    aggregation_rules: Vec<AggregationRule>,
    layout_strategy: TemporalLayoutStrategy,
}

impl EventStreamImporter {
    pub fn import_event_stream(
        &self,
        events: &[DomainEvent],
    ) -> ImportedGraph {
        let mut graph = ImportedGraph::new();
        let mut entity_nodes: HashMap<String, NodeId> = HashMap::new();

        // Group events by time window
        let windowed_events = self.window_events(events);

        for (window_idx, window) in windowed_events.iter().enumerate() {
            // Create nodes for entities in this window
            for event in window {
                let entity_id = event.aggregate_id().to_string();

                let node_id = entity_nodes.entry(entity_id.clone())
                    .or_insert_with(|| {
                        let node = ImportedNode {
                            id: NodeId::new(),
                            label: entity_id,
                            position: self.layout_strategy.position_for_entity(
                                &entity_id,
                                window_idx,
                            ),
                            properties: HashMap::new(),
                        };
                        graph.nodes.push(node);
                        node.id
                    });

                // Update node properties based on event
                if let Some(node) = graph.nodes.iter_mut()
                    .find(|n| n.id == *node_id) {
                    self.update_node_from_event(node, event);
                }
            }

            // Create edges for event relationships
            for rule in &self.aggregation_rules {
                let relationships = rule.find_relationships(window);
                for rel in relationships {
                    graph.edges.push(ImportedEdge {
                        id: EdgeId::new(),
                        source: entity_nodes[&rel.source],
                        target: entity_nodes[&rel.target],
                        edge_type: rel.relationship_type,
                        properties: rel.properties,
                    });
                }
            }
        }

        graph
    }

    fn update_node_from_event(
        &self,
        node: &mut ImportedNode,
        event: &DomainEvent,
    ) {
        // Increment event count
        let count = node.properties
            .entry("event_count".to_string())
            .or_insert(json!(0));
        if let Some(n) = count.as_u64() {
            *count = json!(n + 1);
        }

        // Track event types
        let event_types = node.properties
            .entry("event_types".to_string())
            .or_insert(json!(Vec::<String>::new()));
        if let Some(types) = event_types.as_array_mut() {
            types.push(json!(event.event_type()));
        }

        // Update timestamp
        node.properties.insert(
            "last_event".to_string(),
            json!(event.timestamp()),
        );
    }
}
```

### 5. Signal Processing Import Pattern

Transform continuous signals into graph representations:

```rust
/// Signal to graph converter
pub struct SignalImporter<S: SignalSource> {
    signal_source: S,
    analysis_pipeline: AnalysisPipeline,
    graph_builder: SignalGraphBuilder,
}

impl<S: SignalSource> SignalImporter<S> {
    pub async fn import_signal(
        &self,
        duration: Duration,
        sample_rate: f32,
    ) -> Result<ImportedGraph> {
        // Capture signal data
        let samples = self.signal_source
            .capture(duration, sample_rate)
            .await?;

        // Run analysis pipeline
        let analysis = self.analysis_pipeline.analyze(&samples)?;

        // Build graph from analysis
        let mut graph = ImportedGraph::new();

        // Add frequency domain nodes
        for component in &analysis.frequency_components {
            let node = ImportedNode {
                id: NodeId::new(),
                label: format!("{:.1}Hz", component.frequency),
                position: Position3D {
                    x: component.frequency.log10() * 100.0,
                    y: component.magnitude * 100.0,
                    z: component.phase,
                },
                properties: hashmap! {
                    "frequency" => json!(component.frequency),
                    "magnitude" => json!(component.magnitude),
                    "phase" => json!(component.phase),
                    "harmonic_order" => json!(component.harmonic_order),
                },
            };
            graph.nodes.push(node);
        }

        // Add harmonic relationships
        for (i, comp1) in analysis.frequency_components.iter().enumerate() {
            for (j, comp2) in analysis.frequency_components.iter().enumerate().skip(i + 1) {
                if is_harmonic(comp1.frequency, comp2.frequency) {
                    graph.edges.push(ImportedEdge {
                        id: EdgeId::new(),
                        source: graph.nodes[i].id,
                        target: graph.nodes[j].id,
                        edge_type: "harmonic".to_string(),
                        properties: hashmap! {
                            "ratio" => json!(comp2.frequency / comp1.frequency),
                        },
                    });
                }
            }
        }

        // Add time domain features
        self.add_time_domain_features(&mut graph, &analysis);

        Ok(graph)
    }
}

/// CPU usage signal source
pub struct CpuUsageSignal;

impl SignalSource for CpuUsageSignal {
    async fn capture(&self, duration: Duration, sample_rate: f32) -> Result<Vec<f32>> {
        let mut samples = Vec::new();
        let sample_interval = Duration::from_secs_f32(1.0 / sample_rate);
        let start = Instant::now();

        while start.elapsed() < duration {
            let usage = get_cpu_usage_percentage();
            samples.push(usage);
            tokio::time::sleep(sample_interval).await;
        }

        Ok(samples)
    }
}
```

## Integration Patterns

### 1. Workflow Step Integration

```rust
#[derive(Component)]
pub struct ImportWorkflowStep {
    pub step_id: StepId,
    pub import_config: ImportConfig,
    pub output_binding: String,
}

pub async fn execute_import_workflow_step(
    step: &ImportWorkflowStep,
    context: &mut WorkflowContext,
    services: &WorkflowServices,
) -> Result<StepResult> {
    // Resolve input from context
    let input = match &step.import_config.source {
        ImportSource::ContextBinding { binding } => {
            context.resolve::<String>(binding)?
        }
        ImportSource::Literal { content } => content.clone(),
        _ => return Err(WorkflowError::InvalidSource),
    };

    // Import with configured options
    let graph = services.import_service
        .import_from_content(&input, step.import_config.format, None)?;

    // Apply post-import transformations
    let graph = apply_transformations(graph, &step.import_config.transformations)?;

    // Bind result to context
    context.bind(&step.output_binding, graph);

    Ok(StepResult::Success {
        outputs: vec![step.output_binding.clone()],
    })
}
```

### 2. Real-time Dashboard Integration

```rust
/// Dashboard widget that displays imported graph data
#[derive(Component)]
pub struct GraphImportWidget {
    pub source: ImportSource,
    pub format: ImportFormat,
    pub refresh_rate: Duration,
    pub visualization: VisualizationType,
}

pub fn update_import_widgets(
    mut widgets: Query<(&mut GraphImportWidget, &mut WidgetData)>,
    import_service: Res<GraphImportService>,
    time: Res<Time>,
) {
    for (widget, mut data) in widgets.iter_mut() {
        if should_refresh(&widget, &time) {
            // Import fresh data
            if let Ok(graph) = import_service.import_from_source(
                &widget.source,
                widget.format,
            ) {
                // Transform to widget data
                data.update_from_graph(&graph, &widget.visualization);
            }
        }
    }
}
```

### 3. Conceptual Space Integration

```rust
/// Automatically map imported graphs to conceptual space
pub struct ConceptualImportEnricher {
    embedding_service: Arc<EmbeddingService>,
    conceptual_space: Arc<ConceptualSpace>,
}

impl ImportEnricher for ConceptualImportEnricher {
    async fn enrich(&self, graph: &mut ImportedGraph) -> Result<()> {
        // Generate embeddings for nodes
        for node in &mut graph.nodes {
            let text = format!("{} {}", node.label,
                serde_json::to_string(&node.properties)?);

            let embedding = self.embedding_service
                .embed(&text)
                .await?;

            let conceptual_point = self.conceptual_space
                .map_to_point(&embedding)?;

            node.properties.insert(
                "conceptual_position".to_string(),
                json!(conceptual_point),
            );
        }

        // Add conceptual edges
        let conceptual_edges = self.find_conceptual_relationships(&graph.nodes)?;
        graph.edges.extend(conceptual_edges);

        Ok(())
    }
}
```

## Performance Optimization Patterns

### 1. Incremental Import

```rust
/// Import large datasets incrementally
pub struct IncrementalImporter {
    chunk_size: usize,
    parallel_chunks: usize,
}

impl IncrementalImporter {
    pub async fn import_large_file(
        &self,
        path: &Path,
        format: ImportFormat,
    ) -> Result<GraphId> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let (tx, rx) = mpsc::channel(self.parallel_chunks);

        // Process chunks in parallel
        let processor = tokio::spawn(async move {
            let mut merged = ImportedGraph::new();

            while let Some(chunk_graph) = rx.recv().await {
                merged.merge(chunk_graph);
            }

            merged
        });

        // Read and send chunks
        let mut chunk_buffer = Vec::with_capacity(self.chunk_size);
        for line in reader.lines() {
            chunk_buffer.push(line?);

            if chunk_buffer.len() >= self.chunk_size {
                let chunk = std::mem::take(&mut chunk_buffer);
                let graph = parse_chunk(&chunk, format)?;
                tx.send(graph).await?;
            }
        }

        // Send final chunk
        if !chunk_buffer.is_empty() {
            let graph = parse_chunk(&chunk_buffer, format)?;
            tx.send(graph).await?;
        }

        drop(tx);
        let final_graph = processor.await?;

        // Store and return
        store_graph(final_graph).await
    }
}
```

### 2. Caching Import Results

```rust
/// Cache imported graphs for repeated access
pub struct CachedImportService {
    inner: Arc<GraphImportService>,
    cache: Arc<RwLock<LruCache<ImportCacheKey, Arc<ImportedGraph>>>>,
}

impl CachedImportService {
    pub async fn import_with_cache(
        &self,
        source: &ImportSource,
        format: ImportFormat,
        options: Option<ImportOptions>,
    ) -> Result<Arc<ImportedGraph>> {
        let cache_key = ImportCacheKey::from((source, format, &options));

        // Check cache
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(Arc::clone(cached));
            }
        }

        // Import if not cached
        let graph = self.inner
            .import_from_source(source, format, options)?;
        let graph = Arc::new(graph);

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.put(cache_key, Arc::clone(&graph));
        }

        Ok(graph)
    }
}
```

## Error Handling Patterns

### 1. Graceful Degradation

```rust
/// Import with fallback strategies
pub struct ResilientImporter {
    primary_format: ImportFormat,
    fallback_formats: Vec<ImportFormat>,
    error_recovery: ErrorRecoveryStrategy,
}

impl ResilientImporter {
    pub async fn import_with_fallback(
        &self,
        content: &str,
    ) -> Result<ImportedGraph> {
        // Try primary format
        match self.try_import(content, self.primary_format).await {
            Ok(graph) => return Ok(graph),
            Err(e) => {
                warn!("Primary format failed: {}", e);
            }
        }

        // Try fallback formats
        for format in &self.fallback_formats {
            match self.try_import(content, *format).await {
                Ok(graph) => {
                    info!("Successfully imported with fallback format: {:?}", format);
                    return Ok(graph);
                }
                Err(e) => {
                    warn!("Fallback format {:?} failed: {}", format, e);
                }
            }
        }

        // Apply error recovery
        match self.error_recovery {
            ErrorRecoveryStrategy::PartialImport => {
                self.import_partial(content).await
            }
            ErrorRecoveryStrategy::EmptyGraph => {
                Ok(ImportedGraph::empty())
            }
            ErrorRecoveryStrategy::Fail => {
                Err(ImportError::AllFormatsFailed)
            }
        }
    }
}
```

## Testing Patterns

### 1. Import Verification

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_import_preserves_structure() {
        let original = create_test_graph();
        let exported = export_to_format(&original, ImportFormat::ArrowsApp);
        let imported = import_from_content(&exported, ImportFormat::ArrowsApp, None)
            .unwrap();

        assert_eq!(original.nodes.len(), imported.nodes.len());
        assert_eq!(original.edges.len(), imported.edges.len());

        // Verify node properties preserved
        for (orig, imp) in original.nodes.iter().zip(&imported.nodes) {
            assert_eq!(orig.label, imp.label);
            assert_eq!(orig.properties, imp.properties);
        }
    }

    #[tokio::test]
    async fn test_streaming_import() {
        let stream = futures::stream::iter(vec![
            Ok(b"node1,label1".to_vec()),
            Ok(b"node2,label2".to_vec()),
            Ok(b"edge,node1,node2".to_vec()),
        ]);

        let processor = StreamingImportProcessor::new();
        let (tx, mut rx) = mpsc::channel(10);

        processor.process_stream(stream, ImportFormat::Csv, tx).await.unwrap();

        let graph = rx.recv().await.unwrap();
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }
}
```

This technical reference provides the implementation patterns needed to extend and customize the graph import system for any data source or use case.
