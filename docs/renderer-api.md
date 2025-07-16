# Alchemist Renderer API Reference

## Overview

The Alchemist Renderer API provides a comprehensive event-driven interface for building visualization components that integrate with the Alchemist shell. This document covers the API structure, event types, communication patterns, and implementation guidelines.

## Architecture

The renderer system follows an event-driven architecture with these key components:

1. **Shell**: Core application logic and state management
2. **Renderer**: Visualization components (Iced, Bevy, or custom)
3. **Event Bridge**: NATS-based communication layer
4. **Event Types**: Strongly-typed messages for bidirectional communication

## Core Components

### 1. Event Types

#### Shell to Renderer Events (`ShellToRendererEvent`)

```rust
pub enum ShellToRendererEvent {
    // Dashboard Updates
    DashboardUpdate(DashboardData),
    
    // Dialog Events
    DialogStarted { dialog_id: String },
    UserMessage { dialog_id: String, message: String },
    AiThinking { dialog_id: String },
    AiResponseChunk { dialog_id: String, chunk: String },
    AiResponseComplete { dialog_id: String },
    DialogError { dialog_id: String, error: String },
    
    // Domain Events
    DomainEvent { domain: String, event: serde_json::Value },
    
    // System Events
    SystemEvent { event_type: String, data: serde_json::Value },
    PolicyUpdate { policies: Vec<PolicyInfo> },
    DeploymentUpdate { deployments: Vec<DeploymentInfo> },
    
    // Workflow Events
    WorkflowStarted { workflow_id: String, name: String },
    WorkflowProgress { workflow_id: String, progress: f32 },
    WorkflowCompleted { workflow_id: String, status: String },
    
    // Performance Metrics
    PerformanceMetrics {
        cpu_usage: f32,
        memory_usage: f32,
        event_rate: f32,
        active_connections: u32,
    },
}
```

#### Renderer to Shell Events (`RendererToShellEvent`)

```rust
pub enum RendererToShellEvent {
    // User Input
    UserInput { input: String },
    CommandRequest { command: String },
    
    // Dialog Control
    StartDialog { title: String, model: Option<String> },
    StopDialog { dialog_id: String },
    SendMessage { dialog_id: String, message: String },
    
    // UI Actions
    RefreshDashboard,
    OpenDomain { domain: String },
    CloseDomain { domain: String },
    
    // Policy Actions
    CreatePolicy { name: String, domain: String },
    UpdatePolicy { policy_id: String, changes: serde_json::Value },
    DeletePolicy { policy_id: String },
    
    // Deployment Actions
    StartDeployment { target: String, domains: Vec<String> },
    ApproveDeployment { approval_id: String, approved: bool },
    
    // Window Control
    WindowClosed { window_id: String },
    WindowResized { window_id: String, width: u32, height: u32 },
}
```

### 2. Data Structures

#### DashboardData

```rust
pub struct DashboardData {
    pub total_events: u64,
    pub active_domains: usize,
    pub total_policies: usize,
    pub active_dialogs: usize,
    pub domains: Vec<DomainInfo>,
    pub recent_dialogs: Vec<DialogInfo>,
    pub recent_events: Vec<EventInfo>,
    pub system_status: SystemStatus,
    pub recent_policies: Vec<PolicyInfo>,
}
```

#### DomainInfo

```rust
pub struct DomainInfo {
    pub name: String,
    pub event_count: u64,
    pub is_active: bool,
    pub health: f32,  // 0.0 to 100.0
}
```

#### DialogInfo

```rust
pub struct DialogInfo {
    pub id: String,
    pub title: String,
    pub model: String,
    pub message_count: usize,
    pub last_activity: String,
}
```

#### EventInfo

```rust
pub struct EventInfo {
    pub timestamp: String,
    pub domain: String,
    pub event_type: String,
    pub summary: String,
}
```

### 3. Event Builder

The `EventBuilder` provides convenient methods for creating events:

