# Alchemist Performance Optimization Guide

## Overview

This guide covers performance optimization techniques for Alchemist deployments, from development to production scale.

## Performance Metrics

### Key Metrics to Monitor

1. **Response Time**
   - Dialog API response time (target: <100ms)
   - AI streaming latency (target: <500ms first token)
   - Dashboard update frequency (target: 60fps)

2. **Throughput**
   - Events per second (target: 10k+)
   - Concurrent dialogs (target: 1000+)
   - Dashboard connections (target: 100+)

3. **Resource Usage**
   - Memory usage (target: <500MB base)
   - CPU utilization (target: <50% steady state)
   - Network bandwidth (optimize for streaming)

## Optimization Strategies

### 1. Caching Layer

#### Redis Integration
```rust
use redis::{Client, AsyncCommands};
use serde::{Serialize, Deserialize};

#[derive(Clone)]
pub struct CacheManager {
    client: Client,
    ttl: usize,
}

impl CacheManager {
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let mut conn = self.client.get_async_connection().await.ok()?;
        let data: Vec<u8> = conn.get(key).await.ok()?;
        bincode::deserialize(&data).ok()
    }
    
    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let data = bincode::serialize(value)?;
        conn.set_ex(key, data, self.ttl).await?;
        Ok(())
    }
}
```

#### Caching Strategy
- **Dialog History**: Cache recent messages (5min TTL)
- **AI Responses**: Cache common queries (1hr TTL)
- **Domain Status**: Cache health checks (30s TTL)
- **User Sessions**: Cache auth tokens (15min TTL)

### 2. Database Optimization

#### Connection Pooling
```toml
[database]
max_connections = 100
min_connections = 10
connection_timeout = 30
idle_timeout = 600
max_lifetime = 1800
```

#### Query Optimization
```sql
-- Add indexes for common queries
CREATE INDEX idx_dialogs_user_updated ON dialogs(user_id, updated_at DESC);
CREATE INDEX idx_messages_dialog_timestamp ON messages(dialog_id, timestamp DESC);
CREATE INDEX idx_events_domain_created ON events(domain, created_at DESC);

-- Materialized views for dashboards
CREATE MATERIALIZED VIEW domain_stats AS
SELECT 
    domain,
    COUNT(*) as event_count,
    MAX(created_at) as last_event,
    COUNT(DISTINCT session_id) as unique_sessions
FROM events
WHERE created_at > NOW() - INTERVAL '1 hour'
GROUP BY domain;

-- Refresh every minute
CREATE INDEX idx_domain_stats_domain ON domain_stats(domain);
```

### 3. NATS Optimization

#### Subject Design
```
# Efficient subject hierarchy
cim.{domain}.{entity}.{event}
cim.workflow.execution.started
cim.dialog.message.created

# Use wildcards efficiently
cim.*.*.created  # All creation events
cim.dialog.>     # All dialog events
```

#### JetStream Configuration
```yaml
stream:
  name: ALCHEMIST-EVENTS
  subjects: ["cim.>"]
  retention: limits
  max_msgs: 10000000
  max_bytes: 10GB
  max_age: 7d
  max_msg_size: 1MB
  storage: file
  replicas: 3
  placement:
    cluster: "production"
    tags: ["ssd", "fast"]
```

### 4. AI Model Optimization

#### Response Streaming
```rust
pub async fn stream_with_buffer(
    &self,
    prompt: &str,
    buffer_size: usize,
) -> Result<impl Stream<Item = Result<String>>> {
    let (tx, rx) = mpsc::channel(buffer_size);
    
    // Stream in background with buffering
    tokio::spawn(async move {
        let mut buffer = String::with_capacity(100);
        let mut count = 0;
        
        while let Some(token) = stream.next().await {
            buffer.push_str(&token?);
            count += 1;
            
            // Send buffered chunks
            if count >= 5 || token.contains('\n') {
                tx.send(Ok(buffer.clone())).await?;
                buffer.clear();
                count = 0;
            }
        }
        
        // Send remaining
        if !buffer.is_empty() {
            tx.send(Ok(buffer)).await?;
        }
    });
    
    Ok(ReceiverStream::new(rx))
}
```

#### Model Caching
```rust
// Warm up models on startup
pub async fn preload_models(&self) -> Result<()> {
    for model in &self.config.preload_models {
        self.load_model(model).await?;
        
        // Keep model warm with periodic pings
        let manager = self.clone();
        let model = model.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(300)).await;
                let _ = manager.ping_model(&model).await;
            }
        });
    }
    Ok(())
}
```

### 5. UI Optimization

