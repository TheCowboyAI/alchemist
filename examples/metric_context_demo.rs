use ia::domain::conceptual_graph::{
    ConceptId, ConsumptionFunction, CostFunction, DistanceFunction, MetricContext, MetricType,
    ResourceType,
};

fn main() {
    println!("=== MetricContext Demo ===\n");

    // Create concepts for a software architecture domain
    let ui_concept = ConceptId::new();
    let api_concept = ConceptId::new();
    let db_concept = ConceptId::new();
    let cache_concept = ConceptId::new();
    let auth_concept = ConceptId::new();

    println!("Created architectural concepts:");
    println!("- UI Layer: {:?}", ui_concept);
    println!("- API Layer: {:?}", api_concept);
    println!("- Database: {:?}", db_concept);
    println!("- Cache: {:?}", cache_concept);
    println!("- Auth Service: {:?}", auth_concept);

    // Example 1: Semantic Distance Metric
    println!("\n1. Semantic Distance Metric");
    println!("   Measuring architectural coupling between components");

    let mut semantic_metric = MetricContext::new(
        "Architectural Coupling".to_string(),
        ui_concept, // Base context could be any concept
        MetricType::SemanticDistance {
            distance_function: DistanceFunction::Custom("coupling_distance".to_string()),
        },
    );

    // Set coupling distances (lower = more tightly coupled)
    semantic_metric.set_distance(ui_concept, api_concept, 1.0);
    semantic_metric.set_distance(api_concept, db_concept, 1.5);
    semantic_metric.set_distance(api_concept, cache_concept, 1.2);
    semantic_metric.set_distance(api_concept, auth_concept, 2.0);
    semantic_metric.set_distance(ui_concept, auth_concept, 3.0);
    semantic_metric.set_distance(cache_concept, db_concept, 0.5); // Cache and DB are tightly coupled

    // Find shortest path from UI to Database
    match semantic_metric.shortest_path(ui_concept, db_concept) {
        Ok(path) => {
            println!("   Shortest coupling path from UI to Database:");
            println!("   Path length: {}", path.total_distance);
            println!("   Route: {} nodes", path.nodes.len());
        }
        Err(e) => println!("   Error finding path: {}", e),
    }

    // Find nearest neighbors to API
    let neighbors = semantic_metric.nearest_neighbors(api_concept, 3);
    println!("\n   Nearest neighbors to API Layer:");
    for (concept, distance) in neighbors {
        println!("   - Distance {}: concept {:?}", distance, concept);
    }

    // Example 2: Transformation Cost Metric
    println!("\n2. Transformation Cost Metric");
    println!("   Measuring cost of migrating between technologies");

    let mut cost_metric = MetricContext::new(
        "Migration Cost".to_string(),
        ui_concept,
        MetricType::TransformationCost {
            cost_function: CostFunction::Exponential {
                base: 2.0,
                rate: 0.5,
            },
        },
    );

    // Set migration costs (in developer days)
    cost_metric.set_distance(ui_concept, api_concept, 5.0); // UI to API migration
    cost_metric.set_distance(api_concept, db_concept, 10.0); // API to DB migration
    cost_metric.set_distance(db_concept, cache_concept, 3.0); // DB to Cache migration

    // Find components within budget
    let budget_radius = 8.0;
    let within_budget = cost_metric.metric_ball(ui_concept, budget_radius);
    println!(
        "   Components reachable within {} days from UI:",
        budget_radius
    );
    for concept in within_budget {
        println!("   - {:?}", concept);
    }

    // Example 3: Resource Consumption Metric
    println!("\n3. Resource Consumption Metric");
    println!("   Measuring computational resource usage");

    let mut resource_metric = MetricContext::new(
        "CPU Usage".to_string(),
        ui_concept,
        MetricType::ResourceMetric {
            resource_type: ResourceType::Computational,
            consumption_function: ConsumptionFunction::Linear { rate: 1.5 },
        },
    );

    // Set resource consumption relationships
    resource_metric.set_distance(ui_concept, api_concept, 10.0); // 10 CPU units
    resource_metric.set_distance(api_concept, db_concept, 50.0); // 50 CPU units
    resource_metric.set_distance(api_concept, cache_concept, 5.0); // 5 CPU units
    resource_metric.set_distance(cache_concept, db_concept, 45.0); // 45 CPU units

    // Cluster by resource usage similarity
    let clusters = resource_metric.cluster_by_distance(20.0);
    println!("   Resource usage clusters (threshold: 20 CPU units):");
    for (i, cluster) in clusters.iter().enumerate() {
        println!(
            "   Cluster {}: {} members, avg distance: {:.2}",
            i + 1,
            cluster.members.len(),
            cluster.average_distance
        );
    }

    // Example 4: Metric Properties
    println!("\n4. Metric Space Properties");

    // Create an asymmetric metric for data flow
    let mut data_flow_metric = MetricContext::new(
        "Data Flow Volume".to_string(),
        ui_concept,
        MetricType::TransformationCost {
            cost_function: CostFunction::Linear { rate: 1.0 },
        },
    );

    // Make it asymmetric (data flows differently in each direction)
    data_flow_metric.metric_space.is_symmetric = false;

    // UI sends little data to API, API sends lots to UI
    data_flow_metric.set_distance(ui_concept, api_concept, 10.0);
    data_flow_metric.set_distance(api_concept, ui_concept, 100.0);

    println!(
        "   Data flow from UI to API: {:?} MB/s",
        data_flow_metric.get_distance(ui_concept, api_concept)
    );
    println!(
        "   Data flow from API to UI: {:?} MB/s",
        data_flow_metric.get_distance(api_concept, ui_concept)
    );

    println!("\n=== Demo Complete ===");
}
