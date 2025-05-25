//! Systems for graph validation and constraint checking
//!
//! These systems handle:
//! - Node property validation
//! - Edge connection validation
//! - Graph structure validation
//! - Domain-specific rules enforcement

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::{
    components::*,
    events::*,
    resources::*,
};

/// System that validates node properties
///
/// This system checks if node properties meet domain requirements
pub fn validate_node_properties(
    mut events: EventReader<ValidateNodePropertiesEvent>,
    nodes: Query<(&NodeId, &DomainNodeType)>,
    mut notification_events: EventWriter<ShowNotificationEvent>,
) {
    for event in events.read() {
        if let Ok((node_id, domain_type)) = nodes.get(event.entity) {
            let mut errors = Vec::new();

            // Validate based on domain type
            match domain_type {
                DomainNodeType::Entity => {
                    // Entities must have an ID property
                    if !event.properties.contains_key("id") {
                        errors.push("Entity must have an 'id' property");
                    }
                }
                DomainNodeType::Aggregate => {
                    // Aggregates must have a root entity
                    if !event.properties.contains_key("root_entity") {
                        errors.push("Aggregate must specify a root entity");
                    }
                }
                DomainNodeType::Event => {
                    // Events must have a timestamp
                    if !event.properties.contains_key("timestamp") && !event.properties.contains_key("occurred_at") {
                        errors.push("Event must have a timestamp");
                    }
                }
                DomainNodeType::Command => {
                    // Commands must have a handler
                    if !event.properties.contains_key("handler") {
                        errors.push("Command must specify a handler");
                    }
                }
                _ => {} // Other types have no specific requirements
            }

            // Send notifications for any errors
            if !errors.is_empty() {
                notification_events.send(ShowNotificationEvent {
                    message: format!("Validation errors: {}", errors.join(", ")),
                    notification_type: NotificationType::Warning,
                    duration_seconds: 5.0,
                });
            }
        }
    }
}

/// System that validates edge connections
///
/// This system ensures edges follow domain rules
pub fn validate_edge_connections(
    mut events: EventReader<ValidateEdgeConnectionEvent>,
    source_nodes: Query<&DomainNodeType>,
    target_nodes: Query<&DomainNodeType>,
    mut notification_events: EventWriter<ShowNotificationEvent>,
) {
    for event in events.read() {
        let source_type = source_nodes.get(event.source);
        let target_type = target_nodes.get(event.target);

        if let (Ok(source), Ok(target)) = (source_type, target_type) {
            let is_valid = match (&event.edge_type, source, target) {
                // Commands can only target aggregates or services
                (DomainEdgeType::Commands, _, DomainNodeType::Aggregate) => true,
                (DomainEdgeType::Commands, _, DomainNodeType::Service) => true,
                (DomainEdgeType::Commands, _, _) => false,

                // Events can be published by aggregates, services, or other events
                (DomainEdgeType::Publishes, DomainNodeType::Aggregate, DomainNodeType::Event) => true,
                (DomainEdgeType::Publishes, DomainNodeType::Service, DomainNodeType::Event) => true,
                (DomainEdgeType::Publishes, DomainNodeType::Event, DomainNodeType::Event) => true,
                (DomainEdgeType::Publishes, _, _) => false,

                // Queries can target repositories or read models
                (DomainEdgeType::Queries, _, DomainNodeType::Repository) => true,
                (DomainEdgeType::Queries, _, DomainNodeType::Query) => true,
                (DomainEdgeType::Queries, _, _) => false,

                // Dependencies are more flexible
                (DomainEdgeType::DependsOn, _, _) => true,

                // Contains is for aggregates and entities
                (DomainEdgeType::Contains, DomainNodeType::Aggregate, DomainNodeType::Entity) => true,
                (DomainEdgeType::Contains, DomainNodeType::Entity, DomainNodeType::ValueObject) => true,
                (DomainEdgeType::Contains, _, _) => false,

                // Default allow for other combinations
                _ => true,
            };

            if !is_valid {
                notification_events.send(ShowNotificationEvent {
                    message: format!(
                        "Invalid connection: {} cannot {} {}",
                        format!("{:?}", source),
                        format!("{:?}", event.edge_type),
                        format!("{:?}", target)
                    ),
                    notification_type: NotificationType::Error,
                    duration_seconds: 5.0,
                });
            }
        }
    }
}

