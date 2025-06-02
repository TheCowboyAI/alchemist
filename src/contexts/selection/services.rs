use super::domain::*;
use super::events::*;
use crate::contexts::graph_management::domain::{Edge, EdgeIdentity, Node, NodeIdentity};
use crate::contexts::visualization::services::{EdgeVisual, PerformRaycast};
use bevy::prelude::*;
use bevy::transform::components::GlobalTransform;

// ============= Selection Services =============

/// Service to manage selection state
pub struct ManageSelection;

impl ManageSelection {
    /// Process node selection events
    pub fn handle_node_selected(
        mut commands: Commands,
        mut events: EventReader<NodeSelected>,
        mut selection_state: ResMut<SelectionState>,
        mut selection_changed: EventWriter<SelectionChanged>,
        selected_query: Query<Entity, With<Selected>>,
    ) {
        for event in events.read() {
            let mut added_nodes = Vec::new();
            let mut removed_nodes = Vec::new();

            // Handle selection based on mode
            match selection_state.selection_mode {
                SelectionMode::Single => {
                    // Clear existing selections
                    for entity in selected_query.iter() {
                        commands.entity(entity).remove::<Selected>();
                    }

                    // Clear selection state and track removed nodes
                    for node in selection_state.selected_nodes.drain() {
                        removed_nodes.push(node);
                    }
                    for _edge in selection_state.selected_edges.drain() {
                        // Edge deselection handled separately
                    }

                    // Select the new node
                    commands.entity(event.entity).insert(Selected);
                    selection_state.select_node(event.node);
                    added_nodes.push(event.node);
                }
                SelectionMode::Multiple => {
                    if event.add_to_selection {
                        // Add to selection
                        if !selection_state.is_node_selected(&event.node) {
                            commands.entity(event.entity).insert(Selected);
                            selection_state.select_node(event.node);
                            added_nodes.push(event.node);
                        }
                    } else {
                        // Replace selection
                        for entity in selected_query.iter() {
                            commands.entity(entity).remove::<Selected>();
                        }

                        for node in selection_state.selected_nodes.drain() {
                            removed_nodes.push(node);
                        }
                        selection_state.selected_edges.clear();

                        commands.entity(event.entity).insert(Selected);
                        selection_state.select_node(event.node);
                        added_nodes.push(event.node);
                    }
                }
                _ => {
                    // Box and Lasso modes handled separately
                }
            }

            // Fire selection changed event if anything changed
            if !added_nodes.is_empty() || !removed_nodes.is_empty() {
                selection_changed.write(SelectionChanged {
                    added_nodes,
                    removed_nodes,
                    added_edges: Vec::new(),
                    removed_edges: Vec::new(),
                });
            }
        }
    }

    /// Process node deselection events
    pub fn handle_node_deselected(
        mut commands: Commands,
        mut events: EventReader<NodeDeselected>,
        mut selection_state: ResMut<SelectionState>,
        mut selection_changed: EventWriter<SelectionChanged>,
    ) {
        for event in events.read() {
            if selection_state.is_node_selected(&event.node) {
                commands.entity(event.entity).remove::<Selected>();
                selection_state.deselect_node(&event.node);

                selection_changed.write(SelectionChanged {
                    added_nodes: Vec::new(),
                    removed_nodes: vec![event.node],
                    added_edges: Vec::new(),
                    removed_edges: Vec::new(),
                });
            }
        }
    }

