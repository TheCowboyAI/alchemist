# CIM Monitoring Guide

## Overview

CIM uses an event-driven monitoring approach where all monitoring data is derived from the event streams. This guide covers the monitoring architecture, key metrics, dashboards, and operational procedures.

## Architecture

### Event-Based Monitoring

Unlike traditional systems, CIM does **NOT** use explicit logging. All monitoring data comes from:

1. **Domain Events**: Business events flowing through NATS
2. **System Events**: Infrastructure and performance events
3. **Derived Metrics**: Calculated from event patterns

### Components

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────┐
│   NATS Events   │────▶│  Event Monitor   │────▶│ Prometheus  │
└─────────────────┘     └──────────────────┘     └─────────────┘
                                                          │
                                                          ▼
                        ┌──────────────────┐     ┌─────────────┐
                        │   Alertmanager   │◀────│   Grafana   │
                        └──────────────────┘     └─────────────┘
```

## Key Metrics

### Event Metrics

| Metric                             | Description               | Type    | Labels                    |
| ---------------------------------- | ------------------------- | ------- | ------------------------- |
| `cim_events_total`                 | Total events processed    | Counter | domain, event_type        |
| `cim_events_failed_total`          | Failed events             | Counter | domain, event_type, error |
| `cim_event_processing_lag_seconds` | Event processing delay    | Gauge   | domain                    |
| `cim_active_event_streams`         | Active NATS subscriptions | Gauge   | -                         |
| `cim_event_queue_depth`            | Queued events waiting     | Gauge   | domain                    |
| `cim_event_queue_capacity`         | Max queue size            | Gauge   | domain                    |

### Domain Metrics

| Metric                         | Description             | Type      | Labels                 |
| ------------------------------ | ----------------------- | --------- | ---------------------- |
| `cim_domain_errors_total`      | Domain-specific errors  | Counter   | domain, error_type     |
| `cim_aggregate_count`          | Active aggregates       | Gauge     | domain, aggregate_type |
| `cim_projection_lag_seconds`   | Projection update delay | Gauge     | domain, projection     |
| `cim_command_duration_seconds` | Command processing time | Histogram | domain, command_type   |

### System Metrics

| Metric                             | Description            | Type    | Labels  |
| ---------------------------------- | ---------------------- | ------- | ------- |
| `cim_nats_connections`             | NATS connection count  | Gauge   | state   |
| `cim_nats_messages_sent_total`     | Messages published     | Counter | subject |
| `cim_nats_messages_received_total` | Messages consumed      | Counter | subject |
| `cim_nats_jetstream_storage_bytes` | JetStream storage used | Gauge   | stream  |

## Dashboards

### 1. CIM Event Overview

**Purpose**: High-level view of event flow across all domains

**Key Panels**:
- Event rate by domain (time series)
- Event processing lag (bar gauge)
- Event type distribution (pie chart)
- Domain errors (time series)
- Active event streams (stat)
- Total events processed (stat)
- Event success rate (stat)
- Current event rate (stat)

**When to Use**: Daily operations, health checks

### 2. Domain-Specific Dashboards

Each domain has its own dashboard showing:
- Domain event patterns
- Aggregate states
- Command/query performance
- Business metrics

**Example - Graph Domain**:
- Nodes/edges created per minute
- Graph operation latency
- Conceptual space calculations
- Layout engine performance

### 3. Infrastructure Dashboard

**Panels**:
- CPU/Memory usage by service
- Disk I/O and space
- Network traffic
- GPU utilization (if applicable)
- NATS cluster health

## Alert Configuration

### Alert Severity Levels

1. **Critical**: Immediate action required
   - System down
   - Data loss risk
   - Security breach

2. **Warning**: Investigation needed
   - Performance degradation
   - Approaching limits
   - Unusual patterns

3. **Info**: Awareness only
   - Scheduled maintenance
   - Non-critical changes

### Key Alerts

#### Event Processing Alerts

```yaml
- alert: HighEventProcessingLag
  expr: cim_event_processing_lag_seconds > 10
  severity: warning
  action: Check consumer performance, scale if needed

- alert: EventStreamStalled  
  expr: rate(cim_events_total[5m]) == 0
  severity: critical
  action: Check NATS connectivity, restart services
```

#### Resource Alerts

```yaml
- alert: HighMemoryUsage
  expr: process_resident_memory_bytes / node_memory_MemTotal_bytes > 0.8
  severity: warning
  action: Investigate memory leaks, consider scaling

- alert: DiskSpaceLow
  expr: node_filesystem_free_bytes / node_filesystem_size_bytes < 0.1  
  severity: critical
  action: Clean up old data, expand storage
