# CIM Monitoring Setup Guide

This guide covers comprehensive monitoring setup for CIM production deployments.

## Overview

CIM provides extensive monitoring capabilities through:
- **Metrics**: Prometheus-compatible metrics endpoint
- **Logging**: Structured JSON logging
- **Tracing**: OpenTelemetry distributed tracing
- **Health Checks**: Kubernetes-compatible health endpoints

## Metrics Collection

### Prometheus Setup

#### 1. Install Prometheus

```bash
# Using Nix
nix-env -iA nixpkgs.prometheus

# Or via Docker
docker run -d \
  --name prometheus \
  -p 9090:9090 \
  -v /etc/prometheus:/etc/prometheus \
  -v /var/lib/prometheus:/prometheus \
  prom/prometheus
```

#### 2. Configure Prometheus

```yaml
# /etc/prometheus/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'cim-production'
    region: 'us-east-1'

# Alerting configuration
alerting:
  alertmanagers:
    - static_configs:
        - targets: ['localhost:9093']

# Rules files
rule_files:
  - "alerts/*.yml"

scrape_configs:
  # CIM Application Metrics
  - job_name: 'cim-alchemist'
    static_configs:
      - targets: ['localhost:9091']
    metrics_path: '/metrics'
    scrape_interval: 10s

  # NATS Server Metrics
  - job_name: 'nats'
    static_configs:
      - targets: ['localhost:8222']
    metrics_path: '/metrics'

  # Node Exporter (System Metrics)
  - job_name: 'node'
    static_configs:
      - targets: ['localhost:9100']
```

### CIM Metrics

CIM exposes the following key metrics:

```rust
// Event Processing Metrics
cim_events_processed_total{domain="graph", event_type="NodeAdded"}
cim_events_processing_duration_seconds{domain="graph", quantile="0.99"}
cim_events_failed_total{domain="graph", reason="validation_error"}

// Command Metrics
cim_commands_received_total{domain="graph", command_type="CreateNode"}
cim_commands_duration_seconds{domain="graph", command_type="CreateNode", quantile="0.99"}
cim_commands_failed_total{domain="graph", reason="unauthorized"}

// Query Metrics
cim_queries_total{domain="graph", query_type="FindNodes"}
cim_queries_duration_seconds{domain="graph", query_type="FindNodes", quantile="0.99"}
cim_query_cache_hits_total{domain="graph"}
cim_query_cache_misses_total{domain="graph"}

// NATS Connection Metrics
cim_nats_connections_active
cim_nats_messages_sent_total
cim_nats_messages_received_total
cim_nats_reconnects_total

// System Metrics
cim_memory_usage_bytes
cim_goroutines_count
cim_event_store_size_bytes
cim_projection_lag_seconds{projection="GraphSummary"}
```

## Grafana Dashboards

### 1. Install Grafana

```bash
# Using Nix
nix-env -iA nixpkgs.grafana

# Or via Docker
docker run -d \
  --name grafana \
  -p 3000:3000 \
  -v /var/lib/grafana:/var/lib/grafana \
  grafana/grafana
```

### 2. Import CIM Dashboards

Create these dashboards in Grafana:

#### CIM Overview Dashboard

```json
{
  "dashboard": {
    "title": "CIM Overview",
    "panels": [
      {
        "title": "Event Processing Rate",
        "targets": [{
          "expr": "rate(cim_events_processed_total[5m])"
        }]
      },
      {
        "title": "Command Success Rate",
        "targets": [{
          "expr": "rate(cim_commands_received_total[5m]) - rate(cim_commands_failed_total[5m])"
        }]
      },
      {
        "title": "Query Latency (p99)",
        "targets": [{
          "expr": "histogram_quantile(0.99, rate(cim_queries_duration_seconds_bucket[5m]))"
        }]
      },
      {
        "title": "Active NATS Connections",
        "targets": [{
          "expr": "cim_nats_connections_active"
        }]
      }
    ]
  }
}
```

#### Domain-Specific Dashboard

