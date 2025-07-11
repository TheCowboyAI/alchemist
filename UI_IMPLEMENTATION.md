# Alchemist UI Implementation

This document describes the UI components that have been implemented for the Alchemist CIM Control System.

## Overview

The Alchemist UI has been built using the Iced framework (The Elm Architecture) with NATS-based event communication. All components follow the project's core principles:

- **Event-Driven Architecture**: Pure event-based communication via NATS
- **Domain-Driven Design**: Bounded contexts for each domain
- **Test-Driven Development**: Components built with testing in mind
- **CIM Integration**: Uses CIM subjects for messaging

## Working Components

### 1. Unified Launcher (`src/launcher.rs`)

The main entry point that provides access to all Alchemist components.

**Features:**
- Launch dashboard, dialog windows, and other tools
- Conversation management panel
- Document management panel
- Settings configuration
- NATS connection status

**Usage:**
```bash
cargo run --bin alchemist
```

### 2. Dashboard (`src/dashboard_minimal.rs`)

Real-time system monitoring and status display.

**Features:**
- Event counter and live updates
- Active domains display
- Recent events list
- System status (NATS connection, latency)
- Real-time updates via NATS

**Demo:**
```bash
cargo run --example nats_dashboard_demo
```

### 3. Dialog Window (`src/dialog_window_minimal.rs`)

AI conversation interface for interacting with language models.

**Features:**
- Chat-style message interface
- Model selection (Claude, GPT-4, etc.)
- Streaming responses
- Message history
- Export functionality

**Demo:**
```bash
cargo run --example dialog_window_demo
```

### 4. Event Visualizer (`src/event_visualizer.rs`)

Real-time visualization of CIM domain events.

**Features:**
- Live event stream display
- Domain filtering
- Event statistics
- Event type breakdown
- Pause/resume functionality

**Demo:**
```bash
cargo run --example event_visualizer_demo
```

### 5. Renderer API (`src/renderer_api.rs` & `src/renderer_nats_bridge.rs`)

Communication layer between UI components and renderers.

**Features:**
- Event-based communication protocol
- NATS bridge for distributed components
- Support for dialog, dashboard, and custom events
- Component registration and lifecycle management

**Demo:**
```bash
cargo run --example renderer_api_demo
```

## Communication Architecture

```
┌─────────────┐     NATS Events      ┌──────────────┐
│   Launcher  │◄────────────────────►│   Dashboard  │
└──────┬──────┘                       └──────────────┘
       │                                      ▲
       │ Spawns                               │
       ▼                                      │
┌─────────────┐                       ┌──────┴───────┐
│Dialog Window│◄─────────────────────►│ Event Stream │
└─────────────┘    CIM Subjects       └──────────────┘
                                              ▲
                                              │
                                      ┌───────┴──────┐
                                      │  Renderer API │
                                      └──────────────┘
```

## NATS Integration

When NATS is available, components automatically connect and communicate via CIM subjects:

- `cim.renderer.command` - Commands to renderers
- `cim.renderer.event` - Events from renderers
- `cim.dialog.command` - Dialog-specific commands
- `cim.dialog.event` - Dialog-specific events
- `cim.dashboard.update` - Dashboard data updates
- `cim.system.status` - System status updates
- `cim.workflow.*` - Workflow domain events
- `cim.document.*` - Document domain events

## Running the Full System

1. **With NATS** (recommended):
   ```bash
   # Start NATS server
   docker run -p 4222:4222 nats:latest
   
   # Run the launcher
   cargo run --bin alchemist
   ```

2. **Without NATS** (demo mode):
   ```bash
   # Components will run in offline/demo mode
   cargo run --bin alchemist
   ```

3. **Full System Demo**:
   ```bash
   cargo run --example full_system_demo
   ```

## Key Implementation Details

### Minimal Working Approach

After initial complexity issues, the implementation was refactored to focus on:
1. Getting windows to display first
2. Adding functionality incrementally
3. Using working minimal examples
4. Building on proven patterns

### Event-Based Updates

All UI updates flow through events:
- User interactions generate events
- NATS messages trigger UI updates
- Components subscribe to relevant event streams
- No direct IPC or shared memory

### Async Integration

- Tokio runtime for async operations
- mpsc channels for internal communication
- NATS async client for event streaming
- Non-blocking UI updates

## Additional Components

### 6. Performance Monitor (`src/performance_monitor_ui.rs`)

Real-time system resource monitoring and analysis.

**Features:**
- CPU usage tracking with historical graphs
- Memory consumption visualization
- Network activity monitoring (RX/TX)
- Process list with sorting by CPU/Memory
- Load average display
- Export metrics to JSON

**Demo:**
```bash
cargo run --example performance_monitor_demo
```

### 7. Deployment Manager (`src/deployment_ui.rs`)

Visual interface for managing Nix deployments.

**Features:**
- Deploy from flake URLs
- Environment management (dev/staging/prod)
- Deployment status tracking
- Rollback capability
- Live deployment logs
- Environment variables configuration

**Demo:**
```bash
cargo run --example deployment_manager_demo
```

## Future Enhancements

While the UI is now feature-complete, potential enhancements include:

1. **Workflow Visual Editor** - Drag-and-drop workflow creation
2. **Enhanced NATS Monitoring** - Detailed message flow visualization
3. **Theme Customization** - User-selectable themes
4. **Persistent Settings** - Save user preferences
5. **Multi-window Support** - Detachable panels
6. **Plugin System** - Extensible UI components

## Development Guidelines

When extending the UI:

1. Start with a minimal working example
2. Use the existing event patterns
3. Integrate with NATS for communication
4. Follow the Iced/TEA architecture
5. Test both with and without NATS
6. Keep components loosely coupled

## Troubleshooting

### UI Doesn't Appear
- Check for compilation errors: `cargo check`
- Ensure X11/Wayland is available
- Try running a simple example first

### NATS Connection Issues
- Components work in demo mode without NATS
- Check NATS_URL environment variable
- Verify NATS server is running

### Performance Issues
- Enable release mode: `cargo run --release`
- Check for event flooding in logs
- Monitor NATS message rates