# AI Provider Setup Guide

This guide explains how to set up and use real AI providers (OpenAI, Anthropic Claude, and Ollama) with the CIM Agent domain for graph analysis and workflow optimization.

## Overview

The CIM Agent domain supports multiple AI providers for analyzing graphs and workflows:

- **OpenAI** - GPT-4 and GPT-4-Turbo for advanced analysis
- **Anthropic** - Claude 3.5 Sonnet for semantic and pattern analysis
- **Ollama** - Local models for privacy-conscious deployments
- **Mock** - For testing without API costs

## Prerequisites

### API Keys

For cloud providers, you'll need API keys:

1. **OpenAI**: Get your API key from [platform.openai.com](https://platform.openai.com)
2. **Anthropic**: Get your API key from [console.anthropic.com](https://console.anthropic.com)

### Local Setup (Ollama)

For Ollama, install and run locally:

```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Start Ollama service
ollama serve

# Pull a model (e.g., llama3.2)
ollama pull llama3.2
```

## Configuration

### Environment Variables

Set up your environment variables:

```bash
# Choose default provider
export DEFAULT_AI_PROVIDER=openai  # or anthropic, ollama, mock

# Provider-specific keys
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...
export OLLAMA_HOST=http://localhost:11434  # Optional, defaults to this
```

### Using `.env` File

Create a `.env` file in your project root:

```env
DEFAULT_AI_PROVIDER=openai
OPENAI_API_KEY=sk-your-openai-key
ANTHROPIC_API_KEY=sk-ant-your-anthropic-key
OLLAMA_HOST=http://localhost:11434
```

### Programmatic Configuration

You can also configure providers programmatically:

```rust
use cim_domain_agent::ai_providers::{ProviderConfig, AIProviderFactory};

// OpenAI
let provider = AIProviderFactory::create_provider(&ProviderConfig::OpenAI {
    api_key: "sk-...".to_string(),
    model: "gpt-4-turbo".to_string(),
})?;

// Anthropic
let provider = AIProviderFactory::create_provider(&ProviderConfig::Anthropic {
    api_key: "sk-ant-...".to_string(),
    model: "claude-3-5-sonnet-20241022".to_string(),
})?;

// Ollama
let provider = AIProviderFactory::create_provider(&ProviderConfig::Ollama {
    host: "http://localhost:11434".to_string(),
    model: "llama3.2".to_string(),
})?;
```

## Available Models

### OpenAI Models
- `gpt-4-turbo` - Latest GPT-4 Turbo (recommended)
- `gpt-4` - Standard GPT-4
- `gpt-3.5-turbo` - Faster, cheaper option

### Anthropic Models
- `claude-3-5-sonnet-20241022` - Latest Claude 3.5 Sonnet (recommended)
- `claude-3-opus-20240229` - Most capable but slower
- `claude-3-haiku-20240307` - Fastest, most economical

### Ollama Models
- `llama3.2` - Latest Llama model
- `mistral` - Mistral 7B
- `codellama` - Code-focused model
- Any model available via `ollama list`

## Usage Examples

### Basic Graph Analysis

```rust
use cim_domain_agent::ai_providers::*;
use cim_domain_agent::value_objects::AnalysisCapability;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load provider from environment
    let config = load_provider_config()?;
    let provider = AIProviderFactory::create_provider(&config)?;
    
    // Create graph data
    let graph_data = GraphData {
        graph_id: uuid::Uuid::new_v4(),
        nodes: vec![/* your nodes */],
        edges: vec![/* your edges */],
        metadata: HashMap::new(),
    };
    
    // Analyze the graph
    let result = provider.analyze_graph(
        graph_data,
        AnalysisCapability::WorkflowOptimization,
        HashMap::new(),
    ).await?;
    
    println!("Analysis: {}", result.summary);
    println!("Confidence: {:.0}%", result.confidence_score * 100.0);
    
    Ok(())
}
```

### Advanced Analysis with Parameters

```rust
use serde_json::json;

// Workflow optimization with specific goals
let params = HashMap::from([
    ("focus".to_string(), json!("bottleneck_detection")),
    ("optimization_goals".to_string(), json!([
        "reduce_processing_time",
        "increase_parallelization",
        "minimize_costs"
    ])),
    ("constraints".to_string(), json!({
        "budget_limit": 50000,
        "timeline": "3_months",
        "maintain_quality": true
    })),
]);

let result = provider.analyze_graph(
    graph_data,
    AnalysisCapability::WorkflowOptimization,
    params,
).await?;
```

### Getting Transformation Suggestions

```rust
// Request specific transformations
let transformations = provider.suggest_transformations(
    graph_data,
    vec![
        "Reduce total processing time by 30%".to_string(),
        "Automate manual processes".to_string(),
        "Improve error handling and recovery".to_string(),
    ],
    HashMap::from([
        ("max_cost".to_string(), json!(100000)),
        ("risk_tolerance".to_string(), json!("medium")),
    ]),
).await?;

for transform in transformations {
    println!("Transformation: {}", transform.title);
    println!("Expected improvement: {:.0}%", transform.expected_improvement * 100.0);
    
    for step in &transform.steps {
        println!("  - {}", step);
    }
}
```

## Analysis Capabilities

### GraphAnalysis
General graph structure analysis, identifying patterns and anomalies.

```rust
let result = provider.analyze_graph(
    graph_data,
    AnalysisCapability::GraphAnalysis,
    HashMap::new(),
).await?;
```

### WorkflowOptimization
Focuses on improving workflow efficiency and performance.

```rust
let result = provider.analyze_graph(
    graph_data,
    AnalysisCapability::WorkflowOptimization,
    HashMap::from([
        ("focus".to_string(), json!("bottleneck_removal")),
    ]),
).await?;
```

### PatternDetection
Identifies recurring patterns and anti-patterns in the graph.

```rust
let result = provider.analyze_graph(
    graph_data,
    AnalysisCapability::PatternDetection,
    HashMap::new(),
).await?;
```

### SemanticAnalysis
Analyzes the meaning and relationships in the graph.

```rust
let result = provider.analyze_graph(
    graph_data,
    AnalysisCapability::SemanticAnalysis,
    HashMap::from([
        ("depth".to_string(), json!("comprehensive")),
    ]),
).await?;
```

### Custom Analysis
Provide your own analysis prompt.

```rust
let result = provider.analyze_graph(
    graph_data,
    AnalysisCapability::Custom("Analyze this graph for security vulnerabilities".to_string()),
    HashMap::new(),
).await?;
```

## Cost Considerations

### OpenAI Pricing (as of 2024)
- GPT-4 Turbo: ~$0.01/1K input tokens, $0.03/1K output tokens
- GPT-3.5 Turbo: ~$0.0005/1K input tokens, $0.0015/1K output tokens

### Anthropic Pricing (as of 2024)
- Claude 3.5 Sonnet: ~$0.003/1K input tokens, $0.015/1K output tokens
- Claude 3 Haiku: ~$0.00025/1K input tokens, $0.00125/1K output tokens

### Ollama
- Free (runs locally)
- Requires local compute resources

## Best Practices

### 1. Start with Mock Provider
Test your integration with the mock provider before using paid APIs:

```rust
let provider = AIProviderFactory::create_provider(&ProviderConfig::Mock)?;
```

### 2. Use Appropriate Models
- Use GPT-3.5 or Claude Haiku for simple analyses
- Reserve GPT-4 or Claude Sonnet for complex workflows
- Use Ollama for privacy-sensitive data

### 3. Batch Analyses
Combine multiple small analyses into larger requests to reduce API calls.

### 4. Cache Results
Store analysis results to avoid repeated API calls:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

struct AnalysisCache {
    cache: Arc<Mutex<HashMap<String, AnalysisResult>>>,
}

impl AnalysisCache {
    async fn get_or_analyze(
        &self,
        key: String,
        provider: &dyn GraphAnalysisProvider,
        graph_data: GraphData,
        capability: AnalysisCapability,
    ) -> Result<AnalysisResult, Box<dyn std::error::Error>> {
        // Check cache first
        let cache = self.cache.lock().await;
        if let Some(result) = cache.get(&key) {
            return Ok(result.clone());
        }
        drop(cache);
        
        // Perform analysis
        let result = provider.analyze_graph(
            graph_data,
            capability,
            HashMap::new(),
        ).await?;
        
        // Store in cache
        let mut cache = self.cache.lock().await;
        cache.insert(key, result.clone());
        
        Ok(result)
    }
}
```

### 5. Handle Rate Limits
Implement exponential backoff for rate limit errors:

```rust
use tokio::time::{sleep, Duration};

async fn analyze_with_retry(
    provider: &dyn GraphAnalysisProvider,
    graph_data: GraphData,
    capability: AnalysisCapability,
    max_retries: u32,
) -> Result<AnalysisResult, Box<dyn std::error::Error>> {
    let mut retries = 0;
    let mut delay = Duration::from_millis(1000);
    
    loop {
        match provider.analyze_graph(graph_data.clone(), capability.clone(), HashMap::new()).await {
            Ok(result) => return Ok(result),
            Err(e) if retries < max_retries => {
                eprintln!("Analysis failed, retrying in {:?}: {}", delay, e);
                sleep(delay).await;
                delay *= 2; // Exponential backoff
                retries += 1;
            }
            Err(e) => return Err(e.into()),
        }
    }
}
```

## Troubleshooting

### OpenAI Issues
- **Invalid API Key**: Check that your key starts with `sk-`
- **Rate Limits**: Implement retry logic with exponential backoff
- **Model Not Found**: Ensure you're using a valid model name

### Anthropic Issues
- **Invalid API Key**: Check that your key starts with `sk-ant-`
- **Content Policy**: Some analyses may trigger content filters
- **Region Restrictions**: Some regions may have limited access

### Ollama Issues
- **Connection Refused**: Ensure Ollama is running (`ollama serve`)
- **Model Not Found**: Pull the model first (`ollama pull model-name`)
- **Out of Memory**: Try smaller models or increase system RAM

## Testing

Run integration tests with real providers:

```bash
# Set up environment
export OPENAI_API_KEY=your-key
export ANTHROPIC_API_KEY=your-key

# Run ignored tests
cargo test --test real_ai_provider_integration_test -- --ignored

# Run specific provider test
cargo test --test real_ai_provider_integration_test -- --ignored test_openai_real_analysis
```

## Security Considerations

1. **Never commit API keys** to version control
2. **Use environment variables** or secure key management
3. **Validate and sanitize** graph data before sending to APIs
4. **Be cautious** with sensitive data in graphs
5. **Use Ollama** for highly sensitive workflows

## Next Steps

1. Start with the mock provider to test your integration
2. Set up API keys for your chosen provider
3. Run the example demos in `cim-domain-agent/examples/`
4. Integrate AI analysis into your workflow automation
5. Monitor costs and optimize usage patterns

For more examples, see:
- `examples/ai_powered_workflow_automation.rs`
- `examples/ai_real_providers_demo.rs`
- `tests/real_ai_provider_integration_test.rs` 