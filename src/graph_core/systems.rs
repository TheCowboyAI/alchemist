use bevy::prelude::*;
use petgraph::Direction;

use super::{
    components::*, events::*, graph_data::{EdgeData, GraphData, NodeData},
    GraphNode, NodeVisual,
};

/// System to handle node creation events
pub fn handle_create_node_events(
    mut commands: Commands,
    mut events: EventReader<CreateNodeEvent>,
    mut graph_state: ResMut<GraphState>,
    mut modification_events: EventWriter<GraphModificationEvent>,
) {
    for event in events.read() {
        // Parse color from hex string if provided, otherwise use domain type color
        let color = if let Some(hex_color) = &event.color {
            parse_hex_color(hex_color)
                .unwrap_or_else(|| get_color_for_domain_type(&event.domain_type))
        } else {
            get_color_for_domain_type(&event.domain_type)
        };

        let mut entity_commands = commands.spawn(GraphNodeBundle::new(
            event.id,
            event.domain_type.clone(),
            event.position,
            color,
            event.name.clone(),
            event.labels.clone(),
            event.properties.clone(),
        ));

        entity_commands.insert(Name::new(event.name.clone()));

        if let Some(subgraph_id) = event.subgraph_id {
            entity_commands.insert(SubgraphMember { subgraph_id });
        }

        graph_state.node_count += 1;

        // Emit modification event for event sourcing
        modification_events.write(GraphModificationEvent::NodeCreated {
            id: event.id,
            position: event.position,
            domain_type: event.domain_type.clone(),
            name: event.name.clone(),
        });

        // Only log important nodes or in debug mode
        // info!(
        //     "Created node '{}' at position {:?} (Entity: {:?}). Total nodes: {}",
        //     event.name,
        //     event.position,
        //     entity_commands.id(),
        //     graph_state.node_count
        // );
    }
}

/// System to handle node movement
pub fn handle_move_node_events(
    mut events: EventReader<MoveNodeEvent>,
    mut node_query: Query<(&mut Transform, &mut GraphPosition, &GraphNode)>,
    mut modification_events: EventWriter<GraphModificationEvent>,
) {
    for event in events.read() {
        if let Ok((mut transform, mut position, node)) = node_query.get_mut(event.entity) {
            transform.translation = event.to;
            position.0 = event.to;

            modification_events.write(GraphModificationEvent::NodeMoved {
                id: node.id,
                from: event.from,
                to: event.to,
            });
        }
    }
}

/// System to handle selection events
pub fn handle_selection_events(
    mut commands: Commands,
    mut events: EventReader<SelectEvent>,
    mut deselect_events: EventReader<DeselectAllEvent>,
    mut graph_state: ResMut<GraphState>,
    selected_query: Query<Entity, With<Selected>>,
) {
    // Handle deselect all
    for _ in deselect_events.read() {
        for entity in &selected_query {
            commands.entity(entity).remove::<Selected>();
        }
        graph_state.selected_nodes.clear();
    }

    // Handle selection
    for event in events.read() {
        if !event.multi_select {
            // Clear previous selection
            for entity in &selected_query {
                commands.entity(entity).remove::<Selected>();
            }
            graph_state.selected_nodes.clear();
        }

        // Add new selection
        commands.entity(event.entity).insert(Selected);

        // Update graph state
        if graph_state.selected_nodes.contains(&event.entity) {
            // Already selected, remove it (toggle)
            graph_state.selected_nodes.retain(|&e| e != event.entity);
            commands.entity(event.entity).remove::<Selected>();
        } else {
            graph_state.selected_nodes.push(event.entity);
        }
    }
}

/// System to handle hover events
pub fn handle_hover_events(
    mut commands: Commands,
    mut events: EventReader<HoverEvent>,
    mut graph_state: ResMut<GraphState>,
    hovered_query: Query<Entity, With<Hovered>>,
) {
    for event in events.read() {
        // Clear previous hover
        for entity in &hovered_query {
            commands.entity(entity).remove::<Hovered>();
        }

        // Set new hover
        if let Some(entity) = event.entity {
            commands.entity(entity).insert(Hovered);
            graph_state.hovered_entity = Some(entity);
        } else {
            graph_state.hovered_entity = None;
        }
    }
}

