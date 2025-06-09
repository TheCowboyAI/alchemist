//! Conceptual Graph Plugin for Bevy Integration
//!
//! This plugin bridges the domain conceptual graph system with Bevy ECS visualization

use bevy::prelude::*;
use tracing::info;
use crate::domain::conceptual_graph::*;
use crate::domain::value_objects::{GraphId, NodeId, EdgeId};
use crate::presentation::components::{GraphNode, GraphEdge};
use crate::application::CommandEvent;
use crate::domain::commands::{Command, GraphCommand};
use std::collections::HashMap;

/// Plugin that integrates conceptual graph functionality into the main application
pub struct ConceptualGraphPlugin;

impl Plugin for ConceptualGraphPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<ConceptualSpaceResource>()
            .init_resource::<ConceptualGraphRegistry>()

            // Events
            .add_event::<ConceptualGraphEvent>()

            // Systems
            .add_systems(Startup, setup_conceptual_space)
            .add_systems(Update, (
                process_conceptual_graph_commands,
                update_conceptual_positions,
                visualize_category_types,
                render_quality_dimensions,
            ).chain());
    }
}

/// Resource holding the active conceptual space configuration
#[derive(Resource, Default)]
pub struct ConceptualSpaceResource {
    pub dimensions: Vec<QualityDimension>,
    pub active_space_id: Option<ConceptId>,
    pub visualization_scale: f32,
}

impl ConceptualSpaceResource {
    pub fn new() -> Self {
        Self {
            dimensions: vec![
                QualityDimension::new("Complexity", DimensionType::Continuous, 0.0..10.0),
                QualityDimension::new("Cohesion", DimensionType::Continuous, 0.0..1.0),
                QualityDimension::new("Abstraction", DimensionType::Ordinal, 0.0..5.0),
            ],
            active_space_id: None,
            visualization_scale: 10.0,
        }
    }
}

/// Registry of all conceptual graphs in the system
#[derive(Resource, Default)]
pub struct ConceptualGraphRegistry {
    pub graphs: HashMap<GraphId, ConceptGraph>,
    pub node_to_graph: HashMap<NodeId, GraphId>,
}

/// Component marking an entity as part of a conceptual graph
#[derive(Component)]
pub struct ConceptualGraphEntity {
    pub graph_id: GraphId,
    pub concept_graph: ConceptGraph,
}

/// Component for nodes with conceptual space positioning
#[derive(Component)]
pub struct ConceptualNodeComponent {
    pub node_id: NodeId,
    pub concept_type: ConceptType,
    pub conceptual_point: ConceptualPoint,
    pub quality_values: HashMap<String, f64>,
}

/// Component for edges with conceptual relationships
#[derive(Component)]
pub struct ConceptualEdgeComponent {
    pub edge_id: EdgeId,
    pub relationship: ConceptRelationship,
    pub semantic_distance: f32,
}

/// Events for conceptual graph operations
#[derive(Event)]
pub enum ConceptualGraphEvent {
    GraphCreated {
        graph_id: GraphId,
        concept_graph: ConceptGraph,
    },
    NodeMappedToConceptualSpace {
        node_id: NodeId,
        conceptual_point: ConceptualPoint,
    },
    CategoryTypeChanged {
        graph_id: GraphId,
        new_category: CategoryType,
    },
    MorphismApplied {
        source_graph: GraphId,
        target_graph: GraphId,
        morphism: GraphMorphism,
    },
}

/// Setup the conceptual space with default configuration
fn setup_conceptual_space(
    mut commands: Commands,
    mut conceptual_space: ResMut<ConceptualSpaceResource>,
) {
    info!("Setting up conceptual space for graph visualization");

    // Initialize with default dimensions
    *conceptual_space = ConceptualSpaceResource::new();

    // Create a visual indicator for the conceptual space origin
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        Name::new("ConceptualSpaceOrigin"),
    ));
}

/// Process commands that create or modify conceptual graphs
fn process_conceptual_graph_commands(
    mut commands: Commands,
    mut command_events: EventReader<CommandEvent>,
    mut conceptual_events: EventWriter<ConceptualGraphEvent>,
    mut registry: ResMut<ConceptualGraphRegistry>,
    conceptual_space: Res<ConceptualSpaceResource>,
) {
    for event in command_events.read() {
        match &event.command {
            Command::Graph(GraphCommand::CreateConceptualGraph { graph_id, name, category_type }) => {
                // Create a new conceptual graph
                let mut concept_graph = ConceptGraph::new(name.clone());
                concept_graph.category = category_type.clone();
                concept_graph.quality_dimensions = conceptual_space.dimensions.clone();

                // Register the graph
                registry.graphs.insert(*graph_id, concept_graph.clone());

                // Spawn an entity for the graph
                commands.spawn((
                    ConceptualGraphEntity {
                        graph_id: *graph_id,
                        concept_graph: concept_graph.clone(),
                    },
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::default(),
                    Name::new(format!("ConceptualGraph_{}", name)),
                ));

                // Emit event
                conceptual_events.send(ConceptualGraphEvent::GraphCreated {
                    graph_id: *graph_id,
                    concept_graph,
                });
            }
            _ => {} // Handle other commands as needed
        }
    }
}

