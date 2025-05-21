use bevy::prelude::*;
use crate::graph::{AlchemistGraph, GraphWorkflow};
use crate::graph_patterns::{GraphPattern, PatternCatalog, generate_pattern};
use uuid::Uuid;
use std::collections::HashMap;
use rand::Rng;

// Plugin for the 3D graph visualization
pub struct Graph3DPlugin;

impl Plugin for Graph3DPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<Graph3DState>()
            .add_event::<CreateGraphPatternEvent>()
            .add_systems(Startup, setup_3d_environment)
            .add_systems(Update, (
                handle_pattern_creation,
                update_node_visuals,
                update_edge_visuals,
            ));
    }
}

// Resources
#[derive(Resource)]
pub struct Graph3DState {
    pub graph: AlchemistGraph,
    pub node_entities: HashMap<Uuid, Entity>,
    pub edge_entities: HashMap<Uuid, Entity>,
    pub selected_node: Option<Uuid>,
    pub pattern_catalog: PatternCatalog,
}

impl Default for Graph3DState {
    fn default() -> Self {
        Self {
            graph: AlchemistGraph::new(),
            node_entities: HashMap::new(),
            edge_entities: HashMap::new(),
            selected_node: None,
            pattern_catalog: PatternCatalog::new(),
        }
    }
}

// Events
#[derive(Event)]
pub struct CreateGraphPatternEvent {
    pub pattern: GraphPattern,
}

// Components
#[derive(Component)]
pub struct Node3D {
    pub id: Uuid,
}

#[derive(Component)]
pub struct Edge3D {
    pub source: Uuid,
    pub target: Uuid,
}

// Systems
fn setup_3d_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    
    // Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    
    // Grid for reference
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane::from_size(20.0))),
        material: materials.add(StandardMaterial {
            base_color: Color::rgba(0.5, 0.5, 0.5, 0.2),
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, -0.1, 0.0),
        ..default()
    });
}

fn handle_pattern_creation(
    mut commands: Commands,
    mut state: ResMut<Graph3DState>,
    mut events: EventReader<CreateGraphPatternEvent>,
    query: Query<Entity, Or<(With<Node3D>, With<Edge3D>)>>,
) {
    for event in events.iter() {
        // Clear existing entities
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        
        // Generate new graph from pattern
        state.graph = generate_pattern(event.pattern.clone());
        state.node_entities.clear();
        state.edge_entities.clear();
        
        // Create example workflow if the graph is empty
        if state.graph.nodes.is_empty() {
            let mut workflow = GraphWorkflow::new();
            workflow.create_decision_workflow();
            state.graph = workflow.graph.clone();
        }
    }
}

fn update_node_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<Graph3DState>,
    node_query: Query<Entity, With<Node3D>>,
) {
    // Skip if node entities are already created
    if state.node_entities.len() == state.graph.nodes.len() {
        return;
    }
    
    // Clear existing node entities if sizes don't match
    if !state.node_entities.is_empty() {
        for entity in node_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        state.node_entities.clear();
    }
    
    // Create node sphere mesh
    let node_mesh = meshes.add(Mesh::from(shape::UVSphere {
        radius: 0.3,
        sectors: 16,
        stacks: 16,
    }));
    
    // Create entities for each node
    for (id, node) in &state.graph.nodes {
        // Calculate position
        let position = if let Some(pos) = state.graph.node_positions.get(id) {
            Vec3::new(pos.x, 0.0, pos.y) // Convert 2D pos to 3D space
        } else {
            let mut rng = rand::thread_rng();
            Vec3::new(
                rng.gen_range(-5.0..5.0),
                0.0,
                rng.gen_range(-5.0..5.0),
            )
        };
        
        // Determine color based on node labels
        let color = if node.labels.contains(&"start".to_string()) {
            Color::GREEN
        } else if node.labels.contains(&"end".to_string()) {
            Color::RED
        } else if node.labels.contains(&"decision".to_string()) {
            Color::YELLOW
        } else if node.labels.contains(&"process".to_string()) {
            Color::BLUE
        } else {
            Color::PURPLE
        };
        
        // Create node entity
        let entity = commands.spawn((
            PbrBundle {
                mesh: node_mesh.clone(),
                material: materials.add(StandardMaterial {
                    base_color: color,
                    ..default()
                }),
                transform: Transform::from_translation(position),
                ..default()
            },
            Node3D { id: *id },
            Name::new(format!("Node: {}", node.name)),
        )).id();
        
        // Store the entity
        state.node_entities.insert(*id, entity);
    }
}

fn update_edge_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    state: Res<Graph3DState>,
    edge_query: Query<Entity, With<Edge3D>>,
    node_query: Query<(&Node3D, &Transform)>,
) {
    // Skip if edge entities are already created
    if state.edge_entities.len() == state.graph.edges.len() {
        return;
    }
    
    // Clear existing edge entities if sizes don't match
    if !state.edge_entities.is_empty() {
        for entity in edge_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
    
    // Create a map of node positions
    let mut node_positions = HashMap::new();
    for (node, transform) in node_query.iter() {
        node_positions.insert(node.id, transform.translation);
    }
    
    // Create entities for each edge
    for (id, edge) in &state.graph.edges {
        if let (Some(source_pos), Some(target_pos)) = (
            node_positions.get(&edge.source),
            node_positions.get(&edge.target)
        ) {
            // Calculate edge geometry
            let direction = *target_pos - *source_pos;
            let distance = direction.length();
            if distance < 0.01 {
                continue; // Skip if nodes are too close
            }
            
            let normalized_dir = direction / distance;
            
            // Create a cylinder mesh for the edge
            let edge_mesh = meshes.add(Mesh::from(shape::Cylinder {
                radius: 0.05,
                height: distance,
                resolution: 8,
                segments: 1,
            }));
            
            // Position cylinder in the middle, pointing toward target
            let mid_point = *source_pos + direction * 0.5;
            let rotation = Quat::from_rotation_arc(Vec3::Y, normalized_dir);
            
            // Create edge entity
            let entity = commands.spawn((
                PbrBundle {
                    mesh: edge_mesh,
                    material: materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        ..default()
                    }),
                    transform: Transform {
                        translation: mid_point,
                        rotation,
                        ..default()
                    },
                    ..default()
                },
                Edge3D {
                    source: edge.source,
                    target: edge.target,
                },
                Name::new(format!("Edge: {}-{}", edge.source, edge.target)),
            )).id();
            
            // Create an arrow at the target end
            let arrow_mesh = meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.1,
                subdivisions: 2,
            }));
            
            let arrow_pos = *target_pos - normalized_dir * 0.3;
            
            let arrow_entity = commands.spawn(
                PbrBundle {
                    mesh: arrow_mesh,
                    material: materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        ..default()
                    }),
                    transform: Transform::from_translation(arrow_pos),
                    ..default()
                },
            ).id();
            
            commands.entity(entity).add_child(arrow_entity);
            
            // Store the entity
            //state.edge_entities.insert(*id, entity);
        }
    }
} 