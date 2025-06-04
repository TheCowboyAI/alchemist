# DDD Graph Rendering and Manipulation in Bevy

## Overview

This document describes how we render and manipulate DDD elements (Domains, Aggregates, Entities, Events) and their event streams as interactive graphs in Bevy, enabling visual construction, composition, and algorithm visualization.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    DDD Domain Layer                          │
│  ┌──────────┐  ┌──────────────┐  ┌──────────┐              │
│  │ Aggregates│  │   Entities   │  │  Events  │              │
│  └──────────┘  └──────────────┘  └──────────┘              │
└─────────────────────────────────────────────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    │   Graph Models    │
                    │ (Daggy/Petgraph) │
                    └─────────┬─────────┘
                              │
┌─────────────────────────────┴─────────────────────────────┐
│                 Bevy Rendering Layer                        │
│  ┌──────────┐  ┌──────────────┐  ┌──────────┐            │
│  │  Meshes  │  │  Materials   │  │ UI/Gizmos│            │
│  └──────────┘  └──────────────┘  └──────────┘            │
└────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Graph Visualization Components

```rust
use bevy::prelude::*;
use petgraph::graph::NodeIndex;

/// Component for rendered DDD elements
#[derive(Component)]
pub struct DddElement {
    pub element_type: DddElementType,
    pub graph_index: GraphIndex,
    pub metadata: DddMetadata,
}

#[derive(Debug, Clone)]
pub enum DddElementType {
    Domain { name: String, bounded_context: String },
    Aggregate { root_id: AggregateId, domain: String },
    Entity { id: EntityId, aggregate: AggregateId },
    Event { cid: Cid, event_type: String },
    Command { name: String, target: AggregateId },
    Policy { name: String, triggers: Vec<String> },
}

#[derive(Debug, Clone)]
pub enum GraphIndex {
    Petgraph(NodeIndex),
    Daggy(DagNodeIndex),
}

/// Visual representation of graph nodes
#[derive(Component)]
pub struct GraphNodeVisual {
    pub base_color: Color,
    pub highlight_color: Color,
    pub size: f32,
    pub shape: NodeShape,
}

/// Visual representation of graph edges
#[derive(Component)]
pub struct GraphEdgeVisual {
    pub from: Entity,
    pub to: Entity,
    pub edge_type: EdgeVisualizationType,
    pub animation_state: EdgeAnimationState,
}

#[derive(Debug, Clone)]
pub enum EdgeVisualizationType {
    EventFlow { event_type: String },
    CommandFlow { command: String },
    AggregateRelation { relation_type: String },
    DomainBoundary { crossing_type: CrossingType },
}
```

### 2. Graph Layout System

```rust
/// System for laying out DDD graphs using force-directed algorithms
pub fn layout_ddd_graph_system(
    mut query: Query<(&DddElement, &mut Transform), Without<GraphEdgeVisual>>,
    graph_model: Res<UnifiedGraphService>,
    time: Res<Time>,
) {
    // Use petgraph's layout algorithms
    let layout = match graph_model.get_active_layout() {
        LayoutAlgorithm::ForceDirected => {
            calculate_force_directed_layout(&graph_model.workflows)
        }
        LayoutAlgorithm::Hierarchical => {
            calculate_hierarchical_layout(&graph_model.workflows)
        }
        LayoutAlgorithm::Circular => {
            calculate_circular_layout(&graph_model.workflows)
        }
    };

    // Apply layout to entities
    for (element, mut transform) in query.iter_mut() {
        if let Some(position) = layout.get_position(&element.graph_index) {
            // Smooth animation to new position
            let target = Vec3::new(position.x, position.y, position.z);
            transform.translation = transform.translation.lerp(target, time.delta_seconds() * 2.0);
        }
    }
}

/// System for rendering event streams as animated flows
pub fn render_event_stream_system(
    mut commands: Commands,
    event_store: Res<EventStore>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &DddElement), With<EventProducer>>,
) {
    for (entity, element) in query.iter() {
        // Get recent events from this element
        if let Ok(events) = event_store.get_recent_events(&element.get_aggregate_id()) {
            for event in events {
                // Create visual representation of event flow
                spawn_event_particle(&mut commands, &mut meshes, &mut materials, entity, &event);
            }
        }
    }
}
```

