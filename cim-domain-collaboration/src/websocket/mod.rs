//! WebSocket server for real-time collaboration

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info, warn};
use uuid::Uuid;

use cim_domain::GraphId;
use crate::aggregate::UserId;
// NatsClient would come from cim_infrastructure
// For now, using a mock
pub struct NatsClient;
impl NatsClient {
    pub async fn publish(&self, _subject: &str, _payload: &[u8]) -> Result<()> {
        Ok(())
    }
}

use crate::{
    commands::CollaborationCommand,
    events::{CollaborationEvent, CursorPosition, SelectionState},
    handlers::CollaborationCommandHandler,
};

/// Messages sent between client and server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CollaborationMessage {
    /// Client wants to join a session
    JoinSession {
        graph_id: GraphId,
        user_id: UserId,
        user_name: String,
    },
    
    /// Client wants to leave current session
    LeaveSession,
    
    /// Client cursor moved
    CursorMove {
        x: f64,
        y: f64,
        z: f64,
    },
    
    /// Client selection changed
    SelectionChange {
        nodes: Vec<String>,
        edges: Vec<String>,
    },
    
    /// Client wants to start editing
    StartEdit {
        element_type: String,
        element_id: String,
    },
    
    /// Client finished editing
    FinishEdit {
        element_type: String,
        element_id: String,
    },
    
    /// Server confirms session joined
    SessionJoined {
        session_id: Uuid,
        your_color: String,
        active_users: Vec<UserInfo>,
    },
    
    /// Server notifies of user join
    UserJoined {
        user_id: UserId,
        user_name: String,
        color: String,
    },
    
    /// Server notifies of user leave
    UserLeft {
        user_id: UserId,
    },
    
    /// Server broadcasts cursor position
    UserCursorMoved {
        user_id: UserId,
        x: f64,
        y: f64,
        z: f64,
    },
    
    /// Server broadcasts selection change
    UserSelectionChanged {
        user_id: UserId,
        nodes: Vec<String>,
        edges: Vec<String>,
    },
    
    /// Server notifies editing started
    UserStartedEditing {
        user_id: UserId,
        element_type: String,
        element_id: String,
    },
    
    /// Server notifies editing finished
    UserFinishedEditing {
        user_id: UserId,
        element_type: String,
        element_id: String,
    },
    
    /// Error message
    Error {
        message: String,
    },
}

/// User information sent to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: UserId,
    pub user_name: String,
    pub color: String,
}

/// WebSocket connection handler
struct ConnectionHandler {
    user_id: Option<UserId>,
    session_id: Option<Uuid>,
    command_handler: Arc<CollaborationCommandHandler>,
    nats_client: Arc<NatsClient>,
    outgoing_tx: mpsc::Sender<Message>,
}

impl ConnectionHandler {
    fn new(
        command_handler: Arc<CollaborationCommandHandler>,
        nats_client: Arc<NatsClient>,
        outgoing_tx: mpsc::Sender<Message>,
    ) -> Self {
        Self {
            user_id: None,
            session_id: None,
            command_handler,
            nats_client,
            outgoing_tx,
        }
    }

