# GPU Node Failure Runbook

## Overview

This runbook provides procedures for handling GPU node failures in the CIM cluster, covering both NVIDIA and Apple Silicon GPU nodes.

## GPU Node Types

### NVIDIA GPU Nodes
- **Hardware**: RTX 3080 Ti, RTX 4090, A100
- **Driver**: NVIDIA proprietary driver
- **CUDA**: Version 12.x
- **Monitoring**: nvidia-smi, dcgm-exporter

### Apple Silicon Nodes
- **Hardware**: Mac Studio M3 Ultra
- **Framework**: Metal Performance Shaders
- **Memory**: Unified 192GB/256GB
- **Monitoring**: CIM Metal metrics exporter

## Failure Scenarios

### 1. GPU Driver Crash

**Symptoms**:
- Agents report GPU allocation failures
- nvidia-smi returns error or hangs
- GPU metrics stop reporting
- Kernel errors in dmesg

**Immediate Actions**:

```bash
# Check GPU status
nvidia-smi || echo "NVIDIA driver not responding"

# Check kernel messages
dmesg | grep -E "NVRM|nvidia" | tail -20

# For Apple Silicon
system_profiler SPDisplaysDataType | grep "Metal"
```

**Recovery Procedure**:

1. **Evacuate workloads**:
   ```bash
   # Mark node as unschedulable
   nix run .#mark-node-maintenance -- --node <NODE_NAME>
   
   # Migrate agents to other GPUs
   nix run .#migrate-gpu-agents -- \
     --from-node <NODE_NAME> \
     --strategy redistribute
   ```

2. **Attempt soft recovery**:
   ```bash
   # NVIDIA: Reload driver
   sudo modprobe -r nvidia_uvm nvidia_drm nvidia_modeset nvidia
   sudo modprobe nvidia nvidia_modeset nvidia_drm nvidia_uvm
   
   # Restart GPU monitoring
   sudo systemctl restart nvidia-gpu-prometheus-exporter
   ```

3. **Hard recovery if needed**:
   ```bash
   # Schedule node reboot
   sudo shutdown -r +5 "GPU driver recovery - rebooting in 5 minutes"
   ```

### 2. GPU Memory Errors

**Symptoms**:
- ECC errors in nvidia-smi
- Agent crashes with CUDA out of memory
- Corrupted model outputs
- Memory allocation failures

**Diagnosis**:

```bash
# Check ECC errors (NVIDIA)
nvidia-smi --query-gpu=ecc.errors.corrected.volatile.total,ecc.errors.uncorrected.volatile.total --format=csv

# Check memory integrity
nvidia-smi --query-gpu=memory.total,memory.used,memory.free --format=csv

# Run memory test
nix run .#gpu-memory-test -- --device 0 --duration 300
```

**Recovery Actions**:

1. **For correctable errors**:
   ```bash
   # Clear error counters
   sudo nvidia-smi -r
   
   # Monitor error rate
   watch -n 5 'nvidia-smi -q -d ECC | grep -A 4 "ECC Errors"'
   ```

2. **For uncorrectable errors**:
   ```bash
   # Immediately evacuate node
   nix run .#emergency-gpu-evacuate -- --node <NODE_NAME>
   
   # Disable GPU
   sudo nvidia-smi -i <GPU_ID> -pm 0
   
   # Schedule hardware replacement
   nix run .#create-hardware-ticket -- \
     --node <NODE_NAME> \
     --issue "GPU memory errors" \
     --priority critical
   ```

### 3. Thermal Throttling

**Symptoms**:
- GPU temperature > 85°C
- Performance degradation
- Thermal throttle events in logs
- Fan speed at maximum

**Immediate Response**:

```bash
# Check temperatures
nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader,nounits

# Check power and thermal state
nvidia-smi -q -d POWER,TEMPERATURE

# For Apple Silicon
sudo powermetrics --samplers gpu_power -i 1 -n 10
```

**Mitigation Steps**:

1. **Reduce load**:
   ```bash
   # Lower power limit
   sudo nvidia-smi -pl 250  # Reduce to 250W
   
   # Reduce agent allocation
   nix run .#reduce-gpu-allocation -- \
     --node <NODE_NAME> \
     --max-agents 2
   ```

2. **Check cooling**:
   ```bash
   # Verify fan operation
   nvidia-smi --query-gpu=fan.speed --format=csv
   
   # Check ambient temperature
   sensors | grep -E "Core|Package"
   ```

3. **Physical inspection**:
   - Check for dust buildup
   - Verify airflow paths
   - Check thermal paste (if warranted)

### 4. GPU Not Detected

**Symptoms**:
- nvidia-smi shows no devices
- PCIe device missing
- Boot errors about GPU
- Metal devices not enumerated (Apple)

**Diagnostic Steps**:

```bash
# Check PCIe devices
lspci | grep -E "NVIDIA|VGA"

# Check kernel modules
lsmod | grep nvidia

# Check device files
ls -la /dev/nvidia*

# For Apple Silicon
ioreg -l | grep -i gpu
```

**Recovery Procedure**:

