use crate::domain::commands::MetricContextCommand;
use crate::domain::conceptual_graph::{MetricContext, MetricContextId};
use crate::domain::events::{DomainEvent, MetricContextEvent};
use crate::infrastructure::event_store::EventStore;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Handler for metric context commands
pub struct MetricContextHandler {
    event_store: Arc<dyn EventStore>,
    contexts: Arc<RwLock<HashMap<MetricContextId, MetricContext>>>,
}

impl MetricContextHandler {
    pub fn new(event_store: Arc<dyn EventStore>) -> Self {
        Self {
            event_store,
            contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn handle_command(&self, command: MetricContextCommand) -> Result<Vec<DomainEvent>, String> {
        match command {
            MetricContextCommand::CreateMetricContext { name, base_context, metric_type } => {
                let context_id = MetricContextId::new();
                let mut context = MetricContext::new(name.clone(), base_context, metric_type.clone());
                context.id = context_id;

                let mut contexts = self.contexts.write().await;
                contexts.insert(context_id, context);

                let event = DomainEvent::MetricContext(MetricContextEvent::MetricContextCreated {
                    context_id,
                    name: name.clone(),
                    base_context,
                    metric_type: metric_type.clone(),
                });

                self.event_store.append_events(context_id.to_string(), vec![event.clone()])
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(vec![event])
            }

            MetricContextCommand::SetDistance { context_id, from, to, distance } => {
                let mut contexts = self.contexts.write().await;
                let context = contexts.get_mut(&context_id)
                    .ok_or_else(|| "Metric context not found".to_string())?;

                context.set_distance(from, to, distance);

                let event = DomainEvent::MetricContext(MetricContextEvent::DistanceSet {
                    context_id,
                    from,
                    to,
                    distance,
                });

                self.event_store.append_events(context_id.to_string(), vec![event.clone()])
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(vec![event])
            }

            MetricContextCommand::CalculateShortestPath { context_id, from, to } => {
                let contexts = self.contexts.read().await;
                let context = contexts.get(&context_id)
                    .ok_or_else(|| "Metric context not found".to_string())?;

                let path = context.shortest_path(from, to)?;

                let event = DomainEvent::MetricContext(MetricContextEvent::ShortestPathCalculated {
                    context_id,
                    from,
                    to,
                    path: path.clone(),
                });

                self.event_store.append_events(context_id.to_string(), vec![event.clone()])
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(vec![event])
            }

            MetricContextCommand::FindNearestNeighbors { context_id, concept, k } => {
                let contexts = self.contexts.read().await;
                let context = contexts.get(&context_id)
                    .ok_or_else(|| "Metric context not found".to_string())?;

                let neighbors = context.nearest_neighbors(concept, k);

                let event = DomainEvent::MetricContext(MetricContextEvent::NearestNeighborsFound {
                    context_id,
                    concept,
                    neighbors: neighbors.clone(),
                });

                self.event_store.append_events(context_id.to_string(), vec![event.clone()])
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(vec![event])
            }

            MetricContextCommand::ClusterByDistance { context_id, threshold } => {
                let contexts = self.contexts.read().await;
                let context = contexts.get(&context_id)
                    .ok_or_else(|| "Metric context not found".to_string())?;

                let clusters = context.cluster_by_distance(threshold);

                let event = DomainEvent::MetricContext(MetricContextEvent::ConceptsClustered {
                    context_id,
                    threshold,
                    clusters: clusters.clone(),
                });

                self.event_store.append_events(context_id.to_string(), vec![event.clone()])
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(vec![event])
            }

            MetricContextCommand::FindWithinRadius { context_id, center, radius } => {
                let contexts = self.contexts.read().await;
                let context = contexts.get(&context_id)
                    .ok_or_else(|| "Metric context not found".to_string())?;

                let concepts = context.metric_ball(center, radius);

                let event = DomainEvent::MetricContext(MetricContextEvent::ConceptsWithinRadiusFound {
                    context_id,
                    center,
                    radius,
                    concepts,
                });

                self.event_store.append_events(context_id.to_string(), vec![event.clone()])
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(vec![event])
            }

            MetricContextCommand::UpdateMetricProperties {
                context_id,
                is_symmetric,
                satisfies_triangle_inequality,
                has_zero_self_distance
            } => {
                let mut contexts = self.contexts.write().await;
                let context = contexts.get_mut(&context_id)
                    .ok_or_else(|| "Metric context not found".to_string())?;

                if let Some(symmetric) = is_symmetric {
                    context.metric_space.is_symmetric = symmetric;
                }
                if let Some(triangle) = satisfies_triangle_inequality {
                    context.metric_space.satisfies_triangle_inequality = triangle;
                }
                if let Some(zero_self) = has_zero_self_distance {
                    context.metric_space.has_zero_self_distance = zero_self;
                }

                let event = DomainEvent::MetricContext(MetricContextEvent::MetricPropertiesUpdated {
                    context_id,
                    is_symmetric,
                    satisfies_triangle_inequality,
                    has_zero_self_distance,
                });

                self.event_store.append_events(context_id.to_string(), vec![event.clone()])
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(vec![event])
            }
        }
    }
}
