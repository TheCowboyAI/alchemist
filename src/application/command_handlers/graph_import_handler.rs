use crate::domain::{
    commands::{ImportSource, ImportOptions},
    events::{DomainEvent, GraphEvent, NodeEvent, EdgeEvent},
    services::GraphImportService,
    value_objects::{GraphId, NodeId, EdgeId},
    DomainError,
};
use tracing::info;

/// Handles graph import commands
pub struct GraphImportHandler;

impl GraphImportHandler {
    /// Process a graph import request
    pub async fn handle_import_request(
        &self,
        graph_id: GraphId,
        source: ImportSource,
        options: ImportOptions,
    ) -> Result<Vec<DomainEvent>, DomainError> {
        let import_service = GraphImportService::new();
        let mut events = Vec::new();

        // Import based on source
        let imported_graphs = match source.clone() {
            ImportSource::File { path } => {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| DomainError::ValidationFailed(format!("Failed to read file: {e}")))?;

                let format = import_service.detect_format(&content);
                vec![import_service.import_from_content(&content, format, options.mapping.as_ref())?]
            }
            ImportSource::Url { url } => {
                // TODO: Implement URL fetching
                return Err(DomainError::ValidationFailed("URL import not yet implemented".to_string()));
            }
            ImportSource::GitRepository { url, branch, path } => {
                self.import_from_git_repo(&url, branch.as_deref(), &path, &options).await?
            }
            ImportSource::NixFlake { flake_ref, output } => {
                self.import_from_nix_flake(&flake_ref, &output, &options).await?
            }
            ImportSource::InlineContent { content } => {
                let format = import_service.detect_format(&content);
                vec![import_service.import_from_content(&content, format, options.mapping.as_ref())?]
            }
        };

        // Convert imported graphs to domain events
        for imported_graph in imported_graphs {
            // Generate events for nodes
            for node in imported_graph.nodes {
                let node_id = if let Some(prefix) = &options.id_prefix {
                    // For imported nodes with prefix, we need to maintain the original ID
                    // This is a limitation - NodeId is typically a UUID
                    NodeId::new()
                } else {
                    NodeId::new()
                };

                let mut position = node.position;
                if let Some(offset) = &options.position_offset {
                    position.x += offset.x;
                    position.y += offset.y;
                    position.z += offset.z;
                }

                events.push(DomainEvent::Node(NodeEvent::NodeAdded {
                    graph_id,
                    node_id,
                    metadata: node.properties.clone(),
                    position,
                }));
            }

            // Generate events for edges
            for edge in imported_graph.edges {
                let edge_id = EdgeId::new();
                let source_id = NodeId::new(); // TODO: Need to map from imported node IDs
                let target_id = NodeId::new(); // TODO: Need to map from imported node IDs

                events.push(DomainEvent::Edge(EdgeEvent::EdgeConnected {
                    graph_id,
                    edge_id,
                    source: source_id,
                    target: target_id,
                    relationship: edge.edge_type,
                }));
            }
        }

        // Add import completed event
        events.push(DomainEvent::Graph(GraphEvent::GraphImportCompleted {
            graph_id,
            imported_nodes: events.iter().filter(|e| matches!(e, DomainEvent::Node(_))).count(),
            imported_edges: events.iter().filter(|e| matches!(e, DomainEvent::Edge(_))).count(),
            source,
        }));

        Ok(events)
    }

    async fn import_from_git_repo(
        &self,
        repo_url: &str,
        branch: Option<&str>,
        path: &str,
        options: &ImportOptions,
    ) -> Result<Vec<crate::domain::services::graph_import::ImportedGraph>, DomainError> {
        // TODO: Implement git repository cloning and import
        // For now, return empty vector
        info!("Git repository import requested: {} (branch: {:?}, path: {})", repo_url, branch, path);
        Ok(Vec::new())
    }

    async fn import_from_nix_flake(
        &self,
        flake_ref: &str,
        output: &str,
        options: &ImportOptions,
    ) -> Result<Vec<crate::domain::services::graph_import::ImportedGraph>, DomainError> {
        // TODO: Implement nix flake evaluation and import
        // For now, return empty vector
        info!("Nix flake import requested: {} (output: {})", flake_ref, output);
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        commands::{Command, GraphCommand, ImportSource, ImportOptions},
        commands::graph_commands::MergeBehavior,
        events::{DomainEvent, GraphEvent},
        value_objects::GraphId,
    };

    #[test]
    fn test_import_graph_command_returns_none() {
        // This test documents that ImportGraph commands are not handled
        let graph_id = GraphId::new();
        let cmd = Command::Graph(GraphCommand::ImportGraph {
            graph_id,
            source: ImportSource::File {
                path: "test.json".to_string(),
            },
            format: "arrows_app".to_string(),
            options: ImportOptions {
                merge_behavior: MergeBehavior::AlwaysCreate,
                id_prefix: None,
                position_offset: None,
                mapping: None,
                validate: true,
                max_nodes: None,
            },
        });

        // The handler returns None for ImportGraph commands
        // This is the bug - it should process the import
        if let Command::Graph(graph_cmd) = &cmd {
            let result = super::super::handle_graph_command(graph_cmd);
            assert!(result.is_none(), "ImportGraph commands are not handled - they return None");
        } else {
            panic!("Expected a Graph command");
        }
    }

    #[test]
    fn test_graph_import_requested_event_not_processed() {
        // This test shows that even if we had a GraphImportRequested event,
        // there's no system to process it

        // TODO: There should be a system that:
        // 1. Listens for GraphImportRequested events
        // 2. Reads the file content
        // 3. Calls GraphImportService to parse it
        // 4. Generates NodeAdded and EdgeConnected events

        // For now, this documents the missing functionality
        panic!("No event handler exists for GraphImportRequested events!");
    }
}