#### Virtual Scrolling
```rust
// Only render visible items
pub fn view_messages(&self) -> Element<Message> {
    let viewport_height = 600.0;
    let item_height = 80.0;
    let visible_count = (viewport_height / item_height).ceil() as usize;
    
    let start_idx = self.scroll_offset;
    let end_idx = (start_idx + visible_count).min(self.messages.len());
    
    scrollable(
        column(
            self.messages[start_idx..end_idx]
                .iter()
                .map(render_message)
                .collect()
        )
    )
}
```

#### Debounced Updates
```rust
pub struct DebouncedUpdater {
    pending: Arc<Mutex<Option<DashboardData>>>,
    interval: Duration,
}

impl DebouncedUpdater {
    pub async fn update(&self, data: DashboardData) {
        *self.pending.lock().await = Some(data);
    }
    
    pub async fn start(self, tx: mpsc::Sender<DashboardData>) {
        loop {
            tokio::time::sleep(self.interval).await;
            
            if let Some(data) = self.pending.lock().await.take() {
                let _ = tx.send(data).await;
            }
        }
    }
}
```

### 6. Memory Optimization

#### Arena Allocation
```rust
use typed_arena::Arena;

pub struct MessageArena {
    arena: Arena<Message>,
}

impl MessageArena {
    pub fn alloc_message(&self, content: String) -> &Message {
        self.arena.alloc(Message {
            role: MessageRole::User,
            content,
            timestamp: Utc::now(),
            tokens: None,
        })
    }
}
```

#### String Interning
```rust
use string_cache::DefaultAtom;

#[derive(Clone)]
pub struct InternedString(DefaultAtom);

impl InternedString {
    pub fn new(s: &str) -> Self {
        Self(DefaultAtom::from(s))
    }
}
```

## Production Tuning

### Linux Kernel Parameters
```bash
# /etc/sysctl.conf
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.ip_local_port_range = 1024 65535
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 15
net.core.netdev_max_backlog = 65535
net.core.rmem_max = 134217728
net.core.wmem_max = 134217728
```

### Container Limits
```yaml
resources:
  limits:
    cpu: "4"
    memory: "4Gi"
  requests:
    cpu: "1"
    memory: "1Gi"
```

### JVM Tuning (for JVM-based services)
```bash
-Xms2g -Xmx2g
-XX:+UseG1GC
-XX:MaxGCPauseMillis=100
-XX:+ParallelRefProcEnabled
-XX:+AlwaysPreTouch
```

## Monitoring Performance

### Prometheus Metrics
```rust
use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref REQUEST_COUNTER: Counter = register_counter!(
        "alchemist_requests_total",
        "Total number of requests"
    ).unwrap();
    
    static ref RESPONSE_TIME: Histogram = register_histogram!(
        "alchemist_response_time_seconds",
        "Response time in seconds"
    ).unwrap();
}

pub async fn track_request<F, T>(name: &str, f: F) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    REQUEST_COUNTER.inc();
    let timer = RESPONSE_TIME.start_timer();
    let result = f.await;
    timer.observe_duration();
    result
}
```

### Custom Dashboards
```json
{
  "dashboard": {
    "title": "Alchemist Performance",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "rate(alchemist_requests_total[5m])"
          }
        ]
      },
      {
        "title": "Response Time P95",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(alchemist_response_time_seconds_bucket[5m]))"
          }
        ]
      }
    ]
  }
}
```

## Benchmarking

### Load Testing Script
```bash
#!/bin/bash
# benchmark.sh

# Test dialog creation
echo "Testing dialog creation..."
ab -n 1000 -c 10 -T 'application/json' \
   -H 'Authorization: Bearer $TOKEN' \
   -p dialog_create.json \
   http://localhost:8080/api/dialogs

# Test streaming
echo "Testing AI streaming..."
wrk -t12 -c400 -d30s \
    -s streaming_test.lua \
    http://localhost:8080/api/stream

# Test dashboard updates
echo "Testing dashboard WebSocket..."
wscat -c ws://localhost:8080/dashboard \
    -x '{"type":"subscribe","domain":"all"}'
```

## Best Practices

1. **Profile First**: Use `cargo flamegraph` to identify bottlenecks
2. **Batch Operations**: Group database writes and NATS publishes
3. **Async Everything**: Never block the event loop
4. **Cache Wisely**: Cache computed values, not raw data
5. **Monitor Always**: Set up alerts for performance degradation

## Troubleshooting Performance Issues

### High CPU Usage
- Check for infinite loops in event handlers
- Look for synchronous I/O operations
- Profile with `perf record -g`

### High Memory Usage
- Check for memory leaks with `valgrind`
- Monitor arena sizes
- Look for unbounded collections

### Slow Responses
- Check database query plans
- Monitor NATS consumer lag
- Verify Redis cache hit rates

### UI Lag
- Reduce update frequency
- Implement virtual scrolling
- Use `requestAnimationFrame`