use crate::contexts::visualization::services::{NodePointCloud, EdgePointCloud};
use bevy::prelude::*;

/// Plugin for point cloud rendering
pub struct PointCloudPlugin;

impl Plugin for PointCloudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            render_node_point_clouds,
            render_edge_point_clouds,
        ));
    }
}

/// System to render node point clouds using gizmos
fn render_node_point_clouds(
    point_clouds: Query<(&NodePointCloud, &Transform), Without<EdgePointCloud>>,
    mut gizmos: Gizmos,
) {
    for (cloud, transform) in point_clouds.iter() {
        for (i, point) in cloud.points.iter().enumerate() {
            let world_pos = transform.transform_point(*point);
            let color = cloud.colors.get(i).copied().unwrap_or(Color::WHITE);
            let size = cloud.sizes.get(i).copied().unwrap_or(0.02);

            // Draw point as a small sphere
            gizmos.sphere(world_pos, size, color);
        }
    }
}

/// System to render edge point clouds using gizmos
fn render_edge_point_clouds(
    point_clouds: Query<(&EdgePointCloud, &Transform), Without<NodePointCloud>>,
    mut gizmos: Gizmos,
) {
    for (cloud, transform) in point_clouds.iter() {
        for (i, point) in cloud.points.iter().enumerate() {
            let world_pos = transform.transform_point(*point);
            let color = cloud.colors.get(i).copied().unwrap_or(Color::srgb(0.7, 0.7, 0.7));
            let size = cloud.sizes.get(i).copied().unwrap_or(0.01);

            // Draw point as a small sphere
            gizmos.sphere(world_pos, size, color);
        }
    }
}