/// System to update visual appearance based on selection/hover state
pub fn update_node_visuals(
    mut node_query: Query<
        (&mut NodeVisual, Option<&Selected>, Option<&Hovered>),
        Changed<Selected>,
    >,
) {
    for (mut visual, selected, hovered) in &mut node_query {
        if selected.is_some() {
            visual.current_color = visual.base_color.lighter(0.3);
        } else if hovered.is_some() {
            visual.current_color = visual.base_color.lighter(0.1);
        } else {
            visual.current_color = visual.base_color;
        }
    }
}

/// System to handle graph validation
pub fn handle_validation_events(
    mut events: EventReader<ValidateGraphEvent>,
    graph_data: Res<GraphData>,
) {
    for _ in events.read() {
        let mut validation_errors = Vec::new();

        // Check for orphaned nodes (nodes with no edges)
        for (node_idx, node_data) in graph_data.nodes() {
            let has_incoming = graph_data.graph.edges_directed(node_idx, Direction::Incoming).count() > 0;
            let has_outgoing = graph_data.graph.edges_directed(node_idx, Direction::Outgoing).count() > 0;

            if !has_incoming && !has_outgoing {
                validation_errors.push(format!("Node '{}' ({:?}) has no connections", node_data.name, node_data.id));
            }
        }

        // Check for self-loops
        for (_edge_idx, edge_data, source_idx, target_idx) in graph_data.edges() {
            if source_idx == target_idx {
                validation_errors.push(format!("Edge {:?} is a self-loop", edge_data.id));
            }
        }

        if validation_errors.is_empty() {
            info!("Graph validation passed");
        } else {
            warn!(
                "Graph validation failed with {} errors",
                validation_errors.len()
            );
            for error in validation_errors {
                warn!("  - {}", error);
            }
        }
    }
}

/// Helper function to get color based on domain type
fn get_color_for_domain_type(domain_type: &DomainNodeType) -> Color {
    match domain_type {
        DomainNodeType::Process => Color::srgb(0.2, 0.4, 0.8),
        DomainNodeType::Decision => Color::srgb(0.9, 0.7, 0.1),
        DomainNodeType::Event => Color::srgb(0.1, 0.7, 0.3),
        DomainNodeType::Storage => Color::srgb(0.5, 0.3, 0.8),
        DomainNodeType::Interface => Color::srgb(0.4, 0.7, 0.9),
        DomainNodeType::Custom(_) => Color::srgb(0.6, 0.6, 0.6),
    }
}

/// Helper function to parse hex color string
fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;

    Some(Color::srgb(r, g, b))
}

/// Example of how node creation should work with proper graph data structure
pub fn handle_create_node_with_graph(
    mut commands: Commands,
    mut events: EventReader<CreateNodeEvent>,
    mut graph_data: ResMut<GraphData>,
    mut graph_state: ResMut<GraphState>,
) {
    for event in events.read() {
        // First, add to the graph data structure
        let node_data = NodeData {
            id: event.id,
            name: event.name.clone(),
            domain_type: event.domain_type.clone(),
            position: event.position,
            labels: event.labels.clone(),
            properties: event.properties.clone(),
        };

        let graph_idx = graph_data.add_node(node_data);

        // Then create the ECS entity for rendering
        let color = event
            .color
            .as_ref()
            .and_then(|hex| parse_hex_color(hex))
            .unwrap_or_else(|| get_color_for_domain_type(&event.domain_type));

        let entity = commands
            .spawn(GraphNodeBundle::new(
                event.id,
                event.domain_type.clone(),
                event.position,
                color,
                event.name.clone(),
                event.labels.clone(),
                event.properties.clone(),
            ))
            .insert(Name::new(event.name.clone()))
            .id();

        // Associate the graph node with the ECS entity
        graph_data.set_node_entity(graph_idx, entity);

        if let Some(subgraph_id) = event.subgraph_id {
            commands
                .entity(entity)
                .insert(SubgraphMember { subgraph_id });
        }

        graph_state.node_count = graph_data.node_count();

        // Only log in debug mode or when specifically needed
        // info!(
        //     "Created node '{}' with petgraph index {:?} and entity {:?}",
        //     event.name, graph_idx, entity
        // );
    }
}