### 3. Interactive Graph Manipulation

```rust
/// Component for interactive graph nodes
#[derive(Component)]
pub struct InteractiveNode {
    pub draggable: bool,
    pub connectable: bool,
    pub deletable: bool,
    pub menu_items: Vec<MenuItem>,
}

/// System for handling graph node interactions
pub fn graph_interaction_system(
    mut commands: Commands,
    mouse_button: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut node_query: Query<(Entity, &mut Transform, &InteractiveNode, &DddElement)>,
    mut graph_service: ResMut<UnifiedGraphService>,
    mut events: EventWriter<GraphMutationEvent>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    if let Some(cursor_position) = window.cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            // Handle node selection
            if mouse_button.just_pressed(MouseButton::Left) {
                for (entity, transform, interactive, element) in node_query.iter() {
                    if ray_intersects_node(ray, transform.translation, element.get_size()) {
                        commands.entity(entity).insert(Selected);

                        // Emit selection event
                        events.send(GraphMutationEvent::NodeSelected {
                            entity,
                            element_type: element.element_type.clone(),
                        });
                    }
                }
            }

            // Handle node connection
            if mouse_button.pressed(MouseButton::Right) {
                handle_node_connection(&mut commands, &mut graph_service, &node_query, ray);
            }
        }
    }
}

/// System for creating new DDD elements
pub fn ddd_construction_system(
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    mut graph_service: ResMut<UnifiedGraphService>,
    selected_query: Query<(Entity, &DddElement), With<Selected>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create new aggregate
    if keyboard.just_pressed(KeyCode::A) && keyboard.pressed(KeyCode::LControl) {
        if let Ok((parent_entity, parent_element)) = selected_query.get_single() {
            if let DddElementType::Domain { name, .. } = &parent_element.element_type {
                // Create new aggregate in selected domain
                let aggregate_id = create_aggregate(&mut graph_service, name);
                spawn_aggregate_visual(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    aggregate_id,
                    parent_entity,
                );
            }
        }
    }

    // Create new entity
    if keyboard.just_pressed(KeyCode::E) && keyboard.pressed(KeyCode::LControl) {
        if let Ok((parent_entity, parent_element)) = selected_query.get_single() {
            if let DddElementType::Aggregate { root_id, .. } = &parent_element.element_type {
                // Create new entity in selected aggregate
                let entity_id = create_entity(&mut graph_service, root_id);
                spawn_entity_visual(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    entity_id,
                    parent_entity,
                );
            }
        }
    }
}
```

### 4. Algorithm Visualization