    /// Process edge selection events
    pub fn handle_edge_selected(
        mut commands: Commands,
        mut events: EventReader<EdgeSelected>,
        mut selection_state: ResMut<SelectionState>,
        mut selection_changed: EventWriter<SelectionChanged>,
        selected_query: Query<Entity, With<Selected>>,
    ) {
        for event in events.read() {
            let mut added_edges = Vec::new();
            let mut removed_edges = Vec::new();

            match selection_state.selection_mode {
                SelectionMode::Single => {
                    // Clear existing selections
                    for entity in selected_query.iter() {
                        commands.entity(entity).remove::<Selected>();
                    }

                    selection_state.selected_nodes.clear();
                    for edge in selection_state.selected_edges.drain() {
                        removed_edges.push(edge);
                    }

                    // Select the new edge
                    commands.entity(event.entity).insert(Selected);
                    selection_state.select_edge(event.edge);
                    added_edges.push(event.edge);
                }
                SelectionMode::Multiple => {
                    if event.add_to_selection {
                        if !selection_state.is_edge_selected(&event.edge) {
                            commands.entity(event.entity).insert(Selected);
                            selection_state.select_edge(event.edge);
                            added_edges.push(event.edge);
                        }
                    } else {
                        // Replace selection
                        for entity in selected_query.iter() {
                            commands.entity(entity).remove::<Selected>();
                        }

                        selection_state.selected_nodes.clear();
                        for edge in selection_state.selected_edges.drain() {
                            removed_edges.push(edge);
                        }

                        commands.entity(event.entity).insert(Selected);
                        selection_state.select_edge(event.edge);
                        added_edges.push(event.edge);
                    }
                }
                _ => {}
            }

            if !added_edges.is_empty() || !removed_edges.is_empty() {
                selection_changed.write(SelectionChanged {
                    added_nodes: Vec::new(),
                    removed_nodes: Vec::new(),
                    added_edges,
                    removed_edges,
                });
            }
        }
    }

    /// Clear all selections
    pub fn handle_selection_cleared(
        mut commands: Commands,
        mut events: EventReader<SelectionCleared>,
        mut selection_state: ResMut<SelectionState>,
        selected_query: Query<Entity, With<Selected>>,
    ) {
        for _ in events.read() {
            // Remove Selected component from all entities
            for entity in selected_query.iter() {
                commands.entity(entity).remove::<Selected>();
            }

            // Clear selection state
            selection_state.clear();

            info!("All selections cleared");
        }
    }

    /// Handle selection mode changes
    pub fn handle_selection_mode_changed(
        mut events: EventReader<SelectionModeChanged>,
        mut selection_state: ResMut<SelectionState>,
    ) {
        for event in events.read() {
            selection_state.selection_mode = event.new_mode;
            info!("Selection mode changed to: {:?}", event.new_mode);
        }
    }
}

/// Service to handle selection highlighting
pub struct HighlightSelection;

impl HighlightSelection {
    /// Apply visual highlighting to selected entities
    pub fn apply_selection_highlight(
        mut materials: ResMut<Assets<StandardMaterial>>,
        selected_query: Query<
            (&MeshMaterial3d<StandardMaterial>, &SelectionHighlight),
            Added<Selected>,
        >,
    ) {
        for (material_handle, highlight) in selected_query.iter() {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                // Apply highlight
                material.base_color = highlight.highlight_color;
                material.emissive =
                    LinearRgba::from(highlight.highlight_color) * highlight.highlight_intensity;
                material.metallic = 0.5;
                material.perceptual_roughness = 0.3;
            }
        }
    }

    /// Remove visual highlighting from deselected entities
    pub fn remove_selection_highlight(
        mut materials: ResMut<Assets<StandardMaterial>>,
        deselected_query: Query<
            &MeshMaterial3d<StandardMaterial>,
            (Without<Selected>, Changed<Selected>),
        >,
        nodes: Query<&Node>,
        _edges: Query<&Edge>,
    ) {
        for material_handle in deselected_query.iter() {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                // Restore default appearance
                // Check if it's a node or edge to apply correct default color
                let is_node = nodes.iter().any(|_| true);

                if is_node {
                    material.base_color = Color::srgb(0.3, 0.5, 0.9); // Default node color
                } else {
                    material.base_color = Color::srgb(0.8, 0.8, 0.8); // Default edge color
                }

                material.emissive = LinearRgba::BLACK;
                material.metallic = 0.0;
                material.perceptual_roughness = 0.5;
            }
        }
    }

    /// Highlight hovered entities
    pub fn apply_hover_highlight(
        mut materials: ResMut<Assets<StandardMaterial>>,
        hovered_query: Query<
            &MeshMaterial3d<StandardMaterial>,
            (Added<Hovered>, Without<Selected>),
        >,
    ) {
        for material_handle in hovered_query.iter() {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                // Apply subtle hover effect
                material.emissive = LinearRgba::rgb(0.1, 0.1, 0.2);
            }
        }
    }

    /// Remove hover highlighting
    pub fn remove_hover_highlight(
        mut materials: ResMut<Assets<StandardMaterial>>,
        unhovered_query: Query<
            &MeshMaterial3d<StandardMaterial>,
            (Without<Hovered>, Without<Selected>, Changed<Hovered>),
        >,
    ) {
        for material_handle in unhovered_query.iter() {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                material.emissive = LinearRgba::BLACK;
            }
        }
    }

    /// Process entity hover events
    pub fn handle_entity_hovered(mut commands: Commands, mut events: EventReader<EntityHovered>) {
        for event in events.read() {
            // Add Hovered component to the entity
            commands.entity(event.entity).insert(Hovered);

            info!(
                "Entity {:?} hovered (is_node: {}, is_edge: {})",
                event.entity, event.is_node, event.is_edge
            );
        }
    }

    /// Process entity unhover events
    pub fn handle_entity_unhovered(
        mut commands: Commands,
        mut events: EventReader<EntityUnhovered>,
    ) {
        for event in events.read() {
            // Remove Hovered component from the entity
            commands.entity(event.entity).remove::<Hovered>();

            info!("Entity {:?} unhovered", event.entity);
        }
    }
}