    async fn handle_message(&mut self, msg: CollaborationMessage) -> Result<()> {
        match msg {
            CollaborationMessage::JoinSession { graph_id, user_id, user_name } => {
                let events = self.command_handler.handle(CollaborationCommand::JoinSession {
                    graph_id,
                    user_id: user_id.clone(),
                    user_name,
                }).await?;

                if let Some(CollaborationEvent::UserJoinedSession { session_id, color, .. }) = events.first() {
                    self.user_id = Some(user_id);
                    self.session_id = Some(*session_id);

                    // Subscribe to session events
                    let _subject = format!("collaboration.session.{}", session_id);
                    // self.nats_client.subscribe(&subject).await?;

                    // Send confirmation
                    let msg = CollaborationMessage::SessionJoined {
                        session_id: *session_id,
                        your_color: color.clone(),
                        active_users: vec![], // TODO: Get from session
                    };
                    self.send_message(msg).await?;

                    // Broadcast join event
                    self.broadcast_event(&events[0]).await?;
                }
            }

            CollaborationMessage::LeaveSession => {
                if let (Some(session_id), Some(user_id)) = (self.session_id, &self.user_id) {
                    let events = self.command_handler.handle(CollaborationCommand::LeaveSession {
                        session_id,
                        user_id: user_id.clone(),
                    }).await?;

                    for event in &events {
                        self.broadcast_event(event).await?;
                    }
                }
            }

            CollaborationMessage::CursorMove { x, y, z } => {
                if let (Some(session_id), Some(user_id)) = (self.session_id, &self.user_id) {
                    let events = self.command_handler.handle(CollaborationCommand::UpdateCursor {
                        session_id,
                        user_id: user_id.clone(),
                        position: CursorPosition { x, y, z },
                    }).await?;

                    for event in &events {
                        self.broadcast_event(event).await?;
                    }
                }
            }

            CollaborationMessage::SelectionChange { nodes: _, edges: _ } => {
                if let (Some(session_id), Some(user_id)) = (self.session_id, &self.user_id) {
                    let events = self.command_handler.handle(CollaborationCommand::UpdateSelection {
                        session_id,
                        user_id: user_id.clone(),
                        selection: SelectionState {
                            nodes: vec![], // TODO: Convert string IDs to NodeId
                            edges: vec![], // TODO: Convert string IDs to EdgeId
                        },
                    }).await?;

                    for event in &events {
                        self.broadcast_event(event).await?;
                    }
                }
            }

            CollaborationMessage::StartEdit { element_type, element_id } => {
                if let (Some(session_id), Some(user_id)) = (self.session_id, &self.user_id) {
                    let elem_type = match element_type.as_str() {
                        "node" => crate::events::ElementType::Node,
                        "edge" => crate::events::ElementType::Edge,
                        _ => crate::events::ElementType::Graph,
                    };

                    match self.command_handler.handle(CollaborationCommand::StartEditing {
                        session_id,
                        user_id: user_id.clone(),
                        element_type: elem_type,
                        element_id: element_id.clone(),
                    }).await {
                        Ok(events) => {
                            for event in &events {
                                self.broadcast_event(event).await?;
                            }
                        }
                        Err(e) => {
                            self.send_message(CollaborationMessage::Error {
                                message: e.to_string(),
                            }).await?;
                        }
                    }
                }
            }

            CollaborationMessage::FinishEdit { element_type, element_id } => {
                if let (Some(session_id), Some(user_id)) = (self.session_id, &self.user_id) {
                    let elem_type = match element_type.as_str() {
                        "node" => crate::events::ElementType::Node,
                        "edge" => crate::events::ElementType::Edge,
                        _ => crate::events::ElementType::Graph,
                    };

                    let events = self.command_handler.handle(CollaborationCommand::FinishEditing {
                        session_id,
                        user_id: user_id.clone(),
                        element_type: elem_type,
                        element_id,
                    }).await?;

                    for event in &events {
                        self.broadcast_event(event).await?;
                    }
                }
            }

            _ => {
                warn!("Received unexpected message type from client");
            }
        }

        Ok(())
    }

    async fn send_message(&self, msg: CollaborationMessage) -> Result<()> {
        let json = serde_json::to_string(&msg)?;
        self.outgoing_tx.send(Message::text(json)).await?;
        Ok(())
    }

    async fn broadcast_event(&self, event: &CollaborationEvent) -> Result<()> {
        if let Some(session_id) = self.session_id {
            // Publish to NATS for distribution to all connected clients
            let subject = format!("collaboration.session.{}", session_id);
            let payload = serde_json::to_vec(event)?;
            self.nats_client.publish(&subject, &payload).await?;
        }
        Ok(())
    }
}

/// WebSocket server for collaboration
pub struct CollaborationServer {
    addr: SocketAddr,
    command_handler: Arc<CollaborationCommandHandler>,
    nats_client: Arc<NatsClient>,
}

impl CollaborationServer {
    /// Create a new collaboration server
    pub fn new(
        addr: SocketAddr,
        command_handler: Arc<CollaborationCommandHandler>,
        nats_client: Arc<NatsClient>,
    ) -> Self {
        Self {
            addr,
            command_handler,
            nats_client,
        }
    }

    /// Start the WebSocket server
    pub async fn start(self) -> Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        info!("Collaboration WebSocket server listening on {}", self.addr);

        while let Ok((stream, addr)) = listener.accept().await {
            info!("New WebSocket connection from {}", addr);
            
            let command_handler = self.command_handler.clone();
            let nats_client = self.nats_client.clone();
            
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, command_handler, nats_client).await {
                    error!("Error handling connection from {}: {}", addr, e);
                }
            });
        }

        Ok(())
    }
}

async fn handle_connection(
    stream: TcpStream,
    command_handler: Arc<CollaborationCommandHandler>,
    nats_client: Arc<NatsClient>,
) -> Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (ws_sender, mut ws_receiver) = ws_stream.split();
    
    let (tx, mut rx) = mpsc::channel(100);
    let mut handler = ConnectionHandler::new(command_handler, nats_client, tx);

    // Spawn task to forward messages to WebSocket
    let send_task = tokio::spawn(async move {
        use futures_util::sink::SinkExt;
        let mut ws_sender = ws_sender;
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    use futures_util::stream::StreamExt;
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<CollaborationMessage>(&text) {
                    Ok(msg) => {
                        if let Err(e) = handler.handle_message(msg).await {
                            error!("Error handling message: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Invalid message format: {}", e);
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("Client disconnected");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    // Clean up on disconnect
    if let (Some(session_id), Some(user_id)) = (handler.session_id, handler.user_id) {
        let _ = handler.command_handler.handle(CollaborationCommand::LeaveSession {
            session_id,
            user_id,
        }).await;
    }

    send_task.abort();
    Ok(())
}