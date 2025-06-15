use crate::domain::value_objects::{
    CollapseStrategy, LayoutDirection, LayoutStrategy, NodeId, Position3D,
};
use petgraph::graph::{Graph, NodeIndex};
use std::collections::HashMap;
use std::f32::consts::PI;

/// Service for calculating subgraph layouts
pub struct SubgraphLayoutCalculator {
    force_directed: ForceDirectedLayout,
    hierarchical: HierarchicalLayout,
    circular: CircularLayout,
    grid: GridLayout,
    geometric: GeometricLayout,
}

impl SubgraphLayoutCalculator {
    pub fn new() -> Self {
        Self {
            force_directed: ForceDirectedLayout::new(),
            hierarchical: HierarchicalLayout::new(),
            circular: CircularLayout::new(),
            grid: GridLayout::new(),
            geometric: GeometricLayout::new(),
        }
    }

    /// Calculate the collapsed position for a subgraph
    pub fn calculate_collapsed_position(
        &self,
        node_positions: &HashMap<NodeId, Position3D>,
        collapse_strategy: &CollapseStrategy,
        graph: Option<&Graph<NodeId, ()>>,
    ) -> Position3D {
        match collapse_strategy {
            CollapseStrategy::Centroid => self.calculate_centroid(node_positions),
            CollapseStrategy::MostConnected => {
                if let Some(g) = graph {
                    self.find_most_connected_position(g, node_positions)
                } else {
                    self.calculate_centroid(node_positions)
                }
            }
            CollapseStrategy::WeightedCenter => {
                self.calculate_weighted_center(node_positions, graph)
            }
            CollapseStrategy::FixedPosition(pos) => *pos,
        }
    }

    /// Calculate expansion layout for nodes
    pub fn calculate_expansion_layout(
        &self,
        nodes: &[NodeId],
        center: Position3D,
        layout_strategy: &LayoutStrategy,
        graph: Option<&Graph<NodeId, ()>>,
        previous_positions: Option<&HashMap<NodeId, Position3D>>,
    ) -> HashMap<NodeId, Position3D> {
        match layout_strategy {
            LayoutStrategy::ForceDirected {
                iterations,
                spring_strength,
                repulsion_strength,
            } => self.force_directed.calculate(
                nodes,
                center,
                graph,
                *iterations,
                *spring_strength,
                *repulsion_strength,
            ),
            LayoutStrategy::Hierarchical {
                direction,
                layer_spacing,
                node_spacing,
            } => self.hierarchical.calculate(
                nodes,
                center,
                graph,
                direction,
                *layer_spacing,
                *node_spacing,
            ),
            LayoutStrategy::Circular {
                radius,
                start_angle,
            } => self
                .circular
                .calculate(nodes, center, *radius, *start_angle),
            LayoutStrategy::Grid { columns, spacing } => {
                self.grid
                    .calculate(nodes, center, *columns as usize, *spacing)
            }
            LayoutStrategy::Geometric { spacing } => {
                self.geometric.calculate(nodes, center, *spacing)
            }
            LayoutStrategy::RestorePrevious => {
                if let Some(prev) = previous_positions {
                    prev.clone()
                } else {
                    // Fallback to circular if no previous positions
                    self.circular.calculate(nodes, center, 5.0, 0.0)
                }
            }
        }
    }

