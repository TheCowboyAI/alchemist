//! Event Validation Helpers for Comprehensive Testing
//! These helpers ensure all event fields are properly tested

use crate::contexts::graph_management::domain::*;
use crate::contexts::graph_management::events::*;
use std::collections::HashMap;

/// Trait for validating events have all required fields populated
pub trait EventValidation {
    fn validate_fields(&self) -> Result<(), String>;
}

impl EventValidation for GraphCreated {
    fn validate_fields(&self) -> Result<(), String> {
        // Validate graph identity
        if self.graph.0 == uuid::Uuid::nil() {
            return Err("Graph identity is nil UUID".to_string());
        }

        // Validate metadata
        if self.metadata.name.is_empty() {
            return Err("Graph name is empty".to_string());
        }
        if self.metadata.domain.is_empty() {
            return Err("Graph domain is empty".to_string());
        }

        // Validate timestamp
        let now = std::time::SystemTime::now();
        if self.timestamp > now {
            return Err("Timestamp is in the future".to_string());
        }

        // Check that created <= modified
        if self.metadata.created > self.metadata.modified {
            return Err("Created time is after modified time".to_string());
        }

        Ok(())
    }
}

impl EventValidation for NodeAdded {
    fn validate_fields(&self) -> Result<(), String> {
        // Validate identities
        if self.graph.0 == uuid::Uuid::nil() {
            return Err("Graph identity is nil UUID".to_string());
        }
        if self.node.0 == uuid::Uuid::nil() {
            return Err("Node identity is nil UUID".to_string());
        }

        // Validate content
        if self.content.label.is_empty() {
            return Err("Node label is empty".to_string());
        }
        if self.content.category.is_empty() {
            return Err("Node category is empty".to_string());
        }

        // Validate position
        if self.position.coordinates_3d.x.is_nan()
            || self.position.coordinates_3d.y.is_nan()
            || self.position.coordinates_3d.z.is_nan()
        {
            return Err("Position contains NaN values".to_string());
        }

        Ok(())
    }
}

impl EventValidation for EdgeConnected {
    fn validate_fields(&self) -> Result<(), String> {
        // Validate identities
        if self.graph.0 == uuid::Uuid::nil() {
            return Err("Graph identity is nil UUID".to_string());
        }
        if self.edge.0 == uuid::Uuid::nil() {
            return Err("Edge identity is nil UUID".to_string());
        }

        // Validate relationship
        if self.relationship.source.0 == uuid::Uuid::nil() {
            return Err("Source node identity is nil UUID".to_string());
        }
        if self.relationship.target.0 == uuid::Uuid::nil() {
            return Err("Target node identity is nil UUID".to_string());
        }
        if self.relationship.category.is_empty() {
            return Err("Edge category is empty".to_string());
        }
        if self.relationship.strength < 0.0 || self.relationship.strength > 1.0 {
            return Err(format!(
                "Edge strength {} is outside valid range [0.0, 1.0]",
                self.relationship.strength
            ));
        }

        Ok(())
    }
}

impl EventValidation for NodeMoved {
    fn validate_fields(&self) -> Result<(), String> {
        // Validate identities
        if self.graph.0 == uuid::Uuid::nil() {
            return Err("Graph identity is nil UUID".to_string());
        }
        if self.node.0 == uuid::Uuid::nil() {
            return Err("Node identity is nil UUID".to_string());
        }

        // Validate positions
        for (name, pos) in [("from", &self.from_position), ("to", &self.to_position)] {
            if pos.coordinates_3d.x.is_nan()
                || pos.coordinates_3d.y.is_nan()
                || pos.coordinates_3d.z.is_nan()
            {
                return Err(format!("{name} position contains NaN values"));
            }
        }

        // Validate that positions are different
        if self.from_position.coordinates_3d == self.to_position.coordinates_3d {
            return Err("From and to positions are identical".to_string());
        }

        Ok(())
    }
}

impl EventValidation for PropertyUpdated {
    fn validate_fields(&self) -> Result<(), String> {
        // Validate identities
        if self.graph.0 == uuid::Uuid::nil() {
            return Err("Graph identity is nil UUID".to_string());
        }
        if self.element_id == uuid::Uuid::nil() {
            return Err("Element identity is nil UUID".to_string());
        }

        // Validate property key
        if self.property_key.is_empty() {
            return Err("Property key is empty".to_string());
        }

        // Validate that old and new values are different
        if let Some(old) = &self.old_value {
            if old == &self.new_value {
                return Err("Old and new values are identical".to_string());
            }
        }

        Ok(())
    }
}

