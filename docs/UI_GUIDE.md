# Alchemist UI Guide

This guide covers the Iced-based UI components in Alchemist, including the Dashboard and Dialog windows.

## Overview

Alchemist provides two main UI components:
1. **Dashboard** - System monitoring and domain management
2. **Dialog** - AI conversation interface

Both UIs are built with Iced and follow The Elm Architecture (TEA), integrating seamlessly with the event-based system.

## Dashboard

### Launching the Dashboard

```bash
# Launch dashboard with NATS integration
ia dashboard

# Launch dashboard in-process (for development)
ia dashboard-local
```

### Features

- **System Status**: Real-time monitoring of NATS connection, memory usage, and uptime
- **Domain Overview**: View all registered domains with health status
- **Interactive Elements**: Click on domains to see detailed information
- **Event Stream**: Live updates from NATS events
- **Recent Events**: Scrollable list of recent system events

### Dashboard Architecture

```rust
// Dashboard data flow
NATS Events -> DashboardNatsStream -> DashboardWindow -> Iced UI

// Key components:
- dashboard.rs: Core data structures
- dashboard_window.rs: Iced UI implementation
- dashboard_nats_stream.rs: Real-time event streaming
```

## Dialog UI

### Creating a New Dialog

```bash
# Create a new dialog with default settings
ia dialog new

# Create with specific title and model
ia dialog new --title "Code Review Assistant" --model claude-3-opus
```

### Features

- **Multi-Model Support**: Switch between Claude and GPT models
- **Real-time Streaming**: See tokens as they're generated
- **Export Options**: Save conversations as Markdown, JSON, or text
- **Token Counting**: Track usage for each message
- **Conversation History**: Scrollable message history

### Dialog Architecture

```rust
// Dialog data flow
User Input -> DialogCommand -> DialogHandler -> AI Manager -> Streaming Response

// Key components:
- dialog_window.rs: Iced UI for conversations
- dialog_handler.rs: Connects UI to AI providers
- dialog.rs: Core dialog management
```

## Event-Based Communication

Both UIs use pure event-based communication:

```rust
// Example: Dialog events
pub enum DialogEvent {
    MessageAdded(DialogMessage),
    TokenStreamed(String),
    ResponseComplete,
    Error(String),
}

// Example: Dashboard events
pub enum DashboardUpdate {
    FullUpdate(DashboardData),
    DomainUpdate { domain: String, info: DomainInfo },
    EventReceived(EventInfo),
}
```

## Customization

### Adding New UI Components

1. Create your Iced window module:
```rust
pub struct MyWindow {
    data: MyData,
    event_receiver: Option<mpsc::Receiver<MyEvent>>,
}

impl MyWindow {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        // Handle messages
    }
    
    pub fn view(&self) -> Element<Message> {
        // Build UI
    }
}
```

2. Create an event handler:
```rust
pub struct MyHandler {
    event_sender: mpsc::Sender<MyEvent>,
}

impl MyHandler {
    pub async fn start(mut self, mut cmd_rx: mpsc::Receiver<MyCommand>) {
        while let Some(cmd) = cmd_rx.recv().await {
            // Process commands and send events
        }
    }
}
```

3. Wire it into the shell:
```rust
pub async fn launch_my_window(&self) -> Result<()> {
    let (cmd_tx, cmd_rx) = mpsc::channel(100);
    let (event_tx, event_rx) = mpsc::channel(100);
    
    // Start handler
    let handler = MyHandler::new(event_tx);
    tokio::spawn(async move {
        handler.start(cmd_rx).await;
    });
    
    // Run window
    my_window::run(event_rx).await
}
```

## Styling

Both UIs use Iced's theming system:

```rust
// Dark theme is default
.theme(|_| Theme::Dark)

// Custom styles
.style(button::primary)  // Selected state
.style(button::secondary) // Normal state
.style(container::rounded_box) // Rounded containers
```

## Performance Considerations

1. **Event Batching**: The dashboard batches updates to avoid overwhelming the UI
2. **Lazy Loading**: Large lists use scrollable containers
3. **Async Updates**: All I/O operations are non-blocking
4. **Channel Buffers**: Use appropriate buffer sizes for mpsc channels

## Debugging

Enable debug logging:
```bash
RUST_LOG=alchemist=debug ia dashboard-local
```

Check NATS events:
```bash
# Subscribe to all events
nats sub "cim.>" "dashboard.>"

# Monitor specific domain
nats sub "cim.workflow.>"
```

## Examples

See the examples directory for complete implementations:
- `examples/custom_ui_window.rs` - Creating a custom Iced window
- `examples/dialog_automation.rs` - Automating dialog interactions
- `examples/dashboard_plugin.rs` - Adding custom dashboard panels