/// System that validates overall graph structure
///
/// This system checks for structural issues in the graph
pub fn validate_graph_structure(
    mut events: EventReader<ValidateGraphEvent>,
    nodes: Query<(Entity, &NodeId, &DomainNodeType)>,
    edges: Query<&Edge>,
    mut metrics_events: EventWriter<GraphMetricsEvent>,
    mut notification_events: EventWriter<ShowNotificationEvent>,
) {
    for _ in events.read() {
        let mut issues = Vec::new();

        // Build adjacency lists
        let mut outgoing: HashMap<Entity, Vec<Entity>> = HashMap::new();
        let mut incoming: HashMap<Entity, Vec<Entity>> = HashMap::new();

        for edge in edges.iter() {
            outgoing.entry(edge.source).or_default().push(edge.target);
            incoming.entry(edge.target).or_default().push(edge.source);
        }

        // Check for cycles
        let has_cycles = detect_cycles(&outgoing);
        if has_cycles {
            issues.push("Graph contains cycles");
        }

        // Check for disconnected components
        let components = find_connected_components(&nodes, &outgoing, &incoming);
        if components > 1 {
            issues.push(&format!("Graph has {} disconnected components", components));
        }

        // Check aggregate boundaries
        for (entity, _, domain_type) in nodes.iter() {
            if matches!(domain_type, DomainNodeType::Aggregate) {
                // Aggregates should not have incoming "Contains" edges
                if let Some(sources) = incoming.get(&entity) {
                    for &source in sources {
                        if let Ok(edge) = edges.iter().find(|e| e.source == source && e.target == entity) {
                            if matches!(edge.edge_type, DomainEdgeType::Contains) {
                                issues.push("Aggregate cannot be contained by another node");
                            }
                        }
                    }
                }
            }
        }

        // Send metrics
        metrics_events.send(GraphMetricsEvent {
            node_count: nodes.iter().count(),
            edge_count: edges.iter().count(),
            connected_components: components,
            has_cycles,
        });

        // Send notifications
        if issues.is_empty() {
            notification_events.send(ShowNotificationEvent {
                message: "Graph validation passed".to_string(),
                notification_type: NotificationType::Success,
                duration_seconds: 2.0,
            });
        } else {
            notification_events.send(ShowNotificationEvent {
                message: format!("Graph validation issues: {}", issues.join(", ")),
                notification_type: NotificationType::Warning,
                duration_seconds: 5.0,
            });
        }
    }
}

/// System that enforces node limits
///
/// This system prevents the graph from growing too large
pub fn enforce_node_limits(
    mut events: EventReader<CreateNodeEvent>,
    nodes: Query<&NodeId>,
    graph_limits: Res<GraphLimits>,
    mut validated_events: EventWriter<CreateNodeEvent>,
    mut notification_events: EventWriter<ShowNotificationEvent>,
) {
    let current_count = nodes.iter().count();

    for event in events.read() {
        if current_count >= graph_limits.max_nodes {
            notification_events.send(ShowNotificationEvent {
                message: format!("Cannot create node: limit of {} nodes reached", graph_limits.max_nodes),
                notification_type: NotificationType::Error,
                duration_seconds: 3.0,
            });
        } else {
            // Forward the event if within limits
            validated_events.send(event.clone());
        }
    }
}

/// System that validates graph before save
///
/// This system performs comprehensive validation before saving
pub fn validate_before_save(
    save_events: EventReader<SaveJsonFileEvent>,
    nodes: Query<(&NodeId, &NodeProperties, &DomainNodeType)>,
    edges: Query<&Edge>,
    mut validated_save_events: EventWriter<SaveJsonFileEvent>,
    mut validation_events: EventWriter<ValidateGraphEvent>,
) {
    if !save_events.is_empty() {
        // Trigger full validation
        validation_events.send(ValidateGraphEvent);

        // Check for required properties on all nodes
        let mut all_valid = true;

        for (_, properties, domain_type) in nodes.iter() {
            match domain_type {
                DomainNodeType::Entity => {
                    if properties.name.is_empty() {
                        warn!("Entity node missing name");
                        all_valid = false;
                    }
                }
                DomainNodeType::Aggregate => {
                    if properties.labels.is_empty() {
                        warn!("Aggregate node missing labels");
                        all_valid = false;
                    }
                }
                _ => {}
            }
        }

        // Forward save events if validation passes
        if all_valid {
            for event in save_events.read() {
                validated_save_events.send(event.clone());
            }
        }
    }
}

// Helper functions

fn detect_cycles(adjacency: &HashMap<Entity, Vec<Entity>>) -> bool {
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    for &node in adjacency.keys() {
        if !visited.contains(&node) {
            if dfs_has_cycle(node, adjacency, &mut visited, &mut rec_stack) {
                return true;
            }
        }
    }

    false
}

fn dfs_has_cycle(
    node: Entity,
    adjacency: &HashMap<Entity, Vec<Entity>>,
    visited: &mut HashSet<Entity>,
    rec_stack: &mut HashSet<Entity>,
) -> bool {
    visited.insert(node);
    rec_stack.insert(node);

    if let Some(neighbors) = adjacency.get(&node) {
        for &neighbor in neighbors {
            if !visited.contains(&neighbor) {
                if dfs_has_cycle(neighbor, adjacency, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack.contains(&neighbor) {
                return true;
            }
        }
    }

    rec_stack.remove(&node);
    false
}

fn find_connected_components(
    nodes: &Query<(Entity, &NodeId, &DomainNodeType)>,
    outgoing: &HashMap<Entity, Vec<Entity>>,
    incoming: &HashMap<Entity, Vec<Entity>>,
) -> usize {
    let mut visited = HashSet::new();
    let mut components = 0;

    for (entity, _, _) in nodes.iter() {
        if !visited.contains(&entity) {
            components += 1;
            dfs_component(entity, &mut visited, outgoing, incoming);
        }
    }

    components
}

fn dfs_component(
    node: Entity,
    visited: &mut HashSet<Entity>,
    outgoing: &HashMap<Entity, Vec<Entity>>,
    incoming: &HashMap<Entity, Vec<Entity>>,
) {
    visited.insert(node);

    // Visit outgoing neighbors
    if let Some(neighbors) = outgoing.get(&node) {
        for &neighbor in neighbors {
            if !visited.contains(&neighbor) {
                dfs_component(neighbor, visited, outgoing, incoming);
            }
        }
    }

    // Visit incoming neighbors (for undirected connectivity)
    if let Some(neighbors) = incoming.get(&node) {
        for &neighbor in neighbors {
            if !visited.contains(&neighbor) {
                dfs_component(neighbor, visited, outgoing, incoming);
            }
        }
    }
}