```rust
use alchemist::renderer_events::EventBuilder;

// Dashboard update
let event = EventBuilder::dashboard_update(dashboard_data);

// Dialog events
let event = EventBuilder::dialog_started("dialog123".to_string());
let event = EventBuilder::user_message("dialog123".to_string(), "Hello".to_string());
let event = EventBuilder::ai_thinking("dialog123".to_string());
let event = EventBuilder::ai_response_chunk("dialog123".to_string(), "Hi".to_string());
let event = EventBuilder::ai_response_complete("dialog123".to_string());

// System events
let event = EventBuilder::system_event(
    "deployment_completed".to_string(),
    json!({ "target": "production", "status": "success" })
);
```

## Communication Patterns

### 1. Direct Channel Communication

For embedded renderers running in the same process:

```rust
use tokio::sync::mpsc;
use alchemist::renderer_events::*;

// Create channels
let (shell_tx, shell_rx) = mpsc::channel::<RendererToShellEvent>(100);
let (renderer_tx, renderer_rx) = mpsc::channel::<ShellToRendererEvent>(100);

// In shell
shell_handler.set_renderer_channel(renderer_tx);

// In renderer
renderer.set_shell_channel(shell_tx);
renderer.set_event_receiver(renderer_rx);
```

### 2. NATS Bridge Communication

For distributed or external renderers:

```rust
use alchemist::renderer_nats_bridge::*;

// Create NATS client
let nats_client = NatsClient::new("nats://localhost:4222").await?;

// Create bridge
let (event_tx, event_rx) = mpsc::channel(100);
let bridge = RendererNatsBridge::new(Some(nats_client), event_tx);

// Register component
bridge.register_component("my-renderer", ComponentType::Dashboard).await?;

// Subscribe to shell events
bridge.subscribe_to_shell_events().await?;

// Publish renderer events
bridge.publish_renderer_event(&RendererToShellEvent::RefreshDashboard).await?;
```

### 3. Event Flow Patterns

#### Request-Response Pattern
```rust
// Renderer requests dashboard refresh
renderer_tx.send(RendererToShellEvent::RefreshDashboard).await?;

// Shell responds with update
shell_tx.send(ShellToRendererEvent::DashboardUpdate(data)).await?;
```

#### Streaming Pattern (AI Responses)
```rust
// 1. Start dialog
shell_tx.send(EventBuilder::dialog_started(dialog_id)).await?;

// 2. User message
shell_tx.send(EventBuilder::user_message(dialog_id, message)).await?;

// 3. AI thinking indicator
shell_tx.send(EventBuilder::ai_thinking(dialog_id)).await?;

// 4. Stream response chunks
for chunk in response_chunks {
    shell_tx.send(EventBuilder::ai_response_chunk(dialog_id, chunk)).await?;
}

// 5. Complete
shell_tx.send(EventBuilder::ai_response_complete(dialog_id)).await?;
```

## Implementation Guide

### 1. Creating a Custom Renderer

```rust
use alchemist::renderer_events::*;
use tokio::sync::mpsc;

pub struct MyCustomRenderer {
    shell_tx: mpsc::Sender<RendererToShellEvent>,
    event_rx: mpsc::Receiver<ShellToRendererEvent>,
    state: RendererState,
}

impl MyCustomRenderer {
    pub fn new(
        shell_tx: mpsc::Sender<RendererToShellEvent>,
        event_rx: mpsc::Receiver<ShellToRendererEvent>,
    ) -> Self {
        Self {
            shell_tx,
            event_rx,
            state: RendererState::default(),
        }
    }
    
    pub async fn run(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                // Handle shell events
                Some(event) = self.event_rx.recv() => {
                    self.handle_shell_event(event).await?;
                }
                
                // Handle UI events
                ui_event = self.poll_ui_event() => {
                    self.handle_ui_event(ui_event).await?;
                }
            }
        }
    }
    
    async fn handle_shell_event(&mut self, event: ShellToRendererEvent) -> Result<()> {
        match event {
            ShellToRendererEvent::DashboardUpdate(data) => {
                self.state.update_dashboard(data);
                self.render_dashboard()?;
            }
            ShellToRendererEvent::AiResponseChunk { dialog_id, chunk } => {
                self.state.append_ai_response(&dialog_id, &chunk);
                self.render_dialog(&dialog_id)?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

### 2. Iced Integration Example

```rust
use iced::{Application, Command, Element, Settings, Subscription};
use alchemist::renderer_events::*;

struct IcedRenderer {
    shell_tx: mpsc::Sender<RendererToShellEvent>,
    dashboard_data: Option<DashboardData>,
}