```json
{
  "dashboard": {
    "title": "CIM Graph Domain",
    "panels": [
      {
        "title": "Graph Operations",
        "targets": [{
          "expr": "sum by (operation) (rate(cim_graph_operations_total[5m]))"
        }]
      },
      {
        "title": "Node Count by Type",
        "targets": [{
          "expr": "cim_graph_nodes_total"
        }]
      },
      {
        "title": "Edge Count",
        "targets": [{
          "expr": "cim_graph_edges_total"
        }]
      },
      {
        "title": "Spatial Index Performance",
        "targets": [{
          "expr": "histogram_quantile(0.99, rate(cim_spatial_query_duration_seconds_bucket[5m]))"
        }]
      }
    ]
  }
}
```

## Alerting Rules

### 1. Configure Alert Rules

```yaml
# /etc/prometheus/alerts/cim-alerts.yml
groups:
  - name: cim_alerts
    interval: 30s
    rules:
      # High Error Rate
      - alert: HighEventProcessingErrorRate
        expr: rate(cim_events_failed_total[5m]) > 0.05
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High event processing error rate"
          description: "{{ $labels.domain }} domain has {{ $value }}% error rate"

      # NATS Disconnection
      - alert: NATSDisconnected
        expr: cim_nats_connections_active == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "NATS connection lost"
          description: "CIM has no active NATS connections"

      # High Memory Usage
      - alert: HighMemoryUsage
        expr: cim_memory_usage_bytes / cim_memory_limit_bytes > 0.9
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "CIM is using {{ $value }}% of available memory"

      # Projection Lag
      - alert: ProjectionLag
        expr: cim_projection_lag_seconds > 60
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Projection lagging"
          description: "{{ $labels.projection }} is {{ $value }}s behind"

      # Query Performance
      - alert: SlowQueries
        expr: histogram_quantile(0.99, rate(cim_queries_duration_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Slow query performance"
          description: "99th percentile query latency is {{ $value }}s"
```

### 2. Configure Alertmanager

```yaml
# /etc/alertmanager/alertmanager.yml
global:
  resolve_timeout: 5m
  slack_api_url: 'YOUR_SLACK_WEBHOOK_URL'

route:
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  receiver: 'team-notifications'
  routes:
    - match:
        severity: critical
      receiver: 'pagerduty-critical'

receivers:
  - name: 'team-notifications'
    slack_configs:
      - channel: '#cim-alerts'
        title: 'CIM Alert'
        text: '{{ range .Alerts }}{{ .Annotations.summary }}\n{{ .Annotations.description }}{{ end }}'

  - name: 'pagerduty-critical'
    pagerduty_configs:
      - service_key: 'YOUR_PAGERDUTY_KEY'
```

## Logging Configuration

### 1. Structured Logging

Configure CIM for JSON logging:

```toml
# production.toml
[logging]
format = "json"
level = "info"
output = "stdout"  # Or file path

# Include these fields in every log
[logging.fields]
service = "cim-alchemist"
environment = "production"
version = "0.4.2"
```

### 2. Log Aggregation with Loki

```yaml
# /etc/loki/loki-config.yml
auth_enabled: false

server:
  http_listen_port: 3100

ingester:
  lifecycler:
    address: 127.0.0.1
    ring:
      kvstore:
        store: inmemory
      replication_factor: 1

schema_config:
  configs:
    - from: 2020-10-24
      store: boltdb-shipper
      object_store: filesystem
      schema: v11
      index:
        prefix: index_
        period: 24h

storage_config:
  boltdb_shipper:
    active_index_directory: /loki/boltdb-shipper-active
    cache_location: /loki/boltdb-shipper-cache
    shared_store: filesystem
  filesystem:
    directory: /loki/chunks

limits_config:
  enforce_metric_name: false
  reject_old_samples: true
  reject_old_samples_max_age: 168h
```

### 3. Promtail Configuration

```yaml
# /etc/promtail/promtail-config.yml
server:
  http_listen_port: 9080
  grpc_listen_port: 0

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://localhost:3100/loki/api/v1/push

scrape_configs:
  - job_name: cim
    static_configs:
      - targets:
          - localhost
        labels:
          job: cim-alchemist
          __path__: /var/log/cim/*.log
    pipeline_stages:
      - json:
          expressions:
            level: level
            timestamp: timestamp
            domain: domain
            event_type: event_type
      - labels:
          level:
          domain:
          event_type:
      - timestamp:
          source: timestamp
          format: RFC3339
```

