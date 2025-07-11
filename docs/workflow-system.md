# Alchemist Workflow System

The Alchemist workflow system provides a complete workflow execution engine that allows users to define, execute, and monitor workflows through the shell interface.

## Features

- **DAG-based Workflows**: Define workflows as directed acyclic graphs with steps and dependencies
- **Parallel Execution**: Automatically executes independent steps in parallel
- **Multiple Action Types**:
  - Shell commands
  - HTTP requests
  - NATS messaging
  - Sub-workflows
  - Custom actions
- **Conditions and Branching**: Support for conditional execution based on step results
- **Retry Logic**: Configure retry attempts with exponential backoff
- **Real-time Monitoring**: Track execution progress via NATS events
- **YAML/JSON Support**: Define workflows in either format

## Usage

### Create a Workflow

```bash
# Create from YAML file
alchemist workflow new "My Workflow" -f workflow.yaml

# Create empty workflow
alchemist workflow new "Empty Workflow" -d "Description here"
```

### List Workflows

```bash
alchemist workflow list
```

### Show Workflow Details

```bash
alchemist workflow show <workflow-id>
```

### Execute a Workflow

```bash
# Run with no inputs
alchemist workflow run <workflow-id>

# Run with input variables
alchemist workflow run <workflow-id> -i '{"key": "value"}'

# Run with inputs from file
alchemist workflow run <workflow-id> -f inputs.json
```

### Monitor Execution

```bash
# Check execution status
alchemist workflow status <execution-id>

# Stop running workflow
alchemist workflow stop <execution-id>
```

### Import/Export Workflows

```bash
# Import from file
alchemist workflow import path/to/workflow.yaml

# Export to file
alchemist workflow export <workflow-id> -o workflow.yaml -f yaml
```

## Workflow Definition Format

### Basic Structure

```yaml
name: My Workflow
description: Optional description
metadata:
  author: Your Name
  version: "1.0"
  tags: ["tag1", "tag2"]

steps:
  - id: step1
    name: First Step
    description: What this step does
    action:
      type: Command
      command: echo
      args: ["Hello, World!"]
      env: {}
    
  - id: step2
    name: Second Step
    dependencies: ["step1"]
    action:
      type: HttpRequest
      url: https://api.example.com/webhook
      method: POST
      headers:
        Content-Type: application/json
      body:
        message: "Step 1 completed"
```

### Action Types

#### Command Action
Execute shell commands:

```yaml
action:
  type: Command
  command: python
  args: ["script.py", "--arg", "value"]
  env:
    PYTHONPATH: "./lib"
```

#### HTTP Request Action
Make HTTP calls:

```yaml
action:
  type: HttpRequest
  url: https://api.example.com/endpoint
  method: POST
  headers:
    Authorization: "Bearer token"
  body:
    key: value
```

#### NATS Publish Action
Publish messages to NATS:

```yaml
action:
  type: NatsPublish
  subject: my.subject
  payload:
    event: "workflow.step.completed"
    data: {}
```

#### NATS Subscribe Action
Wait for NATS messages:

```yaml
action:
  type: NatsSubscribe
  subject: my.response.subject
  timeout_seconds: 60
```

#### Sub-workflow Action
Execute another workflow:

```yaml
action:
  type: SubWorkflow
  workflow_id: other-workflow-id
  inputs:
    param1: value1
```

### Conditions

Control step execution based on conditions:

```yaml
conditions:
  - type: StepSuccess
    step_id: previous-step
  
  - type: StepFailed
    step_id: other-step
  
  - type: VariableEquals
    name: status
    value: "approved"
```

### Retry Configuration

Configure retry behavior:

```yaml
retry_config:
  max_attempts: 3
  delay_seconds: 5
  backoff_multiplier: 2.0
```

## Example Workflows

See the `examples/workflows/` directory for complete examples:

- `data_pipeline.yaml` - ETL data processing pipeline
- `deployment_workflow.yaml` - Multi-stage deployment with approvals
- `ai_analysis_workflow.yaml` - AI-powered document analysis

## Events

The workflow system publishes events to NATS for real-time monitoring:

- `alchemist.workflow.created` - New workflow created
- `alchemist.workflow.started` - Execution started
- `alchemist.workflow.step.started` - Step execution started
- `alchemist.workflow.step.completed` - Step completed successfully
- `alchemist.workflow.step.failed` - Step failed
- `alchemist.workflow.completed` - Workflow completed
- `alchemist.workflow.failed` - Workflow failed

## Integration with Renderer

The workflow system integrates with the Alchemist renderer to provide visual workflow monitoring:

- Real-time DAG visualization
- Step status updates
- Execution timeline
- Error highlighting

Launch the workflow visualizer:

```bash
alchemist render workflow <workflow-id>
```

## Best Practices

1. **Keep Steps Atomic**: Each step should do one thing well
2. **Use Dependencies**: Explicitly define dependencies between steps
3. **Handle Errors**: Use retry configs and conditions for robust workflows
4. **Version Control**: Store workflow definitions in Git
5. **Test First**: Test workflows with simple inputs before production use
6. **Monitor Events**: Subscribe to NATS events for real-time monitoring
7. **Document Steps**: Use descriptions to explain what each step does