/// Service to handle selection input
pub struct ProcessSelectionInput;

impl ProcessSelectionInput {
    /// Handle mouse input for selection
    pub fn handle_mouse_selection(
        windows: Query<&Window>,
        camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
        mouse_button: Res<ButtonInput<MouseButton>>,
        keyboard: Res<ButtonInput<KeyCode>>,
        nodes: Query<(Entity, &Transform, &NodeIdentity), (With<Node>, With<Selectable>)>,
        edges: Query<
            (Entity, &Transform, &EdgeIdentity, &EdgeVisual),
            (With<Edge>, With<Selectable>),
        >,
        mut select_node_events: EventWriter<NodeSelected>,
        mut select_edge_events: EventWriter<EdgeSelected>,
        mut start_box_events: EventWriter<BoxSelectionStarted>,
        mut selection_cleared_events: EventWriter<SelectionCleared>,
        mut complete_box_events: EventWriter<BoxSelectionCompleted>,
        mut update_box_events: EventWriter<BoxSelectionUpdated>,
        box_query: Query<&SelectionBox>,
        _selection_state: Res<SelectionState>,
    ) {
        let Ok(window) = windows.single() else {
            return;
        };

        // Check if we're in box selection mode
        let in_box_selection = !box_query.is_empty() && box_query.iter().any(|b| b.active);

        // Handle mouse movement during box selection
        if in_box_selection {
            if let Some(cursor_position) = window.cursor_position() {
                update_box_events.write(BoxSelectionUpdated {
                    current_position: cursor_position,
                });
            }

            // Complete box selection on mouse release
            if mouse_button.just_released(MouseButton::Left) {
                if let Some(cursor_position) = window.cursor_position() {
                    complete_box_events.write(BoxSelectionCompleted {
                        end_position: cursor_position,
                    });
                    info!(
                        "Published BoxSelectionCompleted event at position: {:?}",
                        cursor_position
                    );
                }
                return;
            }
        }

        // Left click for selection
        if mouse_button.just_pressed(MouseButton::Left) {
            let Ok((camera, camera_transform)) = camera.single() else {
                warn!("No camera found for selection");
                return;
            };
            let Some(cursor_position) = window.cursor_position() else {
                warn!("No cursor position found");
                return;
            };

            info!("Mouse clicked at position: {:?}", cursor_position);

            // Check for modifier keys
            let add_to_selection =
                keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
            let box_selection =
                keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

            if box_selection {
                // Start box selection
                start_box_events.write(BoxSelectionStarted {
                    start_position: cursor_position,
                });
                return;
            }

            // Perform raycast
            let Some(ray) =
                PerformRaycast::screen_to_ray(camera, camera_transform, cursor_position)
            else {
                warn!("Failed to create ray from cursor position");
                return;
            };

            info!(
                "Ray created - origin: {:?}, direction: {:?}",
                ray.origin, ray.direction
            );

            // Check node intersections
            let mut closest_node: Option<(Entity, NodeIdentity, f32)> = None;
            let node_count = nodes.iter().count();
            info!("Checking {} nodes for intersection", node_count);

            // Base node sphere radius
            const BASE_NODE_RADIUS: f32 = 0.3; // Match the visual sphere size
            const SELECTION_MARGIN: f32 = 1.3; // 30% larger for easier selection

            for (entity, transform, node_id) in nodes.iter() {
                // Use the actual animated position
                let sphere_center = transform.translation;

                // Account for scale changes from animations (e.g., NodePulse)
                // Take the average of x, y, z scale to handle non-uniform scaling
                let avg_scale = (transform.scale.x + transform.scale.y + transform.scale.z) / 3.0;
                let effective_radius = BASE_NODE_RADIUS * avg_scale * SELECTION_MARGIN;

                if let Some(distance) =
                    PerformRaycast::ray_intersects_sphere(&ray, sphere_center, effective_radius)
                {
                    info!(
                        "Node {:?} at position {:?} with scale {:?} (radius: {}) intersected at distance {}",
                        node_id, sphere_center, transform.scale, effective_radius, distance
                    );
                    match &closest_node {
                        None => closest_node = Some((entity, *node_id, distance)),
                        Some((_, _, closest_distance)) => {
                            if distance < *closest_distance {
                                closest_node = Some((entity, *node_id, distance));
                            }
                        }
                    }
                } else {
                    // Debug: log nodes that were checked but didn't intersect
                    info!(
                        "Node {:?} at position {:?} with scale {:?} (radius: {}) - no intersection",
                        node_id, sphere_center, transform.scale, effective_radius
                    );
                }
            }

            // Check edge intersections if no node was hit
            let mut closest_edge: Option<(Entity, EdgeIdentity, f32)> = None;

            if closest_node.is_none() {
                let edge_count = edges.iter().count();
                info!(
                    "No node hit, checking {} edges for intersection",
                    edge_count
                );

                for (entity, transform, edge_id, visual) in edges.iter() {
                    // For edges, use the animated transform position
                    let edge_center = transform.translation;

                    // Account for edge scale changes from animations (e.g., EdgePulse)
                    // Edges typically scale x and y for thickness changes
                    let scale_factor = (transform.scale.x + transform.scale.y) / 2.0;
                    let edge_radius = visual.thickness * scale_factor * 5.0; // Large hit area

                    if let Some(distance) =
                        PerformRaycast::ray_intersects_sphere(&ray, edge_center, edge_radius)
                    {
                        info!(
                            "Edge {:?} at position {:?} with scale {:?} (radius: {}) intersected at distance {}",
                            edge_id, edge_center, transform.scale, edge_radius, distance
                        );
                        match &closest_edge {
                            None => closest_edge = Some((entity, *edge_id, distance)),
                            Some((_, _, closest_distance)) => {
                                if distance < *closest_distance {
                                    closest_edge = Some((entity, *edge_id, distance));
                                }
                            }
                        }
                    }
                }
            }

            // Process selection
            if let Some((entity, node_id, distance)) = closest_node {
                info!(
                    "Selecting closest node: {:?} at distance {}",
                    node_id, distance
                );
                select_node_events.write(NodeSelected {
                    entity,
                    node: node_id,
                    add_to_selection,
                });
            } else if let Some((entity, edge_id, distance)) = closest_edge {
                info!(
                    "Selecting closest edge: {:?} at distance {}",
                    edge_id, distance
                );
                select_edge_events.write(EdgeSelected {
                    entity,
                    edge: edge_id,
                    add_to_selection,
                });
            } else if !add_to_selection {
                info!("No intersection found, clearing selection");
                // Clicked on empty space - clear selection
                selection_cleared_events.write(SelectionCleared);
            }
        }

        // Right click to clear selection
        if mouse_button.just_pressed(MouseButton::Right) {
            info!("Right click - clearing selection");
            selection_cleared_events.write(SelectionCleared);
        }
    }

