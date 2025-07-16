# Alchemist Tutorial: Building Your First AI Workflow

Welcome to the Alchemist tutorial! In this guide, we'll build a complete AI-powered content creation workflow that demonstrates the power of multi-agent orchestration.

## Prerequisites

Before we begin, make sure you have:
- Rust installed (1.75 or later)
- NATS server running locally
- Basic familiarity with async Rust

## Step 1: Setting Up

First, let's create a new project and add Alchemist as a dependency:

```bash
cargo new my-ai-workflow
cd my-ai-workflow
```

Add to your `Cargo.toml`:

```toml
[dependencies]
alchemist = "0.1"
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
```

## Step 2: Creating Your First Workflow

Let's create a simple workflow that analyzes text sentiment:

```rust
use alchemist::{
    workflow::{Workflow, WorkflowBuilder},
    agent::prelude::*,
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize Alchemist
    alchemist::init().await?;
    
    // Create a workflow
    let workflow = WorkflowBuilder::new("sentiment-analysis")
        .add_agent("analyzer", GPT4Agent::new())
        .add_agent("formatter", ClaudeAgent::new())
        .connect("analyzer", "formatter")
        .build()?;
    
    // Execute the workflow
    let input = json!({
        "text": "I love using Alchemist! It makes AI workflows so easy."
    });
    
    let result = workflow.execute(input).await?;
    println!("Result: {}", serde_json::to_string_pretty(&result)?);
    
    Ok(())
}
```

## Step 3: Custom Agents

Now let's create a custom agent that processes data in a specific way:

```rust
use alchemist::agent::{Agent, AgentConfig};
use async_trait::async_trait;

pub struct DataEnricher {
    config: AgentConfig,
    api_client: ApiClient,
}

#[async_trait]
impl Agent for DataEnricher {
    async fn process(&self, input: Value) -> Result<Value> {
        // Extract text from input
        let text = input["text"].as_str()
            .ok_or("Missing text field")?;
        
        // Enrich with external data
        let metadata = self.api_client.fetch_metadata(text).await?;
        
        // Return enriched data
        Ok(json!({
            "original": text,
            "metadata": metadata,
            "enriched_at": Utc::now(),
        }))
    }
    
    fn capabilities(&self) -> Vec<String> {
        vec!["data-enrichment".to_string()]
    }
}
```

## Step 4: Visualization

Let's visualize our workflow execution in real-time:

```rust
use alchemist::renderer::{RendererManager, RenderData};

async fn visualize_workflow(workflow: &Workflow) -> Result<()> {
    let renderer = RendererManager::new()?;
    
    // Create a 3D workflow visualization
    let nodes = workflow.get_nodes();
    let edges = workflow.get_edges();
    
    renderer.spawn_graph_3d(
        "Workflow Visualization",
        nodes,
        edges,
    ).await?;
    
    Ok(())
}
```

## Step 5: Working with Charts

Alchemist can visualize data as interactive charts. Here's how to display workflow metrics:

```rust
async fn show_metrics(workflow_id: &str) -> Result<()> {
    let metrics = collect_workflow_metrics(workflow_id).await?;
    
    let chart_data = json!([
        {
            "name": "Execution Time",
            "data": metrics.execution_times.iter().enumerate()
                .map(|(i, &time)| json!({
                    "x": i,
                    "y": time
                }))
                .collect::<Vec<_>>()
        },
        {
            "name": "Token Usage",
            "data": metrics.token_usage.iter().enumerate()
                .map(|(i, &tokens)| json!({
                    "x": i,
                    "y": tokens
                }))
                .collect::<Vec<_>>()
        }
    ]);
    
    let renderer = RendererManager::new()?;
    renderer.spawn(RenderRequest {
        id: Uuid::new_v4().to_string(),
        renderer: RendererType::Iced,
        title: "Workflow Metrics".to_string(),
        data: RenderData::Chart {
            data: chart_data,
            chart_type: "line".to_string(),
            options: json!({
                "title": "Workflow Performance",
                "x_label": "Execution",
                "y_label": "Value",
                "show_legend": true
            }),
        },
        config: RenderConfig::default(),
    }).await?;
    
    Ok(())
}
```

## Step 6: Advanced Patterns

### Pattern 1: Parallel Processing

