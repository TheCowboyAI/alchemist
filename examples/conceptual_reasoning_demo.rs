//! Conceptual Reasoning Demo
//!
//! This demo showcases the advanced reasoning capabilities of the conceptual spaces domain,
//! including analogical reasoning, conceptual blending, and semantic similarity.

use cim_domain_conceptualspaces::{
    ConceptualMetric, ConceptualPoint, ConceptualReasoning, ConceptualSpace, ConceptualSpaceId,
    ConvexRegion, DimensionId, DistanceMetric, Hyperplane, PathConstraints,
};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

fn main() {
    println!("=== Conceptual Reasoning Demo ===\n");

    // Create a conceptual space for colors and sizes
    let space = create_demo_space();

    // Create reasoning engine
    let mut reasoning = ConceptualReasoning::new(DistanceMetric::Euclidean);

    // Demo 1: Analogical Reasoning
    println!("1. Analogical Reasoning Demo");
    println!("   'Small Red' is to 'Large Red' as 'Small Blue' is to ?");
    demo_analogical_reasoning(&mut reasoning, &space);

    // Demo 2: Conceptual Blending
    println!("\n2. Conceptual Blending Demo");
    println!("   Blending 'Red' and 'Blue' concepts");
    demo_conceptual_blending(&mut reasoning, &space);

    // Demo 3: Similarity-Based Retrieval
    println!("\n3. Similarity-Based Retrieval Demo");
    println!("   Finding concepts similar to 'Medium Purple'");
    demo_similarity_retrieval(&mut reasoning, &space);

    // Demo 4: Semantic Pathfinding
    println!("\n4. Semantic Pathfinding Demo");
    println!("   Finding path from 'Small Red' to 'Large Blue'");
    demo_semantic_pathfinding(&mut reasoning, &space);

    // Demo 5: Category Inference
    println!("\n5. Category Inference Demo");
    println!("   Inferring category membership for new points");
    demo_category_inference(&mut reasoning, &space);
}

fn create_demo_space() -> ConceptualSpace {
    let color_dim_id = DimensionId::new();
    let size_dim_id = DimensionId::new();

    let mut space = ConceptualSpace::new(
        "Color-Size Space".to_string(),
        vec![color_dim_id, size_dim_id],
        ConceptualMetric::uniform(2, 2.0), // Euclidean metric
    );

    // Add some example points
    let points = vec![
        ("Small Red", vec![0.1, 0.2]),     // Red color, small size
        ("Large Red", vec![0.1, 0.8]),     // Red color, large size
        ("Small Blue", vec![0.7, 0.2]),    // Blue color, small size
        ("Large Blue", vec![0.7, 0.8]),    // Blue color, large size
        ("Medium Green", vec![0.4, 0.5]),  // Green color, medium size
        ("Small Purple", vec![0.9, 0.2]),  // Purple color, small size
        ("Medium Purple", vec![0.9, 0.5]), // Purple color, medium size
    ];

    for (name, coords) in points {
        let mut dim_map = HashMap::new();
        dim_map.insert(color_dim_id, 0);
        dim_map.insert(size_dim_id, 1);

        let mut point = ConceptualPoint::new(coords, dim_map);
        point.id = Some(Uuid::new_v4());

        space.points.insert(point.id.unwrap(), point);
        println!("   Added point: {}", name);
    }

    // Add some regions (categories)
    let red_region = create_color_region("Red", vec![0.0, 0.5], 0.3, &color_dim_id, &size_dim_id);
    let blue_region = create_color_region("Blue", vec![0.7, 0.5], 0.3, &color_dim_id, &size_dim_id);
    let small_region =
        create_size_region("Small", vec![0.5, 0.2], 0.3, &color_dim_id, &size_dim_id);
    let large_region =
        create_size_region("Large", vec![0.5, 0.8], 0.3, &color_dim_id, &size_dim_id);

    space.regions.insert(red_region.id, red_region);
    space.regions.insert(blue_region.id, blue_region);
    space.regions.insert(small_region.id, small_region);
    space.regions.insert(large_region.id, large_region);

    space
}

fn create_color_region(
    name: &str,
    center: Vec<f64>,
    _radius: f64,
    color_dim: &DimensionId,
    size_dim: &DimensionId,
) -> ConvexRegion {
    let mut dim_map = HashMap::new();
    dim_map.insert(*color_dim, 0);
    dim_map.insert(*size_dim, 1);

    let prototype = ConceptualPoint::new(center, dim_map);

    ConvexRegion {
        id: Uuid::new_v4(),
        prototype,
        boundaries: Vec::new(), // Simplified for demo
        member_points: HashSet::new(),
        name: Some(name.to_string()),
        description: Some(format!("{} color region", name)),
    }
}

fn create_size_region(
    name: &str,
    center: Vec<f64>,
    _radius: f64,
    color_dim: &DimensionId,
    size_dim: &DimensionId,
) -> ConvexRegion {
    let mut dim_map = HashMap::new();
    dim_map.insert(*color_dim, 0);
    dim_map.insert(*size_dim, 1);

    let prototype = ConceptualPoint::new(center, dim_map);

    ConvexRegion {
        id: Uuid::new_v4(),
        prototype,
        boundaries: Vec::new(), // Simplified for demo
        member_points: HashSet::new(),
        name: Some(name.to_string()),
        description: Some(format!("{} size category", name)),
    }
}

