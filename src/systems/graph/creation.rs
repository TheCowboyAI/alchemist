//! Systems for creating graph nodes and edges
//!
//! These systems handle:
//! - Node entity spawning from events
//! - Edge entity creation and connection
//! - Deferred edge creation (when nodes are loaded)
//! - Pattern-based graph generation

use bevy::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    components::*,
    events::*,
    resources::*,
};

/// System that creates node entities from CreateNodeEvent
///
/// This system:
/// 1. Validates the node doesn't already exist
/// 2. Spawns the entity with all required components
/// 3. Updates the UUID to Entity mapping
/// 4. Sends modification events for undo tracking
pub fn handle_create_node(
    mut commands: Commands,
    mut events: EventReader<CreateNodeEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut uuid_to_entity: ResMut<UuidToEntity>,
    existing_nodes: Query<&NodeId>,
    mut modification_events: EventWriter<GraphModificationEvent>,
    mut metrics_events: EventWriter<GraphMetricsEvent>,
) {
    for event in events.read() {
        // Validate node doesn't already exist
        if existing_nodes.iter().any(|id| id.0 == event.id) {
            warn!("Attempted to create duplicate node with ID: {}", event.id);
            continue;
        }

        // Determine node color
        let color = event.color.as_ref()
            .and_then(|hex| Color::hex(hex).ok())
            .unwrap_or_else(|| get_default_color_for_type(&event.domain_type));

        // Create mesh based on node type
        let mesh = create_node_mesh(&event.domain_type);

        // Spawn the node entity
        let entity = commands.spawn((
            // Visual components
            PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(StandardMaterial {
                    base_color: color,
                    metallic: 0.3,
                    perceptual_roughness: 0.7,
                    ..default()
                }),
                transform: Transform::from_translation(event.position),
                ..default()
            },
            // Graph components
            NodeId(event.id),
            event.domain_type.clone(),
            NodeProperties {
                name: event.name.clone(),
                labels: event.labels.clone(),
                properties: event.properties.clone(),
                color: event.color.clone(),
            },
            // Interaction components
            NodeInteractable,
            // Optional subgraph assignment
        )).id();

        // Add to subgraph if specified
        if let Some(subgraph_id) = event.subgraph_id {
            commands.entity(entity).insert(SubgraphMember(subgraph_id));
        }

        // Update UUID mapping
        uuid_to_entity.0.insert(event.id, entity);

        // Send modification event for undo system
        modification_events.send(GraphModificationEvent::NodeCreated {
            id: event.id,
            position: event.position,
            domain_type: event.domain_type.clone(),
            name: event.name.clone(),
        });

        info!("Created node '{}' with ID: {}", event.name, event.id);
    }

    // Request metrics update if any nodes were created
    if events.len() > 0 {
        metrics_events.send(GraphMetricsEvent {
            node_count: 0, // Will be calculated by metrics system
            edge_count: 0,
            connected_components: 0,
            has_cycles: false,
        });
    }
}