/// Update node positions based on their conceptual space coordinates
fn update_conceptual_positions(
    mut node_query: Query<(&ConceptualNodeComponent, &mut Transform), With<GraphNode>>,
    conceptual_space: Res<ConceptualSpaceResource>,
    time: Res<Time>,
) {
    for (conceptual_node, mut transform) in node_query.iter_mut() {
        // Map conceptual coordinates to 3D space
        let target_position = map_conceptual_to_visual(
            &conceptual_node.conceptual_point,
            conceptual_space.visualization_scale,
        );

        // Smooth interpolation to target position
        transform.translation = transform.translation.lerp(
            target_position,
            2.0 * time.delta_secs(),
        );
    }
}

/// Visualize different category types with distinct visual representations
fn visualize_category_types(
    graph_query: Query<&ConceptualGraphEntity>,
    mut gizmos: Gizmos,
) {
    for graph_entity in graph_query.iter() {
        let color = match &graph_entity.concept_graph.category {
            CategoryType::Order => Color::srgb(0.2, 0.8, 0.2),        // Green for hierarchies
            CategoryType::Database => Color::srgb(0.2, 0.2, 0.8),     // Blue for data
            CategoryType::Monoidal => Color::srgb(0.8, 0.8, 0.2),     // Yellow for parallel
            CategoryType::Profunctor => Color::srgb(0.8, 0.2, 0.8),   // Magenta for relations
            CategoryType::Enriched { .. } => Color::srgb(0.5, 0.8, 0.8), // Cyan for enriched
            CategoryType::Topos => Color::srgb(0.8, 0.5, 0.2),        // Orange for logic
            CategoryType::Operad => Color::srgb(0.5, 0.5, 0.8),       // Purple for operations
            CategoryType::Simple => Color::srgb(0.7, 0.7, 0.7),       // Gray for simple
            CategoryType::Functor => Color::srgb(0.9, 0.3, 0.3),      // Red for functors
            CategoryType::Slice { .. } => Color::srgb(0.3, 0.9, 0.3), // Light green for slice
        };

        // Draw category type indicator
        gizmos.sphere(
            Isometry3d::from_translation(Vec3::new(0.0, 10.0, 0.0)),
            0.5,
            color,
        );
    }
}

/// Render quality dimensions as visual guides
fn render_quality_dimensions(
    conceptual_space: Res<ConceptualSpaceResource>,
    mut gizmos: Gizmos,
) {
    // Draw axes for each quality dimension
    for (i, _dimension) in conceptual_space.dimensions.iter().enumerate() {
        let axis_color = match i {
            0 => Color::srgb(1.0, 0.0, 0.0), // Red for X
            1 => Color::srgb(0.0, 1.0, 0.0), // Green for Y
            2 => Color::srgb(0.0, 0.0, 1.0), // Blue for Z
            _ => Color::srgb(0.5, 0.5, 0.5), // Gray for others
        };

        // Draw dimension axis
        let axis_length = conceptual_space.visualization_scale;
        let axis_direction = match i {
            0 => Vec3::X,
            1 => Vec3::Y,
            2 => Vec3::Z,
            _ => Vec3::ZERO,
        };

        gizmos.line(
            Vec3::ZERO,
            axis_direction * axis_length,
            axis_color,
        );

        // Add dimension label (would need text in real implementation)
        gizmos.sphere(
            Isometry3d::from_translation(axis_direction * axis_length),
            0.2,
            axis_color,
        );
    }
}

/// Helper function to map conceptual coordinates to 3D visual space
fn map_conceptual_to_visual(point: &ConceptualPoint, scale: f32) -> Vec3 {
    // Map first 3 dimensions to X, Y, Z
    let x = point.coordinates.get(0).copied().unwrap_or(0.0) as f32;
    let y = point.coordinates.get(1).copied().unwrap_or(0.0) as f32;
    let z = point.coordinates.get(2).copied().unwrap_or(0.0) as f32;

    Vec3::new(x * scale, y * scale, z * scale)
}
