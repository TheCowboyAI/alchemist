# Alchemist Quick Start Guide

Get up and running with Alchemist in 5 minutes! This guide covers installation, basic configuration, and your first commands.

## Prerequisites

- **Operating System**: Linux (NixOS recommended) or macOS
- **Rust**: Latest stable version
- **NATS Server**: For event system (optional for basic features)
- **API Keys**: OpenAI or Anthropic for AI features (optional)

## Installation

### Option 1: From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/thecowboyai/alchemist.git
cd alchemist

# Build with Cargo
cargo build --release

# Install to PATH
cargo install --path .

# Verify installation
alchemist version
```

### Option 2: Using Nix

```bash
# With flakes enabled
nix run github:thecowboyai/alchemist

# Or add to your configuration
{
  inputs.alchemist.url = "github:thecowboyai/alchemist";
  # ...
}
```

## Initial Setup

### 1. Create Configuration Directory

```bash
mkdir -p ~/.alchemist/{dialogs,policies}
```

### 2. Create Basic Configuration

Create `~/.alchemist/alchemist.toml`:

```toml
[general]
home_dir = "~/.alchemist"
default_ai_model = "gpt-4"
dialog_history_path = "~/.alchemist/dialogs"

[ai.providers.openai]
api_key_env = "OPENAI_API_KEY"
models = ["gpt-4", "gpt-3.5-turbo"]

[policy]
storage_path = "~/.alchemist/policies"
validation_enabled = true
```

### 3. Set Environment Variables

```bash
# For AI features (optional)
export OPENAI_API_KEY="your-api-key"

# For NATS integration (optional)
export NATS_URL="nats://localhost:4222"
```

### 4. Start NATS Server (Optional)

For full functionality, start NATS with JetStream:

```bash
# Install NATS
# macOS: brew install nats-server
# Linux: Download from https://nats.io/download/

# Start with JetStream
nats-server -js
```

## Your First Commands

### 1. Check System Status

```bash
# Version and system info
alchemist version

# List available domains
alchemist domain list

# Show project progress
alchemist progress
```

### 2. Test AI Integration

```bash
# List AI providers
alchemist ai providers

# Test AI connectivity
alchemist ai test

# Test specific model
alchemist ai test --model gpt-4
```

### 3. Create Your First Dialog

```bash
# Start a new dialog
alchemist dialog new "My First Chat"

# The shell will enter dialog mode
# Type your messages and press Enter
# Type 'exit' to leave dialog mode

# List your dialogs
alchemist dialog list

# Resume a dialog
alchemist dialog open dialog_<id>
```

### 4. Launch the Dashboard

```bash
# Start the visual dashboard
alchemist render dashboard

# Other visualization options
alchemist render dialog      # Dialog window
alchemist render workflow    # Workflow editor
alchemist render nats       # NATS flow visualizer
```

## Common Workflows

### AI-Assisted Development

```bash
# 1. Create a dialog for code review
alchemist dialog new "Code Review Session"

# 2. In dialog mode, paste your code and ask questions
# Example: "Review this Rust function for performance"

# 3. Export the conversation
alchemist dialog export dialog_<id> --format markdown
```

### Policy Configuration

```bash
# 1. Create a basic policy
alchemist policy new read-only graph

# 2. Add claims
alchemist policy claims add graph:read
alchemist policy claims add graph:list

# 3. View policies
alchemist policy list
```

### Deployment Pipeline

```bash
# 1. List deployment targets
alchemist deploy list

# 2. Create a deployment pipeline
alchemist deploy pipeline "my-release" -e dev -e staging -e prod

# 3. Monitor pipeline
alchemist deploy pipeline-status pipe_<id>

# 4. Handle approvals
alchemist deploy approvals
alchemist deploy approve approval_<id> --approve
```

## Tips for New Users

### 1. Use Tab Completion

The shell supports tab completion for commands:
```bash
alchemist dia<TAB>  # Completes to 'dialog'
```

### 2. Get Help Anywhere

```bash
# General help
alchemist --help

# Command-specific help
alchemist dialog --help
alchemist deploy pipeline --help
```

### 3. Enable Verbose Mode

For debugging issues:
```bash
alchemist -v dialog list
```

### 4. Quick Command Reference

| Task | Command |
|------|---------|
| Start new AI chat | `alchemist dialog new "Title"` |
| List all dialogs | `alchemist dialog list` |
| Show domains | `alchemist domain list` |
| Launch dashboard | `alchemist render dashboard` |
| Test AI | `alchemist ai test` |
| Create policy | `alchemist policy new <name> <domain>` |
| Deploy | `alchemist deploy deploy <target>` |

## Next Steps

Now that you have Alchemist running:

1. **Explore AI Features**: Try different AI models and create dialogs
2. **Learn the Shell**: Read the [Shell Commands Reference](SHELL_COMMANDS.md)
3. **Build Visualizations**: Check the [Renderer API Reference](RENDERER_API.md)
4. **Set Up Automation**: Configure [Deployment Pipelines](DEPLOYMENTS.md)
5. **Customize**: Edit your configuration file for your workflow

## Troubleshooting

### Issue: "NATS connection failed"
**Solution**: Start NATS server with `nats-server -js`

### Issue: "API key not found"
**Solution**: Set environment variable: `export OPENAI_API_KEY="your-key"`

### Issue: "Command not found"
**Solution**: Ensure Alchemist is in your PATH: `export PATH=$PATH:~/.cargo/bin`

### Issue: "Permission denied"
**Solution**: Check file permissions: `chmod +x ~/.cargo/bin/alchemist`

## Getting Help

- **Documentation**: See [full documentation](README.md)
- **Examples**: Check `/examples` directory
- **Issues**: Report at [GitHub](https://github.com/thecowboyai/alchemist/issues)

---

🎉 **Congratulations!** You're now ready to use Alchemist. Start with simple commands and gradually explore more advanced features as you become comfortable with the system.