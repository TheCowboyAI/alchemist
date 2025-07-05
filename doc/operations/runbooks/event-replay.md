# Event Replay Runbook

## Overview

This runbook provides procedures for replaying events in the CIM system for recovery, testing, or analysis purposes.

## When to Use Event Replay

### Scenarios Requiring Event Replay

1. **Data Corruption Recovery**
   - Projection state corrupted
   - Aggregate state inconsistent
   - Missing events detected

2. **System Migration**
   - Moving to new infrastructure
   - Upgrading event schema
   - Changing storage backend

3. **Testing and Analysis**
   - Load testing with production data
   - Debugging complex event flows
   - Performance analysis

4. **Disaster Recovery**
   - Complete system failure
   - Restoring from backup
   - Point-in-time recovery

## Prerequisites

- Access to NATS JetStream
- CIM event replay tools installed
- Sufficient storage for replay state
- Network connectivity to all domains

## Event Replay Procedures

### 1. Single Aggregate Replay

**Use Case**: Rebuild state for a specific aggregate

```bash
# Stop the affected service
systemctl stop cim-graph

# Backup current state
nix run .#backup-aggregate -- \
  --aggregate-id <AGGREGATE_ID> \
  --domain graph \
  --output /backup/graph-aggregate-$(date +%Y%m%d-%H%M%S).tar.gz

# Clear projection state
nats kv del CIM_PROJECTIONS graph.<AGGREGATE_ID>

# Replay events
nix run .#event-replay -- \
  --domain graph \
  --aggregate-id <AGGREGATE_ID> \
  --from-beginning \
  --target projection

# Verify state
nix run .#verify-aggregate -- \
  --aggregate-id <AGGREGATE_ID> \
  --domain graph

# Restart service
systemctl start cim-graph
```

### 2. Domain-Wide Replay

**Use Case**: Rebuild all projections for a domain

```bash
# Scale down domain services
kubectl scale deployment cim-<DOMAIN> --replicas=0

# Create snapshot point
SNAPSHOT_TIME=$(date -u +%Y-%m-%dT%H:%M:%SZ)
echo "Snapshot time: $SNAPSHOT_TIME"

# Clear all projections for domain
nats kv purge CIM_PROJECTIONS --filter "<DOMAIN>.*"

# Start replay job
nix run .#domain-replay -- \
  --domain <DOMAIN> \
  --parallel 4 \
  --batch-size 1000 \
  --checkpoint-interval 10000

# Monitor progress
watch -n 5 'nix run .#replay-status -- --domain <DOMAIN>'

# Verify projections
nix run .#verify-domain -- --domain <DOMAIN>

# Scale up services
kubectl scale deployment cim-<DOMAIN> --replicas=3
```

### 3. Point-in-Time Recovery

**Use Case**: Restore system state to specific timestamp

```bash
# Determine target timestamp
TARGET_TIME="2024-01-15T14:30:00Z"

# Stop all services
nix run .#stop-all-services

# Create recovery checkpoint
nix run .#create-checkpoint -- \
  --name "pre-recovery-$(date +%Y%m%d-%H%M%S)"

# Replay to target time
nix run .#point-in-time-replay -- \
  --target-time "$TARGET_TIME" \
  --domains all \
  --verify-checksums

# Validate state
nix run .#validate-system-state -- \
  --expected-time "$TARGET_TIME"

# Resume services
nix run .#start-all-services
```

### 4. Selective Event Replay

**Use Case**: Replay specific event types or patterns

```bash
# Define replay filter
cat > replay-filter.json <<EOF
{
  "event_types": ["NodeAdded", "EdgeCreated"],
  "domains": ["graph", "workflow"],
  "time_range": {
    "start": "2024-01-15T00:00:00Z",
    "end": "2024-01-15T23:59:59Z"
  }
}
EOF

# Execute selective replay
nix run .#selective-replay -- \
  --filter replay-filter.json \
  --target test-projection \
  --dry-run

# Review dry run results
less replay-dry-run.log

# Execute actual replay
nix run .#selective-replay -- \
  --filter replay-filter.json \
  --target test-projection \
  --execute
```

## Monitoring Event Replay

### Key Metrics to Watch

```bash
# Event replay rate
curl -s http://localhost:9091/metrics | grep cim_replay_events_per_second

# Replay lag
curl -s http://localhost:9091/metrics | grep cim_replay_lag_seconds

# Error rate
curl -s http://localhost:9091/metrics | grep cim_replay_errors_total

# Memory usage
curl -s http://localhost:9091/metrics | grep cim_replay_memory_bytes
```

### Progress Tracking

