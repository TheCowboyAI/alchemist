//! Systems for node and edge selection
//!
//! These systems handle:
//! - Single and multi-selection
//! - Hover state management
//! - Selection visualization
//! - Keyboard selection shortcuts

use bevy::prelude::*;

use crate::{
    components::*,
    events::*,
    resources::*,
};

/// System that handles node selection from mouse clicks
///
/// This system:
/// 1. Detects mouse clicks on nodes
/// 2. Manages single vs multi-selection
/// 3. Updates selection state
/// 4. Sends status updates
pub fn handle_mouse_selection(
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GraphCamera>>,
    nodes: Query<(Entity, &Transform, &NodeId), With<NodeInteractable>>,
    mut select_events: EventWriter<SelectEvent>,
    mut deselect_events: EventWriter<DeselectAllEvent>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    if let Some(cursor_pos) = window.cursor_position() {
        // Convert screen position to world ray
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            let mut closest_node = None;
            let mut closest_distance = f32::MAX;

            // Find the closest node to the ray
            for (entity, transform, _) in nodes.iter() {
                // Simple sphere intersection test
                let node_pos = transform.translation;
                let to_node = node_pos - ray.origin;
                let projection = to_node.dot(ray.direction);

                if projection > 0.0 {
                    let closest_point = ray.origin + ray.direction * projection;
                    let distance = (closest_point - node_pos).length();

                    // Check if within node bounds (assuming 1.0 unit radius)
                    if distance < 1.0 && projection < closest_distance {
                        closest_node = Some(entity);
                        closest_distance = projection;
                    }
                }
            }

            // Handle selection
            let multi_select = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ShiftLeft);

            if let Some(entity) = closest_node {
                select_events.send(SelectEvent {
                    entity,
                    multi_select,
                });
            } else if !multi_select {
                // Clicked on empty space - deselect all
                deselect_events.send(DeselectAllEvent);
            }
        }
    }
}

/// System that processes selection events
///
/// This system:
/// 1. Adds/removes Selected components
/// 2. Updates visual feedback
/// 3. Sends status bar updates
pub fn handle_selection_events(
    mut commands: Commands,
    mut select_events: EventReader<SelectEvent>,
    mut deselect_events: EventReader<DeselectAllEvent>,
    selected: Query<Entity, With<Selected>>,
    nodes: Query<&NodeProperties>,
    mut status_events: EventWriter<UpdateStatusBarEvent>,
) {
    // Handle deselect all
    for _ in deselect_events.read() {
        for entity in selected.iter() {
            commands.entity(entity).remove::<Selected>();
        }

        status_events.send(UpdateStatusBarEvent {
            section: StatusBarSection::Selection,
            text: "No selection".to_string(),
        });
    }

    // Handle selection
    for event in select_events.read() {
        if !event.multi_select {
            // Single selection - deselect others first
            for entity in selected.iter() {
                if entity != event.entity {
                    commands.entity(entity).remove::<Selected>();
                }
            }
        }

        // Toggle selection on the target entity
        if selected.get(event.entity).is_ok() {
            // Already selected - deselect
            commands.entity(event.entity).remove::<Selected>();
        } else {
            // Not selected - select
            commands.entity(event.entity).insert(Selected);
        }

        // Update status bar
        let selected_count = selected.iter().count();
        let status_text = if selected_count == 0 {
            "No selection".to_string()
        } else if selected_count == 1 {
            if let Ok(props) = nodes.get(event.entity) {
                format!("Selected: {}", props.name)
            } else {
                "1 node selected".to_string()
            }
        } else {
            format!("{} nodes selected", selected_count)
        };

        status_events.send(UpdateStatusBarEvent {
            section: StatusBarSection::Selection,
            text: status_text,
        });
    }
}

/// System that handles hover state
///
/// This system updates hover state based on mouse position
pub fn handle_hover_state(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GraphCamera>>,
    nodes: Query<(Entity, &Transform), With<NodeInteractable>>,
    hovered: Query<Entity, With<Hovered>>,
    mut hover_events: EventWriter<HoverEvent>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    // Clear previous hover state
    for entity in hovered.iter() {
        commands.entity(entity).remove::<Hovered>();
    }

    if let Some(cursor_pos) = window.cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            let mut closest_node = None;
            let mut closest_distance = f32::MAX;

            // Find the closest node to the ray
            for (entity, transform) in nodes.iter() {
                let node_pos = transform.translation;
                let to_node = node_pos - ray.origin;
                let projection = to_node.dot(ray.direction);

                if projection > 0.0 {
                    let closest_point = ray.origin + ray.direction * projection;
                    let distance = (closest_point - node_pos).length();

                    if distance < 1.2 && projection < closest_distance { // Slightly larger than selection
                        closest_node = Some(entity);
                        closest_distance = projection;
                    }
                }
            }

            if let Some(entity) = closest_node {
                commands.entity(entity).insert(Hovered);
                hover_events.send(HoverEvent {
                    entity: Some(entity),
                });
            } else {
                hover_events.send(HoverEvent { entity: None });
            }
        }
    }
}

