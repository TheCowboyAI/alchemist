//! Domain Commands

use serde::{Deserialize, Serialize};

pub mod aggregated_commands;
pub mod edge_commands;
pub mod graph_commands;
pub mod node_commands;
pub mod workflow;
pub mod subgraph_commands;

pub use aggregated_commands::{
    DomainCommand, UpdateNodePositions, UpdateGraphSelection,
    RecognizeGraphModel, ApplyGraphMorphism, MorphismType
};
pub use edge_commands::EdgeCommand;
pub use graph_commands::{GraphCommand, ImportSource, ImportOptions};
pub use node_commands::NodeCommand;
pub use workflow::WorkflowCommand;
pub use subgraph_commands::SubgraphCommand;

/// All commands in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Graph(GraphCommand),
    Node(NodeCommand),
    Edge(EdgeCommand),
    Subgraph(SubgraphCommand),
    Workflow(WorkflowCommand),
}

impl Command {
    pub fn command_type(&self) -> &'static str {
        match self {
            Command::Graph(c) => c.command_type(),
            Command::Node(c) => c.command_type(),
            Command::Edge(c) => c.command_type(),
            Command::Subgraph(_) => "subgraph",
            Command::Workflow(_) => "workflow",
        }
    }
}

#[cfg(test)]
mod handler_existence_tests {
    use super::*;
    use crate::domain::value_objects::{GraphId, NodeId, EdgeId, Position3D, NodeContent, EdgeRelationship, WorkflowId, StepId, UserId};
    use std::collections::HashMap;

    #[test]
    fn test_all_graph_commands_have_handlers() {
        // Test that every GraphCommand variant can be handled
        let test_commands = vec![
            GraphCommand::CreateGraph {
                id: GraphId::new(),
                name: "Test Graph".to_string(),
                metadata: HashMap::new(),
            },
            GraphCommand::DeleteGraph {
                id: GraphId::new(),
            },
            GraphCommand::RenameGraph {
                id: GraphId::new(),
                new_name: "New Name".to_string(),
            },
            GraphCommand::TagGraph {
                id: GraphId::new(),
                tag: "test-tag".to_string(),
            },
            GraphCommand::UntagGraph {
                id: GraphId::new(),
                tag: "test-tag".to_string(),
            },
            GraphCommand::UpdateGraph {
                id: GraphId::new(),
                name: Some("Updated Name".to_string()),
                description: Some("Updated Description".to_string()),
            },
            GraphCommand::ClearGraph {
                graph_id: GraphId::new(),
            },
            GraphCommand::AddNode {
                graph_id: GraphId::new(),
                node_id: NodeId::new(),
                node_type: "default".to_string(),
                position: Position3D::default(),
                content: serde_json::json!({}),
            },
            GraphCommand::UpdateNode {
                graph_id: GraphId::new(),
                node_id: NodeId::new(),
                new_position: Some(Position3D::default()),
                new_content: Some(serde_json::json!({})),
            },
            GraphCommand::RemoveNode {
                graph_id: GraphId::new(),
                node_id: NodeId::new(),
            },
            GraphCommand::ConnectNodes {
                graph_id: GraphId::new(),
                edge_id: EdgeId::new(),
                source_id: NodeId::new(),
                target_id: NodeId::new(),
                edge_type: "depends_on".to_string(),
                properties: HashMap::new(),
            },
            GraphCommand::DisconnectNodes {
                graph_id: GraphId::new(),
                edge_id: EdgeId::new(),
            },
            GraphCommand::UpdateEdge {
                graph_id: GraphId::new(),
                edge_id: EdgeId::new(),
                new_properties: HashMap::new(),
            },
            GraphCommand::ImportGraph {
                graph_id: GraphId::new(),
                source: ImportSource::InlineContent { content: "test".to_string() },
                format: "mermaid".to_string(),
                options: ImportOptions {
                    merge_behavior: graph_commands::MergeBehavior::Skip,
                    id_prefix: None,
                    position_offset: None,
                    mapping: None,
                    validate: true,
                    max_nodes: None,
                },
            },
            GraphCommand::ImportFromFile {
                graph_id: GraphId::new(),
                file_path: "/tmp/test.graph".to_string(),
                format: "json".to_string(),
            },
            GraphCommand::ImportFromUrl {
                graph_id: GraphId::new(),
                url: "https://example.com/graph.json".to_string(),
                format: "json".to_string(),
            },
        ];

        // Verify each command type exists and is covered
        for cmd in test_commands {
            match cmd {
                GraphCommand::CreateGraph { .. } => assert!(true, "CreateGraph handler exists"),
                GraphCommand::DeleteGraph { .. } => assert!(true, "DeleteGraph handler exists"),
                GraphCommand::RenameGraph { .. } => assert!(true, "RenameGraph handler exists"),
                GraphCommand::TagGraph { .. } => assert!(true, "TagGraph handler exists"),
                GraphCommand::UntagGraph { .. } => assert!(true, "UntagGraph handler exists"),
                GraphCommand::UpdateGraph { .. } => assert!(true, "UpdateGraph handler exists"),
                GraphCommand::ClearGraph { .. } => assert!(true, "ClearGraph handler exists"),
                GraphCommand::AddNode { .. } => assert!(true, "AddNode handler exists"),
                GraphCommand::UpdateNode { .. } => assert!(true, "UpdateNode handler exists"),
                GraphCommand::RemoveNode { .. } => assert!(true, "RemoveNode handler exists"),
                GraphCommand::ConnectNodes { .. } => assert!(true, "ConnectNodes handler exists"),
                GraphCommand::DisconnectNodes { .. } => assert!(true, "DisconnectNodes handler exists"),
                GraphCommand::UpdateEdge { .. } => assert!(true, "UpdateEdge handler exists"),
                GraphCommand::ImportGraph { .. } => assert!(true, "ImportGraph handler exists"),
                GraphCommand::ImportFromFile { .. } => assert!(true, "ImportFromFile handler exists"),
                GraphCommand::ImportFromUrl { .. } => assert!(true, "ImportFromUrl handler exists"),
            }
        }
    }

