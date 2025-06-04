# Graph Algorithms for DDD Structures

## Overview

This document details the specific graph algorithms we apply to DDD structures and how they provide insights into domain design, event flows, and system behavior.

## Core Algorithms

### 1. Event Propagation Analysis

```rust
use petgraph::algo::{dijkstra, has_path_connecting};
use daggy::petgraph::visit::Bfs;

/// Trace how an event propagates through the system
pub struct EventPropagationAnalyzer {
    event_store: Arc<EventStore>,
    workflow_graphs: Arc<HashMap<WorkflowId, WorkflowGraph>>,
}

impl EventPropagationAnalyzer {
    /// Analyze event propagation from a source event
    pub fn trace_propagation(&self, source_event: &Cid) -> PropagationResult {
        let mut result = PropagationResult::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Start from source event
        queue.push_back((source_event.clone(), 0));

        while let Some((event_cid, depth)) = queue.pop_front() {
            if visited.contains(&event_cid) {
                continue;
            }
            visited.insert(event_cid.clone());

            // Get event details
            if let Ok(event) = self.event_store.get_event(&event_cid) {
                result.add_step(PropagationStep {
                    event_cid: event_cid.clone(),
                    event_type: event.event_type.clone(),
                    aggregate_id: event.aggregate_id,
                    depth,
                    timestamp: event.timestamp,
                });

                // Find triggered events
                let triggered = self.find_triggered_events(&event);
                for triggered_cid in triggered {
                    queue.push_back((triggered_cid, depth + 1));
                }
            }
        }

        result
    }

    /// Visualize propagation in Bevy
    pub fn create_propagation_visual(&self, result: &PropagationResult) -> PropagationVisual {
        PropagationVisual {
            steps: result.steps.clone(),
            animation_duration: Duration::from_millis(200),
            wave_effect: WaveEffect {
                color: Color::CYAN,
                radius: 0.5,
                speed: 2.0,
            },
        }
    }
}
```

### 2. Aggregate Dependency Analysis

```rust
/// Analyze dependencies between aggregates
pub struct AggregateDependencyAnalyzer {
    graph_service: Arc<UnifiedGraphService>,
}

impl AggregateDependencyAnalyzer {
    /// Build dependency graph between aggregates
    pub fn build_dependency_graph(&self) -> Graph<AggregateNode, DependencyEdge> {
        let mut dep_graph = Graph::new();
        let mut node_map = HashMap::new();

        // Add all aggregates as nodes
        for (id, aggregate) in self.graph_service.get_all_aggregates() {
            let node = dep_graph.add_node(AggregateNode {
                aggregate_id: id.clone(),
                domain: aggregate.domain.clone(),
                entity_count: aggregate.entities.len(),
            });
            node_map.insert(id, node);
        }

        // Analyze event flows to find dependencies
        for event in self.graph_service.get_all_events() {
            if let Some(source_agg) = event.source_aggregate {
                if let Some(target_agg) = event.target_aggregate {
                    if source_agg != target_agg {
                        let source_node = node_map[&source_agg];
                        let target_node = node_map[&target_agg];

                        dep_graph.add_edge(
                            source_node,
                            target_node,
                            DependencyEdge {
                                event_type: event.event_type.clone(),
                                frequency: 1, // Increment if edge exists
                            },
                        );
                    }
                }
            }
        }

        dep_graph
    }

    /// Find circular dependencies
    pub fn find_circular_dependencies(&self) -> Vec<Vec<AggregateId>> {
        let dep_graph = self.build_dependency_graph();
        let cycles = tarjan_scc(&dep_graph);

        cycles.into_iter()
            .filter(|scc| scc.len() > 1)
            .map(|scc| {
                scc.into_iter()
                    .map(|idx| dep_graph[idx].aggregate_id.clone())
                    .collect()
            })
            .collect()
    }

    /// Calculate aggregate coupling metrics
    pub fn calculate_coupling_metrics(&self) -> HashMap<AggregateId, CouplingMetrics> {
        let dep_graph = self.build_dependency_graph();
        let mut metrics = HashMap::new();

        for node in dep_graph.node_indices() {
            let aggregate_id = &dep_graph[node].aggregate_id;

            metrics.insert(aggregate_id.clone(), CouplingMetrics {
                afferent_coupling: dep_graph.edges_directed(node, Incoming).count(),
                efferent_coupling: dep_graph.edges_directed(node, Outgoing).count(),
                instability: calculate_instability(&dep_graph, node),
            });
        }

        metrics
    }
}
```

