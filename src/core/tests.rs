#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::test_config::{create_test_app, run_test_cycle};
    use bevy::prelude::*;

    #[test]
    fn test_node_creation() {
        let mut app = create_test_app();

        // Add our node system
        app.add_systems(Update, |mut commands: Commands| {
            commands.spawn(Node {
                id: NodeId::new(),
                label: "Test Node".to_string(),
                node_type: NodeType::Process,
                position: Vec2::new(100.0, 100.0),
            });
        });

        run_test_cycle(&mut app);

        // Query for the node
        let mut query = app.world_mut().query::<&Node>();
        let nodes: Vec<_> = query.iter(&app.world()).collect();

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].label, "Test Node");
    }

    #[test]
    fn test_edge_creation() {
        let mut app = create_test_app();

        let node1_id = NodeId::new();
        let node2_id = NodeId::new();

        // Spawn nodes and edge
        app.world_mut().spawn((Node {
            id: node1_id,
            label: "Node 1".to_string(),
            node_type: NodeType::Input,
            position: Vec2::ZERO,
        },));

        app.world_mut().spawn((Node {
            id: node2_id,
            label: "Node 2".to_string(),
            node_type: NodeType::Output,
            position: Vec2::new(200.0, 0.0),
        },));

        app.world_mut().spawn((Edge {
            id: EdgeId::new(),
            source: node1_id,
            target: node2_id,
            edge_type: EdgeType::DataFlow,
        },));

        run_test_cycle(&mut app);

        // Verify edge exists
        let mut edge_query = app.world_mut().query::<&Edge>();
        let edges: Vec<_> = edge_query.iter(&app.world()).collect();

        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].source, node1_id);
        assert_eq!(edges[0].target, node2_id);
    }

    #[test]
    fn test_graph_component() {
        let mut app = create_test_app();

        // Create a graph
        app.world_mut().spawn(Graph {
            id: GraphId::new(),
            name: "Test Graph".to_string(),
            description: Some("A test graph".to_string()),
        });

        run_test_cycle(&mut app);

        // Query for the graph
        let mut query = app.world_mut().query::<&Graph>();
        let graphs: Vec<_> = query.iter(&app.world()).collect();

        assert_eq!(graphs.len(), 1);
        assert_eq!(graphs[0].name, "Test Graph");
    }
}
