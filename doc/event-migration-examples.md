# Event Migration Examples

This document provides concrete examples of migrating from direct state mutations to event-driven patterns in the Alchemist Graph Editor.

## Table of Contents
1. [Component Updates](#component-updates)
2. [Entity Spawning](#entity-spawning)
3. [Entity Deletion](#entity-deletion)
4. [Batch Operations](#batch-operations)
5. [Resource Updates](#resource-updates)
6. [Complex State Changes](#complex-state-changes)

## Component Updates

### Example 1: Updating Node Position

**Before (Direct Mutation):**
```rust
fn drag_node(
    mouse: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut nodes: Query<(&mut Transform, &Selected)>,
) {
    if mouse.pressed(MouseButton::Left) {
        let window = windows.single();
        let (camera, camera_transform) = camera.single();

        if let Some(world_pos) = window.cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate().extend(0.0))
        {
            for (mut transform, _) in nodes.iter_mut() {
                transform.translation = world_pos;
            }
        }
    }
}
```

**After (Event-Driven):**
```rust
// Event definition
#[derive(Event)]
pub struct DragNodeEvent {
    pub entity: Entity,
    pub start_pos: Vec3,
    pub current_pos: Vec3,
}

// Input detection system
fn detect_node_drag(
    mouse: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    selected_nodes: Query<(Entity, &Transform), With<Selected>>,
    mut drag_events: EventWriter<DragNodeEvent>,
    mut drag_state: Local<Option<(Entity, Vec3)>>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera.single();

    if let Some(world_pos) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate().extend(0.0))
    {
        if mouse.just_pressed(MouseButton::Left) {
            // Start drag
            for (entity, transform) in selected_nodes.iter() {
                *drag_state = Some((entity, transform.translation));
            }
        } else if mouse.pressed(MouseButton::Left) {
            // Continue drag
            if let Some((entity, start_pos)) = *drag_state {
                drag_events.send(DragNodeEvent {
                    entity,
                    start_pos,
                    current_pos: world_pos,
                });
            }
        } else if mouse.just_released(MouseButton::Left) {
            // End drag - send final position as MoveNodeEvent
            if let Some((entity, start_pos)) = drag_state.take() {
                drag_events.send(MoveNodeEvent {
                    entity,
                    from: start_pos,
                    to: world_pos,
                });
            }
        }
    }
}

// Movement application system
fn apply_node_drag(
    mut events: EventReader<DragNodeEvent>,
    mut transforms: Query<&mut Transform>,
) {
    for event in events.read() {
        if let Ok(mut transform) = transforms.get_mut(event.entity) {
            transform.translation = event.current_pos;
        }
    }
}
```

### Example 2: Updating Node Properties

**Before (Direct Mutation):**
```rust
fn update_node_from_inspector(
    mut nodes: Query<(&mut NodeProperties, &Selected)>,
    inspector_state: Res<InspectorState>,
) {
    if inspector_state.is_changed() {
        for (mut properties, _) in nodes.iter_mut() {
            properties.name = inspector_state.name.clone();
            properties.labels = inspector_state.labels.clone();
            properties.custom_properties = inspector_state.properties.clone();
        }
    }
}
```

**After (Event-Driven):**
```rust
// Inspector detection system
fn detect_inspector_changes(
    inspector_state: Res<InspectorState>,
    selected: Query<Entity, With<Selected>>,
    mut update_events: EventWriter<UpdateNodeEvent>,
) {
    if inspector_state.is_changed() {
        for entity in selected.iter() {
            update_events.send(UpdateNodeEvent {
                entity,
                name: Some(inspector_state.name.clone()),
                labels: Some(inspector_state.labels.clone()),
                properties: Some(inspector_state.properties.clone()),
                domain_type: None, // Not changing type
            });
        }
    }
}

// Property update system
fn apply_node_updates(
    mut events: EventReader<UpdateNodeEvent>,
    mut nodes: Query<(&mut NodeProperties, &mut DomainNodeType)>,
    mut modification_events: EventWriter<GraphModificationEvent>,
) {
    for event in events.read() {
        if let Ok((mut properties, mut domain_type)) = nodes.get_mut(event.entity) {
            // Track old values for undo
            let old_name = properties.name.clone();

            // Apply updates
            if let Some(name) = &event.name {
                properties.name = name.clone();
            }
            if let Some(labels) = &event.labels {
                properties.labels = labels.clone();
            }
            if let Some(props) = &event.properties {
                properties.custom_properties = props.clone();
            }
            if let Some(new_type) = &event.domain_type {
                *domain_type = new_type.clone();
            }

            // Send modification event for undo system
            modification_events.send(GraphModificationEvent::NodeUpdated {
                id: properties.id,
                old_name,
                new_name: properties.name.clone(),
            });
        }
    }
}
```

## Entity Spawning

### Example 3: Creating Nodes

**Before (Direct Mutation):**
```rust
fn create_node_on_click(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        let window = windows.single();
        let (camera, camera_transform) = camera.single();

        if let Some(world_pos) = window.cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate().extend(0.0))
        {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgb(0.8, 0.7, 0.6),
                        ..default()
                    }),
                    transform: Transform::from_translation(world_pos),
                    ..default()
                },
                NodeProperties {
                    id: Uuid::new_v4(),
                    name: "New Node".to_string(),
                    labels: vec![],
                    custom_properties: HashMap::new(),
                },
                DomainNodeType::Entity,
            ));
        }
    }
}
```

**After (Event-Driven):**
```rust
// Context menu system
fn show_context_menu(
    mouse: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut context_menu_events: EventWriter<ShowContextMenuEvent>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        let window = windows.single();
        let (camera, camera_transform) = camera.single();

        if let Some(world_pos) = window.cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate().extend(0.0))
        {
            context_menu_events.send(ShowContextMenuEvent {
                position: window.cursor_position().unwrap_or_default(),
                context: ContextMenuContext::Background,
            });
        }
    }
}

// Context menu handler (in UI system)
fn handle_context_menu_selection(
    menu_selection: Res<MenuSelection>,
    create_position: Res<CreatePosition>,
    mut create_events: EventWriter<CreateNodeEvent>,
) {
    if let Some(selection) = &menu_selection.0 {
        match selection.as_str() {
            "Create Entity" => {
                create_events.send(CreateNodeEvent {
                    id: Uuid::new_v4(),
                    position: create_position.0,
                    domain_type: DomainNodeType::Entity,
                    name: "New Entity".to_string(),
                    labels: vec![],
                    properties: HashMap::new(),
                    subgraph_id: None,
                    color: None,
                });
            }
            "Create Event" => {
                create_events.send(CreateNodeEvent {
                    id: Uuid::new_v4(),
                    position: create_position.0,
                    domain_type: DomainNodeType::Event,
                    name: "New Event".to_string(),
                    labels: vec![],
                    properties: HashMap::new(),
                    subgraph_id: None,
                    color: Some("#FFD700".to_string()),
                });
            }
            _ => {}
        }
    }
}

// Node creation system
fn create_nodes(
    mut events: EventReader<CreateNodeEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut uuid_to_entity: ResMut<UuidToEntity>,
) {
    for event in events.read() {
        let color = event.color.as_ref()
            .and_then(|hex| Color::hex(hex).ok())
            .unwrap_or_else(|| match event.domain_type {
                DomainNodeType::Entity => Color::rgb(0.8, 0.7, 0.6),
                DomainNodeType::Event => Color::rgb(1.0, 0.84, 0.0),
                DomainNodeType::Command => Color::rgb(0.6, 0.8, 0.6),
                _ => Color::rgb(0.7, 0.7, 0.7),
            });

        let entity = commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(StandardMaterial {
                    base_color: color,
                    ..default()
                }),
                transform: Transform::from_translation(event.position),
                ..default()
            },
            NodeProperties {
                id: event.id,
                name: event.name.clone(),
                labels: event.labels.clone(),
                custom_properties: event.properties.clone(),
            },
            event.domain_type.clone(),
            NodeId(event.id),
        )).id();

        // Track UUID to Entity mapping
        uuid_to_entity.0.insert(event.id, entity);
    }
}
```

## Entity Deletion

### Example 4: Deleting Selected Nodes

**Before (Direct Mutation):**
```rust
fn delete_selected(
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    selected: Query<Entity, With<Selected>>,
    edges: Query<(Entity, &Edge)>,
) {
    if keyboard.just_pressed(KeyCode::Delete) {
        // Delete nodes
        for entity in selected.iter() {
            commands.entity(entity).despawn_recursive();

            // Also delete connected edges
            for (edge_entity, edge) in edges.iter() {
                if edge.source == entity || edge.target == entity {
                    commands.entity(edge_entity).despawn();
                }
            }
        }
    }
}
```

**After (Event-Driven):**
```rust
// Deletion request system
fn request_deletion(
    keyboard: Res<Input<KeyCode>>,
    selected: Query<Entity, With<Selected>>,
    mut delete_events: EventWriter<DeleteNodeEvent>,
) {
    if keyboard.just_pressed(KeyCode::Delete) {
        for entity in selected.iter() {
            delete_events.send(DeleteNodeEvent { entity });
        }
    }
}

// Node deletion system
fn delete_nodes(
    mut events: EventReader<DeleteNodeEvent>,
    mut commands: Commands,
    nodes: Query<&NodeId>,
    edges: Query<(Entity, &Edge)>,
    mut delete_edge_events: EventWriter<DeleteEdgeEvent>,
    mut modification_events: EventWriter<GraphModificationEvent>,
) {
    for event in events.read() {
        if let Ok(node_id) = nodes.get(event.entity) {
            // Record for undo
            modification_events.send(GraphModificationEvent::NodeDeleted {
                id: node_id.0,
            });

            // Find and delete connected edges
            for (edge_entity, edge) in edges.iter() {
                if edge.source == event.entity || edge.target == event.entity {
                    delete_edge_events.send(DeleteEdgeEvent {
                        source: edge.source,
                        edge_id: edge.id,
                    });
                }
            }

            // Delete the node
            commands.entity(event.entity).despawn_recursive();
        }
    }
}

// Edge deletion system
fn delete_edges(
    mut events: EventReader<DeleteEdgeEvent>,
    mut commands: Commands,
    edges: Query<(Entity, &Edge)>,
) {
    for event in events.read() {
        for (entity, edge) in edges.iter() {
            if edge.id == event.edge_id {
                commands.entity(entity).despawn_recursive();
                break;
            }
        }
    }
}
```

## Batch Operations

### Example 5: Aligning Nodes

**Before (Direct Mutation):**
```rust
fn align_nodes_horizontally(
    keyboard: Res<Input<KeyCode>>,
    mut nodes: Query<(&mut Transform, &Selected)>,
) {
    if keyboard.just_pressed(KeyCode::H) && keyboard.pressed(KeyCode::ControlLeft) {
        let positions: Vec<Vec3> = nodes.iter().map(|(t, _)| t.translation).collect();
        if !positions.is_empty() {
            let avg_y = positions.iter().map(|p| p.y).sum::<f32>() / positions.len() as f32;

            for (mut transform, _) in nodes.iter_mut() {
                transform.translation.y = avg_y;
            }
        }
    }
}
```

**After (Event-Driven):**
```rust
// Alignment request system
fn request_alignment(
    keyboard: Res<Input<KeyCode>>,
    selected: Query<(Entity, &Transform), With<Selected>>,
    mut batch_events: EventWriter<BatchMoveNodesEvent>,
) {
    if keyboard.just_pressed(KeyCode::H) && keyboard.pressed(KeyCode::ControlLeft) {
        let nodes: Vec<(Entity, Vec3)> = selected.iter()
            .map(|(e, t)| (e, t.translation))
            .collect();

        if !nodes.is_empty() {
            let avg_y = nodes.iter().map(|(_, p)| p.y).sum::<f32>() / nodes.len() as f32;

            let moves: Vec<(Entity, Vec3, Vec3)> = nodes.into_iter()
                .map(|(entity, pos)| {
                    let new_pos = Vec3::new(pos.x, avg_y, pos.z);
                    (entity, pos, new_pos)
                })
                .collect();

            batch_events.send(BatchMoveNodesEvent { moves });
        }
    }
}

// Batch movement system
fn apply_batch_moves(
    mut events: EventReader<BatchMoveNodesEvent>,
    mut transforms: Query<&mut Transform>,
    mut move_events: EventWriter<MoveNodeEvent>,
) {
    for event in events.read() {
        for (entity, from, to) in &event.moves {
            // Update transform
            if let Ok(mut transform) = transforms.get_mut(*entity) {
                transform.translation = *to;
            }

            // Send individual move events for undo tracking
            move_events.send(MoveNodeEvent {
                entity: *entity,
                from: *from,
                to: *to,
            });
        }
    }
}
```

## Resource Updates

### Example 6: Updating Global State

**Before (Direct Mutation):**
```rust
fn toggle_grid_visibility(
    keyboard: Res<Input<KeyCode>>,
    mut grid_settings: ResMut<GridSettings>,
    mut grid_query: Query<&mut Visibility, With<GridMarker>>,
) {
    if keyboard.just_pressed(KeyCode::G) {
        grid_settings.visible = !grid_settings.visible;

        for mut visibility in grid_query.iter_mut() {
            *visibility = if grid_settings.visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}
```

**After (Event-Driven):**
```rust
// Event definition
#[derive(Event)]
pub struct ToggleGridEvent;

#[derive(Event)]
pub struct GridSettingsChangedEvent {
    pub visible: bool,
    pub snap_enabled: bool,
    pub grid_size: f32,
}

// Input detection
fn detect_grid_toggle(
    keyboard: Res<Input<KeyCode>>,
    mut toggle_events: EventWriter<ToggleGridEvent>,
) {
    if keyboard.just_pressed(KeyCode::G) {
        toggle_events.send(ToggleGridEvent);
    }
}

// Settings update system
fn handle_grid_toggle(
    mut events: EventReader<ToggleGridEvent>,
    mut grid_settings: ResMut<GridSettings>,
    mut settings_changed: EventWriter<GridSettingsChangedEvent>,
) {
    for _ in events.read() {
        grid_settings.visible = !grid_settings.visible;

        settings_changed.send(GridSettingsChangedEvent {
            visible: grid_settings.visible,
            snap_enabled: grid_settings.snap_enabled,
            grid_size: grid_settings.grid_size,
        });
    }
}

// Visual update system
fn update_grid_visibility(
    mut events: EventReader<GridSettingsChangedEvent>,
    mut grid_query: Query<&mut Visibility, With<GridMarker>>,
) {
    for event in events.read() {
        let visibility = if event.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        for mut vis in grid_query.iter_mut() {
            *vis = visibility;
        }
    }
}
```

## Complex State Changes

### Example 7: Graph Layout Application

**Before (Direct Mutation):**
```rust
fn apply_force_directed_layout(
    mut nodes: Query<(&mut Transform, &NodeId, &NodeConnections)>,
    time: Res<Time>,
    layout_settings: Res<LayoutSettings>,
) {
    if !layout_settings.active {
        return;
    }

    let delta = time.delta_seconds();

    // Calculate forces
    let mut forces: HashMap<Entity, Vec3> = HashMap::new();

    // ... complex force calculations ...

    // Apply forces directly
    for (mut transform, _, _) in nodes.iter_mut() {
        if let Some(force) = forces.get(&entity) {
            transform.translation += *force * delta;
        }
    }
}
```

**After (Event-Driven):**
```rust
// Layout calculation system
fn calculate_layout(
    mut events: EventReader<RequestLayoutEvent>,
    nodes: Query<(Entity, &Transform, &NodeId, &NodeConnections)>,
    mut layout_complete: EventWriter<LayoutCompleteEvent>,
    mut batch_move: EventWriter<BatchMoveNodesEvent>,
) {
    for event in events.read() {
        match event.layout_type {
            LayoutType::ForceDirected => {
                let positions = calculate_force_directed_layout(&nodes);

                let moves: Vec<(Entity, Vec3, Vec3)> = positions.into_iter()
                    .filter_map(|(entity, new_pos)| {
                        nodes.get(entity).ok().map(|(_, transform, _, _)| {
                            (entity, transform.translation, new_pos)
                        })
                    })
                    .collect();

                if !moves.is_empty() {
                    batch_move.send(BatchMoveNodesEvent { moves });
                }

                layout_complete.send(LayoutCompleteEvent {
                    layout_type: event.layout_type.clone(),
                    success: true,
                });
            }
            // ... other layout types
        }
    }
}

// Incremental layout system (for animations)
fn animate_layout_changes(
    mut active_animations: ResMut<LayoutAnimations>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
    mut animation_complete: EventWriter<LayoutAnimationCompleteEvent>,
) {
    let delta = time.delta_seconds();

    active_animations.0.retain_mut(|animation| {
        animation.progress += delta / animation.duration;

        if animation.progress >= 1.0 {
            // Final position
            if let Ok(mut transform) = transforms.get_mut(animation.entity) {
                transform.translation = animation.target;
            }

            animation_complete.send(LayoutAnimationCompleteEvent {
                entity: animation.entity,
            });

            false // Remove from active animations
        } else {
            // Interpolate
            if let Ok(mut transform) = transforms.get_mut(animation.entity) {
                transform.translation = animation.start.lerp(
                    animation.target,
                    ease_in_out(animation.progress)
                );
            }
            true // Keep animating
        }
    });
}
```

## Summary

The key principles when migrating to event-driven patterns:

1. **Separate Detection from Action**: Input/state detection systems send events, separate systems handle the actions
2. **Make State Changes Traceable**: Every state change should flow through an event
3. **Enable Composition**: Multiple systems can react to the same event
4. **Support Undo/Redo**: Events can be recorded and reversed
5. **Improve Testability**: Systems can be tested in isolation by sending/receiving events

Remember:
- Events should be immutable
- Include all necessary data in events
- Use SystemSets to control execution order
- Document which systems produce and consume each event
- Consider performance for high-frequency events