### 3. Domain Boundary Analysis

```rust
/// Analyze cross-domain communication patterns
pub struct DomainBoundaryAnalyzer {
    graph_service: Arc<UnifiedGraphService>,
}

impl DomainBoundaryAnalyzer {
    /// Build inter-domain communication graph
    pub fn build_domain_graph(&self) -> Graph<DomainNode, DomainEdge> {
        let mut domain_graph = Graph::new();
        let mut domain_nodes = HashMap::new();

        // Create nodes for each domain
        for domain in self.graph_service.get_all_domains() {
            let node = domain_graph.add_node(DomainNode {
                domain_id: domain.id.clone(),
                name: domain.name.clone(),
                aggregate_count: domain.aggregates.len(),
            });
            domain_nodes.insert(domain.id.clone(), node);
        }

        // Analyze cross-domain events
        for event in self.graph_service.get_cross_domain_events() {
            let source_domain = self.get_domain_for_aggregate(&event.source_aggregate);
            let target_domain = self.get_domain_for_aggregate(&event.target_aggregate);

            if source_domain != target_domain {
                let source_node = domain_nodes[&source_domain];
                let target_node = domain_nodes[&target_domain];

                domain_graph.update_edge(
                    source_node,
                    target_node,
                    DomainEdge {
                        event_types: vec![event.event_type.clone()],
                        message_count: 1,
                    },
                );
            }
        }

        domain_graph
    }

    /// Find anti-corruption layer candidates
    pub fn find_acl_candidates(&self) -> Vec<AclCandidate> {
        let domain_graph = self.build_domain_graph();
        let mut candidates = Vec::new();

        for edge in domain_graph.edge_indices() {
            let (source, target) = domain_graph.edge_endpoints(edge).unwrap();
            let edge_data = &domain_graph[edge];

            // High message count indicates potential ACL need
            if edge_data.message_count > ACL_THRESHOLD {
                candidates.push(AclCandidate {
                    source_domain: domain_graph[source].domain_id.clone(),
                    target_domain: domain_graph[target].domain_id.clone(),
                    event_types: edge_data.event_types.clone(),
                    complexity_score: calculate_interface_complexity(edge_data),
                });
            }
        }

        candidates.sort_by_key(|c| c.complexity_score);
        candidates
    }
}
```

### 4. Workflow Optimization