```bash
# Check replay progress
nix run .#replay-progress -- --format json | jq '.'

# Example output:
{
  "domain": "graph",
  "total_events": 1500000,
  "processed_events": 750000,
  "progress_percent": 50,
  "current_rate": 10000,
  "estimated_completion": "2024-01-15T16:45:00Z",
  "errors": 0
}
```

## Performance Optimization

### 1. Parallel Replay

For large event streams, use parallel processing:

```bash
# Calculate optimal parallelism
CORES=$(nproc)
PARALLEL=$((CORES * 2))

# Run parallel replay
nix run .#event-replay -- \
  --parallel $PARALLEL \
  --batch-size 5000 \
  --prefetch 10000
```

### 2. Memory Management

Prevent OOM during replay:

```bash
# Set memory limits
export REPLAY_MAX_MEMORY="8G"
export REPLAY_BUFFER_SIZE="100000"

# Run with memory constraints
nix run .#event-replay -- \
  --memory-limit $REPLAY_MAX_MEMORY \
  --buffer-size $REPLAY_BUFFER_SIZE \
  --gc-interval 50000
```

### 3. Checkpoint Strategy

For long-running replays:

```bash
# Enable checkpointing
nix run .#event-replay -- \
  --checkpoint-dir /var/lib/cim/checkpoints \
  --checkpoint-interval 100000 \
  --resume-on-failure
```

## Troubleshooting

### Common Issues

#### 1. Replay Hanging

**Symptoms**: No progress for >5 minutes

**Resolution**:
```bash
# Check for deadlocks
nix run .#debug-replay -- --show-locks

# Force checkpoint
kill -USR1 $(pgrep event-replay)

# Resume from checkpoint
nix run .#event-replay -- --resume
```

#### 2. Out of Memory

**Symptoms**: Process killed, OOM in logs

**Resolution**:
```bash
# Reduce batch size
--batch-size 1000

# Increase GC frequency
--gc-interval 10000

# Use disk-based buffering
--buffer-mode disk
```

#### 3. Slow Replay Performance

**Symptoms**: <1000 events/sec

**Resolution**:
```bash
# Profile replay
nix run .#profile-replay -- --duration 60

# Optimize based on profile
# - Increase parallelism
# - Adjust batch size
# - Enable compression
```

## Validation Procedures

### 1. Checksum Validation

```bash
# Generate checksums for original state
nix run .#generate-checksums -- \
  --domain <DOMAIN> \
  --output original-checksums.json

# After replay, verify
nix run .#verify-checksums -- \
  --domain <DOMAIN> \
  --expected original-checksums.json
```

### 2. Event Count Validation

```bash
# Count events in stream
STREAM_COUNT=$(nats stream info EVENTS -j | jq .state.messages)

# Count events in projections
PROJECTION_COUNT=$(nix run .#count-projected-events -- --domain <DOMAIN>)

# Verify match
if [ "$STREAM_COUNT" -eq "$PROJECTION_COUNT" ]; then
  echo "✓ Event counts match"
else
  echo "✗ Event count mismatch: Stream=$STREAM_COUNT, Projection=$PROJECTION_COUNT"
fi
```

### 3. Business Logic Validation

```bash
# Run domain-specific validations
nix run .#validate-domain -- \
  --domain <DOMAIN> \
  --rules /etc/cim/validation-rules.yaml \
  --report validation-report.html
```

## Recovery Procedures

### Failed Replay Recovery

1. **Identify failure point**:
   ```bash
   tail -n 100 /var/log/cim/replay.log
   nix run .#replay-status -- --show-errors
   ```

2. **Analyze root cause**:
   - Event corruption
   - Schema mismatch
   - Resource exhaustion
   - Network issues

3. **Fix and resume**:
   ```bash
   # Fix identified issue
   # ...

   # Resume from last checkpoint
   nix run .#event-replay -- \
     --resume \
     --skip-errors 10
   ```

### Rollback Procedures

If replay causes issues:

```bash
# Stop replay
systemctl stop cim-replay

# Restore from backup
nix run .#restore-projections -- \
  --backup /backup/pre-replay-backup.tar.gz

# Verify restoration
nix run .#verify-system-state

# Resume normal operations
nix run .#start-all-services
```

## Best Practices

1. **Always backup before replay**
2. **Test replay procedures in staging**
3. **Monitor resource usage during replay**
4. **Use checksums for validation**
5. **Document replay parameters used**
6. **Keep replay logs for audit**

## Emergency Contacts

- On-call Engineer: Use PagerDuty
- Domain Expert: Check domain ownership matrix
- Infrastructure Team: #cim-infra Slack channel 