1. **Reseat GPU** (if physical access):
   ```bash
   # Prepare for shutdown
   nix run .#prepare-node-shutdown -- --node <NODE_NAME>
   
   # Power down
   sudo shutdown -h now
   ```
   
   Physical steps:
   - Power off completely
   - Reseat GPU card
   - Check power connectors
   - Clear CMOS if needed

2. **BIOS/UEFI checks**:
   - Enable Above 4G Decoding
   - Set PCIe to Gen3/Gen4
   - Disable CSM if using UEFI

### 5. Agent GPU Allocation Failures

**Symptoms**:
- Events: `AgentGPUAllocationFailed`
- Agents stuck in pending state
- GPU appears available but unusable

**Investigation**:

```bash
# Check GPU allocation status
nix run .#gpu-allocation-status -- --verbose

# Check CUDA context limits
nvidia-smi --query-gpu=count --format=csv

# List GPU processes
nvidia-smi pmon -c 1

# Check cgroup limits
cat /sys/fs/cgroup/devices/devices.list | grep nvidia
```

**Resolution Steps**:

1. **Clear stuck allocations**:
   ```bash
   # Find orphaned processes
   nvidia-smi --query-compute-apps=pid,name --format=csv,noheader | \
     while read pid name; do
       if ! ps -p $pid > /dev/null; then
         echo "Orphaned GPU process: $pid ($name)"
         sudo kill -9 $pid 2>/dev/null || true
       fi
     done
   
   # Reset GPU state
   sudo nvidia-smi --gpu-reset
   ```

2. **Verify agent configuration**:
   ```bash
   # Check agent GPU requirements
   nix run .#check-agent-config -- --agent-type <TYPE>
   
   # Validate GPU capabilities
   nix run .#validate-gpu-capabilities -- \
     --required-memory 24GB \
     --required-compute 8.6
   ```

## Monitoring and Alerts

### Key Metrics to Watch

| Metric          | Warning  | Critical  | Action               |
| --------------- | -------- | --------- | -------------------- |
| GPU Temperature | >80°C    | >85°C     | Reduce load          |
| GPU Memory Used | >90%     | >95%      | Migrate agents       |
| Power Draw      | >90% TDP | >95% TDP  | Lower power limit    |
| ECC Errors      | >10/hour | >100/hour | Schedule replacement |
| Fan Speed       | >90%     | 100%      | Check cooling        |

### Alert Response Times

- **Critical**: Respond within 15 minutes
- **Warning**: Respond within 1 hour
- **Info**: Review daily

## Preventive Maintenance

### Daily Checks

```bash
# Run GPU health check
nix run .#daily-gpu-health-check

# Sample output:
# GPU 0: HEALTHY - Temp: 65°C, Memory: 18/24GB, Power: 250/350W
# GPU 1: WARNING - Temp: 78°C, Memory: 23/24GB, Power: 340/350W
```

### Weekly Maintenance

1. **Clean error counters**:
   ```bash
   sudo nvidia-smi -r
   ```

2. **Update drivers** (if available):
   ```bash
   nix run .#check-gpu-driver-updates
   ```

3. **Performance validation**:
   ```bash
   nix run .#gpu-benchmark -- --quick
   ```

### Monthly Tasks

1. **Thermal paste check** (if temperatures trending up)
2. **Firmware updates** 
3. **Power supply verification**
4. **Cable inspection**

## Emergency Procedures

### Complete GPU Failure

1. **Immediate actions**:
   ```bash
   # Emergency evacuation
   nix run .#emergency-gpu-evacuate -- \
     --node <NODE_NAME> \
     --reason "GPU failure"
   
   # Notify on-call
   nix run .#page-oncall -- \
     --severity critical \
     --message "GPU node <NODE_NAME> failed"
   ```

2. **Isolate node**:
   ```bash
   # Remove from cluster
   nix run .#remove-node -- --node <NODE_NAME>
   
   # Document failure
   nix run .#document-hardware-failure -- \
     --node <NODE_NAME> \
     --component GPU \
     --symptoms "Complete failure"
   ```

### Multi-GPU Failure

If multiple GPUs fail simultaneously:

1. **Check common causes**:
   - Power supply issues
   - Cooling system failure
   - Driver corruption
   - Firmware bug

2. **Escalate immediately**:
   ```bash
   nix run .#declare-gpu-emergency
   ```

3. **Activate DR plan**:
   - Switch to CPU-only agents
   - Reduce service capacity
   - Notify customers of degradation

## Recovery Validation

After any GPU recovery:

```bash
# Run validation suite
nix run .#validate-gpu-recovery -- --node <NODE_NAME>

# Tests performed:
# - GPU detection
# - Memory allocation
# - Compute verification
# - Thermal stability
# - Agent allocation
# - Monitoring restoration
```

## Escalation Matrix

| Severity  | Contact           | Response Time     |
| --------- | ----------------- | ----------------- |
| Info      | Team Slack        | Next business day |
| Warning   | On-call Secondary | 1 hour            |
| Critical  | On-call Primary   | 15 minutes        |
| Emergency | All Hands         | Immediate         |

## Related Documentation

- [Hardware Specifications](../hardware-specs.md)
- [Agent GPU Requirements](../agent-gpu-requirements.md)
- [Monitoring Guide](../monitoring-guide.md)
- [Disaster Recovery Plan](../disaster-recovery.md) 