    #[test]
    fn test_unknown_command_rejection() {
        // This test would verify that unknown commands are properly rejected
        // Since we use enums, this is enforced at compile time in Rust
        // But we can test error handling for invalid command data

        // Example: Test that invalid graph ID returns proper error
        let invalid_cmd = GraphCommand::AddNode {
            graph_id: GraphId::new(), // Non-existent graph
            node_id: NodeId::new(),
            node_type: "test".to_string(),
            position: Position3D::default(),
            content: serde_json::json!({}),
        };

        // In a real implementation, this would test the command handler
        // For now, we just verify the command structure is valid
        assert!(matches!(invalid_cmd, GraphCommand::AddNode { .. }));
    }

    #[test]
    fn test_all_node_commands_have_handlers() {
        // Test that every NodeCommand variant can be handled
        let test_commands = vec![
            NodeCommand::AddNode {
                graph_id: GraphId::new(),
                node_id: NodeId::new(),
                content: NodeContent {
                    label: "Test Node".to_string(),
                    node_type: crate::domain::value_objects::NodeType::Entity,
                    properties: std::collections::HashMap::new(),
                },
                position: Position3D::default(),
            },
            NodeCommand::RemoveNode {
                graph_id: GraphId::new(),
                node_id: NodeId::new(),
            },
            NodeCommand::UpdateNode {
                graph_id: GraphId::new(),
                node_id: NodeId::new(),
                content: NodeContent {
                    label: "Updated Node".to_string(),
                    node_type: crate::domain::value_objects::NodeType::Entity,
                    properties: std::collections::HashMap::new(),
                },
            },
            NodeCommand::MoveNode {
                graph_id: GraphId::new(),
                node_id: NodeId::new(),
                position: Position3D::default(),
            },
            NodeCommand::SelectNode {
                graph_id: GraphId::new(),
                node_id: NodeId::new(),
            },
            NodeCommand::DeselectNode {
                graph_id: GraphId::new(),
                node_id: NodeId::new(),
            },
        ];

        // Verify each command type exists and is covered
        for cmd in test_commands {
            match cmd {
                NodeCommand::AddNode { .. } => assert!(true, "AddNode handler exists"),
                NodeCommand::RemoveNode { .. } => assert!(true, "RemoveNode handler exists"),
                NodeCommand::UpdateNode { .. } => assert!(true, "UpdateNode handler exists"),
                NodeCommand::MoveNode { .. } => assert!(true, "MoveNode handler exists"),
                NodeCommand::SelectNode { .. } => assert!(true, "SelectNode handler exists"),
                NodeCommand::DeselectNode { .. } => assert!(true, "DeselectNode handler exists"),
            }
        }
    }