#[derive(Debug, Clone)]
enum Message {
    EventReceived(ShellToRendererEvent),
    RefreshClicked,
    CommandInput(String),
}

impl Application for IcedRenderer {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Flags = ();
    
    fn new(_flags: ()) -> (Self, Command<Message>) {
        // Initialize with channels
        (Self::default(), Command::none())
    }
    
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::EventReceived(event) => {
                match event {
                    ShellToRendererEvent::DashboardUpdate(data) => {
                        self.dashboard_data = Some(data);
                    }
                    _ => {}
                }
            }
            Message::RefreshClicked => {
                // Send refresh request
                let tx = self.shell_tx.clone();
                return Command::perform(
                    async move {
                        tx.send(RendererToShellEvent::RefreshDashboard).await
                    },
                    |_| Message::RefreshClicked
                );
            }
            _ => {}
        }
        Command::none()
    }
    
    fn view(&self) -> Element<Message> {
        // Build UI based on dashboard_data
        // ...
    }
}
```

### 3. Bevy Integration Example

```rust
use bevy::prelude::*;
use alchemist::renderer_events::*;

#[derive(Resource)]
struct ShellChannel(mpsc::Sender<RendererToShellEvent>);

#[derive(Resource)]
struct EventReceiver(mpsc::Receiver<ShellToRendererEvent>);

fn setup_bevy_renderer(mut commands: Commands) {
    // Create channels and store as resources
    let (shell_tx, _) = mpsc::channel(100);
    let (_, event_rx) = mpsc::channel(100);
    
    commands.insert_resource(ShellChannel(shell_tx));
    commands.insert_resource(EventReceiver(event_rx));
}

fn handle_shell_events(
    mut event_rx: ResMut<EventReceiver>,
    mut dashboard: ResMut<DashboardState>,
) {
    while let Ok(event) = event_rx.0.try_recv() {
        match event {
            ShellToRendererEvent::DashboardUpdate(data) => {
                dashboard.update(data);
            }
            _ => {}
        }
    }
}

fn send_commands(
    shell_tx: Res<ShellChannel>,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::R) {
        // Send refresh command
        let _ = shell_tx.0.try_send(RendererToShellEvent::RefreshDashboard);
    }
}
```

## Advanced Topics

### 1. Custom Event Types

Extend the event system with domain-specific events:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct GraphNodeCreated {
    pub node_id: String,
    pub node_type: String,
    pub position: (f32, f32, f32),
}

// Send as domain event
let event = ShellToRendererEvent::DomainEvent {
    domain: "graph".to_string(),
    event: serde_json::to_value(GraphNodeCreated {
        node_id: "node123".to_string(),
        node_type: "concept".to_string(),
        position: (0.0, 0.0, 0.0),
    })?,
};
```

### 2. Performance Optimization

#### Event Batching
```rust
let mut event_batch = Vec::new();
let mut last_flush = Instant::now();

loop {
    if let Ok(event) = event_rx.try_recv() {
        event_batch.push(event);
    }
    
    if event_batch.len() > 100 || last_flush.elapsed() > Duration::from_millis(16) {
        process_event_batch(&event_batch);
        event_batch.clear();
        last_flush = Instant::now();
    }
}
```

#### Selective Updates
```rust
impl DashboardData {
    pub fn diff(&self, other: &DashboardData) -> DashboardDiff {
        // Calculate what changed
        DashboardDiff {
            events_changed: self.total_events != other.total_events,
            domains_changed: self.domains != other.domains,
            // ...
        }
    }
}
```

