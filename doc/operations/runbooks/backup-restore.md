# Backup and Restore Runbook

## Overview

This runbook provides procedures for backing up and restoring the CIM system, including event streams, projections, configurations, and GPU model states.

## Backup Strategy

### What to Backup

1. **Event Streams** (Primary Data)
   - NATS JetStream data
   - Event CID chains
   - Stream metadata

2. **Projections** (Derived Data)
   - Read model states
   - Aggregate snapshots
   - Query caches

3. **Configuration**
   - NixOS configurations
   - Domain configurations
   - Agent definitions

4. **GPU Models**
   - Model weights
   - Fine-tuned models
   - Training checkpoints

5. **Operational Data**
   - Monitoring history
   - Audit logs
   - Performance baselines

### Backup Schedule

| Component     | Frequency  | Retention | Method                |
| ------------- | ---------- | --------- | --------------------- |
| Event Streams | Continuous | Forever   | JetStream replication |
| Projections   | Hourly     | 7 days    | Snapshot              |
| Configuration | On change  | 90 days   | Git + backup          |
| GPU Models    | Daily      | 30 days   | Object store          |
| Metrics       | Daily      | 1 year    | Prometheus backup     |

## Backup Procedures

### 1. Event Stream Backup

**Continuous Replication**:

```bash
# Verify JetStream replication status
nats stream info EVENTS -j | jq '.cluster.replicas'

# Force sync to backup cluster
nats stream backup EVENTS /backup/jetstream/events-$(date +%Y%m%d)

# Verify backup integrity
nats stream validate /backup/jetstream/events-$(date +%Y%m%d)
```

**Manual Backup**:

```bash
# Create backup job
nix run .#backup-event-streams -- \
  --streams "EVENTS,COMMANDS,QUERIES" \
  --destination s3://cim-backups/events/ \
  --compress zstd \
  --encrypt

# Monitor progress
nix run .#backup-status -- --job-id <JOB_ID>
```

### 2. Projection Backup

**Automated Hourly Snapshots**:

```bash
# Check snapshot schedule
systemctl status cim-projection-backup.timer

# Trigger manual snapshot
nix run .#snapshot-projections -- \
  --domains all \
  --output /backup/projections/$(date +%Y%m%d-%H%M%S)
```

**Selective Backup**:

```bash
# Backup specific domain projections
nix run .#backup-domain-projections -- \
  --domain graph \
  --include "GraphReadModel,NodeIndex,EdgeCache" \
  --format parquet
```

### 3. Configuration Backup

**NixOS Configuration**:

```bash
# Backup system configuration
sudo nix run .#backup-nixos-config -- \
  --include-secrets \
  --output /backup/nixos/$(hostname)-$(date +%Y%m%d).tar.gz

# Backup flake inputs
cp flake.lock /backup/nixos/flake.lock-$(date +%Y%m%d)
```

**Domain Configuration**:

```bash
# Export all domain configs
nix run .#export-domain-configs -- \
  --format yaml \
  --output /backup/configs/domains-$(date +%Y%m%d).yaml

# Backup to git
git add -A
git commit -m "Backup: Domain configurations $(date)"
git push backup main
```

### 4. GPU Model Backup

**Model Weights**:

```bash
# List models requiring backup
nix run .#list-gpu-models -- --modified-since 24h

# Backup models to object store
nix run .#backup-gpu-models -- \
  --source /var/lib/cim/models \
  --destination s3://cim-models/backup/$(date +%Y%m%d)/ \
  --parallel 4

# Verify model integrity
nix run .#verify-model-backup -- \
  --backup-path s3://cim-models/backup/$(date +%Y%m%d)/
```

**Training Checkpoints**:

```bash
# Backup active training checkpoints
find /var/lib/cim/checkpoints -name "*.ckpt" -mtime -1 | \
  xargs -I {} nix run .#backup-checkpoint -- --file {} --compress
```

### 5. Monitoring Data Backup

