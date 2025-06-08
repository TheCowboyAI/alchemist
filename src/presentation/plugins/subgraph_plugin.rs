//! Plugin for subgraph visualization and spatial mapping

use bevy::prelude::*;
use crate::presentation::bevy_systems::{
    SubgraphSpatialMap,
    visualize_subgraph_boundaries,
};

/// Plugin that adds subgraph visualization capabilities
pub struct SubgraphPlugin;

impl Plugin for SubgraphPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<SubgraphSpatialMap>()

            // Systems
            .add_systems(Update, visualize_subgraph_boundaries);
    }
}
