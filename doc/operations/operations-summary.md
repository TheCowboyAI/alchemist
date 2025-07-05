# CIM Operations Summary

## Overview

This document summarizes the operational infrastructure for the Composable Information Machine (CIM), including monitoring, alerting, runbooks, and operational procedures.

## Monitoring Architecture

### Event-Driven Monitoring

CIM uses a unique **event-driven monitoring** approach where ALL monitoring data is derived from event streams. There is NO traditional logging infrastructure.

**Key Components**:
- **Event Monitor Service** (`src/monitoring/event_monitor.rs`) - Converts NATS events to Prometheus metrics
- **Prometheus** - Metrics collection and storage
- **Grafana** - Visualization dashboards
- **Alertmanager** - Alert routing and notification

### Metrics Collection

All metrics are derived from events:

```
NATS Events → Event Monitor → Prometheus Metrics → Grafana/Alerts
```

**Key Metrics**:
- Event rates by domain
- Processing lag
- Error counts
- GPU utilization
- NATS cluster health
- System resources

## Dashboards

### 1. CIM Event Monitoring (`cim-events.json`)
- Event flow visualization across all domains
- Processing lag indicators
- Error tracking
- Success rates
- Real-time event statistics

### 2. GPU Monitoring (`cim-gpu.json`)
- Supports both NVIDIA and Apple Silicon
- GPU utilization and memory usage
- Temperature monitoring
- Power consumption
- Allocation failures

### 3. NATS Cluster Health (`cim-nats.json`)
- Cluster size and node status
- Message rates and data transfer
- JetStream storage usage
- Consumer lag tracking
- Connection metrics

### 4. Domain-Specific Dashboards (Planned)
- Per-domain event patterns
- Business metrics
- Performance analytics

## Alert Configuration

### Alert Categories

1. **Event Processing Alerts**
   - High processing lag (>10s warning, >60s critical)
   - Event stream stalled
   - High error rates
   - Queue backpressure

2. **Infrastructure Alerts**
   - NATS connection loss
   - JetStream storage full
   - High CPU/memory usage
   - GPU failures

3. **Business Alerts**
   - Workflow stalls
   - High failure rates
   - Data quality issues

### Alert Routing

```yaml
Critical → On-call Primary (15 min response)
Warning → On-call Secondary (1 hour response)
Info → Team Slack (next business day)
```

## Operational Runbooks

### 1. NATS Recovery (`nats-recovery.md`)
**Covers**:
- Single node failures
- Split brain resolution
- JetStream storage issues
- Complete cluster recovery
- Connection troubleshooting

**Key Procedures**:
- Node restart sequences
- Data consistency checks
- Client reconnection
- Performance optimization

### 2. Event Replay (`event-replay.md`)
**Covers**:
- Single aggregate replay
- Domain-wide replay
- Point-in-time recovery
- Selective event replay

**Use Cases**:
- Data corruption recovery
- System migration
- Testing with production data
- Debugging complex flows

### 3. GPU Failure Handling (`gpu-failure.md`)
**Covers**:
- Driver crashes
- Memory errors
- Thermal issues
- GPU not detected
- Allocation failures

**Supports**:
- NVIDIA GPUs (RTX 3080 Ti+)
- Apple Silicon (M3 Ultra)
- Automated workload migration
- Performance degradation handling

### 4. Backup and Restore (`backup-restore.md`)
**Covers**:
- Event stream backups
- Projection snapshots
- Configuration backups
- GPU model backups
- Disaster recovery

**Schedules**:
- Continuous event replication
- Hourly projection snapshots
- Daily model backups
- On-change config backups

## Operational Procedures

### Daily Operations

1. **Morning Health Check** (5 minutes)
   ```bash
   # Check alerts
   curl -s http://localhost:9093/api/v1/alerts | jq '.data[] | select(.state=="active")'
   
   # Review dashboards
   open http://localhost:3000/d/cim-events
   
   # Domain health
   nix run .#check-domain-health
   ```

2. **Event Flow Monitoring**
   - Monitor event rates
   - Check processing lag
   - Review error patterns
   - Verify GPU utilization

### Weekly Tasks

1. **Performance Review**
   - Analyze event patterns
   - Check resource trends
   - Review alert history
   - Plan capacity adjustments

2. **Maintenance**
   - GPU health checks
   - Driver updates
   - Clean error counters
   - Backup validation

### Monthly Tasks

1. **Disaster Recovery Drill**
   - Test restore procedures
   - Validate backup integrity
   - Update runbooks
   - Team training

2. **Capacity Planning**
   - Review growth trends
   - Plan infrastructure scaling
   - Update resource allocations

## Key Operational Principles

### 1. Event-First Monitoring
- No logs, only events
- All metrics derived from event streams
- Business-aligned monitoring

### 2. Automation
- Self-healing systems
- Automated failover
- Event-driven recovery

### 3. Observability
- End-to-end event tracing
- Comprehensive dashboards
- Proactive alerting

### 4. Resilience
- No single points of failure
- Graceful degradation
- Fast recovery procedures

## Emergency Contacts

| Role              | Contact Method | Response Time |
| ----------------- | -------------- | ------------- |
| On-Call Primary   | PagerDuty      | 15 minutes    |
| On-Call Secondary | PagerDuty      | 1 hour        |
| Team Lead         | Slack/Phone    | 2 hours       |
| Infrastructure    | #cim-infra     | Best effort   |

## Quick Reference

### Common Commands

```bash
# Check system health
nix run .#health-check

# View active alerts
curl http://localhost:9093/api/v1/alerts

# GPU status
nvidia-smi  # NVIDIA
nix run .#gpu-status  # All GPUs

# NATS cluster status
nats server list

# Event replay
nix run .#replay-events -- --help

# Emergency procedures
nix run .#emergency-stop
nix run .#emergency-gpu-evacuate
nix run .#declare-incident
```

### Dashboard URLs

- Events: http://localhost:3000/d/cim-events
- GPU: http://localhost:3000/d/cim-gpu
- NATS: http://localhost:3000/d/cim-nats
- Alerts: http://localhost:9093

### Metric Endpoints

- Prometheus: http://localhost:9090
- Event Monitor: http://localhost:9091/metrics
- NATS: http://localhost:8222/metrics

## Next Steps

1. **Complete remaining dashboards**:
   - Domain-specific metrics
   - Agent performance tracking

2. **Enhance monitoring**:
   - Distributed tracing
   - SLI/SLO definitions
   - Long-term metric storage

3. **Operational improvements**:
   - Automated remediation
   - Predictive alerting
   - Chaos engineering

## Documentation

All operational documentation is maintained in `/doc/operations/`:
- `monitoring-guide.md` - Comprehensive monitoring guide
- `runbooks/` - All operational runbooks
- `alerts.yaml` - Alert rule definitions
- Dashboard JSON files in `/nix/dashboards/`

This operational infrastructure provides comprehensive observability and recovery procedures for the CIM system, ensuring high availability and rapid incident response. 