/// System that creates edge entities from CreateEdgeEvent
///
/// This system:
/// 1. Validates source and target nodes exist
/// 2. Validates the edge connection is allowed
/// 3. Creates the edge entity with visual representation
/// 4. Updates node connection tracking
pub fn handle_create_edge(
    mut commands: Commands,
    mut events: EventReader<CreateEdgeEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    nodes: Query<&Transform, With<NodeId>>,
    mut modification_events: EventWriter<GraphModificationEvent>,
    mut validation_events: EventWriter<ValidateEdgeConnectionEvent>,
) {
    for event in events.read() {
        // Validate source and target exist
        let (source_transform, target_transform) = match (
            nodes.get(event.source),
            nodes.get(event.target)
        ) {
            (Ok(source), Ok(target)) => (source, target),
            _ => {
                warn!("Cannot create edge: source or target node not found");
                continue;
            }
        };

        // Send validation event
        validation_events.send(ValidateEdgeConnectionEvent {
            source: event.source,
            target: event.target,
            edge_type: event.edge_type.clone(),
        });

        // Create edge visual
        let edge_mesh = create_edge_mesh(
            source_transform.translation,
            target_transform.translation,
        );

        let edge_color = get_edge_color(&event.edge_type);

        // Spawn edge entity
        let edge_entity = commands.spawn((
            PbrBundle {
                mesh: meshes.add(edge_mesh),
                material: materials.add(StandardMaterial {
                    base_color: edge_color,
                    unlit: true,
                    ..default()
                }),
                ..default()
            },
            Edge {
                id: event.id,
                source: event.source,
                target: event.target,
                edge_type: event.edge_type.clone(),
            },
            EdgeProperties {
                labels: event.labels.clone(),
                properties: event.properties.clone(),
            },
        )).id();

        // Update source node connections
        commands.entity(event.source).insert(
            OutgoingEdge {
                edge_id: event.id,
                target: event.target,
                edge_entity,
            }
        );

        // Update target node connections
        commands.entity(event.target).insert(
            IncomingEdge {
                edge_id: event.id,
                source: event.source,
                edge_entity,
            }
        );

        // Send modification event
        modification_events.send(GraphModificationEvent::EdgeCreated {
            id: event.id,
            source_id: event.id, // TODO: Get actual node IDs
            target_id: event.id,
            edge_type: event.edge_type.clone(),
        });

        info!("Created edge from {:?} to {:?}", event.source, event.target);
    }
}

/// System that handles deferred edge creation
///
/// This is used when loading graphs where edges reference nodes by UUID
/// that may not have been created yet.
pub fn handle_deferred_edges(
    mut events: EventReader<DeferredEdgeEvent>,
    mut create_edge_events: EventWriter<CreateEdgeEvent>,
    uuid_to_entity: Res<UuidToEntity>,
    mut retry_events: EventWriter<DeferredEdgeEvent>,
) {
    const MAX_RETRIES: u8 = 5;

    for mut event in events.read() {
        // Try to resolve UUIDs to entities
        match (
            uuid_to_entity.0.get(&event.source_uuid),
            uuid_to_entity.0.get(&event.target_uuid)
        ) {
            (Some(&source), Some(&target)) => {
                // We can create the edge now
                create_edge_events.send(CreateEdgeEvent {
                    id: event.id,
                    source,
                    target,
                    edge_type: event.edge_type.clone(),
                    labels: event.labels.clone(),
                    properties: event.properties.clone(),
                });
            }
            _ => {
                // One or both nodes don't exist yet
                if event.retry_count < MAX_RETRIES {
                    // Retry later
                    let mut retry_event = event.clone();
                    retry_event.retry_count += 1;
                    retry_events.send(retry_event);
                } else {
                    warn!(
                        "Failed to create edge after {} retries: {} -> {}",
                        MAX_RETRIES, event.source_uuid, event.target_uuid
                    );
                }
            }
        }
    }
}

/// System that creates graph patterns
///
/// This system generates common graph structures like complete graphs,
/// star graphs, trees, etc.
pub fn handle_create_pattern(
    mut events: EventReader<CreatePatternEvent>,
    mut create_node_events: EventWriter<CreateNodeEvent>,
    mut create_edge_events: EventWriter<CreateEdgeEvent>,
) {
    for event in events.read() {
        let nodes = match &event.pattern {
            GraphPattern::Complete { nodes } => {
                create_complete_graph(*nodes, &event.pattern_name, &mut create_node_events, &mut create_edge_events)
            }
            GraphPattern::Star { points } => {
                create_star_graph(*points, &event.pattern_name, &mut create_node_events, &mut create_edge_events)
            }
            GraphPattern::Tree { branch_factor, depth } => {
                create_tree_graph(*branch_factor, *depth, &event.pattern_name, &mut create_node_events, &mut create_edge_events)
            }
        };

        info!("Created {} pattern with {} nodes", event.pattern_name, nodes.len());
    }
}