    /// Optimize edge crossings in a layout
    pub fn optimize_edge_crossings(
        &self,
        positions: &mut HashMap<NodeId, Position3D>,
        graph: &Graph<NodeId, ()>,
        iterations: u32,
    ) {
        // Simple optimization: try to reduce edge crossings by swapping positions
        for _ in 0..iterations {
            let mut best_swap: Option<(NodeId, NodeId)> = None;
            let mut best_improvement = 0;

            let nodes: Vec<NodeId> = positions.keys().cloned().collect();

            // Try all pairs of nodes
            for i in 0..nodes.len() {
                for j in (i + 1)..nodes.len() {
                    let current_crossings = self.count_edge_crossings(positions, graph);

                    // Swap positions
                    let pos_i = positions[&nodes[i]];
                    let pos_j = positions[&nodes[j]];
                    positions.insert(nodes[i], pos_j);
                    positions.insert(nodes[j], pos_i);

                    let new_crossings = self.count_edge_crossings(positions, graph);
                    let improvement = current_crossings.saturating_sub(new_crossings);

                    if improvement > best_improvement {
                        best_improvement = improvement;
                        best_swap = Some((nodes[i], nodes[j]));
                    }

                    // Swap back
                    positions.insert(nodes[i], pos_i);
                    positions.insert(nodes[j], pos_j);
                }
            }

            // Apply best swap if found
            if let Some((node_a, node_b)) = best_swap {
                let pos_a = positions[&node_a];
                let pos_b = positions[&node_b];
                positions.insert(node_a, pos_b);
                positions.insert(node_b, pos_a);
            } else {
                break; // No improvement found
            }
        }
    }

    /// Apply constraints to a layout
    pub fn apply_constraints(
        &self,
        positions: &mut HashMap<NodeId, Position3D>,
        constraints: &LayoutConstraints,
    ) {
        // Apply bounding box constraints
        if let Some(bounds) = &constraints.bounding_box {
            for pos in positions.values_mut() {
                pos.x = pos.x.max(bounds.min.x).min(bounds.max.x);
                pos.y = pos.y.max(bounds.min.y).min(bounds.max.y);
                pos.z = pos.z.max(bounds.min.z).min(bounds.max.z);
            }
        }

        // Apply minimum spacing constraints
        if let Some(min_spacing) = constraints.minimum_spacing {
            self.enforce_minimum_spacing(positions, min_spacing);
        }

        // Apply fixed positions
        for (node_id, fixed_pos) in &constraints.fixed_positions {
            if let Some(pos) = positions.get_mut(node_id) {
                *pos = *fixed_pos;
            }
        }
    }

    // Helper methods
    fn calculate_centroid(&self, positions: &HashMap<NodeId, Position3D>) -> Position3D {
        if positions.is_empty() {
            return Position3D::new_unchecked(0.0, 0.0, 0.0);
        }

        let sum = positions
            .values()
            .fold(Position3D::new_unchecked(0.0, 0.0, 0.0), |acc, pos| {
                Position3D::new_unchecked(acc.x + pos.x, acc.y + pos.y, acc.z + pos.z)
            });

        let count = positions.len() as f32;
        Position3D::new_unchecked(sum.x / count, sum.y / count, sum.z / count)
    }

    fn find_most_connected_position(
        &self,
        graph: &Graph<NodeId, ()>,
        positions: &HashMap<NodeId, Position3D>,
    ) -> Position3D {
        let mut max_degree = 0;
        let mut most_connected_node = None;

        // Find node indices for our nodes
        let node_indices: HashMap<NodeId, NodeIndex> = graph
            .node_indices()
            .filter_map(|idx| graph.node_weight(idx).map(|&node_id| (node_id, idx)))
            .collect();

        for (node_id, _) in positions {
            if let Some(&node_idx) = node_indices.get(node_id) {
                let degree = graph.edges(node_idx).count();
                if degree > max_degree {
                    max_degree = degree;
                    most_connected_node = Some(node_id);
                }
            }
        }

        most_connected_node
            .and_then(|node_id| positions.get(node_id))
            .cloned()
            .unwrap_or_else(|| self.calculate_centroid(positions))
    }