**Prometheus Metrics**:

```bash
# Create Prometheus snapshot
curl -X POST http://localhost:9090/api/v1/admin/tsdb/snapshot

# Copy snapshot to backup
SNAPSHOT=$(ls -t /var/lib/prometheus/snapshots | head -1)
cp -r /var/lib/prometheus/snapshots/$SNAPSHOT \
  /backup/prometheus/$(date +%Y%m%d)

# Backup Grafana dashboards
nix run .#backup-grafana -- \
  --include-datasources \
  --output /backup/grafana/$(date +%Y%m%d).tar.gz
```

## Restore Procedures

### 1. Event Stream Restore

**Full Restore from Backup**:

```bash
# Stop all services
nix run .#stop-all-services

# Restore JetStream data
nats stream restore EVENTS /backup/jetstream/events-20240115

# Verify stream integrity
nats stream info EVENTS

# Replay events to rebuild projections
nix run .#replay-all-events -- --parallel 8
```

**Partial Restore (Time Range)**:

```bash
# Restore events from specific time range
nix run .#restore-events -- \
  --stream EVENTS \
  --start "2024-01-15T00:00:00Z" \
  --end "2024-01-15T12:00:00Z" \
  --source /backup/jetstream/events-20240115
```

### 2. Projection Restore

**Quick Restore from Snapshot**:

```bash
# List available snapshots
nix run .#list-projection-snapshots -- --domain graph

# Restore specific snapshot
nix run .#restore-projection-snapshot -- \
  --snapshot /backup/projections/20240115-120000 \
  --domain graph \
  --verify

# Catch up with recent events
nix run .#replay-events -- \
  --domain graph \
  --from-timestamp "2024-01-15T12:00:00Z"
```

**Rebuild from Events**:

```bash
# Clear existing projections
nix run .#clear-projections -- --domain graph --confirm

# Rebuild from event stream
nix run .#rebuild-projections -- \
  --domain graph \
  --from-beginning \
  --batch-size 10000 \
  --checkpoint-interval 100000
```

### 3. Configuration Restore

**System Configuration**:

```bash
# Extract backup
tar -xzf /backup/nixos/node1-20240115.tar.gz -C /tmp/restore

# Compare with current
diff -r /etc/nixos /tmp/restore/etc/nixos

# Restore configuration
sudo cp -r /tmp/restore/etc/nixos/* /etc/nixos/

# Rebuild system
sudo nixos-rebuild switch
```

**Domain Configuration**:

```bash
# Import domain configs
nix run .#import-domain-configs -- \
  --source /backup/configs/domains-20240115.yaml \
  --validate \
  --dry-run

# Apply if validation passes
nix run .#import-domain-configs -- \
  --source /backup/configs/domains-20240115.yaml \
  --apply
```

### 4. GPU Model Restore

**Model Recovery**:

```bash
# List backed up models
aws s3 ls s3://cim-models/backup/20240115/

# Restore specific model
nix run .#restore-gpu-model -- \
  --model-id "llama3-fine-tuned-v2" \
  --source s3://cim-models/backup/20240115/ \
  --destination /var/lib/cim/models/

# Verify model integrity
nix run .#verify-model -- \
  --path /var/lib/cim/models/llama3-fine-tuned-v2 \
  --checksum sha256:abc123...
```

**Checkpoint Recovery**:

```bash
# Restore training checkpoint
nix run .#restore-checkpoint -- \
  --checkpoint-id "training-20240115-epoch-42" \
  --resume-training
```

### 5. Monitoring Data Restore

**Prometheus Restore**:

```bash
# Stop Prometheus
systemctl stop prometheus

# Restore data directory
rm -rf /var/lib/prometheus/data
cp -r /backup/prometheus/20240115 /var/lib/prometheus/data

# Start Prometheus
systemctl start prometheus

# Verify data availability
curl -s http://localhost:9090/api/v1/query?query=up
```

## Disaster Recovery Scenarios