```

## Operational Procedures

### Daily Monitoring Tasks

1. **Morning Check** (5 minutes)
   ```bash
   # Check overnight alerts
   curl -s http://localhost:9093/api/v1/alerts | jq '.data[] | select(.state=="active")'
   
   # Review event rates
   open http://localhost:3000/d/cim-events
   
   # Check domain health
   nix run .#check-domain-health
   ```

2. **Trend Analysis** (15 minutes weekly)
   - Review weekly event patterns
   - Identify performance trends
   - Plan capacity adjustments

### Investigating Issues

#### High Event Lag

1. **Identify affected domain**:
   ```bash
   curl -s http://localhost:9091/metrics | grep cim_event_processing_lag
   ```

2. **Check consumer status**:
   ```bash
   nats consumer info EVENTS <DOMAIN>_CONSUMER
   ```

3. **Review event patterns**:
   ```bash
   nats stream view EVENTS --subject "cim.<DOMAIN>.>"
   ```

4. **Scale consumers if needed**:
   ```bash
   kubectl scale deployment cim-<DOMAIN> --replicas=5
   ```

#### Memory Issues

1. **Identify service**:
   ```bash
   ps aux | sort -nrk 4 | head -10
   ```

2. **Check for leaks**:
   ```bash
   nix run .#profile-service -- --service <SERVICE> --duration 300
   ```

3. **Review recent changes**:
   ```bash
   git log --since="2 days ago" -- src/
   ```

### Performance Tuning

#### Event Processing Optimization

1. **Batch Size Tuning**:
   ```yaml
   # Adjust in service config
   event_processing:
     batch_size: 1000  # Increase for throughput
     batch_timeout: 10ms  # Decrease for latency
   ```

2. **Parallelism**:
   ```yaml
   # Scale based on CPU cores
   workers: ${CORES * 2}
   prefetch: ${batch_size * 2}
   ```

3. **Memory Buffers**:
   ```yaml
   # Balance memory vs disk I/O
   buffer_size: 100000
   gc_interval: 50000
   ```

## Monitoring Best Practices

### 1. Use Event Patterns

- Monitor business outcomes, not just technical metrics
- Track event flows end-to-end
- Alert on missing expected events

### 2. Set Meaningful Thresholds

- Base thresholds on historical data
- Consider business impact
- Avoid alert fatigue

### 3. Automate Responses

- Use runbooks for common issues
- Implement auto-scaling
- Create self-healing systems

### 4. Regular Reviews

- Weekly trend analysis
- Monthly threshold adjustments
- Quarterly architecture review

## Troubleshooting

### No Metrics Available

1. Check event monitor service:
   ```bash
   systemctl status cim-event-monitor
   journalctl -u cim-event-monitor -n 100
   ```

2. Verify NATS connectivity:
   ```bash
   nats server check connection
   ```

3. Test metric endpoint:
   ```bash
   curl -s http://localhost:9091/metrics | head -20
   ```

### Missing Events

1. Check NATS subjects:
   ```bash
   nats stream subjects EVENTS
   ```

2. Verify publishers:
   ```bash
   nats events --subject "cim.>" --count 10
   ```

3. Review consumer status:
   ```bash
   nats consumer report EVENTS
   ```

### Dashboard Not Loading

1. Check Grafana service:
   ```bash
   systemctl status grafana
   ```

2. Verify datasource:
   ```bash
   curl -s http://localhost:9090/api/v1/query?query=up
   ```

3. Review dashboard JSON:
   ```bash
   ls -la /nix/dashboards/
   ```

## Integration with Other Tools

### Slack Notifications

```yaml
# In alertmanager config
receivers:
  - name: slack
    slack_configs:
      - api_url: ${SLACK_WEBHOOK_URL}
        channel: '#cim-alerts'
        title: 'CIM Alert'
```

### PagerDuty Integration

```yaml
receivers:
  - name: pagerduty
    pagerduty_configs:
      - service_key: ${PAGERDUTY_SERVICE_KEY}
        severity_map:
          critical: error
          warning: warning
```

### Custom Webhooks

```yaml
receivers:
  - name: custom
    webhook_configs:
      - url: 'http://internal-api/alerts'
        send_resolved: true
```

## Security Considerations

1. **Metric Access**: Restrict Prometheus/Grafana access
2. **Alert Channels**: Encrypt webhook communications
3. **Sensitive Data**: Never include PII in metrics
4. **Audit Trail**: Monitor metric query patterns

## Capacity Planning

### Metric Storage

- Retention: 15 days local, 1 year remote
- Growth rate: ~10GB/month per domain
- Compression: 10:1 typical ratio

### Resource Requirements

- Event Monitor: 2 CPU, 4GB RAM
- Prometheus: 4 CPU, 16GB RAM
- Grafana: 2 CPU, 2GB RAM

### Scaling Guidelines

- Add Prometheus replicas at 1M active series
- Shard by domain at 10M events/day
- Use remote storage for long-term retention 