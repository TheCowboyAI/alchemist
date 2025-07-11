# Alchemist Shell Commands Reference

## Overview

The Alchemist shell provides a comprehensive command-line interface for managing the CIM (Composable Information Machine) system. This document details all available commands, their syntax, and usage examples.

## Command Structure

Commands follow a hierarchical structure:
```
alchemist <command> <subcommand> [options] [arguments]
```

## Core Commands

### 1. AI Management (`ai`)

Manage AI model configurations and providers.

#### `ai providers`
List all available AI providers.
```bash
alchemist ai providers
```

Output:
```
🤖 AI Providers:
─────────────────────────────
✅ openai (4 models)
✅ anthropic (2 models)
✅ ollama (Local) - http://localhost:11434
```

#### `ai models`
List all available AI models.
```bash
alchemist ai models
```

#### `ai test [--model <model>]`
Test AI connectivity and model functionality.
```bash
alchemist ai test --model gpt-4
alchemist ai test  # Tests all configured models
```

### 2. Dialog Management (`dialog`)

Create and manage AI dialog sessions.

#### `dialog new <title> [--model <model>]`
Create a new dialog session.
```bash
alchemist dialog new "Architecture Discussion"
alchemist dialog new "Code Review" --model claude-3-opus
```

#### `dialog list`
List all dialog sessions.
```bash
alchemist dialog list
```

Output:
```
💬 Dialog History:
─────────────────────────────
• Architecture Discussion (gpt-4) - 2024-01-15 14:30
  Created: 2 hours ago | Messages: 12
  
• Code Review (claude-3-opus) - 2024-01-15 10:15
  Created: 6 hours ago | Messages: 8
```

#### `dialog open <id>`
Open an existing dialog for continuation.
```bash
alchemist dialog open dialog_46814b62_20240115_143000
```

#### `dialog delete <id>`
Delete a dialog session.
```bash
alchemist dialog delete dialog_46814b62_20240115_143000
```

#### `dialog export <id> [--format <format>]`
Export a dialog to file.
```bash
alchemist dialog export dialog_123 --format markdown
alchemist dialog export dialog_123 --format json
```

### 3. Policy Management (`policy`)

Manage security policies and claims.

#### `policy list [--domain <domain>]`
List all policies or filter by domain.
```bash
alchemist policy list
alchemist policy list --domain graph
```

#### `policy new <name> <domain>`
Create a new policy.
```bash
alchemist policy new read-only-access graph
```

#### `policy show <id>`
Show detailed policy information.
```bash
alchemist policy show 6df3a8b9-1234-5678-9abc-def012345678
```

#### `policy edit <id>`
Edit an existing policy.
```bash
alchemist policy edit 6df3a8b9-1234-5678-9abc-def012345678
```

#### Claims Management (`policy claims`)

##### `policy claims list`
List all available claims.
```bash
alchemist policy claims list
```

##### `policy claims add <name> [--description <desc>]`
Add a new claim.
```bash
alchemist policy claims add graph:write --description "Write access to graph domain"
```

##### `policy claims remove <name>`
Remove a claim.
```bash
alchemist policy claims remove graph:write
```

### 4. Domain Management (`domain`)

View and manage CIM domains.

#### `domain list`
List all available domains.
```bash
alchemist domain list
```

Output:
```
📦 CIM Domains:
─────────────────────────────
• graph (cim-domain-graph) - Graph operations and visualization
• workflow (cim-domain-workflow) - Workflow execution engine
• agent (cim-domain-agent) - AI agent management
• document (cim-domain-document) - Document lifecycle
• git (cim-domain-git) - Git integration
• nix (cim-domain-nix) - Nix deployment
• policy (cim-domain-policy) - Policy enforcement
• identity (cim-domain-identity) - Identity management
• location (cim-domain-location) - Geospatial data
• organization (cim-domain-organization) - Org hierarchy
• person (cim-domain-person) - Contact management
• dialog (cim-domain-dialog) - Conversation tracking
• conceptualspaces (cim-domain-conceptualspaces) - Semantic reasoning
• bevy (cim-domain-bevy) - 3D visualization
```