/// Example of how edge creation should work
pub fn handle_create_edge_with_graph(
    mut events: EventReader<CreateEdgeEvent>,
    mut graph_data: ResMut<GraphData>,
    mut graph_state: ResMut<GraphState>,
    mut commands: Commands,
) {
    for event in events.read() {
        // Get the node UUIDs for source and target entities
        let source_node = graph_data
            .nodes()
            .find(|(idx, _)| graph_data.get_node_entity(*idx) == Some(event.source))
            .map(|(_, data)| data.id);

        let target_node = graph_data
            .nodes()
            .find(|(idx, _)| graph_data.get_node_entity(*idx) == Some(event.target))
            .map(|(_, data)| data.id);

        if let (Some(source_id), Some(target_id)) = (source_node, target_node) {
            // Add to graph data structure
            let edge_data = EdgeData {
                id: event.id,
                edge_type: event.edge_type.clone(),
                labels: event.labels.clone(),
                properties: event.properties.clone(),
            };

            match graph_data.add_edge(source_id, target_id, edge_data) {
                Ok(_edge_idx) => {
                    graph_state.edge_count = graph_data.edge_count();

                    // Attach OutgoingEdge component to the source node entity
                    commands.entity(event.source).insert(OutgoingEdge {
                        id: event.id,
                        target: event.target,
                        edge_type: event.edge_type.clone(),
                        labels: event.labels.clone(),
                        properties: event.properties.clone(),
                    });
                }
                Err(e) => {
                    warn!("Failed to create edge: {}", e);
                }
            }
        } else {
            warn!("Failed to create edge: source or target node not found in graph");
        }
    }
}

/// System to process deferred edge events after nodes have been created
pub fn process_deferred_edges(
    mut events: EventReader<DeferredEdgeEvent>,
    mut graph_data: ResMut<GraphData>,
    mut deferred_edge_events: EventWriter<DeferredEdgeEvent>,
) {
    let mut events_to_retry = Vec::new();

    for event in events.read() {
        // Try to add the edge directly to GraphData
        let edge_data = EdgeData {
            id: event.id,
            edge_type: event.edge_type.clone(),
            labels: event.labels.clone(),
            properties: event.properties.clone(),
        };

        match graph_data.add_edge(event.source_uuid, event.target_uuid, edge_data) {
            Ok(_) => {
                // Successfully added edge
            }
            Err(_) if event.retry_count < 3 => {
                // Nodes might not exist yet, retry later
                let mut retry_event = event.clone();
                retry_event.retry_count += 1;
                events_to_retry.push(retry_event);
            }
            Err(e) => {
                // Give up after 3 retries
                warn!("Failed to create edge after {} retries: {}", event.retry_count, e);
            }
        }
    }

    // Re-queue events that need retry
    for event in events_to_retry {
        deferred_edge_events.write(event);
    }
}

/// System to handle edge deletion events and remove OutgoingEdge components
pub fn handle_delete_edge_events(
    mut commands: Commands,
    mut events: EventReader<DeleteEdgeEvent>,
    mut node_query: Query<&mut OutgoingEdge, With<GraphNode>>,
) {
    for event in events.read() {
        if let Ok(outgoing_edge) = node_query.get_mut(event.source) {
            if outgoing_edge.id == event.edge_id {
                commands.entity(event.source).remove::<OutgoingEdge>();
            }
        }
    }
}