### 3. Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("Channel disconnected")]
    ChannelDisconnected,
    
    #[error("NATS error: {0}")]
    NatsError(#[from] async_nats::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

// Graceful error handling
match shell_tx.send(event).await {
    Ok(_) => {},
    Err(_) => {
        log::warn!("Shell channel disconnected, attempting reconnect");
        self.reconnect_shell().await?;
    }
}
```

### 4. Testing Renderers

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_event_handling() {
        let (shell_tx, shell_rx) = mpsc::channel(10);
        let (renderer_tx, mut renderer_rx) = mpsc::channel(10);
        
        let mut renderer = MyRenderer::new(shell_tx, renderer_rx);
        
        // Send test event
        renderer_tx.send(ShellToRendererEvent::DashboardUpdate(
            DashboardData::example()
        )).await.unwrap();
        
        // Process event
        renderer.process_next_event().await.unwrap();
        
        // Verify state updated
        assert!(renderer.has_dashboard_data());
    }
}
```

## Best Practices

### 1. Event Design

- **Keep events focused**: One event type per logical action
- **Use strong typing**: Avoid generic data blobs when possible
- **Version events**: Consider compatibility for future changes
- **Document events**: Clear descriptions of when events are sent

### 2. Performance

- **Batch updates**: Combine multiple small updates
- **Throttle events**: Limit update frequency (e.g., 60 FPS max)
- **Async processing**: Don't block on event handling
- **Selective rendering**: Only update changed components

### 3. Error Handling

- **Graceful degradation**: Continue operating if connection lost
- **Reconnection logic**: Automatically reconnect to services
- **User feedback**: Show connection status in UI
- **Logging**: Log errors for debugging

### 4. State Management

- **Single source of truth**: Shell owns canonical state
- **Local caching**: Cache data for responsive UI
- **Optimistic updates**: Update UI before confirmation
- **Reconciliation**: Handle state conflicts gracefully

## Example Implementations

### 1. Minimal Dashboard

```rust
use alchemist::renderer_events::*;

async fn minimal_dashboard() -> Result<()> {
    let (shell_tx, mut event_rx) = create_channels();
    
    loop {
        match event_rx.recv().await {
            Some(ShellToRendererEvent::DashboardUpdate(data)) => {
                println!("Dashboard Update:");
                println!("  Events: {}", data.total_events);
                println!("  Domains: {}", data.active_domains);
                println!("  Policies: {}", data.total_policies);
            }
            _ => {}
        }
    }
}
```

### 2. Interactive Dialog Window

```rust
async fn dialog_window(
    dialog_id: String,
    shell_tx: mpsc::Sender<RendererToShellEvent>,
    mut event_rx: mpsc::Receiver<ShellToRendererEvent>,
) -> Result<()> {
    let mut messages = Vec::new();
    let mut current_response = String::new();
    
    loop {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                match event {
                    ShellToRendererEvent::UserMessage { message, .. } => {
                        messages.push(("user".to_string(), message));
                    }
                    ShellToRendererEvent::AiResponseChunk { chunk, .. } => {
                        current_response.push_str(&chunk);
                    }
                    ShellToRendererEvent::AiResponseComplete { .. } => {
                        messages.push(("assistant".to_string(), current_response.clone()));
                        current_response.clear();
                    }
                    _ => {}
                }
            }
            
            user_input = read_user_input() => {
                shell_tx.send(RendererToShellEvent::SendMessage {
                    dialog_id: dialog_id.clone(),
                    message: user_input,
                }).await?;
            }
        }
    }
}
```

## Troubleshooting

### Common Issues

1. **Events not received**: Check channel capacity and backpressure
2. **NATS connection failed**: Verify NATS server is running
3. **Serialization errors**: Ensure all custom types implement Serialize/Deserialize
4. **Performance issues**: Profile event handling and rendering separately

### Debug Tools

```rust
// Event logger
#[derive(Clone)]
struct EventLogger;

impl EventLogger {
    async fn log_events(mut rx: mpsc::Receiver<ShellToRendererEvent>) {
        while let Some(event) = rx.recv().await {
            log::debug!("Received event: {:?}", event);
        }
    }
}

// Performance monitor
struct PerformanceMonitor {
    event_count: u64,
    last_report: Instant,
}

impl PerformanceMonitor {
    fn track_event(&mut self) {
        self.event_count += 1;
        if self.last_report.elapsed() > Duration::from_secs(1) {
            log::info!("Events/sec: {}", self.event_count);
            self.event_count = 0;
            self.last_report = Instant::now();
        }
    }
}
```

## Resources

- **Examples**: See `/examples` directory for complete implementations
- **Tests**: See `/tests/renderer_integration_tests.rs` for test examples
- **NATS Docs**: https://docs.nats.io/
- **Iced Docs**: https://docs.rs/iced/
- **Bevy Docs**: https://docs.rs/bevy/