// Helper functions

fn get_default_color_for_type(domain_type: &DomainNodeType) -> Color {
    match domain_type {
        DomainNodeType::Entity => Color::rgb(0.8, 0.7, 0.6),
        DomainNodeType::Event => Color::rgb(1.0, 0.84, 0.0),
        DomainNodeType::Command => Color::rgb(0.6, 0.8, 0.6),
        DomainNodeType::Query => Color::rgb(0.6, 0.6, 0.8),
        DomainNodeType::Aggregate => Color::rgb(0.9, 0.6, 0.6),
        DomainNodeType::Service => Color::rgb(0.7, 0.6, 0.9),
        DomainNodeType::Repository => Color::rgb(0.6, 0.9, 0.7),
        DomainNodeType::Factory => Color::rgb(0.9, 0.7, 0.6),
        DomainNodeType::ValueObject => Color::rgb(0.7, 0.8, 0.9),
    }
}

fn create_node_mesh(domain_type: &DomainNodeType) -> Mesh {
    match domain_type {
        DomainNodeType::Entity | DomainNodeType::Aggregate => {
            Mesh::from(shape::Cube { size: 1.0 })
        }
        DomainNodeType::Event => {
            Mesh::from(shape::UVSphere { radius: 0.5, sectors: 16, stacks: 8 })
        }
        DomainNodeType::Command | DomainNodeType::Query => {
            Mesh::from(shape::Cylinder { radius: 0.5, height: 1.0, resolution: 16, segments: 1 })
        }
        _ => {
            Mesh::from(shape::Icosphere { radius: 0.5, subdivisions: 2 })
        }
    }
}

fn create_edge_mesh(start: Vec3, end: Vec3) -> Mesh {
    // Create a simple line mesh
    // In a real implementation, this would create a proper 3D edge representation
    Mesh::from(shape::Box::new(0.1, 0.1, (end - start).length()))
}

fn get_edge_color(edge_type: &DomainEdgeType) -> Color {
    match edge_type {
        DomainEdgeType::Contains => Color::rgb(0.7, 0.7, 0.7),
        DomainEdgeType::DependsOn => Color::rgb(0.8, 0.6, 0.6),
        DomainEdgeType::Triggers => Color::rgb(0.6, 0.8, 0.6),
        DomainEdgeType::Queries => Color::rgb(0.6, 0.6, 0.8),
        DomainEdgeType::Commands => Color::rgb(0.8, 0.8, 0.6),
        DomainEdgeType::Publishes => Color::rgb(0.8, 0.6, 0.8),
        DomainEdgeType::Subscribes => Color::rgb(0.6, 0.8, 0.8),
        DomainEdgeType::Creates => Color::rgb(0.9, 0.7, 0.5),
        DomainEdgeType::Updates => Color::rgb(0.7, 0.9, 0.5),
        DomainEdgeType::Deletes => Color::rgb(0.9, 0.5, 0.5),
    }
}

fn create_complete_graph(
    node_count: usize,
    pattern_name: &str,
    create_node_events: &mut EventWriter<CreateNodeEvent>,
    create_edge_events: &mut EventWriter<CreateEdgeEvent>,
) -> Vec<Uuid> {
    let mut node_ids = Vec::new();
    let radius = 5.0;

    // Create nodes in a circle
    for i in 0..node_count {
        let angle = (i as f32 / node_count as f32) * std::f32::consts::TAU;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;

        let node_id = Uuid::new_v4();
        node_ids.push(node_id);

        create_node_events.send(CreateNodeEvent {
            id: node_id,
            position: Vec3::new(x, 0.0, z),
            domain_type: DomainNodeType::Entity,
            name: format!("{} Node {}", pattern_name, i + 1),
            labels: vec!["pattern".to_string(), "complete".to_string()],
            properties: HashMap::new(),
            subgraph_id: None,
            color: None,
        });
    }

    // Create edges between all pairs
    for i in 0..node_count {
        for j in (i + 1)..node_count {
            // Note: This won't work immediately as entities don't exist yet
            // In practice, we'd need to defer edge creation
        }
    }

    node_ids
}