#### `domain tree [--root <domain>]`
Display domain hierarchy as a tree.
```bash
alchemist domain tree
alchemist domain tree --root cim
```

#### `domain show <name>`
Show detailed information about a domain.
```bash
alchemist domain show graph
```

#### `domain graph [--format <format>]`
Generate a visualization of domain relationships.
```bash
alchemist domain graph --format mermaid
alchemist domain graph --format dot
alchemist domain graph --format json
```

### 5. Deployment Management (`deploy`)

Manage CIM deployments and automation.

#### `deploy list`
List all configured deployments.
```bash
alchemist deploy list
```

#### `deploy deploy <target> [-d <domain>...]`
Deploy to a target environment.
```bash
alchemist deploy deploy production -d graph -d workflow
alchemist deploy deploy staging -d all
```

#### `deploy status <id>`
Check deployment or task status.
```bash
alchemist deploy status deploy_123
alchemist deploy status task_456
```

#### `deploy generate <target>`
Generate Nix deployment configurations.
```bash
alchemist deploy generate production
```

#### `deploy apply <target>`
Apply Nix deployment to target.
```bash
alchemist deploy apply production
```

#### `deploy validate <target>`
Validate deployment configuration.
```bash
alchemist deploy validate production
```

#### `deploy rollback <deployment-id>`
Rollback a deployment.
```bash
alchemist deploy rollback deploy_123
```

#### Deployment Automation

##### `deploy pipeline <name> -e <env>... [--canary]`
Create a deployment pipeline.
```bash
alchemist deploy pipeline "v2.0-release" -e dev -e staging -e prod
alchemist deploy pipeline "hotfix" -e staging -e prod --canary
```

##### `deploy pipelines`
List all deployment pipelines.
```bash
alchemist deploy pipelines
```

##### `deploy pipeline-status <id>`
Check pipeline execution status.
```bash
alchemist deploy pipeline-status pipe_789
```

##### `deploy approve <id> --approve [-c <comment>]`
Process a deployment approval.
```bash
alchemist deploy approve approval_123 --approve
alchemist deploy approve approval_123 --approve -c "LGTM, tested in staging"
```

##### `deploy approvals`
List pending approvals.
```bash
alchemist deploy approvals
```

### 6. Workflow Management (`workflow`)

Create and execute workflows.

#### `workflow new <name> [--description <desc>] [--file <file>]`
Create a new workflow.
```bash
alchemist workflow new data-pipeline --description "ETL workflow"
alchemist workflow new import-workflow --file workflow.yaml
```

#### `workflow list`
List all workflows.
```bash
alchemist workflow list
```

#### `workflow show <id>`
Show workflow details.
```bash
alchemist workflow show workflow_123
```

#### `workflow execute <id> [--params <key=value>...]`
Execute a workflow.
```bash
alchemist workflow execute workflow_123
alchemist workflow execute workflow_123 --params env=prod --params debug=true
```

#### `workflow status <execution-id>`
Check workflow execution status.
```bash
alchemist workflow status exec_456
```

#### `workflow stop <execution-id>`
Stop a running workflow.
```bash
alchemist workflow stop exec_456
```

### 7. Progress Tracking (`progress`)

View project progress and metrics.

#### `progress [-p <file>] [-f <format>]`
Display project progress.
```bash
alchemist progress
alchemist progress -f json
alchemist progress -p custom-progress.json -f summary
```

Formats:
- `tree` (default): Hierarchical tree view
- `summary`: Condensed summary
- `json`: Raw JSON output
- `detailed`: Comprehensive details

### 8. Renderer Commands (`render`)

Launch visualization windows.

#### `render dashboard`
Launch the main dashboard.
```bash
alchemist render dashboard
```

#### `render dialog`
Launch dialog window.
```bash
alchemist render dialog
```

#### `render nats`
Launch NATS flow visualizer.
```bash
alchemist render nats
```

#### `render workflow`
Launch workflow editor.
```bash
alchemist render workflow
```

#### `render events`
Launch event visualizer.
```bash
alchemist render events
```

#### `render performance`
Launch performance monitor.
```bash
alchemist render performance
```

#### `render bevy`
Launch 3D Bevy renderer.
```bash
alchemist render bevy
```

