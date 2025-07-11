# Alchemist API Reference

## Core API

### Workflow Management

#### `workflow::create`
Creates a new workflow instance.

```rust
use alchemist::workflow::{Workflow, WorkflowConfig};

let config = WorkflowConfig {
    name: "data-processing".to_string(),
    max_agents: 10,
    timeout: Duration::from_secs(300),
};

let workflow = Workflow::create(config).await?;
```

#### `workflow::execute`
Executes a workflow with given inputs.

```rust
let inputs = json!({
    "data": "raw data to process",
    "format": "json"
});

let result = workflow.execute(inputs).await?;
```

### Agent System

#### `Agent` Trait
All AI agents must implement the `Agent` trait:

```rust
#[async_trait]
pub trait Agent: Send + Sync {
    /// Process input and return output
    async fn process(&self, input: Value) -> Result<Value>;
    
    /// Get agent capabilities
    fn capabilities(&self) -> Vec<String>;
    
    /// Agent lifecycle hooks
    async fn on_start(&mut self) -> Result<()>;
    async fn on_stop(&mut self) -> Result<()>;
}
```

#### Creating Custom Agents

```rust
use alchemist::agent::{Agent, AgentConfig};

pub struct CustomAgent {
    config: AgentConfig,
    state: AgentState,
}

#[async_trait]
impl Agent for CustomAgent {
    async fn process(&self, input: Value) -> Result<Value> {
        // Your agent logic here
        Ok(json!({
            "processed": true,
            "data": input
        }))
    }
    
    fn capabilities(&self) -> Vec<String> {
        vec!["text-processing".to_string(), "data-analysis".to_string()]
    }
}
```

### Renderer API

#### `RenderRequest`
Structure for requesting a new renderer window:

```rust
pub struct RenderRequest {
    pub id: String,
    pub renderer: RendererType,
    pub title: String,
    pub data: RenderData,
    pub config: RenderConfig,
}
```

#### Render Data Types

##### Markdown Rendering
```rust
RenderData::Markdown {
    content: String,
    theme: Option<String>, // "light" or "dark"
}
```

##### Chart Visualization
```rust
RenderData::Chart {
    data: json!([
        {
            "name": "Sales",
            "data": [
                {"x": 1, "y": 100},
                {"x": 2, "y": 150},
                {"x": 3, "y": 120}
            ]
        }
    ]),
    chart_type: "line".to_string(),
    options: json!({
        "title": "Monthly Sales",
        "x_label": "Month",
        "y_label": "Revenue"
    })
}
```

### Event System

#### Subscribing to Events
```rust
use alchemist::events::{EventBus, EventType};

let event_bus = EventBus::new();

// Subscribe to workflow events
event_bus.subscribe(EventType::WorkflowStarted, |event| {
    println!("Workflow started: {:?}", event);
}).await;

// Subscribe to agent events
event_bus.subscribe(EventType::AgentProcessing, |event| {
    println!("Agent processing: {:?}", event);
}).await;
```

#### Publishing Events
```rust
use alchemist::events::Event;

let event = Event {
    event_type: EventType::Custom("data-ready".to_string()),
    timestamp: Utc::now(),
    data: json!({
        "source": "data-processor",
        "records": 1000
    }),
};

event_bus.publish(event).await?;
```

## Shell Commands

### Workflow Commands
| Command | Description | Example |
|---------|-------------|---------|
| `workflow create <name>` | Create new workflow | `workflow create etl-pipeline` |
| `workflow list` | List all workflows | `workflow list` |
| `workflow execute <id>` | Execute workflow | `workflow execute wf-123` |
| `workflow status <id>` | Get workflow status | `workflow status wf-123` |

### Agent Commands
| Command | Description | Example |
|---------|-------------|---------|
| `agent create <name>` | Create new agent | `agent create analyzer --model gpt-4` |
| `agent list` | List all agents | `agent list` |
| `agent info <id>` | Get agent details | `agent info agent-456` |

### Renderer Commands
| Command | Description | Example |
|---------|-------------|---------|
| `render markdown <file>` | Open markdown viewer | `render markdown README.md` |
| `render chart <file>` | Open chart viewer | `render chart sales.json` |
| `render report <template>` | Generate report | `render report monthly` |

## Error Handling

All API methods return `Result<T, AlchemistError>`:

```rust
use alchemist::error::AlchemistError;

match workflow.execute(inputs).await {
    Ok(result) => println!("Success: {:?}", result),
    Err(AlchemistError::Timeout) => println!("Workflow timed out"),
    Err(AlchemistError::AgentError(e)) => println!("Agent error: {}", e),
    Err(e) => println!("Other error: {}", e),
}
```

## Best Practices

### 1. Resource Management
- Always clean up workflows after use
- Implement proper timeout handling
- Use connection pooling for agents

### 2. Error Recovery
- Implement retry logic for transient failures
- Use circuit breakers for external services
- Log errors comprehensively

### 3. Performance
- Batch operations when possible
- Use async/await properly
- Monitor memory usage

## Advanced Topics

### Custom Renderers
Implement the `Renderer` trait for custom visualizations:

```rust
pub trait Renderer {
    fn render(&self, data: RenderData) -> Result<()>;
    fn supports(&self, data_type: &str) -> bool;
}
```

### Plugin Development
Create plugins to extend Alchemist functionality:

```rust
use alchemist::plugin::{Plugin, PluginContext};

pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my-plugin"
    }
    
    fn initialize(&mut self, ctx: &PluginContext) -> Result<()> {
        // Plugin initialization
        Ok(())
    }
}
```

---

For more examples and tutorials, visit our [documentation site](https://docs.alchemist.ai).