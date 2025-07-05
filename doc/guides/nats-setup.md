# NATS Setup Guide for CIM

## Overview

NATS is the messaging backbone for the Composable Information Machine (CIM), providing:
- Event-driven communication between domains
- JetStream for persistent event storage
- High-performance pub/sub messaging
- Distributed system coordination

## Quick Start

### 1. Local Development Setup

#### Using Nix (Recommended)
```bash
# The NATS server is already included in our flake.nix
nix develop
nats-server -js
```

#### Using Docker
```bash
docker run -d --name nats-js \
  -p 4222:4222 \
  -p 8222:8222 \
  nats:latest \
  -js
```

#### Manual Installation
```bash
# macOS
brew install nats-server

# Linux
curl -L https://github.com/nats-io/nats-server/releases/download/v2.10.7/nats-server-v2.10.7-linux-amd64.zip -o nats-server.zip
unzip nats-server.zip
sudo cp nats-server-v2.10.7-linux-amd64/nats-server /usr/local/bin/

# Start with JetStream
nats-server -js
```

### 2. Verify Installation

```bash
# Check server is running
nats server info

# Test pub/sub
nats pub test.hello "Hello NATS"
nats sub test.hello &
nats pub test.hello "Message received!"
```

## CIM-Specific Configuration

### 1. Development Configuration (`nats-server.conf`)

```conf
# NATS Server Configuration for CIM Development

# Server identification
server_name: "cim-dev"
port: 4222

# Monitoring
http_port: 8222

# JetStream configuration
jetstream {
  store_dir: "./data/jetstream"
  max_memory_store: 1GB
  max_file_store: 10GB
}

# Logging
debug: false
trace: false
logtime: true
log_file: "./logs/nats-server.log"

# Limits
max_connections: 1000
max_payload: 1MB

# TLS (optional for development)
# tls {
#   cert_file: "./certs/server-cert.pem"
#   key_file: "./certs/server-key.pem"
# }
```

### 2. Create CIM Event Streams

```bash
# Create main event store stream
nats stream add CIM_EVENTS \
  --subjects "events.>" \
  --retention limits \
  --max-msgs=-1 \
  --max-age=365d \
  --storage file \
  --replicas 1 \
  --discard old \
  --dupe-window 2m

# Create domain-specific streams
nats stream add GRAPH_EVENTS \
  --subjects "events.graph.>" \
  --retention limits \
  --max-age=30d

nats stream add WORKFLOW_EVENTS \
  --subjects "events.workflow.>" \
  --retention limits \
  --max-age=30d

# Create dead letter stream
nats stream add DLQ \
  --subjects "dlq.>" \
  --retention limits \
  --max-age=7d
```

### 3. Create Consumers

```bash
# Graph projection consumer
nats consumer add CIM_EVENTS GRAPH_PROJECTION \
  --filter "events.graph.>" \
  --deliver all \
  --ack explicit \
  --replay instant \
  --max-deliver 3

# Workflow processor consumer
nats consumer add CIM_EVENTS WORKFLOW_PROCESSOR \
  --filter "events.workflow.>" \
  --deliver all \
  --ack explicit \
  --wait 5s \
  --max-deliver 5
```

## Integration with CIM

### 1. Environment Variables

```bash
# Add to .env or shell
export NATS_URL="nats://localhost:4222"
export NATS_USER=""  # Empty for development
export NATS_PASSWORD=""  # Empty for development
export NATS_TOKEN=""  # Empty for development
```

### 2. Rust Client Configuration

```rust
use async_nats::{Client, ConnectOptions};

pub async fn connect_to_nats() -> Result<Client, Box<dyn std::error::Error>> {
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    
    let client = async_nats::ConnectOptions::new()
        .name("cim-client")
        .reconnect_buffer_size(1024)
        .connect(&nats_url)
        .await?;
        
    Ok(client)
}
```

### 3. JetStream Setup

