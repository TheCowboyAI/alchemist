//! Conceptual Space Command Handler

use crate::domain::{
    aggregates::conceptual_space::ConceptualSpace,
    commands::conceptual_space::ConceptualSpaceCommand, events::DomainEvent,
};
use crate::infrastructure::event_store::EventStore;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConceptualSpaceCommandError {
    #[error("Conceptual space not found: {0}")]
    NotFound(String),

    #[error("Event store error: {0}")]
    EventStore(#[from] crate::infrastructure::event_store::EventStoreError),

    #[error("Domain error: {0}")]
    Domain(String),
}

pub struct ConceptualSpaceCommandHandler {
    event_store: Arc<dyn EventStore>,
}

impl ConceptualSpaceCommandHandler {
    pub fn new(event_store: Arc<dyn EventStore>) -> Self {
        Self { event_store }
    }

    pub async fn handle_command(
        &self,
        command: ConceptualSpaceCommand,
    ) -> Result<Vec<DomainEvent>, ConceptualSpaceCommandError> {
        let space_id = match &command {
            ConceptualSpaceCommand::CreateConceptualSpace(cmd) => cmd.space_id,
            ConceptualSpaceCommand::AddQualityDimension(cmd) => cmd.space_id,
            ConceptualSpaceCommand::MapConcept(cmd) => cmd.space_id,
            ConceptualSpaceCommand::DefineRegion(cmd) => cmd.space_id,
            ConceptualSpaceCommand::CalculateSimilarity(cmd) => cmd.space_id,
            ConceptualSpaceCommand::UpdateMetric(cmd) => cmd.space_id,
        };

        // Load or create aggregate
        let mut space = self.load_or_create_space(space_id).await?;

        // Handle command
        let events = space
            .handle_command(command)
            .map_err(|e| ConceptualSpaceCommandError::Domain(e.to_string()))?;

        // Store events
        if !events.is_empty() {
            self.event_store
                .append_events(space_id.to_string(), events.clone())
                .await?;
        }

        Ok(events)
    }

    async fn load_or_create_space(
        &self,
        space_id: crate::domain::value_objects::ConceptualSpaceId,
    ) -> Result<ConceptualSpace, ConceptualSpaceCommandError> {
        let events = self.event_store.get_events(space_id.to_string()).await?;

        if events.is_empty() {
            // Create new space
            Ok(ConceptualSpace::new(
                space_id,
                String::new(),
                String::new(),
                crate::domain::value_objects::UserId::new(),
            ))
        } else {
            // Rebuild from events
            let mut space = ConceptualSpace::new(
                space_id,
                String::new(),
                String::new(),
                crate::domain::value_objects::UserId::new(),
            );

            for event in events {
                space
                    .apply_event(&event)
                    .map_err(|e| ConceptualSpaceCommandError::Domain(e.to_string()))?;
            }

            Ok(space)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::commands::conceptual_space::{AddQualityDimension, CreateConceptualSpace};
    use crate::domain::conceptual_graph::{DimensionType, QualityDimension};
    use crate::domain::value_objects::{ConceptualSpaceId, DimensionId, UserId};
    use crate::infrastructure::event_store::memory::InMemoryEventStore;
    use uuid::Uuid;

    async fn create_test_handler() -> ConceptualSpaceCommandHandler {
        let event_store = Arc::new(InMemoryEventStore::new());
        ConceptualSpaceCommandHandler::new(event_store)
    }

    #[tokio::test]
    async fn test_create_conceptual_space() {
        let handler = create_test_handler().await;
        let space_id = ConceptualSpaceId::new();

        let command = ConceptualSpaceCommand::CreateConceptualSpace(CreateConceptualSpace {
            space_id,
            name: "Test Space".to_string(),
            description: "A test conceptual space".to_string(),
            created_by: UserId::new(),
        });

        let events = handler.handle_command(command).await.unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], DomainEvent::ConceptualSpaceCreated(_)));
    }

    #[tokio::test]
    async fn test_add_quality_dimension() {
        let handler = create_test_handler().await;
        let space_id = ConceptualSpaceId::new();

        // First create the space
        let create_cmd = ConceptualSpaceCommand::CreateConceptualSpace(CreateConceptualSpace {
            space_id,
            name: "Test Space".to_string(),
            description: "A test conceptual space".to_string(),
            created_by: UserId::new(),
        });
        handler.handle_command(create_cmd).await.unwrap();

        // Then add a dimension
        let dimension = QualityDimension {
            name: "Temperature".to_string(),
            dimension_type: DimensionType::Continuous,
            range: 0.0..100.0,
            metric: crate::domain::conceptual_graph::DistanceMetric::Euclidean,
            weight: 1.0,
        };

        let add_dim_cmd = ConceptualSpaceCommand::AddQualityDimension(AddQualityDimension {
            space_id,
            dimension_id: DimensionId::new(),
            dimension,
        });

        let events = handler.handle_command(add_dim_cmd).await.unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], DomainEvent::QualityDimensionAdded(_)));
    }
}
