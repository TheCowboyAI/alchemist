# CIM Bridge Implementation Plan

## Overview

This plan outlines the implementation of generic AI model bridges for CIM, enabling seamless integration with Ollama, OpenAI, and Anthropic through a composable architecture.

## Phase 1: Core Bridge Infrastructure (Day 1-2)

### Day 1: Generic Bridge Framework
- [ ] Create `cim-bridge` module with generic trait definitions
- [ ] Implement core types:
  - [ ] `AIModelBridge` trait
  - [ ] `ModelRequest` and `ModelResponse` structs
  - [ ] `Message` and `MessageRole` enums
  - [ ] `ModelParameters` configuration
- [ ] Create NATS integration layer:
  - [ ] `BridgeService` generic implementation
  - [ ] Command/Query handlers
  - [ ] Event publishing
- [ ] Add correlation/causation tracking
- [ ] Implement error types and handling

### Day 2: Bridge Utilities
- [ ] Create message conversion utilities
- [ ] Implement streaming support interfaces
- [ ] Add health check framework
- [ ] Create metrics collection
- [ ] Implement rate limiting abstractions
- [ ] Add configuration management
- [ ] Write unit tests for core components

## Phase 2: Ollama Bridge (Day 3-4)

### Day 3: Ollama Implementation
- [ ] Create `cim-bridge-ollama` module
- [ ] Add ollama-rs dependency
- [ ] Implement `OllamaBridge`:
  - [ ] Connection to localhost:11434
  - [ ] Model query implementation
  - [ ] Model listing
  - [ ] Health checks
- [ ] Create Ollama-specific types:
  - [ ] `OllamaConfig`
  - [ ] `OllamaError`
- [ ] Implement message format conversion

### Day 4: Ollama Testing
- [ ] Write integration tests with local Ollama
- [ ] Test various models (llama2, mistral, etc.)
- [ ] Implement streaming support
- [ ] Add performance benchmarks
- [ ] Create example usage
- [ ] Document Ollama setup requirements

## Phase 3: OpenAI Bridge (Day 5-6)

### Day 5: OpenAI Implementation
- [ ] Create `cim-bridge-openai` module
- [ ] Add async-openai dependency
- [ ] Implement `OpenAIBridge`:
  - [ ] API key authentication
  - [ ] Chat completion support
  - [ ] Model listing (GPT-4, GPT-3.5, etc.)
  - [ ] Usage tracking
- [ ] Create OpenAI-specific types:
  - [ ] `OpenAIConfig`
  - [ ] `OpenAIError`
- [ ] Implement rate limiting

### Day 6: OpenAI Features
- [ ] Add function calling support
- [ ] Implement streaming responses
- [ ] Add retry logic for API errors
- [ ] Create cost tracking utilities
- [ ] Write integration tests
- [ ] Add configuration examples

## Phase 4: Anthropic Bridge (Day 7-8)

### Day 7: Anthropic Implementation
- [ ] Create `cim-bridge-anthropic` module
- [ ] Add anthropic-sdk dependency
- [ ] Implement `AnthropicBridge`:
  - [ ] API key authentication
  - [ ] Messages API support
  - [ ] Claude model support
  - [ ] Usage tracking
- [ ] Create Anthropic-specific types:
  - [ ] `AnthropicConfig`
  - [ ] `AnthropicError`
- [ ] Handle Anthropic-specific constraints

### Day 8: Anthropic Features
- [ ] Implement streaming support
- [ ] Add context window management
- [ ] Create retry logic
- [ ] Write integration tests
- [ ] Document model capabilities
- [ ] Add example conversations

## Phase 5: Domain Composition (Day 9-10)

### Day 9: Compose Integration
- [ ] Update `cim-compose` for bridge support
- [ ] Create `CompositionBuilder` extensions
- [ ] Implement event mapping:
  - [ ] Agent events → Bridge commands
  - [ ] Bridge responses → Agent events
- [ ] Create `cim-domain-ollama`:
  - [ ] Composed domain structure
  - [ ] Event routing
  - [ ] Projection support

### Day 10: Composition Testing
- [ ] Test domain composition
- [ ] Verify event flow through NATS
- [ ] Create multi-provider examples
- [ ] Test provider switching
- [ ] Document composition patterns
- [ ] Add performance benchmarks

