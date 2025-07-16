# Alchemist Examples

This directory contains examples demonstrating various features of Alchemist.

## UI Examples

### Custom UI Window
```bash
cargo run --example custom_ui_window
```
Shows how to create a custom Iced window that integrates with Alchemist's event system.

### Dialog Automation
```bash
cargo run --example dialog_automation
```
Demonstrates programmatic interaction with the dialog system for automation and batch processing.

## Event System Examples

### Show Event Flow
```bash
cargo run --example show_event_flow
```
Visualizes how events flow through the CIM system.

### Show JetStream Events
```bash
cargo run --example show_jetstream_events
```
Monitors and displays events from NATS JetStream.

## AI Agent Examples

### AI Agent with Memory
```bash
cargo run --example ai_agent_with_memory
```
Shows how to create an AI agent that maintains conversation context.

### AI Agent Dialog Memory
```bash
cargo run --example ai_agent_dialog_memory
```
Demonstrates dialog-based memory management for AI agents.

## Workflow Examples

### State Machine Demo
```bash
cargo run --example state_machine_demo
```
Shows workflow state machine implementation.

### Workflow Demo
```bash
cargo run --example workflow_demo
```
Complete workflow execution example with Bevy visualization.

### Workflow Demo Simple
```bash
cargo run --example workflow_demo_simple
```
Non-graphical workflow demonstration.

## Requirements

- Some examples require NATS to be running:
  ```bash
  nats-server -js
  ```

- AI examples require API keys in `.env`:
  ```env
  ANTHROPIC_API_KEY=your_key_here
  OPENAI_API_KEY=your_key_here
  ```

## Creating Your Own Examples

1. Add your example to `Cargo.toml`:
   ```toml
   [[example]]
   name = "my_example"
   path = "examples/my_example.rs"
   ```

2. Follow the patterns in existing examples:
   - Use `tokio::main` for async examples
   - Handle errors with `anyhow::Result`
   - Add descriptive comments
   - Include usage instructions

3. Test your example:
   ```bash
   cargo run --example my_example
   ```