```rust
/// Optimize workflow paths using graph algorithms
pub struct WorkflowOptimizer {
    workflow_graphs: Arc<HashMap<WorkflowId, WorkflowGraph>>,
}

impl WorkflowOptimizer {
    /// Find critical path in workflow
    pub fn find_critical_path(&self, workflow_id: &WorkflowId) -> Option<CriticalPath> {
        let workflow = self.workflow_graphs.get(workflow_id)?;
        let graph = &workflow.graph;

        // Find start and end nodes
        let start_nodes: Vec<_> = graph.node_indices()
            .filter(|&n| graph.edges_directed(n, Incoming).count() == 0)
            .collect();

        let end_nodes: Vec<_> = graph.node_indices()
            .filter(|&n| graph.edges_directed(n, Outgoing).count() == 0)
            .collect();

        let mut longest_path = Vec::new();
        let mut longest_duration = 0.0;

        // Find longest path (critical path)
        for &start in &start_nodes {
            for &end in &end_nodes {
                if let Some(path) = self.find_longest_path(graph, start, end) {
                    let duration = self.calculate_path_duration(&path, graph);
                    if duration > longest_duration {
                        longest_duration = duration;
                        longest_path = path;
                    }
                }
            }
        }

        Some(CriticalPath {
            nodes: longest_path,
            total_duration: longest_duration,
            bottlenecks: self.identify_bottlenecks(&longest_path, graph),
        })
    }

    /// Find parallel optimization opportunities
    pub fn find_parallelization_opportunities(&self, workflow_id: &WorkflowId) -> Vec<ParallelizationOpportunity> {
        let workflow = self.workflow_graphs.get(workflow_id).unwrap();
        let graph = &workflow.graph;
        let mut opportunities = Vec::new();

        // Find sequential nodes that could be parallelized
        for node in graph.node_indices() {
            let successors: Vec<_> = graph.neighbors(node).collect();

            if successors.len() > 1 {
                // Check if successors have dependencies on each other
                let can_parallelize = !self.have_dependencies(&successors, graph);

                if can_parallelize {
                    opportunities.push(ParallelizationOpportunity {
                        fork_node: node,
                        parallel_branches: successors,
                        estimated_speedup: self.calculate_speedup(&successors, graph),
                    });
                }
            }
        }

        opportunities
    }
}
```

### 5. Event Storm Pattern Detection

```rust
/// Detect patterns in event storms
pub struct EventStormAnalyzer {
    event_store: Arc<EventStore>,
}

impl EventStormAnalyzer {
    /// Detect event storms (rapid event generation)
    pub fn detect_event_storms(&self, time_window: Duration) -> Vec<EventStorm> {
        let mut storms = Vec::new();
        let events = self.event_store.get_events_in_window(time_window);

        // Group events by aggregate and time bucket
        let mut event_buckets: HashMap<(AggregateId, TimeBucket), Vec<Event>> = HashMap::new();

        for event in events {
            let bucket = TimeBucket::from_timestamp(event.timestamp, Duration::from_secs(60));
            event_buckets.entry((event.aggregate_id.clone(), bucket))
                .or_default()
                .push(event);
        }

        // Identify storms
        for ((aggregate_id, bucket), events) in event_buckets {
            if events.len() > STORM_THRESHOLD {
                storms.push(EventStorm {
                    aggregate_id,
                    time_bucket: bucket,
                    event_count: events.len(),
                    event_types: events.iter().map(|e| e.event_type.clone()).collect(),
                    severity: self.calculate_storm_severity(&events),
                });
            }
        }

        storms
    }

    /// Find event patterns using sequence mining
    pub fn find_event_patterns(&self, min_support: f32) -> Vec<EventPattern> {
        let sequences = self.extract_event_sequences();
        let patterns = self.mine_sequential_patterns(sequences, min_support);

        patterns.into_iter()
            .map(|pattern| EventPattern {
                sequence: pattern.sequence,
                support: pattern.support,
                confidence: pattern.confidence,
                visual_representation: self.create_pattern_visual(&pattern),
            })
            .collect()
    }
}
```

### 6. Performance Bottleneck Detection

```rust
/// Detect performance bottlenecks in graph structures
pub struct BottleneckDetector {
    graph_service: Arc<UnifiedGraphService>,
    metrics: Arc<PerformanceMetrics>,
}

impl BottleneckDetector {
    /// Find bottleneck nodes using betweenness centrality
    pub fn find_bottleneck_nodes(&self) -> Vec<BottleneckNode> {
        let graph = self.graph_service.get_combined_graph();
        let centrality = betweenness_centrality(&graph, true, true);

        let mut bottlenecks = Vec::new();
        let threshold = self.calculate_bottleneck_threshold(&centrality);

        for (node, score) in centrality {
            if score > threshold {
                let node_data = &graph[node];
                bottlenecks.push(BottleneckNode {
                    node_id: node,
                    element: node_data.clone(),
                    centrality_score: score,
                    throughput: self.metrics.get_node_throughput(&node),
                    latency: self.metrics.get_node_latency(&node),
                });
            }
        }

        bottlenecks.sort_by(|a, b| b.centrality_score.partial_cmp(&a.centrality_score).unwrap());
        bottlenecks
    }

    /// Visualize bottlenecks in Bevy
    pub fn create_bottleneck_visualization(&self, bottlenecks: &[BottleneckNode]) -> BottleneckVisual {
        BottleneckVisual {
            heat_map: self.generate_heat_map(bottlenecks),
            flow_indicators: self.generate_flow_indicators(bottlenecks),
            suggestions: self.generate_optimization_suggestions(bottlenecks),
        }
    }
}
```