    /// Handle keyboard shortcuts for selection
    pub fn handle_keyboard_selection(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut select_all_events: EventWriter<AllSelected>,
        mut invert_events: EventWriter<SelectionInverted>,
        mut mode_changed_events: EventWriter<SelectionModeChanged>,
        selection_state: Res<SelectionState>,
    ) {
        // Ctrl+A to select all
        if (keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight))
            && keyboard.just_pressed(KeyCode::KeyA)
        {
            select_all_events.write(AllSelected);
        }

        // Ctrl+I to invert selection
        if (keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight))
            && keyboard.just_pressed(KeyCode::KeyI)
        {
            invert_events.write(SelectionInverted);
        }

        // Tab to cycle selection modes
        if keyboard.just_pressed(KeyCode::Tab) {
            let new_mode = match selection_state.selection_mode {
                SelectionMode::Single => SelectionMode::Multiple,
                SelectionMode::Multiple => SelectionMode::Box,
                SelectionMode::Box => SelectionMode::Single,
                SelectionMode::Lasso => SelectionMode::Single,
            };

            mode_changed_events.write(SelectionModeChanged {
                new_mode,
                previous_mode: selection_state.selection_mode,
            });
        }
    }
}

/// Service to handle box selection
pub struct PerformBoxSelection;