impl EventValidation for GraphDeleted {
    fn validate_fields(&self) -> Result<(), String> {
        // Validate identity
        if self.graph.0 == uuid::Uuid::nil() {
            return Err("Graph identity is nil UUID".to_string());
        }

        // Validate reason (no additional validation needed for enum)
        Ok(())
    }
}

/// Helper to assert an event passes all field validations
#[macro_export]
macro_rules! assert_event_valid {
    ($event:expr) => {
        match $event.validate_fields() {
            Ok(()) => {}
            Err(e) => panic!("Event validation failed: {}", e),
        }
    };
}

/// Helper to create test events with all fields properly populated
pub struct TestEventBuilder;

impl TestEventBuilder {
    pub fn graph_created(name: &str) -> GraphCreated {
        let now = std::time::SystemTime::now();
        GraphCreated {
            graph: GraphIdentity::new(),
            metadata: GraphMetadata {
                name: name.to_string(),
                description: "Test graph description".to_string(),
                domain: "test-domain".to_string(),
                created: now,
                modified: now,
                tags: vec!["test".to_string(), "automated".to_string()],
            },
            timestamp: now,
        }
    }

    pub fn node_added(graph_id: GraphIdentity, label: &str) -> NodeAdded {
        NodeAdded {
            graph: graph_id,
            node: NodeIdentity::new(),
            content: NodeContent {
                label: label.to_string(),
                category: "test-category".to_string(),
                properties: HashMap::from([
                    ("created_by".to_string(), serde_json::json!("test")),
                    ("version".to_string(), serde_json::json!(1)),
                ]),
            },
            position: SpatialPosition::at_3d(10.0, 20.0, 0.0),
        }
    }

    pub fn edge_connected(
        graph_id: GraphIdentity,
        source: NodeIdentity,
        target: NodeIdentity,
    ) -> EdgeConnected {
        EdgeConnected {
            graph: graph_id,
            edge: EdgeIdentity::new(),
            relationship: EdgeRelationship {
                source,
                target,
                category: "test-relationship".to_string(),
                strength: 0.75,
                properties: HashMap::from([("weight".to_string(), serde_json::json!(1.0))]),
            },
        }
    }

    pub fn node_moved(
        graph_id: GraphIdentity,
        node_id: NodeIdentity,
        from: (f32, f32, f32),
        to: (f32, f32, f32),
    ) -> NodeMoved {
        NodeMoved {
            graph: graph_id,
            node: node_id,
            from_position: SpatialPosition::at_3d(from.0, from.1, from.2),
            to_position: SpatialPosition::at_3d(to.0, to.1, to.2),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_created_validation() {
        // Valid event
        let event = TestEventBuilder::graph_created("Test Graph");
        assert!(event.validate_fields().is_ok());

        // Invalid - empty name
        let mut invalid = event.clone();
        invalid.metadata.name = String::new();
        assert!(invalid.validate_fields().is_err());

        // Invalid - nil UUID
        let mut invalid = event.clone();
        invalid.graph = GraphIdentity(uuid::Uuid::nil());
        assert!(invalid.validate_fields().is_err());
    }

    #[test]
    fn test_node_added_validation() {
        let graph_id = GraphIdentity::new();
        let event = TestEventBuilder::node_added(graph_id, "Test Node");
        assert!(event.validate_fields().is_ok());

        // Invalid - empty label
        let mut invalid = event.clone();
        invalid.content.label = String::new();
        assert!(invalid.validate_fields().is_err());

        // Invalid - NaN position
        let mut invalid = event.clone();
        invalid.position.coordinates_3d.x = f32::NAN;
        assert!(invalid.validate_fields().is_err());
    }

    #[test]
    fn test_edge_connected_validation() {
        let graph_id = GraphIdentity::new();
        let source = NodeIdentity::new();
        let target = NodeIdentity::new();
        let event = TestEventBuilder::edge_connected(graph_id, source, target);
        assert!(event.validate_fields().is_ok());

        // Invalid - strength out of range
        let mut invalid = event.clone();
        invalid.relationship.strength = 1.5;
        assert!(invalid.validate_fields().is_err());

        // Invalid - empty category
        let mut invalid = event.clone();
        invalid.relationship.category = String::new();
        assert!(invalid.validate_fields().is_err());
    }
}