## Phase 6: Advanced Features (Day 11-12)

### Day 11: Enhanced Functionality
- [ ] Implement conversation memory
- [ ] Add context window management
- [ ] Create prompt templates
- [ ] Add model capability detection
- [ ] Implement fallback strategies
- [ ] Create provider abstraction layer

### Day 12: Monitoring & Observability
- [ ] Implement metrics collection
- [ ] Add distributed tracing
- [ ] Create health check endpoints
- [ ] Add performance monitoring
- [ ] Implement alerting hooks
- [ ] Create dashboard templates

## Technical Implementation Details

### Module Structure
```
cim-bridge/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── traits.rs
│   ├── types.rs
│   ├── service.rs
│   ├── metrics.rs
│   └── errors.rs

cim-bridge-ollama/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── bridge.rs
│   ├── config.rs
│   └── conversion.rs

cim-bridge-openai/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── bridge.rs
│   ├── config.rs
│   ├── streaming.rs
│   └── rate_limit.rs

cim-bridge-anthropic/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── bridge.rs
│   ├── config.rs
│   └── context.rs

cim-domain-ollama/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── composition.rs
│   └── projections.rs
```

### Dependencies
```toml
# cim-bridge/Cargo.toml
[dependencies]
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2.0"
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
futures = "0.3"

# cim-bridge-ollama/Cargo.toml
[dependencies]
cim-bridge = { path = "../cim-bridge" }
ollama-rs = "0.2"
reqwest = { version = "0.12", features = ["json"] }

# cim-bridge-openai/Cargo.toml
[dependencies]
cim-bridge = { path = "../cim-bridge" }
async-openai = "0.24"

# cim-bridge-anthropic/Cargo.toml
[dependencies]
cim-bridge = { path = "../cim-bridge" }
anthropic-sdk = "0.1"  # Or custom implementation
```

### NATS Configuration
```yaml
bridges:
  nats:
    url: "nats://localhost:4222"
    subjects:
      command_prefix: "agent.command"
      query_prefix: "agent.query"
      event_prefix: "agent.event"
      bridge_prefix: "bridge"
```

## Testing Strategy

### Unit Tests
- Test each bridge implementation in isolation
- Mock external API calls
- Test error handling scenarios
- Verify message conversions

### Integration Tests
```rust
#[tokio::test]
async fn test_ollama_bridge_integration() {
    // Start local Ollama instance
    let bridge = OllamaBridge::initialize(OllamaConfig::default()).await?;
    
    let request = ModelRequest {
        model: "llama2".to_string(),
        messages: vec![
            Message {
                role: MessageRole::User,
                content: "Hello, Ollama!".to_string(),
                ..Default::default()
            }
        ],
        ..Default::default()
    };
    
    let response = bridge.query(request).await?;
    assert!(!response.content.is_empty());
}
```

### End-to-End Tests
- Test complete flow from NATS command to AI response
- Verify event publishing
- Test correlation tracking
- Measure latency

## Success Criteria

1. **Functional Requirements**
   - [ ] All three bridges operational
   - [ ] NATS integration working
   - [ ] Domain composition functional
   - [ ] Streaming support implemented
   - [ ] Error handling robust

2. **Performance Requirements**
   - [ ] < 100ms overhead for bridge layer
   - [ ] Support 100+ concurrent requests
   - [ ] Efficient streaming implementation
   - [ ] Memory usage < 100MB per bridge

3. **Quality Requirements**
   - [ ] 90%+ test coverage
   - [ ] Zero memory leaks
   - [ ] Comprehensive documentation
   - [ ] Example applications

## Risks and Mitigation

1. **API Changes**: Use version pinning and adapter patterns
2. **Rate Limits**: Implement queuing and backoff strategies
3. **Network Issues**: Add retry logic and circuit breakers
4. **Model Availability**: Implement fallback providers

## Deliverables

1. **Core Components**
   - Generic bridge framework
   - Three provider implementations
   - Domain composition support

2. **Documentation**
   - API documentation
   - Setup guides for each provider
   - Architecture diagrams
   - Example applications

3. **Testing**
   - Comprehensive test suite
   - Performance benchmarks
   - Integration examples

This implementation will enable CIM to seamlessly integrate with multiple AI providers while maintaining its event-driven architecture and enabling easy switching between models. 