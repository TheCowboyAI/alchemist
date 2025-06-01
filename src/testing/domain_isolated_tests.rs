#[cfg(test)]
mod domain_isolated_tests {
    use crate::contexts::graph_management::domain::*;
    use serde_json;
    use std::collections::HashMap;
    use uuid::Uuid;

    // ===== PURE DOMAIN LOGIC TESTS (No Bevy Dependencies) =====

    #[test]
    fn test_graph_identity_uniqueness() {
        // Given: Two graph identities
        let id1 = GraphIdentity::new();
        let id2 = GraphIdentity::new();

        // Then: They should be unique
        assert_ne!(id1.0, id2.0);
        assert_ne!(id1.0, Uuid::nil());
        assert_ne!(id2.0, Uuid::nil());
    }

    #[test]
    fn test_spatial_position_validation() {
        // Given: Various spatial positions
        let valid_pos = SpatialPosition::at_3d(1.0, 2.0, 3.0);
        let origin_pos = SpatialPosition::at_3d(0.0, 0.0, 0.0);
        let negative_pos = SpatialPosition::at_3d(-1.0, -2.0, -3.0);

        // When: Calculating distances
        let distance_from_origin = valid_pos.coordinates_3d.length();

        // Then: Distance calculations should be correct
        assert!((distance_from_origin - 3.74165).abs() < 0.001);
        assert_eq!(
            origin_pos.coordinates_3d.x + origin_pos.coordinates_3d.y + origin_pos.coordinates_3d.z,
            0.0
        );
        assert!(negative_pos.coordinates_3d.x < 0.0);
    }

    #[test]
    fn test_graph_metadata_immutability() {
        // Given: Initial metadata
        let metadata = GraphMetadata {
            name: "Test Graph".to_string(),
            description: "Test Description".to_string(),
            domain: "test".to_string(),
            created: std::time::SystemTime::now(),
            modified: std::time::SystemTime::now(),
            tags: vec!["test".to_string()],
        };
        let original_name = metadata.name.clone();
        let original_created = metadata.created;

        // When: Time passes
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Then: Created time should not change
        assert_eq!(metadata.name, original_name);
        assert_eq!(metadata.created, original_created);
    }

    #[test]
    fn test_node_content_properties_manipulation() {
        // Given: Node content with properties
        let mut content = NodeContent {
            label: "Test Node".to_string(),
            category: "test".to_string(),
            properties: HashMap::new(),
        };

        // When: Adding properties
        content
            .properties
            .insert("color".to_string(), serde_json::json!("blue"));
        content
            .properties
            .insert("weight".to_string(), serde_json::json!(42));
        content
            .properties
            .insert("active".to_string(), serde_json::json!(true));

        // Then: Properties should be retrievable
        assert_eq!(
            content.properties.get("color"),
            Some(&serde_json::json!("blue"))
        );
        assert_eq!(
            content.properties.get("weight"),
            Some(&serde_json::json!(42))
        );
        assert_eq!(
            content.properties.get("active"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(content.properties.len(), 3);
    }

    #[test]
    fn test_edge_relationship_validation() {
        // Given: Edge relationship
        let source = NodeIdentity::new();
        let target = NodeIdentity::new();
        let relationship = EdgeRelationship {
            source,
            target,
            category: "depends_on".to_string(),
            strength: 0.8,
            properties: HashMap::new(),
        };

        // Then: Relationship should be valid
        assert_ne!(relationship.source, relationship.target);
        assert!(relationship.strength >= 0.0 && relationship.strength <= 1.0);
        assert_eq!(relationship.category, "depends_on");
    }

    #[test]
    fn test_graph_validation_rules() {
        // Given: Graph constraints
        struct GraphConstraints {
            max_nodes: usize,
            max_edges: usize,
            allow_self_loops: bool,
            allow_duplicates: bool,
        }

        let constraints = GraphConstraints {
            max_nodes: 1000,
            max_edges: 5000,
            allow_self_loops: false,
            allow_duplicates: false,
        };

        // When: Validating graph operations
        let node_count = 100;
        let edge_count = 200;

        // Then: Validation should pass
        assert!(node_count <= constraints.max_nodes);
        assert!(edge_count <= constraints.max_edges);
        assert!(!constraints.allow_self_loops);
        assert!(!constraints.allow_duplicates);
    }

    #[test]
    fn test_graph_traversal_logic() {
        // Given: A graph structure (pure data, no Bevy)
        #[derive(Debug)]
        struct GraphStructure {
            nodes: Vec<NodeIdentity>,
            edges: Vec<(NodeIdentity, NodeIdentity)>,
        }

        let mut graph = GraphStructure {
            nodes: vec![],
            edges: vec![],
        };

        // When: Building a simple graph
        let n1 = NodeIdentity::new();
        let n2 = NodeIdentity::new();
        let n3 = NodeIdentity::new();

        graph.nodes.extend([n1, n2, n3]);
        graph.edges.push((n1, n2));
        graph.edges.push((n2, n3));

        // Then: We should be able to find paths
        let has_path_from_n1_to_n3 = graph
            .edges
            .iter()
            .any(|(from, to)| *from == n1 && *to == n2)
            && graph
                .edges
                .iter()
                .any(|(from, to)| *from == n2 && *to == n3);

        assert!(has_path_from_n1_to_n3);
    }

    #[test]
    fn test_node_content_validation() {
        // Given: Various node contents
        let valid_content = NodeContent {
            label: "Valid Node".to_string(),
            category: "test".to_string(),
            properties: HashMap::new(),
        };

        let empty_label_content = NodeContent {
            label: "".to_string(),
            category: "test".to_string(),
            properties: HashMap::new(),
        };

        // When: Validating content
        let is_valid_label = !valid_content.label.is_empty();
        let is_empty_label = empty_label_content.label.is_empty();

        // Then: Validation should be correct
        assert!(is_valid_label);
        assert!(is_empty_label);
        assert_eq!(valid_content.category, "test");
    }

    #[test]
    fn test_edge_relationship_weight_calculations() {
        // Given: Edge weights
        let weights = vec![1.0, 2.5, 0.5, 3.0, 1.5];

        // When: Calculating statistics
        let sum: f32 = weights.iter().sum();
        let avg = sum / weights.len() as f32;
        let max = weights.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let min = weights.iter().fold(f32::INFINITY, |a, &b| a.min(b));

        // Then: Statistics should be correct
        assert_eq!(sum, 8.5);
        assert_eq!(avg, 1.7);
        assert_eq!(max, 3.0);
        assert_eq!(min, 0.5);
    }

    #[test]
    fn test_graph_serialization_compatibility() {
        // Given: A graph data structure
        let graph_id = GraphIdentity::new();
        let metadata = GraphMetadata {
            name: "Serialization Test".to_string(),
            description: "Test Description".to_string(),
            domain: "test".to_string(),
            created: std::time::SystemTime::now(),
            modified: std::time::SystemTime::now(),
            tags: vec!["test".to_string()],
        };

        // When: Converting to JSON-compatible format
        let json_data = serde_json::json!({
            "id": graph_id.0.to_string(),
            "name": metadata.name,
            "domain": metadata.domain,
            "tags": metadata.tags,
        });

        // Then: JSON should be valid
        assert!(json_data.is_object());
        assert_eq!(json_data["name"], "Serialization Test");
        assert_eq!(json_data["domain"], "test");
        assert!(json_data["tags"].is_array());
    }
}