    fn calculate_weighted_center(
        &self,
        positions: &HashMap<NodeId, Position3D>,
        graph: Option<&Graph<NodeId, ()>>,
    ) -> Position3D {
        if let Some(g) = graph {
            // Weight by node degree
            let node_indices: HashMap<NodeId, NodeIndex> = g
                .node_indices()
                .filter_map(|idx| g.node_weight(idx).map(|&node_id| (node_id, idx)))
                .collect();

            let mut weighted_sum = Position3D::new_unchecked(0.0, 0.0, 0.0);
            let mut total_weight = 0.0;

            for (node_id, pos) in positions {
                if let Some(&node_idx) = node_indices.get(node_id) {
                    let weight = (g.edges(node_idx).count() + 1) as f32;
                    weighted_sum.x += pos.x * weight;
                    weighted_sum.y += pos.y * weight;
                    weighted_sum.z += pos.z * weight;
                    total_weight += weight;
                }
            }

            if total_weight > 0.0 {
                Position3D::new_unchecked(
                    weighted_sum.x / total_weight,
                    weighted_sum.y / total_weight,
                    weighted_sum.z / total_weight,
                )
            } else {
                self.calculate_centroid(positions)
            }
        } else {
            self.calculate_centroid(positions)
        }
    }

    fn count_edge_crossings(
        &self,
        positions: &HashMap<NodeId, Position3D>,
        graph: &Graph<NodeId, ()>,
    ) -> usize {
        // Simplified 2D edge crossing detection
        let edges: Vec<(NodeId, NodeId)> = graph
            .edge_indices()
            .filter_map(|edge_idx| {
                graph
                    .edge_endpoints(edge_idx)
                    .and_then(|(src_idx, tgt_idx)| {
                        if let (Some(&src_id), Some(&tgt_id)) =
                            (graph.node_weight(src_idx), graph.node_weight(tgt_idx))
                        {
                            Some((src_id, tgt_id))
                        } else {
                            None
                        }
                    })
            })
            .collect();

        let mut crossings = 0;
        for i in 0..edges.len() {
            for j in (i + 1)..edges.len() {
                if self.edges_cross(
                    positions.get(&edges[i].0),
                    positions.get(&edges[i].1),
                    positions.get(&edges[j].0),
                    positions.get(&edges[j].1),
                ) {
                    crossings += 1;
                }
            }
        }
        crossings
    }

    fn edges_cross(
        &self,
        p1: Option<&Position3D>,
        p2: Option<&Position3D>,
        p3: Option<&Position3D>,
        p4: Option<&Position3D>,
    ) -> bool {
        // Simple 2D line intersection test
        if let (Some(p1), Some(p2), Some(p3), Some(p4)) = (p1, p2, p3, p4) {
            let d = (p1.x - p2.x) * (p3.y - p4.y) - (p1.y - p2.y) * (p3.x - p4.x);
            if d.abs() < f32::EPSILON {
                return false; // Parallel lines
            }

            let t = ((p1.x - p3.x) * (p3.y - p4.y) - (p1.y - p3.y) * (p3.x - p4.x)) / d;
            let u = -((p1.x - p2.x) * (p1.y - p3.y) - (p1.y - p2.y) * (p1.x - p3.x)) / d;

            t > 0.0 && t < 1.0 && u > 0.0 && u < 1.0
        } else {
            false
        }
    }

    fn enforce_minimum_spacing(
        &self,
        positions: &mut HashMap<NodeId, Position3D>,
        min_spacing: f32,
    ) {
        let nodes: Vec<NodeId> = positions.keys().cloned().collect();
        let mut adjusted = true;

        while adjusted {
            adjusted = false;
            for i in 0..nodes.len() {
                for j in (i + 1)..nodes.len() {
                    if let (Some(pos_i), Some(pos_j)) =
                        (positions.get(&nodes[i]), positions.get(&nodes[j]))
                    {
                        let dx = pos_j.x - pos_i.x;
                        let dy = pos_j.y - pos_i.y;
                        let dz = pos_j.z - pos_i.z;
                        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

                        if dist < min_spacing && dist > 0.0 {
                            // Push nodes apart
                            let factor = (min_spacing - dist) / (2.0 * dist);
                            let offset_x = dx * factor;
                            let offset_y = dy * factor;
                            let offset_z = dz * factor;

                            if let Some(pos) = positions.get_mut(&nodes[i]) {
                                pos.x -= offset_x;
                                pos.y -= offset_y;
                                pos.z -= offset_z;
                            }
                            if let Some(pos) = positions.get_mut(&nodes[j]) {
                                pos.x += offset_x;
                                pos.y += offset_y;
                                pos.z += offset_z;
                            }
                            adjusted = true;
                        }
                    }
                }
            }
        }
    }
}