/// System that handles box selection
///
/// This system allows selecting multiple nodes by dragging a box
pub fn handle_box_selection(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GraphCamera>>,
    nodes: Query<(Entity, &Transform), With<NodeInteractable>>,
    mut box_selection: ResMut<BoxSelection>,
    mut gizmos: Gizmos,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    if let Some(cursor_pos) = window.cursor_position() {
        if mouse.just_pressed(MouseButton::Left) && keyboard.pressed(KeyCode::ShiftLeft) {
            // Start box selection
            box_selection.start = Some(cursor_pos);
            box_selection.current = cursor_pos;
            box_selection.active = true;
        } else if mouse.pressed(MouseButton::Left) && box_selection.active {
            // Update box selection
            box_selection.current = cursor_pos;
        } else if mouse.just_released(MouseButton::Left) && box_selection.active {
            // Complete box selection
            if let Some(start) = box_selection.start {
                let min_x = start.x.min(cursor_pos.x);
                let max_x = start.x.max(cursor_pos.x);
                let min_y = start.y.min(cursor_pos.y);
                let max_y = start.y.max(cursor_pos.y);

                // Select nodes within box
                for (entity, transform) in nodes.iter() {
                    // Project node position to screen
                    if let Some(screen_pos) = camera.world_to_viewport(camera_transform, transform.translation) {
                        if screen_pos.x >= min_x && screen_pos.x <= max_x &&
                           screen_pos.y >= min_y && screen_pos.y <= max_y {
                            commands.entity(entity).insert(Selected);
                        }
                    }
                }
            }

            // Reset box selection
            box_selection.active = false;
            box_selection.start = None;
        }

        // Draw box selection
        if box_selection.active {
            if let Some(start) = box_selection.start {
                // Convert screen coordinates to world coordinates for visualization
                if let (Some(start_ray), Some(end_ray)) = (
                    camera.viewport_to_world(camera_transform, start),
                    camera.viewport_to_world(camera_transform, cursor_pos)
                ) {
                    // Draw selection box (simplified - in practice would need proper screen-space rendering)
                    gizmos.line(start_ray.origin, end_ray.origin, Color::rgba(0.5, 0.5, 1.0, 0.5));
                }
            }
        }
    }
}

/// System that handles keyboard selection shortcuts
///
/// This system provides keyboard shortcuts for selection operations
pub fn handle_selection_shortcuts(
    keyboard: Res<Input<KeyCode>>,
    nodes: Query<Entity, With<NodeId>>,
    selected: Query<Entity, With<Selected>>,
    mut commands: Commands,
    mut select_events: EventWriter<SelectEvent>,
    mut deselect_events: EventWriter<DeselectAllEvent>,
) {
    // Ctrl+A - Select all
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::A) {
        for entity in nodes.iter() {
            commands.entity(entity).insert(Selected);
        }
        info!("Selected all {} nodes", nodes.iter().count());
    }

    // Escape - Deselect all
    if keyboard.just_pressed(KeyCode::Escape) {
        deselect_events.send(DeselectAllEvent);
    }

    // Ctrl+I - Invert selection
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::I) {
        for entity in nodes.iter() {
            if selected.get(entity).is_ok() {
                commands.entity(entity).remove::<Selected>();
            } else {
                commands.entity(entity).insert(Selected);
            }
        }
        info!("Inverted selection");
    }
}

/// System that updates visual feedback for selection
///
/// This system updates materials and other visual properties based on selection state
pub fn update_selection_visuals(
    mut materials: ResMut<Assets<StandardMaterial>>,
    selected_nodes: Query<&Handle<StandardMaterial>, (With<Selected>, Changed<Selected>)>,
    deselected_nodes: Query<&Handle<StandardMaterial>, (Without<Selected>, Changed<Selected>)>,
    hovered_nodes: Query<&Handle<StandardMaterial>, With<Hovered>>,
) {
    // Update selected nodes
    for material_handle in selected_nodes.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            material.emissive = Color::rgb(0.3, 0.5, 1.0);
        }
    }

    // Update deselected nodes
    for material_handle in deselected_nodes.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            material.emissive = Color::BLACK;
        }
    }

    // Update hovered nodes
    for material_handle in hovered_nodes.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            // Only update if not selected
            if material.emissive == Color::BLACK {
                material.emissive = Color::rgb(0.1, 0.1, 0.3);
            }
        }
    }
}

/// System that handles focus on selection
///
/// This system triggers camera focus when nodes are selected
pub fn focus_on_selection(
    keyboard: Res<Input<KeyCode>>,
    selected: Query<&Transform, With<Selected>>,
    mut focus_events: EventWriter<FocusSelectionEvent>,
) {
    // F key - Focus on selection
    if keyboard.just_pressed(KeyCode::F) && !selected.is_empty() {
        focus_events.send(FocusSelectionEvent {
            instant: keyboard.pressed(KeyCode::ShiftLeft),
        });
    }
}
