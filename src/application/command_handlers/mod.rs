//! Command Handlers - Process commands and generate events

use crate::application::{CommandEvent, EventNotification};
use crate::domain::commands::{Command, GraphCommand};
use crate::domain::events::{DomainEvent, GraphEvent};
use crate::domain::value_objects::*;
use bevy::prelude::*;

/// System that processes commands and generates events
pub fn process_commands(
    mut commands: EventReader<CommandEvent>,
    mut events: EventWriter<EventNotification>,
) {
    for command_event in commands.read() {
        match &command_event.command {
            Command::Graph(graph_cmd) => {
                // Process graph commands
                if let Some(event) = handle_graph_command(graph_cmd) {
                    events.write(EventNotification { event });
                }
            }
            Command::Node(_node_cmd) => {
                // TODO: Handle node commands
            }
            Command::Edge(_edge_cmd) => {
                // TODO: Handle edge commands
            }
        }
    }
}

/// Handle graph commands and generate events
fn handle_graph_command(command: &GraphCommand) -> Option<DomainEvent> {
    match command {
        GraphCommand::CreateGraph { id, name } => {
            Some(DomainEvent::Graph(GraphEvent::GraphCreated {
                id: *id,
                metadata: GraphMetadata::new(name.clone()),
            }))
        }
        GraphCommand::RenameGraph { id, new_name } => {
            Some(DomainEvent::Graph(GraphEvent::GraphRenamed {
                id: *id,
                old_name: String::new(), // TODO: Get from aggregate
                new_name: new_name.clone(),
            }))
        }
        GraphCommand::TagGraph { id, tag } => Some(DomainEvent::Graph(GraphEvent::GraphTagged {
            id: *id,
            tag: tag.clone(),
        })),
        GraphCommand::UntagGraph { id, tag } => {
            Some(DomainEvent::Graph(GraphEvent::GraphUntagged {
                id: *id,
                tag: tag.clone(),
            }))
        }
        GraphCommand::DeleteGraph { id } => {
            Some(DomainEvent::Graph(GraphEvent::GraphDeleted { id: *id }))
        }
    }
}