/// System to handle UUID-based edge creation (for JSON loading and demo setup)
pub fn handle_deferred_edge_events(
    mut events: EventReader<DeferredEdgeEvent>,
    mut graph_data: ResMut<GraphData>,
    mut graph_state: ResMut<GraphState>,
    mut commands: Commands,
) {
    // Only process if we have nodes in the graph
    if graph_data.node_count() == 0 {
        return;
    }

    for event in events.read() {
        // Check if both source and target nodes exist in GraphData
        let source_exists = graph_data.nodes()
            .any(|(_, data)| data.id == event.source_uuid);
        let target_exists = graph_data.nodes()
            .any(|(_, data)| data.id == event.target_uuid);

        if !source_exists {
            warn!("Skipping edge {:?}: Source node {:?} not found in GraphData",
                  event.id, event.source_uuid);
            continue;
        }

        if !target_exists {
            warn!("Skipping edge {:?}: Target node {:?} not found in GraphData",
                  event.id, event.target_uuid);
            continue;
        }

        // Try to add the edge directly to GraphData using UUIDs
        let edge_data = EdgeData {
            id: event.id,
            edge_type: event.edge_type.clone(),
            labels: event.labels.clone(),
            properties: event.properties.clone(),
        };

        match graph_data.add_edge(event.source_uuid, event.target_uuid, edge_data) {
            Ok(_) => {
                graph_state.edge_count = graph_data.edge_count();

                // Find the source and target entities to add OutgoingEdge component
                let source_entity = graph_data.nodes()
                    .find(|(_, data)| data.id == event.source_uuid)
                    .and_then(|(idx, _)| graph_data.get_node_entity(idx));

                let target_entity = graph_data.nodes()
                    .find(|(_, data)| data.id == event.target_uuid)
                    .and_then(|(idx, _)| graph_data.get_node_entity(idx));

                if let (Some(source), Some(target)) = (source_entity, target_entity) {
                    // Add OutgoingEdge component to the source node entity
                    commands.entity(source).insert(OutgoingEdge {
                        id: event.id,
                        target,
                        edge_type: event.edge_type.clone(),
                        labels: event.labels.clone(),
                        properties: event.properties.clone(),
                    });

                    info!("Successfully created edge {:?} from {:?} to {:?}",
                          event.id, event.source_uuid, event.target_uuid);
                } else {
                    warn!("Could not find entities for edge {:?}: source_entity={:?}, target_entity={:?}",
                          event.id, source_entity, target_entity);
                }
            }
            Err(e) => {
                warn!("Failed to create edge {:?}: {}", event.id, e);
            }
        }
    }
}

// NOTE: These systems are deprecated - edges are no longer entities
/*
/// System to ensure all edges in GraphData have corresponding visual entities
pub fn synchronize_edge_entities(
    mut commands: Commands,
    graph_data: Res<GraphData>,
    existing_edges: Query<(Entity, &GraphEdge)>,
) {
    // Create a set of edge IDs that already have entities
    let existing_edge_ids: std::collections::HashSet<_> = existing_edges
        .iter()
        .map(|(_, edge)| edge.id)
        .collect();

    // Check all edges in the graph data
    for (_edge_idx, edge_data, source_idx, target_idx) in graph_data.edges() {
        // Skip if this edge already has an entity
        if existing_edge_ids.contains(&edge_data.id) {
            continue;
        }

        // Get the source and target entities
        let source_entity = graph_data.get_node_entity(source_idx);
        let target_entity = graph_data.get_node_entity(target_idx);

        if let (Some(source), Some(target)) = (source_entity, target_entity) {
            // Create the edge entity as a standalone entity
            let bundle = GraphEdgeBundle::new(
                edge_data.id,
                source,
                target,
                edge_data.edge_type.clone(),
            );

            commands.spawn(bundle);

            // Note: We can't set the edge entity in GraphData here because it's not mutable
            // This would need to be handled in a separate system or by making GraphData mutable

            // Only log in debug mode
            // info!("Created missing edge entity for edge {:?}", edge_data.id);
        } else {
            warn!(
                "Could not create edge entity: missing node entities for edge {:?}",
                edge_data.id
            );
        }
    }
}

/// System to clean up orphaned edges when nodes are despawned
pub fn cleanup_orphaned_edges(
    mut commands: Commands,
    edge_query: Query<(Entity, &GraphEdge)>,
    node_query: Query<Entity, With<GraphNode>>,
) {
    // Collect all valid node entities
    let valid_nodes: std::collections::HashSet<Entity> = node_query.iter().collect();

    // Check each edge and despawn if source or target doesn't exist
    for (edge_entity, edge) in &edge_query {
        if !valid_nodes.contains(&edge.source) || !valid_nodes.contains(&edge.target) {
            commands.entity(edge_entity).despawn_recursive();
        }
    }
}
*/
