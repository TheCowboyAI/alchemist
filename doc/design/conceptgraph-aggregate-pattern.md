# ConceptGraph as DDD Aggregate

## Overview

A **ConceptGraph** is a DDD Aggregate that composes multiple ContextGraphs into a cohesive concept with:
- A **root ContextGraph** serving as the aggregate root
- **Member ContextGraphs** as aggregate members
- **Invariants** maintained across all members
- **Conceptual positioning** in quality space

## Aggregate Structure

```rust
pub struct ConceptGraph {
    /// The aggregate root - a ContextGraph that represents the concept
    pub root: ContextGraph<String, String>,

    /// Member graphs that are part of this concept
    pub members: HashMap<ContextGraphId, Box<dyn Any>>, // ContextGraph<?, ?>

    /// Invariants that must hold across all members
    pub invariants: Vec<ConceptGraphInvariant>,

    /// Conceptual space positioning (via root components)
    // ConceptualSpace component attached to root
    // Morphisms component attached to root
}
```

## DDD Principles Applied

### 1. Aggregate Root
- Only the root ContextGraph is referenced from outside
- All modifications go through the root
- The root maintains references to all members

### 2. Consistency Boundary
- Invariants are enforced across all member graphs
- No member can violate aggregate-level rules
- Consistency is transactional within the aggregate

### 3. Identity
- The ConceptGraph has identity through its root's ContextGraphId
- Members maintain their own identities but are accessed through the root

### 4. Lifecycle
- Members can only be added/removed through aggregate operations
- The aggregate controls the lifecycle of its members
- Deletion of root deletes all members

## Example: User Management Concept

```rust
// Create a ConceptGraph for User Management
let mut user_management = ConceptGraph::new("UserManagement");

// The root graph represents the overall concept
user_management.root.add_node("concept".to_string());
user_management.root.get_node_mut(concept_node)?
    .components.add(ConceptualSpace {
        quality_dimensions: vec![
            QualityDimension::new("complexity", 0.0..10.0),
            QualityDimension::new("security", 0.0..10.0),
        ],
        position: ConceptualPoint::new(vec![5.0, 8.0]),
        category: CategoryType::Domain,
    })?;

// Add User entity as a member graph
let mut user_entity = ContextGraph::<String, String>::new("User");
user_entity.add_node("id".to_string());
user_entity.add_node("email".to_string());
user_entity.add_node("password_hash".to_string());
user_management.add_member(user_entity)?;

// Add Role entity as a member graph
let mut role_entity = ContextGraph::<String, String>::new("Role");
role_entity.add_node("id".to_string());
role_entity.add_node("name".to_string());
role_entity.add_node("permissions".to_string());
user_management.add_member(role_entity)?;

// Add UserRole relationship as a member graph
let mut user_role = ContextGraph::<String, String>::new("UserRole");
user_role.add_node("user_id".to_string());
user_role.add_node("role_id".to_string());
user_role.add_edge(user_id_node, role_id_node, "has_role".to_string())?;
user_management.add_member(user_role)?;

// Add invariants
user_management.add_invariant(UniqueUserEmail);
user_management.add_invariant(ValidRoleHierarchy);
user_management.add_invariant(AtLeastOneAdminRole);
```

## Operations Through Aggregate Root

```rust
impl ConceptGraph {
    /// Add a member graph - validates invariants
    pub fn add_member<N, E>(&mut self, graph: ContextGraph<N, E>) -> Result<()> {
        // Check invariants before adding
        for invariant in &self.invariants {
            invariant.validate_addition(&self, &graph)?;
        }

        // Add to members collection
        let graph_id = graph.id;
        self.members.insert(graph_id, Box::new(graph));

        // Add reference node in root
        let ref_node = self.root.add_node(format!("member:{}", graph_id));
        self.root.get_node_mut(ref_node)?
            .components.add(GraphReference(graph_id))?;

        // Connect to concept node
        let concept_node = self.root.nodes.values()
            .find(|n| n.value == "concept")
            .map(|n| n.id)
            .ok_or(Error::NoConceptNode)?;

        self.root.add_edge(concept_node, ref_node, "contains".to_string())?;

        Ok(())
    }

    /// Remove a member - maintains consistency
    pub fn remove_member(&mut self, graph_id: ContextGraphId) -> Result<()> {
        // Check if removal would violate invariants
        for invariant in &self.invariants {
            invariant.validate_removal(&self, graph_id)?;
        }

        // Remove from members
        self.members.remove(&graph_id);

        // Remove reference from root
        let ref_node = self.root.nodes.values()
            .find(|n| {
                n.components.get::<GraphReference>()
                    .map(|r| r.0 == graph_id)
                    .unwrap_or(false)
            })
            .map(|n| n.id);

        if let Some(node_id) = ref_node {
            self.root.remove_node(node_id);
        }

        Ok(())
    }
}
```

## Invariant Examples

```rust
/// Ensure email uniqueness across all User entities in the concept
struct UniqueUserEmail;

impl ConceptGraphInvariant for UniqueUserEmail {
    fn validate_addition(&self, concept: &ConceptGraph, new_graph: &dyn Any) -> Result<()> {
        // Check if new graph is a User entity
        if let Some(user) = new_graph.downcast_ref::<ContextGraph<String, String>>() {
            if user.metadata.properties.get("type") == Some(&json!("User")) {
                // Check email uniqueness across all existing User graphs
                // ...
            }
        }
        Ok(())
    }
}

/// Ensure role hierarchy is valid (no cycles)
struct ValidRoleHierarchy;

/// Ensure at least one admin role exists
struct AtLeastOneAdminRole;
```

## Benefits of This Pattern

1. **Consistency**: All related graphs maintain consistency through aggregate
2. **Encapsulation**: Internal structure hidden, only root exposed
3. **Transactionality**: Changes are atomic at aggregate level
4. **Flexibility**: Any combination of ContextGraphs can form a concept
5. **Type Safety**: Each member graph maintains its own type parameters
6. **Conceptual Clarity**: Maps directly to DDD aggregate pattern

## Relationship to ContextGraph

- **ContextGraph**: The building block - can represent anything
- **ConceptGraph**: The aggregate - composes ContextGraphs into concepts
- **Components**: Provide metadata and behavior without changing core types
- **Recursion**: ConceptGraphs can contain other ConceptGraphs as members

This creates a fractal structure where concepts at any level follow the same pattern.