#### `render launcher`
Launch the unified launcher window.
```bash
alchemist render launcher
```

## Global Options

These options can be used with any command:

- `--help, -h`: Display help information
- `--version, -V`: Display version information
- `--config <file>`: Use alternate configuration file
- `--verbose, -v`: Enable verbose output
- `--quiet, -q`: Suppress non-error output

## Environment Variables

- `ALCHEMIST_HOME`: Override default home directory
- `ALCHEMIST_CONFIG`: Path to configuration file
- `NATS_URL`: NATS server URL (default: nats://localhost:4222)
- `OPENAI_API_KEY`: OpenAI API key for AI features
- `ANTHROPIC_API_KEY`: Anthropic API key for Claude models

## Configuration File

Default location: `~/.alchemist/alchemist.toml`

Example configuration:
```toml
[general]
home_dir = "~/.alchemist"
default_ai_model = "gpt-4"
dialog_history_path = "~/.alchemist/dialogs"

[ai]
[ai.providers.openai]
api_key_env = "OPENAI_API_KEY"
models = ["gpt-4", "gpt-3.5-turbo"]

[ai.providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"
models = ["claude-3-opus", "claude-3-sonnet"]

[policy]
storage_path = "~/.alchemist/policies"
validation_enabled = true
cache_ttl = 300

[deployments.production]
environment = "production"
nats_url = "nats://prod.example.com:4222"
domains = ["graph", "workflow", "agent"]

[renderer]
default_theme = "dark"
window_size = [1200, 800]
```

## Examples

### Complete Workflow Example
```bash
# 1. Configure AI provider
export OPENAI_API_KEY="your-key"
alchemist ai test --model gpt-4

# 2. Create a new dialog
alchemist dialog new "System Design Discussion"

# 3. Create policies for the system
alchemist policy new api-access api
alchemist policy claims add api:read
alchemist policy claims add api:write

# 4. Create a deployment pipeline
alchemist deploy pipeline "release-v1" -e dev -e staging -e prod --canary

# 5. Monitor progress
alchemist progress

# 6. Launch dashboard to visualize
alchemist render dashboard
```

### Policy Configuration Example
```bash
# Create a read-only policy for graph domain
alchemist policy new graph-readonly graph

# Add required claims
alchemist policy claims add graph:read
alchemist policy claims add graph:list

# Show the policy details
alchemist policy show graph-readonly

# Test policy evaluation (in application)
# The policy engine will enforce these rules
```

### Deployment Automation Example
```bash
# Create a canary deployment pipeline
alchemist deploy pipeline "feature-x" -e staging -e prod --canary

# Check pipeline status
alchemist deploy pipeline-status pipe_123

# When approval is needed
alchemist deploy approvals
alchemist deploy approve approval_456 --approve -c "Tested successfully"

# Monitor deployment
alchemist deploy status deploy_789
```

## Error Handling

Common error messages and solutions:

- **"Unknown command"**: Check command spelling and structure
- **"NATS connection failed"**: Ensure NATS server is running
- **"API key not found"**: Set required environment variables
- **"Permission denied"**: Check policy configuration
- **"Deployment failed"**: Review deployment logs and configuration

## Tips and Best Practices

1. **Use tab completion**: The shell supports command completion
2. **Check help**: Use `--help` with any command for details
3. **Verbose mode**: Use `-v` for debugging issues
4. **Regular backups**: Export important dialogs and policies
5. **Test deployments**: Always validate before applying
6. **Monitor progress**: Use the progress command regularly
7. **Review policies**: Ensure security policies are up to date

## Troubleshooting

### NATS Connection Issues
```bash
# Check NATS status
nc -zv localhost 4222

# Start NATS with JetStream
nats-server -js
```

### AI Provider Issues
```bash
# Test specific provider
alchemist ai test --model gpt-4

# Check API key
echo $OPENAI_API_KEY
```

### Permission Issues
```bash
# List current policies
alchemist policy list

# Check claims
alchemist policy claims list
```

## Getting Help

- **In-shell help**: `alchemist --help`
- **Command help**: `alchemist <command> --help`
- **Documentation**: See `/docs` directory
- **Issues**: Report at GitHub repository