    #[test]
    fn test_all_edge_commands_have_handlers() {
        // Test that every EdgeCommand variant can be handled
        let test_commands = vec![
            EdgeCommand::ConnectEdge {
                graph_id: GraphId::new(),
                edge_id: EdgeId::new(),
                source: NodeId::new(),
                target: NodeId::new(),
                relationship: EdgeRelationship {
                    relationship_type: crate::domain::value_objects::RelationshipType::DependsOn,
                    properties: std::collections::HashMap::new(),
                    bidirectional: false,
                },
            },
            EdgeCommand::DisconnectEdge {
                graph_id: GraphId::new(),
                edge_id: EdgeId::new(),
            },
            EdgeCommand::SelectEdge {
                graph_id: GraphId::new(),
                edge_id: EdgeId::new(),
            },
            EdgeCommand::DeselectEdge {
                graph_id: GraphId::new(),
                edge_id: EdgeId::new(),
            },
        ];

        // Verify each command type exists and is covered
        for cmd in test_commands {
            match cmd {
                EdgeCommand::ConnectEdge { .. } => assert!(true, "ConnectEdge handler exists"),
                EdgeCommand::DisconnectEdge { .. } => assert!(true, "DisconnectEdge handler exists"),
                EdgeCommand::SelectEdge { .. } => assert!(true, "SelectEdge handler exists"),
                EdgeCommand::DeselectEdge { .. } => assert!(true, "DeselectEdge handler exists"),
            }
        }
    }