/// Force-directed layout calculator
pub struct ForceDirectedLayout;

impl ForceDirectedLayout {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate(
        &self,
        nodes: &[NodeId],
        center: Position3D,
        graph: Option<&Graph<NodeId, ()>>,
        iterations: u32,
        spring_strength: f32,
        repulsion_strength: f32,
    ) -> HashMap<NodeId, Position3D> {
        let mut positions = HashMap::new();

        // Initialize with random positions around center
        for (i, &node_id) in nodes.iter().enumerate() {
            let angle = 2.0 * PI * i as f32 / nodes.len() as f32;
            let radius = 5.0;
            if let Ok(pos) = Position3D::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
                center.z,
            ) {
                positions.insert(node_id, pos);
            }
        }

        // Apply force-directed algorithm
        if let Some(g) = graph {
            for _ in 0..iterations {
                let mut forces: HashMap<NodeId, Position3D> = HashMap::new();

                // Calculate repulsion forces
                for i in 0..nodes.len() {
                    for j in (i + 1)..nodes.len() {
                        if let (Some(pos_i), Some(pos_j)) =
                            (positions.get(&nodes[i]), positions.get(&nodes[j]))
                        {
                            let dx = pos_j.x - pos_i.x;
                            let dy = pos_j.y - pos_i.y;
                            let dz = pos_j.z - pos_i.z;
                            let dist_sq = dx * dx + dy * dy + dz * dz + 0.01;
                            let force = repulsion_strength / dist_sq;

                            let fx = force * dx / dist_sq.sqrt();
                            let fy = force * dy / dist_sq.sqrt();
                            let fz = force * dz / dist_sq.sqrt();

                            forces.entry(nodes[i]).or_insert(Position3D::default()).x -= fx;
                            forces.entry(nodes[i]).or_insert(Position3D::default()).y -= fy;
                            forces.entry(nodes[i]).or_insert(Position3D::default()).z -= fz;

                            forces.entry(nodes[j]).or_insert(Position3D::default()).x += fx;
                            forces.entry(nodes[j]).or_insert(Position3D::default()).y += fy;
                            forces.entry(nodes[j]).or_insert(Position3D::default()).z += fz;
                        }
                    }
                }

                // Calculate spring forces for edges
                let node_indices: HashMap<NodeId, NodeIndex> = g
                    .node_indices()
                    .filter_map(|idx| g.node_weight(idx).map(|&node_id| (node_id, idx)))
                    .collect();

                for edge in g.edge_indices() {
                    if let Some((src_idx, tgt_idx)) = g.edge_endpoints(edge) {
                        if let (Some(&src_id), Some(&tgt_id)) =
                            (g.node_weight(src_idx), g.node_weight(tgt_idx))
                        {
                            if let (Some(pos_src), Some(pos_tgt)) =
                                (positions.get(&src_id), positions.get(&tgt_id))
                            {
                                let dx = pos_tgt.x - pos_src.x;
                                let dy = pos_tgt.y - pos_src.y;
                                let dz = pos_tgt.z - pos_src.z;
                                let dist = (dx * dx + dy * dy + dz * dz).sqrt();

                                if dist > 0.0 {
                                    let force = spring_strength * (dist - 3.0); // Ideal distance of 3.0
                                    let fx = force * dx / dist;
                                    let fy = force * dy / dist;
                                    let fz = force * dz / dist;

                                    forces.entry(src_id).or_insert(Position3D::default()).x += fx;
                                    forces.entry(src_id).or_insert(Position3D::default()).y += fy;
                                    forces.entry(src_id).or_insert(Position3D::default()).z += fz;

                                    forces.entry(tgt_id).or_insert(Position3D::default()).x -= fx;
                                    forces.entry(tgt_id).or_insert(Position3D::default()).y -= fy;
                                    forces.entry(tgt_id).or_insert(Position3D::default()).z -= fz;
                                }
                            }
                        }
                    }
                }

                // Apply forces
                for (node_id, force) in forces {
                    if let Some(pos) = positions.get_mut(&node_id) {
                        pos.x += force.x * 0.1; // Damping factor
                        pos.y += force.y * 0.1;
                        pos.z += force.z * 0.1;
                    }
                }
            }
        }

