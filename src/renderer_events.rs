//! Pure event-based communication for renderers
//!
//! This module provides event definitions and routing for communication
//! between the main process and renderer processes. All communication
//! is done through events, which naturally integrate with:
//! - Iced's TEA (The Elm Architecture) 
//! - Bevy's ECS (Entity Component System)
//! - Our domain event system

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Base trait for all renderer events
pub trait RendererEvent: Send + Sync + 'static {
    /// Get the event ID
    fn event_id(&self) -> &str;
    /// Get the timestamp
    fn timestamp(&self) -> DateTime<Utc>;
    /// Get the source renderer ID (if any)
    fn renderer_id(&self) -> Option<&str>;
}

/// Events that flow from the shell to renderers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ShellToRendererEvent {
    /// Initialize a new renderer window
    Initialize {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        config: crate::renderer::RenderRequest,
    },
    
    /// Update dialog with new message
    DialogMessageAdded {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        dialog_id: String,
        role: String,
        content: String,
    },
    
    /// Stream a token to the dialog
    DialogTokenStreamed {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        dialog_id: String,
        token: String,
    },
    
    /// Complete the streaming response
    DialogStreamCompleted {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        dialog_id: String,
    },
    
    /// Update dialog loading state
    DialogLoadingStateChanged {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        dialog_id: String,
        loading: bool,
    },
    
    /// Clear dialog messages
    DialogCleared {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        dialog_id: String,
    },
    
    /// Update graph data
    GraphDataUpdated {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        nodes: Vec<crate::renderer::GraphNode>,
        edges: Vec<crate::renderer::GraphEdge>,
    },
    
    /// Update workflow visualization
    WorkflowUpdated {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        workflow_id: String,
        workflow_data: serde_json::Value,
    },
    
    /// Workflow execution started
    WorkflowStarted {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        execution_id: String,
        workflow_id: String,
    },
    
    /// Workflow step started
    WorkflowStepStarted {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        execution_id: String,
        step_id: String,
        step_name: String,
    },
    
    /// Workflow step completed
    WorkflowStepCompleted {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        execution_id: String,
        step_id: String,
        step_name: String,
    },
    
    /// Workflow step failed
    WorkflowStepFailed {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        execution_id: String,
        step_id: String,
        step_name: String,
        error: String,
    },
    
    /// Workflow completed
    WorkflowCompleted {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        execution_id: String,
        workflow_id: String,
    },
    
    /// Workflow failed
    WorkflowFailed {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        execution_id: String,
        workflow_id: String,
        error: String,
    },
    
    /// Close the renderer
    CloseRequested {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
    },
}

/// Events that flow from renderers to the shell
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RendererToShellEvent {
    /// Renderer has initialized
    Initialized {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        renderer_type: crate::renderer::RendererType,
        pid: u32,
    },
    
    /// User submitted a message in dialog
    DialogMessageSubmitted {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        dialog_id: String,
        content: String,
    },
    
    /// User changed the AI model
    DialogModelChanged {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        dialog_id: String,
        model: String,
    },
    
    /// User requested to clear dialog
    DialogClearRequested {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        dialog_id: String,
    },
    
    /// User requested to export dialog
    DialogExportRequested {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        dialog_id: String,
        format: String,
    },
    
    /// Graph node was clicked
    GraphNodeClicked {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        node_id: String,
    },
    
    /// Graph edge was clicked
    GraphEdgeClicked {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        edge_id: String,
    },
    
    /// Workflow step completed
    WorkflowStepCompleted {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        workflow_id: String,
        step_id: String,
    },
    
    /// Window is closing
    WindowClosing {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
    },
    
    /// Error occurred in renderer
    ErrorOccurred {
        event_id: String,
        timestamp: DateTime<Utc>,
        renderer_id: String,
        error: String,
        context: Option<String>,
    },
}