fn demo_analogical_reasoning(reasoning: &mut ConceptualReasoning, space: &ConceptualSpace) {
    // Get our example points
    let small_red = find_point_by_coords(space, &[0.1, 0.2]).unwrap();
    let large_red = find_point_by_coords(space, &[0.1, 0.8]).unwrap();
    let small_blue = find_point_by_coords(space, &[0.7, 0.2]).unwrap();

    // Perform analogical reasoning
    match reasoning.analogical_reasoning(&small_red, &large_red, &small_blue, space) {
        Ok(result) => {
            println!(
                "   Result: Point at [{:.2}, {:.2}]",
                result.coordinates[0], result.coordinates[1]
            );
            println!("   (This should be near 'Large Blue' at [0.7, 0.8])");
        }
        Err(e) => println!("   Error: {}", e),
    }
}

fn demo_conceptual_blending(reasoning: &mut ConceptualReasoning, space: &ConceptualSpace) {
    // Get red and blue points
    let red = find_point_by_coords(space, &[0.1, 0.5]).unwrap_or_else(|| {
        let mut dim_map = HashMap::new();
        dim_map.insert(DimensionId::new(), 0);
        dim_map.insert(DimensionId::new(), 1);
        ConceptualPoint::new(vec![0.1, 0.5], dim_map)
    });

    let blue = find_point_by_coords(space, &[0.7, 0.5]).unwrap_or_else(|| {
        let mut dim_map = HashMap::new();
        dim_map.insert(DimensionId::new(), 0);
        dim_map.insert(DimensionId::new(), 1);
        ConceptualPoint::new(vec![0.7, 0.5], dim_map)
    });

    // Blend the concepts
    match reasoning.conceptual_blending(&[red, blue], None, space) {
        Ok(blend) => {
            println!(
                "   Blended concept at [{:.2}, {:.2}]",
                blend.blended_concept.coordinates[0], blend.blended_concept.coordinates[1]
            );
            println!("   Coherence score: {:.2}", blend.coherence);
            println!(
                "   Emergent properties: {} found",
                blend.emergent_properties.len()
            );
        }
        Err(e) => println!("   Error: {}", e),
    }
}

fn demo_similarity_retrieval(reasoning: &mut ConceptualReasoning, space: &ConceptualSpace) {
    // Query point: Medium Purple
    let query = find_point_by_coords(space, &[0.9, 0.5]).unwrap();

    // Find similar concepts
    match reasoning.similarity_retrieval(&query, space, 3, Some("color")) {
        Ok(matches) => {
            println!("   Top 3 similar concepts:");
            for (i, m) in matches.iter().enumerate() {
                println!(
                    "   {}. Point at [{:.2}, {:.2}] - Similarity: {:.2} ({})",
                    i + 1,
                    m.concept.coordinates[0],
                    m.concept.coordinates[1],
                    m.similarity_score,
                    format!("{:?}", m.match_type)
                );
            }
        }
        Err(e) => println!("   Error: {}", e),
    }
}

fn demo_semantic_pathfinding(reasoning: &mut ConceptualReasoning, space: &ConceptualSpace) {
    // Start: Small Red, Goal: Large Blue
    let start = find_point_by_coords(space, &[0.1, 0.2]).unwrap();
    let goal = find_point_by_coords(space, &[0.7, 0.8]).unwrap();

    let constraints = PathConstraints {
        max_path_length: 5,
        max_step_size: 0.5,
        goal_threshold: 0.95,
        beam_width: 3,
    };

    match reasoning.semantic_pathfinding(&start, &goal, space, Some(constraints)) {
        Ok(path) => {
            println!("   Path found with {} waypoints:", path.waypoints.len());
            for (i, waypoint) in path.waypoints.iter().enumerate() {
                println!(
                    "   {}. [{:.2}, {:.2}]",
                    i + 1,
                    waypoint.coordinates[0],
                    waypoint.coordinates[1]
                );
            }
            println!("   Total distance: {:.2}", path.total_distance);
            println!("   Semantic coherence: {:.2}", path.semantic_coherence);
        }
        Err(e) => println!("   Error: {}", e),
    }
}

fn demo_category_inference(reasoning: &mut ConceptualReasoning, space: &ConceptualSpace) {
    // Test point between categories
    let mut dim_map = HashMap::new();
    dim_map.insert(DimensionId::new(), 0);
    dim_map.insert(DimensionId::new(), 1);

    let test_point = ConceptualPoint::new(vec![0.3, 0.3], dim_map);

    match reasoning.categorical_inference(&test_point, space) {
        Ok(inference) => {
            println!(
                "   Point at [{:.2}, {:.2}]",
                test_point.coordinates[0], test_point.coordinates[1]
            );
            println!("   Category memberships:");
            for membership in &inference.category_memberships {
                if let Some(name) = &membership.category_name {
                    println!(
                        "   - {}: {:.2} strength",
                        name, membership.membership_strength
                    );
                }
            }
            println!("   Confidence: {:.2}", inference.confidence);
            println!(
                "   Inferred properties: {:?}",
                inference.inferred_properties
            );
        }
        Err(e) => println!("   Error: {}", e),
    }
}

fn find_point_by_coords(space: &ConceptualSpace, target_coords: &[f64]) -> Option<ConceptualPoint> {
    for (_, point) in &space.points {
        if point.coordinates.len() == target_coords.len() {
            let distance: f64 = point
                .coordinates
                .iter()
                .zip(target_coords.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt();

            if distance < 0.01 {
                return Some(point.clone());
            }
        }
    }
    None
}