## Distributed Tracing

### 1. Jaeger Setup

```bash
# Run Jaeger all-in-one
docker run -d --name jaeger \
  -e COLLECTOR_ZIPKIN_HOST_PORT=:9411 \
  -p 5775:5775/udp \
  -p 6831:6831/udp \
  -p 6832:6832/udp \
  -p 5778:5778 \
  -p 16686:16686 \
  -p 14268:14268 \
  -p 14250:14250 \
  -p 9411:9411 \
  jaegertracing/all-in-one:latest
```

### 2. OpenTelemetry Configuration

```toml
# production.toml
[tracing]
enabled = true
service_name = "cim-alchemist"
endpoint = "http://localhost:14268/api/traces"
sample_rate = 0.1  # Sample 10% of requests

[tracing.tags]
environment = "production"
version = "0.4.2"
```

## Health Checks

### 1. Health Endpoints

CIM exposes these health check endpoints:

```bash
# Liveness probe - is the service running?
GET /health/live
Response: 200 OK
{
  "status": "alive",
  "timestamp": "2025-01-23T10:00:00Z"
}

# Readiness probe - is the service ready to accept traffic?
GET /health/ready
Response: 200 OK or 503 Service Unavailable
{
  "status": "ready",
  "checks": {
    "nats": "connected",
    "event_store": "ready",
    "projections": "up_to_date"
  }
}

# Detailed health check
GET /health/detailed
Response: 200 OK
{
  "status": "healthy",
  "version": "0.4.2",
  "uptime_seconds": 3600,
  "checks": {
    "nats": {
      "status": "connected",
      "connections": 3,
      "last_ping": "2025-01-23T10:00:00Z"
    },
    "event_store": {
      "status": "ready",
      "event_count": 150000,
      "last_event": "2025-01-23T09:59:55Z"
    },
    "projections": {
      "GraphSummary": {
        "status": "up_to_date",
        "lag_seconds": 0.5
      }
    }
  }
}
```

### 2. Kubernetes Configuration

```yaml
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: cim-alchemist
    image: cim/alchemist:0.4.2
    livenessProbe:
      httpGet:
        path: /health/live
        port: 8080
      initialDelaySeconds: 10
      periodSeconds: 10
    readinessProbe:
      httpGet:
        path: /health/ready
        port: 8080
      initialDelaySeconds: 20
      periodSeconds: 5
```

## Custom Metrics

### Adding Custom Metrics

```rust
use prometheus::{Counter, Histogram, Registry};

// Define metrics
lazy_static! {
    static ref CUSTOM_COUNTER: Counter = Counter::new(
        "cim_custom_operations_total",
        "Total custom operations"
    ).unwrap();
    
    static ref CUSTOM_HISTOGRAM: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "cim_custom_duration_seconds",
            "Custom operation duration"
        ).buckets(vec![0.001, 0.01, 0.1, 1.0, 10.0])
    ).unwrap();
}

// Register metrics
pub fn register_metrics(registry: &Registry) {
    registry.register(Box::new(CUSTOM_COUNTER.clone())).unwrap();
    registry.register(Box::new(CUSTOM_HISTOGRAM.clone())).unwrap();
}

// Use metrics
CUSTOM_COUNTER.inc();
CUSTOM_HISTOGRAM.observe(duration.as_secs_f64());
```

## Monitoring Best Practices

1. **Set Appropriate Retention**
   - Metrics: 15 days for high-resolution, 1 year for downsampled
   - Logs: 7 days for debug, 30 days for info/warn/error
   - Traces: 3 days for sampled traces

2. **Dashboard Organization**
   - Overview dashboard for high-level health
   - Domain-specific dashboards for detailed metrics
   - Alert dashboard showing current issues

3. **Alert Fatigue Prevention**
   - Only alert on actionable issues
   - Use appropriate thresholds and time windows
   - Group related alerts

4. **Performance Impact**
   - Keep cardinality under control (< 1M series)
   - Sample traces appropriately (1-10%)
   - Use log levels appropriately

5. **Security**
   - Secure metrics endpoints with authentication
   - Encrypt metrics in transit
   - Limit access to sensitive metrics

---

*Last updated: January 2025*
*Version: 0.4.2* 