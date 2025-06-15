use crate::domain::conceptual_graph::ConceptId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// Unique identifier for a metric context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MetricContextId(pub Uuid);

impl MetricContextId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for MetricContextId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a domain context enriched with measurable relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricContext {
    pub id: MetricContextId,
    pub name: String,
    pub base_context: ConceptId,
    pub metric_type: MetricType,
    pub metric_space: MetricSpace,
}

/// Types of metrics that can be applied to domain relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    /// Semantic distance between concepts
    SemanticDistance { distance_function: DistanceFunction },

    /// Cost of transformations or operations
    TransformationCost { cost_function: CostFunction },

    /// Time delays in processes
    TemporalDelay { delay_function: DelayFunction },

    /// Probability of relationships
    Probabilistic {
        probability_function: ProbabilityFunction,
    },

    /// Resource consumption metrics
    ResourceMetric {
        resource_type: ResourceType,
        consumption_function: ConsumptionFunction,
    },
}

/// Distance calculation methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistanceFunction {
    Euclidean,
    Manhattan,
    Cosine,
    Custom(String), // Name of custom function
}

/// Cost calculation methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostFunction {
    Fixed(f64),
    Linear { rate: f64 },
    Exponential { base: f64, rate: f64 },
    Custom(String),
}

/// Delay calculation methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelayFunction {
    Constant(f64),
    Variable { min: f64, max: f64 },
    Stochastic { mean: f64, variance: f64 },
    Custom(String),
}

/// Probability calculation methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProbabilityFunction {
    Fixed(f64),
    Conditional { conditions: Vec<String> },
    Bayesian { priors: HashMap<String, f64> },
    Custom(String),
}

/// Types of resources that can be measured
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Computational,
    Memory,
    Network,
    Human,
    Financial,
    Custom(String),
}

/// Resource consumption calculation methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsumptionFunction {
    Constant(f64),
    Linear { rate: f64 },
    StepFunction { thresholds: Vec<(f64, f64)> },
    Custom(String),
}

/// The metric space containing distances between concepts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSpace {
    /// Distance/cost matrix between concepts
    pub distances: HashMap<(ConceptId, ConceptId), f64>,

    /// Metric properties
    pub is_symmetric: bool,
    pub satisfies_triangle_inequality: bool,
    pub has_zero_self_distance: bool,
}

/// A path through the metric space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    pub nodes: Vec<ConceptId>,
    pub total_distance: f64,
}

/// A cluster of concepts based on metric similarity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptCluster {
    pub id: Uuid,
    pub centroid: Option<ConceptId>,
    pub members: Vec<ConceptId>,
    pub average_distance: f64,
}

impl MetricContext {
    /// Create a new metric context
    pub fn new(name: String, base_context: ConceptId, metric_type: MetricType) -> Self {
        Self {
            id: MetricContextId::new(),
            name,
            base_context,
            metric_type,
            metric_space: MetricSpace {
                distances: HashMap::new(),
                is_symmetric: true,
                satisfies_triangle_inequality: true,
                has_zero_self_distance: true,
            },
        }
    }

    /// Add or update a distance between two concepts
    pub fn set_distance(&mut self, from: ConceptId, to: ConceptId, distance: f64) {
        self.metric_space.distances.insert((from, to), distance);

        // If symmetric, also set the reverse distance
        if self.metric_space.is_symmetric {
            self.metric_space.distances.insert((to, from), distance);
        }
    }

    /// Get the distance between two concepts
    pub fn get_distance(&self, from: ConceptId, to: ConceptId) -> Option<f64> {
        self.metric_space.distances.get(&(from, to)).copied()
    }

    /// Find the shortest path between two concepts using Dijkstra's algorithm
    pub fn shortest_path(&self, from: ConceptId, to: ConceptId) -> Result<Path, String> {
        use std::cmp::Ordering;
        use std::collections::{BinaryHeap, HashSet};

        #[derive(Clone)]
        struct State {
            cost: f64,
            node: ConceptId,
            path: Vec<ConceptId>,
        }

        impl PartialEq for State {
            fn eq(&self, other: &Self) -> bool {
                self.cost.eq(&other.cost)
            }
        }

        impl Eq for State {}

        impl Ord for State {
            fn cmp(&self, other: &Self) -> Ordering {
                other
                    .cost
                    .partial_cmp(&self.cost)
                    .unwrap_or(Ordering::Equal)
            }
        }

        impl PartialOrd for State {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut heap = BinaryHeap::new();
        let mut visited = HashSet::new();

        heap.push(State {
            cost: 0.0,
            node: from,
            path: vec![from],
        });

        while let Some(State { cost, node, path }) = heap.pop() {
            if node == to {
                return Ok(Path {
                    nodes: path,
                    total_distance: cost,
                });
            }

            if visited.contains(&node) {
                continue;
            }

            visited.insert(node);

            // Find all neighbors
            for ((n1, n2), dist) in &self.metric_space.distances {
                if *n1 == node && !visited.contains(n2) {
                    let mut new_path = path.clone();
                    new_path.push(*n2);

                    heap.push(State {
                        cost: cost + dist,
                        node: *n2,
                        path: new_path,
                    });
                }
            }
        }

        Err("No path found".to_string())
    }

    /// Find k-nearest neighbors to a concept
    pub fn nearest_neighbors(&self, concept: ConceptId, k: usize) -> Vec<(ConceptId, f64)> {
        let mut neighbors: Vec<(ConceptId, f64)> = self
            .metric_space
            .distances
            .iter()
            .filter_map(|((from, to), dist)| {
                if *from == concept {
                    Some((*to, *dist))
                } else {
                    None
                }
            })
            .collect();

        neighbors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        neighbors.truncate(k);
        neighbors
    }

    /// Find all concepts within a given radius
    pub fn metric_ball(&self, center: ConceptId, radius: f64) -> Vec<ConceptId> {
        self.metric_space
            .distances
            .iter()
            .filter_map(|((from, to), dist)| {
                if *from == center && *dist <= radius {
                    Some(*to)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Simple hierarchical clustering based on distance threshold
    pub fn cluster_by_distance(&self, threshold: f64) -> Vec<ConceptCluster> {
        let mut clusters: Vec<ConceptCluster> = Vec::new();
        let mut assigned: HashSet<ConceptId> = HashSet::new();

        // Get all unique concepts
        let concepts: HashSet<ConceptId> = self
            .metric_space
            .distances
            .keys()
            .flat_map(|(a, b)| vec![*a, *b])
            .collect();

        for concept in concepts {
            if assigned.contains(&concept) {
                continue;
            }

            // Create new cluster
            let mut cluster = ConceptCluster {
                id: Uuid::new_v4(),
                centroid: Some(concept),
                members: vec![concept],
                average_distance: 0.0,
            };

            assigned.insert(concept);

            // Find all concepts within threshold
            let neighbors = self.metric_ball(concept, threshold);
            for neighbor in neighbors {
                if !assigned.contains(&neighbor) {
                    cluster.members.push(neighbor);
                    assigned.insert(neighbor);
                }
            }

            // Calculate average distance within cluster
            if cluster.members.len() > 1 {
                let mut total_distance = 0.0;
                let mut count = 0;

                for i in 0..cluster.members.len() {
                    for j in i + 1..cluster.members.len() {
                        if let Some(dist) =
                            self.get_distance(cluster.members[i], cluster.members[j])
                        {
                            total_distance += dist;
                            count += 1;
                        }
                    }
                }

                if count > 0 {
                    cluster.average_distance = total_distance / count as f64;
                }
            }

            clusters.push(cluster);
        }

        clusters
    }
}

use std::collections::HashSet;