## Visualization Components

### Algorithm Visualization States

```rust
/// Component for algorithm visualization state
#[derive(Component)]
pub struct AlgorithmVisualState {
    pub current_step: usize,
    pub total_steps: usize,
    pub step_duration: Duration,
    pub highlights: StepHighlights,
}

#[derive(Debug, Clone)]
pub struct StepHighlights {
    pub active_nodes: Vec<Entity>,
    pub active_edges: Vec<Entity>,
    pub annotations: Vec<Annotation>,
}

/// System for stepping through algorithm visualization
pub fn algorithm_step_system(
    mut query: Query<&mut AlgorithmVisualState>,
    time: Res<Time>,
    mut step_timer: Local<Timer>,
) {
    step_timer.tick(time.delta());

    if step_timer.finished() {
        for mut state in query.iter_mut() {
            if state.current_step < state.total_steps {
                state.current_step += 1;
                // Update highlights for new step
                update_step_highlights(&mut state);
            }
        }

        step_timer.reset();
    }
}
```

### Interactive Algorithm Controls

```rust
/// UI for algorithm control and visualization
pub fn algorithm_control_ui(
    mut contexts: EguiContexts,
    mut algorithm_state: ResMut<AlgorithmState>,
    selected_algorithm: Res<SelectedAlgorithm>,
) {
    egui::Window::new("Algorithm Controls")
        .show(contexts.ctx_mut(), |ui| {
            ui.heading(format!("Current: {:?}", selected_algorithm.0));

            ui.horizontal(|ui| {
                if ui.button("⏮").clicked() {
                    algorithm_state.reset();
                }
                if ui.button("⏸").clicked() {
                    algorithm_state.paused = true;
                }
                if ui.button("▶").clicked() {
                    algorithm_state.paused = false;
                }
                if ui.button("⏭").clicked() {
                    algorithm_state.step_forward();
                }
            });

            ui.add(egui::Slider::new(&mut algorithm_state.speed, 0.1..=5.0)
                .text("Speed"));

            ui.separator();

            // Algorithm-specific controls
            match selected_algorithm.0 {
                GraphAlgorithm::EventPropagation { .. } => {
                    ui.label("Event Propagation Settings");
                    ui.checkbox(&mut algorithm_state.show_timestamps, "Show Timestamps");
                    ui.checkbox(&mut algorithm_state.show_paths, "Show Paths");
                }
                GraphAlgorithm::CycleDetection => {
                    ui.label("Cycle Detection");
                    ui.checkbox(&mut algorithm_state.highlight_cycles, "Highlight Cycles");
                }
                _ => {}
            }
        });
}
```

## Benefits

1. **Design Insights**: Identify coupling, circular dependencies, and design issues
2. **Performance Analysis**: Find bottlenecks and optimization opportunities
3. **Event Flow Understanding**: Trace how events propagate through the system
4. **Visual Debugging**: See algorithms execute step-by-step
5. **Pattern Recognition**: Identify recurring patterns in event streams
6. **Optimization Guidance**: Get suggestions for improving system design

This comprehensive set of algorithms provides deep insights into DDD structures while enabling powerful visualizations in Bevy.
