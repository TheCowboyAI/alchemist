# NATS Cluster Recovery Runbook

## Overview
This runbook describes procedures for recovering from NATS cluster failures in the CIM infrastructure.

## Prerequisites
- SSH access to all NATS cluster nodes
- Access to CIM event monitor metrics
- Understanding of NATS JetStream concepts

## Common Issues and Solutions

### 1. Single Node Failure

**Symptoms:**
- `EventStreamDisconnected` alert firing for specific node
- Reduced cluster quorum warnings
- Increased latency on remaining nodes

**Steps:**
1. **Verify node status:**
   ```bash
   ssh root@nats-node-1.cim.internal
   systemctl status nats
   ```

2. **Check logs for errors:**
   ```bash
   journalctl -u nats -n 100 --no-pager
   ```

3. **Attempt restart:**
   ```bash
   systemctl restart nats
   ```

4. **If restart fails, check disk space:**
   ```bash
   df -h /var/lib/nats
   # Clear old snapshots if needed
   find /var/lib/nats/jetstream -name "*.snap" -mtime +7 -delete
   ```

5. **Verify cluster reconnection:**
   ```bash
   nats server list
   nats server info --server nats://nats-node-1:4222
   ```

### 2. Split Brain Recovery

**Symptoms:**
- Multiple nodes claiming to be leaders
- Event processing inconsistencies
- `AbnormalEventPattern` alerts

**Steps:**
1. **Identify split clusters:**
   ```bash
   # On each node
   nats server list
   nats server report jetstream
   ```

2. **Determine authoritative cluster:**
   - Check event sequence numbers
   - Verify which partition has majority of nodes
   - Check with CIM event monitor for latest processed events

3. **Stop minority partition:**
   ```bash
   # On minority nodes
   systemctl stop nats
   ```

4. **Clear state on minority nodes:**
   ```bash
   # WARNING: This will lose data on these nodes
   rm -rf /var/lib/nats/jetstream/*
   ```

5. **Rejoin cluster:**
   ```bash
   systemctl start nats
   # Monitor logs
   journalctl -f -u nats
   ```

### 3. JetStream Storage Full

**Symptoms:**
- `nats: maximum storage exceeded` errors
- Event publishing failures
- `EventProcessingFailure` alerts

**Steps:**
1. **Check storage usage:**
   ```bash
   nats stream list -a
   nats stream info CIM-EVENTS
   ```

2. **Identify old events for cleanup:**
   ```bash
   # Show stream configuration
   nats stream info CIM-EVENTS -j | jq '.config.max_age'
   ```

3. **Adjust retention if needed:**
   ```bash
   nats stream edit CIM-EVENTS --max-age=7d
   ```

4. **Force cleanup:**
   ```bash
   nats stream purge CIM-EVENTS --keep=1000000
   ```

5. **Monitor recovery:**
   ```bash
   watch -n 5 'nats stream info CIM-EVENTS | grep -E "Messages|Storage"'
   ```

### 4. Complete Cluster Failure

**Symptoms:**
- All NATS nodes unreachable
- Complete event processing stoppage
- All monitoring alerts firing

**Steps:**
1. **Check network connectivity:**
   ```bash
   # From a leaf node
   for i in {1..3}; do
     ping -c 3 nats-cluster-$i.cim.internal
   done
   ```

2. **If network is OK, check all nodes:**
   ```bash
   # Parallel SSH to all nodes
   for i in {1..3}; do
     ssh root@nats-cluster-$i.cim.internal "systemctl status nats" &
   done
   wait
   ```

3. **Start nodes in sequence:**
   ```bash
   # Start seed node first
   ssh root@nats-cluster-1.cim.internal "systemctl start nats"
   sleep 10
   
   # Start other nodes
   for i in {2..3}; do
     ssh root@nats-cluster-$i.cim.internal "systemctl start nats"
     sleep 5
   done
   ```

4. **Verify cluster formation:**
   ```bash
   nats server list
   nats server report jetstream
   ```

5. **Check event flow recovery:**
   ```bash
   # Subscribe to test
   nats sub "cim.>" --count=10
   
   # In another terminal, publish test
   nats pub cim.test.recovery "Recovery test"
   ```

## Event Replay Procedures

### Replaying Events After Recovery

1. **Determine last processed event:**
   ```bash
   # Check CIM event monitor metrics
   curl -s http://localhost:9091/metrics | grep cim_events_total
   ```

2. **Get last known sequence from consumers:**
   ```bash
   nats consumer info CIM-EVENTS CIM-PROCESSOR
   ```

3. **Create replay consumer:**
   ```bash
   nats consumer add CIM-EVENTS REPLAY \
     --filter="cim.>" \
     --deliver=all \
     --start-seq=<LAST_KNOWN_SEQ> \
     --max-deliver=1
   ```

4. **Monitor replay progress:**
   ```bash
   watch -n 2 'nats consumer info CIM-EVENTS REPLAY | grep -E "Last Delivered|Pending"'
   ```

## Prevention Measures

1. **Regular Health Checks:**
   ```bash
   # Add to cron
   */5 * * * * /usr/local/bin/check-nats-health.sh
   ```

2. **Automated Backups:**
   ```bash
   # Backup JetStream state
   0 */6 * * * nats stream backup CIM-EVENTS /backup/nats/
   ```

3. **Capacity Monitoring:**
   - Set up alerts at 80% storage capacity
   - Monitor message rates and adjust retention
   - Plan storage expansion before hitting limits

## Escalation

If recovery procedures fail:
1. Check event monitor metrics for anomalies
2. Verify no ongoing network issues
3. Check for corrupted stream data
4. Consider restoring from backup (data loss possible)
5. Contact senior SRE if issues persist

## Post-Recovery Verification

1. **Check all consumers are active:**
   ```bash
   nats consumer report CIM-EVENTS
   ```

2. **Verify event flow:**
   ```bash
   # Monitor event rates
   curl -s http://localhost:9091/metrics | grep -E "cim_events_total|rate"
   ```

3. **Check for processing lag:**
   ```bash
   curl -s http://localhost:9091/metrics | grep cim_event_processing_lag
   ```

4. **Verify all leaf nodes connected:**
   ```bash
   nats server list | grep -E "LEAF|GPU|MAC"
   ```

## Related Documentation
- [NATS Cluster Setup Guide](../cluster-setup-guide.md)
- [Event Monitoring Design](../../design/event-monitoring.md)
- [Disaster Recovery Plan](./disaster-recovery.md) 