/// Implementation of RendererEvent for ShellToRendererEvent
impl RendererEvent for ShellToRendererEvent {
    fn event_id(&self) -> &str {
        match self {
            Self::Initialize { event_id, .. } => event_id,
            Self::DialogMessageAdded { event_id, .. } => event_id,
            Self::DialogTokenStreamed { event_id, .. } => event_id,
            Self::DialogStreamCompleted { event_id, .. } => event_id,
            Self::DialogLoadingStateChanged { event_id, .. } => event_id,
            Self::DialogCleared { event_id, .. } => event_id,
            Self::GraphDataUpdated { event_id, .. } => event_id,
            Self::WorkflowUpdated { event_id, .. } => event_id,
            Self::WorkflowStarted { event_id, .. } => event_id,
            Self::WorkflowStepStarted { event_id, .. } => event_id,
            Self::WorkflowStepCompleted { event_id, .. } => event_id,
            Self::WorkflowStepFailed { event_id, .. } => event_id,
            Self::WorkflowCompleted { event_id, .. } => event_id,
            Self::WorkflowFailed { event_id, .. } => event_id,
            Self::CloseRequested { event_id, .. } => event_id,
        }
    }
    
    fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::Initialize { timestamp, .. } => *timestamp,
            Self::DialogMessageAdded { timestamp, .. } => *timestamp,
            Self::DialogTokenStreamed { timestamp, .. } => *timestamp,
            Self::DialogStreamCompleted { timestamp, .. } => *timestamp,
            Self::DialogLoadingStateChanged { timestamp, .. } => *timestamp,
            Self::DialogCleared { timestamp, .. } => *timestamp,
            Self::GraphDataUpdated { timestamp, .. } => *timestamp,
            Self::WorkflowUpdated { timestamp, .. } => *timestamp,
            Self::WorkflowStarted { timestamp, .. } => *timestamp,
            Self::WorkflowStepStarted { timestamp, .. } => *timestamp,
            Self::WorkflowStepCompleted { timestamp, .. } => *timestamp,
            Self::WorkflowStepFailed { timestamp, .. } => *timestamp,
            Self::WorkflowCompleted { timestamp, .. } => *timestamp,
            Self::WorkflowFailed { timestamp, .. } => *timestamp,
            Self::CloseRequested { timestamp, .. } => *timestamp,
        }
    }
    
    fn renderer_id(&self) -> Option<&str> {
        match self {
            Self::Initialize { renderer_id, .. } => Some(renderer_id),
            Self::DialogMessageAdded { renderer_id, .. } => Some(renderer_id),
            Self::DialogTokenStreamed { renderer_id, .. } => Some(renderer_id),
            Self::DialogStreamCompleted { renderer_id, .. } => Some(renderer_id),
            Self::DialogLoadingStateChanged { renderer_id, .. } => Some(renderer_id),
            Self::DialogCleared { renderer_id, .. } => Some(renderer_id),
            Self::GraphDataUpdated { renderer_id, .. } => Some(renderer_id),
            Self::WorkflowUpdated { renderer_id, .. } => Some(renderer_id),
            Self::WorkflowStarted { renderer_id, .. } => Some(renderer_id),
            Self::WorkflowStepStarted { renderer_id, .. } => Some(renderer_id),
            Self::WorkflowStepCompleted { renderer_id, .. } => Some(renderer_id),
            Self::WorkflowStepFailed { renderer_id, .. } => Some(renderer_id),
            Self::WorkflowCompleted { renderer_id, .. } => Some(renderer_id),
            Self::WorkflowFailed { renderer_id, .. } => Some(renderer_id),
            Self::CloseRequested { renderer_id, .. } => Some(renderer_id),
        }
    }
}