impl PerformBoxSelection {
    /// Start box selection
    pub fn handle_box_selection_started(
        mut commands: Commands,
        mut events: EventReader<BoxSelectionStarted>,
    ) {
        for event in events.read() {
            // Create selection box entity
            commands.spawn(SelectionBox::new(event.start_position));
            info!("Started box selection at {:?}", event.start_position);
        }
    }

    /// Update box selection
    pub fn handle_box_selection_updated(
        mut events: EventReader<BoxSelectionUpdated>,
        mut box_query: Query<&mut SelectionBox>,
    ) {
        for event in events.read() {
            for mut selection_box in box_query.iter_mut() {
                if selection_box.active {
                    selection_box.end = event.current_position;
                }
            }
        }
    }

    /// Complete box selection
    pub fn handle_box_selection_completed(
        mut commands: Commands,
        mut events: EventReader<BoxSelectionCompleted>,
        mut box_query: Query<(Entity, &SelectionBox)>,
        camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
        nodes: Query<(Entity, &Transform, &NodeIdentity), (With<Node>, With<Selectable>)>,
        mut select_node_events: EventWriter<NodeSelected>,
    ) {
        for event in events.read() {
            let Ok((camera, camera_transform)) = camera.single() else {
                continue;
            };

            for (box_entity, selection_box) in box_query.iter_mut() {
                if !selection_box.active {
                    continue;
                }

                // Get box bounds using the event's end position
                let min = Vec2::new(
                    selection_box.start.x.min(event.end_position.x),
                    selection_box.start.y.min(event.end_position.y),
                );
                let max = Vec2::new(
                    selection_box.start.x.max(event.end_position.x),
                    selection_box.start.y.max(event.end_position.y),
                );

                info!(
                    "Completing box selection from {:?} to {:?}",
                    selection_box.start, event.end_position
                );

                // Check which nodes are within the box
                for (entity, transform, node_id) in nodes.iter() {
                    // Project node position to screen space using the animated transform
                    if let Ok(screen_pos) =
                        camera.world_to_viewport(camera_transform, transform.translation)
                    {
                        // For box selection, we should also consider the node's visual size
                        // Account for scale to make selection more forgiving
                        let avg_scale =
                            (transform.scale.x + transform.scale.y + transform.scale.z) / 3.0;
                        let screen_radius = 20.0 * avg_scale; // Approximate screen-space radius

                        // Check if node center or any part of it is within box bounds
                        let node_min_x = screen_pos.x - screen_radius;
                        let node_max_x = screen_pos.x + screen_radius;
                        let node_min_y = screen_pos.y - screen_radius;
                        let node_max_y = screen_pos.y + screen_radius;

                        // Check for box-circle intersection
                        let intersects = !(node_max_x < min.x
                            || node_min_x > max.x
                            || node_max_y < min.y
                            || node_min_y > max.y);

                        if intersects {
                            select_node_events.write(NodeSelected {
                                entity,
                                node: *node_id,
                                add_to_selection: true,
                            });
                            info!(
                                "Box selected node {:?} at screen pos {:?} with scale {:?}",
                                node_id, screen_pos, transform.scale
                            );
                        }
                    }
                }

                // Remove selection box
                commands.entity(box_entity).despawn();
            }

            info!(
                "Completed box selection at position: {:?}",
                event.end_position
            );
        }
    }
}