        positions
    }
}

/// Hierarchical layout calculator
pub struct HierarchicalLayout;

impl HierarchicalLayout {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate(
        &self,
        nodes: &[NodeId],
        center: Position3D,
        graph: Option<&Graph<NodeId, ()>>,
        direction: &LayoutDirection,
        layer_spacing: f32,
        node_spacing: f32,
    ) -> HashMap<NodeId, Position3D> {
        let mut positions = HashMap::new();

        // Simple hierarchical layout - arrange in layers
        // In a real implementation, this would use topological sorting
        let layers = self.assign_layers(nodes, graph);

        for (layer_idx, layer_nodes) in layers.iter().enumerate() {
            let layer_offset = layer_idx as f32 * layer_spacing;

            for (node_idx, &node_id) in layer_nodes.iter().enumerate() {
                let node_offset =
                    (node_idx as f32 - (layer_nodes.len() - 1) as f32 / 2.0) * node_spacing;

                let position = match direction {
                    LayoutDirection::TopToBottom => {
                        Position3D::new(center.x + node_offset, center.y - layer_offset, center.z)
                    }
                    LayoutDirection::BottomToTop => {
                        Position3D::new(center.x + node_offset, center.y + layer_offset, center.z)
                    }
                    LayoutDirection::LeftToRight => {
                        Position3D::new(center.x + layer_offset, center.y + node_offset, center.z)
                    }
                    LayoutDirection::RightToLeft => {
                        Position3D::new(center.x - layer_offset, center.y + node_offset, center.z)
                    }
                };

                if let Ok(pos) = position {
                    positions.insert(node_id, pos);
                }
            }
        }

        positions
    }

    fn assign_layers(
        &self,
        nodes: &[NodeId],
        graph: Option<&Graph<NodeId, ()>>,
    ) -> Vec<Vec<NodeId>> {
        // Simple layer assignment - in practice would use proper graph algorithms
        if nodes.len() <= 3 {
            vec![nodes.to_vec()]
        } else {
            // Split into roughly equal layers
            let layer_size = (nodes.len() as f32 / 3.0).ceil() as usize;
            nodes
                .chunks(layer_size)
                .map(|chunk| chunk.to_vec())
                .collect()
        }
    }
}

/// Circular layout calculator
pub struct CircularLayout;

impl CircularLayout {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate(
        &self,
        nodes: &[NodeId],
        center: Position3D,
        radius: f32,
        start_angle: f32,
    ) -> HashMap<NodeId, Position3D> {
        let mut positions = HashMap::new();
        let angle_step = 2.0 * PI / nodes.len() as f32;

        for (i, &node_id) in nodes.iter().enumerate() {
            let angle = start_angle + i as f32 * angle_step;
            if let Ok(pos) = Position3D::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
                center.z,
            ) {
                positions.insert(node_id, pos);
            }
        }

        positions
    }
}

/// Grid layout calculator
pub struct GridLayout;