```rust
/// Component for algorithm visualization
#[derive(Component)]
pub struct AlgorithmVisualization {
    pub algorithm_type: GraphAlgorithm,
    pub state: AlgorithmState,
    pub highlighted_nodes: Vec<Entity>,
    pub highlighted_edges: Vec<Entity>,
}

#[derive(Debug, Clone)]
pub enum GraphAlgorithm {
    ShortestPath { from: NodeIndex, to: NodeIndex },
    CommunityDetection,
    CycleDetection,
    TopologicalSort,
    MaxFlow { source: NodeIndex, sink: NodeIndex },
    EventPropagation { start_event: Cid },
}

/// System for visualizing graph algorithms
pub fn visualize_algorithm_system(
    mut query: Query<(&mut GraphNodeVisual, &DddElement)>,
    mut edge_query: Query<&mut GraphEdgeVisual>,
    algorithm: Res<AlgorithmVisualization>,
    time: Res<Time>,
) {
    // Highlight nodes in current algorithm step
    for (mut visual, element) in query.iter_mut() {
        if algorithm.highlighted_nodes.contains(&element.entity) {
            // Pulse effect for active nodes
            let pulse = (time.elapsed_seconds() * 3.0).sin() * 0.5 + 0.5;
            visual.base_color = visual.highlight_color.with_a(pulse);
        } else {
            // Dim non-active nodes
            visual.base_color = visual.base_color.with_a(0.3);
        }
    }

    // Animate edges in algorithm path
    for mut edge_visual in edge_query.iter_mut() {
        if algorithm.highlighted_edges.contains(&edge_visual.entity) {
            edge_visual.animation_state = EdgeAnimationState::Flowing {
                speed: 2.0,
                color: Color::CYAN,
            };
        }
    }
}

/// System for running graph algorithms on DDD structures
pub fn run_graph_algorithm_system(
    mut algorithm: ResMut<AlgorithmVisualization>,
    graph_service: Res<UnifiedGraphService>,
    selected_nodes: Query<&DddElement, With<Selected>>,
    keyboard: Res<Input<KeyCode>>,
) {
    // Shortest path between aggregates
    if keyboard.just_pressed(KeyCode::P) && selected_nodes.iter().count() == 2 {
        let nodes: Vec<_> = selected_nodes.iter().collect();
        if let (Some(from), Some(to)) = (nodes.get(0), nodes.get(1)) {
            let path = graph_service.find_shortest_path(
                from.graph_index,
                to.graph_index,
            );

            algorithm.algorithm_type = GraphAlgorithm::ShortestPath {
                from: from.graph_index,
                to: to.graph_index,
            };
            algorithm.highlighted_nodes = path.nodes;
            algorithm.highlighted_edges = path.edges;
        }
    }

    // Event propagation visualization
    if keyboard.just_pressed(KeyCode::E) && keyboard.pressed(KeyCode::LShift) {
        if let Ok(element) = selected_nodes.get_single() {
            if let DddElementType::Event { cid, .. } = &element.element_type {
                let propagation = graph_service.trace_event_propagation(cid);

                algorithm.algorithm_type = GraphAlgorithm::EventPropagation {
                    start_event: cid.clone(),
                };
                algorithm.highlighted_nodes = propagation.affected_nodes;
                algorithm.highlighted_edges = propagation.event_paths;
            }
        }
    }
}
```

### 5. Event Stream Visualization

```rust
/// Component for event stream visualization
#[derive(Component)]
pub struct EventStreamVisual {
    pub stream_id: StreamId,
    pub event_particles: Vec<EventParticle>,
    pub flow_rate: f32,
}

#[derive(Debug, Clone)]
pub struct EventParticle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub event_type: String,
    pub color: Color,
    pub lifetime: f32,
}

/// System for animating event streams between DDD elements
pub fn animate_event_streams_system(
    mut query: Query<(&mut EventStreamVisual, &GraphEdgeVisual)>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    for (mut stream, edge) in query.iter_mut() {
        // Update particle positions
        for particle in &mut stream.event_particles {
            particle.position += particle.velocity * time.delta_seconds();
            particle.lifetime -= time.delta_seconds();

            // Draw particle
            gizmos.sphere(
                particle.position,
                Quat::IDENTITY,
                0.1,
                particle.color,
            );
        }

        // Remove expired particles
        stream.event_particles.retain(|p| p.lifetime > 0.0);

        // Spawn new particles based on flow rate
        if stream.flow_rate > 0.0 {
            let spawn_chance = stream.flow_rate * time.delta_seconds();
            if rand::random::<f32>() < spawn_chance {
                stream.event_particles.push(create_event_particle(&edge));
            }
        }
    }
}

/// System for visualizing MerkleDAG structure
pub fn visualize_merkle_dag_system(
    event_store: Res<EventStore>,
    mut gizmos: Gizmos,
    query: Query<(&Transform, &DddElement)>,
) {
    // Draw MerkleDAG connections
    for (transform, element) in query.iter() {
        if let DddElementType::Event { cid, .. } = &element.element_type {
            // Get parent events in MerkleDAG
            if let Ok(parents) = event_store.get_parents(cid) {
                for parent_cid in parents {
                    // Find visual entity for parent
                    if let Some(parent_entity) = find_event_entity(&query, &parent_cid) {
                        let parent_pos = parent_entity.translation;

                        // Draw DAG edge
                        gizmos.line(
                            transform.translation,
                            parent_pos,
                            Color::rgba(0.5, 0.5, 1.0, 0.5),
                        );

                        // Draw arrow
                        draw_arrow(&mut gizmos, parent_pos, transform.translation);
                    }
                }
            }
        }
    }
}
```