/// Service to handle advanced selection operations
pub struct AdvancedSelection;

impl AdvancedSelection {
    /// Select all visible entities
    pub fn handle_all_selected(
        mut events: EventReader<AllSelected>,
        nodes: Query<(Entity, &NodeIdentity), (With<Node>, With<Selectable>)>,
        edges: Query<(Entity, &EdgeIdentity), (With<Edge>, With<Selectable>)>,
        mut select_node_events: EventWriter<NodeSelected>,
        mut select_edge_events: EventWriter<EdgeSelected>,
    ) {
        for _ in events.read() {
            // Select all nodes
            for (entity, node_id) in nodes.iter() {
                select_node_events.write(NodeSelected {
                    entity,
                    node: *node_id,
                    add_to_selection: true,
                });
            }

            // Select all edges
            for (entity, edge_id) in edges.iter() {
                select_edge_events.write(EdgeSelected {
                    entity,
                    edge: *edge_id,
                    add_to_selection: true,
                });
            }

            info!("Selected all entities");
        }
    }

    /// Invert current selection
    pub fn handle_selection_inverted(
        mut events: EventReader<SelectionInverted>,
        selection_state: Res<SelectionState>,
        nodes: Query<(Entity, &NodeIdentity), (With<Node>, With<Selectable>)>,
        edges: Query<(Entity, &EdgeIdentity), (With<Edge>, With<Selectable>)>,
        mut select_node_events: EventWriter<NodeSelected>,
        mut deselect_node_events: EventWriter<NodeDeselected>,
        mut select_edge_events: EventWriter<EdgeSelected>,
        mut deselect_edge_events: EventWriter<EdgeDeselected>,
    ) {
        for _ in events.read() {
            // Process nodes
            for (entity, node_id) in nodes.iter() {
                if selection_state.is_node_selected(node_id) {
                    deselect_node_events.write(NodeDeselected {
                        entity,
                        node: *node_id,
                    });
                } else {
                    select_node_events.write(NodeSelected {
                        entity,
                        node: *node_id,
                        add_to_selection: true,
                    });
                }
            }

            // Process edges
            for (entity, edge_id) in edges.iter() {
                if selection_state.is_edge_selected(edge_id) {
                    deselect_edge_events.write(EdgeDeselected {
                        entity,
                        edge: *edge_id,
                    });
                } else {
                    select_edge_events.write(EdgeSelected {
                        entity,
                        edge: *edge_id,
                        add_to_selection: true,
                    });
                }
            }

            info!("Inverted selection");
        }
    }

    /// Select all nodes connected to a given node
    pub fn handle_connected_nodes_selected(
        mut events: EventReader<ConnectedNodesSelected>,
        nodes: Query<(Entity, &NodeIdentity), (With<Node>, With<Selectable>)>,
        edges: Query<&Edge, With<Selectable>>,
        mut select_node_events: EventWriter<NodeSelected>,
    ) {
        for event in events.read() {
            // Build a set to track visited nodes
            let mut visited = std::collections::HashSet::new();
            let mut to_visit = vec![(event.from_node, 0u32)];

            // Breadth-first search to find connected nodes
            while let Some((current_node, depth)) = to_visit.pop() {
                if depth > event.depth || visited.contains(&current_node) {
                    continue;
                }

                visited.insert(current_node);

                // Find and select the node entity
                for (entity, node_id) in nodes.iter() {
                    if *node_id == current_node {
                        select_node_events.write(NodeSelected {
                            entity,
                            node: *node_id,
                            add_to_selection: true,
                        });
                        break;
                    }
                }

                // Find connected nodes through edges
                if depth < event.depth {
                    for edge in edges.iter() {
                        if edge.relationship.source == current_node {
                            to_visit.push((edge.relationship.target, depth + 1));
                        } else if edge.relationship.target == current_node {
                            to_visit.push((edge.relationship.source, depth + 1));
                        }
                    }
                }
            }

            info!(
                "Selected nodes connected to {:?} within {} hops",
                event.from_node, event.depth
            );
        }
    }
}