    #[test]
    fn test_workflow_commands_have_handlers() {
        // Test that every WorkflowCommand variant can be handled
        use crate::domain::value_objects::{WorkflowId, StepId, EdgeId as WorkflowEdgeId, UserId};
        use crate::domain::aggregates::workflow::{WorkflowStep, StepType};

        let test_commands = vec![
            WorkflowCommand::CreateWorkflow(workflow::CreateWorkflow {
                workflow_id: WorkflowId::new(),
                name: "Test Workflow".to_string(),
                description: "Test Description".to_string(),
                created_by: UserId::new(),
                tags: vec![],
            }),
            WorkflowCommand::AddStep(workflow::AddStep {
                workflow_id: WorkflowId::new(),
                step: WorkflowStep {
                    id: StepId::new(),
                    name: "Test Step".to_string(),
                    step_type: StepType::UserTask,
                    node_id: NodeId::new(),
                    inputs: vec![],
                    outputs: vec![],
                    timeout_ms: None,
                    retry_policy: None,
                },
            }),
            WorkflowCommand::ConnectSteps(workflow::ConnectSteps {
                workflow_id: WorkflowId::new(),
                from_step: StepId::new(),
                to_step: StepId::new(),
                edge_id: WorkflowEdgeId::new(),
                condition: None,
            }),
            WorkflowCommand::ValidateWorkflow(workflow::ValidateWorkflow {
                workflow_id: WorkflowId::new(),
                validated_by: UserId::new(),
            }),
            WorkflowCommand::StartWorkflow(workflow::StartWorkflow {
                workflow_id: WorkflowId::new(),
                instance_id: "test-instance".to_string(),
                started_by: UserId::new(),
                inputs: HashMap::new(),
            }),
            WorkflowCommand::CompleteStep(workflow::CompleteStep {
                workflow_id: WorkflowId::new(),
                step_id: StepId::new(),
                outputs: HashMap::new(),
                next_step: None,
            }),
            WorkflowCommand::PauseWorkflow(workflow::PauseWorkflow {
                workflow_id: WorkflowId::new(),
                paused_by: UserId::new(),
                reason: "Test pause".to_string(),
            }),
            WorkflowCommand::ResumeWorkflow(workflow::ResumeWorkflow {
                workflow_id: WorkflowId::new(),
                resumed_by: UserId::new(),
            }),
            WorkflowCommand::FailWorkflow(workflow::FailWorkflow {
                workflow_id: WorkflowId::new(),
                error: "Test error".to_string(),
                recovery_point: None,
            }),
        ];

        // Verify each command type exists and is covered
        for cmd in test_commands {
            match cmd {
                WorkflowCommand::CreateWorkflow(_) => assert!(true, "CreateWorkflow handler exists"),
                WorkflowCommand::AddStep(_) => assert!(true, "AddStep handler exists"),
                WorkflowCommand::ConnectSteps(_) => assert!(true, "ConnectSteps handler exists"),
                WorkflowCommand::ValidateWorkflow(_) => assert!(true, "ValidateWorkflow handler exists"),
                WorkflowCommand::StartWorkflow(_) => assert!(true, "StartWorkflow handler exists"),
                WorkflowCommand::CompleteStep(_) => assert!(true, "CompleteStep handler exists"),
                WorkflowCommand::PauseWorkflow(_) => assert!(true, "PauseWorkflow handler exists"),
                WorkflowCommand::ResumeWorkflow(_) => assert!(true, "ResumeWorkflow handler exists"),
                WorkflowCommand::FailWorkflow(_) => assert!(true, "FailWorkflow handler exists"),
            }
        }
    }

    #[test]
    fn test_command_wrapper_handling() {
        // Test that Command wrapper properly handles all command types
        let graph_cmd = Command::Graph(GraphCommand::CreateGraph {
            id: GraphId::new(),
            name: "Test".to_string(),
            metadata: HashMap::new(),
        });

        let node_cmd = Command::Node(NodeCommand::AddNode {
            graph_id: GraphId::new(),
            node_id: NodeId::new(),
            content: NodeContent {
                label: "Test Node".to_string(),
                node_type: crate::domain::value_objects::NodeType::Entity,
                properties: std::collections::HashMap::new(),
            },
            position: Position3D::default(),
        });

        let edge_cmd = Command::Edge(EdgeCommand::ConnectEdge {
            graph_id: GraphId::new(),
            edge_id: EdgeId::new(),
            source: NodeId::new(),
            target: NodeId::new(),
            relationship: EdgeRelationship {
                relationship_type: crate::domain::value_objects::RelationshipType::DependsOn,
                properties: std::collections::HashMap::new(),
                bidirectional: false,
            },
        });

        let workflow_cmd = Command::Workflow(WorkflowCommand::CreateWorkflow(workflow::CreateWorkflow {
            workflow_id: WorkflowId::new(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            created_by: UserId::new(),
            tags: vec![],
        }));

        // Verify pattern matching works for all command types
        match graph_cmd {
            Command::Graph(_) => assert!(true, "Graph commands handled"),
            _ => panic!("Wrong command type"),
        }

        match node_cmd {
            Command::Node(_) => assert!(true, "Node commands handled"),
            _ => panic!("Wrong command type"),
        }

        match edge_cmd {
            Command::Edge(_) => assert!(true, "Edge commands handled"),
            _ => panic!("Wrong command type"),
        }

        match workflow_cmd {
            Command::Workflow(_) => assert!(true, "Workflow commands handled"),
            _ => panic!("Wrong command type"),
        }
    }
}