### Scenario 1: Complete System Failure

**Recovery Steps**:

1. **Provision new infrastructure**:
   ```bash
   nix run .#provision-cluster -- \
     --config /backup/infrastructure/cluster-config.yaml
   ```

2. **Restore system configurations**:
   ```bash
   nix run .#restore-all-configs -- \
     --source /backup/nixos/latest
   ```

3. **Restore event streams**:
   ```bash
   nix run .#restore-all-streams -- \
     --source s3://cim-backups/events/latest
   ```

4. **Rebuild projections**:
   ```bash
   nix run .#rebuild-all-projections -- --parallel
   ```

5. **Restore models**:
   ```bash
   nix run .#restore-all-models -- \
     --source s3://cim-models/backup/latest
   ```

6. **Validate system**:
   ```bash
   nix run .#validate-full-restore
   ```

### Scenario 2: Data Corruption

**Detection**:

```bash
# Run integrity checks
nix run .#check-data-integrity -- --deep

# Sample output:
# Event Stream: CORRUPTED - CID chain broken at seq 1234567
# Projections: INCONSISTENT - 42 aggregates affected
```

**Recovery**:

1. **Identify corruption point**:
   ```bash
   nix run .#find-corruption-point -- \
     --stream EVENTS \
     --start-seq 1234000
   ```

2. **Restore from last good backup**:
   ```bash
   nix run .#restore-from-point -- \
     --stream EVENTS \
     --sequence 1234000 \
     --source /backup/jetstream/events-20240114
   ```

3. **Replay affected events**:
   ```bash
   nix run .#replay-corrupted-events -- \
     --from-seq 1234000 \
     --validate-cids
   ```

### Scenario 3: Accidental Deletion

**Immediate Response**:

```bash
# Stop writes to prevent further damage
nix run .#emergency-read-only-mode

# Identify what was deleted
nix run .#audit-log -- --last 1h | grep DELETE

# Restore deleted data
nix run .#restore-deleted -- \
  --type projection \
  --id <DELETED_ID> \
  --timestamp "2024-01-15T14:30:00Z"
```

## Backup Validation

### Daily Validation

```bash
# Automated validation job
systemctl status cim-backup-validation.timer

# Manual validation
nix run .#validate-backups -- \
  --check-integrity \
  --verify-restore \
  --report /tmp/backup-validation-$(date +%Y%m%d).html
```

### Restore Testing

**Monthly Restore Drill**:

```bash
# Use test environment
export CIM_ENV=test

# Perform full restore
nix run .#disaster-recovery-drill -- \
  --scenario complete-failure \
  --backup-date $(date -d "yesterday" +%Y%m%d)

# Validate functionality
nix run .#run-integration-tests -- --environment test

# Generate report
nix run .#dr-drill-report -- \
  --output /reports/dr-drill-$(date +%Y%m).pdf
```

## Best Practices

1. **3-2-1 Rule**:
   - 3 copies of data
   - 2 different storage types
   - 1 offsite backup

2. **Test Restores Regularly**:
   - Monthly restore drills
   - Quarterly full DR tests
   - Document restore times

3. **Monitor Backup Health**:
   - Alert on backup failures
   - Track backup sizes
   - Verify backup integrity

4. **Document Everything**:
   - Backup procedures
   - Restore steps
   - Contact information
   - Recovery time objectives

## Recovery Time Objectives

| Component     | RTO     | RPO            | Notes                 |
| ------------- | ------- | -------------- | --------------------- |
| Event Streams | 1 hour  | 0 (continuous) | JetStream replication |
| Projections   | 2 hours | 1 hour         | From snapshots        |
| GPU Models    | 4 hours | 24 hours       | From S3               |
| Full System   | 8 hours | 1 hour         | Complete DR           |

## Contacts

- **Backup Team**: #cim-backup-team
- **On-Call**: See PagerDuty
- **Storage Admin**: storage@cim.io
- **DR Coordinator**: dr-team@cim.io 