impl GridLayout {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate(
        &self,
        nodes: &[NodeId],
        center: Position3D,
        columns: usize,
        spacing: f32,
    ) -> HashMap<NodeId, Position3D> {
        let mut positions = HashMap::new();
        let columns = columns.max(1);
        let rows = (nodes.len() + columns - 1) / columns;

        for (i, &node_id) in nodes.iter().enumerate() {
            let row = i / columns;
            let col = i % columns;

            let x = center.x + (col as f32 - (columns - 1) as f32 / 2.0) * spacing;
            let y = center.y + (row as f32 - (rows - 1) as f32 / 2.0) * spacing;

            if let Ok(pos) = Position3D::new(x, y, center.z) {
                positions.insert(node_id, pos);
            }
        }

        positions
    }
}

/// Geometric layout calculator (equiangular polygons)
pub struct GeometricLayout;

impl GeometricLayout {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate(
        &self,
        nodes: &[NodeId],
        center: Position3D,
        spacing: f32,
    ) -> HashMap<NodeId, Position3D> {
        let mut positions = HashMap::new();

        if nodes.is_empty() {
            return positions;
        }

        if nodes.len() == 1 {
            positions.insert(nodes[0], center);
            return positions;
        }

        // Calculate radius based on spacing and number of nodes
        let angle = 2.0 * PI / nodes.len() as f32;
        let radius = spacing / (2.0 * (angle / 2.0).sin());

        // Place nodes in a regular polygon
        for (i, &node_id) in nodes.iter().enumerate() {
            let node_angle = i as f32 * angle;
            if let Ok(pos) = Position3D::new(
                center.x + radius * node_angle.cos(),
                center.y + radius * node_angle.sin(),
                center.z,
            ) {
                positions.insert(node_id, pos);
            }
        }

        positions
    }
}

/// Layout constraints
#[derive(Debug, Clone)]
pub struct LayoutConstraints {
    pub bounding_box: Option<BoundingBox>,
    pub minimum_spacing: Option<f32>,
    pub fixed_positions: HashMap<NodeId, Position3D>,
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub min: Position3D,
    pub max: Position3D,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centroid_calculation() {
        let calculator = SubgraphLayoutCalculator::new();
        let mut positions = HashMap::new();
        positions.insert(NodeId::new(), Position3D::new(0.0, 0.0, 0.0).unwrap());
        positions.insert(NodeId::new(), Position3D::new(4.0, 0.0, 0.0).unwrap());
        positions.insert(NodeId::new(), Position3D::new(2.0, 2.0, 0.0).unwrap());

        let centroid = calculator.calculate_centroid(&positions);
        assert_eq!(centroid.x, 2.0);
        assert!((centroid.y - 2.0 / 3.0).abs() < 0.01);
        assert_eq!(centroid.z, 0.0);
    }

    #[test]
    fn test_circular_layout() {
        let calculator = SubgraphLayoutCalculator::new();
        let nodes = vec![NodeId::new(), NodeId::new(), NodeId::new(), NodeId::new()];
        let center = Position3D::new(0.0, 0.0, 0.0).unwrap();

        let positions = calculator.calculate_expansion_layout(
            &nodes,
            center,
            &LayoutStrategy::Circular {
                radius: 5.0,
                start_angle: 0.0,
            },
            None,
            None,
        );

        assert_eq!(positions.len(), 4);

        // Check that all nodes are at the correct radius
        for pos in positions.values() {
            let dist = (pos.x * pos.x + pos.y * pos.y).sqrt();
            assert!((dist - 5.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_grid_layout() {
        let calculator = SubgraphLayoutCalculator::new();
        let nodes: Vec<NodeId> = (0..6).map(|_| NodeId::new()).collect();
        let center = Position3D::new(0.0, 0.0, 0.0).unwrap();

        let positions = calculator.calculate_expansion_layout(
            &nodes,
            center,
            &LayoutStrategy::Grid {
                columns: 3,
                spacing: 2.0,
            },
            None,
            None,
        );

        assert_eq!(positions.len(), 6);

        // Check that nodes are arranged in a 2x3 grid
        let y_values: HashSet<i32> = positions
            .values()
            .map(|p| (p.y / 2.0).round() as i32)
            .collect();
        assert_eq!(y_values.len(), 2); // 2 rows
    }
}