/// Implementation of RendererEvent for RendererToShellEvent
impl RendererEvent for RendererToShellEvent {
    fn event_id(&self) -> &str {
        match self {
            Self::Initialized { event_id, .. } => event_id,
            Self::DialogMessageSubmitted { event_id, .. } => event_id,
            Self::DialogModelChanged { event_id, .. } => event_id,
            Self::DialogClearRequested { event_id, .. } => event_id,
            Self::DialogExportRequested { event_id, .. } => event_id,
            Self::GraphNodeClicked { event_id, .. } => event_id,
            Self::GraphEdgeClicked { event_id, .. } => event_id,
            Self::WorkflowStepCompleted { event_id, .. } => event_id,
            Self::WindowClosing { event_id, .. } => event_id,
            Self::ErrorOccurred { event_id, .. } => event_id,
        }
    }
    
    fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::Initialized { timestamp, .. } => *timestamp,
            Self::DialogMessageSubmitted { timestamp, .. } => *timestamp,
            Self::DialogModelChanged { timestamp, .. } => *timestamp,
            Self::DialogClearRequested { timestamp, .. } => *timestamp,
            Self::DialogExportRequested { timestamp, .. } => *timestamp,
            Self::GraphNodeClicked { timestamp, .. } => *timestamp,
            Self::GraphEdgeClicked { timestamp, .. } => *timestamp,
            Self::WorkflowStepCompleted { timestamp, .. } => *timestamp,
            Self::WindowClosing { timestamp, .. } => *timestamp,
            Self::ErrorOccurred { timestamp, .. } => *timestamp,
        }
    }
    
    fn renderer_id(&self) -> Option<&str> {
        match self {
            Self::Initialized { renderer_id, .. } => Some(renderer_id),
            Self::DialogMessageSubmitted { renderer_id, .. } => Some(renderer_id),
            Self::DialogModelChanged { renderer_id, .. } => Some(renderer_id),
            Self::DialogClearRequested { renderer_id, .. } => Some(renderer_id),
            Self::DialogExportRequested { renderer_id, .. } => Some(renderer_id),
            Self::GraphNodeClicked { renderer_id, .. } => Some(renderer_id),
            Self::GraphEdgeClicked { renderer_id, .. } => Some(renderer_id),
            Self::WorkflowStepCompleted { renderer_id, .. } => Some(renderer_id),
            Self::WindowClosing { renderer_id, .. } => Some(renderer_id),
            Self::ErrorOccurred { renderer_id, .. } => Some(renderer_id),
        }
    }
}

/// Helper to create events with automatic ID and timestamp
pub struct EventBuilder;

impl EventBuilder {
    /// Create a new event ID
    pub fn new_id() -> String {
        Uuid::new_v4().to_string()
    }
    
    /// Get current timestamp
    pub fn now() -> DateTime<Utc> {
        Utc::now()
    }
    
    /// Create a dialog message added event
    pub fn dialog_message_added(
        renderer_id: String,
        dialog_id: String,
        role: String,
        content: String,
    ) -> ShellToRendererEvent {
        ShellToRendererEvent::DialogMessageAdded {
            event_id: Self::new_id(),
            timestamp: Self::now(),
            renderer_id,
            dialog_id,
            role,
            content,
        }
    }
    
    /// Create a dialog message submitted event
    pub fn dialog_message_submitted(
        renderer_id: String,
        dialog_id: String,
        content: String,
    ) -> RendererToShellEvent {
        RendererToShellEvent::DialogMessageSubmitted {
            event_id: Self::new_id(),
            timestamp: Self::now(),
            renderer_id,
            dialog_id,
            content,
        }
    }
}

/// For Iced TEA integration
pub mod iced_integration {
    use super::*;
    
    /// Convert renderer events to Iced messages
    pub trait ToIcedMessage {
        type Message;
        fn to_message(&self) -> Self::Message;
    }
}

/// For Bevy ECS integration
pub mod bevy_integration {
    use super::*;
    
    /// Marker component for entities that handle renderer events
    #[derive(Debug, Clone)]
    pub struct RendererEventHandler {
        pub renderer_id: String,
    }
    
    /// Resource for queuing events to be processed by systems
    #[derive(Default)]
    pub struct RendererEventQueue {
        pub events: Vec<ShellToRendererEvent>,
    }
}