### 6. UI Integration

```rust
/// UI for graph manipulation and algorithm control
pub fn graph_ui_system(
    mut contexts: EguiContexts,
    mut graph_service: ResMut<UnifiedGraphService>,
    mut algorithm_viz: ResMut<AlgorithmVisualization>,
    selected: Query<&DddElement, With<Selected>>,
) {
    egui::Window::new("DDD Graph Editor")
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Graph Operations");

            // Domain operations
            if ui.button("Create Domain").clicked() {
                let domain_id = graph_service.create_domain("New Domain");
                // Spawn visual representation
            }

            // Algorithm selection
            ui.separator();
            ui.heading("Graph Algorithms");

            ui.horizontal(|ui| {
                if ui.button("Shortest Path").clicked() {
                    // Enable shortest path mode
                }
                if ui.button("Event Flow").clicked() {
                    // Show event propagation
                }
                if ui.button("Cycle Detection").clicked() {
                    // Run cycle detection
                }
            });

            // Selected element properties
            if let Ok(element) = selected.get_single() {
                ui.separator();
                ui.heading("Selected Element");

                match &element.element_type {
                    DddElementType::Aggregate { root_id, domain } => {
                        ui.label(format!("Aggregate: {}", root_id));
                        ui.label(format!("Domain: {}", domain));

                        if ui.button("Add Entity").clicked() {
                            // Create entity in this aggregate
                        }
                    }
                    DddElementType::Event { cid, event_type } => {
                        ui.label(format!("Event: {}", event_type));
                        ui.label(format!("CID: {}", cid));

                        if ui.button("Trace Propagation").clicked() {
                            // Visualize event propagation
                        }
                    }
                    _ => {}
                }
            }

            // Layout options
            ui.separator();
            ui.heading("Layout");

            ui.radio_value(&mut graph_service.layout_algorithm, LayoutAlgorithm::ForceDirected, "Force Directed");
            ui.radio_value(&mut graph_service.layout_algorithm, LayoutAlgorithm::Hierarchical, "Hierarchical");
            ui.radio_value(&mut graph_service.layout_algorithm, LayoutAlgorithm::Circular, "Circular");
        });
}
```

## Usage Examples

### 1. Visualizing Domain Boundaries

```rust
pub fn setup_domain_visualization(
    mut commands: Commands,
    mut graph_service: ResMut<UnifiedGraphService>,
) {
    // Create domains
    let order_domain = graph_service.create_domain("Order Management");
    let inventory_domain = graph_service.create_domain("Inventory");
    let payment_domain = graph_service.create_domain("Payment Processing");

    // Create visual boundaries
    spawn_domain_boundary(&mut commands, order_domain, Color::BLUE);
    spawn_domain_boundary(&mut commands, inventory_domain, Color::GREEN);
    spawn_domain_boundary(&mut commands, payment_domain, Color::YELLOW);

    // Show cross-domain event flows
    visualize_domain_events(&mut commands, &graph_service);
}
```

### 2. Algorithm Execution Visualization

```rust
pub fn visualize_event_propagation(
    mut commands: Commands,
    event_store: Res<EventStore>,
    graph_service: Res<UnifiedGraphService>,
) {
    // Get a specific event
    let event_cid = Cid::from_str("QmXyz...").unwrap();

    // Trace its propagation through the system
    let propagation = graph_service.trace_event_propagation(&event_cid);

    // Animate the propagation
    for (step, affected_node) in propagation.steps.iter().enumerate() {
        commands.spawn(PropagationAnimation {
            node: affected_node.clone(),
            delay: Duration::from_millis(step as u64 * 200),
            effect: PropagationEffect::Ripple,
        });
    }
}
```

## Benefits

1. **Visual Understanding**: See DDD structures and relationships
2. **Interactive Design**: Build and modify domains visually
3. **Algorithm Insights**: Watch algorithms execute on your domain
4. **Event Flow Tracking**: See how events propagate through the system
5. **Performance Analysis**: Identify bottlenecks and cycles
6. **Educational Tool**: Learn DDD concepts through visualization

This architecture enables powerful visualization and manipulation of DDD concepts while maintaining the integrity of the underlying domain model.