```rust
use async_nats::jetstream;

pub async fn setup_jetstream(client: Client) -> Result<jetstream::Context, Box<dyn std::error::Error>> {
    let jetstream = jetstream::new(client);
    
    // Ensure streams exist
    let stream_config = jetstream::stream::Config {
        name: "CIM_EVENTS".to_string(),
        subjects: vec!["events.>".to_string()],
        retention: jetstream::stream::RetentionPolicy::Limits,
        max_messages: 10_000_000,
        max_age: std::time::Duration::from_days(365),
        duplicate_window: std::time::Duration::from_secs(120),
        ..Default::default()
    };
    
    jetstream.create_stream(stream_config).await?;
    
    Ok(jetstream)
}
```

## Testing NATS Integration

### 1. Run Integration Tests

```bash
# Start NATS server
nats-server -js &

# Run NATS integration tests
cargo test --test nats_integration_test -- --ignored --show-output

# Stop NATS server
pkill nats-server
```

### 2. Manual Testing

```bash
# Subscribe to all events
nats sub "events.>"

# Publish test event
nats pub events.test.created '{"id":"123","type":"test"}'

# View stream info
nats stream info CIM_EVENTS

# View consumer info
nats consumer info CIM_EVENTS GRAPH_PROJECTION
```

## Production Configuration

### 1. Clustering

```conf
# nats-cluster.conf
port: 4222
cluster {
  port: 6222
  routes: [
    nats://node1:6222
    nats://node2:6222
    nats://node3:6222
  ]
}

jetstream {
  store_dir: "/data/jetstream"
  max_memory_store: 4GB
  max_file_store: 100GB
}
```

### 2. Security

```conf
# Enable TLS
tls {
  cert_file: "/etc/nats/certs/server-cert.pem"
  key_file: "/etc/nats/certs/server-key.pem"
  ca_file: "/etc/nats/certs/ca.pem"
  verify: true
}

# Authentication
authorization {
  users: [
    {
      user: cim_publisher
      password: $2a$11$...  # bcrypt hash
      permissions: {
        publish: ["events.>"]
        subscribe: ["_INBOX.>"]
      }
    }
    {
      user: cim_consumer
      password: $2a$11$...
      permissions: {
        publish: ["_INBOX.>"]
        subscribe: ["events.>", "$JS.>"]
      }
    }
  ]
}
```

### 3. Monitoring

```bash
# Prometheus metrics
nats-exporter -connz -varz -serverz -http_addr :7777

# Health checks
curl http://localhost:8222/healthz

# View connections
nats server report connections

# Monitor message rates
nats server report jetstream
```

## Troubleshooting

### Common Issues

1. **Connection Refused**
   ```bash
   # Check if server is running
   ps aux | grep nats-server
   
   # Check port availability
   lsof -i :4222
   ```

2. **JetStream Not Enabled**
   ```bash
   # Restart with JetStream
   nats-server -js
   ```

3. **Stream Already Exists**
   ```bash
   # Update existing stream
   nats stream edit CIM_EVENTS
   
   # Or delete and recreate
   nats stream rm CIM_EVENTS
   ```

4. **Consumer Lag**
   ```bash
   # Check consumer status
   nats consumer report CIM_EVENTS
   
   # Reset consumer
   nats consumer rm CIM_EVENTS SLOW_CONSUMER
   ```

### Performance Tuning

```conf
# Optimize for throughput
max_pending: 10MB
write_deadline: "10s"

# Optimize for latency
max_pending: 1MB
write_deadline: "2s"
```

## Best Practices

1. **Subject Naming**
   - Use hierarchical subjects: `events.{domain}.{aggregate}.{event}`
   - Example: `events.graph.node.created`

2. **Message Size**
   - Keep messages under 1MB
   - Use object storage for large payloads

3. **Error Handling**
   - Implement retry logic with exponential backoff
   - Use dead letter queues for failed messages

4. **Monitoring**
   - Set up alerts for consumer lag
   - Monitor stream disk usage
   - Track message rates

## Resources

- [NATS Documentation](https://docs.nats.io/)
- [JetStream Guide](https://docs.nats.io/nats-concepts/jetstream)
- [NATS CLI Reference](https://docs.nats.io/using-nats/nats-tools/nats_cli)
- [async-nats Rust Client](https://github.com/nats-io/nats.rs) 