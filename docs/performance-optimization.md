# Performance Optimization Guide

This guide covers the performance optimization features implemented in Alchemist, including caching, rate limiting, and performance monitoring.

## Overview

Alchemist includes several performance optimization features:

1. **Caching Layer** - Redis-based caching with in-memory fallback
2. **Rate Limiting** - Token bucket algorithm for API protection
3. **Performance Monitoring** - Real-time metrics collection and analysis
4. **Benchmarking Tools** - Measure and compare performance improvements

## Caching

### Configuration

Enable caching in your `alchemist.toml`:

```toml
[cache]
redis_url = "redis://localhost:6379"  # Optional, falls back to memory cache
default_ttl = 3600                     # Default TTL in seconds
max_memory_items = 10000              # Max items in memory cache
```

### Architecture

The caching system uses a layered approach:

```
┌─────────────────┐
│   Application   │
└────────┬────────┘
         │
┌────────▼────────┐
│  Cache Manager  │
└────────┬────────┘
         │
┌────────▼────────┐
│ Layered Cache   │
├─────────────────┤
│ Primary: Redis  │
│ Fallback: Memory│
└─────────────────┘
```

### Usage

The caching is automatically integrated into:

- **AI Completions** - Responses are cached based on prompt hash
- **Dialog Summaries** - Generated summaries are cached
- **System Metrics** - Aggregated metrics are cached

### Cache Keys

- AI completions: `ai:completion:{model}:{prompt_hash}`
- Dialog messages: `dialog:messages:{dialog_id}`
- Dialog summaries: `dialog:summary:{dialog_id}`
- Metrics: `metrics:{type}:{period}`

## Rate Limiting

### Token Bucket Algorithm

Each user/key gets a bucket with:
- **Capacity**: Maximum tokens
- **Refill Rate**: Tokens added per second
- **Window**: Time period for tracking

### Configuration

Rate limits are configured per model:

```rust
// Claude models
"claude-3-sonnet" => RateLimiterConfig {
    capacity: 100,
    refill_rate: 2.0,
    window: Duration::from_secs(60),
}

// GPT models
"gpt-4" => RateLimiterConfig {
    capacity: 60,
    refill_rate: 1.0,
    window: Duration::from_secs(60),
}
```

### Hierarchical Rate Limiting

Different tiers get different limits:

- **Free Tier**: 100 requests/hour
- **Pro Tier**: 1000 requests/hour
- **Enterprise Tier**: 10000 requests/hour

### Circuit Breaker

Protects against cascading failures:

```rust
CircuitBreakerConfig {
    failure_threshold: 5,     // Failures before opening
    success_threshold: 2,     // Successes to close
    timeout: Duration::from_secs(60),
}
```

## Performance Monitoring

### Metrics Collected

1. **Request Latencies**
   - Average, min, max, p50, p95, p99
   - Success/failure rates

2. **Cache Performance**
   - Hit/miss rates
   - Eviction counts

3. **Rate Limiting**
   - Violations by key
   - Token consumption

4. **System Resources**
   - Memory usage
   - CPU usage
   - Open connections

5. **AI Model Performance**
   - Requests per model
   - Token usage
   - Error rates

### Real-time Dashboard

View performance metrics in the dashboard:

```bash
alchemist perf stats
```

Output:
```
🔍 Performance Statistics:
  Cache hit rate: 85.3%
  Rate limit violations: 2
  Memory usage: 234.5 MB
  
Model Performance:
  claude-3-sonnet:
    Requests: 1234
    Success rate: 99.2%
    Avg latency: 523ms
```

### Export Metrics

Export for analysis:

```bash
alchemist perf export --format json > metrics.json
```

## Benchmarking

### Running Benchmarks

Compare standard vs enhanced AI managers:

```bash
alchemist benchmark --model claude-3-sonnet --requests 100
```

### Benchmark Configuration

```rust
BenchmarkConfig {
    requests: 100,
    concurrent_workers: 4,
    warmup_requests: 5,
    test_prompt: "What is the capital of France?",
}
```

### Results

```
📊 Benchmark Results

Test: claude-3-sonnet (Standard)
──────────────────────────────────────────────────
Total Requests: 100
Successful: 100 (100.0%)
Throughput: 12.5 req/s

Latency Statistics:
  Average: 523.2ms
  P95: 892.1ms
  P99: 1234.5ms

Test: claude-3-sonnet (Enhanced)
──────────────────────────────────────────────────
Total Requests: 100
Successful: 100 (100.0%)
Throughput: 45.2 req/s
Cache Hit Rate: 85.0%

Latency Statistics:
  Average: 142.3ms
  P95: 234.5ms
  P99: 456.7ms

🔄 Comparison
──────────────────────────────────────────────────
Average Latency Improvement: 3.68x faster
Throughput Improvement: 3.62x higher
Cache Effectiveness: 85.0% hit rate
```

## Best Practices

### 1. Cache Warming

Warm up caches before high load:

```rust
// Pre-cache common queries
for prompt in common_prompts {
    ai_manager.get_completion("claude-3-sonnet", prompt).await?;
}
```

### 2. Rate Limit Monitoring

Monitor rate limits in production:

```rust
if !rate_limiter.check_model_limit(model, user).await? {
    // Log and alert
    warn!("Rate limit exceeded for user: {}", user);
    metrics.increment_counter("rate_limit_violations");
}
```

### 3. Circuit Breaker Usage

Wrap external calls in circuit breakers:

```rust
circuit_breaker.call(async {
    external_api.call().await
}).await?
```

### 4. Performance Testing

Include performance tests in CI:

```yaml
- name: Run Performance Tests
  run: |
    cargo test --test performance -- --nocapture
    cargo bench
```

## Troubleshooting

### Cache Misses

High cache miss rate? Check:
- TTL settings (too short?)
- Cache key generation (too specific?)
- Memory limits (evicting too early?)

### Rate Limit Issues

Getting rate limited? Consider:
- Implementing request queuing
- Using exponential backoff
- Upgrading to higher tier

### Memory Usage

High memory usage? Try:
- Reducing cache sizes
- Shorter TTLs
- More aggressive eviction

## Future Improvements

1. **Distributed Caching** - Redis Cluster support
2. **Advanced Rate Limiting** - Sliding window, distributed counters
3. **ML-based Optimization** - Predict and pre-cache queries
4. **Request Coalescing** - Deduplicate in-flight requests
5. **Adaptive Caching** - Dynamic TTL based on usage patterns