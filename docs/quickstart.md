# Alchemist Quick Start Guide

Get up and running with Alchemist in minutes!

## Prerequisites

- Rust 1.70+ 
- NATS Server (optional, for event streaming)
- API keys for AI providers (optional)

## Installation

```bash
# Clone the repository
git clone https://github.com/thecowboyai/alchemist.git
cd alchemist

# Build the project
cargo build --release

# Install the binary (optional)
cargo install --path .
```

## Basic Setup

1. **Create configuration file** (optional):
   ```bash
   cp alchemist.example.toml alchemist.toml
   ```

2. **Set up API keys** (for AI features):
   ```bash
   # Create .env file
   cat > .env << EOF
   ANTHROPIC_API_KEY=your_anthropic_key
   OPENAI_API_KEY=your_openai_key
   EOF
   ```

3. **Start NATS** (for event streaming):
   ```bash
   # Using Docker
   docker run -p 4222:4222 -p 8222:8222 nats:latest -js

   # Or native installation
   nats-server -js
   ```

## Quick Tour

### 1. Launch the Dashboard

```bash
# Start the dashboard
ia dashboard-local

# You'll see:
# - System status with NATS connection
# - Domain health monitoring  
# - Real-time event stream
# - Interactive domain details
```

### 2. Create an AI Dialog

```bash
# Start a new conversation
ia dialog new --title "My Assistant"

# This opens an Iced window where you can:
# - Chat with AI models
# - Switch between Claude and GPT
# - Export conversations
# - See token usage
```

### 3. Interactive Shell

```bash
# Start interactive mode
ia --interactive

# Available commands:
alchemist> help
alchemist> ai list
alchemist> dialog list
alchemist> workflow list
alchemist> domain status
```

### 4. Create a Workflow

```yaml
# Save as my-workflow.yaml
id: hello-workflow
name: Hello World Workflow
version: 1.0.0
steps:
  - id: greet
    name: Greeting
    action:
      type: execute
      command: echo
      args: ["Hello from Alchemist!"]
```

```bash
# Import and run
ia workflow import my-workflow.yaml
ia workflow run hello-workflow
```

## Key Features to Explore

### Event Monitoring
```bash
# Watch all events
ia event watch

# Filter specific domains
ia event watch --filter "domain:workflow"

# Export events
ia event export --format json --output events.json
```

### AI Model Management
```bash
# List available models
ia ai list

# Test a model
ia ai test claude-3-sonnet

# Add a new model
ia ai add my-llama --provider ollama --endpoint http://localhost:11434
```

### Policy Engine
```bash
# List policies
ia policy list

# Create a rate limit policy
ia policy add api-rate-limit --domain agent --rules rate_limit.yaml
```

## Common Use Cases

### 1. AI-Powered Development Assistant
```bash
# Create a code review dialog
ia dialog new --title "Code Review" --model claude-3-opus

# Ask questions about your code
# Get suggestions and improvements
# Export the conversation for documentation
```

### 2. Workflow Automation
```bash
# Create deployment workflow
ia workflow import deploy.yaml

# Execute with variables
ia workflow run deploy --var environment=staging
```

### 3. System Monitoring
```bash
# Launch dashboard
ia dashboard

# Monitor specific domain
ia event watch --filter "domain:agent"

# Set up alerts
ia event alert add high-error-rate --condition "errors > 10"
```

## Next Steps

1. **Read the UI Guide**: Learn about dashboard and dialog customization
2. **Explore Examples**: Check out `examples/` directory
3. **Join Community**: Contribute to the project
4. **Build Extensions**: Create custom UI components

## Troubleshooting

### NATS Connection Issues
```bash
# Check NATS status
nats server ping

# Verify configuration
grep nats_url alchemist.toml
```

### UI Not Launching
```bash
# Check display server (Linux)
echo $DISPLAY

# Run with debug logging
RUST_LOG=alchemist=debug ia dashboard-local
```

### AI Model Errors
```bash
# Test API keys
ia ai test claude-3-sonnet

# Check rate limits
ia policy status api-rate-limit
```

## Getting Help

- Run `ia help` for command documentation
- Check `docs/` for detailed guides
- Report issues on GitHub
- Join our Discord community

Happy building with Alchemist! 🚀