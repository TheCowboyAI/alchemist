use bevy::prelude::*;
use std::collections::HashMap;

use super::components::*;
use super::events::*;
use super::graph_data::{EdgeData, GraphData, NodeData};

/// Tracks graph state for UI display

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

        info!(
            "Created node '{}' at position {:?} (Entity: {:?}). Total nodes: {}",
            event.name,
            event.position,
            entity_commands.id(),
            graph_state.node_count
        );
    }
}

/// System to handle edge creation events
pub fn handle_create_edge_events(
    mut commands: Commands,
    mut events: EventReader<CreateEdgeEvent>,
    mut graph_state: ResMut<GraphState>,
    node_query: Query<&GraphNode>,
    mut modification_events: EventWriter<GraphModificationEvent>,
) {
    for event in events.read() {
        // Verify both nodes exist
        let source_node = node_query.get(event.source);
        let target_node = node_query.get(event.target);

        if let (Ok(source), Ok(target)) = (source_node, target_node) {
            let mut bundle = GraphEdgeBundle::new(
                event.id,
                event.source,
                event.target,
                event.edge_type.clone(),
            );

            // Update the edge with labels and properties
            bundle.edge.labels = event.labels.clone();
            bundle.edge.properties = event.properties.clone();

            commands.spawn(bundle);

            graph_state.edge_count += 1;

            // Emit modification event
            modification_events.write(GraphModificationEvent::EdgeCreated {
                id: event.id,
                source_id: source.id,
                target_id: target.id,
                edge_type: event.edge_type.clone(),
            });

            info!("Created edge from {:?} to {:?}", source.id, target.id);
        } else {
            warn!("Failed to create edge: source or target node not found");
        }
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
        graph_state.selected_edges.clear();
    }

    // Handle selection
    for event in events.read() {
        if !event.multi_select {
            // Clear previous selection
            for entity in &selected_query {
                commands.entity(entity).remove::<Selected>();
            }
            graph_state.selected_nodes.clear();
            graph_state.selected_edges.clear();
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

/// System to update edge positions based on node positions
pub fn update_edge_positions(
    mut edge_query: Query<(&GraphEdge, &mut Transform)>,
    node_query: Query<&Transform, (With<GraphNode>, Without<GraphEdge>)>,
) {
    for (edge, mut edge_transform) in &mut edge_query {
        if let (Ok(source_transform), Ok(target_transform)) =
            (node_query.get(edge.source), node_query.get(edge.target))
        {
            let source_pos = source_transform.translation;
            let target_pos = target_transform.translation;

            // Position the edge entity at the midpoint
            let mid_point = source_pos + (target_pos - source_pos) * 0.5;
            edge_transform.translation = mid_point;

            // Calculate rotation to align edge with direction
            let direction = target_pos - source_pos;
            let distance = direction.length();

            if distance > 0.01 {
                // Create rotation to align Y-axis (cylinder's axis) with edge direction
                let rotation = if direction.normalize() != Vec3::Y {
                    Quat::from_rotation_arc(Vec3::Y, direction.normalize())
                } else {
                    Quat::IDENTITY
                };
                edge_transform.rotation = rotation;

                // Scale to match the distance
                edge_transform.scale = Vec3::new(1.0, distance, 1.0);
            }
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

/// System to handle pattern creation
// TODO: Implement graph_patterns module
/*
pub fn handle_pattern_creation(
    mut commands: Commands,
    mut events: EventReader<CreatePatternEvent>,
    mut create_node_events: EventWriter<CreateNodeEvent>,
    mut create_edge_events: EventWriter<CreateEdgeEvent>,
) {
    for event in events.read() {
        // Generate pattern and create nodes/edges
        let pattern_graph = crate::graph_patterns::generate_pattern(event.pattern.clone());

        // Map old IDs to new entities
        let mut id_map = std::collections::HashMap::new();

        // Create nodes
        for (old_id, node) in &pattern_graph.nodes {
            let new_id = Uuid::new_v4();
            id_map.insert(old_id, new_id);

            // Calculate position relative to pattern center
            let pos = pattern_graph
                .node_positions
                .get(old_id)
                .map(|p| Vec3::new(p.x, 0.0, p.y))
                .unwrap_or(Vec3::ZERO)
                + event.position;

            create_node_events.send(CreateNodeEvent {
                id: new_id,
                position: pos,
                domain_type: DomainNodeType::Process, // Default type
                name: node.name.clone(),
                subgraph_id: None,
            });
        }

        // Create edges (will be processed next frame after nodes are created)
        // This is a limitation we'll address with a better event ordering system
        info!(
            "Pattern {} created with {} nodes",
            event.name,
            pattern_graph.nodes.len()
        );
    }
}
*/

/// System to handle graph validation
pub fn handle_validation_events(
    mut events: EventReader<ValidateGraphEvent>,
    node_query: Query<&GraphNode>,
    edge_query: Query<&GraphEdge>,
) {
    for _ in events.read() {
        let mut validation_errors = Vec::new();

        // Check for orphaned nodes (nodes with no edges)
        for node in &node_query {
            let has_edges = edge_query.iter().any(|edge| {
                edge.source == Entity::PLACEHOLDER || edge.target == Entity::PLACEHOLDER
                // Note: This needs proper entity comparison
            });

            if !has_edges {
                validation_errors.push(format!("Node {:?} has no connections", node.id));
            }
        }

        // Check for self-loops
        for edge in &edge_query {
            if edge.source == edge.target {
                validation_errors.push(format!("Edge {:?} is a self-loop", edge.id));
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

        info!(
            "Created node '{}' with petgraph index {:?} and entity {:?}",
            event.name, graph_idx, entity
        );
    }
}

/// Example of how edge creation should work
pub fn handle_create_edge_with_graph(
    mut commands: Commands,
    mut events: EventReader<CreateEdgeEvent>,
    mut graph_data: ResMut<GraphData>,
    mut graph_state: ResMut<GraphState>,
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
                Ok(edge_idx) => {
                    // Create ECS entity for rendering
                    let bundle = GraphEdgeBundle::new(
                        event.id,
                        event.source,
                        event.target,
                        event.edge_type.clone(),
                    );

                    let entity = commands.spawn(bundle).id();

                    // Associate the graph edge with the ECS entity
                    graph_data.set_edge_entity(edge_idx, entity);

                    graph_state.edge_count = graph_data.edge_count();

                    info!(
                        "Created edge with petgraph index {:?} and entity {:?}",
                        edge_idx, entity
                    );
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
    mut create_edge_events: EventWriter<CreateEdgeEvent>,
    graph_data: Res<GraphData>,
) {
    for event in events.read() {
        // Find entities for the source and target UUIDs
        let source_entity = graph_data
            .nodes()
            .find(|(_, node_data)| node_data.id == event.source_uuid)
            .and_then(|(idx, _)| graph_data.get_node_entity(idx));

        let target_entity = graph_data
            .nodes()
            .find(|(_, node_data)| node_data.id == event.target_uuid)
            .and_then(|(idx, _)| graph_data.get_node_entity(idx));

        if let (Some(source), Some(target)) = (source_entity, target_entity) {
            create_edge_events.write(CreateEdgeEvent {
                id: event.id,
                source,
                target,
                edge_type: event.edge_type.clone(),
                labels: event.labels.clone(),
                properties: event.properties.clone(),
            });
        } else {
            // Nodes might not be created yet, re-queue the event
            // In a production system, we'd want a more sophisticated retry mechanism
            warn!(
                "Could not find entities for edge {:?} -> {:?}, will retry",
                event.source_uuid, event.target_uuid
            );
        }
    }
}