Execute multiple agents in parallel:

```rust
let workflow = WorkflowBuilder::new("parallel-processing")
    .add_agent("splitter", DataSplitter::new())
    .add_agent("processor1", GPT4Agent::new())
    .add_agent("processor2", ClaudeAgent::new())
    .add_agent("merger", DataMerger::new())
    .connect("splitter", "processor1")
    .connect("splitter", "processor2")
    .connect("processor1", "merger")
    .connect("processor2", "merger")
    .build()?;
```

### Pattern 2: Conditional Routing

Route data based on conditions:

```rust
let workflow = WorkflowBuilder::new("conditional-routing")
    .add_agent("classifier", Classifier::new())
    .add_agent("simple_handler", SimpleHandler::new())
    .add_agent("complex_handler", ComplexHandler::new())
    .add_router("router", |data| {
        match data["complexity"].as_str() {
            Some("simple") => "simple_handler",
            Some("complex") => "complex_handler",
            _ => "simple_handler",
        }
    })
    .connect("classifier", "router")
    .connect("router", "simple_handler")
    .connect("router", "complex_handler")
    .build()?;
```

### Pattern 3: Feedback Loops

Implement iterative improvement:

```rust
let workflow = WorkflowBuilder::new("feedback-loop")
    .add_agent("generator", ContentGenerator::new())
    .add_agent("evaluator", QualityEvaluator::new())
    .add_agent("improver", ContentImprover::new())
    .add_feedback_loop("evaluator", "improver", |result| {
        result["score"].as_f64().unwrap_or(0.0) < 0.8
    })
    .connect("generator", "evaluator")
    .connect("improver", "evaluator")
    .build()?;
```

## Step 7: Monitoring and Debugging

Use the event monitor to track workflow execution:

```rust
use alchemist::monitoring::EventMonitor;

let monitor = EventMonitor::new();

monitor.on_event(|event| {
    match event.event_type {
        EventType::AgentStarted => {
            println!("Agent started: {:?}", event.data["agent_id"]);
        }
        EventType::AgentCompleted => {
            println!("Agent completed: {:?}", event.data["agent_id"]);
        }
        EventType::WorkflowError => {
            eprintln!("Error: {:?}", event.data["error"]);
        }
        _ => {}
    }
}).await;
```

## Step 8: Generating Reports

Combine markdown and charts for comprehensive reports:

```rust
async fn generate_report(workflow_id: &str) -> Result<()> {
    let stats = collect_workflow_stats(workflow_id).await?;
    
    // Create markdown content
    let markdown = format!(r#"
# Workflow Report: {}

Generated: {}

## Summary

- Total Executions: {}
- Success Rate: {:.2}%
- Average Duration: {:.2}s

## Performance Trends

*See chart below for detailed metrics*

## Recommendations

Based on the analysis:
1. Consider optimizing agent prompt lengths
2. Implement caching for repeated queries
3. Monitor token usage for cost optimization
"#, 
        workflow_id,
        Utc::now().format("%Y-%m-%d %H:%M:%S"),
        stats.total_executions,
        stats.success_rate * 100.0,
        stats.avg_duration
    );
    
    // Display report with embedded chart
    let renderer = RendererManager::new()?;
    
    // Show markdown
    renderer.spawn(RenderRequest {
        id: Uuid::new_v4().to_string(),
        renderer: RendererType::Iced,
        title: "Workflow Report".to_string(),
        data: RenderData::Markdown {
            content: markdown,
            theme: Some("light".to_string()),
        },
        config: RenderConfig::default(),
    }).await?;
    
    // Show performance chart
    show_metrics(workflow_id).await?;
    
    Ok(())
}
```

## Next Steps

Congratulations! You've learned the basics of Alchemist. Here are some next steps:

1. **Explore More Agents**: Try different LLM providers and custom agents
2. **Complex Workflows**: Build multi-stage pipelines with branching logic
3. **Integration**: Connect Alchemist to your existing systems via APIs
4. **Optimization**: Learn about performance tuning and cost optimization

## Resources

- [API Reference](API_REFERENCE.md)
- [Example Workflows](https://github.com/alchemist/examples)
- [Community Forum](https://forum.alchemist.ai)
- [Video Tutorials](https://youtube.com/alchemist)

Happy workflow building!