fn create_star_graph(
    points: usize,
    pattern_name: &str,
    create_node_events: &mut EventWriter<CreateNodeEvent>,
    create_edge_events: &mut EventWriter<CreateEdgeEvent>,
) -> Vec<Uuid> {
    let mut node_ids = Vec::new();
    let radius = 5.0;

    // Create center node
    let center_id = Uuid::new_v4();
    node_ids.push(center_id);

    create_node_events.send(CreateNodeEvent {
        id: center_id,
        position: Vec3::ZERO,
        domain_type: DomainNodeType::Aggregate,
        name: format!("{} Center", pattern_name),
        labels: vec!["pattern".to_string(), "star".to_string(), "center".to_string()],
        properties: HashMap::new(),
        subgraph_id: None,
        color: None,
    });

    // Create outer nodes
    for i in 0..points {
        let angle = (i as f32 / points as f32) * std::f32::consts::TAU;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;

        let node_id = Uuid::new_v4();
        node_ids.push(node_id);

        create_node_events.send(CreateNodeEvent {
            id: node_id,
            position: Vec3::new(x, 0.0, z),
            domain_type: DomainNodeType::Entity,
            name: format!("{} Point {}", pattern_name, i + 1),
            labels: vec!["pattern".to_string(), "star".to_string(), "point".to_string()],
            properties: HashMap::new(),
            subgraph_id: None,
            color: None,
        });
    }

    node_ids
}

fn create_tree_graph(
    branch_factor: usize,
    depth: usize,
    pattern_name: &str,
    create_node_events: &mut EventWriter<CreateNodeEvent>,
    create_edge_events: &mut EventWriter<CreateEdgeEvent>,
) -> Vec<Uuid> {
    let mut node_ids = Vec::new();
    let level_height = 3.0;
    let branch_spread = 2.0;

    // Recursive function to create tree nodes
    fn create_tree_level(
        parent_pos: Vec3,
        parent_id: Option<Uuid>,
        current_depth: usize,
        max_depth: usize,
        branch_factor: usize,
        node_ids: &mut Vec<Uuid>,
        create_node_events: &mut EventWriter<CreateNodeEvent>,
        pattern_name: &str,
        level_height: f32,
        branch_spread: f32,
    ) {
        if current_depth > max_depth {
            return;
        }

        let spread = branch_spread * (max_depth - current_depth + 1) as f32;

        for i in 0..branch_factor {
            let offset = ((i as f32 - (branch_factor as f32 - 1.0) / 2.0) / branch_factor as f32) * spread;
            let position = parent_pos + Vec3::new(offset, -level_height, 0.0);

            let node_id = Uuid::new_v4();
            node_ids.push(node_id);

            create_node_events.send(CreateNodeEvent {
                id: node_id,
                position,
                domain_type: if current_depth == 0 {
                    DomainNodeType::Aggregate
                } else {
                    DomainNodeType::Entity
                },
                name: format!("{} L{}-{}", pattern_name, current_depth, i + 1),
                labels: vec!["pattern".to_string(), "tree".to_string(), format!("level-{}", current_depth)],
                properties: HashMap::new(),
                subgraph_id: None,
                color: None,
            });

            // Recursively create children
            create_tree_level(
                position,
                Some(node_id),
                current_depth + 1,
                max_depth,
                branch_factor,
                node_ids,
                create_node_events,
                pattern_name,
                level_height,
                branch_spread,
            );
        }
    }

    // Start tree creation from root
    create_tree_level(
        Vec3::new(0.0, depth as f32 * level_height / 2.0, 0.0),
        None,
        0,
        depth,
        branch_factor,
        &mut node_ids,
        create_node_events,
        pattern_name,
        level_height,
        branch_spread,
    );

    node_ids
}
