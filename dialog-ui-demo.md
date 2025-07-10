# Alchemist Dialog UI Demo

## Overview

The Alchemist Dialog UI provides a graphical interface for AI conversations using the Iced renderer. This document demonstrates the key features and API.

## Architecture

### 1. Dialog UI Components

The dialog UI is built with Iced and provides:

- **Message Display**: Shows conversation history with role-based styling
  - System messages: Gray background
  - User messages: Blue background, right-aligned  
  - Assistant messages: Light gray background, left-aligned

- **Input Area**: Text input field with Send button
- **Controls**: Clear dialog, Export, and Close buttons
- **Model Display**: Shows current AI model in header

### 2. Renderer API

The main application communicates with the renderer through:

```rust
// Spawn a new dialog window
let window_id = renderer_manager.spawn_dialog(
    "AI Dialog",                    // Window title
    dialog_id,                      // Unique dialog ID
    "gpt-4",                       // AI model name
    messages,                      // Initial messages
    Some("System prompt"),         // Optional system prompt
).await?;
```

### 3. Message Streaming

The renderer API supports real-time message streaming:

```rust
// Send streaming tokens to dialog
renderer_api.send_dialog_command(
    &window_id,
    DialogCommand::StreamToken { token: "Hello" }
).await?;

// Complete the stream
renderer_api.send_dialog_command(
    &window_id,
    DialogCommand::CompleteStream
).await?;
```

### 4. Event Handling

The dialog UI sends events back to the main application:

```rust
match event {
    DialogEvent::UserMessage { content } => {
        // User sent a message - send to AI
    }
    DialogEvent::ClearRequested => {
        // User wants to clear the dialog
    }
    DialogEvent::ExportRequested { format } => {
        // User wants to export the dialog
    }
}
```

## Usage Example

### From the Interactive Shell

```bash
# Launch interactive shell
./run-ia.sh -i

# Open a dialog UI window
dialog ui

# Open dialog with specific ID
dialog ui my-dialog-123
```

### From Code

```rust
use alchemist::renderer::{RendererManager, DialogMessage};

// Create renderer manager
let renderer_manager = RendererManager::new()?;

// Create initial messages
let messages = vec![
    DialogMessage {
        role: "system".to_string(),
        content: "You are a helpful assistant.".to_string(),
        timestamp: Utc::now(),
    },
];

// Spawn dialog window
let window_id = renderer_manager.spawn_dialog(
    "Chat with GPT-4",
    "dialog-123",
    "gpt-4",
    messages,
    None,
).await?;
```

## Current Implementation Status

### ✅ Completed
- Dialog window rendering structure
- Message type definitions
- Renderer spawn API
- Command-line integration
- Basic renderer process management

### 🚧 In Progress
- Actual Iced UI implementation (currently placeholder)
- IPC communication between processes
- Message streaming support
- Event handling pipeline

### 📋 TODO
- Full Iced dialog UI with all features
- WebSocket/IPC bridge for real-time updates
- Copy to clipboard functionality
- Export dialog to file
- Theme customization
- Multi-model switching

## Testing

Run the test script to see the dialog UI in action:

```bash
./test-dialog-ui.sh
```

In a GUI environment, this would open an Iced window with the dialog interface. Currently, it demonstrates the API and shows placeholder output.

## API Reference

### RenderData::Dialog

```rust
Dialog {
    dialog_id: String,
    ai_model: String,
    messages: Vec<DialogMessage>,
    system_prompt: Option<String>,
}
```

### DialogMessage

```rust
pub struct DialogMessage {
    pub role: String,      // "system", "user", "assistant"
    pub content: String,
    pub timestamp: DateTime<Utc>,
}
```

### RendererCommand

```rust
pub enum DialogCommand {
    AddMessage { role: String, content: String },
    StreamToken { token: String },
    CompleteStream,
    SetLoading { loading: bool },
    UpdateSystemPrompt { prompt: String },
    ClearMessages,
}
```

## Future Enhancements

1. **Rich Message Content**: Support for markdown, code blocks, tables
2. **File Attachments**: Drag & drop file support
3. **Voice Input**: Speech-to-text integration
4. **Multi-Turn Editing**: Edit previous messages and regenerate
5. **Conversation Branching**: Explore different conversation paths
6. **Model Comparison**: Side-by-side model responses
7. **Custom Themes**: Light/dark mode, custom colors
8. **Keyboard Shortcuts**: Quick actions via hotkeys